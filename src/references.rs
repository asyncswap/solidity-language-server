use std::collections::{HashMap, HashSet};
use tower_lsp::lsp_types::{Location, Position, Range, Url};

use crate::goto::{
    CachedBuild, ExternalRefs, NodeInfo, bytes_to_pos, pos_to_bytes, src_to_location,
};
use crate::types::{AbsPath, NodeId, SourceLoc};

pub fn all_references(
    nodes: &HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
) -> HashMap<NodeId, Vec<NodeId>> {
    let mut all_refs: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for file_nodes in nodes.values() {
        for (node_id, node_info) in file_nodes {
            if let Some(reference_id) = node_info.referenced_declaration {
                all_refs.entry(reference_id).or_default().push(*node_id);
                all_refs.entry(*node_id).or_default().push(reference_id);
            }
        }
    }
    all_refs
}

/// Check if cursor byte position falls on a Yul external reference in the given file.
/// Returns the Solidity declaration id if so.
pub fn byte_to_decl_via_external_refs(
    external_refs: &ExternalRefs,
    id_to_path: &HashMap<crate::types::SolcFileId, String>,
    abs_path: &str,
    byte_position: usize,
) -> Option<NodeId> {
    // Build reverse map: file_path -> file_id
    let path_to_file_id: HashMap<&str, &crate::types::SolcFileId> =
        id_to_path.iter().map(|(id, p)| (p.as_str(), id)).collect();
    let current_file_id = path_to_file_id.get(abs_path)?;

    for (src_str, decl_id) in external_refs {
        let Some(src_loc) = SourceLoc::parse(src_str.as_str()) else {
            continue;
        };
        // Only consider refs in the current file
        if src_loc.file_id_str() != **current_file_id {
            continue;
        }
        if src_loc.offset <= byte_position && byte_position < src_loc.end() {
            return Some(*decl_id);
        }
    }
    None
}

pub fn byte_to_id(
    nodes: &HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
    abs_path: &str,
    byte_position: usize,
) -> Option<NodeId> {
    let file_nodes = nodes.get(abs_path)?;
    let mut refs: HashMap<usize, (NodeId, bool)> = HashMap::new();
    for (id, node_info) in file_nodes {
        let Some(src_loc) = SourceLoc::parse(node_info.src.as_str()) else {
            continue;
        };

        if src_loc.offset <= byte_position && byte_position < src_loc.end() {
            let diff = src_loc.length;
            let has_ref = node_info.referenced_declaration.is_some();
            match refs.entry(diff) {
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert((*id, has_ref));
                }
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    // When two nodes share the same span length, prefer the one
                    // with referencedDeclaration set. This resolves ambiguity
                    // between InheritanceSpecifier and its child baseName
                    // IdentifierPath — both have identical src ranges but only
                    // the IdentifierPath carries referencedDeclaration.
                    if has_ref && !e.get().1 {
                        e.insert((*id, has_ref));
                    }
                }
            }
        }
    }
    refs.keys().min().map(|min_diff| refs[min_diff].0)
}

pub fn id_to_location(
    nodes: &HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
    id_to_path: &HashMap<crate::types::SolcFileId, String>,
    node_id: NodeId,
) -> Option<Location> {
    id_to_location_with_index(nodes, id_to_path, node_id, None)
}

pub fn id_to_location_with_index(
    nodes: &HashMap<AbsPath, HashMap<NodeId, NodeInfo>>,
    id_to_path: &HashMap<crate::types::SolcFileId, String>,
    node_id: NodeId,
    name_location_index: Option<usize>,
) -> Option<Location> {
    let mut target_node: Option<&NodeInfo> = None;
    for file_nodes in nodes.values() {
        if let Some(node) = file_nodes.get(&node_id) {
            target_node = Some(node);
            break;
        }
    }
    let node = target_node?;

    let loc_str = if let Some(index) = name_location_index
        && let Some(name_loc) = node.name_locations.get(index)
    {
        name_loc.as_str()
    } else if let Some(name_location) = &node.name_location {
        name_location.as_str()
    } else {
        // Fallback to src location for nodes without nameLocation
        node.src.as_str()
    };

    let loc = SourceLoc::parse(loc_str)?;
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

/// Find all references using pre-built `CachedBuild` indices.
/// Avoids redundant O(N) AST traversal by reusing cached node maps.
pub fn goto_references_cached(
    build: &CachedBuild,
    file_uri: &Url,
    position: Position,
    source_bytes: &[u8],
    name_location_index: Option<usize>,
    include_declaration: bool,
) -> Vec<Location> {
    let all_refs = all_references(&build.nodes);
    let path = match file_uri.to_file_path() {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    let path_str = match path.to_str() {
        Some(s) => s,
        None => return vec![],
    };
    let abs_path = match build.path_to_abs.get(path_str) {
        Some(ap) => ap,
        None => return vec![],
    };
    let byte_position = pos_to_bytes(source_bytes, position);

    // Check if cursor is on a Yul external reference first
    let target_node_id = if let Some(decl_id) = byte_to_decl_via_external_refs(
        &build.external_refs,
        &build.id_to_path_map,
        abs_path,
        byte_position,
    ) {
        decl_id
    } else {
        let node_id = match byte_to_id(&build.nodes, abs_path, byte_position) {
            Some(id) => id,
            None => return vec![],
        };
        let file_nodes = match build.nodes.get(abs_path) {
            Some(nodes) => nodes,
            None => return vec![],
        };
        if let Some(node_info) = file_nodes.get(&node_id) {
            node_info.referenced_declaration.unwrap_or(node_id)
        } else {
            node_id
        }
    };

    let mut results: HashSet<NodeId> = HashSet::new();
    if include_declaration {
        results.insert(target_node_id);
    }
    if let Some(refs) = all_refs.get(&target_node_id) {
        results.extend(refs.iter().copied());
    }
    let mut locations = Vec::new();
    for id in results {
        if let Some(location) =
            id_to_location_with_index(&build.nodes, &build.id_to_path_map, id, name_location_index)
        {
            locations.push(location);
        }
    }

    // Also add Yul external reference use sites
    for (src_str, decl_id) in &build.external_refs {
        if *decl_id == target_node_id
            && let Some(location) = src_to_location(src_str.as_str(), &build.id_to_path_map)
        {
            locations.push(location);
        }
    }

    let mut unique_locations = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for location in locations {
        let key = (
            location.uri.clone(),
            location.range.start.line,
            location.range.start.character,
            location.range.end.line,
            location.range.end.character,
        );
        if seen.insert(key) {
            unique_locations.push(location);
        }
    }
    unique_locations
}

/// Resolve cursor position to the target definition's location (abs_path + byte offset).
/// Node IDs are not stable across builds, but byte offsets within a file are.
/// Returns (abs_path, byte_offset) of the definition node, usable with byte_to_id
/// in any other build that includes that file.
pub fn resolve_target_location(
    build: &CachedBuild,
    file_uri: &Url,
    position: Position,
    source_bytes: &[u8],
) -> Option<(String, usize)> {
    let path = file_uri.to_file_path().ok()?;
    let path_str = path.to_str()?;
    let abs_path = build.path_to_abs.get(path_str)?;
    let byte_position = pos_to_bytes(source_bytes, position);

    // Check Yul external references first
    let target_node_id = if let Some(decl_id) = byte_to_decl_via_external_refs(
        &build.external_refs,
        &build.id_to_path_map,
        abs_path,
        byte_position,
    ) {
        decl_id
    } else {
        let node_id = byte_to_id(&build.nodes, abs_path, byte_position)?;
        let file_nodes = build.nodes.get(abs_path)?;
        if let Some(node_info) = file_nodes.get(&node_id) {
            node_info.referenced_declaration.unwrap_or(node_id)
        } else {
            node_id
        }
    };

    // Find the definition node and extract its file + byte offset.
    // Prefer `nameLocation` over `src` — for declarations like
    // `IPoolManager manager;`, `src` spans the entire declaration
    // (starting at `IPoolManager`) while `nameLocation` points at
    // `manager`. Using `src.offset` would cause `byte_to_id` in other
    // builds to land on the type name node instead of the variable,
    // contaminating cross-file references with the type's references.
    for (file_abs_path, file_nodes) in &build.nodes {
        if let Some(node_info) = file_nodes.get(&target_node_id) {
            let loc_str = node_info
                .name_location
                .as_deref()
                .unwrap_or(node_info.src.as_str());
            if let Some(src_loc) = SourceLoc::parse(loc_str) {
                return Some((file_abs_path.to_string(), src_loc.offset));
            }
        }
    }
    None
}

/// Find all references to a definition in a single AST build, identified by
/// the definition's file path + byte offset (stable across builds).
/// Uses byte_to_id to find this build's node ID for the same definition,
/// then scans for referenced_declaration matches.
///
/// When `exclude_abs_path` is provided, results whose resolved file path
/// matches that path are skipped.  This prevents stale AST byte offsets
/// from producing bogus locations when the caller already has correct
/// results for that file from a fresher build (e.g. the per-file build
/// compiled from the current editor buffer).
pub fn goto_references_for_target(
    build: &CachedBuild,
    def_abs_path: &str,
    def_byte_offset: usize,
    name_location_index: Option<usize>,
    include_declaration: bool,
    exclude_abs_path: Option<&str>,
) -> Vec<Location> {
    // Find this build's node ID for the definition using byte offset
    let target_node_id = match byte_to_id(&build.nodes, def_abs_path, def_byte_offset) {
        Some(id) => {
            // If it's a reference, follow to the definition
            if let Some(file_nodes) = build.nodes.get(def_abs_path) {
                if let Some(node_info) = file_nodes.get(&id) {
                    node_info.referenced_declaration.unwrap_or(id)
                } else {
                    id
                }
            } else {
                id
            }
        }
        None => return vec![],
    };

    // Build a set of node IDs that live in the excluded file so we can
    // skip them cheaply during the id_to_location loop.
    let excluded_ids: HashSet<NodeId> = if let Some(excl) = exclude_abs_path {
        build
            .nodes
            .get(excl)
            .map(|file_nodes| file_nodes.keys().copied().collect())
            .unwrap_or_default()
    } else {
        HashSet::new()
    };

    // Collect the definition node + all nodes whose referenced_declaration matches
    let mut results: HashSet<NodeId> = HashSet::new();
    if include_declaration {
        results.insert(target_node_id);
    }
    for file_nodes in build.nodes.values() {
        for (id, node_info) in file_nodes {
            if node_info.referenced_declaration == Some(target_node_id) {
                results.insert(*id);
            }
        }
    }

    let mut locations = Vec::new();
    for id in results {
        if excluded_ids.contains(&id) {
            continue;
        }
        if let Some(location) =
            id_to_location_with_index(&build.nodes, &build.id_to_path_map, id, name_location_index)
        {
            locations.push(location);
        }
    }

    // Yul external reference use sites
    for (src_str, decl_id) in &build.external_refs {
        if *decl_id == target_node_id {
            // Skip external refs in the excluded file.
            if let Some(excl) = exclude_abs_path {
                if let Some(src_loc) = SourceLoc::parse(src_str.as_str()) {
                    if let Some(ref_path) = build.id_to_path_map.get(&src_loc.file_id_str()) {
                        if ref_path == excl {
                            continue;
                        }
                    }
                }
            }
            if let Some(location) = src_to_location(src_str.as_str(), &build.id_to_path_map) {
                locations.push(location);
            }
        }
    }

    locations
}
