#!/usr/bin/env python3
"""
Inspect a lsp-bench results.json by printing each Location with the
resolved symbol text and its source line.

Usage:
  python3 inspect_results.py <path/to/results.json> [benchmark_index] [server_index]

Defaults: benchmark 0, server 0.

Exit status:
  0 always (printer, not a verifier).
"""

import json
import sys
from collections import defaultdict
from pathlib import Path


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2

    results_path = Path(sys.argv[1])
    bench_idx = int(sys.argv[2]) if len(sys.argv) > 2 else 0
    server_idx = int(sys.argv[3]) if len(sys.argv) > 3 else 0

    with results_path.open() as f:
        data = json.load(f)

    bench = data["benchmarks"][bench_idx]
    server = bench["servers"][server_idx]
    method = bench.get("name", "?")
    server_name = server.get("server", "?")
    response = server.get("response")

    print(f"results: {results_path}")
    print(f"benchmark[{bench_idx}]: {method}")
    print(f"server[{server_idx}]:    {server_name}")
    print(f"input:    {bench.get('input', '')[:200]}{'...' if len(bench.get('input', '')) > 200 else ''}")
    print()

    if response is None:
        print("response: null")
        return 0
    if not isinstance(response, list):
        print(f"response (type={type(response).__name__}): {json.dumps(response, indent=2)[:500]}")
        return 0

    print(f"total locations: {len(response)}")
    print()

    by_file: dict[str, list[dict]] = defaultdict(list)
    for loc in response:
        uri = loc.get("uri", "")
        path = uri.replace("file://", "")
        by_file[path].append(loc.get("range", {}))

    # Find common prefix to shorten paths in output
    paths = list(by_file.keys())
    common = ""
    if paths:
        common = paths[0]
        for p in paths[1:]:
            i = 0
            while i < len(common) and i < len(p) and common[i] == p[i]:
                i += 1
            common = common[:i]
        # Trim back to a directory boundary
        common = common.rsplit("/", 1)[0] + "/" if "/" in common else ""

    for path in sorted(by_file):
        rel = path[len(common):] if common and path.startswith(common) else path
        ranges = sorted(
            by_file[path],
            key=lambda r: (r.get("start", {}).get("line", 0), r.get("start", {}).get("character", 0)),
        )
        try:
            lines = Path(path).read_text().splitlines()
        except OSError:
            lines = []

        print(f"=== {rel}  ({len(ranges)} refs) ===")
        for r in ranges:
            start = r.get("start", {})
            end = r.get("end", {})
            ln = start.get("line", 0)        # 0-indexed
            sc = start.get("character", 0)
            ec = end.get("character", 0)
            line_text = lines[ln] if 0 <= ln < len(lines) else "<out-of-range>"
            symbol = line_text[sc:ec] if 0 <= ln < len(lines) else ""
            print(f"  {ln + 1:>5}:{sc + 1:<3}-{ec + 1:<3}  [{symbol}]  {line_text.strip()}")
        print()

    if common:
        print(f"common prefix stripped: {common}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
