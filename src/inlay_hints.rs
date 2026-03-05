use crate::gas;
use crate::goto::CachedBuild;
use crate::types::{AbsPath, NodeId, SourceLoc};
use serde_json::Value;
use std::collections::HashMap;
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Parser};

/// Where to place gas inlay hints on function definitions.
/// Contracts always use the opening brace.
#[allow(dead_code)]
enum FnGasHintPosition {
    /// Show after the opening `{` brace.
    Opening,
    /// Show after the closing `}` brace.
    Closing,
}

/// Change this to switch function gas hint placement.
const FN_GAS_HINT_POSITION: FnGasHintPosition = FnGasHintPosition::Closing;

/// Parameter info resolved from the AST for a callable.
#[derive(Debug, Clone)]
struct ParamInfo {
    /// Parameter names from the declaration.
    names: Vec<String>,
    /// Number of leading params to skip (1 for using-for library calls).
    skip: usize,
}

/// Call-site info extracted from the AST, keyed by source byte offset.
#[derive(Debug, Clone)]
struct CallSite {
    /// The resolved parameter info for this specific call.
    info: ParamInfo,
    /// Function/event name (for matching with tree-sitter).
    name: String,
    /// The AST node id of the called function/event declaration (for DocIndex lookup).
    decl_id: NodeId,
}

/// Resolved callsite info returned to hover for param doc lookup.
#[derive(Debug, Clone)]
pub struct ResolvedCallSite {
    /// The parameter name at the given argument index.
    pub param_name: String,
    /// The AST node id of the called function/event declaration.
    pub decl_id: NodeId,
}

/// Both lookup strategies: exact byte-offset match and (name, arg_count) fallback.
/// Built once per file when the AST is cached, reused on every inlay hint request.
#[derive(Debug, Clone)]
pub struct HintLookup {
    /// Primary: byte_offset → CallSite (exact match when AST offsets are fresh).
    by_offset: HashMap<usize, CallSite>,
    /// Fallback: (name, arg_count) → CallSite (works even with stale offsets).
    by_name: HashMap<(String, usize), CallSite>,
}

impl HintLookup {
    /// Resolve a callsite to get the declaration id and skip count.
    ///
    /// Returns `(decl_id, skip)` where skip is the number of leading
    /// parameters to skip (1 for `using-for` library calls, 0 otherwise).
    ///
    /// For signature help, this uses a relaxed lookup: exact offset first,
    /// then `(name, arg_count)`, then **name-only** fallback (ignoring arg
    /// count, since the user may still be typing arguments).
    pub fn resolve_callsite_with_skip(
        &self,
        call_offset: usize,
        func_name: &str,
        arg_count: usize,
    ) -> Option<(NodeId, usize)> {
        // Try exact match first
        if let Some(site) = lookup_call_site(self, call_offset, func_name, arg_count) {
            return Some((site.decl_id, site.info.skip));
        }
        // Fallback: match by name only (any arg count)
        self.by_name
            .iter()
            .find(|((name, _), _)| name == func_name)
            .map(|(_, site)| (site.decl_id, site.info.skip))
    }

    /// Resolve callsite parameter info for hover.
    ///
    /// Given a call's byte offset (from tree-sitter), the function name,
    /// the argument count, and the 0-based argument index, returns a
    /// `ResolvedCallSite` with the parameter name and declaration id.
    pub fn resolve_callsite_param(
        &self,
        call_offset: usize,
        func_name: &str,
        arg_count: usize,
        arg_index: usize,
    ) -> Option<ResolvedCallSite> {
        let site = lookup_call_site(self, call_offset, func_name, arg_count)?;
        let param_idx = arg_index + site.info.skip;
        if param_idx >= site.info.names.len() {
            return None;
        }
        let param_name = &site.info.names[param_idx];
        if param_name.is_empty() {
            return None;
        }
        Some(ResolvedCallSite {
            param_name: param_name.clone(),
            decl_id: site.decl_id,
        })
    }
}

/// Pre-computed hint lookups for all files, keyed by absolutePath.
/// Built once in `CachedBuild::new()`, reused on every inlay hint request.
pub type HintIndex = HashMap<AbsPath, HintLookup>;

/// Build the hint index for all files from the AST sources.
/// Pre-computed constructor info for `new ContractName(args)` resolution.
/// Maps contract declaration ID → (constructor declaration ID, contract name, param names).
#[derive(Debug, Clone)]
pub struct ConstructorInfo {
    pub constructor_id: NodeId,
    pub contract_name: String,
    pub param_names: Vec<String>,
}

/// Index of contract ID → constructor info, built from the typed `decl_index`.
pub type ConstructorIndex = HashMap<NodeId, ConstructorInfo>;

/// Build the constructor index from the `decl_index`.
///
/// For every constructor in the decl_index, record contract ID → constructor info.
pub fn build_constructor_index(
    decl_index: &HashMap<NodeId, crate::solc_ast::DeclNode>,
) -> ConstructorIndex {
    let mut index = HashMap::with_capacity(decl_index.len() / 8);
    for (id, decl) in decl_index {
        if decl.is_constructor()
            && let Some(scope) = decl.scope()
        {
            let scope = NodeId(scope);
            let names = decl.param_names().unwrap_or_default();
            // Look up contract name from the scope's decl entry
            let contract_name = decl_index
                .get(&scope)
                .map(|d| d.name().to_string())
                .unwrap_or_default();
            index.insert(
                scope,
                ConstructorInfo {
                    constructor_id: *id,
                    contract_name,
                    param_names: names,
                },
            );
        }
    }
    index
}

/// Called once in `CachedBuild::new()`.
///
/// Accepts the typed `decl_index` and `constructor_index` for O(1) param name
/// resolution, replacing the previous `id_index: HashMap<u64, Value>`.
pub fn build_hint_index(
    sources: &Value,
    decl_index: &HashMap<NodeId, crate::solc_ast::DeclNode>,
    constructor_index: &ConstructorIndex,
) -> HintIndex {
    let source_count = sources.as_object().map_or(0, |obj| obj.len());
    let mut hint_index = HashMap::with_capacity(source_count);

    if let Some(obj) = sources.as_object() {
        for (_, source_data) in obj {
            if let Some(ast) = source_data.get("ast")
                && let Some(abs_path) = ast.get("absolutePath").and_then(|v| v.as_str())
            {
                let lookup = build_hint_lookup(ast, decl_index, constructor_index);
                hint_index.insert(AbsPath::new(abs_path), lookup);
            }
        }
    }

    hint_index
}

/// Generate inlay hints for a given range of source.
///
/// Uses tree-sitter on the **live buffer** for argument positions (so hints
/// follow edits in real time) and the pre-cached hint index for semantic
/// info (parameter names via `referencedDeclaration`).
pub fn inlay_hints(
    build: &CachedBuild,
    uri: &Url,
    range: Range,
    live_source: &[u8],
) -> Vec<InlayHint> {
    let path_str = match uri.to_file_path() {
        Ok(p) => p.to_str().unwrap_or("").to_string(),
        Err(_) => return vec![],
    };

    let abs = match build
        .path_to_abs
        .iter()
        .find(|(k, _)| path_str.ends_with(k.as_str()))
    {
        Some((_, v)) => v.clone(),
        None => return vec![],
    };

    // Use the pre-cached hint lookup for this file
    let lookup = match build.hint_index.get(&abs) {
        Some(l) => l,
        None => return vec![],
    };

    // Walk tree-sitter on the live buffer for real-time argument positions
    let source_str = String::from_utf8_lossy(live_source);
    let tree = match ts_parse(&source_str) {
        Some(t) => t,
        None => return vec![],
    };

    let mut hints = Vec::new();
    collect_ts_hints(tree.root_node(), &source_str, &range, lookup, &mut hints);

    // Gas inlay hints: use tree-sitter positions (tracks live buffer)
    if !build.gas_index.is_empty() {
        collect_ts_gas_hints(
            tree.root_node(),
            &source_str,
            &range,
            &build.gas_index,
            &abs,
            &mut hints,
        );
    }

    hints
}

/// Parse Solidity source with tree-sitter.
pub fn ts_parse(source: &str) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_solidity::LANGUAGE.into())
        .expect("failed to load Solidity grammar");
    parser.parse(source, None)
}

/// Build both lookup strategies from the AST.
fn build_hint_lookup(
    file_ast: &Value,
    decl_index: &HashMap<NodeId, crate::solc_ast::DeclNode>,
    constructor_index: &ConstructorIndex,
) -> HintLookup {
    let mut lookup = HintLookup {
        by_offset: HashMap::new(),
        by_name: HashMap::new(),
    };
    collect_ast_calls(file_ast, decl_index, constructor_index, &mut lookup);
    lookup
}

/// Parse the `src` field ("offset:length:fileId") and return the byte offset.
fn parse_src_offset(node: &Value) -> Option<usize> {
    let src = node.get("src").and_then(|v| v.as_str())?;
    SourceLoc::parse(src).map(|loc| loc.offset)
}

/// Recursively walk AST nodes collecting call site info.
fn collect_ast_calls(
    node: &Value,
    decl_index: &HashMap<NodeId, crate::solc_ast::DeclNode>,
    constructor_index: &ConstructorIndex,
    lookup: &mut HintLookup,
) {
    let node_type = node.get("nodeType").and_then(|v| v.as_str()).unwrap_or("");

    match node_type {
        "FunctionCall" => {
            if let Some(call_info) = extract_call_info(node, decl_index, constructor_index) {
                let arg_count = node
                    .get("arguments")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let site = CallSite {
                    info: ParamInfo {
                        names: call_info.params.names,
                        skip: call_info.params.skip,
                    },
                    name: call_info.name,
                    decl_id: call_info.decl_id,
                };
                if let Some(offset) = parse_src_offset(node) {
                    lookup.by_offset.insert(offset, site.clone());
                }

                lookup
                    .by_name
                    .entry((site.name.clone(), arg_count))
                    .or_insert(site);
            }
        }
        "EmitStatement" => {
            if let Some(event_call) = node.get("eventCall")
                && let Some(call_info) =
                    extract_call_info(event_call, decl_index, constructor_index)
            {
                let arg_count = event_call
                    .get("arguments")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let site = CallSite {
                    info: ParamInfo {
                        names: call_info.params.names,
                        skip: call_info.params.skip,
                    },
                    name: call_info.name,
                    decl_id: call_info.decl_id,
                };
                if let Some(offset) = parse_src_offset(node) {
                    lookup.by_offset.insert(offset, site.clone());
                }

                lookup
                    .by_name
                    .entry((site.name.clone(), arg_count))
                    .or_insert(site);
            }
        }
        _ => {}
    }

    // Recurse into children
    for key in crate::goto::CHILD_KEYS {
        if let Some(child) = node.get(*key) {
            if child.is_array() {
                if let Some(arr) = child.as_array() {
                    for item in arr {
                        collect_ast_calls(item, decl_index, constructor_index, lookup);
                    }
                }
            } else if child.is_object() {
                collect_ast_calls(child, decl_index, constructor_index, lookup);
            }
        }
    }
}

/// Resolved call info including the declaration id of the called function/event.
struct CallInfo {
    /// Function/event name.
    name: String,
    /// Parameter names and skip count.
    params: ParamInfo,
    /// The AST node id of the referenced declaration (for DocIndex lookup).
    decl_id: NodeId,
}

/// Extract function/event name and parameter info from an AST FunctionCall node.
fn extract_call_info(
    node: &Value,
    decl_index: &HashMap<NodeId, crate::solc_ast::DeclNode>,
    constructor_index: &ConstructorIndex,
) -> Option<CallInfo> {
    let args = node.get("arguments")?.as_array()?;
    if args.is_empty() {
        return None;
    }

    // Skip struct constructors with named args
    let kind = node.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    if kind == "structConstructorCall"
        && node
            .get("names")
            .and_then(|v| v.as_array())
            .is_some_and(|n| !n.is_empty())
    {
        return None;
    }

    let expr = node.get("expression")?;
    let expr_type = expr.get("nodeType").and_then(|v| v.as_str()).unwrap_or("");

    // Contract creation: `new Token(args)` — the expression is a NewExpression
    // whose typeName holds the referencedDeclaration pointing to the contract.
    // We resolve the constructor from the constructor_index.
    if expr_type == "NewExpression" {
        return extract_new_expression_call_info(expr, args.len(), constructor_index);
    }

    let decl_id = NodeId(expr.get("referencedDeclaration").and_then(|v| v.as_i64())?);

    let decl = decl_index.get(&decl_id)?;
    let names = decl.param_names()?;

    // Extract the function name from the expression
    let func_name = extract_function_name(expr)?;

    // Using-for library calls pass the receiver as the implicit first param,
    // so the AST has one fewer arg than the declaration has params.
    // Direct library calls (Transaction.addTax) and struct constructors
    // pass all params explicitly — arg count matches param count.
    let arg_count = node
        .get("arguments")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let skip = if is_member_access(expr) && arg_count < names.len() {
        1
    } else {
        0
    };

    Some(CallInfo {
        name: func_name,
        params: ParamInfo { names, skip },
        decl_id,
    })
}

/// Extract call info for a `new ContractName(args)` expression.
///
/// The AST represents this as a `FunctionCall` whose `expression` is a
/// `NewExpression`.  The `NewExpression` has a `typeName` with a
/// `referencedDeclaration` pointing to the `ContractDefinition`.  We resolve
/// the constructor via the pre-built `constructor_index`.
fn extract_new_expression_call_info(
    new_expr: &Value,
    _arg_count: usize,
    constructor_index: &ConstructorIndex,
) -> Option<CallInfo> {
    let type_name = new_expr.get("typeName")?;
    let contract_id = NodeId(
        type_name
            .get("referencedDeclaration")
            .and_then(|v| v.as_i64())?,
    );

    let info = constructor_index.get(&contract_id)?;

    // Prefer contract name from AST typeName path, fall back to constructor_index
    let contract_name = type_name
        .get("pathNode")
        .and_then(|p| p.get("name"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| info.contract_name.clone());

    Some(CallInfo {
        name: contract_name,
        params: ParamInfo {
            names: info.param_names.clone(),
            skip: 0,
        },
        decl_id: info.constructor_id,
    })
}

/// Extract the function/event name from an AST expression node.
fn extract_function_name(expr: &Value) -> Option<String> {
    let node_type = expr.get("nodeType").and_then(|v| v.as_str())?;
    match node_type {
        "Identifier" => expr.get("name").and_then(|v| v.as_str()).map(String::from),
        "MemberAccess" => expr
            .get("memberName")
            .and_then(|v| v.as_str())
            .map(String::from),
        "NewExpression" => expr
            .get("typeName")
            .and_then(|t| {
                t.get("pathNode")
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                    .or_else(|| t.get("name").and_then(|v| v.as_str()))
            })
            .map(String::from),
        _ => None,
    }
}

/// Check if expression is a MemberAccess (potential using-for call).
fn is_member_access(expr: &Value) -> bool {
    expr.get("nodeType")
        .and_then(|v| v.as_str())
        .is_some_and(|t| t == "MemberAccess")
}

// ── Tree-sitter walk ──────────────────────────────────────────────────────

/// Look up call site info: try exact byte-offset match first, fall back to (name, arg_count).
fn lookup_call_site<'a>(
    lookup: &'a HintLookup,
    offset: usize,
    name: &str,
    arg_count: usize,
) -> Option<&'a CallSite> {
    // Exact match by byte offset (works when AST is fresh)
    if let Some(site) = lookup.by_offset.get(&offset)
        && site.name == name
    {
        return Some(site);
    }
    // Fallback by (name, arg_count) (works with stale offsets after edits)
    lookup.by_name.get(&(name.to_string(), arg_count))
}

/// Recursively walk tree-sitter nodes, emitting hints for calls in the visible range.
fn collect_ts_hints(
    node: Node,
    source: &str,
    range: &Range,
    lookup: &HintLookup,
    hints: &mut Vec<InlayHint>,
) {
    // Quick range check — skip nodes entirely outside the visible range
    let node_start = node.start_position();
    let node_end = node.end_position();
    if (node_end.row as u32) < range.start.line || (node_start.row as u32) > range.end.line {
        return;
    }

    match node.kind() {
        "call_expression" => {
            emit_call_hints(node, source, lookup, hints);
        }
        "emit_statement" => {
            emit_emit_hints(node, source, lookup, hints);
        }
        _ => {}
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_ts_hints(child, source, range, lookup, hints);
    }
}

/// Emit parameter hints for a `call_expression` node.
fn emit_call_hints(node: Node, source: &str, lookup: &HintLookup, hints: &mut Vec<InlayHint>) {
    let func_name = match ts_call_function_name(node, source) {
        Some(n) => n,
        None => return,
    };

    let args = ts_call_arguments(node);
    if args.is_empty() {
        return;
    }

    let site = match lookup_call_site(lookup, node.start_byte(), func_name, args.len()) {
        Some(s) => s,
        None => return,
    };

    emit_param_hints(&args, &site.info, hints);
}

/// Emit parameter hints for an `emit_statement` node.
fn emit_emit_hints(node: Node, source: &str, lookup: &HintLookup, hints: &mut Vec<InlayHint>) {
    let event_name = match ts_emit_event_name(node, source) {
        Some(n) => n,
        None => return,
    };

    let args = ts_call_arguments(node);
    if args.is_empty() {
        return;
    }

    let site = match lookup_call_site(lookup, node.start_byte(), event_name, args.len()) {
        Some(s) => s,
        None => return,
    };

    emit_param_hints(&args, &site.info, hints);
}

/// Emit InlayHint items for each argument, using tree-sitter positions.
fn emit_param_hints(args: &[Node], info: &ParamInfo, hints: &mut Vec<InlayHint>) {
    for (i, arg) in args.iter().enumerate() {
        let pi = i + info.skip;
        if pi >= info.names.len() || info.names[pi].is_empty() {
            continue;
        }

        let start = arg.start_position();
        let position = Position::new(start.row as u32, start.column as u32);

        hints.push(InlayHint {
            position,
            kind: Some(InlayHintKind::PARAMETER),
            label: InlayHintLabel::String(format!("{}:", info.names[pi])),
            text_edits: None,
            tooltip: None,
            padding_left: None,
            padding_right: Some(true),
            data: None,
        });
    }
}

// ── Tree-sitter helpers ───────────────────────────────────────────────────

/// Get the function name from a `call_expression` node.
///
/// For `transfer(...)` → "transfer"
/// For `PRICE.addTax(...)` → "addTax"
/// For `router.swap{value: 100}(...)` → "swap"  (value modifier)
fn ts_call_function_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    let func_expr = node.child_by_field_name("function")?;
    // The expression wrapper has one named child
    let inner = first_named_child(func_expr)?;
    extract_name_from_expr(inner, source)
}

/// Recursively extract the function/event name from an expression node.
///
/// tree-sitter-solidity parses `foo{value: 100}(args)` as a `call_expression`
/// whose `function` field is a `struct_expression` (because the grammar lacks
/// `function_call_options_expression`). We handle this by drilling into the
/// struct_expression's `type` field to find the real function name.
fn extract_name_from_expr<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    match node.kind() {
        "identifier" => Some(&source[node.byte_range()]),
        "member_expression" => {
            let prop = node.child_by_field_name("property")?;
            Some(&source[prop.byte_range()])
        }
        "struct_expression" => {
            // foo{value: 100} → type field holds the actual callee expression
            let type_expr = node.child_by_field_name("type")?;
            extract_name_from_expr(type_expr, source)
        }
        "new_expression" => {
            // new Token(...) → the `name` field holds the type_name
            ts_new_expression_name(node, source)
        }
        "expression" => {
            // tree-sitter wraps many nodes in an `expression` wrapper — unwrap it
            let inner = first_named_child(node)?;
            extract_name_from_expr(inner, source)
        }
        _ => None,
    }
}

/// Get the event name from an `emit_statement` node.
fn ts_emit_event_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    let name_expr = node.child_by_field_name("name")?;
    let inner = first_named_child(name_expr)?;
    match inner.kind() {
        "identifier" => Some(&source[inner.byte_range()]),
        "member_expression" => {
            let prop = inner.child_by_field_name("property")?;
            Some(&source[prop.byte_range()])
        }
        _ => None,
    }
}

/// Get the contract name from a `new_expression` node.
///
/// For `new Token(...)` → "Token"
/// For `new Router(...)` → "Router"
fn ts_new_expression_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    let name_node = node.child_by_field_name("name")?;
    // The `name` field is a type_name; extract the identifier text from it
    if name_node.kind() == "user_defined_type" || name_node.kind() == "type_name" {
        // May have a single identifier child
        let mut cursor = name_node.walk();
        for child in name_node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Some(&source[child.byte_range()]);
            }
        }
        // Fallback: use the full text of the name node
        Some(&source[name_node.byte_range()])
    } else {
        Some(&source[name_node.byte_range()])
    }
}

/// Collect `call_argument` children from a node (works for both
/// `call_expression` and `emit_statement` since `_call_arguments` is hidden).
fn ts_call_arguments(node: Node) -> Vec<Node> {
    let mut args = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "call_argument" {
            args.push(child);
        }
    }
    args
}

/// Get the first named child of a node.
fn first_named_child(node: Node) -> Option<Node> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|c| c.is_named())
}

/// Result of finding the enclosing call site at a byte position via tree-sitter.
pub struct TsCallContext<'a> {
    /// The function/event name.
    pub name: &'a str,
    /// 0-based index of the argument the cursor is on.
    pub arg_index: usize,
    /// Total number of arguments in the call.
    pub arg_count: usize,
    /// Start byte of the call_expression/emit_statement node (for HintIndex lookup).
    pub call_start_byte: usize,
    /// True when context is a mapping/array index access (`name[key]`)
    /// rather than a function/event call (`name(args)`).
    pub is_index_access: bool,
}

/// Find the enclosing `call_expression` or `emit_statement` for a given byte
/// position in the live buffer using tree-sitter.
///
/// Returns `None` if the position is not inside a call argument.
pub fn ts_find_call_at_byte<'a>(
    root: tree_sitter::Node<'a>,
    source: &'a str,
    byte_pos: usize,
) -> Option<TsCallContext<'a>> {
    // Find the deepest node containing byte_pos
    let mut node = root.descendant_for_byte_range(byte_pos, byte_pos)?;

    // Walk up the tree to find a call_argument parent
    loop {
        if node.kind() == "call_argument" {
            break;
        }
        node = node.parent()?;
    }

    // The call_argument's parent should be the call_expression or emit_statement
    let call_node = node.parent()?;
    let args = ts_call_arguments(call_node);
    let arg_index = args.iter().position(|a| a.id() == node.id())?;

    match call_node.kind() {
        "call_expression" => {
            let name = ts_call_function_name(call_node, source)?;
            Some(TsCallContext {
                name,
                arg_index,
                arg_count: args.len(),
                call_start_byte: call_node.start_byte(),
                is_index_access: false,
            })
        }
        "emit_statement" => {
            let name = ts_emit_event_name(call_node, source)?;
            Some(TsCallContext {
                name,
                arg_index,
                arg_count: args.len(),
                call_start_byte: call_node.start_byte(),
                is_index_access: false,
            })
        }
        _ => None,
    }
}

/// Find the enclosing call for signature help at a byte position.
///
/// Unlike `ts_find_call_at_byte`, this handles:
/// - Cursor right after `(` with no arguments yet
/// - Cursor between `,` and next argument
/// - Incomplete calls without closing `)`
///
/// Falls back to text-based scanning when tree-sitter can't produce a
/// `call_expression` (e.g. broken syntax during typing).
pub fn ts_find_call_for_signature<'a>(
    root: tree_sitter::Node<'a>,
    source: &'a str,
    byte_pos: usize,
) -> Option<TsCallContext<'a>> {
    // First try the normal path (cursor is on an argument)
    if let Some(ctx) = ts_find_call_at_byte(root, source, byte_pos) {
        return Some(ctx);
    }

    // Walk up from the deepest node looking for a call_expression or array_access
    let mut node = root.descendant_for_byte_range(byte_pos, byte_pos)?;
    loop {
        match node.kind() {
            "call_expression" => {
                let name = ts_call_function_name(node, source)?;
                let arg_index = count_commas_before(source, node.start_byte(), byte_pos);
                let args = ts_call_arguments(node);
                let arg_count = args.len().max(arg_index + 1);
                return Some(TsCallContext {
                    name,
                    arg_index,
                    arg_count,
                    call_start_byte: node.start_byte(),
                    is_index_access: false,
                });
            }
            "emit_statement" => {
                let name = ts_emit_event_name(node, source)?;
                let arg_index = count_commas_before(source, node.start_byte(), byte_pos);
                let args = ts_call_arguments(node);
                let arg_count = args.len().max(arg_index + 1);
                return Some(TsCallContext {
                    name,
                    arg_index,
                    arg_count,
                    call_start_byte: node.start_byte(),
                    is_index_access: false,
                });
            }
            "array_access" => {
                // Mapping/array index access: `name[key]`
                let base_node = node.child_by_field_name("base")?;
                // For member_expression (e.g. self.orders), use property name;
                // for plain identifier, use the identifier text.
                let name_node = if base_node.kind() == "member_expression" {
                    base_node
                        .child_by_field_name("property")
                        .unwrap_or(base_node)
                } else {
                    base_node
                };
                let name = &source[name_node.byte_range()];
                return Some(TsCallContext {
                    name,
                    arg_index: 0,
                    arg_count: 1,
                    call_start_byte: node.start_byte(),
                    is_index_access: true,
                });
            }
            "source_file" => break,
            _ => {
                node = node.parent()?;
            }
        }
    }

    // Fallback: scan backwards from cursor for `identifier(` pattern
    if let Some(ctx) = find_call_by_text_scan(source, byte_pos) {
        return Some(ctx);
    }

    // Fallback: scan backwards for `identifier[` (mapping/array access)
    find_index_by_text_scan(source, byte_pos)
}

/// Scan backwards from `byte_pos` to find an enclosing `name(` call.
///
/// Looks for the nearest unmatched `(` before the cursor, then extracts
/// the function name preceding it. Counts commas at depth 1 to determine
/// the active argument index.
fn find_call_by_text_scan<'a>(source: &'a str, byte_pos: usize) -> Option<TsCallContext<'a>> {
    let before = &source[..byte_pos.min(source.len())];

    // Find the nearest unmatched `(` by scanning backwards
    let mut depth: i32 = 0;
    let mut paren_pos = None;
    for (i, ch) in before.char_indices().rev() {
        match ch {
            ')' => depth += 1,
            '(' => {
                if depth == 0 {
                    paren_pos = Some(i);
                    break;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    let paren_pos = paren_pos?;

    // Extract the function name before the `(`
    // If there's a `{...}` block before the paren (e.g. `swap{value: 100}(`),
    // skip over it to find the real function name.
    let mut scan_end = paren_pos;
    let before_paren = source[..scan_end].trim_end();
    if before_paren.ends_with('}') {
        // Skip the `{...}` block by finding the matching `{`
        let mut brace_depth: i32 = 0;
        for (i, ch) in before_paren.char_indices().rev() {
            match ch {
                '}' => brace_depth += 1,
                '{' => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        scan_end = i;
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    let before_name = &source[..scan_end];
    let name_end = before_name.trim_end().len();
    let name_start = before_name[..name_end]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
        .map(|i| i + 1)
        .unwrap_or(0);
    // For member expressions like `router.swap`, take only the part after the last dot
    let raw_name = &source[name_start..name_end];
    let name = match raw_name.rfind('.') {
        Some(dot) => &raw_name[dot + 1..],
        None => raw_name,
    };

    if name.is_empty() || !name.chars().next()?.is_alphabetic() {
        return None;
    }

    // Count commas between `(` and cursor at depth 0
    let arg_index = count_commas_before(source, paren_pos, byte_pos);

    Some(TsCallContext {
        name,
        arg_index,
        arg_count: arg_index + 1,
        call_start_byte: name_start,
        is_index_access: false,
    })
}

/// Scan backwards from `byte_pos` to find an enclosing `name[` index access.
///
/// Similar to `find_call_by_text_scan` but for `[` brackets instead of `(`.
/// Returns a context with `is_index_access = true`.
fn find_index_by_text_scan<'a>(source: &'a str, byte_pos: usize) -> Option<TsCallContext<'a>> {
    let before = &source[..byte_pos.min(source.len())];

    // Find the nearest unmatched `[` by scanning backwards
    let mut depth: i32 = 0;
    let mut bracket_pos = None;
    for (i, c) in before.char_indices().rev() {
        match c {
            ']' => depth += 1,
            '[' => {
                if depth == 0 {
                    bracket_pos = Some(i);
                    break;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    let bracket_pos = bracket_pos?;

    // Extract the identifier name before the `[`
    let before_bracket = &source[..bracket_pos];
    let name_end = before_bracket.trim_end().len();
    let name_start = before_bracket[..name_end]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);
    let name = &source[name_start..name_end];

    if name.is_empty() || !name.chars().next()?.is_alphabetic() {
        return None;
    }

    Some(TsCallContext {
        name,
        arg_index: 0,
        arg_count: 1,
        call_start_byte: name_start,
        is_index_access: true,
    })
}

/// Count commas at depth 1 between `start` and `byte_pos` to determine argument index.
fn count_commas_before(source: &str, start: usize, byte_pos: usize) -> usize {
    let end = byte_pos.min(source.len());
    let text = &source[start..end];

    let mut count = 0;
    let mut depth = 0;
    let mut found_open = false;
    for ch in text.chars() {
        match ch {
            '(' if !found_open => {
                found_open = true;
                depth = 1;
            }
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if found_open && depth == 1 => count += 1,
            _ => {}
        }
    }
    count
}

// ── Gas inlay hints (tree-sitter based) ──────────────────────────────────

/// Walk tree-sitter nodes for function/contract definitions, emitting gas
/// cost hints using **live buffer positions** so they track edits in real time.
fn collect_ts_gas_hints(
    node: Node,
    source: &str,
    range: &Range,
    gas_index: &gas::GasIndex,
    abs_path: &str,
    hints: &mut Vec<InlayHint>,
) {
    let node_start = node.start_position();
    let node_end = node.end_position();
    if (node_end.row as u32) < range.start.line || (node_start.row as u32) > range.end.line {
        return;
    }

    match node.kind() {
        "function_definition" => {
            if let Some(hint) = ts_gas_hint_for_function(node, source, range, gas_index, abs_path) {
                hints.push(hint);
            }
        }
        "contract_declaration" | "library_declaration" | "interface_declaration" => {
            if let Some(hint) = ts_gas_hint_for_contract(node, source, range, gas_index, abs_path) {
                hints.push(hint);
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_ts_gas_hints(child, source, range, gas_index, abs_path, hints);
    }
}

/// Extract the identifier (name) child from a tree-sitter node.
fn ts_node_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|c| c.kind() == "identifier" && c.is_named())
        .map(|c| &source[c.byte_range()])
}

/// Find the opening `{` position of a body node.
fn ts_body_open_brace(node: Node, body_kind: &str) -> Option<Position> {
    let mut cursor = node.walk();
    let body = node.children(&mut cursor).find(|c| c.kind() == body_kind)?;
    let start = body.start_position();
    Some(Position::new(start.row as u32, start.column as u32))
}

/// Find the closing `}` position of a body node.
fn ts_body_close_brace(node: Node, body_kind: &str) -> Option<Position> {
    let mut cursor = node.walk();
    let body = node.children(&mut cursor).find(|c| c.kind() == body_kind)?;
    let end = body.end_position();
    // end_position points one past the `}`, so column - 1
    Some(Position::new(
        end.row as u32,
        end.column.saturating_sub(1) as u32,
    ))
}

/// Find the enclosing contract name for a function_definition node.
fn ts_enclosing_contract_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    let mut parent = node.parent();
    while let Some(p) = parent {
        if p.kind() == "contract_declaration"
            || p.kind() == "library_declaration"
            || p.kind() == "interface_declaration"
        {
            return ts_node_name(p, source);
        }
        parent = p.parent();
    }
    None
}

/// Find the gas index key matching a file path and contract name.
fn find_gas_key<'a>(
    gas_index: &'a gas::GasIndex,
    abs_path: &str,
    contract_name: &str,
) -> Option<&'a str> {
    let exact = format!("{abs_path}:{contract_name}");
    if gas_index.contains_key(&exact) {
        return Some(gas_index.get_key_value(&exact)?.0.as_str());
    }
    let file_name = std::path::Path::new(abs_path).file_name()?.to_str()?;
    let suffix = format!("{file_name}:{contract_name}");
    gas_index
        .keys()
        .find(|k| k.ends_with(&suffix))
        .map(|k| k.as_str())
}

/// Check if a tree-sitter node has a preceding comment containing the gas sentinel.
///
/// Looks at the previous named sibling for a comment node whose text contains
/// `@lsp-enable gas-estimates`.
fn has_gas_sentinel(node: Node, source: &str) -> bool {
    let mut prev = node.prev_named_sibling();
    while let Some(sibling) = prev {
        if sibling.kind() == "comment" {
            let text = &source[sibling.byte_range()];
            if text.contains(gas::GAS_SENTINEL) {
                return true;
            }
        } else {
            break;
        }
        prev = sibling.prev_named_sibling();
    }
    false
}

/// Create a gas inlay hint for a function definition using tree-sitter positions.
fn ts_gas_hint_for_function(
    node: Node,
    source: &str,
    range: &Range,
    gas_index: &gas::GasIndex,
    abs_path: &str,
) -> Option<InlayHint> {
    // Only show gas hints for functions annotated with @lsp-enable gas-estimates
    if !has_gas_sentinel(node, source) {
        return None;
    }
    let fn_name = ts_node_name(node, source)?;
    let contract_name = ts_enclosing_contract_name(node, source)?;
    let gas_key = find_gas_key(gas_index, abs_path, contract_name)?;
    let contract_gas = gas_index.get(gas_key)?;

    let prefix = format!("{fn_name}(");
    let cost = contract_gas
        .external_by_sig
        .iter()
        .find(|(sig, _)| sig.as_str().starts_with(&prefix))
        .map(|(_, c)| c.as_str())
        .or_else(|| {
            contract_gas
                .internal
                .iter()
                .find(|(sig, _)| sig.starts_with(&prefix))
                .map(|(_, c)| c.as_str())
        })?;

    // Position: opening or closing brace based on FN_GAS_HINT_POSITION
    let (brace_pos, offset) = match FN_GAS_HINT_POSITION {
        FnGasHintPosition::Opening => (ts_body_open_brace(node, "function_body")?, 1),
        FnGasHintPosition::Closing => (ts_body_close_brace(node, "function_body")?, 1),
    };
    if brace_pos.line < range.start.line || brace_pos.line > range.end.line {
        return None;
    }

    Some(InlayHint {
        position: Position::new(brace_pos.line, brace_pos.character + offset),
        kind: Some(InlayHintKind::TYPE),
        label: InlayHintLabel::String(format!("🔥 gas: {}", gas::format_gas(cost))),
        text_edits: None,
        tooltip: Some(InlayHintTooltip::String("Estimated gas cost".to_string())),
        padding_left: Some(true),
        padding_right: None,
        data: None,
    })
}

/// Create a gas inlay hint for a contract/library/interface definition.
/// Always uses the opening brace.
fn ts_gas_hint_for_contract(
    node: Node,
    source: &str,
    range: &Range,
    gas_index: &gas::GasIndex,
    abs_path: &str,
) -> Option<InlayHint> {
    // Only show deploy cost for contracts annotated with @lsp-enable gas-estimates
    if !has_gas_sentinel(node, source) {
        return None;
    }
    let contract_name = ts_node_name(node, source)?;
    let gas_key = find_gas_key(gas_index, abs_path, contract_name)?;
    let contract_gas = gas_index.get(gas_key)?;

    // Prefer totalCost, but when it's "infinite" show codeDepositCost instead
    let display_cost = match contract_gas.creation.get("totalCost").map(|s| s.as_str()) {
        Some("infinite") | None => contract_gas
            .creation
            .get("codeDepositCost")
            .map(|s| s.as_str())?,
        Some(total) => total,
    };

    let brace_pos = ts_body_open_brace(node, "contract_body")?;
    if brace_pos.line < range.start.line || brace_pos.line > range.end.line {
        return None;
    }

    Some(InlayHint {
        position: Position::new(brace_pos.line, brace_pos.character + 1),
        kind: Some(InlayHintKind::TYPE),
        label: InlayHintLabel::String(format!("🔥 deploy: {} ", gas::format_gas(display_cost))),
        text_edits: None,
        tooltip: Some(InlayHintTooltip::String(format!(
            "Deploy cost — code deposit: {}, execution: {}",
            gas::format_gas(
                contract_gas
                    .creation
                    .get("codeDepositCost")
                    .map(|s| s.as_str())
                    .unwrap_or("?")
            ),
            gas::format_gas(
                contract_gas
                    .creation
                    .get("executionCost")
                    .map(|s| s.as_str())
                    .unwrap_or("?")
            )
        ))),
        padding_left: Some(true),
        padding_right: None,
        data: None,
    })
}

// ── AST helpers ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_sentinel_present() {
        let source = r#"
contract Foo {
    /// @custom:lsp-enable gas-estimates
    function bar() public {}
}
"#;
        let tree = ts_parse(source).unwrap();
        let root = tree.root_node();
        // Find the function_definition node
        let contract = root.child(0).unwrap();
        let body = contract.child_by_field_name("body").unwrap();
        let mut cursor = body.walk();
        let fn_node = body
            .children(&mut cursor)
            .find(|c| c.kind() == "function_definition")
            .unwrap();
        assert!(has_gas_sentinel(fn_node, source));
    }

    #[test]
    fn test_gas_sentinel_absent() {
        let source = r#"
contract Foo {
    function bar() public {}
}
"#;
        let tree = ts_parse(source).unwrap();
        let root = tree.root_node();
        let contract = root.child(0).unwrap();
        let body = contract.child_by_field_name("body").unwrap();
        let mut cursor = body.walk();
        let fn_node = body
            .children(&mut cursor)
            .find(|c| c.kind() == "function_definition")
            .unwrap();
        assert!(!has_gas_sentinel(fn_node, source));
    }

    #[test]
    fn test_gas_sentinel_with_other_natspec() {
        let source = r#"
contract Foo {
    /// @notice Does something
    /// @custom:lsp-enable gas-estimates
    function bar() public {}
}
"#;
        let tree = ts_parse(source).unwrap();
        let root = tree.root_node();
        let contract = root.child(0).unwrap();
        let body = contract.child_by_field_name("body").unwrap();
        let mut cursor = body.walk();
        let fn_node = body
            .children(&mut cursor)
            .find(|c| c.kind() == "function_definition")
            .unwrap();
        assert!(has_gas_sentinel(fn_node, source));
    }

    #[test]
    fn test_ts_call_function_name() {
        let source = r#"
contract Foo {
    function bar(uint x) public {}
    function test() public {
        bar(42);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let mut found = Vec::new();
        find_calls(tree.root_node(), source, &mut found);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], "bar");
    }

    #[test]
    fn test_ts_member_call_name() {
        let source = r#"
contract Foo {
    function test() public {
        PRICE.addTax(TAX, TAX_BASE);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let mut found = Vec::new();
        find_calls(tree.root_node(), source, &mut found);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], "addTax");
    }

    #[test]
    fn test_ts_call_with_value_modifier() {
        let source = r#"
contract Foo {
    function test() public {
        router.swap{value: 100}(nativeKey, SWAP_PARAMS, testSettings, ZERO_BYTES);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let mut found = Vec::new();
        find_calls(tree.root_node(), source, &mut found);
        assert_eq!(found.len(), 1, "should find one call");
        assert_eq!(
            found[0], "swap",
            "should extract 'swap' through struct_expression"
        );
    }

    #[test]
    fn test_ts_call_simple_with_value_modifier() {
        let source = r#"
contract Foo {
    function test() public {
        foo{value: 1 ether}(42);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let mut found = Vec::new();
        find_calls(tree.root_node(), source, &mut found);
        assert_eq!(found.len(), 1, "should find one call");
        assert_eq!(
            found[0], "foo",
            "should extract 'foo' through struct_expression"
        );
    }

    #[test]
    fn test_ts_call_with_gas_modifier() {
        let source = r#"
contract Foo {
    function test() public {
        addr.call{gas: 5000, value: 1 ether}("");
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let mut found = Vec::new();
        find_calls(tree.root_node(), source, &mut found);
        assert_eq!(found.len(), 1, "should find one call");
        assert_eq!(
            found[0], "call",
            "should extract 'call' through struct_expression"
        );
    }

    #[test]
    fn test_find_call_by_text_scan_with_value_modifier() {
        // Simulate cursor inside args of `router.swap{value: 100}(arg1, |)`
        let source = "router.swap{value: 100}(nativeKey, SWAP_PARAMS)";
        // Place cursor after comma: position after "nativeKey, "
        let byte_pos = source.find("SWAP_PARAMS").unwrap();
        let ctx = find_call_by_text_scan(source, byte_pos).unwrap();
        assert_eq!(ctx.name, "swap");
        assert_eq!(ctx.arg_index, 1);
    }

    #[test]
    fn test_find_call_by_text_scan_simple_value_modifier() {
        let source = "foo{value: 1 ether}(42)";
        let byte_pos = source.find("42").unwrap();
        let ctx = find_call_by_text_scan(source, byte_pos).unwrap();
        assert_eq!(ctx.name, "foo");
        assert_eq!(ctx.arg_index, 0);
    }

    #[test]
    fn test_ts_emit_event_name() {
        let source = r#"
contract Foo {
    event Purchase(address buyer, uint256 price);
    function test() public {
        emit Purchase(msg.sender, 100);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let mut found = Vec::new();
        find_emits(tree.root_node(), source, &mut found);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], "Purchase");
    }

    #[test]
    fn test_ts_new_expression_name() {
        let source = r#"
contract Token {
    constructor(string memory _name, uint256 _supply) {}
}
contract Factory {
    function create() public {
        Token t = new Token("MyToken", 1000);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let mut found = Vec::new();
        find_new_exprs(tree.root_node(), source, &mut found);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], "Token");
    }

    #[test]
    fn test_ts_new_expression_arguments() {
        // tree-sitter wraps `new Token(args)` as a call_expression whose
        // callee is a new_expression. The call_argument nodes live on the
        // call_expression, so we count them there.
        let source = r#"
contract Router {
    constructor(address _manager, address _hook) {}
}
contract Factory {
    function create() public {
        Router r = new Router(address(this), address(0));
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let mut arg_counts = Vec::new();
        find_call_arg_counts(tree.root_node(), &mut arg_counts);
        // call_expression for `new Router(...)` has 2 args
        assert_eq!(arg_counts, vec![2]);
    }

    #[test]
    fn test_ts_find_call_at_byte_new_expression() {
        let source = r#"
contract Token {
    constructor(string memory _name, uint256 _supply) {}
}
contract Factory {
    function create() public {
        Token t = new Token("MyToken", 1000);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let pos = source.find("1000").unwrap();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos).unwrap();
        assert_eq!(ctx.name, "Token");
        assert_eq!(ctx.arg_index, 1);
        assert_eq!(ctx.arg_count, 2);
    }

    #[test]
    fn test_ts_find_call_at_byte_new_expression_first_arg() {
        let source = r#"
contract Token {
    constructor(string memory _name, uint256 _supply) {}
}
contract Factory {
    function create() public {
        Token t = new Token("MyToken", 1000);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let pos = source.find("\"MyToken\"").unwrap();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos).unwrap();
        assert_eq!(ctx.name, "Token");
        assert_eq!(ctx.arg_index, 0);
        assert_eq!(ctx.arg_count, 2);
    }

    #[test]
    fn test_extract_new_expression_call_info() {
        // Simulate `new Token("MyToken", 1000)` using a ConstructorIndex
        let new_expr: Value = serde_json::json!({
            "nodeType": "NewExpression",
            "typeName": {
                "nodeType": "UserDefinedTypeName",
                "referencedDeclaration": 22,
                "pathNode": {
                    "name": "Token"
                }
            }
        });

        let mut constructor_index = ConstructorIndex::new();
        constructor_index.insert(
            NodeId(22),
            ConstructorInfo {
                constructor_id: NodeId(21),
                contract_name: "Token".to_string(),
                param_names: vec!["_name".to_string(), "_supply".to_string()],
            },
        );

        let info = extract_new_expression_call_info(&new_expr, 2, &constructor_index).unwrap();
        assert_eq!(info.name, "Token");
        assert_eq!(info.params.names, vec!["_name", "_supply"]);
        assert_eq!(info.params.skip, 0);
        assert_eq!(info.decl_id, NodeId(21));
    }

    #[test]
    fn test_ts_call_arguments_count() {
        let source = r#"
contract Foo {
    function bar(uint x, uint y) public {}
    function test() public {
        bar(1, 2);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let mut arg_counts = Vec::new();
        find_call_arg_counts(tree.root_node(), &mut arg_counts);
        assert_eq!(arg_counts, vec![2]);
    }

    #[test]
    fn test_ts_argument_positions_follow_live_buffer() {
        // Simulate an edited buffer with extra whitespace
        let source = r#"
contract Foo {
    function bar(uint x, uint y) public {}
    function test() public {
        bar(
            1,
            2
        );
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let mut positions = Vec::new();
        find_arg_positions(tree.root_node(), &mut positions);
        // First arg "1" is on line 5 (0-indexed), second "2" on line 6
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].0, 5); // row of "1"
        assert_eq!(positions[1].0, 6); // row of "2"
    }

    // Test helpers

    fn find_calls<'a>(node: Node<'a>, source: &'a str, out: &mut Vec<&'a str>) {
        if node.kind() == "call_expression"
            && let Some(name) = ts_call_function_name(node, source)
        {
            out.push(name);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_calls(child, source, out);
        }
    }

    fn find_emits<'a>(node: Node<'a>, source: &'a str, out: &mut Vec<&'a str>) {
        if node.kind() == "emit_statement"
            && let Some(name) = ts_emit_event_name(node, source)
        {
            out.push(name);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_emits(child, source, out);
        }
    }

    fn find_new_exprs<'a>(node: Node<'a>, source: &'a str, out: &mut Vec<&'a str>) {
        if node.kind() == "new_expression"
            && let Some(name) = ts_new_expression_name(node, source)
        {
            out.push(name);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_new_exprs(child, source, out);
        }
    }

    fn find_call_arg_counts(node: Node, out: &mut Vec<usize>) {
        if node.kind() == "call_expression" {
            out.push(ts_call_arguments(node).len());
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_call_arg_counts(child, out);
        }
    }

    fn find_arg_positions(node: Node, out: &mut Vec<(usize, usize)>) {
        if node.kind() == "call_expression" {
            for arg in ts_call_arguments(node) {
                let p = arg.start_position();
                out.push((p.row, p.column));
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_arg_positions(child, out);
        }
    }

    #[test]
    fn test_ts_find_call_at_byte_first_arg() {
        let source = r#"
contract Foo {
    function bar(uint x, uint y) public {}
    function test() public {
        bar(42, 99);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        // "42" is the first argument — find its byte offset
        let pos_42 = source.find("42").unwrap();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos_42).unwrap();
        assert_eq!(ctx.name, "bar");
        assert_eq!(ctx.arg_index, 0);
        assert_eq!(ctx.arg_count, 2);
    }

    #[test]
    fn test_ts_find_call_at_byte_second_arg() {
        let source = r#"
contract Foo {
    function bar(uint x, uint y) public {}
    function test() public {
        bar(42, 99);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let pos_99 = source.find("99").unwrap();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos_99).unwrap();
        assert_eq!(ctx.name, "bar");
        assert_eq!(ctx.arg_index, 1);
        assert_eq!(ctx.arg_count, 2);
    }

    #[test]
    fn test_ts_find_call_at_byte_outside_call_returns_none() {
        let source = r#"
contract Foo {
    function bar(uint x) public {}
    function test() public {
        uint z = 10;
        bar(42);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        // "10" is a local variable assignment, not a call argument
        let pos_10 = source.find("10").unwrap();
        assert!(ts_find_call_at_byte(tree.root_node(), source, pos_10).is_none());
    }

    #[test]
    fn test_ts_find_call_at_byte_member_call() {
        let source = r#"
contract Foo {
    function test() public {
        PRICE.addTax(TAX, TAX_BASE);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let pos_tax = source.find("TAX,").unwrap();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos_tax).unwrap();
        assert_eq!(ctx.name, "addTax");
        assert_eq!(ctx.arg_index, 0);
        assert_eq!(ctx.arg_count, 2);
    }

    #[test]
    fn test_ts_find_call_at_byte_emit_statement() {
        let source = r#"
contract Foo {
    event Purchase(address buyer, uint256 price);
    function test() public {
        emit Purchase(msg.sender, 100);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let pos_100 = source.find("100").unwrap();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos_100).unwrap();
        assert_eq!(ctx.name, "Purchase");
        assert_eq!(ctx.arg_index, 1);
        assert_eq!(ctx.arg_count, 2);
    }

    #[test]
    fn test_ts_find_call_at_byte_multiline() {
        let source = r#"
contract Foo {
    function bar(uint x, uint y, uint z) public {}
    function test() public {
        bar(
            1,
            2,
            3
        );
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        // Find "2" — the second argument on its own line
        // Need to be careful since "2" appears in the source in multiple places
        let pos_2 = source.find("            2").unwrap() + 12; // skip whitespace
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos_2).unwrap();
        assert_eq!(ctx.name, "bar");
        assert_eq!(ctx.arg_index, 1);
        assert_eq!(ctx.arg_count, 3);
    }

    #[test]
    fn test_resolve_callsite_param_basic() {
        // Build a HintLookup manually with a known call site
        let mut lookup = HintLookup {
            by_offset: HashMap::new(),
            by_name: HashMap::new(),
        };
        lookup.by_name.insert(
            ("transfer".to_string(), 2),
            CallSite {
                info: ParamInfo {
                    names: vec!["to".to_string(), "amount".to_string()],
                    skip: 0,
                },
                name: "transfer".to_string(),
                decl_id: NodeId(42),
            },
        );

        // Resolve first argument
        let result = lookup.resolve_callsite_param(0, "transfer", 2, 0).unwrap();
        assert_eq!(result.param_name, "to");
        assert_eq!(result.decl_id, NodeId(42));

        // Resolve second argument
        let result = lookup.resolve_callsite_param(0, "transfer", 2, 1).unwrap();
        assert_eq!(result.param_name, "amount");
        assert_eq!(result.decl_id, NodeId(42));
    }

    #[test]
    fn test_resolve_callsite_param_with_skip() {
        // Simulate a using-for library call where skip=1
        let mut lookup = HintLookup {
            by_offset: HashMap::new(),
            by_name: HashMap::new(),
        };
        lookup.by_name.insert(
            ("addTax".to_string(), 2),
            CallSite {
                info: ParamInfo {
                    names: vec!["self".to_string(), "tax".to_string(), "base".to_string()],
                    skip: 1,
                },
                name: "addTax".to_string(),
                decl_id: NodeId(99),
            },
        );

        // First arg maps to param index 1 (skip=1), which is "tax"
        let result = lookup.resolve_callsite_param(0, "addTax", 2, 0).unwrap();
        assert_eq!(result.param_name, "tax");

        // Second arg maps to param index 2, which is "base"
        let result = lookup.resolve_callsite_param(0, "addTax", 2, 1).unwrap();
        assert_eq!(result.param_name, "base");
    }

    #[test]
    fn test_resolve_callsite_param_out_of_bounds() {
        let mut lookup = HintLookup {
            by_offset: HashMap::new(),
            by_name: HashMap::new(),
        };
        lookup.by_name.insert(
            ("foo".to_string(), 1),
            CallSite {
                info: ParamInfo {
                    names: vec!["x".to_string()],
                    skip: 0,
                },
                name: "foo".to_string(),
                decl_id: NodeId(1),
            },
        );

        // Arg index 1 is out of bounds for a single-param function
        assert!(lookup.resolve_callsite_param(0, "foo", 1, 1).is_none());
    }

    #[test]
    fn test_resolve_callsite_param_unknown_function() {
        let lookup = HintLookup {
            by_offset: HashMap::new(),
            by_name: HashMap::new(),
        };
        assert!(lookup.resolve_callsite_param(0, "unknown", 1, 0).is_none());
    }

    #[test]
    fn test_ts_find_call_at_byte_emit_member_access() {
        // Simulates: emit ModifyLiquidity(id, msg.sender, params.tickLower, ...);
        // Hovering on "tickLower" (the member name in params.tickLower) should
        // resolve to arg_index=2 of the ModifyLiquidity emit.
        let source = r#"
contract Foo {
    event ModifyLiquidity(uint id, address sender, int24 tickLower, int24 tickUpper);
    function test() public {
        emit ModifyLiquidity(id, msg.sender, params.tickLower, params.tickUpper);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        // Find "tickLower" inside "params.tickLower" — the first occurrence after "params."
        let params_tick = source.find("params.tickLower,").unwrap();
        let tick_lower_pos = params_tick + "params.".len(); // points at "tickLower"

        let ctx = ts_find_call_at_byte(tree.root_node(), source, tick_lower_pos).unwrap();
        assert_eq!(ctx.name, "ModifyLiquidity");
        assert_eq!(
            ctx.arg_index, 2,
            "params.tickLower is the 3rd argument (index 2)"
        );
        assert_eq!(ctx.arg_count, 4);
    }

    #[test]
    fn test_ts_find_call_at_byte_member_access_on_property() {
        // Hovering on "sender" in "msg.sender" as an argument
        let source = r#"
contract Foo {
    event Transfer(address from, address to);
    function test() public {
        emit Transfer(msg.sender, addr);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let sender_pos = source.find("sender").unwrap();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, sender_pos).unwrap();
        assert_eq!(ctx.name, "Transfer");
        assert_eq!(ctx.arg_index, 0, "msg.sender is the 1st argument");
    }

    #[test]
    fn test_ts_find_call_at_byte_emit_all_args() {
        // Verify each argument position in an emit with member accesses
        let source = r#"
contract Foo {
    event ModifyLiquidity(uint id, address sender, int24 tickLower, int24 tickUpper);
    function test() public {
        emit ModifyLiquidity(id, msg.sender, params.tickLower, params.tickUpper);
    }
}
"#;
        let tree = ts_parse(source).unwrap();

        // arg 0: "id"
        let pos_id = source.find("(id,").unwrap() + 1;
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos_id).unwrap();
        assert_eq!(ctx.name, "ModifyLiquidity");
        assert_eq!(ctx.arg_index, 0);

        // arg 1: "msg.sender" — hover on "msg"
        let pos_msg = source.find("msg.sender").unwrap();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos_msg).unwrap();
        assert_eq!(ctx.arg_index, 1);

        // arg 2: "params.tickLower" — hover on "tickLower"
        let pos_tl = source.find("params.tickLower").unwrap() + "params.".len();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos_tl).unwrap();
        assert_eq!(ctx.arg_index, 2);

        // arg 3: "params.tickUpper" — hover on "params"
        let pos_tu = source.find("params.tickUpper").unwrap();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos_tu).unwrap();
        assert_eq!(ctx.arg_index, 3);
    }

    #[test]
    fn test_ts_find_call_at_byte_nested_call_arg() {
        // When an argument is itself a function call, hovering inside
        // the inner call should find the inner call, not the outer.
        let source = r#"
contract Foo {
    function inner(uint x) public returns (uint) {}
    function outer(uint a, uint b) public {}
    function test() public {
        outer(inner(42), 99);
    }
}
"#;
        let tree = ts_parse(source).unwrap();

        // "42" is an arg to inner(), not outer()
        let pos_42 = source.find("42").unwrap();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos_42).unwrap();
        assert_eq!(ctx.name, "inner");
        assert_eq!(ctx.arg_index, 0);

        // "99" is an arg to outer()
        let pos_99 = source.find("99").unwrap();
        let ctx = ts_find_call_at_byte(tree.root_node(), source, pos_99).unwrap();
        assert_eq!(ctx.name, "outer");
        assert_eq!(ctx.arg_index, 1);
    }

    #[test]
    fn test_ts_find_call_for_signature_incomplete_call() {
        // Cursor right after `(` with no arguments yet
        let source = r#"
contract Foo {
    function bar(uint x, uint y) public {}
    function test() public {
        bar(
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let pos = source.find("bar(").unwrap() + 4;
        let ctx = ts_find_call_for_signature(tree.root_node(), source, pos).unwrap();
        assert_eq!(ctx.name, "bar");
        assert_eq!(ctx.arg_index, 0);
    }

    #[test]
    fn test_ts_find_call_for_signature_after_comma() {
        // Cursor right after `,` — on second argument
        let source = r#"
contract Foo {
    function bar(uint x, uint y) public {}
    function test() public {
        bar(42,
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let pos = source.find("42,").unwrap() + 3;
        let ctx = ts_find_call_for_signature(tree.root_node(), source, pos).unwrap();
        assert_eq!(ctx.name, "bar");
        assert_eq!(ctx.arg_index, 1);
    }

    #[test]
    fn test_ts_find_call_for_signature_complete_call() {
        // Normal complete call still works
        let source = r#"
contract Foo {
    function bar(uint x, uint y) public {}
    function test() public {
        bar(42, 99);
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let pos = source.find("42").unwrap();
        let ctx = ts_find_call_for_signature(tree.root_node(), source, pos).unwrap();
        assert_eq!(ctx.name, "bar");
        assert_eq!(ctx.arg_index, 0);
    }

    #[test]
    fn test_ts_find_call_for_signature_member_call() {
        // Member access call like PRICE.addTax(
        let source = r#"
contract Foo {
    function test() public {
        PRICE.addTax(
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let pos = source.find("addTax(").unwrap() + 7;
        let ctx = ts_find_call_for_signature(tree.root_node(), source, pos).unwrap();
        assert_eq!(ctx.name, "addTax");
        assert_eq!(ctx.arg_index, 0);
    }

    #[test]
    fn test_ts_find_call_for_signature_array_access() {
        // Mapping index access like orders[orderId]
        let source = r#"
contract Foo {
    mapping(bytes32 => uint256) public orders;
    function test() public {
        orders[orderId];
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        // Cursor inside the brackets, on "orderId"
        let pos = source.find("[orderId]").unwrap() + 1;
        let ctx = ts_find_call_for_signature(tree.root_node(), source, pos).unwrap();
        assert_eq!(ctx.name, "orders");
        assert_eq!(ctx.arg_index, 0);
        assert!(ctx.is_index_access);
    }

    #[test]
    fn test_ts_find_call_for_signature_array_access_empty() {
        // Cursor right after `[` with no key yet
        let source = r#"
contract Foo {
    mapping(bytes32 => uint256) public orders;
    function test() public {
        orders[
    }
}
"#;
        let tree = ts_parse(source).unwrap();
        let pos = source.find("orders[").unwrap() + 7;
        let ctx = ts_find_call_for_signature(tree.root_node(), source, pos).unwrap();
        assert_eq!(ctx.name, "orders");
        assert!(ctx.is_index_access);
    }
}
