#!/usr/bin/env python3
"""
extract_solc_error_codes.py
===========================
Extracts all solc error codes from the solidity compiler source tree and writes
a structured JSON database to data/error_codes.json.

Usage
-----
Run from the root of the solidity-language-server repo:

    python3 docs/scripts/extract_solc_error_codes.py \
        --solidity-repo /path/to/argotorg/solidity \
        --output data/error_codes.json

Dependencies
------------
- ripgrep (rg) must be on PATH
- Python 3.8+

How it works
------------
1.  Calls rg with -C 6 (6 lines of context) so that multiline error reporter
    calls are fully captured in one window.
2.  Parses each rg context block to locate the reporter call and extract:
      code     – 4-digit error code (integer)
      severity – error | warning | info  (inferred from reporter call name)
      message  – the string argument passed to the reporter call, extracted
                 by finding the last string literal *after* the code token in
                 the window; handles:
                   • single-line calls  (message on same line as code)
                   • multiline calls    (message 1-4 lines after code)
                   • struct-init style  {"message", NNNN_error}  (before code)
                   • variable-assign    error = NNNN_error; message = "...";
                   • concatenated msgs  "prefix" + expr + "suffix"  → prefix
      source   – lib/file.cpp (last two path components)
      line     – line number of the match

3.  Writes one JSON object per unique code, sorted by code, with an `action`
    field (null by default) reserved for codeAction fix descriptors.

Regenerating
------------
Re-run this script whenever the solidity compiler repo is updated.  The output
file is checked into the language server repo so the server binary does not
need the compiler sources at runtime (it embeds the JSON with include_str!).
"""

import argparse
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path


# ---------------------------------------------------------------------------
# Patterns
# ---------------------------------------------------------------------------

CODE_PAT = re.compile(r"(\d{4})_error")

# Reporter call names → severity
REPORTER_PAT = re.compile(
    r"\b("
    r"fatalParserError|parserError|parserWarning|parserInfo"
    r"|typeError|typeWarning|typeInfo"
    r"|declarationError|declarationWarning|declarationInfo"
    r"|fatalDeclarationError"
    r"|syntaxError|syntaxWarning"
    r"|unimplementedFeatureError|docstringParsingError"
    r")\s*\("
    # also m_errorReporter.warning / .error / .info / .typeError / etc.
    r"|m_errorReporter\s*\.\s*("
    r"warning|error|info|fatalError"
    r"|typeError|typeWarning|typeInfo"
    r"|declarationError|declarationWarning"
    r"|syntaxError|syntaxWarning"
    r"|parserError|parserWarning"
    r"|docstringParsingError|unimplementedFeatureError"
    r")\s*\("
    # m_uniqueErrorReporter (SMTChecker)
    r"|m_uniqueErrorReporter\s*\.\s*("
    r"warning|error|info"
    r")\s*\("
)

SEV_MAP = {
    "fatalParserError": "error",
    "parserError": "error",
    "parserWarning": "warning",
    "parserInfo": "info",
    "typeError": "error",
    "typeWarning": "warning",
    "typeInfo": "info",
    "declarationError": "error",
    "fatalDeclarationError": "error",
    "declarationWarning": "warning",
    "declarationInfo": "info",
    "syntaxError": "error",
    "syntaxWarning": "warning",
    "unimplementedFeatureError": "error",
    "docstringParsingError": "error",
    "warning": "warning",
    "error": "error",
    "fatalError": "error",
    "info": "info",
}


def severity_from_window(window: str) -> str:
    m = REPORTER_PAT.search(window)
    if not m:
        return "error"
    # one of the three capture groups will be set
    call = m.group(1) or m.group(2) or m.group(3) or ""
    return SEV_MAP.get(call, "error")


def shorten_path(p: str) -> str:
    parts = p.replace("\\", "/").split("/")
    return "/".join(parts[-2:]) if len(parts) >= 2 else p


# ---------------------------------------------------------------------------
# Message extraction
# ---------------------------------------------------------------------------

# Matches a complete C-string literal on a single line (handles \" escapes inside).
# No DOTALL — we do not want cross-line matches which would corrupt extraction.
STRING_LIT = re.compile(r'"((?:[^"\\]|\\.)*)"')


def _clean(s: str) -> str:
    """Unescape C string escapes for display."""
    # rg output preserves literal backslash+quote as \" in the text stream
    return s.replace('\\"', '"').replace("\\n", " ").replace("\\t", " ").strip()


def _is_noise(s: str) -> bool:
    """Return True if s is clearly not a human-readable message."""
    if len(s) < 4:
        return True
    # single identifier-like token with no spaces → probably a variable/keyword
    if re.fullmatch(r"[A-Za-z_]\w*", s):
        return True
    return False


def extract_message(window: str, code: str) -> str:
    """
    Extract the human-readable message string for a given error code from
    the 13-line context window.

    Strategy (in order):
    1. Find the position of `NNNN_error` in the window.
    2. Look *after* the code token for string literals on the same or
       following lines (handles single-line and multiline calls).
    3. If nothing found after, look *before* the code token
       (handles struct-init style  {"message", NNNN_error}).
    4. For variable-assignment style (error = NNNN_error; message = "...";)
       the message appears after a `message =` or `msg =` on the next line.
    """
    code_pos = window.find(f"{code}_error")
    if code_pos == -1:
        return ""

    after = window[code_pos:]
    before = window[:code_pos]

    # --- style 3 (highest priority): string literal before the code on the
    #     SAME LINE — handles struct-init  {"message", NNNN_error}
    same_line_before = ""
    for line in window.splitlines():
        if f"{code}_error" in line:
            same_line_before = line[: line.index(f"{code}_error")]
            break

    if same_line_before:
        slb_candidates = [
            _clean(m.group(1))
            for m in STRING_LIT.finditer(same_line_before)
            if not _is_noise(_clean(m.group(1)))
        ]
        if slb_candidates:
            return slb_candidates[-1]  # closest to the code token

    # --- style 4: message = "..." assignment after the code ---
    m = re.search(r'\bmessage\s*=\s*"((?:[^"\\]|\\.)*)"', after)
    if m:
        s = _clean(m.group(1))
        if not _is_noise(s):
            return s

    # --- styles 1 & 2: string literals after the code token (same or next lines) ---
    candidates_after = []
    for m in STRING_LIT.finditer(after):
        s = _clean(m.group(1))
        if not _is_noise(s):
            candidates_after.append(s)

    if candidates_after:
        # Prefer first candidate that starts with a capital letter (prose)
        for s in candidates_after:
            if s and (s[0].isupper() or s[0] in ("`", "'")):
                return s
        return candidates_after[0]

    # --- fallback: any string before the code anywhere in the window ---
    candidates_before = [
        _clean(m.group(1))
        for m in STRING_LIT.finditer(before)
        if not _is_noise(_clean(m.group(1)))
    ]
    if candidates_before:
        return candidates_before[-1]

    return ""


# ---------------------------------------------------------------------------
# Parsing rg -C N output
# ---------------------------------------------------------------------------


def parse_context_output(text: str) -> list[dict]:
    """
    Parse rg -C 6 output into a list of unique error code entries.

    rg context output format:
        file:lineno:content   <- match line  (contains the code)
        file-lineno-content   <- context line
        --                    <- block separator between non-adjacent groups

    Key subtlety: when two matches are within 2*C lines of each other rg
    merges their context into a single block with no '--' separator.  A merged
    block can therefore contain multiple distinct error codes.

    Strategy: parse every line into (filepath, lineno, content, is_match).
    Then for each *match* line, build a local ±6-line window from the
    surrounding already-parsed lines and process it independently.  This
    gives each code its own isolated context regardless of merging.
    """
    seen: dict[str, dict] = {}

    # Step 1 – parse every rg output line into structured records.
    match_re = re.compile(r"^(.+?):(\d+):(.*)")
    context_re = re.compile(r"^(.+?)-(\d+)-(.*)")

    # Each record: (filepath, lineno, content, is_match_line)
    records: list[tuple[str, int, str, bool]] = []

    for raw in text.splitlines():
        if raw == "--":
            continue  # separator — we handle proximity via index arithmetic
        m = match_re.match(raw)
        if m:
            records.append((m.group(1), int(m.group(2)), m.group(3), True))
            continue
        m = context_re.match(raw)
        if m:
            records.append((m.group(1), int(m.group(2)), m.group(3), False))

    # Step 2 – for each match line, build a window of nearby records from
    # the same file and process it.
    WINDOW = 6  # lines of context on each side

    for i, (filepath, lineno, content, is_match) in enumerate(records):
        if not is_match:
            continue
        if not CODE_PAT.search(content):
            continue

        # Collect surrounding records from the same file within ±WINDOW lines.
        window_lines: list[tuple[int, str]] = []
        for j in range(max(0, i - WINDOW), min(len(records), i + WINDOW + 1)):
            fp_j, ln_j, c_j, _ = records[j]
            if fp_j == filepath and abs(ln_j - lineno) <= WINDOW:
                window_lines.append((ln_j, c_j))

        window_lines.sort(key=lambda x: x[0])
        window = "\n".join(c for _, c in window_lines)

        m_code = CODE_PAT.search(content)  # code is on the match line
        if not m_code:
            continue
        code = m_code.group(1)
        if int(code) == 0:
            continue  # 0000_error is a placeholder in comments, not a real code
        if code in seen:
            continue

        severity = severity_from_window(window)
        message = extract_message(window, code)

        seen[code] = {
            "code": int(code),
            "severity": severity,
            "message": message,
            "source": shorten_path(filepath),
            "line": lineno,
            "action": None,
        }

    return sorted(seen.values(), key=lambda x: x["code"])


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

SEARCH_DIRS = [
    "libsolidity",
    "liblangutil",
    "libyul",
    "libevmasm",
    "libsolutil",
]


def run_rg(solidity_repo: Path) -> str:
    dirs = [d for d in SEARCH_DIRS if (solidity_repo / d).exists()]
    cmd = [
        "rg",
        "--no-heading",
        "-n",
        r"\b\d{4}_error\b",
        "-C",
        "6",  # 6 lines context — enough for any multiline call
        "-S",  # smart case
        "-g",
        "*.cpp",
        "-g",
        "*.h",
    ] + dirs

    result = subprocess.run(
        cmd,
        cwd=solidity_repo,
        capture_output=True,
        text=True,
    )
    if result.returncode not in (0, 1):
        print(f"rg failed: {result.stderr}", file=sys.stderr)
        sys.exit(1)
    return result.stdout


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--solidity-repo",
        default="../argotorg/solidity",
        help="Path to the solidity compiler source tree (default: ../argotorg/solidity)",
    )
    parser.add_argument(
        "--output",
        default="data/error_codes.json",
        help="Output JSON file path (default: data/error_codes.json)",
    )
    parser.add_argument(
        "--from-file",
        help="Use a pre-captured rg output file instead of running rg",
    )
    args = parser.parse_args()

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    if args.from_file:
        print(f"Reading rg output from {args.from_file} ...")
        raw = Path(args.from_file).read_text()
    else:
        solidity_repo = Path(args.solidity_repo).expanduser().resolve()
        if not solidity_repo.is_dir():
            print(f"error: solidity repo not found at {solidity_repo}", file=sys.stderr)
            sys.exit(1)
        print(f"Searching {solidity_repo} ...")
        raw = run_rg(solidity_repo)

    print("Parsing context blocks ...")
    entries = parse_context_output(raw)

    counts = Counter(e["severity"] for e in entries)
    no_msg = sum(1 for e in entries if not e["message"])
    print(f"  Total unique codes : {len(entries)}")
    for sev, n in sorted(counts.items()):
        print(f"    {sev:8s}: {n}")
    print(f"  No message found   : {no_msg}")

    with open(output_path, "w") as f:
        json.dump(entries, f, indent=2)
        f.write("\n")

    print(f"Written {output_path}")


if __name__ == "__main__":
    main()
