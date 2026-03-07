use serde_json::Value;
use solidity_language_server::goto::CachedBuild;
use solidity_language_server::references;
use solidity_language_server::types::NodeId;
use std::collections::HashSet;
use std::fs;

/// Load poolmanager.json as a CachedBuild.
fn load_cached_build() -> CachedBuild {
    let raw: Value =
        serde_json::from_str(&fs::read_to_string("poolmanager.json").unwrap()).unwrap();
    let ast_data = solidity_language_server::solc::normalize_solc_output(raw, None);
    CachedBuild::new(ast_data, 0, None)
}

/// Find the abs-path key that ends with the given suffix.
fn find_key<'a>(build: &'a CachedBuild, suffix: &str) -> &'a str {
    build
        .nodes
        .keys()
        .find(|k| k.ends_with(suffix))
        .unwrap_or_else(|| panic!("no key ending with {suffix}"))
}

// =============================================================================
// CachedBuild construction tests
// =============================================================================

#[test]
fn test_cached_build_has_nodes() {
    let build = load_cached_build();
    assert!(
        !build.nodes.is_empty(),
        "CachedBuild should have nodes after construction"
    );
}

#[test]
fn test_cached_build_has_path_to_abs() {
    let build = load_cached_build();
    assert!(
        !build.path_to_abs.is_empty(),
        "CachedBuild should have path_to_abs mappings"
    );
    let has_pm = build
        .path_to_abs
        .values()
        .any(|v| v.contains("PoolManager"));
    assert!(has_pm, "path_to_abs should contain PoolManager.sol");
}

#[test]
fn test_cached_build_has_id_to_path_map() {
    let build = load_cached_build();
    assert!(
        !build.id_to_path_map.is_empty(),
        "CachedBuild should have id_to_path_map"
    );
    // Hooks.sol has source id 22 in the solc fixture
    let hooks_path = build.id_to_path_map.get("22").expect("id 22 should exist");
    assert!(
        hooks_path.ends_with("src/libraries/Hooks.sol"),
        "id 22 should map to Hooks.sol, got: {}",
        hooks_path
    );
}

#[test]
fn test_cached_build_has_external_refs() {
    let build = load_cached_build();
    assert!(
        !build.external_refs.is_empty(),
        "CachedBuild should have external_refs for Yul identifiers"
    );
}

#[test]
fn test_cached_build_drops_raw_ast() {
    let build = load_cached_build();
    // Raw AST JSON is fully consumed during construction — no `ast` field retained.
    // All data lives in pre-built indexes: nodes, decl_index, gas_index, doc_index, etc.
    assert!(
        !build.decl_index.is_empty(),
        "decl_index should be populated from the consumed AST"
    );
    assert!(
        !build.id_to_path_map.is_empty(),
        "id_to_path_map should be populated from source_id_to_path"
    );
}

// =============================================================================
// byte_to_id tests — the core cross-file bridging mechanism
// =============================================================================

#[test]
fn test_byte_to_id_finds_hooks_definition() {
    let build = load_cached_build();
    let hooks_key = find_key(&build, "src/libraries/Hooks.sol");

    // library Hooks: id=3530, src="1039:15471:22"
    let node_id = references::byte_to_id(&build.nodes, hooks_key, 1039);
    assert!(node_id.is_some(), "should find node at byte 1039");

    let id = node_id.unwrap();
    let node = build.nodes.get(hooks_key).unwrap().get(&id).unwrap();
    assert!(
        node.src.starts_with("1039:"),
        "node src should start at 1039, got: {}",
        node.src
    );
}

#[test]
fn test_byte_to_id_returns_none_for_unknown_path() {
    let build = load_cached_build();
    assert!(references::byte_to_id(&build.nodes, "nonexistent/File.sol", 0).is_none());
}

#[test]
fn test_byte_to_id_returns_none_for_invalid_offset() {
    let build = load_cached_build();
    let hooks_key = find_key(&build, "src/libraries/Hooks.sol");
    assert!(references::byte_to_id(&build.nodes, hooks_key, 999999).is_none());
}

// =============================================================================
// Cross-file reference resolution tests
// =============================================================================

#[test]
fn test_resolve_hooks_reference_from_pool_manager_node() {
    let build = load_cached_build();
    let pm_key = find_key(&build, "src/PoolManager.sol");
    let hooks_key = find_key(&build, "src/libraries/Hooks.sol");

    // In PoolManager.sol, node id=2 is an Identifier "Hooks" at src="70:5:5"
    // with referencedDeclaration=3530 (the library Hooks in Hooks.sol).
    let pm_nodes = build
        .nodes
        .get(pm_key)
        .expect("PoolManager.sol should be in nodes");

    // Verify node 2 exists and points to 3530
    let ref_node = pm_nodes.get(&NodeId(2)).expect("node 2 should exist");
    assert_eq!(ref_node.referenced_declaration, Some(NodeId(3530)));
    assert_eq!(ref_node.src, "70:5:5");

    // Resolve the definition: find node 3530 across all files
    let mut def_location = None;
    for (file_path, file_nodes) in &build.nodes {
        if let Some(def_node) = file_nodes.get(&NodeId(3530)) {
            let parts: Vec<&str> = def_node.src.split(':').collect();
            if parts.len() == 3 {
                let byte_offset: usize = parts[0].parse().unwrap();
                def_location = Some((file_path.clone(), byte_offset));
            }
            break;
        }
    }

    let (def_path, def_offset) = def_location.expect("should find definition node 3530");
    assert_eq!(def_path.as_str(), hooks_key);
    assert_eq!(def_offset, 1039);
}

#[test]
fn test_cross_file_scan_finds_pool_manager_references_to_hooks() {
    let build = load_cached_build();
    let pm_key = find_key(&build, "src/PoolManager.sol");
    let hooks_key = find_key(&build, "src/libraries/Hooks.sol");

    let target_id = references::byte_to_id(&build.nodes, hooks_key, 1039)
        .expect("byte_to_id should resolve Hooks definition");

    let hooks_nodes = build.nodes.get(hooks_key).unwrap();
    let node_info = hooks_nodes.get(&target_id).unwrap();
    let final_target = node_info.referenced_declaration.unwrap_or(target_id);

    // Scan all files for references
    let mut refs_by_file: std::collections::HashMap<String, Vec<NodeId>> =
        std::collections::HashMap::new();
    for (file_path, file_nodes) in &build.nodes {
        for (id, info) in file_nodes {
            if info.referenced_declaration == Some(final_target) {
                refs_by_file
                    .entry(file_path.to_string())
                    .or_default()
                    .push(*id);
            }
        }
    }

    // PoolManager.sol should have at least 3 references (ids 2, 72, 255)
    let pm_refs = refs_by_file
        .get(pm_key)
        .expect("PoolManager.sol should have references to Hooks");
    assert!(
        pm_refs.len() >= 3,
        "expected >= 3 PoolManager refs, got {}: {:?}",
        pm_refs.len(),
        pm_refs
    );

    // Known node IDs from the AST
    assert!(
        pm_refs.contains(&NodeId(2)),
        "should contain Identifier at byte 70"
    );
    assert!(
        pm_refs.contains(&NodeId(72)),
        "should contain IdentifierPath at byte 4953"
    );
    assert!(
        pm_refs.contains(&NodeId(255)),
        "should contain Identifier at byte 6804"
    );
}

#[test]
fn test_cross_file_scan_finds_references_across_multiple_files() {
    let build = load_cached_build();
    let pm_key = find_key(&build, "src/PoolManager.sol");
    let hooks_key = find_key(&build, "src/libraries/Hooks.sol");

    let target_id = references::byte_to_id(&build.nodes, hooks_key, 1039)
        .expect("byte_to_id should resolve Hooks definition");
    let hooks_nodes = build.nodes.get(hooks_key).unwrap();
    let node_info = hooks_nodes.get(&target_id).unwrap();
    let final_target = node_info.referenced_declaration.unwrap_or(target_id);

    // Collect all files that have references to Hooks
    let mut ref_files = HashSet::new();
    for (file_path, file_nodes) in &build.nodes {
        let tmp = file_nodes.iter();
        for (_id, info) in tmp {
            if info.referenced_declaration == Some(final_target) {
                ref_files.insert(file_path.clone());
            }
        }
    }

    assert!(
        ref_files.contains(pm_key),
        "PoolManager.sol should reference Hooks"
    );
    assert!(
        ref_files.contains(hooks_key),
        "Hooks.sol should have self-references"
    );
}

#[test]
fn test_end_to_end_cross_file_flow() {
    // Full flow using only AST data:
    // 1. Start at a usage node in PoolManager.sol
    // 2. Follow referencedDeclaration to find the definition
    // 3. Extract the definition's stable identity (abs_path, byte_offset)
    // 4. Re-resolve that identity via byte_to_id in the same (or another) build
    // 5. Scan for all references to the re-resolved target

    let build = load_cached_build();
    let pm_key = find_key(&build, "src/PoolManager.sol");
    let hooks_key = find_key(&build, "src/libraries/Hooks.sol");

    // Step 1-2: Start at node 2 (Hooks usage in PoolManager.sol), follow to definition
    let pm_nodes = build.nodes.get(pm_key).unwrap();
    let usage_node = pm_nodes.get(&NodeId(2)).unwrap();
    let def_id = usage_node
        .referenced_declaration
        .expect("usage should have referencedDeclaration");

    // Step 3: Find definition node, extract stable identity
    let mut def_abs_path = String::new();
    let mut def_byte_offset = 0usize;
    for (file_path, file_nodes) in &build.nodes {
        if let Some(def_node) = file_nodes.get(&def_id) {
            let parts: Vec<&str> = def_node.src.split(':').collect();
            def_abs_path = file_path.to_string();
            def_byte_offset = parts[0].parse().unwrap();
            break;
        }
    }
    assert_eq!(def_abs_path, hooks_key);
    assert_eq!(def_byte_offset, 1039);

    // Step 4: Re-resolve via byte_to_id (simulates cross-build bridging)
    let re_resolved_id = references::byte_to_id(&build.nodes, &def_abs_path, def_byte_offset)
        .expect("byte_to_id should re-resolve the definition");
    let re_node = build
        .nodes
        .get(def_abs_path.as_str())
        .unwrap()
        .get(&re_resolved_id)
        .unwrap();
    let final_target = re_node.referenced_declaration.unwrap_or(re_resolved_id);

    // Step 5: Scan for references
    let mut total_refs = 0;
    let mut ref_files = HashSet::new();
    for (file_path, file_nodes) in &build.nodes {
        let tmp = file_nodes.iter();
        for (_id, info) in tmp {
            if info.referenced_declaration == Some(final_target) {
                total_refs += 1;
                ref_files.insert(file_path.clone());
            }
        }
    }

    assert!(
        total_refs >= 4,
        "should find >= 4 total references, got {}",
        total_refs
    );
    assert!(ref_files.contains(pm_key));
    assert!(ref_files.contains(hooks_key));
}
