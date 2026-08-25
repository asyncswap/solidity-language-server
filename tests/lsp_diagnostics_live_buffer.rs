//! Diagnostics must be positioned against the text that was compiled.
//!
//! `on_change` hands solc the editor's live buffer, so the byte offsets it
//! returns describe that buffer. Mapping them through the on-disk file instead
//! puts every squiggle on the wrong line whenever the two differ — which is the
//! normal state of an editor with unsaved edits.
//!
//! Requires the Foundry toolchain on PATH, like `tests/build.rs`. CI installs it
//! via `foundry-rs/foundry-toolchain` before `cargo test`.

use serde_json::{Value, json};
use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tower_lsp::lsp_types::Url;

/// What sits on disk: compiles cleanly.
const ON_DISK: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Probe {
    uint256 public total;

    function ok() external view returns (uint256) {
        return total;
    }
}
"#;

/// The unsaved buffer: four extra lines above, and a type error further down.
const IN_BUFFER: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// unsaved line A
// unsaved line B
// unsaved line C
// unsaved line D

contract Probe {
    uint256 public total;

    function ok() external view returns (uint256) {
        total = "not a number";
        return total;
    }
}
"#;

/// On disk for the second case: same contract, padded well past the buffer's
/// length so a bad offset still lands *inside* this text.
const PADDED_ON_DISK: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Padding so this file is comfortably longer than the buffer below.
// Padding line 2.
// Padding line 3.
// Padding line 4.
// Padding line 5.
// Padding line 6.
// Padding line 7.
// Padding line 8.

contract Probe {
    uint256 public total;

    function ok() external view returns (uint256) {
        return total;
    }
}
"#;

/// The unsaved buffer for the second case: shorter than what is on disk, with
/// the same injected type error.
const SHORT_BUFFER: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Probe {
    uint256 public total;

    function ok() external view returns (uint256) {
        total = "not a number";
        return total;
    }
}
"#;

struct LspProc {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<Value>,
    stash: VecDeque<Value>,
    next_id: u64,
    /// Set by `shutdown`; without it `Drop` reaps a server orphaned by a panic.
    graceful: bool,
}

impl LspProc {
    fn spawn(cwd: &Path) -> Self {
        let bin = option_env!("CARGO_BIN_EXE_solidity-language-server")
            .or(option_env!("CARGO_BIN_EXE_solidity_language_server"))
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("target")
                    .join("debug")
                    .join("solidity-language-server")
            });

        let mut child = Command::new(bin)
            .arg("--stdio")
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn solidity-language-server");

        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        let rx = spawn_reader(stdout);
        Self {
            child,
            stdin,
            rx,
            stash: VecDeque::new(),
            next_id: 1,
            graceful: false,
        }
    }

    fn send_notification(&mut self, method: &str, params: Value) {
        self.write_msg(&json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    fn send_request(&mut self, method: &str, params: Value) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.write_msg(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        }));
        id
    }

    fn write_msg(&mut self, msg: &Value) {
        let body = serde_json::to_vec(msg).expect("serialize");
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        self.stdin
            .write_all(header.as_bytes())
            .expect("write header");
        self.stdin.write_all(&body).expect("write body");
        self.stdin.flush().expect("flush");
    }

    fn wait_response(&mut self, id: u64, timeout: Duration) -> Value {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(idx) = self
                .stash
                .iter()
                .position(|m| m.get("id").and_then(Value::as_u64) == Some(id))
            {
                return self.stash.remove(idx).expect("stashed response");
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(!remaining.is_zero(), "timed out waiting for response {id}");
            match self.rx.recv_timeout(remaining) {
                Ok(msg) => self.stash.push_back(msg),
                Err(RecvTimeoutError::Timeout) => panic!("timed out waiting for response {id}"),
                Err(RecvTimeoutError::Disconnected) => panic!("server exited unexpectedly"),
            }
        }
    }

    /// Wait for `n` `publishDiagnostics` payloads for `uri`, empty included.
    ///
    /// One is not enough to mean "the build finished": `on_change` publishes an
    /// empty payload up front to clear stale squiggles, before solc is invoked
    /// at all. The second payload is the compiled result, so that is what tells
    /// us the initial build is done and the buffer is safe to dirty.
    fn wait_for_diagnostics_count(&mut self, uri: &str, n: usize, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let is_match = |msg: &Value| {
            msg.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
                && msg.pointer("/params/uri").and_then(Value::as_str) == Some(uri)
        };
        let mut seen = 0;
        loop {
            while let Some(idx) = self.stash.iter().position(is_match) {
                self.stash.remove(idx);
                seen += 1;
                if seen >= n {
                    return true;
                }
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return false;
            }
            match self.rx.recv_timeout(remaining) {
                Ok(msg) => {
                    if is_match(&msg) {
                        seen += 1;
                        if seen >= n {
                            return true;
                        }
                    } else {
                        self.stash.push_back(msg);
                    }
                }
                Err(_) => return false,
            }
        }
    }

    /// Wait for a `publishDiagnostics` for `uri` that carries at least one
    /// error-severity entry. Empty payloads are published routinely (to clear
    /// stale squiggles), so they are skipped rather than accepted.
    fn wait_for_error_diagnostics(&mut self, uri: &str, timeout: Duration) -> Vec<Value> {
        let deadline = Instant::now() + timeout;
        loop {
            while let Some(msg) = self.stash.pop_front() {
                if let Some(diags) = error_diagnostics_for(&msg, uri) {
                    return diags;
                }
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return vec![];
            }
            match self.rx.recv_timeout(remaining) {
                Ok(msg) => {
                    if let Some(diags) = error_diagnostics_for(&msg, uri) {
                        return diags;
                    }
                }
                Err(_) => return vec![],
            }
        }
    }

    fn shutdown(mut self) {
        self.graceful = true;
        let id = self.send_request("shutdown", Value::Null);
        let _ = self.wait_response(id, Duration::from_secs(5));
        self.send_notification("exit", Value::Null);
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(20));
                }
                Ok(None) | Err(_) => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    break;
                }
            }
        }
    }
}

/// Any assertion that fires before `shutdown` unwinds past it, and dropping a
/// `Child` does not kill the process, so reap it here.
impl Drop for LspProc {
    fn drop(&mut self) {
        if !self.graceful {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn error_diagnostics_for(msg: &Value, uri: &str) -> Option<Vec<Value>> {
    if msg.get("method").and_then(Value::as_str) != Some("textDocument/publishDiagnostics") {
        return None;
    }
    let params = msg.get("params")?;
    if params.get("uri").and_then(Value::as_str) != Some(uri) {
        return None;
    }
    let errors: Vec<Value> = params
        .get("diagnostics")?
        .as_array()?
        .iter()
        .filter(|d| d.get("severity").and_then(Value::as_u64) == Some(1))
        .cloned()
        .collect();
    (!errors.is_empty()).then_some(errors)
}

fn spawn_reader(stdout: ChildStdout) -> Receiver<Value> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            let mut content_length: usize = 0;
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) | Err(_) => return,
                Ok(_) => {}
            }
            loop {
                if line == "\r\n" {
                    break;
                }
                if let Some(v) = line.strip_prefix("Content-Length:") {
                    content_length = v.trim().parse::<usize>().unwrap_or(0);
                }
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => return,
                    Ok(_) => {}
                }
            }
            if content_length == 0 {
                continue;
            }
            let mut body = vec![0u8; content_length];
            if reader.read_exact(&mut body).is_err() {
                return;
            }
            if let Ok(msg) = serde_json::from_slice::<Value>(&body) {
                let _ = tx.send(msg);
            }
        }
    });
    rx
}

fn write_project(dir: &Path, on_disk: &str) -> (String, String) {
    let src = dir.join("src");
    fs::create_dir_all(&src).expect("create src");
    // Lint is deliberately left at its default (on). That is the branch a real
    // Foundry project takes, and it is the one of the two changed call sites
    // that would otherwise go untested. The assertions filter to severity 1, so
    // forge-lint warnings cannot interfere.
    fs::write(
        dir.join("foundry.toml"),
        "[profile.default]\nsrc = \"src\"\n",
    )
    .expect("write foundry.toml");

    let probe = src.join("Probe.sol");
    fs::write(&probe, on_disk).expect("write Probe.sol");

    let root_uri = Url::from_file_path(dir).expect("root uri").to_string();
    let probe_uri = Url::from_file_path(&probe).expect("probe uri").to_string();
    (root_uri, probe_uri)
}

fn initialize_server(lsp: &mut LspProc, root_uri: &str) {
    let id = lsp.send_request(
        "initialize",
        json!({
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {},
            "initializationOptions": {
                "projectIndex": { "fullProjectScan": false }
            }
        }),
    );
    let resp = lsp.wait_response(id, Duration::from_secs(30));
    assert!(resp.get("result").is_some(), "initialize failed: {resp}");
    lsp.send_notification("initialized", json!({}));
}

/// Drive one disk-vs-buffer divergence and return the first error diagnostic's
/// range as (start_line, start_char, end_line, end_char).
fn reported_error_range(on_disk: &str, in_buffer: &str) -> (u64, u64, u64, u64) {
    let dir = TempDir::new().expect("tempdir");
    let (root_uri, probe_uri) = write_project(dir.path(), on_disk);

    let mut lsp = LspProc::spawn(dir.path());
    initialize_server(&mut lsp, &root_uri);

    lsp.send_notification(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": probe_uri,
                "languageId": "solidity",
                "version": 1,
                "text": on_disk,
            }
        }),
    );

    // Two payloads: the up-front clear, then the compiled result. Waiting for
    // only the first would return before solc has run and race the edit below.
    assert!(
        lsp.wait_for_diagnostics_count(&probe_uri, 2, Duration::from_secs(180)),
        "server never published a compiled result for the freshly opened file — \
         is the Foundry toolchain on PATH?"
    );

    // Unsaved edit: the buffer now differs from what is on disk.
    lsp.send_notification(
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": probe_uri, "version": 2 },
            "contentChanges": [{ "text": in_buffer }],
        }),
    );

    // didSave without the optional `text`, so the server compiles from its
    // text_cache — the dirty buffer — while the file on disk is unchanged.
    lsp.send_notification(
        "textDocument/didSave",
        json!({ "textDocument": { "uri": probe_uri } }),
    );

    let diags = lsp.wait_for_error_diagnostics(&probe_uri, Duration::from_secs(120));
    lsp.shutdown();

    assert!(
        !diags.is_empty(),
        "no error diagnostics arrived — the Foundry toolchain may be missing, \
         or solc never reported the injected type error"
    );

    let range = &diags[0]["range"];
    let field = |a: &str, b: &str| {
        range[a][b]
            .as_u64()
            .unwrap_or_else(|| panic!("diagnostic range missing {a}.{b}"))
    };
    (
        field("start", "line"),
        field("start", "character"),
        field("end", "line"),
        field("end", "character"),
    )
}

/// Locate `needle` in `text` as (line, start_char, end_char), all zero-indexed
/// and in UTF-16 units — the fixtures are ASCII, so bytes and units coincide.
fn locate(text: &str, needle: &str) -> (u64, u64, u64) {
    let (line_no, line) = text
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(needle))
        .unwrap_or_else(|| panic!("fixture does not contain {needle:?}"));
    let col = line.find(needle).expect("needle on the located line") as u64;
    (line_no as u64, col, col + needle.len() as u64)
}

/// Buffer LONGER than disk: the error's byte offset runs past the end of the
/// on-disk text, so a disk-based mapping clamps to end-of-file.
#[test]
fn diagnostics_are_positioned_against_the_live_buffer() {
    let (line, start_col, end_col) = locate(IN_BUFFER, "\"not a number\"");
    let got = reported_error_range(ON_DISK, IN_BUFFER);

    assert_eq!(
        got,
        (line, start_col, line, end_col),
        "diagnostic landed at {got:?}, but the error spans line {line} \
         cols {start_col}..{end_col} of the live buffer; offsets were mapped \
         against the on-disk text"
    );
}

/// Buffer SHORTER than disk: the offset still resolves inside the on-disk text,
/// so a disk-based mapping yields a wrong-but-plausible line rather than an
/// obvious end-of-file clamp. This is the variant that hides in a bug report.
#[test]
fn diagnostics_are_positioned_against_a_buffer_shorter_than_disk() {
    let (line, start_col, end_col) = locate(SHORT_BUFFER, "\"not a number\"");
    let got = reported_error_range(PADDED_ON_DISK, SHORT_BUFFER);

    assert_eq!(
        got,
        (line, start_col, line, end_col),
        "diagnostic landed at {got:?}, but the error spans line {line} \
         cols {start_col}..{end_col} of the live buffer"
    );
}
