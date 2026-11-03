//! The real binary, run with `--check-invariants`, over multibyte CRLF source.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;
use tower_lsp::lsp_types::Url;

const SOURCE: &str = "// SPDX-License-Identifier: MIT\r\npragma solidity ^0.8.0;\r\n\r\n/* é😀 notes\r\n   more é😀 */\r\ncontract Probe {\r\n    string label = \"é😀\"; uint256 public total;\r\n\r\n    function bump(uint256 by) external {\r\n        total += by; // é😀\r\n    }\r\n}\r\n";

fn send(stdin: &mut ChildStdin, message: &Value) {
    let body = message.to_string();
    write!(stdin, "Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
    stdin.flush().unwrap();
}

fn reader(stdout: impl Read + Send + 'static) -> Receiver<Value> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut stdout = BufReader::new(stdout);
        loop {
            let mut length = 0;
            loop {
                let mut header = String::new();
                if stdout.read_line(&mut header).unwrap_or(0) == 0 {
                    return;
                }
                if let Some(value) = header.strip_prefix("Content-Length: ") {
                    length = value.trim().parse().unwrap();
                }
                if header == "\r\n" {
                    break;
                }
            }
            let mut body = vec![0; length];
            if stdout.read_exact(&mut body).is_err() {
                return;
            }
            if tx.send(serde_json::from_slice(&body).unwrap()).is_err() {
                return;
            }
        }
    });
    rx
}

fn request(
    stdin: &mut ChildStdin,
    rx: &Receiver<Value>,
    id: u64,
    method: &str,
    params: Value,
) -> Value {
    send(
        stdin,
        &json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}),
    );
    loop {
        let message = rx
            .recv_timeout(Duration::from_secs(30))
            .unwrap_or_else(|_| panic!("no response to {method}"));
        // Answer anything the server asks so it never blocks on the client.
        if message.get("method").is_some() {
            if let Some(server_id) = message.get("id") {
                send(
                    stdin,
                    &json!({"jsonrpc": "2.0", "id": server_id, "result": null}),
                );
            }
            continue;
        }
        if message.get("id") == Some(&json!(id)) {
            return message;
        }
    }
}

fn exercise(encoding: &str) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("Probe.sol");
    std::fs::write(&path, SOURCE).unwrap();
    let uri = Url::from_file_path(&path).unwrap().to_string();

    let mut child = Command::new(env!("CARGO_BIN_EXE_solidity-language-server"))
        .args(["--stdio", "--check-invariants"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let rx = reader(child.stdout.take().unwrap());

    let initialized = request(
        &mut stdin,
        &rx,
        1,
        "initialize",
        json!({
            "processId": null,
            "rootUri": Url::from_directory_path(dir.path()).unwrap(),
            "capabilities": {"general": {"positionEncodings": [encoding]}},
        }),
    );
    assert_eq!(
        initialized.pointer("/result/capabilities/positionEncoding"),
        Some(&json!(encoding))
    );
    send(
        &mut stdin,
        &json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
    );
    send(
        &mut stdin,
        &json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": {"uri": uri, "languageId": "solidity", "version": 1, "text": SOURCE},
        }}),
    );

    let document = json!({"textDocument": {"uri": uri}});
    let at = |line: u32, character: u32| json!({"textDocument": {"uri": uri}, "position": {"line": line, "character": character}});
    let methods = [
        ("textDocument/documentSymbol", document.clone()),
        ("textDocument/foldingRange", document.clone()),
        ("textDocument/documentLink", document.clone()),
        ("textDocument/semanticTokens/full", document.clone()),
        (
            "textDocument/selectionRange",
            json!({"textDocument": {"uri": uri}, "positions": [
                {"line": 6, "character": 30}, {"line": 9, "character": 15}, {"line": 99, "character": 99},
            ]}),
        ),
        ("textDocument/documentHighlight", at(8, 14)),
        ("textDocument/definition", at(9, 8)),
    ];
    for (index, (method, params)) in methods.into_iter().enumerate() {
        let response = request(&mut stdin, &rx, 10 + index as u64, method, params);
        assert!(
            response.get("error").is_none(),
            "{encoding} {method}: {response}"
        );
        assert!(response.get("result").is_some(), "{method}: {response}");
    }

    let _ = request(&mut stdin, &rx, 99, "shutdown", Value::Null);
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn responses_satisfy_invariants_with_utf16_positions() {
    exercise("utf-16");
}

#[test]
fn responses_satisfy_invariants_with_utf8_positions() {
    exercise("utf-8");
}
