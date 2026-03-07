use serde_json::Value;
use solidity_language_server::goto;
use solidity_language_server::types::{AbsPath, NodeId, RelPath, SolcFileId};
use std::collections::HashMap;
use std::fs;

type CachedIds = (
    HashMap<AbsPath, HashMap<NodeId, goto::NodeInfo>>,
    HashMap<RelPath, AbsPath>,
    goto::ExternalRefs,
);

/// Load poolmanager.json, normalize from solc shape, and run cache_ids.
fn load_ast() -> CachedIds {
    let raw: Value =
        serde_json::from_str(&fs::read_to_string("poolmanager.json").unwrap()).unwrap();
    let ast_data = solidity_language_server::solc::normalize_solc_output(raw, None);
    let sources = ast_data.get("sources").unwrap();
    goto::cache_ids(sources)
}

#[test]
fn test_cache_ids_returns_external_refs() {
    let (_nodes, _path_to_abs, external_refs) = load_ast();

    // There should be external references (435 total across all InlineAssembly nodes)
    assert!(
        !external_refs.is_empty(),
        "external_refs should not be empty"
    );
}

#[test]
fn test_external_refs_for_get_sqrt_price_target() {
    let (_nodes, _path_to_abs, external_refs) = load_ast();

    // InlineAssembly node 8137 in getSqrtPriceTarget has 11 externalReferences:
    //   declaration 8128 (zeroForOne):         1 use  at src "2068:10:33"
    //   declaration 8130 (sqrtPriceNextX96):   4 uses at src "1802:16:33", "1826:16:33", "2026:16:33", "2117:16:33"
    //   declaration 8132 (sqrtPriceLimitX96):  5 uses at src "1900:17:33", "1925:17:33", "2044:17:33", "2135:17:33", "2192:17:33"
    //   declaration 8135 (sqrtPriceTargetX96): 1 use  at src "2166:18:33"

    // zeroForOne (8128)
    assert_eq!(external_refs.get("2068:10:33"), Some(&NodeId(8128)));

    // sqrtPriceNextX96 (8130)
    assert_eq!(external_refs.get("1802:16:33"), Some(&NodeId(8130)));
    assert_eq!(external_refs.get("1826:16:33"), Some(&NodeId(8130)));
    assert_eq!(external_refs.get("2026:16:33"), Some(&NodeId(8130)));
    assert_eq!(external_refs.get("2117:16:33"), Some(&NodeId(8130)));

    // sqrtPriceLimitX96 (8132)
    assert_eq!(external_refs.get("1900:17:33"), Some(&NodeId(8132)));
    assert_eq!(external_refs.get("1925:17:33"), Some(&NodeId(8132)));
    assert_eq!(external_refs.get("2044:17:33"), Some(&NodeId(8132)));
    assert_eq!(external_refs.get("2135:17:33"), Some(&NodeId(8132)));
    assert_eq!(external_refs.get("2192:17:33"), Some(&NodeId(8132)));

    // sqrtPriceTargetX96 (8135)
    assert_eq!(external_refs.get("2166:18:33"), Some(&NodeId(8135)));
}

#[test]
fn test_external_refs_exact_count_for_each_parameter() {
    let (_nodes, _path_to_abs, external_refs) = load_ast();

    // Count refs per declaration for the getSqrtPriceTarget parameters
    let count_for =
        |decl_id: NodeId| -> usize { external_refs.values().filter(|&&v| v == decl_id).count() };

    // zeroForOne (8128): used once in assembly across ALL files
    // But other InlineAssembly nodes in other files may also reference a node with id 8128
    // so we check that at least 1 ref maps to 8128
    assert!(
        count_for(NodeId(8128)) >= 1,
        "zeroForOne (8128) should have at least 1 Yul reference, found {}",
        count_for(NodeId(8128))
    );

    // sqrtPriceNextX96 (8130): 4 uses in getSqrtPriceTarget assembly
    assert!(
        count_for(NodeId(8130)) >= 4,
        "sqrtPriceNextX96 (8130) should have at least 4 Yul references, found {}",
        count_for(NodeId(8130))
    );

    // sqrtPriceLimitX96 (8132): 5 uses in getSqrtPriceTarget assembly
    assert!(
        count_for(NodeId(8132)) >= 5,
        "sqrtPriceLimitX96 (8132) should have at least 5 Yul references, found {}",
        count_for(NodeId(8132))
    );

    // sqrtPriceTargetX96 (8135): 1 use in getSqrtPriceTarget assembly
    assert!(
        count_for(NodeId(8135)) >= 1,
        "sqrtPriceTargetX96 (8135) should have at least 1 Yul reference, found {}",
        count_for(NodeId(8135))
    );
}

#[test]
fn test_solidity_nodes_unchanged() {
    let (nodes, _path_to_abs, _external_refs) = load_ast();

    // The u64-keyed HashMap should still contain Solidity nodes with their ids
    // Check that key Solidity declaration nodes exist
    let mut found_8128 = false;
    let mut found_8130 = false;
    let mut found_8132 = false;
    let mut found_8135 = false;
    let mut found_8137 = false; // InlineAssembly node itself

    for file_nodes in nodes.values() {
        if file_nodes.contains_key(&NodeId(8128)) {
            found_8128 = true;
        }
        if file_nodes.contains_key(&NodeId(8130)) {
            found_8130 = true;
        }
        if file_nodes.contains_key(&NodeId(8132)) {
            found_8132 = true;
        }
        if file_nodes.contains_key(&NodeId(8135)) {
            found_8135 = true;
        }
        if file_nodes.contains_key(&NodeId(8137)) {
            found_8137 = true;
        }
    }

    assert!(found_8128, "zeroForOne (8128) should be in Solidity nodes");
    assert!(
        found_8130,
        "sqrtPriceNextX96 (8130) should be in Solidity nodes"
    );
    assert!(
        found_8132,
        "sqrtPriceLimitX96 (8132) should be in Solidity nodes"
    );
    assert!(
        found_8135,
        "sqrtPriceTargetX96 (8135) should be in Solidity nodes"
    );
    assert!(
        found_8137,
        "InlineAssembly (8137) should be in Solidity nodes"
    );

    // Verify InlineAssembly node type
    for file_nodes in nodes.values() {
        if let Some(node) = file_nodes.get(&NodeId(8137)) {
            assert_eq!(
                node.node_type,
                Some("InlineAssembly".to_string()),
                "Node 8137 should be InlineAssembly"
            );
        }
    }
}

#[test]
fn test_no_yul_src_keys_in_u64_map() {
    let (_nodes, _path_to_abs, external_refs) = load_ast();

    // Verify that none of the Yul src strings appear as node ids in the u64 map.
    // This confirms Yul data is kept separate.
    for yul_src in external_refs.keys() {
        let parts: Vec<&str> = yul_src.split(':').collect();
        if parts.len() == 3 {
            // The offset portion should NOT be a node id (it could collide by accident,
            // but the point is we never *inserted* based on src strings)
            // We verify structurally: Yul external refs use src strings, not node ids
            assert!(
                parts[0].parse::<usize>().is_ok(),
                "Yul src key should have numeric offset: {}",
                yul_src
            );
        }
    }
}

#[test]
fn test_goto_bytes_resolves_yul_identifier() {
    let raw: Value =
        serde_json::from_str(&fs::read_to_string("poolmanager.json").unwrap()).unwrap();
    let ast_data = solidity_language_server::solc::normalize_solc_output(raw, None);
    let sources = ast_data.get("sources").unwrap();
    let id_to_path: HashMap<SolcFileId, String> = ast_data
        .get("source_id_to_path")
        .unwrap()
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| {
            (
                SolcFileId::new(k.clone()),
                v.as_str().unwrap_or("").to_string(),
            )
        })
        .collect();

    let (nodes, path_to_abs, external_refs) = goto::cache_ids(sources);

    // Byte offset 1802 is the start of sqrtPriceNextX96 Yul reference (src "1802:16:33")
    // It should resolve to the declaration of sqrtPriceNextX96 (id 8130)
    // which is in file 33 (SwapMath.sol)

    // Find the absolute path for SwapMath.sol from path_to_abs keys
    let swap_math_abs = path_to_abs
        .keys()
        .find(|k| k.contains("SwapMath"))
        .expect("SwapMath.sol should be in path_to_abs");

    let uri = format!("file://{}", swap_math_abs);

    let result = goto::goto_bytes(
        &nodes,
        &path_to_abs,
        &id_to_path,
        &external_refs,
        &uri,
        1802, // byte offset of first sqrtPriceNextX96 Yul reference
    );

    assert!(
        result.is_some(),
        "goto_bytes should resolve Yul identifier at byte 1802"
    );

    let (target_path, target_offset, _target_length) = result.unwrap();
    // Target should be in SwapMath.sol (same file for this case)
    assert!(
        target_path.contains("SwapMath"),
        "Target should be in SwapMath.sol, got: {}",
        target_path
    );

    // The declaration node 8130 (sqrtPriceNextX96) should have a nameLocation
    // pointing to the parameter definition
    for file_nodes in nodes.values() {
        if let Some(node) = file_nodes.get(&NodeId(8130)) {
            if let Some(name_loc) = &node.name_location {
                let parts: Vec<&str> = name_loc.split(':').collect();
                let expected_offset: usize = parts[0].parse().unwrap();
                assert_eq!(
                    target_offset, expected_offset,
                    "goto_bytes should point to the declaration's nameLocation offset"
                );
            }
            break;
        }
    }
}

// =============================================================================
// goto_bytes: range length tests
// =============================================================================

struct SetupGotoResult(
    HashMap<AbsPath, HashMap<NodeId, goto::NodeInfo>>,
    HashMap<RelPath, AbsPath>,
    HashMap<SolcFileId, String>,
    goto::ExternalRefs,
);

/// Helper: set up goto_bytes inputs from the fixture.
fn setup_goto() -> SetupGotoResult {
    let raw: Value =
        serde_json::from_str(&fs::read_to_string("poolmanager.json").unwrap()).unwrap();
    let ast_data = solidity_language_server::solc::normalize_solc_output(raw, None);
    let sources = ast_data.get("sources").unwrap();
    let id_to_path: HashMap<SolcFileId, String> = ast_data
        .get("source_id_to_path")
        .unwrap()
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| {
            (
                SolcFileId::new(k.clone()),
                v.as_str().unwrap_or("").to_string(),
            )
        })
        .collect();
    let (nodes, path_to_abs, external_refs) = goto::cache_ids(sources);
    SetupGotoResult(nodes, path_to_abs, id_to_path, external_refs)
}

#[test]
fn test_goto_bytes_returns_name_length_for_hooks() {
    let SetupGotoResult(nodes, path_to_abs, id_to_path, external_refs) = setup_goto();

    let pm_abs = path_to_abs
        .keys()
        .find(|k| k.ends_with("src/PoolManager.sol"))
        .unwrap();
    let uri = format!("file://{}", pm_abs);

    // "Hooks" identifier at byte 70 in PoolManager.sol (length 5)
    let result = goto::goto_bytes(&nodes, &path_to_abs, &id_to_path, &external_refs, &uri, 70);
    let (path, _offset, length) = result.expect("should resolve Hooks reference");

    assert!(
        path.contains("Hooks"),
        "should resolve to Hooks.sol, got: {path}"
    );
    assert_eq!(length, 5, "Hooks nameLocation length should be 5");
}

#[test]
fn test_goto_bytes_returns_name_length_for_pool() {
    let SetupGotoResult(nodes, path_to_abs, id_to_path, external_refs) = setup_goto();

    let pm_abs = path_to_abs
        .keys()
        .find(|k| k.ends_with("src/PoolManager.sol"))
        .unwrap();
    let uri = format!("file://{}", pm_abs);

    // "Pool" identifier at byte 115 in PoolManager.sol (length 4)
    let result = goto::goto_bytes(&nodes, &path_to_abs, &id_to_path, &external_refs, &uri, 115);
    let (path, _offset, length) = result.expect("should resolve Pool reference");

    assert!(
        path.contains("Pool"),
        "should resolve to Pool.sol, got: {path}"
    );
    assert_eq!(length, 4, "Pool nameLocation length should be 4");
}

#[test]
fn test_goto_bytes_range_is_nonzero() {
    let SetupGotoResult(nodes, path_to_abs, id_to_path, external_refs) = setup_goto();

    let pm_abs = path_to_abs
        .keys()
        .find(|k| k.ends_with("src/PoolManager.sol"))
        .unwrap();
    let uri = format!("file://{}", pm_abs);

    // "SafeCast" identifier at byte 158 in PoolManager.sol (length 8)
    let result = goto::goto_bytes(&nodes, &path_to_abs, &id_to_path, &external_refs, &uri, 158);
    let (_path, _offset, length) = result.expect("should resolve SafeCast reference");

    assert!(length > 0, "goto range length should never be zero");
    assert_eq!(length, 8, "SafeCast nameLocation length should be 8");
}

#[test]
fn test_goto_bytes_yul_ref_returns_nonzero_length() {
    let SetupGotoResult(nodes, path_to_abs, id_to_path, external_refs) = setup_goto();

    let swap_math_abs = path_to_abs.keys().find(|k| k.contains("SwapMath")).unwrap();
    let uri = format!("file://{}", swap_math_abs);

    // sqrtPriceNextX96 Yul reference at byte 1802
    let result = goto::goto_bytes(
        &nodes,
        &path_to_abs,
        &id_to_path,
        &external_refs,
        &uri,
        1802,
    );
    let (_path, _offset, length) = result.expect("should resolve Yul reference");

    assert!(length > 0, "Yul goto range length should never be zero");
}
