use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, CompletionResponse, Position, Range,
    TextEdit,
};

use crate::goto::CHILD_KEYS;
use crate::hover::build_function_signature;
use crate::types::{FileId, NodeId, SourceLoc};
use crate::utils::push_if_node_or_array;

/// A directly-declared top-level symbol that can be imported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopLevelImportable {
    /// Symbol name.
    pub name: String,
    /// Absolute source path where the symbol is declared.
    pub declaring_path: String,
    /// AST node type for this declaration.
    pub node_type: String,
    /// LSP completion kind mapped from the AST node type.
    pub kind: CompletionItemKind,
}

/// A declaration found within a specific scope.
#[derive(Debug, Clone)]
pub struct ScopedDeclaration {
    /// Variable/function/type name.
    pub name: String,
    /// typeIdentifier from typeDescriptions (e.g. "t_struct$_PoolKey_$8887_memory_ptr").
    pub type_id: String,
}

/// A byte range identifying a scope-creating AST node.
#[derive(Debug, Clone)]
pub struct ScopeRange {
    /// AST node id of this scope.
    pub node_id: NodeId,
    /// Byte offset where this scope starts (from `src` field).
    pub start: usize,
    /// Byte offset where this scope ends (start + length).
    pub end: usize,
    /// Source file id (from `src` field).
    pub file_id: FileId,
}

/// Completion cache built from the AST.
#[derive(Debug)]
pub struct CompletionCache {
    /// All named identifiers as completion items (flat, unscoped).
    pub names: Vec<CompletionItem>,

    /// name → typeIdentifier (for dot-completion: look up what type a variable is).
    pub name_to_type: HashMap<String, String>,

    /// node id → Vec<CompletionItem> (members of structs, contracts, enums, libraries).
    pub node_members: HashMap<NodeId, Vec<CompletionItem>>,

    /// typeIdentifier → node id (resolve a type string to its definition).
    pub type_to_node: HashMap<String, NodeId>,

    /// contract/library/interface name → node id (for direct name dot-completion like `FullMath.`).
    pub name_to_node_id: HashMap<String, NodeId>,

    /// node id → Vec<CompletionItem> from methodIdentifiers in .contracts section.
    /// Full function signatures with 4-byte selectors for contracts/interfaces.
    pub method_identifiers: HashMap<NodeId, Vec<CompletionItem>>,

    /// (contract_node_id, fn_name) → return typeIdentifier.
    /// For resolving `foo().` — look up what `foo` returns.
    pub function_return_types: HashMap<(NodeId, String), String>,

    /// typeIdentifier → Vec<CompletionItem> from UsingForDirective.
    /// Library functions available on a type via `using X for Y`.
    pub using_for: HashMap<String, Vec<CompletionItem>>,

    /// Wildcard using-for: `using X for *` — available on all types.
    pub using_for_wildcard: Vec<CompletionItem>,

    /// Pre-built general completions (AST names + keywords + globals + units).
    /// Built once, returned by reference on every non-dot completion request.
    pub general_completions: Vec<CompletionItem>,

    /// scope node_id → declarations in that scope.
    /// Each scope (Block, FunctionDefinition, ContractDefinition, SourceUnit)
    /// has the variables/functions/types declared directly within it.
    pub scope_declarations: HashMap<NodeId, Vec<ScopedDeclaration>>,

    /// node_id → parent scope node_id.
    /// Walk this chain upward to widen the search scope.
    pub scope_parent: HashMap<NodeId, NodeId>,

    /// All scope ranges, for finding which scope a byte position falls in.
    /// Sorted by span size ascending (smallest first) for efficient innermost-scope lookup.
    pub scope_ranges: Vec<ScopeRange>,

    /// absolute file path → AST source file id.
    /// Used to map a URI to the file_id needed for scope resolution.
    pub path_to_file_id: HashMap<String, FileId>,

    /// contract node_id → linearized base contracts (C3 linearization order).
    /// First element is the contract itself, followed by parents in resolution order.
    /// Used to search inherited state variables and functions during scope resolution.
    pub linearized_base_contracts: HashMap<NodeId, Vec<NodeId>>,

    /// contract/interface/library node_id → contractKind string.
    /// Values are `"contract"`, `"interface"`, or `"library"`.
    /// Used to determine which `type(X).` members to offer.
    pub contract_kinds: HashMap<NodeId, String>,

    /// Directly-declared importable top-level symbols keyed by symbol name.
    ///
    /// This intentionally excludes imported aliases/re-exports and excludes
    /// non-constant variables. It is used for import-on-completion candidate
    /// lookup without re-scanning the full AST per request.
    pub top_level_importables_by_name: HashMap<String, Vec<TopLevelImportable>>,

    /// Directly-declared importable top-level symbols keyed by declaring file path.
    ///
    /// This enables cheap incremental invalidation/update on file edits/deletes:
    /// only the changed file's symbols need to be replaced.
    pub top_level_importables_by_file: HashMap<String, Vec<TopLevelImportable>>,
}

/// Map AST nodeType to LSP CompletionItemKind.
fn node_type_to_completion_kind(node_type: &str) -> CompletionItemKind {
    match node_type {
        "FunctionDefinition" => CompletionItemKind::FUNCTION,
        "VariableDeclaration" => CompletionItemKind::VARIABLE,
        "ContractDefinition" => CompletionItemKind::CLASS,
        "StructDefinition" => CompletionItemKind::STRUCT,
        "EnumDefinition" => CompletionItemKind::ENUM,
        "EnumValue" => CompletionItemKind::ENUM_MEMBER,
        "EventDefinition" => CompletionItemKind::EVENT,
        "ErrorDefinition" => CompletionItemKind::EVENT,
        "ModifierDefinition" => CompletionItemKind::METHOD,
        "ImportDirective" => CompletionItemKind::MODULE,
        _ => CompletionItemKind::TEXT,
    }
}

/// Parse the `src` field of an AST node: "offset:length:fileId".
/// Returns the parsed SourceLoc or None if the format is invalid.
fn parse_src(node: &Value) -> Option<SourceLoc> {
    let src = node.get("src").and_then(|v| v.as_str())?;
    SourceLoc::parse(src)
}

/// Extract the trailing node id from a typeIdentifier string.
/// e.g. `t_struct$_PoolKey_$8887_storage_ptr` → Some(8887)
///      `t_contract$_IHooks_$2248` → Some(2248)
///      `t_uint256` → None
pub fn extract_node_id_from_type(type_id: &str) -> Option<NodeId> {
    // Pattern: ..._$<digits>... where digits follow the last _$
    // We find all _$<digits> groups and take the last one that's part of the type name
    let mut last_id = None;
    let mut i = 0;
    let bytes = type_id.as_bytes();
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'_' && bytes[i + 1] == b'$' {
            i += 2;
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i > start
                && let Ok(id) = type_id[start..i].parse::<u64>()
            {
                last_id = Some(NodeId(id));
            }
        } else {
            i += 1;
        }
    }
    last_id
}

/// Extract the deepest value type from a mapping typeIdentifier.
/// Peels off all `t_mapping$_<key>_$_<value>` layers and returns the innermost value type.
///
/// e.g. `t_mapping$_t_address_$_t_uint256_$` → `t_uint256`
///      `t_mapping$_t_address_$_t_mapping$_t_uint256_$_t_uint256_$_$` → `t_uint256`
///      `t_mapping$_t_userDefinedValueType$_PoolId_$8841_$_t_struct$_State_$4809_storage_$` → `t_struct$_State_$4809_storage`
pub fn extract_mapping_value_type(type_id: &str) -> Option<String> {
    let mut current = type_id;

    loop {
        if !current.starts_with("t_mapping$_") {
            // Not a mapping — this is the value type
            // Strip trailing _$ suffixes (mapping closers)
            let result = current.trim_end_matches("_$");
            return if result.is_empty() {
                None
            } else {
                Some(result.to_string())
            };
        }

        // Strip "t_mapping$_" prefix to get "<key>_$_<value>_$"
        let inner = &current["t_mapping$_".len()..];

        // Find the boundary between key and value.
        // We need to find the _$_ that separates key from value at depth 0.
        // Each $_ opens a nesting level, each _$ closes one.
        let mut depth = 0i32;
        let bytes = inner.as_bytes();
        let mut split_pos = None;

        let mut i = 0;
        while i < bytes.len() {
            if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'_' {
                depth += 1;
                i += 2;
            } else if i + 2 < bytes.len()
                && bytes[i] == b'_'
                && bytes[i + 1] == b'$'
                && bytes[i + 2] == b'_'
                && depth == 0
            {
                // This is the _$_ separator at depth 0
                split_pos = Some(i);
                break;
            } else if i + 1 < bytes.len() && bytes[i] == b'_' && bytes[i + 1] == b'$' {
                depth -= 1;
                i += 2;
            } else {
                i += 1;
            }
        }

        if let Some(pos) = split_pos {
            // Value type starts after "_$_"
            current = &inner[pos + 3..];
        } else {
            return None;
        }
    }
}

/// Count parameters in an ABI method signature like "swap((address,address),uint256,bytes)".
/// Counts commas at depth 0 (inside the outer parens), handling nested tuples.
fn count_abi_params(signature: &str) -> usize {
    // Find the first '(' and work from there
    let start = match signature.find('(') {
        Some(i) => i + 1,
        None => return 0,
    };
    let bytes = signature.as_bytes();
    if start >= bytes.len() {
        return 0;
    }
    // Check for empty params "()"
    if bytes[start] == b')' {
        return 0;
    }
    let mut count = 1; // at least one param if not empty
    let mut depth = 0;
    for &b in &bytes[start..] {
        match b {
            b'(' => depth += 1,
            b')' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            b',' if depth == 0 => count += 1,
            _ => {}
        }
    }
    count
}

/// Count parameters in an AST-derived signature like "swap(PoolKey key, SwapParams params, bytes hookData)".
fn count_signature_params(sig: &str) -> usize {
    count_abi_params(sig)
}

fn is_top_level_importable_decl(node_type: &str, node: &Value) -> bool {
    match node_type {
        "ContractDefinition"
        | "StructDefinition"
        | "EnumDefinition"
        | "UserDefinedValueTypeDefinition"
        | "FunctionDefinition" => true,
        "VariableDeclaration" => node.get("constant").and_then(|v| v.as_bool()) == Some(true),
        _ => false,
    }
}

fn build_top_level_importables_by_name(
    by_file: &HashMap<String, Vec<TopLevelImportable>>,
) -> HashMap<String, Vec<TopLevelImportable>> {
    let mut by_name: HashMap<String, Vec<TopLevelImportable>> = HashMap::new();
    for symbols in by_file.values() {
        for symbol in symbols {
            by_name
                .entry(symbol.name.clone())
                .or_default()
                .push(symbol.clone());
        }
    }
    by_name
}

/// Extract directly-declared importable top-level symbols from a file AST.
///
/// - Includes: contract/interface/library/struct/enum/UDVT/top-level free function/top-level constant
/// - Excludes: imported aliases/re-exports, nested declarations, non-constant variables
pub fn extract_top_level_importables_for_file(path: &str, ast: &Value) -> Vec<TopLevelImportable> {
    let mut out: Vec<TopLevelImportable> = Vec::new();
    let mut stack: Vec<&Value> = vec![ast];
    let mut source_unit_id: Option<NodeId> = None;

    while let Some(tree) = stack.pop() {
        let node_type = tree.get("nodeType").and_then(|v| v.as_str()).unwrap_or("");
        let node_id = tree.get("id").and_then(|v| v.as_u64()).map(NodeId);
        if node_type == "SourceUnit" {
            source_unit_id = node_id;
        }
        let name = tree.get("name").and_then(|v| v.as_str()).unwrap_or("");

        if !name.is_empty()
            && is_top_level_importable_decl(node_type, tree)
            && let Some(src_scope) = source_unit_id
            && tree.get("scope").and_then(|v| v.as_u64()) == Some(src_scope.0)
        {
            out.push(TopLevelImportable {
                name: name.to_string(),
                declaring_path: path.to_string(),
                node_type: node_type.to_string(),
                kind: node_type_to_completion_kind(node_type),
            });
        }

        for key in CHILD_KEYS {
            push_if_node_or_array(tree, key, &mut stack);
        }
    }

    out
}

impl CompletionCache {
    /// Replace top-level importables for a file path and rebuild the by-name index.
    /// Pass an empty `symbols` list when the file is deleted.
    pub fn replace_top_level_importables_for_path(
        &mut self,
        path: String,
        symbols: Vec<TopLevelImportable>,
    ) {
        self.top_level_importables_by_file.insert(path, symbols);
        self.top_level_importables_by_name =
            build_top_level_importables_by_name(&self.top_level_importables_by_file);
    }
}

/// Build a CompletionCache from AST sources and contracts.
/// `contracts` is the `.contracts` section of the compiler output (optional).
pub fn build_completion_cache(sources: &Value, contracts: Option<&Value>) -> CompletionCache {
    let source_count = sources.as_object().map_or(0, |obj| obj.len());
    // Pre-size collections based on source count to reduce rehash churn.
    // Estimates: ~20 names/file, ~5 contracts/file, ~10 functions/file.
    let est_names = source_count * 20;
    let est_contracts = source_count * 5;

    let mut names: Vec<CompletionItem> = Vec::with_capacity(est_names);
    let mut seen_names: HashMap<String, usize> = HashMap::with_capacity(est_names);
    let mut name_to_type: HashMap<String, String> = HashMap::with_capacity(est_names);
    let mut node_members: HashMap<NodeId, Vec<CompletionItem>> =
        HashMap::with_capacity(est_contracts);
    let mut type_to_node: HashMap<String, NodeId> = HashMap::with_capacity(est_contracts);
    let mut method_identifiers: HashMap<NodeId, Vec<CompletionItem>> =
        HashMap::with_capacity(est_contracts);
    let mut name_to_node_id: HashMap<String, NodeId> = HashMap::with_capacity(est_names);
    let mut contract_kinds: HashMap<NodeId, String> = HashMap::with_capacity(est_contracts);

    // Collect (path, contract_name, node_id) during AST walk for methodIdentifiers lookup after.
    let mut contract_locations: Vec<(String, String, NodeId)> = Vec::with_capacity(est_contracts);

    // contract_node_id → fn_name → Vec<signature> (for matching method_identifiers to AST signatures)
    let mut function_signatures: HashMap<NodeId, HashMap<String, Vec<String>>> =
        HashMap::with_capacity(est_contracts);

    // (contract_node_id, fn_name) → return typeIdentifier
    let mut function_return_types: HashMap<(NodeId, String), String> =
        HashMap::with_capacity(source_count * 10);

    // typeIdentifier → Vec<CompletionItem> from UsingForDirective
    let mut using_for: HashMap<String, Vec<CompletionItem>> = HashMap::with_capacity(source_count);
    let mut using_for_wildcard: Vec<CompletionItem> = Vec::new();

    // Temp: (library_node_id, target_type_id_or_none) for resolving after walk
    let mut using_for_directives: Vec<(NodeId, Option<String>)> = Vec::new();

    // Scope-aware completion data
    let mut scope_declarations: HashMap<NodeId, Vec<ScopedDeclaration>> =
        HashMap::with_capacity(est_contracts);
    let mut scope_parent: HashMap<NodeId, NodeId> = HashMap::with_capacity(est_contracts);
    let mut scope_ranges: Vec<ScopeRange> = Vec::with_capacity(est_contracts);
    let mut path_to_file_id: HashMap<String, FileId> = HashMap::with_capacity(source_count);
    let mut linearized_base_contracts: HashMap<NodeId, Vec<NodeId>> =
        HashMap::with_capacity(est_contracts);
    let mut top_level_importables_by_file: HashMap<String, Vec<TopLevelImportable>> =
        HashMap::with_capacity(est_names);

    if let Some(sources_obj) = sources.as_object() {
        for (path, source_data) in sources_obj {
            if let Some(ast) = source_data.get("ast") {
                // Map file path → source file id for scope resolution
                if let Some(fid) = source_data.get("id").and_then(|v| v.as_u64()) {
                    path_to_file_id.insert(path.clone(), FileId(fid));
                }
                let file_importables = extract_top_level_importables_for_file(path, ast);
                if !file_importables.is_empty() {
                    top_level_importables_by_file.insert(path.clone(), file_importables);
                }
                let mut stack: Vec<&Value> = vec![ast];

                while let Some(tree) = stack.pop() {
                    let node_type = tree.get("nodeType").and_then(|v| v.as_str()).unwrap_or("");
                    let name = tree.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let node_id = tree.get("id").and_then(|v| v.as_u64()).map(NodeId);

                    // --- Scope-aware data collection ---

                    // Record scope-creating nodes (SourceUnit, ContractDefinition,
                    // FunctionDefinition, ModifierDefinition, Block) and their byte ranges.
                    let is_scope_node = matches!(
                        node_type,
                        "SourceUnit"
                            | "ContractDefinition"
                            | "FunctionDefinition"
                            | "ModifierDefinition"
                            | "Block"
                            | "UncheckedBlock"
                    );
                    if is_scope_node && let Some(nid) = node_id {
                        if let Some(src_loc) = parse_src(tree) {
                            scope_ranges.push(ScopeRange {
                                node_id: nid,
                                start: src_loc.offset,
                                end: src_loc.end(),
                                file_id: src_loc.file_id,
                            });
                        }
                        // Record parent link: this node's scope → its parent
                        if let Some(parent_id) = tree.get("scope").and_then(|v| v.as_u64()) {
                            scope_parent.insert(nid, NodeId(parent_id));
                        }
                    }

                    // For ContractDefinitions, record linearizedBaseContracts
                    if node_type == "ContractDefinition"
                        && let Some(nid) = node_id
                        && let Some(bases) = tree
                            .get("linearizedBaseContracts")
                            .and_then(|v| v.as_array())
                    {
                        let base_ids: Vec<NodeId> = bases
                            .iter()
                            .filter_map(|b| b.as_u64())
                            .map(NodeId)
                            .collect();
                        if !base_ids.is_empty() {
                            linearized_base_contracts.insert(nid, base_ids);
                        }
                    }

                    // For VariableDeclarations, record the declaration in its scope
                    if node_type == "VariableDeclaration"
                        && !name.is_empty()
                        && let Some(scope_raw) = tree.get("scope").and_then(|v| v.as_u64())
                        && let Some(tid) = tree
                            .get("typeDescriptions")
                            .and_then(|td| td.get("typeIdentifier"))
                            .and_then(|v| v.as_str())
                    {
                        scope_declarations
                            .entry(NodeId(scope_raw))
                            .or_default()
                            .push(ScopedDeclaration {
                                name: name.to_string(),
                                type_id: tid.to_string(),
                            });
                    }

                    // For FunctionDefinitions, record them in their parent scope (the contract)
                    if node_type == "FunctionDefinition"
                        && !name.is_empty()
                        && let Some(scope_raw) = tree.get("scope").and_then(|v| v.as_u64())
                        && let Some(tid) = tree
                            .get("typeDescriptions")
                            .and_then(|td| td.get("typeIdentifier"))
                            .and_then(|v| v.as_str())
                    {
                        scope_declarations
                            .entry(NodeId(scope_raw))
                            .or_default()
                            .push(ScopedDeclaration {
                                name: name.to_string(),
                                type_id: tid.to_string(),
                            });
                    }

                    // Collect named nodes as completion items
                    if !name.is_empty() && !seen_names.contains_key(name) {
                        let type_string = tree
                            .get("typeDescriptions")
                            .and_then(|td| td.get("typeString"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let type_id = tree
                            .get("typeDescriptions")
                            .and_then(|td| td.get("typeIdentifier"))
                            .and_then(|v| v.as_str());

                        let kind = node_type_to_completion_kind(node_type);

                        let item = CompletionItem {
                            label: name.to_string(),
                            kind: Some(kind),
                            detail: type_string,
                            ..Default::default()
                        };

                        let idx = names.len();
                        names.push(item);
                        seen_names.insert(name.to_string(), idx);

                        // Store name → typeIdentifier mapping
                        if let Some(tid) = type_id {
                            name_to_type.insert(name.to_string(), tid.to_string());
                        }
                    }

                    // Collect struct members
                    if node_type == "StructDefinition"
                        && let Some(id) = node_id
                    {
                        let mut members = Vec::new();
                        if let Some(member_array) = tree.get("members").and_then(|v| v.as_array()) {
                            for member in member_array {
                                let member_name =
                                    member.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                if member_name.is_empty() {
                                    continue;
                                }
                                let member_type = member
                                    .get("typeDescriptions")
                                    .and_then(|td| td.get("typeString"))
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());

                                members.push(CompletionItem {
                                    label: member_name.to_string(),
                                    kind: Some(CompletionItemKind::FIELD),
                                    detail: member_type,
                                    ..Default::default()
                                });
                            }
                        }
                        if !members.is_empty() {
                            node_members.insert(id, members);
                        }

                        // Map typeIdentifier → node id
                        if let Some(tid) = tree
                            .get("typeDescriptions")
                            .and_then(|td| td.get("typeIdentifier"))
                            .and_then(|v| v.as_str())
                        {
                            type_to_node.insert(tid.to_string(), id);
                        }
                    }

                    // Collect contract/library members (functions, state variables, events, etc.)
                    if node_type == "ContractDefinition"
                        && let Some(id) = node_id
                    {
                        let mut members = Vec::new();
                        let mut fn_sigs: HashMap<String, Vec<String>> = HashMap::new();
                        if let Some(nodes_array) = tree.get("nodes").and_then(|v| v.as_array()) {
                            for member in nodes_array {
                                let member_type = member
                                    .get("nodeType")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let member_name =
                                    member.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                if member_name.is_empty() {
                                    continue;
                                }

                                // Build function signature and collect return types for FunctionDefinitions
                                let (member_detail, label_details) =
                                    if member_type == "FunctionDefinition" {
                                        // Collect return type for chain resolution.
                                        // Only single-return functions can be dot-chained
                                        // (tuples require destructuring).
                                        if let Some(ret_params) = member
                                            .get("returnParameters")
                                            .and_then(|rp| rp.get("parameters"))
                                            .and_then(|v| v.as_array())
                                            && ret_params.len() == 1
                                            && let Some(ret_tid) = ret_params[0]
                                                .get("typeDescriptions")
                                                .and_then(|td| td.get("typeIdentifier"))
                                                .and_then(|v| v.as_str())
                                        {
                                            function_return_types.insert(
                                                (id, member_name.to_string()),
                                                ret_tid.to_string(),
                                            );
                                        }

                                        if let Some(sig) = build_function_signature(member) {
                                            fn_sigs
                                                .entry(member_name.to_string())
                                                .or_default()
                                                .push(sig.clone());
                                            (Some(sig), None)
                                        } else {
                                            (
                                                member
                                                    .get("typeDescriptions")
                                                    .and_then(|td| td.get("typeString"))
                                                    .and_then(|v| v.as_str())
                                                    .map(|s| s.to_string()),
                                                None,
                                            )
                                        }
                                    } else {
                                        (
                                            member
                                                .get("typeDescriptions")
                                                .and_then(|td| td.get("typeString"))
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string()),
                                            None,
                                        )
                                    };

                                let kind = node_type_to_completion_kind(member_type);
                                members.push(CompletionItem {
                                    label: member_name.to_string(),
                                    kind: Some(kind),
                                    detail: member_detail,
                                    label_details,
                                    ..Default::default()
                                });
                            }
                        }
                        if !members.is_empty() {
                            node_members.insert(id, members);
                        }
                        if !fn_sigs.is_empty() {
                            function_signatures.insert(id, fn_sigs);
                        }

                        if let Some(tid) = tree
                            .get("typeDescriptions")
                            .and_then(|td| td.get("typeIdentifier"))
                            .and_then(|v| v.as_str())
                        {
                            type_to_node.insert(tid.to_string(), id);
                        }

                        // Record for methodIdentifiers lookup after traversal
                        if !name.is_empty() {
                            contract_locations.push((path.clone(), name.to_string(), id));
                            name_to_node_id.insert(name.to_string(), id);
                        }

                        // Record contractKind (contract, interface, library) for type(X). completions
                        if let Some(ck) = tree.get("contractKind").and_then(|v| v.as_str()) {
                            contract_kinds.insert(id, ck.to_string());
                        }
                    }

                    // Collect enum members
                    if node_type == "EnumDefinition"
                        && let Some(id) = node_id
                    {
                        let mut members = Vec::new();
                        if let Some(member_array) = tree.get("members").and_then(|v| v.as_array()) {
                            for member in member_array {
                                let member_name =
                                    member.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                if member_name.is_empty() {
                                    continue;
                                }
                                members.push(CompletionItem {
                                    label: member_name.to_string(),
                                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                                    detail: None,
                                    ..Default::default()
                                });
                            }
                        }
                        if !members.is_empty() {
                            node_members.insert(id, members);
                        }

                        if let Some(tid) = tree
                            .get("typeDescriptions")
                            .and_then(|td| td.get("typeIdentifier"))
                            .and_then(|v| v.as_str())
                        {
                            type_to_node.insert(tid.to_string(), id);
                        }
                    }

                    // Collect UsingForDirective: using Library for Type
                    if node_type == "UsingForDirective" {
                        // Get target type (None = wildcard `for *`)
                        let target_type = tree.get("typeName").and_then(|tn| {
                            tn.get("typeDescriptions")
                                .and_then(|td| td.get("typeIdentifier"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        });

                        // Form 1: library name object with referencedDeclaration
                        if let Some(lib) = tree.get("libraryName") {
                            if let Some(lib_id) =
                                lib.get("referencedDeclaration").and_then(|v| v.as_u64())
                            {
                                using_for_directives.push((NodeId(lib_id), target_type));
                            }
                        }
                        // Form 2: functionList array — individual function references
                        // These are typically operator overloads (not dot-callable),
                        // but collect non-operator ones just in case
                        else if let Some(func_list) =
                            tree.get("functionList").and_then(|v| v.as_array())
                        {
                            for entry in func_list {
                                // Skip operator overloads
                                if entry.get("operator").is_some() {
                                    continue;
                                }
                                if let Some(def) = entry.get("definition") {
                                    let fn_name =
                                        def.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                    if !fn_name.is_empty() {
                                        let items = if let Some(ref tid) = target_type {
                                            using_for.entry(tid.clone()).or_default()
                                        } else {
                                            &mut using_for_wildcard
                                        };
                                        items.push(CompletionItem {
                                            label: fn_name.to_string(),
                                            kind: Some(CompletionItemKind::FUNCTION),
                                            detail: None,
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                        }
                    }

                    // Traverse children
                    for key in CHILD_KEYS {
                        push_if_node_or_array(tree, key, &mut stack);
                    }
                }
            }
        }
    }

    // Resolve UsingForDirective library references (Form 1)
    // Now that node_members is populated, look up each library's functions
    for (lib_id, target_type) in &using_for_directives {
        if let Some(lib_members) = node_members.get(lib_id) {
            let items: Vec<CompletionItem> = lib_members
                .iter()
                .filter(|item| item.kind == Some(CompletionItemKind::FUNCTION))
                .cloned()
                .collect();
            if !items.is_empty() {
                if let Some(tid) = target_type {
                    using_for.entry(tid.clone()).or_default().extend(items);
                } else {
                    using_for_wildcard.extend(items);
                }
            }
        }
    }

    // Build method_identifiers from .contracts section
    if let Some(contracts_val) = contracts
        && let Some(contracts_obj) = contracts_val.as_object()
    {
        for (path, contract_name, node_id) in &contract_locations {
            // Get AST function signatures for this contract (if available)
            let fn_sigs = function_signatures.get(node_id);

            if let Some(path_entry) = contracts_obj.get(path)
                && let Some(contract_entry) = path_entry.get(contract_name)
                && let Some(evm) = contract_entry.get("evm")
                && let Some(methods) = evm.get("methodIdentifiers")
                && let Some(methods_obj) = methods.as_object()
            {
                let mut items: Vec<CompletionItem> = Vec::new();
                for (signature, selector_val) in methods_obj {
                    // signature is e.g. "swap((address,address,uint24,int24,address),(bool,int256,uint160),bytes)"
                    // selector_val is e.g. "f3cd914c"
                    let fn_name = signature.split('(').next().unwrap_or(signature).to_string();
                    let selector_str = selector_val
                        .as_str()
                        .map(|s| crate::types::FuncSelector::new(s).to_prefixed())
                        .unwrap_or_default();

                    // Look up the AST signature with parameter names
                    let description =
                        fn_sigs
                            .and_then(|sigs| sigs.get(&fn_name))
                            .and_then(|sig_list| {
                                if sig_list.len() == 1 {
                                    // Only one overload — use it directly
                                    Some(sig_list[0].clone())
                                } else {
                                    // Multiple overloads — match by parameter count
                                    let abi_param_count = count_abi_params(signature);
                                    sig_list
                                        .iter()
                                        .find(|s| count_signature_params(s) == abi_param_count)
                                        .cloned()
                                }
                            });

                    items.push(CompletionItem {
                        label: fn_name,
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(signature.clone()),
                        label_details: Some(tower_lsp::lsp_types::CompletionItemLabelDetails {
                            detail: Some(selector_str),
                            description,
                        }),
                        ..Default::default()
                    });
                }
                if !items.is_empty() {
                    method_identifiers.insert(*node_id, items);
                }
            }
        }
    }

    // Pre-build the general completions list (names + statics) once
    let mut general_completions = names.clone();
    general_completions.extend(get_static_completions());

    // Sort scope_ranges by span size ascending (smallest first) for innermost-scope lookup
    scope_ranges.sort_by_key(|r| r.end - r.start);

    // Infer parent links for Block/UncheckedBlock/ModifierDefinition nodes.
    // These AST nodes have no `scope` field, so scope_parent has no entry for them.
    // For each orphan, find the next-larger enclosing scope range in the same file.
    // scope_ranges is sorted smallest-first, so we scan forward from each orphan's
    // position to find the first range that strictly contains it.
    let orphan_ids: Vec<NodeId> = scope_ranges
        .iter()
        .filter(|r| !scope_parent.contains_key(&r.node_id))
        .map(|r| r.node_id)
        .collect();
    // Build a lookup from node_id → (start, end, file_id) for quick access
    let range_by_id: HashMap<NodeId, (usize, usize, FileId)> = scope_ranges
        .iter()
        .map(|r| (r.node_id, (r.start, r.end, r.file_id)))
        .collect();
    for orphan_id in &orphan_ids {
        if let Some(&(start, end, file_id)) = range_by_id.get(orphan_id) {
            // Find the smallest range that strictly contains this orphan's range
            // (same file, starts at or before, ends at or after, and is strictly larger)
            let parent = scope_ranges
                .iter()
                .find(|r| {
                    r.node_id != *orphan_id
                        && r.file_id == file_id
                        && r.start <= start
                        && r.end >= end
                        && (r.end - r.start) > (end - start)
                })
                .map(|r| r.node_id);
            if let Some(parent_id) = parent {
                scope_parent.insert(*orphan_id, parent_id);
            }
        }
    }

    let top_level_importables_by_name =
        build_top_level_importables_by_name(&top_level_importables_by_file);

    CompletionCache {
        names,
        name_to_type,
        node_members,
        type_to_node,
        name_to_node_id,
        method_identifiers,
        function_return_types,
        using_for,
        using_for_wildcard,
        general_completions,
        scope_declarations,
        scope_parent,
        scope_ranges,
        path_to_file_id,
        linearized_base_contracts,
        contract_kinds,
        top_level_importables_by_name,
        top_level_importables_by_file,
    }
}

/// Magic type member definitions (msg, block, tx, abi, address).
fn magic_members(name: &str) -> Option<Vec<CompletionItem>> {
    let items = match name {
        "msg" => vec![
            ("data", "bytes calldata"),
            ("sender", "address"),
            ("sig", "bytes4"),
            ("value", "uint256"),
        ],
        "block" => vec![
            ("basefee", "uint256"),
            ("blobbasefee", "uint256"),
            ("chainid", "uint256"),
            ("coinbase", "address payable"),
            ("difficulty", "uint256"),
            ("gaslimit", "uint256"),
            ("number", "uint256"),
            ("prevrandao", "uint256"),
            ("timestamp", "uint256"),
        ],
        "tx" => vec![("gasprice", "uint256"), ("origin", "address")],
        "abi" => vec![
            ("decode(bytes memory, (...))", "..."),
            ("encode(...)", "bytes memory"),
            ("encodePacked(...)", "bytes memory"),
            ("encodeWithSelector(bytes4, ...)", "bytes memory"),
            ("encodeWithSignature(string memory, ...)", "bytes memory"),
            ("encodeCall(function, (...))", "bytes memory"),
        ],
        // type(X) — contract type properties
        // Also includes interface (interfaceId) and integer (min, max) properties
        "type" => vec![
            ("name", "string"),
            ("creationCode", "bytes memory"),
            ("runtimeCode", "bytes memory"),
            ("interfaceId", "bytes4"),
            ("min", "T"),
            ("max", "T"),
        ],
        // bytes and string type-level members
        "bytes" => vec![("concat(...)", "bytes memory")],
        "string" => vec![("concat(...)", "string memory")],
        _ => return None,
    };

    Some(
        items
            .into_iter()
            .map(|(label, detail)| CompletionItem {
                label: label.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some(detail.to_string()),
                ..Default::default()
            })
            .collect(),
    )
}

/// The kind of argument passed to `type(X)`, which determines which
/// meta-type members are available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeMetaKind {
    /// `type(SomeContract)` — has `name`, `creationCode`, `runtimeCode`
    Contract,
    /// `type(SomeInterface)` — has `name`, `interfaceId`
    Interface,
    /// `type(uint256)` / `type(int8)` — has `min`, `max`
    IntegerType,
    /// Unknown argument — return all possible members as a fallback
    Unknown,
}

/// Classify the argument of `type(X)` based on the cache.
fn classify_type_arg(arg: &str, cache: Option<&CompletionCache>) -> TypeMetaKind {
    // Check if it's an integer type: int, uint, int8..int256, uint8..uint256
    if arg == "int" || arg == "uint" {
        return TypeMetaKind::IntegerType;
    }
    if let Some(suffix) = arg.strip_prefix("uint").or_else(|| arg.strip_prefix("int"))
        && let Ok(n) = suffix.parse::<u16>()
        && (8..=256).contains(&n)
        && n % 8 == 0
    {
        return TypeMetaKind::IntegerType;
    }

    // With a cache, look up the name to determine contract vs interface
    if let Some(c) = cache
        && let Some(&node_id) = c.name_to_node_id.get(arg)
    {
        return match c.contract_kinds.get(&node_id).map(|s| s.as_str()) {
            Some("interface") => TypeMetaKind::Interface,
            Some("library") => TypeMetaKind::Contract, // libraries have name/creationCode/runtimeCode
            _ => TypeMetaKind::Contract,
        };
    }

    TypeMetaKind::Unknown
}

/// Return context-sensitive `type(X).` completions based on what `X` is.
fn type_meta_members(arg: Option<&str>, cache: Option<&CompletionCache>) -> Vec<CompletionItem> {
    let kind = match arg {
        Some(a) => classify_type_arg(a, cache),
        None => TypeMetaKind::Unknown,
    };

    let items: Vec<(&str, &str)> = match kind {
        TypeMetaKind::Contract => vec![
            ("name", "string"),
            ("creationCode", "bytes memory"),
            ("runtimeCode", "bytes memory"),
        ],
        TypeMetaKind::Interface => vec![("name", "string"), ("interfaceId", "bytes4")],
        TypeMetaKind::IntegerType => vec![("min", "T"), ("max", "T")],
        TypeMetaKind::Unknown => vec![
            ("name", "string"),
            ("creationCode", "bytes memory"),
            ("runtimeCode", "bytes memory"),
            ("interfaceId", "bytes4"),
            ("min", "T"),
            ("max", "T"),
        ],
    };

    items
        .into_iter()
        .map(|(label, detail)| CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(detail.to_string()),
            ..Default::default()
        })
        .collect()
}

/// Address type members (available on any address value).
fn address_members() -> Vec<CompletionItem> {
    [
        ("balance", "uint256", CompletionItemKind::PROPERTY),
        ("code", "bytes memory", CompletionItemKind::PROPERTY),
        ("codehash", "bytes32", CompletionItemKind::PROPERTY),
        ("transfer(uint256)", "", CompletionItemKind::FUNCTION),
        ("send(uint256)", "bool", CompletionItemKind::FUNCTION),
        (
            "call(bytes memory)",
            "(bool, bytes memory)",
            CompletionItemKind::FUNCTION,
        ),
        (
            "delegatecall(bytes memory)",
            "(bool, bytes memory)",
            CompletionItemKind::FUNCTION,
        ),
        (
            "staticcall(bytes memory)",
            "(bool, bytes memory)",
            CompletionItemKind::FUNCTION,
        ),
    ]
    .iter()
    .map(|(label, detail, kind)| CompletionItem {
        label: label.to_string(),
        kind: Some(*kind),
        detail: if detail.is_empty() {
            None
        } else {
            Some(detail.to_string())
        },
        ..Default::default()
    })
    .collect()
}

/// What kind of access precedes the dot.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessKind {
    /// Plain identifier: `foo.`
    Plain,
    /// Function call: `foo().` or `foo(x, bar()).`
    Call,
    /// Index/storage access: `foo[key].` or `foo[func()].`
    Index,
}

/// A segment of a dot-expression chain.
#[derive(Debug, Clone, PartialEq)]
pub struct DotSegment {
    pub name: String,
    pub kind: AccessKind,
    /// For `Call` segments, the raw text inside the parentheses.
    /// e.g. `type(uint256).` → `call_args = Some("uint256")`
    pub call_args: Option<String>,
}

/// Skip backwards over a matched bracket pair (parens or square brackets).
/// `pos` should point to the closing bracket. Returns the position of the matching
/// opening bracket, or 0 if not found.
fn skip_brackets_backwards(bytes: &[u8], pos: usize) -> usize {
    let close = bytes[pos];
    let open = match close {
        b')' => b'(',
        b']' => b'[',
        _ => return pos,
    };
    let mut depth = 1u32;
    let mut i = pos;
    while i > 0 && depth > 0 {
        i -= 1;
        if bytes[i] == close {
            depth += 1;
        } else if bytes[i] == open {
            depth -= 1;
        }
    }
    i
}

/// Parse the expression chain before the dot into segments.
/// e.g. `poolManager.swap(key, params).` → [("poolManager", Plain), ("swap", Call)]
///      `_pools[poolId].fee.` → [("_pools", Index), ("fee", Plain)]
///      `msg.` → [("msg", Plain)]
pub fn parse_dot_chain(line: &str, character: u32) -> Vec<DotSegment> {
    let col = character as usize;
    if col == 0 {
        return vec![];
    }

    let bytes = line.as_bytes();
    let mut segments: Vec<DotSegment> = Vec::new();

    // Start from the cursor position, skip trailing dot
    let mut pos = col;
    if pos > 0 && pos <= bytes.len() && bytes[pos - 1] == b'.' {
        pos -= 1;
    }

    loop {
        if pos == 0 {
            break;
        }

        // Determine access kind by what's immediately before: ')' = Call, ']' = Index, else Plain
        let (kind, call_args) = if bytes[pos - 1] == b')' {
            let close = pos - 1; // position of ')'
            pos = skip_brackets_backwards(bytes, close);
            // Extract the text between '(' and ')'
            let args_text = String::from_utf8_lossy(&bytes[pos + 1..close])
                .trim()
                .to_string();
            let args = if args_text.is_empty() {
                None
            } else {
                Some(args_text)
            };
            (AccessKind::Call, args)
        } else if bytes[pos - 1] == b']' {
            pos -= 1; // point to ']'
            pos = skip_brackets_backwards(bytes, pos);
            (AccessKind::Index, None)
        } else {
            (AccessKind::Plain, None)
        };

        // Now extract the identifier name (walk backwards over alphanumeric + underscore)
        let end = pos;
        while pos > 0 && (bytes[pos - 1].is_ascii_alphanumeric() || bytes[pos - 1] == b'_') {
            pos -= 1;
        }

        if pos == end {
            // No identifier found (could be something like `().`)
            break;
        }

        let name = String::from_utf8_lossy(&bytes[pos..end]).to_string();
        segments.push(DotSegment {
            name,
            kind,
            call_args,
        });

        // Check if there's a dot before this segment (meaning more chain)
        if pos > 0 && bytes[pos - 1] == b'.' {
            pos -= 1; // skip the dot, continue parsing next segment
        } else {
            break;
        }
    }

    segments.reverse(); // We parsed right-to-left, flip to left-to-right
    segments
}

/// Extract the identifier before the cursor (the word before the dot).
/// Returns just the last identifier name for backward compatibility.
pub fn extract_identifier_before_dot(line: &str, character: u32) -> Option<String> {
    let segments = parse_dot_chain(line, character);
    segments.last().map(|s| s.name.clone())
}

#[doc = r"Strip all storage/memory location suffixes from a typeIdentifier to get the base type.
Solidity AST uses different suffixes in different contexts:
  - `t_struct$_State_$4809_storage_ptr` (UsingForDirective typeName)
  - `t_struct$_State_$4809_storage` (mapping value type after extraction)
  - `t_struct$_PoolKey_$8887_memory_ptr` (function parameter)
All refer to the same logical type. This strips `_ptr` and `_storage`/`_memory`/`_calldata`."]
fn strip_type_suffix(type_id: &str) -> &str {
    let s = type_id.strip_suffix("_ptr").unwrap_or(type_id);
    s.strip_suffix("_storage")
        .or_else(|| s.strip_suffix("_memory"))
        .or_else(|| s.strip_suffix("_calldata"))
        .unwrap_or(s)
}

/// Look up using-for completions for a type, trying suffix variants.
/// The AST stores types with different suffixes (_storage_ptr, _storage, _memory_ptr, etc.)
/// across different contexts, so we try multiple forms.
fn lookup_using_for(cache: &CompletionCache, type_id: &str) -> Vec<CompletionItem> {
    // Exact match first
    if let Some(items) = cache.using_for.get(type_id) {
        return items.clone();
    }

    // Strip to base form, then try all common suffix variants
    let base = strip_type_suffix(type_id);
    let variants = [
        base.to_string(),
        format!("{}_storage", base),
        format!("{}_storage_ptr", base),
        format!("{}_memory", base),
        format!("{}_memory_ptr", base),
        format!("{}_calldata", base),
    ];
    for variant in &variants {
        if variant.as_str() != type_id
            && let Some(items) = cache.using_for.get(variant.as_str())
        {
            return items.clone();
        }
    }

    vec![]
}

/// Collect completions available for a given typeIdentifier.
/// Includes node_members, method_identifiers, using_for, and using_for_wildcard.
fn completions_for_type(cache: &CompletionCache, type_id: &str) -> Vec<CompletionItem> {
    // Address type
    if type_id == "t_address" || type_id == "t_address_payable" {
        let mut items = address_members();
        // Also add using-for on address
        if let Some(uf) = cache.using_for.get(type_id) {
            items.extend(uf.iter().cloned());
        }
        items.extend(cache.using_for_wildcard.iter().cloned());
        return items;
    }

    let resolved_node_id = extract_node_id_from_type(type_id)
        .or_else(|| cache.type_to_node.get(type_id).copied())
        .or_else(|| {
            // Handle synthetic __node_id_ markers from name_to_node_id fallback
            type_id
                .strip_prefix("__node_id_")
                .and_then(|s| s.parse::<u64>().ok())
                .map(NodeId)
        });

    let mut items = Vec::new();
    let mut seen_labels: std::collections::HashSet<String> = std::collections::HashSet::new();

    if let Some(node_id) = resolved_node_id {
        // Method identifiers first — they have full signatures with selectors
        if let Some(method_items) = cache.method_identifiers.get(&node_id) {
            for item in method_items {
                seen_labels.insert(item.label.clone());
                items.push(item.clone());
            }
        }

        // Supplement with node_members (state variables, events, errors, modifiers, etc.)
        if let Some(members) = cache.node_members.get(&node_id) {
            for item in members {
                if !seen_labels.contains(&item.label) {
                    seen_labels.insert(item.label.clone());
                    items.push(item.clone());
                }
            }
        }
    }

    // Add using-for library functions, but only for value types — not for
    // contract/library/interface names. When you type `Lock.`, you want Lock's
    // own members, not functions from `using Pool for *` or `using SafeCast for *`.
    let is_contract_name = resolved_node_id
        .map(|nid| cache.contract_kinds.contains_key(&nid))
        .unwrap_or(false);

    if !is_contract_name {
        // Try exact match first, then try normalized variants (storage_ptr vs storage vs memory_ptr etc.)
        let uf_items = lookup_using_for(cache, type_id);
        for item in &uf_items {
            if !seen_labels.contains(&item.label) {
                seen_labels.insert(item.label.clone());
                items.push(item.clone());
            }
        }

        // Add wildcard using-for (using X for *)
        for item in &cache.using_for_wildcard {
            if !seen_labels.contains(&item.label) {
                seen_labels.insert(item.label.clone());
                items.push(item.clone());
            }
        }
    }

    items
}

/// Resolve a type identifier for a name, considering name_to_type and name_to_node_id.
fn resolve_name_to_type_id(cache: &CompletionCache, name: &str) -> Option<String> {
    // Direct type lookup
    if let Some(tid) = cache.name_to_type.get(name) {
        return Some(tid.clone());
    }
    // Contract/library/interface name → synthesize a type id from node id
    if let Some(node_id) = cache.name_to_node_id.get(name) {
        // Find a matching typeIdentifier in type_to_node (reverse lookup)
        for (tid, nid) in &cache.type_to_node {
            if nid == node_id {
                return Some(tid.clone());
            }
        }
        // Fallback: use a synthetic marker so completions_for_type can resolve via node_id
        return Some(format!("__node_id_{}", node_id));
    }
    None
}

/// Find the innermost scope node that contains the given byte position and file.
/// `scope_ranges` must be sorted by span size ascending (smallest first).
/// Returns the node_id of the smallest scope enclosing the position.
pub fn find_innermost_scope(
    cache: &CompletionCache,
    byte_pos: usize,
    file_id: FileId,
) -> Option<NodeId> {
    // scope_ranges is sorted smallest-first, so the first match is the innermost scope
    cache
        .scope_ranges
        .iter()
        .find(|r| r.file_id == file_id && r.start <= byte_pos && byte_pos < r.end)
        .map(|r| r.node_id)
}

/// Resolve a variable name to its type by walking up the scope chain.
///
/// Starting from the innermost scope at the cursor position, check each scope's
/// declarations for a matching name. If not found, follow `scope_parent` to the
/// next enclosing scope and check again. Stop at the first match.
///
/// Falls back to `resolve_name_to_type_id` (flat lookup) if scope resolution
/// finds nothing, or if the scope data is unavailable.
pub fn resolve_name_in_scope(
    cache: &CompletionCache,
    name: &str,
    byte_pos: usize,
    file_id: FileId,
) -> Option<String> {
    let mut current_scope = find_innermost_scope(cache, byte_pos, file_id)?;

    // Walk up the scope chain
    loop {
        // Check declarations in this scope
        if let Some(decls) = cache.scope_declarations.get(&current_scope) {
            for decl in decls {
                if decl.name == name {
                    return Some(decl.type_id.clone());
                }
            }
        }

        // If this scope is a contract, also search inherited contracts
        // in C3 linearization order (skipping index 0 which is the contract itself,
        // since we already checked its declarations above).
        if let Some(bases) = cache.linearized_base_contracts.get(&current_scope) {
            for &base_id in bases.iter().skip(1) {
                if let Some(decls) = cache.scope_declarations.get(&base_id) {
                    for decl in decls {
                        if decl.name == name {
                            return Some(decl.type_id.clone());
                        }
                    }
                }
            }
        }

        // Move up to parent scope
        match cache.scope_parent.get(&current_scope) {
            Some(&parent_id) => current_scope = parent_id,
            None => break, // reached the top (SourceUnit has no parent)
        }
    }

    // Scope walk found nothing — fall back to flat lookup
    // (handles contract/library names which aren't in scope_declarations)
    resolve_name_to_type_id(cache, name)
}

/// Resolve a name within a type context to get the member's type.
/// `context_type_id` is the type of the object before the dot.
/// `member_name` is the name after the dot.
/// `kind` determines how to interpret the result (Call = return type, Index = mapping value, Plain = member type).
fn resolve_member_type(
    cache: &CompletionCache,
    context_type_id: &str,
    member_name: &str,
    kind: &AccessKind,
) -> Option<String> {
    let resolved_node_id = extract_node_id_from_type(context_type_id)
        .or_else(|| cache.type_to_node.get(context_type_id).copied())
        .or_else(|| {
            // Handle synthetic __node_id_ markers
            context_type_id
                .strip_prefix("__node_id_")
                .and_then(|s| s.parse::<u64>().ok())
                .map(NodeId)
        });

    let node_id = resolved_node_id?;

    match kind {
        AccessKind::Call => {
            // Look up the function's return type
            cache
                .function_return_types
                .get(&(node_id, member_name.to_string()))
                .cloned()
        }
        AccessKind::Index => {
            // Look up the member's type, then extract mapping value type
            if let Some(members) = cache.node_members.get(&node_id) {
                for member in members {
                    if member.label == member_name {
                        // Get the typeIdentifier from name_to_type
                        if let Some(tid) = cache.name_to_type.get(member_name) {
                            if tid.starts_with("t_mapping") {
                                return extract_mapping_value_type(tid);
                            }
                            return Some(tid.clone());
                        }
                    }
                }
            }
            // Also check: the identifier itself might be a mapping variable
            if let Some(tid) = cache.name_to_type.get(member_name)
                && tid.starts_with("t_mapping")
            {
                return extract_mapping_value_type(tid);
            }
            None
        }
        AccessKind::Plain => {
            // Look up member's own type from name_to_type
            cache.name_to_type.get(member_name).cloned()
        }
    }
}

/// Scope context for scope-aware completion resolution.
/// When present, type resolution uses the scope chain at the cursor position
/// instead of the flat first-wins `name_to_type` map.
pub struct ScopeContext {
    /// Byte offset of the cursor in the source file.
    pub byte_pos: usize,
    /// Source file id (from the AST `src` field).
    pub file_id: FileId,
}

/// Resolve a name to a type, using scope context if available.
/// With scope context: walks up the scope chain from the cursor position.
/// Without: falls back to flat `name_to_type` lookup.
fn resolve_name(
    cache: &CompletionCache,
    name: &str,
    scope_ctx: Option<&ScopeContext>,
) -> Option<String> {
    if let Some(ctx) = scope_ctx {
        resolve_name_in_scope(cache, name, ctx.byte_pos, ctx.file_id)
    } else {
        resolve_name_to_type_id(cache, name)
    }
}

/// Get completions for a dot-completion request by resolving the full expression chain.
pub fn get_dot_completions(
    cache: &CompletionCache,
    identifier: &str,
    scope_ctx: Option<&ScopeContext>,
) -> Vec<CompletionItem> {
    // Simple single-segment case (backward compat) — just use the identifier directly
    if let Some(items) = magic_members(identifier) {
        return items;
    }

    // Try to resolve the identifier's type
    let type_id = resolve_name(cache, identifier, scope_ctx);

    if let Some(tid) = type_id {
        return completions_for_type(cache, &tid);
    }

    vec![]
}

/// Get completions by resolving a full dot-expression chain.
/// This is the main entry point for dot-completions with chaining support.
pub fn get_chain_completions(
    cache: &CompletionCache,
    chain: &[DotSegment],
    scope_ctx: Option<&ScopeContext>,
) -> Vec<CompletionItem> {
    if chain.is_empty() {
        return vec![];
    }

    // Single segment: simple dot-completion
    if chain.len() == 1 {
        let seg = &chain[0];

        // For Call/Index on the single segment, we need to resolve the return/value type
        match seg.kind {
            AccessKind::Plain => {
                return get_dot_completions(cache, &seg.name, scope_ctx);
            }
            AccessKind::Call => {
                // type(X). — Solidity metatype expression
                if seg.name == "type" {
                    return type_meta_members(seg.call_args.as_deref(), Some(cache));
                }
                // foo(). — could be a function call or a type cast like IFoo(addr).
                // First check if it's a type cast: name matches a contract/interface/library
                if let Some(type_id) = resolve_name(cache, &seg.name, scope_ctx) {
                    return completions_for_type(cache, &type_id);
                }
                // Otherwise look up as a function call — check all function_return_types
                for ((_, fn_name), ret_type) in &cache.function_return_types {
                    if fn_name == &seg.name {
                        return completions_for_type(cache, ret_type);
                    }
                }
                return vec![];
            }
            AccessKind::Index => {
                // foo[key]. — look up foo's type and extract mapping value type
                if let Some(tid) = resolve_name(cache, &seg.name, scope_ctx)
                    && tid.starts_with("t_mapping")
                    && let Some(val_type) = extract_mapping_value_type(&tid)
                {
                    return completions_for_type(cache, &val_type);
                }
                return vec![];
            }
        }
    }

    // Multi-segment chain: resolve step by step
    // First segment: resolve to a type (scope-aware when available)
    let first = &chain[0];
    let mut current_type = match first.kind {
        AccessKind::Plain => resolve_name(cache, &first.name, scope_ctx),
        AccessKind::Call => {
            // Type cast (e.g. IFoo(addr).) or free function call at the start
            resolve_name(cache, &first.name, scope_ctx).or_else(|| {
                cache
                    .function_return_types
                    .iter()
                    .find(|((_, fn_name), _)| fn_name == &first.name)
                    .map(|(_, ret_type)| ret_type.clone())
            })
        }
        AccessKind::Index => {
            // Mapping access at the start
            resolve_name(cache, &first.name, scope_ctx).and_then(|tid| {
                if tid.starts_with("t_mapping") {
                    extract_mapping_value_type(&tid)
                } else {
                    Some(tid)
                }
            })
        }
    };

    // Middle segments: resolve each to advance the type
    for seg in &chain[1..] {
        let ctx_type = match &current_type {
            Some(t) => t.clone(),
            None => return vec![],
        };

        current_type = resolve_member_type(cache, &ctx_type, &seg.name, &seg.kind);
    }

    // Return completions for the final resolved type
    match current_type {
        Some(tid) => completions_for_type(cache, &tid),
        None => vec![],
    }
}

/// Get static completions that never change (keywords, magic globals, global functions, units).
/// These are available immediately without an AST cache.
pub fn get_static_completions() -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Add Solidity keywords
    for kw in SOLIDITY_KEYWORDS {
        items.push(CompletionItem {
            label: kw.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        });
    }

    // Add magic globals
    for (name, detail) in MAGIC_GLOBALS {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some(detail.to_string()),
            ..Default::default()
        });
    }

    // Add global functions
    for (name, detail) in GLOBAL_FUNCTIONS {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(detail.to_string()),
            ..Default::default()
        });
    }

    // Add ether denomination units
    for (name, detail) in ETHER_UNITS {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::UNIT),
            detail: Some(detail.to_string()),
            ..Default::default()
        });
    }

    // Add time units
    for (name, detail) in TIME_UNITS {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::UNIT),
            detail: Some(detail.to_string()),
            ..Default::default()
        });
    }

    items
}

/// Get general completions (all known names + static completions).
pub fn get_general_completions(cache: &CompletionCache) -> Vec<CompletionItem> {
    let mut items = cache.names.clone();
    items.extend(get_static_completions());
    items
}

/// Append auto-import candidates at the tail of completion results.
///
/// This enforces lower priority ordering by:
/// 1) appending after base completions
/// 2) assigning high `sortText` values (`zz_autoimport_*`) when missing
pub fn append_auto_import_candidates_last(
    mut base: Vec<CompletionItem>,
    mut auto_import_candidates: Vec<CompletionItem>,
) -> Vec<CompletionItem> {
    let mut unique_label_edits: HashMap<String, Option<Vec<TextEdit>>> = HashMap::new();
    for item in &auto_import_candidates {
        let entry = unique_label_edits
            .entry(item.label.clone())
            .or_insert_with(|| item.additional_text_edits.clone());
        if *entry != item.additional_text_edits {
            *entry = None;
        }
    }

    // If a label maps to exactly one import edit, attach it to the corresponding
    // base completion item too. This ensures accepting the normal item can still
    // apply import edits in clients that de-prioritize or collapse duplicate labels.
    for item in &mut base {
        if item.additional_text_edits.is_none()
            && let Some(Some(edits)) = unique_label_edits.get(&item.label)
        {
            item.additional_text_edits = Some(edits.clone());
        }
    }

    for (idx, item) in auto_import_candidates.iter_mut().enumerate() {
        if item.sort_text.is_none() {
            item.sort_text = Some(format!("zz_autoimport_{idx:06}"));
        }
    }

    base.extend(auto_import_candidates);
    base
}

/// Convert cached top-level importable symbols into completion items.
///
/// These are intended as low-priority tail candidates appended after normal
/// per-file completions.
pub fn top_level_importable_completion_candidates(
    cache: &CompletionCache,
    current_file_path: Option<&str>,
    source_text: &str,
) -> Vec<CompletionItem> {
    let mut out = Vec::new();
    for symbols in cache.top_level_importables_by_name.values() {
        for symbol in symbols {
            if let Some(cur) = current_file_path
                && cur == symbol.declaring_path
            {
                continue;
            }

            let import_path = match current_file_path.and_then(|cur| {
                to_relative_import_path(Path::new(cur), Path::new(&symbol.declaring_path))
            }) {
                Some(p) => p,
                None => continue,
            };

            if import_statement_already_present(source_text, &symbol.name, &import_path) {
                continue;
            }

            let import_edit = build_import_text_edit(source_text, &symbol.name, &import_path);
            out.push(CompletionItem {
                label: symbol.name.clone(),
                kind: Some(symbol.kind),
                detail: Some(format!("{} ({import_path})", symbol.node_type)),
                additional_text_edits: import_edit.map(|e| vec![e]),
                ..Default::default()
            });
        }
    }
    out
}

fn to_relative_import_path(current_file: &Path, target_file: &Path) -> Option<String> {
    let from_dir = current_file.parent()?;
    let rel = pathdiff::diff_paths(target_file, from_dir)?;
    let mut s = rel.to_string_lossy().replace('\\', "/");
    if !s.starts_with("./") && !s.starts_with("../") {
        s = format!("./{s}");
    }
    Some(s)
}

fn import_statement_already_present(source_text: &str, symbol: &str, import_path: &str) -> bool {
    let named = format!("import {{{symbol}}} from \"{import_path}\";");
    let full = format!("import \"{import_path}\";");
    source_text.contains(&named) || source_text.contains(&full)
}

fn build_import_text_edit(source_text: &str, symbol: &str, import_path: &str) -> Option<TextEdit> {
    let import_stmt = format!("import {{{symbol}}} from \"{import_path}\";\n");
    let lines: Vec<&str> = source_text.lines().collect();

    let last_import_line = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("import "))
        .map(|(idx, _)| idx)
        .last();

    let insert_line = if let Some(idx) = last_import_line {
        idx + 1
    } else if let Some(idx) = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("pragma "))
        .map(|(idx, _)| idx)
        .last()
    {
        idx + 1
    } else {
        0
    };

    Some(TextEdit {
        range: Range {
            start: Position {
                line: insert_line as u32,
                character: 0,
            },
            end: Position {
                line: insert_line as u32,
                character: 0,
            },
        },
        new_text: import_stmt,
    })
}

/// Handle a completion request with optional tail candidates.
///
/// Tail candidates are only appended for non-dot completions and are always
/// ordered last via `append_auto_import_candidates_last`.
pub fn handle_completion_with_tail_candidates(
    cache: Option<&CompletionCache>,
    source_text: &str,
    position: Position,
    trigger_char: Option<&str>,
    file_id: Option<FileId>,
    tail_candidates: Vec<CompletionItem>,
) -> Option<CompletionResponse> {
    let lines: Vec<&str> = source_text.lines().collect();
    let line = lines.get(position.line as usize)?;

    // Convert encoding-aware column to a byte offset within this line.
    let abs_byte = crate::utils::position_to_byte_offset(source_text, position);
    let line_start_byte: usize = source_text[..abs_byte]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let col_byte = (abs_byte - line_start_byte) as u32;

    // Build scope context for scope-aware type resolution
    let scope_ctx = file_id.map(|fid| ScopeContext {
        byte_pos: abs_byte,
        file_id: fid,
    });

    let items = if trigger_char == Some(".") {
        let chain = parse_dot_chain(line, col_byte);
        if chain.is_empty() {
            return None;
        }
        match cache {
            Some(c) => get_chain_completions(c, &chain, scope_ctx.as_ref()),
            None => {
                // No cache yet — serve magic dot completions (msg., block., etc.)
                if chain.len() == 1 {
                    let seg = &chain[0];
                    if seg.name == "type" && seg.kind == AccessKind::Call {
                        // type(X). without cache — classify based on name alone
                        type_meta_members(seg.call_args.as_deref(), None)
                    } else if seg.kind == AccessKind::Plain {
                        magic_members(&seg.name).unwrap_or_default()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
        }
    } else {
        match cache {
            Some(c) => {
                append_auto_import_candidates_last(c.general_completions.clone(), tail_candidates)
            }
            None => get_static_completions(),
        }
    };

    Some(CompletionResponse::List(CompletionList {
        is_incomplete: cache.is_none(),
        items,
    }))
}

/// Handle a completion request.
///
/// When `cache` is `Some`, full AST-aware completions are returned.
/// When `cache` is `None`, only static completions (keywords, globals, units)
/// and magic dot completions (msg., block., tx., abi., type().) are returned
/// immediately — no blocking.
///
/// `file_id` is the AST source file id, needed for scope-aware resolution.
/// When `None`, scope resolution is skipped and flat lookup is used.
pub fn handle_completion(
    cache: Option<&CompletionCache>,
    source_text: &str,
    position: Position,
    trigger_char: Option<&str>,
    file_id: Option<FileId>,
) -> Option<CompletionResponse> {
    handle_completion_with_tail_candidates(
        cache,
        source_text,
        position,
        trigger_char,
        file_id,
        vec![],
    )
}

const SOLIDITY_KEYWORDS: &[&str] = &[
    "abstract",
    "address",
    "assembly",
    "bool",
    "break",
    "bytes",
    "bytes1",
    "bytes4",
    "bytes32",
    "calldata",
    "constant",
    "constructor",
    "continue",
    "contract",
    "delete",
    "do",
    "else",
    "emit",
    "enum",
    "error",
    "event",
    "external",
    "fallback",
    "false",
    "for",
    "function",
    "if",
    "immutable",
    "import",
    "indexed",
    "int8",
    "int24",
    "int128",
    "int256",
    "interface",
    "internal",
    "library",
    "mapping",
    "memory",
    "modifier",
    "new",
    "override",
    "payable",
    "pragma",
    "private",
    "public",
    "pure",
    "receive",
    "return",
    "returns",
    "revert",
    "storage",
    "string",
    "struct",
    "true",
    "type",
    "uint8",
    "uint24",
    "uint128",
    "uint160",
    "uint256",
    "unchecked",
    "using",
    "view",
    "virtual",
    "while",
];

/// Ether denomination units — suffixes for literal numbers.
const ETHER_UNITS: &[(&str, &str)] = &[("wei", "1"), ("gwei", "1e9"), ("ether", "1e18")];

/// Time units — suffixes for literal numbers.
const TIME_UNITS: &[(&str, &str)] = &[
    ("seconds", "1"),
    ("minutes", "60 seconds"),
    ("hours", "3600 seconds"),
    ("days", "86400 seconds"),
    ("weeks", "604800 seconds"),
];

const MAGIC_GLOBALS: &[(&str, &str)] = &[
    ("msg", "msg"),
    ("block", "block"),
    ("tx", "tx"),
    ("abi", "abi"),
    ("this", "address"),
    ("super", "contract"),
    ("type", "type information"),
];

const GLOBAL_FUNCTIONS: &[(&str, &str)] = &[
    // Mathematical and Cryptographic Functions
    ("addmod(uint256, uint256, uint256)", "uint256"),
    ("mulmod(uint256, uint256, uint256)", "uint256"),
    ("keccak256(bytes memory)", "bytes32"),
    ("sha256(bytes memory)", "bytes32"),
    ("ripemd160(bytes memory)", "bytes20"),
    (
        "ecrecover(bytes32 hash, uint8 v, bytes32 r, bytes32 s)",
        "address",
    ),
    // Block and Transaction Properties (functions)
    ("blockhash(uint256 blockNumber)", "bytes32"),
    ("blobhash(uint256 index)", "bytes32"),
    ("gasleft()", "uint256"),
    // Error Handling
    ("assert(bool condition)", ""),
    ("require(bool condition)", ""),
    ("require(bool condition, string memory message)", ""),
    ("revert()", ""),
    ("revert(string memory reason)", ""),
    // Contract-related
    ("selfdestruct(address payable recipient)", ""),
];

// ---------------------------------------------------------------------------
// Import path completions
// ---------------------------------------------------------------------------

/// Walk `project_root` recursively and return every `.sol` file as:
///   1. A relative path from the current file's directory (e.g. `./libraries/Pool.sol`)
///   2. A remapped path for each matching remapping (e.g. `forge-std/Test.sol`)
///
/// Skips `out/`, `cache/`, `node_modules/`, `.git/`.
///
/// `typed_range` is `Some((line, start_col, end_col))` where `start_col` is the
/// character column right after the opening quote and `end_col` is the cursor
/// column. When provided, each item gets a `text_edit` that replaces the
/// already-typed text so the insert doesn't double-up, plus `filter_text = label`
/// so clients filter on the full path rather than the word at cursor.
pub fn all_sol_import_paths(
    current_file: &Path,
    project_root: &Path,
    remappings: &[String],
    typed_range: Option<(u32, u32, u32)>,
) -> Vec<CompletionItem> {
    let current_dir = match current_file.parent() {
        Some(d) => d,
        None => return vec![],
    };

    // Pre-parse remappings into (prefix, abs_target_path) pairs.
    // e.g. "forge-std/=lib/forge-std/src/" → ("forge-std/", "/project/lib/forge-std/src/")
    // Canonicalize the target so strip_prefix works even when the path contains
    // `..` segments (e.g. lib/forge-std/../src/ would otherwise produce bad labels).
    let parsed_remappings: Vec<(String, std::path::PathBuf)> = remappings
        .iter()
        .filter_map(|r| {
            let mut it = r.splitn(2, '=');
            let prefix = it.next()?.to_string();
            let target = it.next()?;
            if prefix.is_empty() || target.is_empty() {
                return None;
            }
            let raw = project_root.join(target);
            let canonical = raw.canonicalize().unwrap_or(raw);
            Some((prefix, canonical))
        })
        .collect();

    let skip_dirs: &[&str] = &["out", "cache", "node_modules", ".git"];
    let mut items = Vec::new();

    collect_sol_files(
        project_root,
        current_dir,
        &parsed_remappings,
        skip_dirs,
        typed_range,
        &mut items,
    );
    items.sort_by(|a, b| a.label.cmp(&b.label));
    items
}

fn collect_sol_files(
    dir: &Path,
    current_dir: &Path,
    remappings: &[(String, std::path::PathBuf)],
    skip_dirs: &[&str],
    typed_range: Option<(u32, u32, u32)>,
    out: &mut Vec<CompletionItem>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            if skip_dirs.contains(&name_str.as_ref()) {
                continue;
            }
            collect_sol_files(&path, current_dir, remappings, skip_dirs, typed_range, out);
            continue;
        }

        if !path.is_file() || !name_str.ends_with(".sol") {
            continue;
        }

        // 1. Relative path (always emitted)
        if let Some(rel) = pathdiff::diff_paths(&path, current_dir) {
            let s = rel.to_string_lossy().to_string();
            let label = if s.starts_with("../") || s.starts_with("./") {
                s
            } else {
                format!("./{s}")
            };
            out.push(make_import_item(label, typed_range));
        }

        // 2. Remapped paths — for every remapping whose target is a prefix of
        //    this file's absolute path, emit the remapped form.
        //    e.g. file=/project/lib/forge-std/src/Test.sol
        //         remap forge-std/=/project/lib/forge-std/src/
        //         → label = "forge-std/Test.sol"
        for (prefix, target_abs) in remappings {
            if let Ok(suffix) = path.strip_prefix(target_abs) {
                let suffix_str = suffix
                    .to_string_lossy()
                    .trim_start_matches(['/', '\\'])
                    .to_string();
                let label = format!("{}{}", prefix, suffix_str);
                out.push(make_import_item(label, typed_range));
            }
        }
    }
}

fn make_import_item(label: String, typed_range: Option<(u32, u32, u32)>) -> CompletionItem {
    // filter_text = label so that clients whose word-at-cursor is shorter than
    // the full path (e.g. just "Pool" when the label is "./src/Pool.sol") still
    // match correctly via substring/fuzzy matching against the full path.
    //
    // text_edit replaces the already-typed text inside the quotes so the insert
    // doesn't append after what was already typed.
    let text_edit = typed_range.map(|(line, start_col, end_col)| {
        tower_lsp::lsp_types::CompletionTextEdit::Edit(TextEdit {
            range: Range {
                start: Position {
                    line,
                    character: start_col,
                },
                end: Position {
                    line,
                    character: end_col,
                },
            },
            new_text: label.clone(),
        })
    });
    CompletionItem {
        label: label.clone(),
        filter_text: Some(label.clone()),
        text_edit,
        kind: Some(CompletionItemKind::FILE),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CompletionCache, TopLevelImportable, append_auto_import_candidates_last,
        build_completion_cache, extract_top_level_importables_for_file,
    };
    use serde_json::json;
    use std::collections::HashMap;
    use tower_lsp::lsp_types::CompletionItemKind;
    use tower_lsp::lsp_types::{CompletionItem, CompletionResponse, Position, Range, TextEdit};

    fn empty_cache() -> CompletionCache {
        CompletionCache {
            names: vec![],
            name_to_type: HashMap::new(),
            node_members: HashMap::new(),
            type_to_node: HashMap::new(),
            name_to_node_id: HashMap::new(),
            method_identifiers: HashMap::new(),
            function_return_types: HashMap::new(),
            using_for: HashMap::new(),
            using_for_wildcard: vec![],
            general_completions: vec![],
            scope_declarations: HashMap::new(),
            scope_parent: HashMap::new(),
            scope_ranges: vec![],
            path_to_file_id: HashMap::new(),
            linearized_base_contracts: HashMap::new(),
            contract_kinds: HashMap::new(),
            top_level_importables_by_name: HashMap::new(),
            top_level_importables_by_file: HashMap::new(),
        }
    }

    #[test]
    fn top_level_importables_include_only_direct_declared_symbols() {
        let sources = json!({
            "/tmp/A.sol": {
                "id": 0,
                "ast": {
                    "id": 1,
                    "nodeType": "SourceUnit",
                    "src": "0:100:0",
                    "nodes": [
                        { "id": 10, "nodeType": "ImportDirective", "name": "Alias", "scope": 1, "src": "1:1:0" },
                        { "id": 11, "nodeType": "ContractDefinition", "name": "C", "scope": 1, "src": "2:1:0", "nodes": [
                            { "id": 21, "nodeType": "VariableDeclaration", "name": "inside", "scope": 11, "constant": true, "src": "3:1:0" }
                        ] },
                        { "id": 12, "nodeType": "StructDefinition", "name": "S", "scope": 1, "src": "4:1:0" },
                        { "id": 13, "nodeType": "EnumDefinition", "name": "E", "scope": 1, "src": "5:1:0" },
                        { "id": 14, "nodeType": "UserDefinedValueTypeDefinition", "name": "Wad", "scope": 1, "src": "6:1:0" },
                        { "id": 15, "nodeType": "FunctionDefinition", "name": "freeFn", "scope": 1, "src": "7:1:0" },
                        { "id": 16, "nodeType": "VariableDeclaration", "name": "TOP_CONST", "scope": 1, "constant": true, "src": "8:1:0" },
                        { "id": 17, "nodeType": "VariableDeclaration", "name": "TOP_VAR", "scope": 1, "constant": false, "src": "9:1:0" }
                    ]
                }
            }
        });

        let cache = build_completion_cache(&sources, None);
        let map = &cache.top_level_importables_by_name;
        let by_file = &cache.top_level_importables_by_file;

        assert!(map.contains_key("C"));
        assert!(map.contains_key("S"));
        assert!(map.contains_key("E"));
        assert!(map.contains_key("Wad"));
        assert!(map.contains_key("freeFn"));
        assert!(map.contains_key("TOP_CONST"));

        assert!(!map.contains_key("Alias"));
        assert!(!map.contains_key("inside"));
        assert!(!map.contains_key("TOP_VAR"));

        let file_symbols = by_file.get("/tmp/A.sol").unwrap();
        let file_names: Vec<&str> = file_symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(file_names.contains(&"C"));
        assert!(file_names.contains(&"TOP_CONST"));
        assert!(!file_names.contains(&"Alias"));
    }

    #[test]
    fn top_level_importables_keep_multiple_declarations_for_same_name() {
        let sources = json!({
            "/tmp/A.sol": {
                "id": 0,
                "ast": {
                    "id": 1,
                    "nodeType": "SourceUnit",
                    "src": "0:100:0",
                    "nodes": [
                        { "id": 11, "nodeType": "FunctionDefinition", "name": "dup", "scope": 1, "src": "1:1:0" }
                    ]
                }
            },
            "/tmp/B.sol": {
                "id": 1,
                "ast": {
                    "id": 2,
                    "nodeType": "SourceUnit",
                    "src": "0:100:1",
                    "nodes": [
                        { "id": 22, "nodeType": "FunctionDefinition", "name": "dup", "scope": 2, "src": "2:1:1" }
                    ]
                }
            }
        });

        let cache = build_completion_cache(&sources, None);
        let entries = cache.top_level_importables_by_name.get("dup").unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn extract_top_level_importables_for_file_finds_expected_symbols() {
        let ast = json!({
            "id": 1,
            "nodeType": "SourceUnit",
            "src": "0:100:0",
            "nodes": [
                { "id": 2, "nodeType": "FunctionDefinition", "name": "f", "scope": 1, "src": "1:1:0" },
                { "id": 3, "nodeType": "VariableDeclaration", "name": "K", "scope": 1, "constant": true, "src": "2:1:0" },
                { "id": 4, "nodeType": "VariableDeclaration", "name": "V", "scope": 1, "constant": false, "src": "3:1:0" }
            ]
        });

        let symbols = extract_top_level_importables_for_file("/tmp/A.sol", &ast);
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"f"));
        assert!(names.contains(&"K"));
        assert!(!names.contains(&"V"));
    }

    #[test]
    fn top_level_importables_can_be_replaced_per_file() {
        let sources = json!({
            "/tmp/A.sol": {
                "id": 0,
                "ast": {
                    "id": 1,
                    "nodeType": "SourceUnit",
                    "src": "0:100:0",
                    "nodes": [
                        { "id": 11, "nodeType": "FunctionDefinition", "name": "dup", "scope": 1, "src": "1:1:0" }
                    ]
                }
            },
            "/tmp/B.sol": {
                "id": 1,
                "ast": {
                    "id": 2,
                    "nodeType": "SourceUnit",
                    "src": "0:100:1",
                    "nodes": [
                        { "id": 22, "nodeType": "FunctionDefinition", "name": "dup", "scope": 2, "src": "2:1:1" }
                    ]
                }
            }
        });

        let mut cache = build_completion_cache(&sources, None);
        assert_eq!(cache.top_level_importables_by_name["dup"].len(), 2);

        cache.replace_top_level_importables_for_path(
            "/tmp/A.sol".to_string(),
            vec![TopLevelImportable {
                name: "newA".to_string(),
                declaring_path: "/tmp/A.sol".to_string(),
                node_type: "FunctionDefinition".to_string(),
                kind: CompletionItemKind::FUNCTION,
            }],
        );
        assert_eq!(cache.top_level_importables_by_name["dup"].len(), 1);
        assert!(cache.top_level_importables_by_name.contains_key("newA"));

        cache.replace_top_level_importables_for_path("/tmp/A.sol".to_string(), vec![]);
        assert!(!cache.top_level_importables_by_name.contains_key("newA"));
    }

    #[test]
    fn append_auto_import_candidates_last_sets_tail_sort_text() {
        let base = vec![CompletionItem {
            label: "localVar".to_string(),
            ..Default::default()
        }];
        let auto = vec![CompletionItem {
            label: "ImportMe".to_string(),
            ..Default::default()
        }];

        let out = append_auto_import_candidates_last(base, auto);
        assert_eq!(out.len(), 2);
        assert_eq!(out[1].label, "ImportMe");
        assert!(
            out[1]
                .sort_text
                .as_deref()
                .is_some_and(|s| s.starts_with("zz_autoimport_"))
        );
    }

    #[test]
    fn append_auto_import_candidates_last_keeps_same_label_candidates() {
        let base = vec![CompletionItem {
            label: "B".to_string(),
            ..Default::default()
        }];
        let auto = vec![
            CompletionItem {
                label: "B".to_string(),
                detail: Some("ContractDefinition (./B.sol)".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "B".to_string(),
                detail: Some("ContractDefinition (./deps/B.sol)".to_string()),
                ..Default::default()
            },
        ];

        let out = append_auto_import_candidates_last(base, auto);
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn append_auto_import_candidates_last_enriches_unique_base_label_with_edit() {
        let base = vec![CompletionItem {
            label: "B".to_string(),
            ..Default::default()
        }];
        let auto = vec![CompletionItem {
            label: "B".to_string(),
            additional_text_edits: Some(vec![TextEdit {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
                new_text: "import {B} from \"./B.sol\";\n".to_string(),
            }]),
            ..Default::default()
        }];
        let out = append_auto_import_candidates_last(base, auto);
        assert!(
            out[0].additional_text_edits.is_some(),
            "base item should inherit unique import edit"
        );
    }

    #[test]
    fn top_level_importable_candidates_include_import_edit() {
        let mut cache = empty_cache();
        cache.top_level_importables_by_name.insert(
            "B".to_string(),
            vec![TopLevelImportable {
                name: "B".to_string(),
                declaring_path: "/tmp/example/B.sol".to_string(),
                node_type: "ContractDefinition".to_string(),
                kind: CompletionItemKind::CLASS,
            }],
        );

        let source = "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.26;\n\ncontract A {}\n";
        let items = super::top_level_importable_completion_candidates(
            &cache,
            Some("/tmp/example/A.sol"),
            source,
        );
        assert_eq!(items.len(), 1);
        let edit_text = items[0]
            .additional_text_edits
            .as_ref()
            .and_then(|edits| edits.first())
            .map(|e| e.new_text.clone())
            .unwrap_or_default();
        assert!(edit_text.contains("import {B} from \"./B.sol\";"));
    }

    #[test]
    fn handle_completion_general_path_keeps_base_items() {
        let mut cache = empty_cache();
        cache.general_completions = vec![CompletionItem {
            label: "A".to_string(),
            ..Default::default()
        }];

        let resp = super::handle_completion(
            Some(&cache),
            "contract X {}",
            Position {
                line: 0,
                character: 0,
            },
            None,
            None,
        );
        match resp {
            Some(CompletionResponse::List(list)) => {
                assert_eq!(list.items.len(), 1);
                assert_eq!(list.items[0].label, "A");
            }
            _ => panic!("expected completion list"),
        }
    }

    // --- import path completion tests ---

    #[test]
    fn make_import_item_has_filter_text_and_text_edit() {
        let item = super::make_import_item("./src/Pool.sol".to_string(), Some((0, 8, 12)));
        assert_eq!(item.filter_text.as_deref(), Some("./src/Pool.sol"));
        assert!(item.text_edit.is_some(), "text_edit should be set");
        assert!(
            item.insert_text.is_none(),
            "insert_text should be absent when text_edit is set"
        );
    }

    #[test]
    fn all_sol_import_paths_no_panic_missing_dir() {
        let current = std::path::Path::new("/nonexistent/src/Foo.sol");
        let root = std::path::Path::new("/nonexistent");
        let items = super::all_sol_import_paths(current, root, &[], None);
        assert!(items.is_empty());
    }

    #[test]
    fn all_sol_import_paths_relative_labels() {
        // Use real paths from the repo fixture
        let root = std::path::Path::new("example");
        let current = root.join("A.sol");
        let items = super::all_sol_import_paths(&current, root, &[], None);
        // All labels should start with ./ or ../
        for item in &items {
            assert!(
                item.label.starts_with("./") || item.label.starts_with("../"),
                "label should be relative: {}",
                item.label
            );
            assert!(
                item.label.ends_with(".sol"),
                "label should end with .sol: {}",
                item.label
            );
        }
        assert!(!items.is_empty(), "should find at least one .sol file");
    }
}
