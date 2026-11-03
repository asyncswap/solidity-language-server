//! Runtime checks of the rules every LSP response must satisfy.
//!
//! [`check_response`] validates one response against the LSP specification and
//! the document it describes. [`InvariantService`] applies it to every response
//! when the server runs with `--check-invariants`, so tests, `lsp-bench` runs
//! and dogfooding sessions all verify themselves. A violation is printed to
//! stderr and turns the response into a JSON-RPC error.
//!
//! Only the structure of a response is checked, not whether it is the right
//! answer. Locations from solc's AST can legitimately describe text older than
//! the buffer, so they are only checked for being well formed; document bounds
//! apply to methods computed from the live buffer.

use crate::semantic_tokens;
use crate::utils::{self, PositionEncoding};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::io::Write;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tower_lsp::jsonrpc::{Error, ErrorCode, Request, Response};
use tower_service::Service;

/// One broken rule in a response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub rule: &'static str,
    pub detail: String,
}

type Pos = (u64, u64);
type Violations = Vec<Violation>;

fn fail(out: &mut Violations, rule: &'static str, detail: impl Into<String>) {
    out.push(Violation {
        rule,
        detail: detail.into(),
    });
}

/// Line lengths of a document in the client's encoding.
struct DocLines(Vec<u64>);

impl DocLines {
    /// Tree-sitter breaks lines on `\n` only while LSP also breaks on a bare
    /// `\r`, so ranges in such a file have no single meaning to check against.
    fn new(text: &str, encoding: PositionEncoding) -> Option<Self> {
        let bytes = text.as_bytes();
        let bare_cr =
            (0..bytes.len()).any(|i| bytes[i] == b'\r' && bytes.get(i + 1) != Some(&b'\n'));
        if bare_cr {
            return None;
        }
        Some(Self(
            text.split('\n')
                .map(|line| {
                    let line = line.strip_suffix('\r').unwrap_or(line);
                    u64::from(utils::encoded_len(line, encoding))
                })
                .collect(),
        ))
    }

    fn contains(&self, (line, character): Pos) -> bool {
        usize::try_from(line)
            .ok()
            .and_then(|line| self.0.get(line))
            .is_some_and(|len| character <= *len)
    }
}

fn parse_pos(value: &Value) -> Option<Pos> {
    Some((
        value.get("line")?.as_u64()?,
        value.get("character")?.as_u64()?,
    ))
}

fn parse_range(value: &Value) -> Option<(Pos, Pos)> {
    Some((
        parse_pos(value.get("start")?)?,
        parse_pos(value.get("end")?)?,
    ))
}

/// Parse a range that must exist and must not be inverted.
fn range_of(value: Option<&Value>, out: &mut Violations) -> Option<(Pos, Pos)> {
    let Some(range) = value.and_then(parse_range) else {
        fail(out, "range-malformed", format!("not a range: {value:?}"));
        return None;
    };
    if range.0 > range.1 {
        fail(out, "range-inverted", format!("{range:?}"));
        return None;
    }
    Some(range)
}

fn check_in_doc(range: (Pos, Pos), doc: Option<&DocLines>, out: &mut Violations) {
    if let Some(doc) = doc
        && !(doc.contains(range.0) && doc.contains(range.1))
    {
        fail(out, "range-outside-document", format!("{range:?}"));
    }
}

fn contains(outer: (Pos, Pos), inner: (Pos, Pos)) -> bool {
    outer.0 <= inner.0 && inner.1 <= outer.1
}

fn check_document_symbols(items: &[Value], doc: Option<&DocLines>, out: &mut Violations) {
    for item in items {
        // Flat `SymbolInformation` carries a location instead of ranges.
        if let Some(location) = item.get("location") {
            if let Some(range) = range_of(location.get("range"), out) {
                check_in_doc(range, doc, out);
            }
            continue;
        }
        let range = range_of(item.get("range"), out);
        let selection = range_of(item.get("selectionRange"), out);
        if let (Some(range), Some(selection)) = (range, selection) {
            check_in_doc(range, doc, out);
            if !contains(range, selection) {
                fail(
                    out,
                    "symbol-selection-outside-range",
                    format!("{selection:?} not in {range:?}"),
                );
            }
        }
        if let Some(children) = item.get("children").and_then(Value::as_array) {
            check_document_symbols(children, doc, out);
        }
    }
}

fn check_selection_ranges(
    items: &[Value],
    params: Option<&Value>,
    doc: Option<&DocLines>,
    out: &mut Violations,
) {
    let requested = params
        .and_then(|p| p.get("positions"))
        .and_then(Value::as_array)
        .map(Vec::len);
    if let Some(requested) = requested
        && requested != items.len()
    {
        fail(
            out,
            "selection-count-mismatch",
            format!("{} results for {requested} positions", items.len()),
        );
    }
    for item in items {
        let mut current = item;
        let mut child = None;
        while let Some(range) = range_of(current.get("range"), out) {
            check_in_doc(range, doc, out);
            if let Some(child) = child
                && !contains(range, child)
            {
                fail(
                    out,
                    "selection-parent-excludes-child",
                    format!("{child:?} not in {range:?}"),
                );
            }
            child = Some(range);
            match current.get("parent") {
                Some(parent) if !parent.is_null() => current = parent,
                _ => break,
            }
        }
    }
}

fn check_folding_ranges(items: &[Value], doc: Option<&DocLines>, out: &mut Violations) {
    for item in items {
        let start = item.get("startLine").and_then(Value::as_u64);
        let end = item.get("endLine").and_then(Value::as_u64);
        let (Some(start), Some(end)) = (start, end) else {
            fail(out, "folding-malformed", format!("{item}"));
            continue;
        };
        if start > end {
            fail(out, "range-inverted", format!("folding {start}..{end}"));
        }
        if let Some(doc) = doc
            && end >= doc.0.len() as u64
        {
            fail(out, "range-outside-document", format!("folding end {end}"));
        }
    }
}

fn check_token_data(data: &[Value], doc: Option<&DocLines>, out: &mut Violations) {
    if !data.len().is_multiple_of(5) {
        fail(out, "tokens-malformed", format!("{} integers", data.len()));
        return;
    }
    let legend = semantic_tokens::legend().token_types.len() as u64;
    let (mut line, mut col, mut previous_end) = (0u64, 0u64, (0u64, 0u64));
    for token in data.chunks_exact(5) {
        let field = |i: usize| token[i].as_u64().unwrap_or(u64::MAX);
        let (delta_line, delta_start, length, kind) = (field(0), field(1), field(2), field(3));
        if [delta_line, delta_start, length, kind].contains(&u64::MAX) {
            fail(out, "tokens-malformed", format!("{token:?}"));
            return;
        }
        if delta_line > 0 {
            line += delta_line;
            col = delta_start;
        } else {
            col += delta_start;
        }
        if kind >= legend {
            fail(out, "token-type-outside-legend", format!("type {kind}"));
        }
        if (line, col) < previous_end {
            fail(out, "tokens-overlap", format!("{line}:{col}"));
        }
        previous_end = (line, col + length);
        check_in_doc(((line, col), previous_end), doc, out);
    }
}

fn check_semantic_tokens(result: &Value, doc: Option<&DocLines>, out: &mut Violations) {
    if let Some(data) = result.get("data").and_then(Value::as_array) {
        check_token_data(data, doc, out);
    }
    for edit in result
        .get("edits")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let start = edit.get("start").and_then(Value::as_u64);
        let delete = edit.get("deleteCount").and_then(Value::as_u64);
        if !matches!((start, delete), (Some(s), Some(d)) if s % 5 == 0 && d % 5 == 0) {
            fail(out, "token-delta-misaligned", format!("{edit}"));
        }
    }
}

/// Edits to one file may touch but must not overlap (several may share a start
/// only if they are insertions).
fn check_text_edits(edits: &[Value], out: &mut Violations) {
    let mut ranges: Vec<(Pos, Pos)> = edits
        .iter()
        .filter_map(|edit| range_of(edit.get("range"), out))
        .collect();
    ranges.sort();
    for pair in ranges.windows(2) {
        if pair[1].0 < pair[0].1 {
            fail(
                out,
                "text-edits-overlap",
                format!("{:?} and {:?}", pair[0], pair[1]),
            );
        }
    }
}

fn check_workspace_edit(result: &Value, out: &mut Violations) {
    if let Some(changes) = result.get("changes").and_then(Value::as_object) {
        for edits in changes.values().filter_map(Value::as_array) {
            check_text_edits(edits, out);
        }
    }
    let document_changes = result.get("documentChanges").and_then(Value::as_array);
    for change in document_changes.into_iter().flatten() {
        if let Some(edits) = change.get("edits").and_then(Value::as_array) {
            check_text_edits(edits, out);
        }
    }
}

fn check_locations(result: &Value, out: &mut Violations) {
    let single = [result.clone()];
    let items = result.as_array().map_or(&single[..], Vec::as_slice);
    for item in items {
        if item.is_null() {
            continue;
        }
        // `LocationLink` and `Location` name their ranges differently.
        if let Some(target) = item.get("targetRange") {
            let target = range_of(Some(target), out);
            let selection = range_of(item.get("targetSelectionRange"), out);
            if let (Some(target), Some(selection)) = (target, selection)
                && !contains(target, selection)
            {
                fail(
                    out,
                    "link-selection-outside-range",
                    format!("{selection:?} not in {target:?}"),
                );
            }
        } else {
            range_of(item.get("range"), out);
        }
    }
}

fn check_call_item(item: &Value, out: &mut Violations) {
    let range = range_of(item.get("range"), out);
    let selection = range_of(item.get("selectionRange"), out);
    if let (Some(range), Some(selection)) = (range, selection)
        && !contains(range, selection)
    {
        fail(
            out,
            "call-item-selection-outside-range",
            format!("{selection:?} not in {range:?}"),
        );
    }
}

/// Check one response. `params` are the request's parameters and `document` the
/// text of the document it was about, when known.
pub fn check_response(
    method: &str,
    params: Option<&Value>,
    result: &Value,
    document: Option<&str>,
    encoding: PositionEncoding,
) -> Vec<Violation> {
    let mut out = Vec::new();
    if result.is_null() {
        return out;
    }
    let doc = document.and_then(|text| DocLines::new(text, encoding));
    let doc = doc.as_ref();
    let items = result.as_array().map_or(&[][..], Vec::as_slice);

    match method {
        "textDocument/documentSymbol" => check_document_symbols(items, doc, &mut out),
        "textDocument/selectionRange" => check_selection_ranges(items, params, doc, &mut out),
        "textDocument/foldingRange" => check_folding_ranges(items, doc, &mut out),
        "textDocument/semanticTokens/full"
        | "textDocument/semanticTokens/full/delta"
        | "textDocument/semanticTokens/range" => check_semantic_tokens(result, doc, &mut out),
        "textDocument/documentHighlight" | "textDocument/documentLink" => {
            for item in items {
                if let Some(range) = range_of(item.get("range"), &mut out) {
                    check_in_doc(range, doc, &mut out);
                }
            }
        }
        "textDocument/formatting" => check_text_edits(items, &mut out),
        "textDocument/rename"
        | "workspace/willRenameFiles"
        | "workspace/willCreateFiles"
        | "workspace/willDeleteFiles" => check_workspace_edit(result, &mut out),
        "textDocument/definition"
        | "textDocument/declaration"
        | "textDocument/implementation"
        | "textDocument/references" => check_locations(result, &mut out),
        "textDocument/prepareCallHierarchy" => {
            for item in items {
                check_call_item(item, &mut out);
            }
        }
        "callHierarchy/incomingCalls" | "callHierarchy/outgoingCalls" => {
            for call in items {
                check_call_item(
                    call.get("from").or(call.get("to")).unwrap_or(call),
                    &mut out,
                );
                let from_ranges = call.get("fromRanges").and_then(Value::as_array);
                for range in from_ranges.into_iter().flatten() {
                    range_of(Some(range), &mut out);
                }
            }
        }
        _ => {}
    }
    out
}

/// The buffers the client has open, tracked from its notifications so the
/// checker sees what the client sees without reaching into the server.
#[derive(Default)]
struct OpenDocs(HashMap<String, TrackedDoc>);

struct TrackedDoc {
    version: u64,
    text: Arc<str>,
    previous: Option<Arc<str>>,
}

impl OpenDocs {
    fn observe(&mut self, method: &str, params: Option<&Value>) {
        let Some(params) = params else { return };
        let uri = params
            .pointer("/textDocument/uri")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let Some(uri) = uri else { return };
        match method {
            "textDocument/didOpen" => {
                if let Some(text) = params.pointer("/textDocument/text").and_then(Value::as_str) {
                    self.replace(uri, text);
                }
            }
            "textDocument/didChange" => {
                // The server negotiates full sync, so the change is the document.
                if let Some(text) = params
                    .pointer("/contentChanges/0/text")
                    .and_then(Value::as_str)
                {
                    self.replace(uri, text);
                }
            }
            "textDocument/didClose" => {
                self.0.remove(&uri);
            }
            _ => {}
        }
    }

    fn replace(&mut self, uri: String, text: &str) {
        let doc = self.0.entry(uri).or_insert_with(|| TrackedDoc {
            version: 0,
            text: Arc::from(""),
            previous: None,
        });
        doc.previous = Some(std::mem::replace(&mut doc.text, Arc::from(text)));
        doc.version += 1;
    }
}

fn request_uri(params: Option<&Value>) -> Option<String> {
    params?
        .pointer("/textDocument/uri")?
        .as_str()
        .map(str::to_owned)
}

/// Middleware running [`check_response`] on every response of `S`.
pub struct InvariantService<S> {
    inner: S,
    docs: Arc<Mutex<OpenDocs>>,
    encoding: fn() -> PositionEncoding,
}

impl<S> InvariantService<S> {
    pub fn new(inner: S) -> Self {
        Self::with_encoding(inner, utils::encoding)
    }

    fn with_encoding(inner: S, encoding: fn() -> PositionEncoding) -> Self {
        Self {
            inner,
            docs: Arc::default(),
            encoding,
        }
    }
}

impl<S> Service<Request> for InvariantService<S>
where
    S: Service<Request, Response = Option<Response>>,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = Option<Response>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let method = request.method().to_owned();
        let params = request.params().cloned();
        let uri = request_uri(params.as_ref());
        let snapshot = {
            let mut docs = self.docs.lock().expect("invariant doc lock poisoned");
            docs.observe(&method, params.as_ref());
            uri.as_ref()
                .and_then(|uri| docs.0.get(uri))
                .map(|doc| (doc.version, doc.text.clone(), doc.previous.clone()))
        };
        let docs = self.docs.clone();
        let encoding = (self.encoding)();
        let future = self.inner.call(request);

        Box::pin(async move {
            let response = future.await?;
            let Some(response) = response else {
                return Ok(None);
            };
            let Some(result) = response.result() else {
                return Ok(Some(response));
            };

            let (document, previous) = match snapshot {
                Some((version, text, previous)) => {
                    let current = docs
                        .lock()
                        .expect("invariant doc lock poisoned")
                        .0
                        .get(uri.as_deref().unwrap_or_default())
                        .map(|doc| doc.version);
                    // Edited while the handler ran: it may have seen either text.
                    if current != Some(version) {
                        return Ok(Some(response));
                    }
                    (Some(text), previous)
                }
                None => (None, None),
            };

            let violations = check_response(
                &method,
                params.as_ref(),
                result,
                document.as_deref(),
                encoding,
            );
            if violations.is_empty() {
                return Ok(Some(response));
            }
            // The handler may have started just before the latest edit landed
            // in its own text cache, in which case the older text is correct.
            let matches_previous = previous.is_some_and(|text| {
                check_response(&method, params.as_ref(), result, Some(&text), encoding).is_empty()
            });
            if matches_previous {
                return Ok(Some(response));
            }

            let message = violations
                .iter()
                .map(|v| format!("{}: {}", v.rule, v.detail))
                .collect::<Vec<_>>()
                .join("; ");
            // One write, so editors that log stderr by chunk keep it on one line.
            let target = uri.as_deref().unwrap_or("no document");
            let report = format!("[invariant] {method} {target}: {message}\n");
            let _ = std::io::stderr().write_all(report.as_bytes());
            Ok(Some(Response::from_error(
                response.id().clone(),
                Error {
                    code: ErrorCode::InternalError,
                    message: format!("invariant violated in {method}: {message}").into(),
                    data: None,
                },
            )))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::future::{Ready, ready};

    const ENC: PositionEncoding = PositionEncoding::Utf16;

    fn rules(method: &str, params: Option<Value>, result: Value, doc: &str) -> Vec<&'static str> {
        check_response(method, params.as_ref(), &result, Some(doc), ENC)
            .into_iter()
            .map(|v| v.rule)
            .collect()
    }

    fn range(a: u64, b: u64, c: u64, d: u64) -> Value {
        json!({"start": {"line": a, "character": b}, "end": {"line": c, "character": d}})
    }

    #[test]
    fn selection_outside_symbol_range_is_rejected() {
        let symbol = |selection| json!([{"name": "x", "kind": 12, "range": range(0, 0, 0, 3), "selectionRange": selection}]);
        let method = "textDocument/documentSymbol";
        assert!(rules(method, None, symbol(range(0, 1, 0, 2)), "abc").is_empty());
        assert_eq!(
            rules(method, None, symbol(range(0, 1, 0, 4)), "abcdef"),
            ["symbol-selection-outside-range"]
        );
    }

    #[test]
    fn nested_symbols_are_checked() {
        let result = json!([{
            "range": range(0, 0, 0, 3), "selectionRange": range(0, 0, 0, 1),
            "children": [{"range": range(0, 2, 0, 1), "selectionRange": range(0, 2, 0, 2)}],
        }]);
        assert_eq!(
            rules("textDocument/documentSymbol", None, result, "abc"),
            ["range-inverted"]
        );
    }

    #[test]
    fn selection_range_rules() {
        let method = "textDocument/selectionRange";
        let params = || Some(json!({"positions": [{"line": 0, "character": 0}]}));
        let good = json!([{"range": range(0, 0, 0, 1), "parent": {"range": range(0, 0, 0, 3)}}]);
        assert!(rules(method, params(), good, "abc").is_empty());
        let excluded =
            json!([{"range": range(0, 0, 0, 2), "parent": {"range": range(0, 1, 0, 3)}}]);
        assert_eq!(
            rules(method, params(), excluded, "abc"),
            ["selection-parent-excludes-child"]
        );
        assert_eq!(
            rules(method, params(), json!([]), "abc"),
            ["selection-count-mismatch"]
        );
    }

    #[test]
    fn folding_rules() {
        let method = "textDocument/foldingRange";
        assert!(
            rules(
                method,
                None,
                json!([{"startLine": 0, "endLine": 1}]),
                "a\nb"
            )
            .is_empty()
        );
        assert_eq!(
            rules(
                method,
                None,
                json!([{"startLine": 1, "endLine": 0}]),
                "a\nb"
            ),
            ["range-inverted"]
        );
        assert_eq!(
            rules(
                method,
                None,
                json!([{"startLine": 0, "endLine": 2}]),
                "a\nb"
            ),
            ["range-outside-document"]
        );
    }

    #[test]
    fn token_rules() {
        let method = "textDocument/semanticTokens/full";
        let check = |data: Value, doc: &str| rules(method, None, json!({"data": data}), doc);
        assert!(check(json!([0, 0, 2, 0, 0, 0, 3, 1, 0, 0]), "abcd").is_empty());
        assert_eq!(check(json!([0, 0, 2, 0]), "ab"), ["tokens-malformed"]);
        assert_eq!(
            check(json!([0, 0, 2, 999, 0]), "ab"),
            ["token-type-outside-legend"]
        );
        assert_eq!(
            check(json!([0, 0, 2, 0, 0, 0, 1, 1, 0, 0]), "abc"),
            ["tokens-overlap"]
        );
        // One UTF-16 unit long, so a length of 2 runs off the line.
        assert_eq!(
            check(json!([0, 0, 2, 0, 0]), "é"),
            ["range-outside-document"]
        );
    }

    #[test]
    fn token_bounds_follow_the_encoding() {
        let result = json!({"data": [0, 0, 2, 0, 0]});
        let method = "textDocument/semanticTokens/full";
        let utf8 = check_response(method, None, &result, Some("é"), PositionEncoding::Utf8);
        assert!(utf8.is_empty());
    }

    #[test]
    fn delta_must_align_to_tokens() {
        let result = json!({"edits": [{"start": 3, "deleteCount": 5}]});
        assert_eq!(
            rules("textDocument/semanticTokens/full/delta", None, result, ""),
            ["token-delta-misaligned"]
        );
    }

    #[test]
    fn workspace_edits_must_not_overlap() {
        let edit = |r| json!({"range": r, "newText": "x"});
        let touching =
            json!({"changes": {"file:///a": [edit(range(0, 0, 0, 2)), edit(range(0, 2, 0, 3))]}});
        assert!(rules("textDocument/rename", None, touching, "").is_empty());
        let overlapping = json!({"documentChanges": [{"edits": [edit(range(0, 0, 0, 2)), edit(range(0, 1, 0, 3))]}]});
        assert_eq!(
            rules("textDocument/rename", None, overlapping, ""),
            ["text-edits-overlap"]
        );
    }

    #[test]
    fn locations_and_call_items() {
        assert_eq!(
            rules(
                "textDocument/references",
                None,
                json!([{"range": range(1, 0, 0, 0)}]),
                ""
            ),
            ["range-inverted"]
        );
        let link =
            json!({"targetRange": range(0, 0, 0, 1), "targetSelectionRange": range(0, 0, 0, 2)});
        assert_eq!(
            rules("textDocument/definition", None, link, ""),
            ["link-selection-outside-range"]
        );
        let item = json!({"range": range(0, 0, 0, 1), "selectionRange": range(0, 0, 0, 2)});
        assert_eq!(
            rules("textDocument/prepareCallHierarchy", None, json!([item]), ""),
            ["call-item-selection-outside-range"]
        );
        assert_eq!(
            rules(
                "callHierarchy/incomingCalls",
                None,
                json!([{"from": item, "fromRanges": []}]),
                ""
            ),
            ["call-item-selection-outside-range"]
        );
        assert_eq!(
            rules(
                "callHierarchy/outgoingCalls",
                None,
                json!([{"to": item, "fromRanges": []}]),
                ""
            ),
            ["call-item-selection-outside-range"]
        );
    }

    #[test]
    fn crlf_lines_exclude_the_line_ending() {
        let method = "textDocument/documentHighlight";
        assert!(
            rules(
                method,
                None,
                json!([{"range": range(0, 0, 0, 2)}]),
                "ab\r\ncd"
            )
            .is_empty()
        );
        assert_eq!(
            rules(
                method,
                None,
                json!([{"range": range(0, 0, 0, 3)}]),
                "ab\r\ncd"
            ),
            ["range-outside-document"]
        );
    }

    #[test]
    fn bare_cr_documents_skip_bounds_checks() {
        let result = json!([{"range": range(9, 0, 9, 9)}]);
        assert!(rules("textDocument/documentHighlight", None, result, "a\rb").is_empty());
    }

    /// Returns a fixed result for requests, nothing for notifications.
    struct Canned(Value);

    impl Service<Request> for Canned {
        type Response = Option<Response>;
        type Error = std::convert::Infallible;
        type Future = Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, request: Request) -> Self::Future {
            let response = request
                .id()
                .cloned()
                .map(|id| Response::from_ok(id, self.0.clone()));
            ready(Ok(response))
        }
    }

    fn notify(method: &'static str, params: Value) -> Request {
        Request::build(method).params(params).finish()
    }

    fn highlight_request() -> Request {
        Request::build("textDocument/documentHighlight")
            .id(1)
            .params(json!({"textDocument": {"uri": "file:///a.sol"}}))
            .finish()
    }

    #[tokio::test]
    async fn service_turns_violations_into_errors() {
        let bad = json!([{"range": range(5, 0, 5, 1)}]);
        let mut service = InvariantService::with_encoding(Canned(bad), PositionEncoding::default);
        service
            .call(notify(
                "textDocument/didOpen",
                json!({"textDocument": {"uri": "file:///a.sol", "text": "abc"}}),
            ))
            .await
            .unwrap();

        let response = service.call(highlight_request()).await.unwrap().unwrap();
        let message = response
            .error()
            .expect("violation reported")
            .message
            .clone();
        assert!(message.contains("range-outside-document"), "{message}");
    }

    #[tokio::test]
    async fn service_passes_valid_responses_through() {
        let good = json!([{"range": range(0, 0, 0, 3)}]);
        let mut service =
            InvariantService::with_encoding(Canned(good.clone()), PositionEncoding::default);
        service
            .call(notify(
                "textDocument/didOpen",
                json!({"textDocument": {"uri": "file:///a.sol", "text": "abc"}}),
            ))
            .await
            .unwrap();
        let response = service.call(highlight_request()).await.unwrap().unwrap();
        assert_eq!(response.result(), Some(&good));
    }

    #[tokio::test]
    async fn service_tracks_changes_and_accepts_the_previous_text() {
        let result = json!([{"range": range(0, 0, 0, 6)}]);
        let mut service =
            InvariantService::with_encoding(Canned(result), PositionEncoding::default);
        let change = |text: &str| {
            notify(
                "textDocument/didChange",
                json!({"textDocument": {"uri": "file:///a.sol"}, "contentChanges": [{"text": text}]}),
            )
        };
        service.call(change("abcdef")).await.unwrap();
        service.call(change("abc")).await.unwrap();
        // Valid for the text before the last edit, so it may be a benign race.
        let response = service.call(highlight_request()).await.unwrap().unwrap();
        assert!(response.is_ok());
        // Valid for neither text once the older one is gone.
        service.call(change("a")).await.unwrap();
        let response = service.call(highlight_request()).await.unwrap().unwrap();
        assert!(response.is_error());
    }

    #[tokio::test]
    async fn service_stops_checking_bounds_after_close() {
        let result = json!([{"range": range(9, 0, 9, 1)}]);
        let mut service =
            InvariantService::with_encoding(Canned(result), PositionEncoding::default);
        let doc = json!({"textDocument": {"uri": "file:///a.sol", "text": "abc"}});
        service
            .call(notify("textDocument/didOpen", doc.clone()))
            .await
            .unwrap();
        service
            .call(notify("textDocument/didClose", doc))
            .await
            .unwrap();
        let response = service.call(highlight_request()).await.unwrap().unwrap();
        assert!(response.is_ok());
    }
}
