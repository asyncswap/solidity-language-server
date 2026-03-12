use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tower_lsp::lsp_types::{Location, Position, Range, TextEdit, Url};
use tree_sitter::{Node, Parser};

use crate::types::{
    AbsPath, FileId, NodeId, PathInterner, RelPath, SolcFileId, SourceLoc, SrcLocation,
};
use crate::utils::push_if_node_or_array;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub src: SrcLocation,
    pub name_location: Option<String>,
    pub name_locations: Vec<String>,
    pub referenced_declaration: Option<NodeId>,
    pub node_type: Option<String>,
    pub member_location: Option<String>,
    pub absolute_path: Option<String>,
    /// The AST `scope` field — the node ID of the containing declaration
    /// (contract, library, interface, function, etc.). Used to resolve the
    /// qualifier in qualified type paths like `Pool.State` where `scope`
    /// on the `State` struct points to the `Pool` library.
    #[serde(default)]
    pub scope: Option<NodeId>,
    /// The AST `baseFunctions` field — node IDs of interface/parent
    /// function declarations that this function implements or overrides.
    /// Present on `FunctionDefinition`, `ModifierDefinition`, and
    /// `VariableDeclaration` nodes. Used to build the bidirectional
    /// `base_function_implementation` index for interface ↔ implementation
    /// equivalence in references and call hierarchy.
    #[serde(default)]
    pub base_functions: Vec<NodeId>,
}

/// All AST child keys to traverse (Solidity + Yul).
pub const CHILD_KEYS: &[&str] = &[
    "AST",
    "arguments",
    "baseContracts",
    "baseExpression",
    "baseName",
    "baseType",
    "block",
    "body",
    "components",
    "condition",
    "declarations",
    "endExpression",
    "errorCall",
    "eventCall",
    "expression",
    "externalCall",
    "falseBody",
    "falseExpression",
    "file",
    "foreign",
    "functionName",
    "indexExpression",
    "initialValue",
    "initializationExpression",
    "keyType",
    "leftExpression",
    "leftHandSide",
    "libraryName",
    "literals",
    "loopExpression",
    "members",
    "modifierName",
    "modifiers",
    "name",
    "names",
    "nodes",
    "options",
    "overrides",
    "parameters",
    "pathNode",
    "post",
    "pre",
    "returnParameters",
    "rightExpression",
    "rightHandSide",
    "startExpression",
    "statements",
    "storageLayout",
    "subExpression",
    "subdenomination",
    "symbolAliases",
    "trueBody",
    "trueExpression",
    "typeName",
    "unitAlias",
    "value",
    "valueType",
    "variableNames",
    "variables",
];

/// Maps `"offset:length:fileId"` src strings from Yul externalReferences
/// to the Solidity declaration node id they refer to.
pub type ExternalRefs = HashMap<SrcLocation, NodeId>;

/// Pre-computed AST index. Built once when an AST enters the cache,
/// then reused on every goto/references/rename/hover request.
///
/// All data from the raw solc JSON is consumed during `new()` into
/// pre-built indexes. The raw JSON is not retained.
#[derive(Debug, Clone)]
pub struct CachedBuild {
    pub nodes: HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
    pub path_to_abs: HashMap<RelPath, AbsPath>,
    pub external_refs: ExternalRefs,
    pub id_to_path_map: HashMap<crate::types::SolcFileId, String>,
    /// O(1) typed declaration node lookup by AST node ID.
    /// Built from the typed AST via visitor. Contains functions, variables,
    /// contracts, events, errors, structs, enums, modifiers, and UDVTs.
    pub decl_index: HashMap<NodeId, crate::solc_ast::DeclNode>,
    /// O(1) lookup from any declaration/source-unit node ID to its source file path.
    /// Built from `typed_ast` during construction. Replaces the O(N)
    /// `find_source_path_for_node()` that walked raw JSON.
    pub node_id_to_source_path: HashMap<NodeId, AbsPath>,
    /// Pre-built hint lookup per file. Built once, reused on every
    /// inlay hint request (avoids O(n²) declaration resolution per request).
    pub hint_index: crate::inlay_hints::HintIndex,
    /// Pre-built documentation index from solc userdoc/devdoc.
    /// Merged and keyed by selector for fast hover lookup.
    pub doc_index: crate::hover::DocIndex,
    /// Pre-built completion cache. Built from sources during construction
    /// before the sources key is stripped.
    pub completion_cache: std::sync::Arc<crate::completion::CompletionCache>,
    /// The text_cache version this build was created from.
    /// Used to detect dirty files (unsaved edits since last build).
    pub build_version: i32,
    /// Qualifier reference index: maps a container declaration ID
    /// (contract/library/interface) to `IdentifierPath` node IDs that use
    /// it as a qualifier prefix in qualified type paths (e.g., `Pool.State`).
    ///
    /// Built at cache time by following `referencedDeclaration` on multi-segment
    /// `IdentifierPath` nodes to their declaration, then reading the declaration's
    /// `scope` field to find the container.
    pub qualifier_refs: HashMap<NodeId, Vec<NodeId>>,
    /// Bidirectional implementation index built from `baseFunctions`/`baseModifiers`.
    ///
    /// Maps each declaration ID to the set of IDs that are semantically
    /// equivalent (interface ↔ implementation). For example, if
    /// `PoolManager.swap` (616) has `baseFunctions: [2036]` (IPoolManager.swap),
    /// this will contain `616 → [2036]` and `2036 → [616]`.
    ///
    /// Used by `textDocument/implementation`, `textDocument/references`, and
    /// `callHierarchy/incomingCalls` to unify interface and implementation IDs.
    /// Empty in warm-loaded builds (`from_reference_index`).
    pub base_function_implementation: HashMap<NodeId, Vec<NodeId>>,
}

impl CachedBuild {
    /// Build the index from normalized AST output.
    ///
    /// Canonical shape:
    /// - `sources[path] = { id, ast }`
    /// - `contracts[path][name] = { evm, devdoc, ... }`
    /// - `source_id_to_path = { "0": "path", ... }`
    ///
    /// When `interner` is provided, solc's per-compilation file IDs in all
    /// `src` strings are translated into canonical IDs from the project-wide
    /// [`PathInterner`].  This ensures all `CachedBuild` instances share the
    /// same file-ID space regardless of which solc invocation produced them.
    pub fn new(ast: Value, build_version: i32, interner: Option<&mut PathInterner>) -> Self {
        let (mut nodes, path_to_abs, mut external_refs) = if let Some(sources) = ast.get("sources")
        {
            cache_ids(sources)
        } else {
            (HashMap::new(), HashMap::new(), HashMap::new())
        };

        // Parse solc's source_id_to_path from the AST output.
        let solc_id_to_path: HashMap<SolcFileId, String> = ast
            .get("source_id_to_path")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| {
                        (
                            SolcFileId::new(k.clone()),
                            v.as_str().unwrap_or("").to_string(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        // When an interner is available, canonicalize file IDs in all src
        // strings so that merges across different compilations are safe.
        // `canonical_remap` is kept around so that `build_completion_cache`
        // can translate its own file IDs in the same way.
        let (id_to_path_map, canonical_remap) = if let Some(interner) = interner {
            let remap = interner.build_remap(&solc_id_to_path);

            // Rewrite all NodeInfo src strings.
            for file_nodes in nodes.values_mut() {
                for info in file_nodes.values_mut() {
                    canonicalize_node_info(info, &remap);
                }
            }

            // Rewrite external ref src strings.
            let old_refs = std::mem::take(&mut external_refs);
            for (src, decl_id) in old_refs {
                let new_src = SrcLocation::new(remap_src_canonical(src.as_str(), &remap));
                external_refs.insert(new_src, decl_id);
            }

            // Build id_to_path_map from canonical IDs.
            (interner.to_id_to_path_map(), Some(remap))
        } else {
            // No interner — use solc's original mapping (legacy path).
            (solc_id_to_path, None)
        };

        let doc_index = crate::hover::build_doc_index(&ast);

        // Extract declaration nodes directly from the raw sources JSON.
        // Instead of deserializing the entire typed AST (SourceUnit, all
        // expressions, statements, Yul blocks), this walks the raw Value
        // tree and only deserializes nodes whose nodeType matches one of the
        // 9 declaration types. Heavy fields (body, modifiers, value, etc.)
        // are stripped before deserialization.
        let (decl_index, node_id_to_source_path) = if let Some(sources) = ast.get("sources") {
            match crate::solc_ast::extract_decl_nodes(sources) {
                Some(extracted) => (
                    extracted
                        .decl_index
                        .into_iter()
                        .map(|(id, decl)| (NodeId(id), decl))
                        .collect(),
                    extracted
                        .node_id_to_source_path
                        .into_iter()
                        .map(|(id, path)| (NodeId(id), AbsPath::new(path)))
                        .collect(),
                ),
                None => (HashMap::new(), HashMap::new()),
            }
        } else {
            (HashMap::new(), HashMap::new())
        };

        // Build constructor index and hint index from the typed decl_index.
        let constructor_index = crate::inlay_hints::build_constructor_index(&decl_index);
        let hint_index = if let Some(sources) = ast.get("sources") {
            crate::inlay_hints::build_hint_index(sources, &decl_index, &constructor_index)
        } else {
            HashMap::new()
        };

        // Build completion cache before stripping sources.
        let completion_cache = {
            let sources = ast.get("sources");
            let contracts = ast.get("contracts");
            let cc = if let Some(s) = sources {
                crate::completion::build_completion_cache(s, contracts, canonical_remap.as_ref())
            } else {
                crate::completion::build_completion_cache(
                    &serde_json::Value::Object(Default::default()),
                    contracts,
                    canonical_remap.as_ref(),
                )
            };
            std::sync::Arc::new(cc)
        };

        // Build the qualifier reference index: for each multi-segment
        // IdentifierPath (e.g., `Pool.State`), follow referencedDeclaration
        // to the declaration node, then read its `scope` to find the
        // container (contract/library/interface). Map container_id → [node_id].
        let qualifier_refs = build_qualifier_refs(&nodes);

        // Build the bidirectional implementation index from base_functions on NodeInfo.
        // Works uniformly on both fresh and warm-loaded builds since NodeInfo
        // now persists the base_functions field.
        let base_function_implementation = build_base_function_implementation(&nodes);

        // The raw AST JSON is fully consumed — all data has been extracted
        // into the pre-built indexes above. `ast` is dropped here.

        Self {
            nodes,
            path_to_abs,
            external_refs,
            id_to_path_map,
            decl_index,
            node_id_to_source_path,
            hint_index,
            doc_index,
            completion_cache,
            build_version,
            qualifier_refs,
            base_function_implementation,
        }
    }

    /// Absorb data from a previous build for files this build doesn't cover.
    ///
    /// For each file in `other.nodes` that is **not** already present in
    /// `self.nodes`, copies the node map, path mapping, and any related
    /// entries.  This ensures a freshly compiled project index never loses
    /// coverage compared to the warm-loaded cache it replaces.
    pub fn merge_missing_from(&mut self, other: &CachedBuild) {
        for (abs_path, file_nodes) in &other.nodes {
            if !self.nodes.contains_key(abs_path) {
                self.nodes.insert(abs_path.clone(), file_nodes.clone());
            }
        }
        for (k, v) in &other.path_to_abs {
            self.path_to_abs
                .entry(k.clone())
                .or_insert_with(|| v.clone());
        }
        for (k, v) in &other.external_refs {
            self.external_refs.entry(k.clone()).or_insert(*v);
        }
        for (k, v) in &other.id_to_path_map {
            self.id_to_path_map
                .entry(k.clone())
                .or_insert_with(|| v.clone());
        }
        // Merge qualifier_refs: for each container, add any qualifier node
        // IDs from `other` that aren't already present in `self`.
        for (container_id, other_qrefs) in &other.qualifier_refs {
            let entry = self.qualifier_refs.entry(*container_id).or_default();
            for &qnode_id in other_qrefs {
                if !entry.contains(&qnode_id) {
                    entry.push(qnode_id);
                }
            }
        }
        // Merge base_function_implementation: add any equivalences from `other`
        // that aren't already present in `self`.
        for (node_id, other_impls) in &other.base_function_implementation {
            let entry = self
                .base_function_implementation
                .entry(*node_id)
                .or_default();
            for &impl_id in other_impls {
                if !entry.contains(&impl_id) {
                    entry.push(impl_id);
                }
            }
        }
    }

    /// Construct a minimal cached build from persisted reference/goto indexes.
    ///
    /// This is used for fast startup warm-cache restores where we only need
    /// cross-file node/reference maps (not full gas/doc/hint indexes).
    ///
    /// When `interner` is provided, the `id_to_path_map` entries are
    /// registered in the interner so that subsequent compilations will
    /// assign the same canonical IDs to the same paths.
    pub fn from_reference_index(
        nodes: HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
        path_to_abs: HashMap<RelPath, AbsPath>,
        external_refs: ExternalRefs,
        id_to_path_map: HashMap<SolcFileId, String>,
        build_version: i32,
        interner: Option<&mut PathInterner>,
    ) -> Self {
        // Seed the interner with paths from the persisted cache so that
        // canonical IDs remain consistent across restart.
        if let Some(interner) = interner {
            for path in id_to_path_map.values() {
                interner.intern(path);
            }
        }

        let completion_cache = std::sync::Arc::new(crate::completion::build_completion_cache(
            &serde_json::Value::Object(Default::default()),
            None,
            None,
        ));

        // Build qualifier refs from the warm-loaded nodes.
        let qualifier_refs = build_qualifier_refs(&nodes);

        // Build base_function_implementation from the warm-loaded nodes.
        // NodeInfo now persists `base_functions`, so this works on warm loads.
        let base_function_implementation = build_base_function_implementation(&nodes);

        Self {
            nodes,
            path_to_abs,
            external_refs,
            id_to_path_map,
            decl_index: HashMap::new(),
            node_id_to_source_path: HashMap::new(),
            hint_index: HashMap::new(),
            doc_index: HashMap::new(),
            completion_cache,
            build_version,
            qualifier_refs,
            base_function_implementation,
        }
    }
}

/// Build the qualifier reference index from the cached node maps.
///
/// For each `IdentifierPath` node with `nameLocations.len() > 1` (i.e., a
/// qualified path like `Pool.State`), follows `referencedDeclaration` to the
/// declaration node, reads its `scope` field (the containing contract /
/// library / interface), and records `scope_id → [identifierpath_node_id]`.
///
/// This allows "Find All References" on a container to include qualified
/// type references where the container appears as a prefix.
fn build_qualifier_refs(
    nodes: &HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
) -> HashMap<NodeId, Vec<NodeId>> {
    // First pass: collect all nodes' scope fields into a lookup table.
    // We need this to look up the scope of a referencedDeclaration target.
    let mut node_scope: HashMap<NodeId, NodeId> = HashMap::new();
    for file_nodes in nodes.values() {
        for (id, info) in file_nodes {
            if let Some(scope_id) = info.scope {
                node_scope.insert(*id, scope_id);
            }
        }
    }

    // Second pass: for each multi-segment IdentifierPath, resolve the
    // container via referencedDeclaration → scope.
    let mut qualifier_refs: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for file_nodes in nodes.values() {
        for (id, info) in file_nodes {
            if info.name_locations.len() > 1 && info.node_type.as_deref() == Some("IdentifierPath")
            {
                if let Some(ref_decl) = info.referenced_declaration
                    && let Some(&scope_id) = node_scope.get(&ref_decl)
                {
                    qualifier_refs.entry(scope_id).or_default().push(*id);
                }
            }
        }
    }

    qualifier_refs
}

/// Build the bidirectional `base_function_implementation` index from nodes.
///
/// Scans all nodes for entries with non-empty `base_functions` (these are
/// implementing/overriding function declarations). Creates bidirectional
/// mappings: `impl_id → [base_ids]` (forward) and `base_id → [impl_id]`
/// (reverse), so lookups work in both directions.
///
/// This function works on both fresh and warm-loaded builds because
/// `base_functions` is persisted as part of `NodeInfo`.
fn build_base_function_implementation(
    nodes: &HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
) -> HashMap<NodeId, Vec<NodeId>> {
    let mut index: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    for file_nodes in nodes.values() {
        for (id, info) in file_nodes {
            if info.base_functions.is_empty() {
                continue;
            }
            // Forward: impl_id → base_ids
            for &base_id in &info.base_functions {
                index.entry(*id).or_default().push(base_id);
            }
            // Reverse: base_id → impl_id
            for &base_id in &info.base_functions {
                index.entry(base_id).or_default().push(*id);
            }
        }
    }

    // Deduplicate values.
    for ids in index.values_mut() {
        ids.sort_unstable();
        ids.dedup();
    }

    index
}

/// Return type of [`cache_ids`]: `(nodes, path_to_abs, external_refs)`.
type CachedIds = (
    HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
    HashMap<RelPath, AbsPath>,
    ExternalRefs,
);

/// Rewrite the file-ID component of a `"offset:length:fileId"` string using
/// a canonical remap table.  Returns the original string unchanged if the
/// file ID is not in the remap or if the format is invalid.
pub fn remap_src_canonical(src: &str, remap: &HashMap<u64, FileId>) -> String {
    let Some(last_colon) = src.rfind(':') else {
        return src.to_owned();
    };
    let old_id_str = &src[last_colon + 1..];
    let Ok(old_id) = old_id_str.parse::<u64>() else {
        return src.to_owned();
    };
    let Some(canonical) = remap.get(&old_id) else {
        return src.to_owned();
    };
    if canonical.0 == old_id {
        return src.to_owned();
    }
    format!("{}{}", &src[..=last_colon], canonical.0)
}

/// Rewrite all file-ID references in a [`NodeInfo`] using the canonical
/// remap table.
fn canonicalize_node_info(info: &mut NodeInfo, remap: &HashMap<u64, FileId>) {
    info.src = SrcLocation::new(remap_src_canonical(info.src.as_str(), remap));
    if let Some(loc) = info.name_location.as_mut() {
        *loc = remap_src_canonical(loc, remap);
    }
    for loc in &mut info.name_locations {
        *loc = remap_src_canonical(loc, remap);
    }
    if let Some(loc) = info.member_location.as_mut() {
        *loc = remap_src_canonical(loc, remap);
    }
}

pub fn cache_ids(sources: &Value) -> CachedIds {
    let source_count = sources.as_object().map_or(0, |obj| obj.len());

    // Pre-size top-level maps based on source file count to avoid rehashing.
    // Typical project: ~200 nodes/file, ~10 external refs/file.
    let mut nodes: HashMap<AbsPath, HashMap<NodeId, NodeInfo>> =
        HashMap::with_capacity(source_count);
    let mut path_to_abs: HashMap<RelPath, AbsPath> = HashMap::with_capacity(source_count);
    let mut external_refs: ExternalRefs = HashMap::with_capacity(source_count * 10);

    if let Some(sources_obj) = sources.as_object() {
        for (path, source_data) in sources_obj {
            if let Some(ast) = source_data.get("ast") {
                // Get the absolute path for this file
                let abs_path = AbsPath::new(
                    ast.get("absolutePath")
                        .and_then(|v| v.as_str())
                        .unwrap_or(path)
                        .to_string(),
                );

                path_to_abs.insert(RelPath::new(path), abs_path.clone());

                // Initialize the per-file node map with a size hint.
                // Use the top-level `nodes` array length as a proxy for total
                // AST node count (actual count is higher due to nesting, but
                // this avoids the first few rehashes).
                let size_hint = ast
                    .get("nodes")
                    .and_then(|v| v.as_array())
                    .map_or(64, |arr| arr.len() * 8);
                if !nodes.contains_key(&abs_path) {
                    nodes.insert(abs_path.clone(), HashMap::with_capacity(size_hint));
                }

                if let Some(id) = ast.get("id").and_then(|v| v.as_i64())
                    && let Some(src) = ast.get("src").and_then(|v| v.as_str())
                {
                    nodes.get_mut(&abs_path).unwrap().insert(
                        NodeId(id),
                        NodeInfo {
                            src: SrcLocation::new(src),
                            name_location: None,
                            name_locations: vec![],
                            referenced_declaration: None,
                            node_type: ast
                                .get("nodeType")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            member_location: None,
                            absolute_path: ast
                                .get("absolutePath")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            scope: ast.get("scope").and_then(|v| v.as_i64()).map(NodeId),
                            base_functions: vec![],
                        },
                    );
                }

                let mut stack = vec![ast];

                while let Some(tree) = stack.pop() {
                    if let Some(raw_id) = tree.get("id").and_then(|v| v.as_i64())
                        && let Some(src) = tree.get("src").and_then(|v| v.as_str())
                    {
                        let id = NodeId(raw_id);
                        // Check for nameLocation first
                        let mut name_location = tree
                            .get("nameLocation")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        // Check for nameLocations array and use appropriate element
                        // For IdentifierPath (qualified names like D.State), use the last element (the actual identifier)
                        // For other nodes, use the first element
                        if name_location.is_none()
                            && let Some(name_locations) = tree.get("nameLocations")
                            && let Some(locations_array) = name_locations.as_array()
                            && !locations_array.is_empty()
                        {
                            let node_type = tree.get("nodeType").and_then(|v| v.as_str());
                            if node_type == Some("IdentifierPath") {
                                name_location = locations_array
                                    .last()
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                            } else {
                                name_location = locations_array[0].as_str().map(|s| s.to_string());
                            }
                        }

                        let name_locations = if let Some(name_locations) = tree.get("nameLocations")
                            && let Some(locations_array) = name_locations.as_array()
                        {
                            locations_array
                                .iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        } else {
                            vec![]
                        };

                        let mut final_name_location = name_location;
                        if final_name_location.is_none()
                            && let Some(member_loc) =
                                tree.get("memberLocation").and_then(|v| v.as_str())
                        {
                            final_name_location = Some(member_loc.to_string());
                        }

                        let node_info = NodeInfo {
                            src: SrcLocation::new(src),
                            name_location: final_name_location,
                            name_locations,
                            referenced_declaration: tree
                                .get("referencedDeclaration")
                                .and_then(|v| v.as_i64())
                                .map(NodeId),
                            node_type: tree
                                .get("nodeType")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            member_location: tree
                                .get("memberLocation")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            absolute_path: tree
                                .get("absolutePath")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            scope: tree.get("scope").and_then(|v| v.as_i64()).map(NodeId),
                            base_functions: tree
                                .get("baseFunctions")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter().filter_map(|v| v.as_i64().map(NodeId)).collect()
                                })
                                .unwrap_or_default(),
                        };

                        nodes.get_mut(&abs_path).unwrap().insert(id, node_info);

                        // Collect externalReferences from InlineAssembly nodes
                        if tree.get("nodeType").and_then(|v| v.as_str()) == Some("InlineAssembly")
                            && let Some(ext_refs) =
                                tree.get("externalReferences").and_then(|v| v.as_array())
                        {
                            for ext_ref in ext_refs {
                                if let Some(src_str) = ext_ref.get("src").and_then(|v| v.as_str())
                                    && let Some(decl_id) =
                                        ext_ref.get("declaration").and_then(|v| v.as_i64())
                                {
                                    external_refs
                                        .insert(SrcLocation::new(src_str), NodeId(decl_id));
                                }
                            }
                        }
                    }

                    for key in CHILD_KEYS {
                        push_if_node_or_array(tree, key, &mut stack);
                    }
                }
            }
        }
    }

    (nodes, path_to_abs, external_refs)
}

pub fn pos_to_bytes(source_bytes: &[u8], position: Position) -> usize {
    let text = String::from_utf8_lossy(source_bytes);
    crate::utils::position_to_byte_offset(&text, position)
}

pub fn bytes_to_pos(source_bytes: &[u8], byte_offset: usize) -> Option<Position> {
    let text = String::from_utf8_lossy(source_bytes);
    let pos = crate::utils::byte_offset_to_position(&text, byte_offset);
    Some(pos)
}

/// Convert a `"offset:length:fileId"` src string to an LSP Location.
pub fn src_to_location(
    src: &str,
    id_to_path: &HashMap<crate::types::SolcFileId, String>,
) -> Option<Location> {
    let loc = SourceLoc::parse(src)?;
    let file_path = id_to_path.get(&loc.file_id_str())?;

    let absolute_path = if std::path::Path::new(file_path).is_absolute() {
        std::path::PathBuf::from(file_path)
    } else {
        std::env::current_dir().ok()?.join(file_path)
    };

    let source_bytes = std::fs::read(&absolute_path).ok()?;
    let start_pos = bytes_to_pos(&source_bytes, loc.offset)?;
    let end_pos = bytes_to_pos(&source_bytes, loc.end())?;
    let uri = Url::from_file_path(&absolute_path).ok()?;

    Some(Location {
        uri,
        range: Range {
            start: start_pos,
            end: end_pos,
        },
    })
}

pub fn goto_bytes(
    nodes: &HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
    path_to_abs: &HashMap<RelPath, AbsPath>,
    id_to_path: &HashMap<crate::types::SolcFileId, String>,
    external_refs: &ExternalRefs,
    uri: &str,
    position: usize,
) -> Option<(String, usize, usize)> {
    let path = match uri.starts_with("file://") {
        true => &uri[7..],
        false => uri,
    };

    // Get absolute path for this file
    let abs_path = path_to_abs.get(path)?;

    // Get nodes for the current file only
    let current_file_nodes = nodes.get(abs_path)?;

    // Build reverse map: file_path -> file_id for filtering external refs by current file
    let path_to_file_id: HashMap<&str, &crate::types::SolcFileId> =
        id_to_path.iter().map(|(id, p)| (p.as_str(), id)).collect();

    // Determine the file id for the current file
    // path_to_abs maps filesystem path -> absolutePath (e.g. "src/libraries/SwapMath.sol")
    // id_to_path maps file_id -> relative path (e.g. "34" -> "src/libraries/SwapMath.sol")
    let current_file_id = path_to_file_id.get(abs_path.as_str());

    // Check if cursor is on a Yul external reference first
    for (src_str, decl_id) in external_refs {
        let Some(src_loc) = SourceLoc::parse(src_str.as_str()) else {
            continue;
        };

        // Only consider external refs in the current file
        if let Some(file_id) = current_file_id {
            if src_loc.file_id_str() != **file_id {
                continue;
            }
        } else {
            continue;
        }

        if src_loc.offset <= position && position < src_loc.end() {
            // Found a Yul external reference — resolve to the declaration target
            let mut target_node: Option<&NodeInfo> = None;
            for file_nodes in nodes.values() {
                if let Some(node) = file_nodes.get(decl_id) {
                    target_node = Some(node);
                    break;
                }
            }
            let node = target_node?;
            let loc_str = node.name_location.as_deref().unwrap_or(node.src.as_str());
            let loc = SourceLoc::parse(loc_str)?;
            let file_path = id_to_path.get(&loc.file_id_str())?.clone();
            return Some((file_path, loc.offset, loc.length));
        }
    }

    let mut refs = HashMap::new();

    // Only consider nodes from the current file that have references
    for (id, content) in current_file_nodes {
        if content.referenced_declaration.is_none() {
            continue;
        }

        let Some(src_loc) = SourceLoc::parse(content.src.as_str()) else {
            continue;
        };

        if src_loc.offset <= position && position < src_loc.end() {
            let diff = src_loc.length;
            if !refs.contains_key(&diff) || refs[&diff] <= *id {
                refs.insert(diff, *id);
            }
        }
    }

    if refs.is_empty() {
        // Check if we're on the string part of an import statement
        // ImportDirective nodes have absolutePath pointing to the imported file
        let tmp = current_file_nodes.iter();
        for (_id, content) in tmp {
            if content.node_type == Some("ImportDirective".to_string()) {
                let Some(src_loc) = SourceLoc::parse(content.src.as_str()) else {
                    continue;
                };

                if src_loc.offset <= position
                    && position < src_loc.end()
                    && let Some(import_path) = &content.absolute_path
                {
                    return Some((import_path.clone(), 0, 0));
                }
            }
        }
        return None;
    }

    // Find the reference with minimum diff (most specific)
    let min_diff = *refs.keys().min()?;
    let chosen_id = refs[&min_diff];
    let ref_id = current_file_nodes[&chosen_id].referenced_declaration?;

    // Search for the referenced declaration across all files
    let mut target_node: Option<&NodeInfo> = None;
    for file_nodes in nodes.values() {
        if let Some(node) = file_nodes.get(&ref_id) {
            target_node = Some(node);
            break;
        }
    }

    let node = target_node?;

    // Get location from nameLocation or src
    let loc_str = node.name_location.as_deref().unwrap_or(node.src.as_str());
    let loc = SourceLoc::parse(loc_str)?;
    let file_path = id_to_path.get(&loc.file_id_str())?.clone();

    Some((file_path, loc.offset, loc.length))
}

/// Check if cursor is on the qualifier segment (first `nameLocations` entry)
/// of a multi-segment `IdentifierPath` (e.g., `Pool` in `Pool.State`).
/// If so, resolve the container declaration via `referencedDeclaration → scope`
/// and return a Location pointing to the container's name.
fn resolve_qualifier_goto(
    build: &CachedBuild,
    file_uri: &Url,
    byte_position: usize,
) -> Option<Location> {
    let path = file_uri.to_file_path().ok()?;
    let path_str = path.to_str()?;
    let abs_path = build.path_to_abs.get(path_str)?;
    let file_nodes = build.nodes.get(abs_path)?;

    // Find the IdentifierPath node under the cursor.
    let node_id = crate::references::byte_to_id(&build.nodes, abs_path, byte_position)?;
    let node_info = file_nodes.get(&node_id)?;

    // Must be a multi-segment IdentifierPath.
    if node_info.node_type.as_deref() != Some("IdentifierPath")
        || node_info.name_locations.len() <= 1
    {
        return None;
    }

    // Check if cursor is on the first segment (the qualifier).
    let first_loc = SourceLoc::parse(&node_info.name_locations[0])?;
    if byte_position < first_loc.offset || byte_position >= first_loc.end() {
        return None;
    }

    // Follow referencedDeclaration to the declaration node, then read scope.
    let ref_decl_id = node_info.referenced_declaration?;
    // Find the declaration node to get its scope.
    let decl_node = find_node_info(&build.nodes, ref_decl_id)?;
    let scope_id = decl_node.scope?;

    // Resolve the container declaration's location.
    let container_node = find_node_info(&build.nodes, scope_id)?;
    let loc_str = container_node
        .name_location
        .as_deref()
        .unwrap_or(container_node.src.as_str());
    let loc = SourceLoc::parse(loc_str)?;
    let file_path = build.id_to_path_map.get(&loc.file_id_str())?;

    let absolute_path = if std::path::Path::new(file_path).is_absolute() {
        std::path::PathBuf::from(file_path)
    } else {
        std::env::current_dir().ok()?.join(file_path)
    };

    let target_bytes = std::fs::read(&absolute_path).ok()?;
    let start_pos = bytes_to_pos(&target_bytes, loc.offset)?;
    let end_pos = bytes_to_pos(&target_bytes, loc.end())?;
    let target_uri = Url::from_file_path(&absolute_path).ok()?;

    Some(Location {
        uri: target_uri,
        range: Range {
            start: start_pos,
            end: end_pos,
        },
    })
}

/// Find a `NodeInfo` by node ID across all files.
fn find_node_info<'a>(
    nodes: &'a HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
    node_id: NodeId,
) -> Option<&'a NodeInfo> {
    for file_nodes in nodes.values() {
        if let Some(node) = file_nodes.get(&node_id) {
            return Some(node);
        }
    }
    None
}

/// Go-to-declaration using pre-built `CachedBuild` indices.
/// Avoids redundant O(N) AST traversal by reusing cached node maps.
pub fn goto_declaration_cached(
    build: &CachedBuild,
    file_uri: &Url,
    position: Position,
    source_bytes: &[u8],
) -> Option<Location> {
    let byte_position = pos_to_bytes(source_bytes, position);

    // Check if cursor is on the qualifier segment of a multi-segment
    // IdentifierPath (e.g., cursor on `Pool` in `Pool.State`).
    // If so, navigate to the container declaration (via scope) instead
    // of the struct/enum that referencedDeclaration points to.
    if let Some(location) = resolve_qualifier_goto(build, file_uri, byte_position) {
        return Some(location);
    }

    if let Some((file_path, location_bytes, length)) = goto_bytes(
        &build.nodes,
        &build.path_to_abs,
        &build.id_to_path_map,
        &build.external_refs,
        file_uri.as_ref(),
        byte_position,
    ) {
        let target_file_path = std::path::Path::new(&file_path);
        let absolute_path = if target_file_path.is_absolute() {
            target_file_path.to_path_buf()
        } else {
            // Resolve relative paths against the current file's directory,
            // not CWD. This handles solc standard-json output where
            // absolutePath is relative (e.g. "A.sol") and the server's CWD
            // differs from the project root.
            let base = file_uri
                .to_file_path()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_default();
            base.join(target_file_path)
        };

        if let Ok(target_source_bytes) = std::fs::read(&absolute_path)
            && let Some(start_pos) = bytes_to_pos(&target_source_bytes, location_bytes)
            && let Some(end_pos) = bytes_to_pos(&target_source_bytes, location_bytes + length)
            && let Ok(target_uri) = Url::from_file_path(&absolute_path)
        {
            return Some(Location {
                uri: target_uri,
                range: Range {
                    start: start_pos,
                    end: end_pos,
                },
            });
        }
    };

    None
}

/// Name-based AST goto — resolves by searching cached AST nodes for identifiers
/// matching `name` in the current file, then following `referencedDeclaration`.
///
/// Unlike `goto_declaration_cached` which matches by byte offset (breaks on dirty files),
/// this reads the identifier text from the **built source** (on disk) at each node's
/// `src` range and compares it to the cursor name. Works on dirty files because the
/// AST node relationships (referencedDeclaration) are still valid — only the byte
/// offsets in the current buffer are stale.
/// `byte_hint` is the cursor's byte offset in the dirty buffer, used to pick
/// the closest matching node when multiple nodes share the same name (overloads).
pub fn goto_declaration_by_name(
    cached_build: &CachedBuild,
    file_uri: &Url,
    name: &str,
    byte_hint: usize,
) -> Option<Location> {
    let path = match file_uri.as_ref().starts_with("file://") {
        true => &file_uri.as_ref()[7..],
        false => file_uri.as_ref(),
    };
    let abs_path = cached_build.path_to_abs.get(path)?;
    // Read the built source from disk to extract identifier text at src ranges
    let built_source = std::fs::read_to_string(abs_path).ok()?;

    // Collect all matching nodes: (distance_to_hint, span_size, ref_id)
    let mut candidates: Vec<(usize, usize, NodeId)> = Vec::new();

    let tmp = {
        let this = cached_build.nodes.get(abs_path)?;
        this.iter()
    };
    for (_id, node) in tmp {
        let ref_id = match node.referenced_declaration {
            Some(id) => id,
            None => continue,
        };

        // Parse the node's src to get the byte range in the built source
        let Some(src_loc) = SourceLoc::parse(node.src.as_str()) else {
            continue;
        };
        let start = src_loc.offset;
        let length = src_loc.length;

        if start + length > built_source.len() {
            continue;
        }

        let node_text = &built_source[start..start + length];

        // Check if this node's text matches the name we're looking for.
        // For simple identifiers, the text equals the name directly.
        // For member access (e.g. `x.toInt128()`), check if the text contains
        // `.name(` or ends with `.name`.
        let matches = node_text == name
            || node_text.contains(&format!(".{name}("))
            || node_text.ends_with(&format!(".{name}"));

        if matches {
            // Distance from the byte_hint (cursor in dirty buffer) to the
            // node's src range. The closest node is most likely the one the
            // cursor is on, even if byte offsets shifted slightly.
            let distance = if byte_hint >= start && byte_hint < start + length {
                0 // cursor is inside this node's range
            } else if byte_hint < start {
                start - byte_hint
            } else {
                byte_hint - (start + length)
            };
            candidates.push((distance, length, ref_id));
        }
    }

    // Sort by distance (closest to cursor hint), then by span size (narrowest)
    candidates.sort_by_key(|&(dist, span, _)| (dist, span));
    let ref_id = candidates.first()?.2;

    // Find the declaration node across all files
    let mut target_node: Option<&NodeInfo> = None;
    for file_nodes in cached_build.nodes.values() {
        if let Some(node) = file_nodes.get(&ref_id) {
            target_node = Some(node);
            break;
        }
    }

    let node = target_node?;

    // Parse the target's nameLocation or src
    let loc_str = node.name_location.as_deref().unwrap_or(node.src.as_str());
    let loc = SourceLoc::parse(loc_str)?;

    let file_path = cached_build.id_to_path_map.get(&loc.file_id_str())?;
    let location_bytes = loc.offset;
    let length = loc.length;

    let target_file_path = std::path::Path::new(file_path);
    let absolute_path = if target_file_path.is_absolute() {
        target_file_path.to_path_buf()
    } else {
        let base = file_uri
            .to_file_path()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_default();
        base.join(target_file_path)
    };

    let target_source_bytes = std::fs::read(&absolute_path).ok()?;
    let start_pos = bytes_to_pos(&target_source_bytes, location_bytes)?;
    let end_pos = bytes_to_pos(&target_source_bytes, location_bytes + length)?;
    let target_uri = Url::from_file_path(&absolute_path).ok()?;

    Some(Location {
        uri: target_uri,
        range: Range {
            start: start_pos,
            end: end_pos,
        },
    })
}

// ── Tree-sitter enhanced goto ──────────────────────────────────────────────

/// Context extracted from the cursor position via tree-sitter.
#[derive(Debug, Clone)]
pub struct CursorContext {
    /// The identifier text under the cursor.
    pub name: String,
    /// Enclosing function name (if any).
    pub function: Option<String>,
    /// Enclosing contract/interface/library name (if any).
    pub contract: Option<String>,
    /// Object in a member access expression (e.g. `SqrtPriceMath` in
    /// `SqrtPriceMath.getAmount0Delta`). Set when the cursor is on the
    /// property side of a dot expression.
    pub object: Option<String>,
    /// Number of arguments at the call site (for overload disambiguation).
    /// Set when the cursor is on a function name inside a `call_expression`.
    pub arg_count: Option<usize>,
    /// Inferred argument types at the call site (e.g. `["uint160", "uint160", "int128"]`).
    /// `None` entries mean the type couldn't be inferred for that argument.
    pub arg_types: Vec<Option<String>>,
}

/// Parse Solidity source with tree-sitter.
fn ts_parse(source: &str) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_solidity::LANGUAGE.into())
        .expect("failed to load Solidity grammar");
    parser.parse(source, None)
}

/// Validate that the text at a goto target location matches the expected name.
///
/// Used to reject tree-sitter results that land on the wrong identifier.
/// AST results are NOT validated because the AST can legitimately resolve
/// to a different name (e.g. `.selector` → error declaration).
pub fn validate_goto_target(target_source: &str, location: &Location, expected_name: &str) -> bool {
    let line = location.range.start.line as usize;
    let start_col = location.range.start.character as usize;
    let end_col = location.range.end.character as usize;

    if let Some(line_text) = target_source.lines().nth(line)
        && end_col <= line_text.len()
    {
        return &line_text[start_col..end_col] == expected_name;
    }
    // Can't read target — assume valid
    true
}

/// Find the deepest named node at the given byte offset.
fn ts_node_at_byte(node: Node, byte: usize) -> Option<Node> {
    if byte < node.start_byte() || byte >= node.end_byte() {
        return None;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.start_byte() <= byte
            && byte < child.end_byte()
            && let Some(deeper) = ts_node_at_byte(child, byte)
        {
            return Some(deeper);
        }
    }
    Some(node)
}

/// Get the identifier name from a node (first `identifier` child or the node itself).
fn ts_child_id_text<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|c| c.kind() == "identifier" && c.is_named())
        .map(|c| &source[c.byte_range()])
}

/// Infer the type of an expression node using tree-sitter.
///
/// For identifiers, walks up to find the variable declaration and extracts its type.
/// For literals, infers the type from the literal kind.
/// For function calls, returns None (would need return type resolution).
fn infer_argument_type<'a>(arg_node: Node<'a>, source: &'a str) -> Option<String> {
    // Unwrap call_argument → get inner expression
    let expr = if arg_node.kind() == "call_argument" {
        let mut c = arg_node.walk();
        arg_node.children(&mut c).find(|ch| ch.is_named())?
    } else {
        arg_node
    };

    match expr.kind() {
        "identifier" => {
            let var_name = &source[expr.byte_range()];
            // Walk up scopes to find the variable declaration
            find_variable_type(expr, source, var_name)
        }
        "number_literal" | "decimal_number" | "hex_number" => Some("uint256".into()),
        "boolean_literal" => Some("bool".into()),
        "string_literal" | "hex_string_literal" => Some("string".into()),
        _ => None,
    }
}

/// Find the type of a variable by searching upward through enclosing scopes.
///
/// Looks for `parameter`, `variable_declaration`, and `state_variable_declaration`
/// nodes whose identifier matches the variable name.
fn find_variable_type(from: Node, source: &str, var_name: &str) -> Option<String> {
    let mut scope = from.parent();
    while let Some(node) = scope {
        match node.kind() {
            "function_definition" | "modifier_definition" | "constructor_definition" => {
                // Check parameters
                let mut c = node.walk();
                for child in node.children(&mut c) {
                    if child.kind() == "parameter"
                        && let Some(id) = ts_child_id_text(child, source)
                        && id == var_name
                    {
                        // Extract the type from this parameter
                        let mut pc = child.walk();
                        return child
                            .children(&mut pc)
                            .find(|c| {
                                matches!(
                                    c.kind(),
                                    "type_name"
                                        | "primitive_type"
                                        | "user_defined_type"
                                        | "mapping"
                                )
                            })
                            .map(|t| source[t.byte_range()].trim().to_string());
                    }
                }
            }
            "function_body" | "block_statement" | "unchecked_block" => {
                // Check local variable declarations
                let mut c = node.walk();
                for child in node.children(&mut c) {
                    if (child.kind() == "variable_declaration_statement"
                        || child.kind() == "variable_declaration")
                        && let Some(id) = ts_child_id_text(child, source)
                        && id == var_name
                    {
                        let mut pc = child.walk();
                        return child
                            .children(&mut pc)
                            .find(|c| {
                                matches!(
                                    c.kind(),
                                    "type_name"
                                        | "primitive_type"
                                        | "user_defined_type"
                                        | "mapping"
                                )
                            })
                            .map(|t| source[t.byte_range()].trim().to_string());
                    }
                }
            }
            "contract_declaration" | "library_declaration" | "interface_declaration" => {
                // Check state variables
                if let Some(body) = ts_find_child(node, "contract_body") {
                    let mut c = body.walk();
                    for child in body.children(&mut c) {
                        if child.kind() == "state_variable_declaration"
                            && let Some(id) = ts_child_id_text(child, source)
                            && id == var_name
                        {
                            let mut pc = child.walk();
                            return child
                                .children(&mut pc)
                                .find(|c| {
                                    matches!(
                                        c.kind(),
                                        "type_name"
                                            | "primitive_type"
                                            | "user_defined_type"
                                            | "mapping"
                                    )
                                })
                                .map(|t| source[t.byte_range()].trim().to_string());
                        }
                    }
                }
            }
            _ => {}
        }
        scope = node.parent();
    }
    None
}

/// Infer argument types at a call site by examining each `call_argument` child.
fn infer_call_arg_types(call_node: Node, source: &str) -> Vec<Option<String>> {
    let mut cursor = call_node.walk();
    call_node
        .children(&mut cursor)
        .filter(|c| c.kind() == "call_argument")
        .map(|arg| infer_argument_type(arg, source))
        .collect()
}

/// Pick the best overload from multiple declarations based on argument types.
///
/// Strategy:
/// 1. If only one declaration, return it.
/// 2. Filter by argument count first.
/// 3. Among count-matched declarations, score by how many argument types match.
/// 4. Return the highest-scoring declaration.
fn best_overload<'a>(
    decls: &'a [TsDeclaration],
    arg_count: Option<usize>,
    arg_types: &[Option<String>],
) -> Option<&'a TsDeclaration> {
    if decls.len() == 1 {
        return decls.first();
    }
    if decls.is_empty() {
        return None;
    }

    // Filter to only function declarations (skip parameters, variables, etc.)
    let func_decls: Vec<&TsDeclaration> =
        decls.iter().filter(|d| d.param_count.is_some()).collect();

    if func_decls.is_empty() {
        return decls.first();
    }

    // If we have arg_count, filter by it
    let count_matched: Vec<&&TsDeclaration> = if let Some(ac) = arg_count {
        let matched: Vec<_> = func_decls
            .iter()
            .filter(|d| d.param_count == Some(ac))
            .collect();
        if matched.len() == 1 {
            return Some(matched[0]);
        }
        if matched.is_empty() {
            // No count match — fall back to all
            func_decls.iter().collect()
        } else {
            matched
        }
    } else {
        func_decls.iter().collect()
    };

    // Score each candidate by how many argument types match parameter types
    if !arg_types.is_empty() {
        let mut best: Option<(&TsDeclaration, usize)> = None;
        for &&decl in &count_matched {
            let score = arg_types
                .iter()
                .zip(decl.param_types.iter())
                .filter(|(arg_ty, param_ty)| {
                    if let Some(at) = arg_ty {
                        at == param_ty.as_str()
                    } else {
                        false
                    }
                })
                .count();
            if best.is_none() || score > best.unwrap().1 {
                best = Some((decl, score));
            }
        }
        if let Some((decl, _)) = best {
            return Some(decl);
        }
    }

    // Fallback: return first count-matched or first overall
    count_matched.first().map(|d| **d).or(decls.first())
}

/// Extract cursor context: the identifier under the cursor and its ancestor names.
///
/// Walks up the tree-sitter parse tree to find the enclosing function and contract.
pub fn cursor_context(source: &str, position: Position) -> Option<CursorContext> {
    let tree = ts_parse(source)?;
    let byte = pos_to_bytes(source.as_bytes(), position);
    let leaf = ts_node_at_byte(tree.root_node(), byte)?;

    // The leaf should be an identifier (or we find the nearest identifier)
    let id_node = if leaf.kind() == "identifier" {
        leaf
    } else {
        // Check parent — cursor might be just inside a node that contains an identifier
        let parent = leaf.parent()?;
        if parent.kind() == "identifier" {
            parent
        } else {
            return None;
        }
    };

    let name = source[id_node.byte_range()].to_string();
    let mut function = None;
    let mut contract = None;

    // Detect member access: if the identifier is the `property` side of a
    // member_expression (e.g. `SqrtPriceMath.getAmount0Delta`), extract
    // the object name so the caller can resolve cross-file.
    let object = id_node.parent().and_then(|parent| {
        if parent.kind() == "member_expression" {
            let prop = parent.child_by_field_name("property")?;
            // Only set object when cursor is on the property, not the object side
            if prop.id() == id_node.id() {
                let obj = parent.child_by_field_name("object")?;
                Some(source[obj.byte_range()].to_string())
            } else {
                None
            }
        } else {
            None
        }
    });

    // Count arguments and infer types at the call site for overload disambiguation.
    // Walk up from the identifier to find an enclosing `call_expression`,
    // then count its `call_argument` children and infer their types.
    let (arg_count, arg_types) = {
        let mut node = id_node.parent();
        let mut result = (None, vec![]);
        while let Some(n) = node {
            if n.kind() == "call_expression" {
                let types = infer_call_arg_types(n, source);
                result = (Some(types.len()), types);
                break;
            }
            node = n.parent();
        }
        result
    };

    // Walk ancestors
    let mut current = id_node.parent();
    while let Some(node) = current {
        match node.kind() {
            "function_definition" | "modifier_definition" if function.is_none() => {
                function = ts_child_id_text(node, source).map(String::from);
            }
            "constructor_definition" if function.is_none() => {
                function = Some("constructor".into());
            }
            "contract_declaration" | "interface_declaration" | "library_declaration"
                if contract.is_none() =>
            {
                contract = ts_child_id_text(node, source).map(String::from);
            }
            _ => {}
        }
        current = node.parent();
    }

    Some(CursorContext {
        name,
        function,
        contract,
        object,
        arg_count,
        arg_types,
    })
}

/// Information about a declaration found by tree-sitter.
#[derive(Debug, Clone)]
pub struct TsDeclaration {
    /// Position range of the declaration identifier.
    pub range: Range,
    /// What kind of declaration (contract, function, state_variable, etc.).
    pub kind: &'static str,
    /// Container name (contract/struct that owns this declaration).
    pub container: Option<String>,
    /// Number of parameters (for function/modifier declarations).
    pub param_count: Option<usize>,
    /// Parameter type signature (e.g. `["uint160", "uint160", "int128"]`).
    /// Used for overload disambiguation.
    pub param_types: Vec<String>,
}

/// Find all declarations of a name in a source file using tree-sitter.
///
/// Scans the parse tree for declaration nodes (state variables, functions, events,
/// errors, structs, enums, contracts, etc.) whose identifier matches `name`.
pub fn find_declarations_by_name(source: &str, name: &str) -> Vec<TsDeclaration> {
    let tree = match ts_parse(source) {
        Some(t) => t,
        None => return vec![],
    };
    let mut results = Vec::new();
    collect_declarations(tree.root_node(), source, name, None, &mut results);
    results
}

fn collect_declarations(
    node: Node,
    source: &str,
    name: &str,
    container: Option<&str>,
    out: &mut Vec<TsDeclaration>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if !child.is_named() {
            continue;
        }
        match child.kind() {
            "contract_declaration" | "interface_declaration" | "library_declaration" => {
                if let Some(id_name) = ts_child_id_text(child, source) {
                    if id_name == name {
                        out.push(TsDeclaration {
                            range: id_range(child),
                            kind: child.kind(),
                            container: container.map(String::from),
                            param_count: None,
                            param_types: vec![],
                        });
                    }
                    // Recurse into contract body
                    if let Some(body) = ts_find_child(child, "contract_body") {
                        collect_declarations(body, source, name, Some(id_name), out);
                    }
                }
            }
            "function_definition" | "modifier_definition" => {
                if let Some(id_name) = ts_child_id_text(child, source) {
                    if id_name == name {
                        let types = parameter_type_signature(child, source);
                        out.push(TsDeclaration {
                            range: id_range(child),
                            kind: child.kind(),
                            container: container.map(String::from),
                            param_count: Some(types.len()),
                            param_types: types.into_iter().map(String::from).collect(),
                        });
                    }
                    // Check function parameters
                    collect_parameters(child, source, name, container, out);
                    // Recurse into function body for local variables
                    if let Some(body) = ts_find_child(child, "function_body") {
                        collect_declarations(body, source, name, container, out);
                    }
                }
            }
            "constructor_definition" => {
                if name == "constructor" {
                    let types = parameter_type_signature(child, source);
                    out.push(TsDeclaration {
                        range: ts_range(child),
                        kind: "constructor_definition",
                        container: container.map(String::from),
                        param_count: Some(types.len()),
                        param_types: types.into_iter().map(String::from).collect(),
                    });
                }
                // Check constructor parameters
                collect_parameters(child, source, name, container, out);
                if let Some(body) = ts_find_child(child, "function_body") {
                    collect_declarations(body, source, name, container, out);
                }
            }
            "state_variable_declaration" | "variable_declaration" => {
                if let Some(id_name) = ts_child_id_text(child, source)
                    && id_name == name
                {
                    out.push(TsDeclaration {
                        range: id_range(child),
                        kind: child.kind(),
                        container: container.map(String::from),
                        param_count: None,
                        param_types: vec![],
                    });
                }
            }
            "struct_declaration" => {
                if let Some(id_name) = ts_child_id_text(child, source) {
                    if id_name == name {
                        out.push(TsDeclaration {
                            range: id_range(child),
                            kind: "struct_declaration",
                            container: container.map(String::from),
                            param_count: None,
                            param_types: vec![],
                        });
                    }
                    if let Some(body) = ts_find_child(child, "struct_body") {
                        collect_declarations(body, source, name, Some(id_name), out);
                    }
                }
            }
            "enum_declaration" => {
                if let Some(id_name) = ts_child_id_text(child, source) {
                    if id_name == name {
                        out.push(TsDeclaration {
                            range: id_range(child),
                            kind: "enum_declaration",
                            container: container.map(String::from),
                            param_count: None,
                            param_types: vec![],
                        });
                    }
                    // Check enum values
                    if let Some(body) = ts_find_child(child, "enum_body") {
                        let mut ecur = body.walk();
                        for val in body.children(&mut ecur) {
                            if val.kind() == "enum_value" && &source[val.byte_range()] == name {
                                out.push(TsDeclaration {
                                    range: ts_range(val),
                                    kind: "enum_value",
                                    container: Some(id_name.to_string()),
                                    param_count: None,
                                    param_types: vec![],
                                });
                            }
                        }
                    }
                }
            }
            "event_definition" | "error_declaration" => {
                if let Some(id_name) = ts_child_id_text(child, source)
                    && id_name == name
                {
                    out.push(TsDeclaration {
                        range: id_range(child),
                        kind: child.kind(),
                        container: container.map(String::from),
                        param_count: None,
                        param_types: vec![],
                    });
                }
            }
            "user_defined_type_definition" => {
                if let Some(id_name) = ts_child_id_text(child, source)
                    && id_name == name
                {
                    out.push(TsDeclaration {
                        range: id_range(child),
                        kind: "user_defined_type_definition",
                        container: container.map(String::from),
                        param_count: None,
                        param_types: vec![],
                    });
                }
            }
            // Recurse into blocks, if-else, loops, etc.
            _ => {
                collect_declarations(child, source, name, container, out);
            }
        }
    }
}

/// Extract the type signature from a function's parameters.
///
/// Returns a list of type strings, e.g. `["uint160", "uint160", "int128"]`.
/// For complex types (mappings, arrays, user-defined), returns the full
/// text of the type node.
fn parameter_type_signature<'a>(node: Node<'a>, source: &'a str) -> Vec<&'a str> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|c| c.kind() == "parameter")
        .filter_map(|param| {
            let mut pc = param.walk();
            param
                .children(&mut pc)
                .find(|c| {
                    matches!(
                        c.kind(),
                        "type_name" | "primitive_type" | "user_defined_type" | "mapping"
                    )
                })
                .map(|t| source[t.byte_range()].trim())
        })
        .collect()
}

/// Collect parameter declarations from a function/constructor node.
fn collect_parameters(
    node: Node,
    source: &str,
    name: &str,
    container: Option<&str>,
    out: &mut Vec<TsDeclaration>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "parameter"
            && let Some(id_name) = ts_child_id_text(child, source)
            && id_name == name
        {
            out.push(TsDeclaration {
                range: id_range(child),
                kind: "parameter",
                container: container.map(String::from),
                param_count: None,
                param_types: vec![],
            });
        }
    }
}

/// Tree-sitter range helper.
fn ts_range(node: Node) -> Range {
    let s = node.start_position();
    let e = node.end_position();
    Range {
        start: Position::new(s.row as u32, s.column as u32),
        end: Position::new(e.row as u32, e.column as u32),
    }
}

/// Get the range of the identifier child within a declaration node.
fn id_range(node: Node) -> Range {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|c| c.kind() == "identifier" && c.is_named())
        .map(|c| ts_range(c))
        .unwrap_or_else(|| ts_range(node))
}

fn ts_find_child<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|c| c.kind() == kind)
}

/// Tree-sitter enhanced goto definition.
///
/// Uses tree-sitter to find the identifier under the cursor and its scope,
/// then resolves via the CompletionCache (for cross-file/semantic resolution),
/// and finally uses tree-sitter to find the declaration position in the target file.
///
/// Falls back to None if resolution fails — caller should try the existing AST-based path.
pub fn goto_definition_ts(
    source: &str,
    position: Position,
    file_uri: &Url,
    completion_cache: &crate::completion::CompletionCache,
    text_cache: &HashMap<crate::types::DocumentUri, (i32, String)>,
) -> Option<Location> {
    let ctx = cursor_context(source, position)?;

    // Member access: cursor is on `getAmount0Delta` in `SqrtPriceMath.getAmount0Delta`.
    // Look up the object (SqrtPriceMath) in the completion cache to find its file,
    // then search that file for the member declaration.
    // When multiple overloads exist, disambiguate by argument count and types.
    if let Some(obj_name) = &ctx.object {
        if let Some(path) = find_file_for_contract(completion_cache, obj_name, file_uri) {
            let target_source = read_target_source(&path, text_cache)?;
            let target_uri = Url::from_file_path(&path).ok()?;
            let decls = find_declarations_by_name(&target_source, &ctx.name);
            if let Some(d) = best_overload(&decls, ctx.arg_count, &ctx.arg_types) {
                return Some(Location {
                    uri: target_uri,
                    range: d.range,
                });
            }
        }
        // Object might be in the same file (e.g. a struct or contract in this file)
        let decls = find_declarations_by_name(source, &ctx.name);
        if let Some(d) = best_overload(&decls, ctx.arg_count, &ctx.arg_types) {
            return Some(Location {
                uri: file_uri.clone(),
                range: d.range,
            });
        }
    }

    // Step 1: Try to resolve via CompletionCache to find which file + name the declaration is in.
    // Use the scope chain by names: find the contract scope, then resolve the name.
    let resolved = resolve_via_cache(&ctx, file_uri, completion_cache);

    match resolved {
        Some(ResolvedTarget::SameFile) => {
            // Declaration is in the same file — find it with tree-sitter
            find_best_declaration(source, &ctx, file_uri)
        }
        Some(ResolvedTarget::OtherFile { path, name }) => {
            // Declaration is in another file — read target source and find by name
            let target_source = read_target_source(&path, text_cache);
            let target_source = target_source?;
            let target_uri = Url::from_file_path(&path).ok()?;
            let decls = find_declarations_by_name(&target_source, &name);
            decls.first().map(|d| Location {
                uri: target_uri,
                range: d.range,
            })
        }
        None => {
            // CompletionCache couldn't resolve — try same-file tree-sitter lookup as fallback
            find_best_declaration(source, &ctx, file_uri)
        }
    }
}

#[derive(Debug)]
enum ResolvedTarget {
    /// Declaration is in the same file as the usage.
    SameFile,
    /// Declaration is in a different file.
    OtherFile { path: String, name: String },
}

/// Try to resolve an identifier using the CompletionCache.
///
/// Finds the scope by matching ancestor names (contract, function) against
/// the cache's scope data, then resolves the name to a type and traces
/// back to the declaring file.
fn resolve_via_cache(
    ctx: &CursorContext,
    file_uri: &Url,
    cache: &crate::completion::CompletionCache,
) -> Option<ResolvedTarget> {
    // Find the contract scope node_id by name
    let contract_scope = ctx
        .contract
        .as_ref()
        .and_then(|name| cache.name_to_node_id.get(name.as_str()))
        .copied();

    // Try scope-based resolution: look in the contract's scope_declarations
    if let Some(contract_id) = contract_scope {
        // Check function scope if we're inside one
        if let Some(func_name) = &ctx.function {
            // Find the function scope: look for a scope whose parent is this contract
            // and which has a declaration for this function name
            if let Some(func_scope_id) = find_function_scope(cache, contract_id, func_name) {
                // Check declarations in this function scope first
                if let Some(decls) = cache.scope_declarations.get(&func_scope_id)
                    && decls.iter().any(|d| d.name == ctx.name)
                {
                    return Some(ResolvedTarget::SameFile);
                }
            }
        }

        // Check contract scope declarations (state variables, functions)
        if let Some(decls) = cache.scope_declarations.get(&contract_id)
            && decls.iter().any(|d| d.name == ctx.name)
        {
            return Some(ResolvedTarget::SameFile);
        }

        // Check inherited contracts (C3 linearization)
        if let Some(bases) = cache.linearized_base_contracts.get(&contract_id) {
            for &base_id in bases.iter().skip(1) {
                if let Some(decls) = cache.scope_declarations.get(&base_id)
                    && decls.iter().any(|d| d.name == ctx.name)
                {
                    // Found in a base contract — find which file it's in
                    // Reverse lookup: base_id → contract name → file
                    let base_name = cache
                        .name_to_node_id
                        .iter()
                        .find(|&(_, &id)| id == base_id)
                        .map(|(name, _)| name.clone());

                    if let Some(base_name) = base_name
                        && let Some(path) =
                            find_file_for_contract(cache, base_name.as_str(), file_uri)
                    {
                        return Some(ResolvedTarget::OtherFile {
                            path,
                            name: ctx.name.clone(),
                        });
                    }
                    // Base contract might be in the same file
                    return Some(ResolvedTarget::SameFile);
                }
            }
        }
    }

    // Check if the name is a contract/library/interface name
    if cache.name_to_node_id.contains_key(ctx.name.as_str()) {
        // Could be same file or different file — check if it's in the current file
        if let Some(path) = find_file_for_contract(cache, &ctx.name, file_uri) {
            let current_path = file_uri.to_file_path().ok()?;
            let current_str = current_path.to_str()?;
            if path == current_str || path.ends_with(current_str) || current_str.ends_with(&path) {
                return Some(ResolvedTarget::SameFile);
            }
            return Some(ResolvedTarget::OtherFile {
                path,
                name: ctx.name.clone(),
            });
        }
        return Some(ResolvedTarget::SameFile);
    }

    // Flat fallback — name_to_type knows about it but we can't determine the file
    if cache.name_to_type.contains_key(ctx.name.as_str()) {
        return Some(ResolvedTarget::SameFile);
    }

    None
}

/// Find the scope node_id for a function within a contract.
fn find_function_scope(
    cache: &crate::completion::CompletionCache,
    contract_id: NodeId,
    func_name: &str,
) -> Option<NodeId> {
    // Look for a scope whose parent is the contract and which is a function scope.
    // The function name should appear as a declaration in the contract scope,
    // and the function's own scope is the one whose parent is the contract.
    for (&scope_id, &parent_id) in &cache.scope_parent {
        if parent_id == contract_id {
            // This scope's parent is our contract — it might be a function scope.
            // Check if this scope has declarations (functions/blocks do).
            // We also check if the contract declares a function with this name.
            if let Some(contract_decls) = cache.scope_declarations.get(&contract_id)
                && contract_decls.iter().any(|d| d.name == func_name)
            {
                // Found a child scope of the contract — could be the function.
                // Check if this scope_id has child scopes or declarations
                // that match what we'd expect for a function body.
                if cache.scope_declarations.contains_key(&scope_id)
                    || cache.scope_parent.values().any(|&p| p == scope_id)
                {
                    return Some(scope_id);
                }
            }
        }
    }
    None
}

/// Find the file path for a contract by searching the CompletionCache's path_to_file_id.
fn find_file_for_contract(
    cache: &crate::completion::CompletionCache,
    contract_name: &str,
    _file_uri: &Url,
) -> Option<String> {
    // The completion cache doesn't directly map contract → file.
    // But scope_ranges + path_to_file_id can help.
    // For now, check if the contract's node_id appears in any scope_range,
    // then map file_id back to path.
    let node_id = cache.name_to_node_id.get(contract_name)?;
    let scope_range = cache.scope_ranges.iter().find(|r| r.node_id == *node_id)?;
    let file_id = scope_range.file_id;

    // Reverse lookup: file_id → path
    cache
        .path_to_file_id
        .iter()
        .find(|&(_, &fid)| fid == file_id)
        .map(|(path, _)| path.to_string())
}

/// Read source for a target file — prefer text_cache (open buffers), fallback to disk.
fn read_target_source(
    path: &str,
    text_cache: &HashMap<crate::types::DocumentUri, (i32, String)>,
) -> Option<String> {
    // Try text_cache by URI
    let uri = Url::from_file_path(path).ok()?;
    if let Some((_, content)) = text_cache.get(&uri.to_string()) {
        return Some(content.clone());
    }
    // Fallback to disk
    std::fs::read_to_string(path).ok()
}

/// Find the best matching declaration in the same file.
fn find_best_declaration(source: &str, ctx: &CursorContext, file_uri: &Url) -> Option<Location> {
    let decls = find_declarations_by_name(source, &ctx.name);
    if decls.is_empty() {
        return None;
    }

    // If there's only one declaration, use it
    if decls.len() == 1 {
        return Some(Location {
            uri: file_uri.clone(),
            range: decls[0].range,
        });
    }

    // Multiple declarations — prefer the one in the same contract
    if let Some(contract_name) = &ctx.contract
        && let Some(d) = decls
            .iter()
            .find(|d| d.container.as_deref() == Some(contract_name))
    {
        return Some(Location {
            uri: file_uri.clone(),
            range: d.range,
        });
    }

    // Fallback: return first declaration
    Some(Location {
        uri: file_uri.clone(),
        range: decls[0].range,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Code-action helpers (used by lsp.rs `code_action` handler)
// ─────────────────────────────────────────────────────────────────────────────

/// What kind of edit a code action should produce.
#[derive(Debug, Clone, Copy)]
pub(crate) enum CodeActionKind<'a> {
    /// Insert fixed text at the very start of the file (line 0, col 0).
    InsertAtFileStart { text: &'a str },

    /// Replace the token at `diag_range.start` with `replacement`.
    /// Used for deprecated-builtin fixes (now→block.timestamp, sha3→keccak256, …).
    /// When `walk_to` is `Some`, walk up to that ancestor node and replace its
    /// full span instead of just the leaf (e.g. `member_expression` for msg.gas).
    ReplaceToken {
        replacement: &'a str,
        walk_to: Option<&'a str>,
    },

    /// Delete the token whose start byte falls inside `diag_range`
    /// (+ one trailing space when present).
    DeleteToken,

    /// Delete the entire `variable_declaration_statement` containing the
    /// identifier at `diag_range.start`, including leading whitespace/newline.
    DeleteLocalVar,

    /// Walk up the TS tree to the first ancestor whose kind matches `node_kind`,
    /// then delete that whole node including its preceding newline+indentation.
    /// Used for any "delete this whole statement/declaration" fix (e.g. unused import).
    DeleteNodeByKind { node_kind: &'a str },

    /// Walk the TS tree up to `walk_to`, then delete the first child whose
    /// kind matches any entry in `child_kinds` (tried in order).
    DeleteChildNode {
        walk_to: &'a str,
        child_kinds: &'a [&'a str],
    },

    /// Walk the TS tree up to `walk_to`, then replace the first child whose
    /// kind matches `child_kind` with `replacement`.
    ReplaceChildNode {
        walk_to: &'a str,
        child_kind: &'a str,
        replacement: &'a str,
    },

    /// Walk the TS tree up to `walk_to`, then insert `text` immediately before
    /// the first child whose kind matches any entry in `before_child`.
    /// Used for 5424 (insert `virtual` before `returns`/`;`).
    InsertBeforeNode {
        walk_to: &'a str,
        before_child: &'a [&'a str],
        text: &'a str,
    },
}

/// Compute the `TextEdit` for a code action using tree-sitter for precision.
///
/// Returns `None` when the tree cannot be parsed or the target node cannot be
/// located (caller should fall back to returning no action for that diagnostic).
pub(crate) fn code_action_edit(
    source: &str,
    diag_range: Range,
    kind: CodeActionKind<'_>,
) -> Option<TextEdit> {
    let source_bytes = source.as_bytes();

    match kind {
        // ── Insert fixed text at the top of the file ──────────────────────────
        CodeActionKind::InsertAtFileStart { text } => Some(TextEdit {
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
            new_text: text.to_string(),
        }),

        // ── Replace the token at diag_range.start ─────────────────────────────
        CodeActionKind::ReplaceToken {
            replacement,
            walk_to,
        } => {
            let tree = ts_parse(source)?;
            let byte = pos_to_bytes(source_bytes, diag_range.start);
            let mut node = ts_node_at_byte(tree.root_node(), byte)?;
            // When a walk_to node kind is specified, walk up to that ancestor
            // so we replace its full span (e.g. `member_expression` for
            // `msg.gas` or `block.blockhash`).
            if let Some(target_kind) = walk_to {
                loop {
                    if node.kind() == target_kind {
                        break;
                    }
                    node = node.parent()?;
                }
            }
            let start_pos = bytes_to_pos(source_bytes, node.start_byte())?;
            let end_pos = bytes_to_pos(source_bytes, node.end_byte())?;
            Some(TextEdit {
                range: Range {
                    start: start_pos,
                    end: end_pos,
                },
                new_text: replacement.to_string(),
            })
        }

        // ── Delete the token at diag_range.start (+ optional trailing space) ──
        CodeActionKind::DeleteToken => {
            let tree = ts_parse(source)?;
            let byte = pos_to_bytes(source_bytes, diag_range.start);
            let node = ts_node_at_byte(tree.root_node(), byte)?;
            let start = node.start_byte();
            let end =
                if node.end_byte() < source_bytes.len() && source_bytes[node.end_byte()] == b' ' {
                    node.end_byte() + 1
                } else {
                    node.end_byte()
                };
            let start_pos = bytes_to_pos(source_bytes, start)?;
            let end_pos = bytes_to_pos(source_bytes, end)?;
            Some(TextEdit {
                range: Range {
                    start: start_pos,
                    end: end_pos,
                },
                new_text: String::new(),
            })
        }

        // ── Delete the enclosing variable_declaration_statement ───────────────
        CodeActionKind::DeleteLocalVar => {
            let tree = ts_parse(source)?;
            let byte = pos_to_bytes(source_bytes, diag_range.start);
            let mut node = ts_node_at_byte(tree.root_node(), byte)?;

            loop {
                if node.kind() == "variable_declaration_statement" {
                    break;
                }
                node = node.parent()?;
            }

            // Consume the preceding newline + indentation so no blank line remains.
            let stmt_start = node.start_byte();
            let delete_from = if stmt_start > 0 {
                let mut i = stmt_start - 1;
                while i > 0 && (source_bytes[i] == b' ' || source_bytes[i] == b'\t') {
                    i -= 1;
                }
                if source_bytes[i] == b'\n' {
                    i
                } else {
                    stmt_start
                }
            } else {
                stmt_start
            };

            let start_pos = bytes_to_pos(source_bytes, delete_from)?;
            let end_pos = bytes_to_pos(source_bytes, node.end_byte())?;
            Some(TextEdit {
                range: Range {
                    start: start_pos,
                    end: end_pos,
                },
                new_text: String::new(),
            })
        }

        // ── Walk up to `node_kind`, delete that whole node (+ preceding newline) ─
        CodeActionKind::DeleteNodeByKind { node_kind } => {
            let tree = ts_parse(source)?;
            let byte = pos_to_bytes(source_bytes, diag_range.start);
            let mut node = ts_node_at_byte(tree.root_node(), byte)?;
            loop {
                if node.kind() == node_kind {
                    break;
                }
                node = node.parent()?;
            }
            // Consume the preceding newline + indentation so no blank line remains.
            let node_start = node.start_byte();
            let delete_from = if node_start > 0 {
                let mut i = node_start - 1;
                while i > 0 && (source_bytes[i] == b' ' || source_bytes[i] == b'\t') {
                    i -= 1;
                }
                if source_bytes[i] == b'\n' {
                    i
                } else {
                    node_start
                }
            } else {
                node_start
            };
            let start_pos = bytes_to_pos(source_bytes, delete_from)?;
            let end_pos = bytes_to_pos(source_bytes, node.end_byte())?;
            Some(TextEdit {
                range: Range {
                    start: start_pos,
                    end: end_pos,
                },
                new_text: String::new(),
            })
        }

        // ── Walk up to `walk_to`, delete first child of `child_kind` ─────────
        //
        // Used for 4126 (free function visibility) and payable codes where the
        // diagnostic points to the whole function, not the bad token.
        CodeActionKind::DeleteChildNode {
            walk_to,
            child_kinds,
        } => {
            let tree = ts_parse(source)?;
            let byte = pos_to_bytes(source_bytes, diag_range.start);
            let mut node = ts_node_at_byte(tree.root_node(), byte)?;
            loop {
                if node.kind() == walk_to {
                    break;
                }
                node = node.parent()?;
            }
            let mut cursor = node.walk();
            let children: Vec<_> = node.children(&mut cursor).collect();
            let child = child_kinds
                .iter()
                .find_map(|k| children.iter().find(|c| c.kind() == *k))?;
            let start = child.start_byte();
            let end = if child.end_byte() < source_bytes.len()
                && source_bytes[child.end_byte()] == b' '
            {
                child.end_byte() + 1
            } else {
                child.end_byte()
            };
            let start_pos = bytes_to_pos(source_bytes, start)?;
            let end_pos = bytes_to_pos(source_bytes, end)?;
            Some(TextEdit {
                range: Range {
                    start: start_pos,
                    end: end_pos,
                },
                new_text: String::new(),
            })
        }

        // ── Walk up to `walk_to`, replace first child of `child_kind` ─────────
        //
        // Used for 1560/1159/4095: replace wrong visibility with `external`.
        CodeActionKind::ReplaceChildNode {
            walk_to,
            child_kind,
            replacement,
        } => {
            let tree = ts_parse(source)?;
            let byte = pos_to_bytes(source_bytes, diag_range.start);
            let mut node = ts_node_at_byte(tree.root_node(), byte)?;
            loop {
                if node.kind() == walk_to {
                    break;
                }
                node = node.parent()?;
            }
            let mut cursor = node.walk();
            let child = node
                .children(&mut cursor)
                .find(|c| c.kind() == child_kind)?;
            let start_pos = bytes_to_pos(source_bytes, child.start_byte())?;
            let end_pos = bytes_to_pos(source_bytes, child.end_byte())?;
            Some(TextEdit {
                range: Range {
                    start: start_pos,
                    end: end_pos,
                },
                new_text: replacement.to_string(),
            })
        }

        // ── Walk up to `walk_to`, insert `text` before first matching child ───
        //
        // Used for 5424 (insert `virtual` before `return_type_definition` / `;`).
        //
        // `before_child` is tried in order — the first matching child kind wins.
        // This lets callers express "prefer returns, fall back to semicolon".
        CodeActionKind::InsertBeforeNode {
            walk_to,
            before_child,
            text,
        } => {
            let tree = ts_parse(source)?;
            let byte = pos_to_bytes(source_bytes, diag_range.start);
            let mut node = ts_node_at_byte(tree.root_node(), byte)?;

            loop {
                if node.kind() == walk_to {
                    break;
                }
                node = node.parent()?;
            }

            let mut cursor = node.walk();
            let children: Vec<_> = node.children(&mut cursor).collect();

            // Try each `before_child` kind in order.
            for target_kind in before_child {
                if let Some(child) = children.iter().find(|c| c.kind() == *target_kind) {
                    let insert_pos = bytes_to_pos(source_bytes, child.start_byte())?;
                    return Some(TextEdit {
                        range: Range {
                            start: insert_pos,
                            end: insert_pos,
                        },
                        new_text: text.to_string(),
                    });
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod ts_tests {
    use super::*;

    #[test]
    fn test_cursor_context_state_var() {
        let source = r#"
contract Token {
    uint256 public totalSupply;
    function mint(uint256 amount) public {
        totalSupply += amount;
    }
}
"#;
        // Cursor on `totalSupply` inside mint (line 4, col 8)
        let ctx = cursor_context(source, Position::new(4, 8)).unwrap();
        assert_eq!(ctx.name, "totalSupply");
        assert_eq!(ctx.function.as_deref(), Some("mint"));
        assert_eq!(ctx.contract.as_deref(), Some("Token"));
    }

    #[test]
    fn test_cursor_context_top_level() {
        let source = r#"
contract Foo {}
contract Bar {}
"#;
        // Cursor on `Foo` (line 1, col 9) — the identifier of the contract declaration
        let ctx = cursor_context(source, Position::new(1, 9)).unwrap();
        assert_eq!(ctx.name, "Foo");
        assert!(ctx.function.is_none());
        // The identifier `Foo` is a child of contract_declaration, so contract is set
        assert_eq!(ctx.contract.as_deref(), Some("Foo"));
    }

    #[test]
    fn test_find_declarations() {
        let source = r#"
contract Token {
    uint256 public totalSupply;
    function mint(uint256 amount) public {
        totalSupply += amount;
    }
}
"#;
        let decls = find_declarations_by_name(source, "totalSupply");
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].kind, "state_variable_declaration");
        assert_eq!(decls[0].container.as_deref(), Some("Token"));
    }

    #[test]
    fn test_find_declarations_multiple_contracts() {
        let source = r#"
contract A {
    uint256 public value;
}
contract B {
    uint256 public value;
}
"#;
        let decls = find_declarations_by_name(source, "value");
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].container.as_deref(), Some("A"));
        assert_eq!(decls[1].container.as_deref(), Some("B"));
    }

    #[test]
    fn test_find_declarations_enum_value() {
        let source = "contract Foo { enum Status { Active, Paused } }";
        let decls = find_declarations_by_name(source, "Active");
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].kind, "enum_value");
        assert_eq!(decls[0].container.as_deref(), Some("Status"));
    }

    #[test]
    fn test_cursor_context_short_param() {
        let source = r#"
contract Shop {
    uint256 public TAX;
    constructor(uint256 price, uint16 tax, uint16 taxBase) {
        TAX = tax;
    }
}
"#;
        // Cursor on `tax` usage at line 4, col 14 (TAX = tax;)
        let ctx = cursor_context(source, Position::new(4, 14)).unwrap();
        assert_eq!(ctx.name, "tax");
        assert_eq!(ctx.contract.as_deref(), Some("Shop"));

        // Cursor on `TAX` at line 4, col 8
        let ctx2 = cursor_context(source, Position::new(4, 8)).unwrap();
        assert_eq!(ctx2.name, "TAX");

        // Parameters are found as declarations
        let decls = find_declarations_by_name(source, "tax");
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].kind, "parameter");

        let decls_tax_base = find_declarations_by_name(source, "taxBase");
        assert_eq!(decls_tax_base.len(), 1);
        assert_eq!(decls_tax_base[0].kind, "parameter");

        let decls_price = find_declarations_by_name(source, "price");
        assert_eq!(decls_price.len(), 1);
        assert_eq!(decls_price[0].kind, "parameter");

        // State variable is also found
        let decls_tax_upper = find_declarations_by_name(source, "TAX");
        assert_eq!(decls_tax_upper.len(), 1);
        assert_eq!(decls_tax_upper[0].kind, "state_variable_declaration");
    }

    #[test]
    fn test_delete_child_node_2462_constructor_public() {
        // ConstructorVisibility.sol: `constructor() public {}`
        // Diagnostic range: start={line:9, char:4}, end={line:11, char:5}
        let source = "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.26;\n\n// Warning 2462\n\ncontract ConstructorVisibility {\n    uint256 public value;\n\n    constructor() public {\n        value = 1;\n    }\n}\n";
        let diag_range = Range {
            start: Position {
                line: 8,
                character: 4,
            },
            end: Position {
                line: 10,
                character: 5,
            },
        };
        let source_bytes = source.as_bytes();
        let tree = ts_parse(source).expect("parse failed");
        let byte = pos_to_bytes(source_bytes, diag_range.start);
        eprintln!("2462 byte offset: {byte}");
        if let Some(mut n) = ts_node_at_byte(tree.root_node(), byte) {
            loop {
                eprintln!(
                    "  ancestor: kind={} start={} end={}",
                    n.kind(),
                    n.start_byte(),
                    n.end_byte()
                );
                if n.kind() == "constructor_definition" {
                    let mut cursor = n.walk();
                    for child in n.children(&mut cursor) {
                        eprintln!(
                            "    child: kind={:?} text={:?}",
                            child.kind(),
                            &source[child.start_byte()..child.end_byte()]
                        );
                    }
                    break;
                }
                match n.parent() {
                    Some(p) => n = p,
                    None => break,
                }
            }
        }
        let ck: Vec<&str> = vec!["public", "modifier_invocation"];
        let edit = code_action_edit(
            source,
            diag_range,
            CodeActionKind::DeleteChildNode {
                walk_to: "constructor_definition",
                child_kinds: &ck,
            },
        );
        assert!(edit.is_some(), "2462 fix returned None");
        let edit = edit.unwrap();
        assert_eq!(edit.new_text, "");
        let lines: Vec<&str> = source.lines().collect();
        let deleted = &lines[edit.range.start.line as usize]
            [edit.range.start.character as usize..edit.range.end.character as usize];
        assert!(deleted.contains("public"), "deleted: {:?}", deleted);
    }

    #[test]
    fn test_delete_child_node_9239_constructor_private() {
        // Fixture mirrors example/codeaction/ConstructorInvalidVisibility.sol
        // Diagnostic range from server: start={line:9, char:4}, end={line:11, char:5}
        let source = "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.26;\n\n// Error 9239: Constructor cannot have visibility.\n// Fix: remove the visibility specifier (private/external) from the constructor.\n\ncontract ConstructorPrivateVisibility {\n    uint256 public value;\n\n    constructor() private {\n        value = 1;\n    }\n}\n";
        let diag_range = Range {
            start: Position {
                line: 9,
                character: 4,
            },
            end: Position {
                line: 11,
                character: 5,
            },
        };
        let ck: Vec<&str> = vec!["modifier_invocation"];
        let edit = code_action_edit(
            source,
            diag_range,
            CodeActionKind::DeleteChildNode {
                walk_to: "constructor_definition",
                child_kinds: &ck,
            },
        );
        assert!(edit.is_some(), "expected Some(TextEdit) for 9239, got None");
        let edit = edit.unwrap();
        // The edit should delete 'private ' — new_text must be empty
        assert_eq!(edit.new_text, "", "expected deletion (empty new_text)");
        // The deleted text should contain 'private'
        let lines: Vec<&str> = source.lines().collect();
        let line_text = lines[edit.range.start.line as usize];
        let deleted =
            &line_text[edit.range.start.character as usize..edit.range.end.character as usize];
        assert!(
            deleted.contains("private"),
            "expected deleted text to contain 'private', got: {:?}",
            deleted
        );
    }

    #[test]
    fn test_find_best_declaration_same_contract() {
        let source = r#"
contract A { uint256 public x; }
contract B { uint256 public x; }
"#;
        let ctx = CursorContext {
            name: "x".into(),
            function: None,
            contract: Some("B".into()),
            object: None,
            arg_count: None,
            arg_types: vec![],
        };
        let uri = Url::parse("file:///test.sol").unwrap();
        let loc = find_best_declaration(source, &ctx, &uri).unwrap();
        // Should pick B's x (line 2), not A's x (line 1)
        assert_eq!(loc.range.start.line, 2);
    }
}
// temp
