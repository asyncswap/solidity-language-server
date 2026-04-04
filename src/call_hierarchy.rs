//! Call hierarchy support for the Solidity language server.
//!
//! This module provides the query logic and LSP conversion helpers for
//! the Call Hierarchy feature (`textDocument/prepareCallHierarchy`,
//! `callHierarchy/incomingCalls`, `callHierarchy/outgoingCalls`).
//!
//! # Architecture
//!
//! Call hierarchy queries are derived from the same `nodes` index that
//! powers `textDocument/references`. Every AST node with a
//! `referencedDeclaration` is a potential call edge. The caller/callee
//! relationship is resolved via **span containment**: for each reference
//! node, we find the smallest enclosing `FunctionDefinition` or
//! `ModifierDefinition` — that is the "caller".
//!
//! This approach works uniformly on both fresh builds (`CachedBuild::new()`)
//! and warm-loaded builds (`from_reference_index()`) because the `nodes`
//! index is always populated.
//!
//! # Equivalence
//!
//! When `base_function_implementation` is populated, incoming calls expand
//! the target to include all equivalent IDs (interface ↔ implementation),
//! so callers via interface-typed references are included.
//!
//! # Low-level calls
//!
//! Solidity's `.call()`, `.staticcall()`, and `.delegatecall()` have no
//! `referencedDeclaration` in the AST because they are built-in address
//! methods. These are detected separately:
//!
//! - **Solidity `MemberAccess`**: Nodes with `member_name ∈ {call, staticcall,
//!   delegatecall}` and `referenced_declaration: None`. These have `id` fields
//!   and are in the `nodes` index.
//! - **Yul opcodes**: `YulFunctionCall` nodes with `functionName.name ∈ {call,
//!   staticcall, delegatecall}`. These have NO `id` field and are stored in
//!   `CachedBuild::low_level_calls`.
//!
//! Low-level calls produce synthetic `CallHierarchyItem`s with no backing
//! declaration node.

use std::collections::HashMap;
use tower_lsp::lsp_types::{CallHierarchyItem, Range, SymbolKind, Url};

use crate::goto::{CachedBuild, LOW_LEVEL_CALL_NAMES, NodeInfo, bytes_to_pos};
use crate::references::byte_to_id;
use crate::solc_ast::DeclNode;
use crate::types::{AbsPath, NodeId, SolcFileId, SourceLoc};

// ── Node identity verification ─────────────────────────────────────────────

/// Verify that a `NodeId` in a specific build refers to the expected source entity.
///
/// Node IDs are per-compilation — the same numeric ID can refer to completely
/// different functions in different builds (e.g. ID 616 = `swap` in the file
/// build, but ID 616 = some library function in a sub-cache).  This function
/// checks three properties to confirm the node is the one we expect:
///
/// 1. **File** — the node must exist in `expected_abs_path` within this build.
/// 2. **Position** — the node's `name_location` byte offset must equal
///    `expected_name_offset`.
/// 3. **Name** — the source text at the `name_location` must match
///    `expected_name`.
///
/// If all three match the node is guaranteed to be the same source entity,
/// regardless of which compilation produced the build.  The check is O(1) —
/// a single HashMap lookup + integer/string comparison.
///
/// Returns `true` if the node passes identity verification.
pub fn verify_node_identity(
    nodes: &HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
    node_id: NodeId,
    expected_abs_path: &str,
    expected_name_offset: usize,
    expected_name: &str,
) -> bool {
    let Some(file_nodes) = nodes.get(expected_abs_path) else {
        return false;
    };
    let Some(info) = file_nodes.get(&node_id) else {
        return false;
    };
    // Check name_location byte offset (canonical file IDs, stable byte offset).
    let name_offset_matches = info
        .name_location
        .as_deref()
        .and_then(SourceLoc::parse)
        .is_some_and(|loc| loc.offset == expected_name_offset);
    if !name_offset_matches {
        return false;
    }
    // Check node name via decl or source text extraction.
    // For callable nodes (FunctionDefinition, ModifierDefinition, ContractDefinition),
    // we can read the name from the source at name_location.  But since reading the
    // file is expensive, we use a simpler heuristic first: if the node_type is a
    // callable and the name_location offset matched, the name_location span in the
    // source file covers exactly `expected_name`.  We verify by reading the source
    // only when the offset already matched (so this is the rare confirmation path,
    // not the hot path).
    if let Some(nl) = info.name_location.as_deref() {
        if let Some(loc) = SourceLoc::parse(nl) {
            // Read the source bytes at the name_location range and compare.
            // The file read is cached by the OS and is the same file we'll read
            // later for building CallHierarchyItems anyway.
            let name_matches = std::path::Path::new(expected_abs_path)
                .exists()
                .then(|| std::fs::read(expected_abs_path).ok())
                .flatten()
                .is_some_and(|source_bytes| {
                    source_bytes
                        .get(loc.offset..loc.end())
                        .is_some_and(|slice| slice == expected_name.as_bytes())
                });
            return name_matches;
        }
    }
    false
}

/// Resolve a target callable's node IDs within a single build.
///
/// Given an expected function identity (`abs_path`, `name`, `name_offset`),
/// finds the matching node IDs in `build` using a two-tier strategy:
///
/// 1. **Verify by ID** — if the original `node_id` exists in this build's
///    target file and passes [`verify_node_identity`], accept it immediately
///    (O(1) fast path).
/// 2. **Resolve by position** — fall back to [`byte_to_id`] using the
///    expected `name_offset` to find the build-local node ID for the same
///    source location (O(n) over file nodes, but only when the fast path
///    fails — typically only in sub-caches).
///
/// Returns the resolved node IDs (may be empty if the build doesn't contain
/// the target file).
pub fn resolve_target_in_build(
    build: &CachedBuild,
    node_id: NodeId,
    target_abs: &str,
    target_name: &str,
    target_name_offset: usize,
) -> Vec<NodeId> {
    let mut ids = Vec::new();

    // Fast path: verify the original node ID in this build.
    if verify_node_identity(
        &build.nodes,
        node_id,
        target_abs,
        target_name_offset,
        target_name,
    ) {
        ids.push(node_id);
        return ids;
    }

    // Slow path: the numeric ID doesn't exist or failed verification.
    // Re-resolve by byte offset (stable across compilations).
    if let Some(resolved_id) = byte_to_id(&build.nodes, target_abs, target_name_offset) {
        // Double-check that the resolved node is a callable with the right name.
        if let Some(file_nodes) = build.nodes.get(target_abs) {
            if let Some(info) = file_nodes.get(&resolved_id) {
                let nt = info.node_type.as_deref().unwrap_or("");
                if matches!(
                    nt,
                    "FunctionDefinition" | "ModifierDefinition" | "ContractDefinition"
                ) {
                    ids.push(resolved_id);
                }
            }
        }
    }

    ids
}

// ── Query functions ────────────────────────────────────────────────────────

/// Find all incoming calls to the given target IDs.
///
/// Scans the `nodes` index for all reference nodes whose
/// `referenced_declaration` matches any of the `target_ids`, then resolves
/// the enclosing callable (function/modifier) for each reference via span
/// containment. Returns `(caller_id, call_src)` pairs.
///
/// `target_ids` should include equivalent IDs from `base_function_implementation`
/// so that callers via interface-typed references are captured.
pub fn incoming_calls(
    nodes: &HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
    target_ids: &[NodeId],
) -> Vec<(NodeId, String)> {
    let mut results = Vec::new();

    for (_abs_path, file_nodes) in nodes {
        // Collect all callable nodes in this file for enclosing-span lookup.
        let callables: Vec<(NodeId, &NodeInfo)> = file_nodes
            .iter()
            .filter(|(_id, info)| {
                info.node_type
                    .as_deref()
                    .is_some_and(|nt| nt == "FunctionDefinition" || nt == "ModifierDefinition")
            })
            .map(|(id, info)| (*id, info))
            .collect();

        for (ref_id, ref_info) in file_nodes {
            let Some(ref_decl) = ref_info.referenced_declaration else {
                continue;
            };
            if !target_ids.contains(&ref_decl) {
                continue;
            }
            // Don't count self-references (the declaration itself).
            if target_ids.contains(ref_id) {
                continue;
            }

            // Find the smallest enclosing callable by span containment.
            let ref_src = match SourceLoc::parse(ref_info.src.as_str()) {
                Some(s) => s,
                None => continue,
            };

            let mut best_callable: Option<(NodeId, usize)> = None;
            for &(callable_id, callable_info) in &callables {
                let Some(callable_src) = SourceLoc::parse(callable_info.src.as_str()) else {
                    continue;
                };
                if callable_src.file_id != ref_src.file_id {
                    continue;
                }
                if callable_src.offset <= ref_src.offset && ref_src.end() <= callable_src.end() {
                    let span = callable_src.length;
                    if best_callable.is_none() || span < best_callable.unwrap().1 {
                        best_callable = Some((callable_id, span));
                    }
                }
            }

            if let Some((caller_id, _)) = best_callable {
                // Prefer member_location (just the identifier) over src
                // (the whole expression) so fromRanges point precisely
                // at the function name, not the entire chain.
                let call_src = ref_info
                    .member_location
                    .as_deref()
                    .unwrap_or(ref_info.src.as_str())
                    .to_string();
                results.push((caller_id, call_src));
            }
        }
    }

    results.sort_by(|a, b| a.0.0.cmp(&b.0.0).then_with(|| a.1.cmp(&b.1)));
    results.dedup();
    results
}

/// Find all outgoing calls from a given caller function/modifier.
///
/// Finds all nodes inside the caller's span whose `referenced_declaration`
/// points to a callable (FunctionDefinition or ModifierDefinition).
/// Returns `(callee_id, call_src)` pairs.
pub fn outgoing_calls(
    nodes: &HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
    caller_id: NodeId,
) -> Vec<(NodeId, String)> {
    let caller_info = match find_node_info(nodes, caller_id) {
        Some(info) => info,
        None => return vec![],
    };
    let caller_src = match SourceLoc::parse(caller_info.src.as_str()) {
        Some(s) => s,
        None => return vec![],
    };

    // Collect all callable node IDs across all files.
    let callable_ids: std::collections::HashSet<NodeId> = nodes
        .values()
        .flat_map(|file_nodes| {
            file_nodes.iter().filter_map(|(id, info)| {
                info.node_type.as_deref().and_then(|nt| {
                    if nt == "FunctionDefinition" || nt == "ModifierDefinition" {
                        Some(*id)
                    } else {
                        None
                    }
                })
            })
        })
        .collect();

    let mut results = Vec::new();

    for (_abs_path, file_nodes) in nodes {
        for (_ref_id, ref_info) in file_nodes {
            let Some(ref_decl) = ref_info.referenced_declaration else {
                continue;
            };
            if !callable_ids.contains(&ref_decl) {
                continue;
            }
            let Some(ref_src) = SourceLoc::parse(ref_info.src.as_str()) else {
                continue;
            };
            if ref_src.file_id != caller_src.file_id {
                continue;
            }
            if caller_src.offset <= ref_src.offset && ref_src.end() <= caller_src.end() {
                if ref_decl == caller_id {
                    continue;
                }
                // Prefer member_location (just the identifier) over src
                // (the whole expression) so fromRanges point precisely
                // at the function name, not the entire chain.
                let call_src = ref_info
                    .member_location
                    .as_deref()
                    .unwrap_or(ref_info.src.as_str())
                    .to_string();
                results.push((ref_decl, call_src));
            }
        }
    }

    results.sort_by(|a, b| a.0.0.cmp(&b.0.0).then_with(|| a.1.cmp(&b.1)));
    results.dedup();
    results
}

/// Find outgoing low-level calls from a given caller function/modifier.
///
/// Detects two patterns:
/// 1. **Solidity `MemberAccess`**: nodes in the `nodes` index with
///    `member_name ∈ {call, staticcall, delegatecall}` and no
///    `referenced_declaration`, inside the caller's span.
/// 2. **Yul opcodes**: entries in `low_level_calls` whose `src` falls
///    inside the caller's span.  Only actual call opcodes are included
///    (not `mstore`, `sload`, etc.) — scoped by `YUL_CALL_OPCODES` at
///    index time.
///
/// Returns `(CallHierarchyItem, call_range)` pairs.  The
/// `CallHierarchyItem` is synthetic — there is no backing declaration
/// node, so it carries no `nodeId` in `data`.
pub fn outgoing_low_level_calls(
    build: &CachedBuild,
    caller_id: NodeId,
) -> Vec<(CallHierarchyItem, Range)> {
    let caller_info = match find_node_info(&build.nodes, caller_id) {
        Some(info) => info,
        None => return vec![],
    };
    let caller_src = match SourceLoc::parse(caller_info.src.as_str()) {
        Some(s) => s,
        None => return vec![],
    };

    let mut results: Vec<(CallHierarchyItem, Range)> = Vec::new();

    // ── Pass 1: Solidity MemberAccess low-level calls ──────────────────
    for (_abs_path, file_nodes) in &build.nodes {
        for (_ref_id, ref_info) in file_nodes {
            // Only MemberAccess nodes with no referenced_declaration.
            if ref_info.referenced_declaration.is_some() {
                continue;
            }
            let Some(mn) = ref_info.member_name.as_deref() else {
                continue;
            };
            if !LOW_LEVEL_CALL_NAMES.contains(&mn) {
                continue;
            }
            let Some(ref_src) = SourceLoc::parse(ref_info.src.as_str()) else {
                continue;
            };
            if ref_src.file_id != caller_src.file_id {
                continue;
            }
            if !(caller_src.offset <= ref_src.offset && ref_src.end() <= caller_src.end()) {
                continue;
            }

            // Build a synthetic CallHierarchyItem for this low-level call.
            // Use member_location (the `.call` identifier) for selection_range
            // so the cursor lands on the call site, not the whole expression.
            let name_src = ref_info
                .member_location
                .as_deref()
                .unwrap_or(ref_info.src.as_str());
            if let Some((item, call_range)) = low_level_call_item_from_src(
                mn,
                ref_info.src.as_str(),
                name_src,
                &build.id_to_path_map,
            ) {
                results.push((item, call_range));
            }
        }
    }

    // ── Pass 2: Yul call opcodes ───────────────────────────────────────
    for llc in &build.low_level_calls {
        let Some(llc_src) = SourceLoc::parse(llc.src.as_str()) else {
            continue;
        };
        if llc_src.file_id != caller_src.file_id {
            continue;
        }
        if !(caller_src.offset <= llc_src.offset && llc_src.end() <= caller_src.end()) {
            continue;
        }

        // Use functionName.src for selection_range (just the opcode name).
        if let Some((item, call_range)) = low_level_call_item_from_src(
            &llc.kind,
            llc.src.as_str(),
            llc.name_src.as_str(),
            &build.id_to_path_map,
        ) {
            results.push((item, call_range));
        }
    }

    results
}

/// Build a synthetic `CallHierarchyItem` and call-site `Range` for a
/// low-level call from its kind (`"call"`, `"staticcall"`, `"delegatecall"`).
///
/// - `src` is the full expression span (used for `range` and `fromRanges`).
/// - `name_src` is the identifier-only span (used for `selection_range` so
///   the cursor lands on the `.call` / `call` site).
fn low_level_call_item_from_src(
    kind: &str,
    src: &str,
    name_src: &str,
    id_to_path_map: &HashMap<SolcFileId, String>,
) -> Option<(CallHierarchyItem, Range)> {
    let loc = SourceLoc::parse(src)?;
    let file_path_str = id_to_path_map.get(&loc.file_id_str())?;

    let file_path = if std::path::Path::new(file_path_str).is_absolute() {
        std::path::PathBuf::from(file_path_str)
    } else {
        std::env::current_dir().ok()?.join(file_path_str)
    };

    let source_bytes = std::fs::read(&file_path).ok()?;
    let uri = Url::from_file_path(&file_path).ok()?;

    let start = bytes_to_pos(&source_bytes, loc.offset)?;
    let end = bytes_to_pos(&source_bytes, loc.end())?;
    let range = Range { start, end };

    // Parse the name_src for the selection range (the .call / call site).
    let selection_range = SourceLoc::parse(name_src)
        .and_then(|nl| {
            let ns = bytes_to_pos(&source_bytes, nl.offset)?;
            let ne = bytes_to_pos(&source_bytes, nl.end())?;
            Some(Range { start: ns, end: ne })
        })
        .unwrap_or(range);

    let name = format!(".{}()", kind);

    let item = CallHierarchyItem {
        name,
        kind: SymbolKind::FUNCTION,
        tags: None,
        detail: Some(format!("low-level {}", kind)),
        uri,
        range,
        selection_range,
        data: Some(serde_json::json!({ "lowLevelCall": kind })),
    };

    Some((item, selection_range))
}

// ── LSP conversion helpers ─────────────────────────────────────────────────

/// Convert a `DeclNode` into a `CallHierarchyItem`.
///
/// Uses `node_id_to_source_path` (not `decl.src()` file IDs) to find the
/// file path — this is critical because `decl.src()` uses raw
/// (pre-canonicalization) file IDs that don't match `id_to_path_map`.
///
/// `nodes` is the full node index, used to look up `nameLocation` from
/// the `NodeInfo` (since `extract_decl_nodes` strips `nameLocation` from
/// declaration nodes for memory efficiency).
pub fn decl_to_hierarchy_item(
    decl: &DeclNode,
    node_id: NodeId,
    node_id_to_source_path: &HashMap<NodeId, AbsPath>,
    id_to_path_map: &HashMap<SolcFileId, String>,
    nodes: &HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
) -> Option<CallHierarchyItem> {
    let abs_path = node_id_to_source_path.get(&node_id)?;

    let file_path = find_file_path(abs_path.as_str(), id_to_path_map)?;
    let source_bytes = std::fs::read(&file_path).ok()?;
    let uri = Url::from_file_path(&file_path).ok()?;

    let src_loc = SourceLoc::parse(decl.src())?;
    let start = bytes_to_pos(&source_bytes, src_loc.offset)?;
    let end = bytes_to_pos(&source_bytes, src_loc.end())?;
    let range = Range { start, end };

    let selection_range = find_node_info(nodes, node_id)
        .and_then(|info| {
            name_loc_range(
                &info.name_location.as_ref().map(|s| s.to_string()),
                &source_bytes,
            )
        })
        .unwrap_or(range);

    let symbol_kind = match decl {
        DeclNode::FunctionDefinition(_) => SymbolKind::FUNCTION,
        DeclNode::ModifierDefinition(_) => SymbolKind::FUNCTION,
        DeclNode::ContractDefinition(c) => match c.contract_kind {
            crate::solc_ast::ContractKind::Interface => SymbolKind::INTERFACE,
            _ => SymbolKind::CLASS,
        },
        _ => SymbolKind::FUNCTION,
    };

    let data = Some(serde_json::json!({ "nodeId": node_id.0 }));

    Some(CallHierarchyItem {
        name: decl.name().to_string(),
        kind: symbol_kind,
        tags: None,
        detail: decl.build_signature(),
        uri,
        range,
        selection_range,
        data,
    })
}

/// Convert a `NodeInfo` (from the nodes index) into a `CallHierarchyItem`.
///
/// This is the fallback used when `decl_index` doesn't contain the node
/// (e.g., warm-loaded project builds where `decl_index` is empty).
pub fn node_info_to_hierarchy_item(
    node_id: NodeId,
    info: &NodeInfo,
    id_to_path_map: &HashMap<SolcFileId, String>,
) -> Option<CallHierarchyItem> {
    let src_loc = SourceLoc::parse(info.src.as_str())?;
    let file_path_str = id_to_path_map.get(&src_loc.file_id_str())?;

    let file_path = if std::path::Path::new(file_path_str).is_absolute() {
        std::path::PathBuf::from(file_path_str)
    } else {
        std::env::current_dir().ok()?.join(file_path_str)
    };

    let source_bytes = std::fs::read(&file_path).ok()?;
    let uri = Url::from_file_path(&file_path).ok()?;

    let start = bytes_to_pos(&source_bytes, src_loc.offset)?;
    let end = bytes_to_pos(&source_bytes, src_loc.end())?;
    let range = Range { start, end };

    let selection_range = info
        .name_location
        .as_deref()
        .and_then(|nl| {
            let nl_loc = SourceLoc::parse(nl)?;
            let ns = bytes_to_pos(&source_bytes, nl_loc.offset)?;
            let ne = bytes_to_pos(&source_bytes, nl_loc.end())?;
            Some(Range { start: ns, end: ne })
        })
        .unwrap_or(range);

    let node_type = info.node_type.as_deref().unwrap_or("");
    let kind = match node_type {
        "FunctionDefinition" | "ModifierDefinition" => SymbolKind::FUNCTION,
        "ContractDefinition" => SymbolKind::CLASS,
        _ => SymbolKind::FUNCTION,
    };

    let name = extract_name_from_source(&source_bytes, &selection_range)
        .unwrap_or_else(|| node_type.to_string());

    let data = Some(serde_json::json!({ "nodeId": node_id.0 }));

    Some(CallHierarchyItem {
        name,
        kind,
        tags: None,
        detail: None,
        uri,
        range,
        selection_range,
        data,
    })
}

/// Convert a `call_src` string to an LSP `Range`.
pub fn call_src_to_range(
    call_src: &str,
    id_to_path_map: &HashMap<SolcFileId, String>,
) -> Option<Range> {
    let loc = SourceLoc::parse(call_src)?;
    let file_path_str = id_to_path_map.get(&loc.file_id_str())?;

    let file_path = if std::path::Path::new(file_path_str).is_absolute() {
        std::path::PathBuf::from(file_path_str)
    } else {
        std::env::current_dir().ok()?.join(file_path_str)
    };

    let source_bytes = std::fs::read(&file_path).ok()?;
    let start = bytes_to_pos(&source_bytes, loc.offset)?;
    let end = bytes_to_pos(&source_bytes, loc.end())?;
    Some(Range { start, end })
}

// ── Internal helpers ───────────────────────────────────────────────────────

/// Find a node in the `nodes` index by ID, searching all files.
pub fn find_node_info<'a>(
    nodes: &'a HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
    node_id: NodeId,
) -> Option<&'a NodeInfo> {
    for file_nodes in nodes.values() {
        if let Some(info) = file_nodes.get(&node_id) {
            return Some(info);
        }
    }
    None
}

/// Resolve an absolute path to a filesystem path by searching `id_to_path_map`.
fn find_file_path(
    abs_path: &str,
    id_to_path_map: &HashMap<SolcFileId, String>,
) -> Option<std::path::PathBuf> {
    let as_path = std::path::Path::new(abs_path);
    if as_path.is_absolute() && as_path.exists() {
        return Some(as_path.to_path_buf());
    }

    for file_path in id_to_path_map.values() {
        if file_path == abs_path || file_path.ends_with(abs_path) {
            let fp = std::path::Path::new(file_path);
            if fp.is_absolute() {
                return Some(fp.to_path_buf());
            } else {
                return std::env::current_dir().ok().map(|cwd| cwd.join(fp));
            }
        }
    }

    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join(abs_path))
        .filter(|p| p.exists())
}

/// Parse a `nameLocation` string into a `Range`.
fn name_loc_range(name_location: &Option<String>, source_bytes: &[u8]) -> Option<Range> {
    let loc_str = name_location.as_deref()?;
    let loc = SourceLoc::parse(loc_str)?;
    let start = bytes_to_pos(source_bytes, loc.offset)?;
    let end = bytes_to_pos(source_bytes, loc.end())?;
    Some(Range { start, end })
}

/// Extract the text at a given range from source bytes.
fn extract_name_from_source(source_bytes: &[u8], range: &Range) -> Option<String> {
    let text = String::from_utf8_lossy(source_bytes);
    let lines: Vec<&str> = text.lines().collect();
    let line = lines.get(range.start.line as usize)?;
    let start = range.start.character as usize;
    let end = range.end.character as usize;
    if range.start.line == range.end.line && start < line.len() && end <= line.len() {
        Some(line[start..end].to_string())
    } else {
        None
    }
}

/// Resolve a callable at a cursor position.
///
/// Uses `byte_to_id()` to find the innermost node at the cursor, then:
/// 1. If the node itself is a callable declaration (FunctionDefinition,
///    ModifierDefinition, ContractDefinition), return its ID.
/// 2. If the node has `referencedDeclaration` pointing to a callable, return that.
/// 3. Walk up via scope chain to find the enclosing callable.
pub fn resolve_callable_at_position(
    build: &CachedBuild,
    abs_path: &str,
    byte_position: usize,
) -> Option<NodeId> {
    let node_id = byte_to_id(&build.nodes, abs_path, byte_position)?;
    let file_nodes = build.nodes.get(abs_path)?;
    let info = file_nodes.get(&node_id)?;
    let node_type = info.node_type.as_deref().unwrap_or("");

    // Case 1: cursor is directly on a callable declaration.
    if matches!(
        node_type,
        "FunctionDefinition" | "ModifierDefinition" | "ContractDefinition"
    ) {
        return Some(node_id);
    }

    // Case 2: the node references a callable declaration.
    if let Some(ref_id) = info.referenced_declaration {
        if let Some(decl) = build.decl_index.get(&ref_id) {
            if matches!(
                decl,
                DeclNode::FunctionDefinition(_)
                    | DeclNode::ModifierDefinition(_)
                    | DeclNode::ContractDefinition(_)
            ) {
                return Some(ref_id);
            }
        }
        if let Some(ref_info) = find_node_info(&build.nodes, ref_id) {
            let ref_type = ref_info.node_type.as_deref().unwrap_or("");
            if matches!(
                ref_type,
                "FunctionDefinition" | "ModifierDefinition" | "ContractDefinition"
            ) {
                return Some(ref_id);
            }
        }
    }

    // Case 3: find the narrowest enclosing callable by span containment.
    let mut best_callable: Option<(NodeId, usize)> = None;
    for (id, ni) in file_nodes {
        let nt = ni.node_type.as_deref().unwrap_or("");
        if !matches!(
            nt,
            "FunctionDefinition" | "ModifierDefinition" | "ContractDefinition"
        ) {
            continue;
        }
        if let Some(src_loc) = SourceLoc::parse(ni.src.as_str()) {
            if src_loc.offset <= byte_position && byte_position < src_loc.end() {
                match best_callable {
                    None => best_callable = Some((*id, src_loc.length)),
                    Some((_, best_len)) if src_loc.length < best_len => {
                        best_callable = Some((*id, src_loc.length));
                    }
                    _ => {}
                }
            }
        }
    }

    best_callable.map(|(id, _)| id)
}
