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

struct LspProc {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<Value>,
    stash: VecDeque<Value>,
    next_id: u64,
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
        }
    }

    fn send_notification(&mut self, method: &str, params: Value) {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        self.write_msg(&msg);
    }

    fn send_request(&mut self, method: &str, params: Value) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        self.write_msg(&msg);
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

    fn send_result(&mut self, id: u64, result: Value) {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        });
        self.write_msg(&msg);
    }

    fn wait_for_method(&mut self, method: &str, timeout: Duration) -> Option<Value> {
        if let Some(idx) = self
            .stash
            .iter()
            .position(|msg| msg.get("method").and_then(Value::as_str) == Some(method))
        {
            return self.stash.remove(idx);
        }

        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return None;
            }
            match self.rx.recv_timeout(remaining) {
                Ok(msg) => {
                    if msg.get("method").and_then(Value::as_str) == Some(method) {
                        return Some(msg);
                    }
                    self.stash.push_back(msg);
                }
                Err(RecvTimeoutError::Timeout) => return None,
                Err(RecvTimeoutError::Disconnected) => return None,
            }
        }
    }

    fn wait_response(&mut self, id: u64, timeout: Duration) -> Value {
        let deadline = Instant::now() + timeout;
        loop {
            assert!(
                Instant::now() < deadline,
                "timed out waiting response id={id}"
            );
            if let Some(msg) = self.stash.pop_front() {
                if msg.get("id").and_then(|v| v.as_u64()) == Some(id) {
                    return msg;
                }
                continue;
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(remaining) {
                Ok(msg) => {
                    if msg.get("id").and_then(|v| v.as_u64()) == Some(id) {
                        return msg;
                    }
                    self.stash.push_back(msg);
                }
                Err(RecvTimeoutError::Timeout) => {
                    panic!("timed out waiting response id={id}");
                }
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("LSP reader disconnected while waiting response id={id}");
                }
            }
        }
    }

    fn shutdown(mut self) {
        let shutdown_id = self.send_request("shutdown", Value::Null);
        let _ = self.wait_response(shutdown_id, Duration::from_secs(5));
        self.send_notification("exit", Value::Null);
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
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

fn spawn_reader(stdout: ChildStdout) -> Receiver<Value> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            let mut content_length: usize = 0;
            line.clear();
            let n = match reader.read_line(&mut line) {
                Ok(n) => n,
                Err(_) => return,
            };
            if n == 0 {
                return;
            }
            loop {
                if line == "\r\n" {
                    break;
                }
                if let Some(v) = line.strip_prefix("Content-Length:") {
                    content_length = v.trim().parse::<usize>().unwrap_or(0);
                }
                line.clear();
                let n = match reader.read_line(&mut line) {
                    Ok(n) => n,
                    Err(_) => return,
                };
                if n == 0 {
                    return;
                }
            }
            if content_length == 0 {
                continue;
            }
            let mut body = vec![0u8; content_length];
            if reader.read_exact(&mut body).is_err() {
                return;
            }
            let Ok(msg) = serde_json::from_slice::<Value>(&body) else {
                continue;
            };
            let _ = tx.send(msg);
        }
    });
    rx
}

fn write_foundry_project(dir: &Path) -> (String, String, String) {
    let src = dir.join("src");
    fs::create_dir_all(&src).expect("create src");
    fs::write(
        dir.join("foundry.toml"),
        r#"[profile.default]
src = "src"
"#,
    )
    .expect("write foundry.toml");

    let a = src.join("A.sol");
    let b = src.join("B.sol");
    fs::write(
        &a,
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract A {}\n",
    )
    .expect("write A.sol");
    fs::write(
        &b,
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {A} from \"./A.sol\";\ncontract B is A {}\n",
    )
    .expect("write B.sol");

    let root_uri = Url::from_file_path(dir).expect("root uri").to_string();
    let a_uri = Url::from_file_path(&a).expect("a uri").to_string();
    let b_uri = Url::from_file_path(&b).expect("b uri").to_string();
    (root_uri, a_uri, b_uri)
}

fn initialize_server(lsp: &mut LspProc, root_uri: &str) {
    let init_id = lsp.send_request(
        "initialize",
        json!({
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {
                "workspace": {
                    "fileOperations": {
                        "willDelete": true
                    }
                }
            }
        }),
    );
    let init_resp = lsp.wait_response(init_id, Duration::from_secs(10));
    assert!(
        init_resp.get("result").is_some(),
        "initialize failed: {init_resp}"
    );
    lsp.send_notification("initialized", json!({}));
}

fn set_file_op_settings(
    lsp: &mut LspProc,
    template_on_create: bool,
    update_imports_on_rename: bool,
    update_imports_on_delete: bool,
) {
    lsp.send_notification(
        "workspace/didChangeConfiguration",
        json!({
            "settings": {
                "solidity-language-server": {
                    "fileOperations": {
                        "templateOnCreate": template_on_create,
                        "updateImportsOnRename": update_imports_on_rename,
                        "updateImportsOnDelete": update_imports_on_delete
                    }
                }
            }
        }),
    );
}

#[test]
fn will_delete_files_returns_edits_when_enabled() {
    let tmp = TempDir::new().expect("tmp dir");
    let (root_uri, a_uri, b_uri) = write_foundry_project(tmp.path());
    let mut lsp = LspProc::spawn(tmp.path());
    initialize_server(&mut lsp, &root_uri);

    let req_id = lsp.send_request(
        "workspace/willDeleteFiles",
        json!({
            "files": [{ "uri": a_uri }]
        }),
    );
    let resp = lsp.wait_response(req_id, Duration::from_secs(15));
    let result = resp.get("result").expect("missing result");
    let changes = result
        .get("changes")
        .and_then(Value::as_object)
        .expect("expected workspace edit changes");
    assert!(
        changes.contains_key(&b_uri),
        "expected B.sol import edit in response: {resp}"
    );

    lsp.shutdown();
}

#[test]
fn will_delete_files_returns_null_when_update_imports_on_delete_disabled() {
    let tmp = TempDir::new().expect("tmp dir");
    let (root_uri, a_uri, _b_uri) = write_foundry_project(tmp.path());
    let mut lsp = LspProc::spawn(tmp.path());
    initialize_server(&mut lsp, &root_uri);

    lsp.send_notification(
        "workspace/didChangeConfiguration",
        json!({
            "settings": {
                "solidity-language-server": {
                    "fileOperations": {
                        "updateImportsOnDelete": false
                    }
                }
            }
        }),
    );

    let req_id = lsp.send_request(
        "workspace/willDeleteFiles",
        json!({
            "files": [{ "uri": a_uri }]
        }),
    );
    let resp = lsp.wait_response(req_id, Duration::from_secs(15));
    assert!(
        resp.get("result").is_some_and(Value::is_null),
        "expected null result when disabled: {resp}"
    );

    lsp.shutdown();
}

#[test]
fn did_create_files_requests_apply_edit_when_template_enabled() {
    let tmp = TempDir::new().expect("tmp dir");
    let (root_uri, _a_uri, _b_uri) = write_foundry_project(tmp.path());
    let mut lsp = LspProc::spawn(tmp.path());
    initialize_server(&mut lsp, &root_uri);

    // didCreateFiles: server should request workspace/applyEdit scaffolding.
    let c_path = tmp.path().join("src").join("C.sol");
    fs::write(&c_path, "").expect("create C.sol");
    let c_uri = Url::from_file_path(&c_path).expect("c uri").to_string();
    lsp.send_notification(
        "workspace/didCreateFiles",
        json!({
            "files": [{ "uri": c_uri }]
        }),
    );

    let apply_req = lsp
        .wait_for_method("workspace/applyEdit", Duration::from_secs(10))
        .expect("expected workspace/applyEdit request for didCreateFiles");
    let apply_id = apply_req
        .get("id")
        .and_then(Value::as_u64)
        .expect("applyEdit request id");
    let apply_changes = apply_req
        .pointer("/params/edit/changes")
        .and_then(Value::as_object)
        .expect("applyEdit edit.changes");
    let c_edits = apply_changes
        .get(&c_uri)
        .and_then(Value::as_array)
        .expect("expected C.sol edit");
    let new_text = c_edits
        .first()
        .and_then(|v| v.get("newText"))
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        new_text.contains("contract C"),
        "expected scaffold contract in applyEdit: {new_text}"
    );
    lsp.send_result(apply_id, json!({ "applied": true }));

    lsp.shutdown();
}

#[test]
fn did_create_files_skips_apply_edit_when_template_disabled() {
    let tmp = TempDir::new().expect("tmp dir");
    let (root_uri, _a_uri, _b_uri) = write_foundry_project(tmp.path());
    let mut lsp = LspProc::spawn(tmp.path());
    initialize_server(&mut lsp, &root_uri);
    set_file_op_settings(&mut lsp, false, true, true);

    let c_path = tmp.path().join("src").join("C.sol");
    fs::write(&c_path, "").expect("create C.sol");
    let c_uri = Url::from_file_path(&c_path).expect("c uri").to_string();
    lsp.send_notification(
        "workspace/didCreateFiles",
        json!({
            "files": [{ "uri": c_uri }]
        }),
    );

    let apply_req = lsp.wait_for_method("workspace/applyEdit", Duration::from_secs(2));
    assert!(
        apply_req.is_none(),
        "expected no workspace/applyEdit when templateOnCreate=false: {apply_req:?}"
    );

    lsp.shutdown();
}

#[test]
fn will_rename_files_returns_edits_when_enabled() {
    let tmp = TempDir::new().expect("tmp dir");
    let (root_uri, a_uri, b_uri) = write_foundry_project(tmp.path());
    let mut lsp = LspProc::spawn(tmp.path());
    initialize_server(&mut lsp, &root_uri);

    // willRenameFiles: should preview import rewrite in B.sol.
    let req_rename = lsp.send_request(
        "workspace/willRenameFiles",
        json!({
            "files": [{
                "oldUri": a_uri,
                "newUri": Url::from_file_path(tmp.path().join("src").join("AA.sol"))
                    .expect("aa uri")
                    .to_string()
            }]
        }),
    );
    let resp_rename = lsp.wait_response(req_rename, Duration::from_secs(15));
    let rename_changes = resp_rename
        .get("result")
        .and_then(|v| v.get("changes"))
        .and_then(Value::as_object)
        .expect("expected willRenameFiles workspace edit changes");
    assert!(
        rename_changes.contains_key(&b_uri),
        "expected B.sol rename import edit: {resp_rename}"
    );

    lsp.shutdown();
}

#[test]
fn will_rename_files_returns_null_when_disabled() {
    let tmp = TempDir::new().expect("tmp dir");
    let (root_uri, a_uri, _b_uri) = write_foundry_project(tmp.path());
    let mut lsp = LspProc::spawn(tmp.path());
    initialize_server(&mut lsp, &root_uri);
    set_file_op_settings(&mut lsp, true, false, true);

    // willRenameFiles: should return null when disabled.
    let req_rename = lsp.send_request(
        "workspace/willRenameFiles",
        json!({
            "files": [{
                "oldUri": a_uri,
                "newUri": Url::from_file_path(tmp.path().join("src").join("AA.sol"))
                    .expect("aa uri")
                    .to_string()
            }]
        }),
    );
    let resp_rename = lsp.wait_response(req_rename, Duration::from_secs(15));
    assert!(
        resp_rename.get("result").is_some_and(Value::is_null),
        "expected null willRenameFiles result when disabled: {resp_rename}"
    );

    lsp.shutdown();
}
