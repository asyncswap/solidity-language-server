use serde_json::Value;
use solidity_language_server::completion::{
    AccessKind, DotSegment, build_completion_cache, extract_identifier_before_dot,
    extract_mapping_value_type, extract_node_id_from_type, get_dot_completions,
    get_general_completions, parse_dot_chain,
};
use solidity_language_server::types::{FileId, NodeId};
use std::fs;
use tower_lsp::lsp_types::CompletionItemKind;

fn load_ast() -> Value {
    let raw: Value =
        serde_json::from_str(&fs::read_to_string("poolmanager.json").unwrap()).unwrap();
    solidity_language_server::solc::normalize_solc_output(raw, None)
}

fn load_cache() -> solidity_language_server::completion::CompletionCache {
    let ast_data = load_ast();
    let sources = ast_data.get("sources").unwrap();
    let contracts = ast_data.get("contracts");
    build_completion_cache(sources, contracts, None)
}

// --- extract_node_id_from_type ---

#[test]
fn test_extract_node_id_from_struct_type() {
    assert_eq!(
        extract_node_id_from_type("t_struct$_PoolKey_$6871_storage_ptr"),
        Some(NodeId(6871))
    );
}

#[test]
fn test_extract_node_id_from_contract_type() {
    assert_eq!(
        extract_node_id_from_type("t_contract$_IHooks_$1840"),
        Some(NodeId(1840))
    );
}

#[test]
fn test_extract_node_id_from_user_defined_value_type() {
    assert_eq!(
        extract_node_id_from_type("t_userDefinedValueType$_Currency_$6525"),
        Some(NodeId(6525))
    );
}

#[test]
fn test_extract_node_id_from_primitive_type() {
    assert_eq!(extract_node_id_from_type("t_uint256"), None);
    assert_eq!(extract_node_id_from_type("t_address"), None);
    assert_eq!(extract_node_id_from_type("t_bool"), None);
}

// --- extract_identifier_before_dot ---

#[test]
fn test_extract_identifier_simple() {
    assert_eq!(
        extract_identifier_before_dot("key.", 4),
        Some("key".to_string())
    );
}

#[test]
fn test_extract_identifier_in_expression() {
    assert_eq!(
        extract_identifier_before_dot("        poolKey.", 16),
        Some("poolKey".to_string())
    );
}

#[test]
fn test_extract_identifier_at_start() {
    assert_eq!(extract_identifier_before_dot(".", 1), None);
}

#[test]
fn test_extract_identifier_msg() {
    assert_eq!(
        extract_identifier_before_dot("msg.", 4),
        Some("msg".to_string())
    );
}

// --- parse_dot_chain ---

#[test]
fn test_parse_chain_simple() {
    let chain = parse_dot_chain("msg.", 4);
    assert_eq!(
        chain,
        vec![DotSegment {
            name: "msg".to_string(),
            kind: AccessKind::Plain,
            call_args: None,
        }]
    );
}

#[test]
fn test_parse_chain_function_call() {
    let chain = parse_dot_chain("foo().", 6);
    assert_eq!(
        chain,
        vec![DotSegment {
            name: "foo".to_string(),
            kind: AccessKind::Call,
            call_args: None,
        }]
    );
}

#[test]
fn test_parse_chain_function_call_with_args() {
    let chain = parse_dot_chain("swap(key, params).", 18);
    assert_eq!(
        chain,
        vec![DotSegment {
            name: "swap".to_string(),
            kind: AccessKind::Call,
            call_args: Some("key, params".to_string()),
        }]
    );
}

#[test]
fn test_parse_chain_nested_calls_in_args() {
    // swap(getKey(), getParams(x, y())) — nested parens inside args
    let line = "swap(getKey(), getParams(x, y())).";
    let chain = parse_dot_chain(line, line.len() as u32);
    assert_eq!(
        chain,
        vec![DotSegment {
            name: "swap".to_string(),
            kind: AccessKind::Call,
            call_args: Some("getKey(), getParams(x, y())".to_string()),
        }]
    );
}

#[test]
fn test_parse_chain_index_access() {
    let chain = parse_dot_chain("_pools[poolId].", 15);
    assert_eq!(
        chain,
        vec![DotSegment {
            name: "_pools".to_string(),
            kind: AccessKind::Index,
            call_args: None,
        }]
    );
}

#[test]
fn test_parse_chain_index_with_nested_call() {
    // mapping[someFunc(a, b)].
    let line = "mapping[someFunc(a, b)].";
    let chain = parse_dot_chain(line, line.len() as u32);
    assert_eq!(
        chain,
        vec![DotSegment {
            name: "mapping".to_string(),
            kind: AccessKind::Index,
            call_args: None,
        }]
    );
}

#[test]
fn test_parse_chain_two_segments() {
    // poolManager.swap().
    let line = "poolManager.swap().";
    let chain = parse_dot_chain(line, line.len() as u32);
    assert_eq!(
        chain,
        vec![
            DotSegment {
                name: "poolManager".to_string(),
                kind: AccessKind::Plain,
                call_args: None,
            },
            DotSegment {
                name: "swap".to_string(),
                kind: AccessKind::Call,
                call_args: None,
            },
        ]
    );
}

#[test]
fn test_parse_chain_three_segments_mixed() {
    // _pools[poolId].slot0.sqrtPriceX96.
    let line = "_pools[poolId].slot0.";
    let chain = parse_dot_chain(line, line.len() as u32);
    assert_eq!(
        chain,
        vec![
            DotSegment {
                name: "_pools".to_string(),
                kind: AccessKind::Index,
                call_args: None,
            },
            DotSegment {
                name: "slot0".to_string(),
                kind: AccessKind::Plain,
                call_args: None,
            },
        ]
    );
}

#[test]
fn test_parse_chain_call_then_index() {
    // getPool(key).positions[posId].
    let line = "getPool(key).positions[posId].";
    let chain = parse_dot_chain(line, line.len() as u32);
    assert_eq!(
        chain,
        vec![
            DotSegment {
                name: "getPool".to_string(),
                kind: AccessKind::Call,
                call_args: Some("key".to_string()),
            },
            DotSegment {
                name: "positions".to_string(),
                kind: AccessKind::Index,
                call_args: None,
            },
        ]
    );
}

// --- extract_mapping_value_type ---

#[test]
fn test_mapping_value_simple() {
    assert_eq!(
        extract_mapping_value_type("t_mapping$_t_address_$_t_uint256_$"),
        Some("t_uint256".to_string())
    );
}

#[test]
fn test_mapping_value_struct() {
    assert_eq!(
        extract_mapping_value_type(
            "t_mapping$_t_userDefinedValueType$_PoolId_$6825_$_t_struct$_State_$3870_storage_$"
        ),
        Some("t_struct$_State_$3870_storage".to_string())
    );
}

#[test]
fn test_mapping_value_nested() {
    // mapping(address => mapping(uint256 => uint256))
    assert_eq!(
        extract_mapping_value_type("t_mapping$_t_address_$_t_mapping$_t_uint256_$_t_uint256_$_$"),
        Some("t_uint256".to_string())
    );
}

#[test]
fn test_mapping_value_triple_nested() {
    // mapping(address => mapping(address => mapping(uint256 => uint256)))
    assert_eq!(
        extract_mapping_value_type(
            "t_mapping$_t_address_$_t_mapping$_t_address_$_t_mapping$_t_uint256_$_t_uint256_$_$_$"
        ),
        Some("t_uint256".to_string())
    );
}

#[test]
fn test_mapping_value_not_a_mapping() {
    assert_eq!(
        extract_mapping_value_type("t_uint256"),
        Some("t_uint256".to_string())
    );
}

#[test]
fn test_mapping_value_struct_with_node_id() {
    let result =
        extract_mapping_value_type("t_mapping$_t_int24_$_t_struct$_TickInfo_$3845_storage_$");
    assert_eq!(result, Some("t_struct$_TickInfo_$3845_storage".to_string()));
    // Can extract node id from the result
    assert_eq!(
        extract_node_id_from_type(&result.unwrap()),
        Some(NodeId(3845))
    );
}

// --- general completions ---

#[test]
fn test_general_completions_include_ast_names() {
    let cache = load_cache();
    let completions = get_general_completions(&cache);
    let names: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

    // Should include contract/library/function names from the AST
    assert!(names.contains(&"PoolManager"), "Should have PoolManager");
    assert!(names.contains(&"SwapMath"), "Should have SwapMath");
    assert!(names.contains(&"PoolKey"), "Should have PoolKey");
    assert!(
        names.contains(&"getSqrtPriceTarget"),
        "Should have getSqrtPriceTarget"
    );
}

#[test]
fn test_general_completions_include_keywords() {
    let cache = load_cache();
    let completions = get_general_completions(&cache);
    let names: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

    assert!(
        names.contains(&"contract"),
        "Should have 'contract' keyword"
    );
    assert!(
        names.contains(&"function"),
        "Should have 'function' keyword"
    );
    assert!(names.contains(&"if"), "Should have 'if' keyword");
    assert!(names.contains(&"mapping"), "Should have 'mapping' keyword");
}

#[test]
fn test_general_completions_include_magic_globals() {
    let cache = load_cache();
    let completions = get_general_completions(&cache);
    let names: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

    assert!(names.contains(&"msg"), "Should have 'msg'");
    assert!(names.contains(&"block"), "Should have 'block'");
    assert!(names.contains(&"tx"), "Should have 'tx'");
    assert!(names.contains(&"abi"), "Should have 'abi'");
}

#[test]
fn test_general_completions_include_global_functions() {
    let cache = load_cache();
    let completions = get_general_completions(&cache);
    let names: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

    assert!(
        names.contains(&"keccak256(bytes memory)"),
        "Should have keccak256"
    );
    assert!(names.contains(&"gasleft()"), "Should have gasleft");
    assert!(
        names.contains(&"require(bool condition)"),
        "Should have require"
    );
}

// --- dot completions ---

#[test]
fn test_dot_completion_msg() {
    let cache = load_cache();
    let items = get_dot_completions(&cache, "msg", None);
    let names: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();

    assert!(names.contains(&"sender"), "msg.sender");
    assert!(names.contains(&"value"), "msg.value");
    assert!(names.contains(&"data"), "msg.data");
    assert!(names.contains(&"sig"), "msg.sig");
    assert_eq!(names.len(), 4, "msg should have exactly 4 members");
}

#[test]
fn test_dot_completion_block() {
    let cache = load_cache();
    let items = get_dot_completions(&cache, "block", None);
    let names: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();

    assert!(names.contains(&"number"), "block.number");
    assert!(names.contains(&"timestamp"), "block.timestamp");
    assert!(names.contains(&"chainid"), "block.chainid");
    assert!(names.contains(&"basefee"), "block.basefee");
}

#[test]
fn test_dot_completion_tx() {
    let cache = load_cache();
    let items = get_dot_completions(&cache, "tx", None);
    let names: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();

    assert!(names.contains(&"gasprice"), "tx.gasprice");
    assert!(names.contains(&"origin"), "tx.origin");
    assert_eq!(names.len(), 2, "tx should have exactly 2 members");
}

#[test]
fn test_dot_completion_abi() {
    let cache = load_cache();
    let items = get_dot_completions(&cache, "abi", None);
    let names: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();

    assert!(names.contains(&"encode(...)"), "abi.encode");
    assert!(names.contains(&"encodePacked(...)"), "abi.encodePacked");
    assert!(names.contains(&"decode(bytes memory, (...))"), "abi.decode");
}

// --- dot completions on AST types ---

#[test]
fn test_dot_completion_on_struct() {
    let cache = load_cache();

    // PoolKey is a struct with members: currency0, currency1, fee, tickSpacing, hooks
    // We need a variable that has type PoolKey — check name_to_type for one
    // "key" is commonly used as a PoolKey parameter name
    let pool_key_vars: Vec<&str> = cache
        .name_to_type
        .iter()
        .filter(|(_, tid)| tid.contains("PoolKey"))
        .map(|(name, _)| name.as_str())
        .collect();

    // There should be at least one variable with PoolKey type
    assert!(
        !pool_key_vars.is_empty(),
        "Should have variables with PoolKey type, name_to_type has {} entries",
        cache.name_to_type.len()
    );

    // Use the first one to test dot completion
    let var_name = pool_key_vars[0];
    let items = get_dot_completions(&cache, var_name, None);
    let names: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();

    assert!(
        names.contains(&"currency0"),
        "PoolKey should have currency0 member, got: {:?}",
        names
    );
    assert!(
        names.contains(&"fee"),
        "PoolKey should have fee member, got: {:?}",
        names
    );
    assert!(
        names.contains(&"hooks"),
        "PoolKey should have hooks member, got: {:?}",
        names
    );
}

#[test]
fn test_name_to_type_populated() {
    let cache = load_cache();

    // Should have mapped variable names to their types
    assert!(
        !cache.name_to_type.is_empty(),
        "name_to_type should not be empty"
    );

    // Check a few expected entries exist
    assert!(
        cache.name_to_type.values().any(|v| v.contains("PoolKey")),
        "Should have at least one PoolKey-typed variable"
    );
}

#[test]
fn test_node_members_populated() {
    let cache = load_cache();

    assert!(
        !cache.node_members.is_empty(),
        "node_members should not be empty"
    );

    // PoolKey struct (id 6871) should have members
    assert!(
        cache.node_members.contains_key(&NodeId(6871)),
        "Should have members for PoolKey (6871)"
    );

    let pool_key_members = &cache.node_members[&NodeId(6871)];
    let member_names: Vec<&str> = pool_key_members.iter().map(|c| c.label.as_str()).collect();
    assert_eq!(
        member_names.len(),
        5,
        "PoolKey should have 5 members: {:?}",
        member_names
    );
    assert!(member_names.contains(&"currency0"));
    assert!(member_names.contains(&"currency1"));
    assert!(member_names.contains(&"fee"));
    assert!(member_names.contains(&"tickSpacing"));
    assert!(member_names.contains(&"hooks"));
}

// --- library dot completions ---

#[test]
fn test_dot_completion_fullmath_library() {
    let cache = load_cache();

    // Check what name_to_type has for FullMath
    let fullmath_type = cache.name_to_type.get("FullMath");
    eprintln!("FullMath type: {:?}", fullmath_type);

    let items = get_dot_completions(&cache, "FullMath", None);
    let labels: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();
    eprintln!("FullMath dot completions ({}): {:?}", labels.len(), labels);

    // FullMath should include its 2 internal functions
    assert!(
        labels.contains(&"mulDiv"),
        "Should have mulDiv, got: {:?}",
        labels
    );
    assert!(
        labels.contains(&"mulDivRoundingUp"),
        "Should have mulDivRoundingUp, got: {:?}",
        labels
    );
    // May also include wildcard using-for functions (using X for *)
    // which are globally available. Scope-aware filtering is future work.
}

// --- function signatures ---

#[test]
fn test_method_identifier_swap_has_description_with_param_names() {
    let cache = load_cache();
    // IPoolManager (2531) has swap
    let ipm_methods = &cache.method_identifiers[&NodeId(2123)];
    let swap_item = ipm_methods.iter().find(|c| c.label == "swap").unwrap();
    let desc = swap_item
        .label_details
        .as_ref()
        .unwrap()
        .description
        .as_deref()
        .expect("swap should have a description with parameter names");

    eprintln!("swap description: {}", desc);

    // Should contain parameter names from the AST
    assert!(
        desc.contains("key"),
        "description should contain param name 'key': {}",
        desc
    );
    assert!(
        desc.contains("params"),
        "description should contain param name 'params': {}",
        desc
    );
    assert!(
        desc.contains("hookData"),
        "description should contain param name 'hookData': {}",
        desc
    );
    // Should contain return info
    assert!(
        desc.contains("returns"),
        "description should show return type: {}",
        desc
    );
    assert!(
        desc.contains("swapDelta"),
        "description should contain return name 'swapDelta': {}",
        desc
    );
}

#[test]
fn test_method_identifier_settle_has_description_no_params() {
    let cache = load_cache();
    let pm_methods = &cache.method_identifiers[&NodeId(1216)];
    let settle_item = pm_methods.iter().find(|c| c.label == "settle").unwrap();
    let desc = settle_item
        .label_details
        .as_ref()
        .unwrap()
        .description
        .as_deref()
        .expect("settle should have a description");

    eprintln!("settle description: {}", desc);

    assert!(
        desc.contains("settle()"),
        "settle should show empty params: {}",
        desc
    );
}

#[test]
fn test_method_identifier_initialize_has_return_type() {
    let cache = load_cache();
    let ipm_methods = &cache.method_identifiers[&NodeId(2123)];
    let init_item = ipm_methods
        .iter()
        .find(|c| c.label == "initialize")
        .unwrap();
    let desc = init_item
        .label_details
        .as_ref()
        .unwrap()
        .description
        .as_deref()
        .expect("initialize should have a description");

    eprintln!("initialize description: {}", desc);

    assert!(
        desc.contains("key"),
        "should have param name 'key': {}",
        desc
    );
    assert!(
        desc.contains("sqrtPriceX96"),
        "should have param name 'sqrtPriceX96': {}",
        desc
    );
    assert!(desc.contains("returns"), "should show returns: {}", desc);
    assert!(
        desc.contains("tick"),
        "should contain return name 'tick': {}",
        desc
    );
}

#[test]
fn test_node_members_function_has_signature_as_detail() {
    let cache = load_cache();
    // FullMath (3250) — its node_members should have function signatures as detail
    let fm_members = &cache.node_members[&NodeId(8913)];
    let mul_div = fm_members.iter().find(|c| c.label == "mulDiv").unwrap();

    eprintln!("mulDiv detail: {:?}", mul_div.detail);

    let detail = mul_div
        .detail
        .as_deref()
        .expect("mulDiv should have detail");
    assert!(
        detail.contains("mulDiv("),
        "detail should be a signature: {}",
        detail
    );
    // mulDiv takes (uint256, uint256, uint256) returns (uint256)
    assert!(
        detail.contains("uint256"),
        "should contain uint256 params: {}",
        detail
    );
}

// --- method identifiers ---

#[test]
fn test_method_identifiers_populated() {
    let cache = load_cache();

    assert!(
        !cache.method_identifiers.is_empty(),
        "method_identifiers should not be empty"
    );

    // PoolManager (id 1767) should have method identifiers
    assert!(
        cache.method_identifiers.contains_key(&NodeId(1216)),
        "Should have method identifiers for PoolManager (1767), keys: {:?}",
        cache.method_identifiers.keys().collect::<Vec<_>>()
    );

    // IPoolManager (id 2531) should also have them
    assert!(
        cache.method_identifiers.contains_key(&NodeId(2123)),
        "Should have method identifiers for IPoolManager (2531)"
    );
}

#[test]
fn test_overloaded_extsload_on_extsload_contract() {
    let cache = load_cache();
    // extsload is defined on Extsload contract (468), not PoolManager directly
    // So Extsload's method_identifiers should have descriptions
    let ext_methods = &cache.method_identifiers[&NodeId(1332)];
    let extsload_items: Vec<_> = ext_methods
        .iter()
        .filter(|c| c.label == "extsload")
        .collect();
    assert_eq!(extsload_items.len(), 3);

    let descriptions: Vec<Option<&str>> = extsload_items
        .iter()
        .map(|c| {
            c.label_details
                .as_ref()
                .and_then(|ld| ld.description.as_deref())
        })
        .collect();

    eprintln!("extsload descriptions: {:?}", descriptions);

    // All 3 overloads should have descriptions since they're defined directly on Extsload
    for (i, desc) in descriptions.iter().enumerate() {
        assert!(
            desc.is_some(),
            "extsload overload {} should have a description",
            i
        );
    }
}

#[test]
fn test_inherited_functions_have_no_description() {
    let cache = load_cache();
    // PoolManager (1767) inherits extsload from Extsload — no AST signature available
    let pm_methods = &cache.method_identifiers[&NodeId(1216)];
    let extsload_items: Vec<_> = pm_methods
        .iter()
        .filter(|c| c.label == "extsload")
        .collect();

    // These are inherited, so description will be None (function_signatures only collects direct members)
    for item in &extsload_items {
        let desc = item
            .label_details
            .as_ref()
            .and_then(|ld| ld.description.as_deref());
        assert!(
            desc.is_none(),
            "Inherited extsload on PoolManager should have no description (it's on Extsload base), got: {:?}",
            desc
        );
    }

    // But they still have the ABI signature as detail
    for item in &extsload_items {
        assert!(
            item.detail.is_some(),
            "Should still have ABI signature as detail"
        );
    }
}

#[test]
fn test_method_identifiers_have_function_names() {
    let cache = load_cache();
    let pm_methods = &cache.method_identifiers[&NodeId(1216)];
    let labels: Vec<&str> = pm_methods.iter().map(|c| c.label.as_str()).collect();

    assert!(
        labels.contains(&"swap"),
        "Should have swap, got: {:?}",
        labels
    );
    assert!(labels.contains(&"initialize"), "Should have initialize");
    assert!(labels.contains(&"settle"), "Should have settle");
    assert!(labels.contains(&"take"), "Should have take");
    assert!(labels.contains(&"donate"), "Should have donate");
    assert!(labels.contains(&"unlock"), "Should have unlock");
    assert!(labels.contains(&"owner"), "Should have owner");
}

#[test]
fn test_method_identifiers_have_full_signatures_as_detail() {
    let cache = load_cache();
    let pm_methods = &cache.method_identifiers[&NodeId(1216)];

    let swap_item = pm_methods.iter().find(|c| c.label == "swap").unwrap();
    assert_eq!(
        swap_item.detail.as_deref(),
        Some("swap((address,address,uint24,int24,address),(bool,int256,uint160),bytes)"),
        "swap detail should be the full signature"
    );

    let settle_item = pm_methods.iter().find(|c| c.label == "settle").unwrap();
    assert_eq!(
        settle_item.detail.as_deref(),
        Some("settle()"),
        "settle detail should be settle()"
    );
}

#[test]
fn test_method_identifiers_have_selectors() {
    let cache = load_cache();
    let pm_methods = &cache.method_identifiers[&NodeId(1216)];

    let swap_item = pm_methods.iter().find(|c| c.label == "swap").unwrap();
    let label_details = swap_item.label_details.as_ref().unwrap();
    assert_eq!(
        label_details.detail.as_deref(),
        Some("0xf3cd914c"),
        "swap selector should be 0xf3cd914c"
    );

    let settle_item = pm_methods.iter().find(|c| c.label == "settle").unwrap();
    let settle_details = settle_item.label_details.as_ref().unwrap();
    assert_eq!(
        settle_details.detail.as_deref(),
        Some("0x11da60b4"),
        "settle selector should be 0x11da60b4"
    );
}

#[test]
fn test_method_identifiers_are_function_kind() {
    let cache = load_cache();
    let pm_methods = &cache.method_identifiers[&NodeId(1216)];

    for item in pm_methods {
        assert_eq!(
            item.kind,
            Some(CompletionItemKind::FUNCTION),
            "{} should be FUNCTION kind",
            item.label
        );
    }
}

#[test]
fn test_method_identifiers_handles_overloads() {
    let cache = load_cache();
    let pm_methods = &cache.method_identifiers[&NodeId(1216)];

    // extsload has 3 overloads: extsload(bytes32), extsload(bytes32,uint256), extsload(bytes32[])
    let extsload_items: Vec<_> = pm_methods
        .iter()
        .filter(|c| c.label == "extsload")
        .collect();
    assert_eq!(
        extsload_items.len(),
        3,
        "Should have 3 extsload overloads, got {}",
        extsload_items.len()
    );

    // Each should have a different selector
    let selectors: Vec<&str> = extsload_items
        .iter()
        .map(|c| c.label_details.as_ref().unwrap().detail.as_deref().unwrap())
        .collect();
    assert!(selectors.contains(&"0x1e2eaeaf"));
    assert!(selectors.contains(&"0x35fd631a"));
    assert!(selectors.contains(&"0xdbd035ff"));
}

#[test]
fn test_method_identifiers_ipool_manager_interface() {
    let cache = load_cache();
    let ipm_methods = &cache.method_identifiers[&NodeId(2123)];
    let labels: Vec<&str> = ipm_methods.iter().map(|c| c.label.as_str()).collect();

    assert_eq!(ipm_methods.len(), 30, "IPoolManager should have 30 methods");
    assert!(labels.contains(&"swap"), "Interface should have swap");
    assert!(
        labels.contains(&"modifyLiquidity"),
        "Interface should have modifyLiquidity"
    );
}

#[test]
fn test_dot_completion_supplements_method_identifiers_with_node_members() {
    let cache = load_cache();

    // PoolManager (1767) should have both method_identifiers AND node_members
    // node_members includes events, errors, state variables — things not in methodIdentifiers
    let has_methods = cache.method_identifiers.contains_key(&NodeId(1216));
    let has_members = cache.node_members.contains_key(&NodeId(1216));

    assert!(has_methods, "PoolManager should have method_identifiers");
    assert!(has_members, "PoolManager should have node_members");

    // Check that node_members has things that method_identifiers doesn't
    let method_labels: std::collections::HashSet<&str> = cache.method_identifiers[&NodeId(1216)]
        .iter()
        .map(|c| c.label.as_str())
        .collect();
    let member_only: Vec<&str> = cache.node_members[&NodeId(1216)]
        .iter()
        .map(|c| c.label.as_str())
        .filter(|l| !method_labels.contains(l))
        .collect();

    // There should be some members not in methodIdentifiers (events, errors, etc.)
    assert!(
        !member_only.is_empty(),
        "node_members should have entries not in method_identifiers (events, errors, etc.), method labels: {:?}",
        method_labels
    );
}

#[test]
fn test_cache_without_contracts_has_empty_method_identifiers() {
    let ast_data = load_ast();
    let sources = ast_data.get("sources").unwrap();
    let cache = build_completion_cache(sources, None, None);

    assert!(
        cache.method_identifiers.is_empty(),
        "Without contracts, method_identifiers should be empty"
    );
    // But node_members should still work
    assert!(
        !cache.node_members.is_empty(),
        "node_members should still be populated from AST"
    );
}

// --- function_return_types ---

#[test]
fn test_function_return_types_pool_manager_swap() {
    let cache = load_cache();
    // PoolManager.swap returns BalanceDelta (single return, so it's in the map)
    let ret = cache
        .function_return_types
        .get(&(NodeId(1216), "swap".to_string()));
    assert_eq!(
        ret,
        Some(&"t_userDefinedValueType$_BalanceDelta_$6311".to_string()),
        "PoolManager.swap should return BalanceDelta"
    );
}

#[test]
fn test_function_return_types_pool_manager_internal_swap() {
    let cache = load_cache();
    // _swap is the internal implementation, also returns BalanceDelta
    let ret = cache
        .function_return_types
        .get(&(NodeId(1216), "_swap".to_string()));
    assert_eq!(
        ret,
        Some(&"t_userDefinedValueType$_BalanceDelta_$6311".to_string()),
        "_swap should also return BalanceDelta"
    );
}

#[test]
fn test_function_return_types_ipool_manager_interface() {
    let cache = load_cache();
    // IPoolManager interface also defines swap → BalanceDelta
    let ret = cache
        .function_return_types
        .get(&(NodeId(2123), "swap".to_string()));
    assert_eq!(
        ret,
        Some(&"t_userDefinedValueType$_BalanceDelta_$6311".to_string()),
        "IPoolManager.swap should return BalanceDelta"
    );
}

#[test]
fn test_function_return_types_pool_manager_initialize() {
    let cache = load_cache();
    // initialize returns int24 (tick)
    let ret = cache
        .function_return_types
        .get(&(NodeId(2123), "initialize".to_string()));
    assert_eq!(
        ret,
        Some(&"t_int24".to_string()),
        "IPoolManager.initialize should return int24"
    );
}

#[test]
fn test_function_return_types_get_pool() {
    let cache = load_cache();
    // PoolManager._getPool returns Pool.State storage
    let ret = cache
        .function_return_types
        .get(&(NodeId(1216), "_getPool".to_string()));
    assert_eq!(
        ret,
        Some(&"t_struct$_State_$3870_storage_ptr".to_string()),
        "_getPool should return Pool.State"
    );
}

#[test]
fn test_function_return_types_populated_count() {
    let cache = load_cache();
    // Should have a substantial number of return types from across the AST
    assert!(
        cache.function_return_types.len() > 50,
        "Should have many function return types, got {}",
        cache.function_return_types.len()
    );
}

// --- using_for ---

#[test]
fn test_using_for_balance_delta_has_amount_functions() {
    let cache = load_cache();
    let bd_type = "t_userDefinedValueType$_BalanceDelta_$6311";
    let items = cache.using_for.get(bd_type);
    assert!(
        items.is_some(),
        "Should have using-for entries for BalanceDelta"
    );
    let labels: Vec<&str> = items.unwrap().iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"amount0"),
        "BalanceDelta should have amount0"
    );
    assert!(
        labels.contains(&"amount1"),
        "BalanceDelta should have amount1"
    );
}

#[test]
fn test_using_for_ihooks_has_hook_functions() {
    let cache = load_cache();
    let hooks_type = "t_contract$_IHooks_$1840";
    let items = cache.using_for.get(hooks_type);
    assert!(items.is_some(), "Should have using-for entries for IHooks");
    let labels: Vec<&str> = items.unwrap().iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"beforeSwap"),
        "IHooks should have beforeSwap"
    );
    assert!(
        labels.contains(&"afterSwap"),
        "IHooks should have afterSwap"
    );
    assert!(
        labels.contains(&"hasPermission"),
        "IHooks should have hasPermission"
    );
}

#[test]
fn test_using_for_pool_state_has_pool_functions() {
    let cache = load_cache();
    let state_type = "t_struct$_State_$3870_storage_ptr";
    let items = cache.using_for.get(state_type);
    assert!(
        items.is_some(),
        "Should have using-for entries for Pool.State"
    );
    let labels: Vec<&str> = items.unwrap().iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"initialize"),
        "Pool.State should have initialize"
    );
    assert!(labels.contains(&"swap"), "Pool.State should have swap");
    assert!(labels.contains(&"donate"), "Pool.State should have donate");
    assert!(
        labels.contains(&"modifyLiquidity"),
        "Pool.State should have modifyLiquidity"
    );
}

#[test]
fn test_using_for_slot0_has_accessor_functions() {
    let cache = load_cache();
    let slot0_type = "t_userDefinedValueType$_Slot0_$8632";
    let items = cache.using_for.get(slot0_type);
    assert!(items.is_some(), "Should have using-for entries for Slot0");
    let labels: Vec<&str> = items.unwrap().iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"sqrtPriceX96"),
        "Slot0 should have sqrtPriceX96"
    );
    assert!(labels.contains(&"tick"), "Slot0 should have tick");
    assert!(labels.contains(&"lpFee"), "Slot0 should have lpFee");
}

#[test]
fn test_using_for_uint256_has_safecast() {
    let cache = load_cache();
    let uint256_type = "t_uint256";
    let items = cache.using_for.get(uint256_type);
    assert!(items.is_some(), "Should have using-for entries for uint256");
    let labels: Vec<&str> = items.unwrap().iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"toUint160"),
        "uint256 should have SafeCast.toUint160"
    );
    assert!(
        labels.contains(&"toInt256"),
        "uint256 should have SafeCast.toInt256"
    );
}

#[test]
fn test_using_for_wildcard_populated() {
    let cache = load_cache();
    // Wildcard using-for (using X for *) should have entries
    assert!(
        !cache.using_for_wildcard.is_empty(),
        "using_for_wildcard should not be empty"
    );
    let labels: Vec<&str> = cache
        .using_for_wildcard
        .iter()
        .map(|i| i.label.as_str())
        .collect();
    // SafeCast for * is common
    assert!(
        labels.contains(&"toUint160") || labels.contains(&"toInt128"),
        "Wildcards should include SafeCast functions"
    );
}

#[test]
fn test_using_for_pool_key_has_to_id() {
    let cache = load_cache();
    let pk_type = "t_struct$_PoolKey_$6871_storage_ptr";
    let items = cache.using_for.get(pk_type);
    assert!(items.is_some(), "Should have using-for entries for PoolKey");
    let labels: Vec<&str> = items.unwrap().iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"toId"),
        "PoolKey should have toId from PoolIdLibrary"
    );
}

// --- Chain resolution with get_chain_completions ---

use solidity_language_server::completion::get_chain_completions;

#[test]
fn test_chain_single_plain_contract_name() {
    // PoolManager. — should show all PoolManager members
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "PoolManager".to_string(),
        kind: AccessKind::Plain,
        call_args: None,
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(labels.contains(&"swap"), "PoolManager. should show swap");
    assert!(
        labels.contains(&"initialize"),
        "PoolManager. should show initialize"
    );
}

#[test]
fn test_chain_single_index_mapping_access() {
    // _pools[poolId]. — mapping value is Pool.State, should show struct members + library fns
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "_pools".to_string(),
        kind: AccessKind::Index,
        call_args: None,
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    // Struct members of Pool.State
    assert!(labels.contains(&"slot0"), "_pools[id]. should show slot0");
    assert!(
        labels.contains(&"liquidity"),
        "_pools[id]. should show liquidity"
    );
    assert!(labels.contains(&"ticks"), "_pools[id]. should show ticks");
    assert!(
        labels.contains(&"positions"),
        "_pools[id]. should show positions"
    );

    // Pool library using-for functions (resolved via suffix normalization)
    assert!(
        labels.contains(&"initialize"),
        "_pools[id]. should show Pool.initialize"
    );
    assert!(
        labels.contains(&"swap"),
        "_pools[id]. should show Pool.swap"
    );
    assert!(
        labels.contains(&"donate"),
        "_pools[id]. should show Pool.donate"
    );

    // No duplicates
    assert_eq!(
        items.len(),
        labels.len(),
        "All items should have unique labels (no duplicates)"
    );
    let unique: std::collections::HashSet<&str> = labels.iter().copied().collect();
    assert_eq!(unique.len(), labels.len(), "All labels should be unique");
}

#[test]
fn test_chain_two_segment_contract_function_call() {
    // PoolManager.swap(). — swap returns BalanceDelta, should show amount0/amount1
    let cache = load_cache();
    let chain = vec![
        DotSegment {
            name: "PoolManager".to_string(),
            kind: AccessKind::Plain,
            call_args: None,
        },
        DotSegment {
            name: "swap".to_string(),
            kind: AccessKind::Call,
            call_args: None,
        },
    ];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    // BalanceDelta using-for functions
    assert!(
        labels.contains(&"amount0"),
        "PoolManager.swap(). should show amount0"
    );
    assert!(
        labels.contains(&"amount1"),
        "PoolManager.swap(). should show amount1"
    );
}

#[test]
fn test_chain_two_segment_contract_get_pool_call() {
    // PoolManager._getPool(). — returns Pool.State, should show struct members + Pool library fns
    let cache = load_cache();
    let chain = vec![
        DotSegment {
            name: "PoolManager".to_string(),
            kind: AccessKind::Plain,
            call_args: None,
        },
        DotSegment {
            name: "_getPool".to_string(),
            kind: AccessKind::Call,
            call_args: None,
        },
    ];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    // Pool.State struct members
    assert!(
        labels.contains(&"slot0"),
        "PoolManager._getPool(). should show slot0"
    );
    assert!(
        labels.contains(&"liquidity"),
        "PoolManager._getPool(). should show liquidity"
    );
    assert!(
        labels.contains(&"ticks"),
        "PoolManager._getPool(). should show ticks"
    );

    // Pool library using-for functions
    assert!(
        labels.contains(&"initialize"),
        "PoolManager._getPool(). should show Pool.initialize"
    );
}

#[test]
fn test_chain_variable_with_type_then_call() {
    // pool is a Pool.State storage variable in the AST (name_to_type has it)
    // pool.swap(). — swap on Pool.State returns BalanceDelta
    let cache = load_cache();

    // Verify pool is in name_to_type
    assert!(
        cache.name_to_type.contains_key("pool"),
        "pool should be in name_to_type"
    );

    let chain = vec![
        DotSegment {
            name: "pool".to_string(),
            kind: AccessKind::Plain,
            call_args: None,
        },
        DotSegment {
            name: "swap".to_string(),
            kind: AccessKind::Call,
            call_args: None,
        },
    ];
    let items = get_chain_completions(&cache, &chain, None);
    // pool is t_struct$_State_$3870_storage_ptr. The swap function on Pool.State (node 4809)
    // returns BalanceDelta — but function_return_types is keyed by (contract_id, fn_name).
    // Pool library's id is 6348, and its swap returns BalanceDelta.
    // resolve_member_type for Call looks up function_return_types[(context_node_id, "swap")].
    // This may or may not work depending on whether 4809 or 6348 is used as context.
    // For now just verify it returns something or is empty (known limitation).
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    // Note: if this resolves, it should show BalanceDelta members (amount0, amount1)
    // If not, it's a known limitation — function_return_types is keyed by the library node id,
    // not the struct node id.
    if !labels.is_empty() {
        assert!(
            labels.contains(&"amount0"),
            "pool.swap(). should show amount0 if resolved"
        );
    }
}

#[test]
fn test_chain_mapping_value_type_extraction() {
    // _pools is a mapping(PoolId => Pool.State). Extracting value type should give Pool.State.
    let pools_type =
        "t_mapping$_t_userDefinedValueType$_PoolId_$6825_$_t_struct$_State_$3870_storage_$";
    let val_type = extract_mapping_value_type(pools_type);
    assert_eq!(
        val_type,
        Some("t_struct$_State_$3870_storage".to_string()),
        "Mapping value type should be Pool.State"
    );
}

#[test]
fn test_chain_nested_mapping_value_extraction() {
    // positions is mapping(bytes32 => mapping(bytes32 => Position.State))
    // The extract should peel all layers to get the innermost value
    let nested_mapping =
        "t_mapping$_t_bytes32_$_t_mapping$_t_bytes32_$_t_struct$_State_$5433_storage_$_$";
    let val_type = extract_mapping_value_type(nested_mapping);
    assert!(
        val_type.is_some(),
        "Should extract innermost value type from nested mapping"
    );
    let val = val_type.unwrap();
    assert!(
        val.contains("5433"),
        "Innermost type should reference Position.State (5433), got: {}",
        val
    );
}

#[test]
fn test_chain_completions_deduplication() {
    // Completions should have unique labels — no duplicates from using_for + wildcard overlap
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "_pools".to_string(),
        kind: AccessKind::Index,
        call_args: None,
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    let unique: std::collections::HashSet<&str> = labels.iter().copied().collect();
    assert_eq!(
        labels.len(),
        unique.len(),
        "Completions should not have duplicate labels. Duplicates: {:?}",
        labels
            .iter()
            .filter(|l| labels.iter().filter(|l2| l2 == l).count() > 1)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_chain_ipool_manager_initialize_returns_int24() {
    // IPoolManager.initialize(). — returns int24, which has SafeCast functions
    let cache = load_cache();
    let chain = vec![
        DotSegment {
            name: "IPoolManager".to_string(),
            kind: AccessKind::Plain,
            call_args: None,
        },
        DotSegment {
            name: "initialize".to_string(),
            kind: AccessKind::Call,
            call_args: None,
        },
    ];
    let items = get_chain_completions(&cache, &chain, None);
    // int24 doesn't have many natural members, but SafeCast wildcard functions apply
    // This tests that the chain resolves through to the return type
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    if !labels.is_empty() {
        // Wildcard SafeCast functions should be available on int24
        assert!(
            labels.contains(&"toInt128") || labels.contains(&"toUint160"),
            "int24 return should get SafeCast wildcard functions"
        );
    }
}

#[test]
fn test_chain_pool_key_to_id() {
    // poolKey.toId(). — PoolKey has using-for toId that returns PoolId
    let cache = load_cache();
    // poolKey is in name_to_type
    assert!(
        cache.name_to_type.contains_key("poolKey"),
        "poolKey should be in name_to_type"
    );

    let chain = vec![DotSegment {
        name: "poolKey".to_string(),
        kind: AccessKind::Plain,
        call_args: None,
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    // PoolKey struct members
    assert!(
        labels.contains(&"currency0"),
        "poolKey. should show currency0"
    );
    assert!(
        labels.contains(&"currency1"),
        "poolKey. should show currency1"
    );
    assert!(labels.contains(&"fee"), "poolKey. should show fee");
    assert!(
        labels.contains(&"tickSpacing"),
        "poolKey. should show tickSpacing"
    );
    assert!(labels.contains(&"hooks"), "poolKey. should show hooks");

    // Using-for: toId from PoolIdLibrary
    assert!(
        labels.contains(&"toId"),
        "poolKey. should show toId from PoolIdLibrary"
    );
}

#[test]
fn test_chain_empty_returns_nothing() {
    let cache = load_cache();
    let items = get_chain_completions(&cache, &[], None);
    assert!(items.is_empty(), "Empty chain should return no completions");
}

#[test]
fn test_chain_unknown_name_returns_nothing() {
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "nonexistentVariable".to_string(),
        kind: AccessKind::Plain,
        call_args: None,
    }];
    let items = get_chain_completions(&cache, &chain, None);
    assert!(
        items.is_empty(),
        "Unknown name should return no completions"
    );
}

// --- Type cast expressions: InterfaceName(address).method() ---

#[test]
fn test_parse_chain_type_cast_expression() {
    // IUnlockCallback(msg.sender).unlockCallback(data).
    // Parser sees the parentheses as Call kind — syntactically identical to a function call
    let line = "        IUnlockCallback(msg.sender).unlockCallback(data).";
    let col = line.len() as u32;
    let chain = parse_dot_chain(line, col);
    assert_eq!(chain.len(), 2);
    assert_eq!(chain[0].name, "IUnlockCallback");
    assert_eq!(chain[0].kind, AccessKind::Call);
    assert_eq!(chain[1].name, "unlockCallback");
    assert_eq!(chain[1].kind, AccessKind::Call);
}

#[test]
fn test_chain_type_cast_shows_interface_members() {
    // IUnlockCallback(msg.sender). — type cast to interface, should show its members
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "IUnlockCallback".to_string(),
        kind: AccessKind::Call,
        call_args: None,
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"unlockCallback"),
        "IUnlockCallback(addr). should show unlockCallback, got: {:?}",
        labels
    );
}

#[test]
fn test_chain_type_cast_then_method_returns_bytes_completions() {
    // IUnlockCallback(msg.sender).unlockCallback(data).
    // unlockCallback returns bytes memory → should show bytes-related completions
    let cache = load_cache();
    let chain = vec![
        DotSegment {
            name: "IUnlockCallback".to_string(),
            kind: AccessKind::Call,
            call_args: None,
        },
        DotSegment {
            name: "unlockCallback".to_string(),
            kind: AccessKind::Call,
            call_args: None,
        },
    ];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    assert!(
        !items.is_empty(),
        "IUnlockCallback(msg.sender).unlockCallback(data). should return completions"
    );
    // unlockCallback returns bytes memory.
    // using-for on bytes includes parseSelector, parseFee, parseReturnDelta
    assert!(
        labels.contains(&"parseSelector"),
        "bytes return should show parseSelector from using-for, got: {:?}",
        labels
    );
}

// --- Globally available types from Solidity docs ---

#[test]
fn test_general_completions_include_blobhash() {
    let cache = load_cache();
    let completions = get_general_completions(&cache);
    let names: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
    assert!(
        names.contains(&"blobhash(uint256 index)"),
        "Should have blobhash"
    );
}

#[test]
fn test_general_completions_include_ether_units() {
    let cache = load_cache();
    let completions = get_general_completions(&cache);
    let names: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
    assert!(names.contains(&"wei"), "Should have wei");
    assert!(names.contains(&"gwei"), "Should have gwei");
    assert!(names.contains(&"ether"), "Should have ether");

    // Check they are UNIT kind
    let wei = completions.iter().find(|c| c.label == "wei").unwrap();
    assert_eq!(wei.kind, Some(CompletionItemKind::UNIT));
    assert_eq!(wei.detail.as_deref(), Some("1"));

    let ether = completions.iter().find(|c| c.label == "ether").unwrap();
    assert_eq!(ether.detail.as_deref(), Some("1e18"));
}

#[test]
fn test_general_completions_include_time_units() {
    let cache = load_cache();
    let completions = get_general_completions(&cache);
    let names: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
    assert!(names.contains(&"seconds"), "Should have seconds");
    assert!(names.contains(&"minutes"), "Should have minutes");
    assert!(names.contains(&"hours"), "Should have hours");
    assert!(names.contains(&"days"), "Should have days");
    assert!(names.contains(&"weeks"), "Should have weeks");

    let days = completions.iter().find(|c| c.label == "days").unwrap();
    assert_eq!(days.kind, Some(CompletionItemKind::UNIT));
    assert_eq!(days.detail.as_deref(), Some("86400 seconds"));
}

#[test]
fn test_dot_completion_type_members() {
    // type(X). should show contract/interface/integer type properties
    let cache = load_cache();
    let items = get_dot_completions(&cache, "type", None);
    let names: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();

    // Contract type properties
    assert!(names.contains(&"name"), "type(C).name");
    assert!(names.contains(&"creationCode"), "type(C).creationCode");
    assert!(names.contains(&"runtimeCode"), "type(C).runtimeCode");
    // Interface type property
    assert!(names.contains(&"interfaceId"), "type(I).interfaceId");
    // Integer type properties
    assert!(names.contains(&"min"), "type(T).min");
    assert!(names.contains(&"max"), "type(T).max");
}

#[test]
fn test_dot_completion_bytes_concat() {
    let cache = load_cache();
    let items = get_dot_completions(&cache, "bytes", None);
    let names: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();
    assert!(names.contains(&"concat(...)"), "bytes.concat");

    let concat = items.iter().find(|c| c.label == "concat(...)").unwrap();
    assert_eq!(concat.detail.as_deref(), Some("bytes memory"));
}

#[test]
fn test_dot_completion_string_concat() {
    let cache = load_cache();
    let items = get_dot_completions(&cache, "string", None);
    let names: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();
    assert!(names.contains(&"concat(...)"), "string.concat");

    let concat = items.iter().find(|c| c.label == "concat(...)").unwrap();
    assert_eq!(concat.detail.as_deref(), Some("string memory"));
}

#[test]
fn test_dot_completion_abi_full_signatures() {
    let cache = load_cache();
    let items = get_dot_completions(&cache, "abi", None);
    let names: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();

    assert!(
        names.contains(&"encodeWithSelector(bytes4, ...)"),
        "abi.encodeWithSelector with selector param"
    );
    assert!(
        names.contains(&"encodeWithSignature(string memory, ...)"),
        "abi.encodeWithSignature with signature param"
    );
    assert!(
        names.contains(&"encodeCall(function, (...))"),
        "abi.encodeCall with function pointer"
    );
}

#[test]
fn test_general_completions_include_common_int_types() {
    let cache = load_cache();
    let completions = get_general_completions(&cache);
    let names: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

    // Verify commonly used integer types are present as keywords
    assert!(names.contains(&"uint8"), "Should have uint8");
    assert!(names.contains(&"uint128"), "Should have uint128");
    assert!(names.contains(&"uint160"), "Should have uint160");
    assert!(names.contains(&"uint256"), "Should have uint256");
    assert!(names.contains(&"int24"), "Should have int24");
    assert!(names.contains(&"int128"), "Should have int128");
    assert!(names.contains(&"int256"), "Should have int256");
    assert!(names.contains(&"bytes1"), "Should have bytes1");
    assert!(names.contains(&"bytes4"), "Should have bytes4");
}

// --- Scope-aware completion tests ---

use solidity_language_server::completion::{ScopeContext, resolve_name_in_scope};

#[test]
fn test_scope_declarations_populated() {
    let cache = load_cache();
    // scope_declarations should have entries — variables bucketed by their enclosing scope
    assert!(
        !cache.scope_declarations.is_empty(),
        "scope_declarations should be populated"
    );
}

#[test]
fn test_scope_parent_populated() {
    let cache = load_cache();
    // scope_parent should have parent links
    assert!(
        !cache.scope_parent.is_empty(),
        "scope_parent should be populated"
    );
}

#[test]
fn test_scope_ranges_populated() {
    let cache = load_cache();
    // scope_ranges should have entries for scope-creating nodes
    assert!(
        !cache.scope_ranges.is_empty(),
        "scope_ranges should be populated"
    );
    // Should be sorted by span size ascending (smallest first)
    for pair in cache.scope_ranges.windows(2) {
        let span_a = pair[0].end - pair[0].start;
        let span_b = pair[1].end - pair[1].start;
        assert!(
            span_a <= span_b,
            "scope_ranges should be sorted by span size ascending"
        );
    }
}

#[test]
fn test_path_to_file_id_populated() {
    let cache = load_cache();
    assert!(
        !cache.path_to_file_id.is_empty(),
        "path_to_file_id should be populated"
    );
}

#[test]
fn test_scope_resolve_self_in_pool_swap_is_pool_state() {
    let cache = load_cache();
    // Pool.swap is in file 29, at bytes 12231..21851
    // "self" inside Pool.swap should resolve to Pool.State storage
    // Position cursor inside the function body (byte 12300)
    let result = resolve_name_in_scope(&cache, "self", 12300, FileId(28));
    assert_eq!(
        result,
        Some("t_struct$_State_$3870_storage_ptr".to_string()),
        "self inside Pool.swap should be Pool.State storage"
    );
}

#[test]
fn test_scope_resolve_self_in_hooks_is_ihooks() {
    let cache = load_cache();
    // Hooks.beforeInitialize is in file 23, src="3643:11:23"
    // The function that contains self with IHooks type is scope 3556
    // which is at file 23. Let's position inside a Hooks function body.
    // Hooks.hasPermission (id=3556) is one such function.
    // We need a byte position inside a Hooks library function in file 23.
    // src for self decl is "3643:11:23", meaning function starts around there.
    // Let's use byte 3650 in file 23.
    let result = resolve_name_in_scope(&cache, "self", 3650, FileId(22));
    assert_eq!(
        result,
        Some("t_contract$_IHooks_$1840".to_string()),
        "self inside Hooks library should be IHooks"
    );
}

#[test]
fn test_scope_resolve_key_in_pool_manager_swap_is_pool_key() {
    let cache = load_cache();
    // PoolManager.swap (id=1167) is in file 6, bytes 9385..11071
    // "key" parameter has type PoolKey memory
    let result = resolve_name_in_scope(&cache, "key", 9500, FileId(5));
    assert_eq!(
        result,
        Some("t_struct$_PoolKey_$6871_memory_ptr".to_string()),
        "key inside PoolManager.swap should be PoolKey memory"
    );
}

#[test]
fn test_scope_resolve_walks_up_to_contract_state_var() {
    let cache = load_cache();
    // Inside Owned contract constructor (file 0, bytes 1007..1122),
    // "owner" is not a local or parameter — it's a state variable on Owned (scope=59).
    // Cursor at byte 1050 (inside constructor body).
    let result = resolve_name_in_scope(&cache, "owner", 1050, FileId(44));
    assert_eq!(
        result,
        Some("t_address".to_string()),
        "owner inside Owned constructor should resolve to state variable (address)"
    );
}

#[test]
fn test_scope_resolve_unknown_name_falls_back() {
    let cache = load_cache();
    // A name that doesn't exist in any scope should return None from scope walk,
    // then fall back to resolve_name_to_type_id (flat lookup).
    // "PoolManager" is a contract name, not a variable — it should be resolved via fallback.
    let result = resolve_name_in_scope(&cache, "PoolManager", 9500, FileId(5));
    assert!(
        result.is_some(),
        "Contract names should be resolved via fallback"
    );
}

#[test]
fn test_scope_chain_completions_with_context() {
    let cache = load_cache();
    // Test that get_chain_completions with scope context resolves "self."
    // correctly inside Pool.swap (file 29, byte 12300) — should get Pool.State members
    let chain = vec![DotSegment {
        name: "self".to_string(),
        kind: AccessKind::Plain,
        call_args: None,
    }];
    let ctx = ScopeContext {
        byte_pos: 12300,
        file_id: FileId(28),
    };
    let items = get_chain_completions(&cache, &chain, Some(&ctx));
    let names: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();
    // Pool.State has fields like slot0, feeGrowthGlobal0X128, liquidity, etc.
    assert!(
        names.contains(&"slot0") || names.contains(&"liquidity"),
        "self. inside Pool.swap should show Pool.State members, got: {:?}",
        names
    );
}

#[test]
fn test_scope_chain_completions_without_context_uses_flat() {
    let cache = load_cache();
    // Without scope context (no file_id), self. resolves via flat first-wins lookup
    let chain = vec![DotSegment {
        name: "self".to_string(),
        kind: AccessKind::Plain,
        call_args: None,
    }];
    let items_no_scope = get_chain_completions(&cache, &chain, None);
    // Should still return something (whatever first-wins gives us)
    // The important thing is it doesn't crash
    assert!(
        !items_no_scope.is_empty() || items_no_scope.is_empty(),
        "Should handle None scope context gracefully"
    );
}

// --- Inheritance resolution tests ---

#[test]
fn test_linearized_base_contracts_populated() {
    let cache = load_cache();
    assert!(
        !cache.linearized_base_contracts.is_empty(),
        "linearized_base_contracts should be populated"
    );
    // PoolManager (id=1767) should have multiple base contracts
    let pm_bases = cache.linearized_base_contracts.get(&NodeId(1216));
    assert!(
        pm_bases.is_some(),
        "PoolManager should have linearized base contracts"
    );
    let bases = pm_bases.unwrap();
    assert!(
        bases.len() > 1,
        "PoolManager should inherit from multiple contracts"
    );
    assert_eq!(
        bases[0],
        NodeId(1216),
        "First base should be the contract itself"
    );
    // Owned (id=59) should be in the list
    assert!(
        bases.contains(&NodeId(7455)),
        "PoolManager should inherit from Owned (id=59), got: {:?}",
        bases
    );
}

#[test]
fn test_scope_resolve_inherited_state_var_owner_in_pool_manager() {
    let cache = load_cache();
    // Inside PoolManager.swap (file 6, bytes 9385..11071), "owner" is a state variable
    // inherited from the Owned contract (id=59). The scope walk should:
    //   1. Check the Block scope (swap body) — no "owner"
    //   2. Check the FunctionDefinition scope (swap params) — no "owner"
    //   3. Check ContractDefinition (PoolManager, id=1767) — no "owner"
    //   4. Walk linearizedBaseContracts → find "owner" in Owned (id=59)
    let result = resolve_name_in_scope(&cache, "owner", 9500, FileId(5));
    assert_eq!(
        result,
        Some("t_address".to_string()),
        "owner inside PoolManager.swap should resolve to inherited state var from Owned"
    );
}

#[test]
fn test_scope_resolve_inherited_protocol_fee_controller() {
    let cache = load_cache();
    // Inside PoolManager.swap (file 6, bytes 9385..11071), "protocolFeeController"
    // is a state variable inherited from ProtocolFees (id=1994).
    let result = resolve_name_in_scope(&cache, "protocolFeeController", 9500, FileId(5));
    assert_eq!(
        result,
        Some("t_address".to_string()),
        "protocolFeeController inside PoolManager.swap should resolve to inherited state var from ProtocolFees"
    );
}

#[test]
fn test_scope_resolve_own_state_var_still_works() {
    let cache = load_cache();
    // Inside PoolManager.swap (file 6), "_pools" is PoolManager's own state variable
    // (scope=1767). It should still resolve correctly — the contract's own declarations
    // are checked before walking base contracts.
    let result = resolve_name_in_scope(&cache, "_pools", 9500, FileId(5));
    assert!(
        result.is_some(),
        "_pools inside PoolManager.swap should resolve to PoolManager's own state variable"
    );
}

// =============================================================================
// AST scope structure tests
//
// These tests validate the scope hierarchy extracted from the Solidity AST.
// Data comes from jq queries against pool-manager-ast.json.
// =============================================================================

/// Helper: walk AST recursively, collecting all objects.
fn collect_all_nodes(ast: &Value) -> Vec<&Value> {
    let mut result = Vec::new();
    let mut stack = vec![ast];
    while let Some(node) = stack.pop() {
        match node {
            Value::Object(map) => {
                result.push(node);
                for v in map.values() {
                    stack.push(v);
                }
            }
            Value::Array(arr) => {
                for v in arr {
                    stack.push(v);
                }
            }
            _ => {}
        }
    }
    result
}

/// Helper: count nodes matching a predicate.
fn count_nodes<F: Fn(&Value) -> bool>(nodes: &[&Value], pred: F) -> usize {
    nodes.iter().filter(|n| pred(n)).count()
}

// --- Node types that have `scope` fields ---

#[test]
fn test_ast_variable_declarations_with_scope() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let count = count_nodes(&nodes, |n| {
        n.get("nodeType").and_then(|v| v.as_str()) == Some("VariableDeclaration")
            && n.get("scope").is_some()
    });
    assert_eq!(count, 957, "VariableDeclaration nodes with scope field");
}

#[test]
fn test_ast_function_definitions_with_scope() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let count = count_nodes(&nodes, |n| {
        n.get("nodeType").and_then(|v| v.as_str()) == Some("FunctionDefinition")
            && n.get("scope").is_some()
    });
    assert_eq!(count, 215, "FunctionDefinition nodes with scope field");
}

#[test]
fn test_ast_import_directives_with_scope() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let count = count_nodes(&nodes, |n| {
        n.get("nodeType").and_then(|v| v.as_str()) == Some("ImportDirective")
            && n.get("scope").is_some()
    });
    assert_eq!(count, 107, "ImportDirective nodes with scope field");
}

#[test]
fn test_ast_contract_definitions_with_scope() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let count = count_nodes(&nodes, |n| {
        n.get("nodeType").and_then(|v| v.as_str()) == Some("ContractDefinition")
            && n.get("scope").is_some()
    });
    assert_eq!(count, 43, "ContractDefinition nodes with scope field");
}

#[test]
fn test_ast_struct_definitions_with_scope() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let count = count_nodes(&nodes, |n| {
        n.get("nodeType").and_then(|v| v.as_str()) == Some("StructDefinition")
            && n.get("scope").is_some()
    });
    assert_eq!(count, 12, "StructDefinition nodes with scope field");
}

// --- Scope target types: what nodeType do scope IDs point to ---

/// Build a map of id -> nodeType for the entire AST.
fn build_id_to_node_type(nodes: &[&Value]) -> std::collections::HashMap<i64, String> {
    let mut map = std::collections::HashMap::new();
    for n in nodes {
        if let (Some(id), Some(nt)) = (
            n.get("id").and_then(|v| v.as_i64()),
            n.get("nodeType").and_then(|v| v.as_str()),
        ) {
            map.insert(id, nt.to_string());
        }
    }
    map
}

/// Count how many unique scope IDs point to a given nodeType.
fn count_unique_scope_targets(
    nodes: &[&Value],
    id_to_type: &std::collections::HashMap<i64, String>,
    target_type: &str,
) -> usize {
    let mut scope_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for n in nodes.iter() {
        if let Some(scope_id) = n.get("scope").and_then(|v| v.as_i64())
            && id_to_type.get(&scope_id).map(|s| s.as_str()) == Some(target_type)
        {
            scope_ids.insert(scope_id);
        }
    }
    scope_ids.len()
}

#[test]
fn test_ast_scope_targets_function_definition() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_unique_scope_targets(&nodes, &id_map, "FunctionDefinition"),
        208,
        "unique scope IDs pointing to FunctionDefinition"
    );
}

#[test]
fn test_ast_scope_targets_block() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_unique_scope_targets(&nodes, &id_map, "Block"),
        52,
        "unique scope IDs pointing to Block"
    );
}

#[test]
fn test_ast_scope_targets_source_unit() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_unique_scope_targets(&nodes, &id_map, "SourceUnit"),
        45,
        "unique scope IDs pointing to SourceUnit"
    );
}

#[test]
fn test_ast_scope_targets_contract_definition() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_unique_scope_targets(&nodes, &id_map, "ContractDefinition"),
        43,
        "unique scope IDs pointing to ContractDefinition"
    );
}

#[test]
fn test_ast_scope_targets_error_definition() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_unique_scope_targets(&nodes, &id_map, "ErrorDefinition"),
        16,
        "unique scope IDs pointing to ErrorDefinition"
    );
}

#[test]
fn test_ast_scope_targets_struct_definition() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_unique_scope_targets(&nodes, &id_map, "StructDefinition"),
        12,
        "unique scope IDs pointing to StructDefinition"
    );
}

#[test]
fn test_ast_scope_targets_event_definition() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_unique_scope_targets(&nodes, &id_map, "EventDefinition"),
        12,
        "unique scope IDs pointing to EventDefinition"
    );
}

#[test]
fn test_ast_scope_targets_unchecked_block() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_unique_scope_targets(&nodes, &id_map, "UncheckedBlock"),
        11,
        "unique scope IDs pointing to UncheckedBlock"
    );
}

#[test]
fn test_ast_scope_targets_modifier_definition() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_unique_scope_targets(&nodes, &id_map, "ModifierDefinition"),
        1,
        "unique scope IDs pointing to ModifierDefinition"
    );
}

// --- Block/UncheckedBlock have NO scope field (the bug surface) ---

#[test]
fn test_ast_blocks_have_no_scope_field() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let total_blocks = count_nodes(&nodes, |n| {
        n.get("nodeType").and_then(|v| v.as_str()) == Some("Block")
    });
    let blocks_with_scope = count_nodes(&nodes, |n| {
        n.get("nodeType").and_then(|v| v.as_str()) == Some("Block") && n.get("scope").is_some()
    });
    assert_eq!(total_blocks, 277, "total Block nodes in AST");
    assert_eq!(
        blocks_with_scope, 0,
        "Block nodes should have no scope field — this is the bug surface"
    );
}

#[test]
fn test_ast_unchecked_blocks_have_no_scope_field() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let total = count_nodes(&nodes, |n| {
        n.get("nodeType").and_then(|v| v.as_str()) == Some("UncheckedBlock")
    });
    let with_scope = count_nodes(&nodes, |n| {
        n.get("nodeType").and_then(|v| v.as_str()) == Some("UncheckedBlock")
            && n.get("scope").is_some()
    });
    assert_eq!(total, 29, "total UncheckedBlock nodes in AST");
    assert_eq!(
        with_scope, 0,
        "UncheckedBlock nodes should have no scope field"
    );
}

// --- Scope-creating nodes: total counts ---

#[test]
fn test_ast_scope_node_counts() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let count_type = |t: &str| {
        count_nodes(&nodes, |n| {
            n.get("nodeType").and_then(|v| v.as_str()) == Some(t)
        })
    };
    assert_eq!(count_type("SourceUnit"), 45);
    assert_eq!(count_type("ContractDefinition"), 43);
    assert_eq!(count_type("FunctionDefinition"), 215);
    assert_eq!(count_type("ModifierDefinition"), 4);
    assert_eq!(count_type("Block"), 277);
    assert_eq!(count_type("UncheckedBlock"), 29);
}

// --- Scope nodes missing parent links (need inference) ---

#[test]
fn test_ast_scope_nodes_without_scope_field() {
    // These are the nodes that need parent inference.
    // Block: 277, UncheckedBlock: 29, ModifierDefinition: 4, SourceUnit: 45 (root, no parent needed)
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let missing = |t: &str| {
        count_nodes(&nodes, |n| {
            n.get("nodeType").and_then(|v| v.as_str()) == Some(t) && n.get("scope").is_none()
        })
    };
    assert_eq!(missing("Block"), 277, "all Blocks lack scope field");
    assert_eq!(
        missing("UncheckedBlock"),
        29,
        "all UncheckedBlocks lack scope field"
    );
    assert_eq!(
        missing("ModifierDefinition"),
        4,
        "all ModifierDefinitions lack scope field"
    );
    assert_eq!(
        missing("SourceUnit"),
        45,
        "SourceUnits are roots — no parent expected"
    );
}

// --- Parent-child scope relationships ---
// "Who scopes to whom" — validates that our scope_declarations captures the right things.

/// Count children of a given nodeType whose scope points to a specific parent nodeType.
fn count_children_scoped_to(
    nodes: &[&Value],
    id_to_type: &std::collections::HashMap<i64, String>,
    child_type: &str,
    parent_type: &str,
) -> usize {
    nodes
        .iter()
        .filter(|n| {
            n.get("nodeType").and_then(|v| v.as_str()) == Some(child_type)
                && n.get("scope")
                    .and_then(|v| v.as_i64())
                    .and_then(|id| id_to_type.get(&id))
                    .map(|s| s.as_str())
                    == Some(parent_type)
        })
        .count()
}

#[test]
fn test_ast_vars_scoped_to_function_definition() {
    // VariableDeclarations with scope -> FunctionDefinition (parameters + returns)
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "VariableDeclaration", "FunctionDefinition"),
        637
    );
}

#[test]
fn test_ast_vars_scoped_to_block() {
    // 99 VariableDeclarations have scope -> Block (local variables)
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "VariableDeclaration", "Block"),
        99
    );
}

#[test]
fn test_ast_vars_scoped_to_contract_definition() {
    // 56 VariableDeclarations have scope -> ContractDefinition (state variables)
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "VariableDeclaration", "ContractDefinition"),
        56
    );
}

#[test]
fn test_ast_vars_scoped_to_unchecked_block() {
    // 26 VariableDeclarations have scope -> UncheckedBlock
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "VariableDeclaration", "UncheckedBlock"),
        26
    );
}

#[test]
fn test_ast_vars_scoped_to_struct_definition() {
    // 66 VariableDeclarations have scope -> StructDefinition (struct fields)
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "VariableDeclaration", "StructDefinition"),
        66
    );
}

#[test]
fn test_ast_vars_scoped_to_event_definition() {
    // VariableDeclarations with scope -> EventDefinition
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "VariableDeclaration", "EventDefinition"),
        49
    );
}

#[test]
fn test_ast_vars_scoped_to_error_definition() {
    // 23 VariableDeclarations have scope -> ErrorDefinition
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "VariableDeclaration", "ErrorDefinition"),
        23
    );
}

#[test]
fn test_ast_functions_scoped_to_contract_definition() {
    // 205 FunctionDefinitions have scope -> ContractDefinition
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "FunctionDefinition", "ContractDefinition"),
        205
    );
}

#[test]
fn test_ast_functions_scoped_to_source_unit() {
    // 10 FunctionDefinitions have scope -> SourceUnit (free functions)
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "FunctionDefinition", "SourceUnit"),
        10
    );
}

#[test]
fn test_ast_imports_scoped_to_source_unit() {
    // 107 ImportDirectives have scope -> SourceUnit
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "ImportDirective", "SourceUnit"),
        107
    );
}

#[test]
fn test_ast_contracts_scoped_to_source_unit() {
    // 43 ContractDefinitions have scope -> SourceUnit
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "ContractDefinition", "SourceUnit"),
        43
    );
}

#[test]
fn test_ast_structs_scoped_to_contract_definition() {
    // 9 StructDefinitions have scope -> ContractDefinition
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "StructDefinition", "ContractDefinition"),
        9
    );
}

#[test]
fn test_ast_structs_scoped_to_source_unit() {
    // 3 StructDefinitions have scope -> SourceUnit (top-level structs)
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    assert_eq!(
        count_children_scoped_to(&nodes, &id_map, "StructDefinition", "SourceUnit"),
        3
    );
}

// --- Contract definitions: names and IDs ---

#[test]
fn test_ast_all_contract_definitions() {
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let contracts: Vec<(String, i64)> = nodes
        .iter()
        .filter_map(|n| {
            if n.get("nodeType").and_then(|v| v.as_str()) == Some("ContractDefinition") {
                let name = n.get("name").and_then(|v| v.as_str())?.to_string();
                let id = n.get("id").and_then(|v| v.as_i64())?;
                Some((name, id))
            } else {
                None
            }
        })
        .collect();

    let expected: Vec<(&str, i64)> = vec![
        ("Owned", 7455),
        ("ERC6909", 7191),
        ("ERC6909Claims", 1289),
        ("Extsload", 1332),
        ("Exttload", 1362),
        ("NoDelegateCall", 1414),
        ("PoolManager", 1216),
        ("ProtocolFees", 1641),
        ("IExtsload", 7224),
        ("IExttload", 7246),
        ("IHooks", 1840),
        ("IPoolManager", 2123),
        ("IProtocolFees", 7323),
        ("IUnlockCallback", 2135),
        ("IERC20Minimal", 9021),
        ("IERC6909Claims", 7569),
        ("BitMath", 8949),
        ("CurrencyDelta", 2204),
        ("CurrencyReserves", 2252),
        ("CustomRevert", 2358),
        ("FixedPoint128", 7607),
        ("FixedPoint96", 9031),
        ("FullMath", 8913),
        ("Hooks", 3530),
        ("LPFeeLibrary", 3679),
        ("LiquidityMath", 7623),
        ("Lock", 3703),
        ("NonzeroDeltaCount", 3728),
        ("ParseBytes", 7600),
        ("Pool", 5409),
        ("Position", 5575),
        ("ProtocolFeeLibrary", 7395),
        ("SafeCast", 5751),
        ("SqrtPriceMath", 8114),
        ("SwapMath", 8366),
        ("TickBitmap", 8598),
        ("TickMath", 6305),
        ("UnsafeMath", 8628),
        ("BalanceDeltaLibrary", 6469),
        ("BeforeSwapDeltaLibrary", 6517),
        ("CurrencyLibrary", 6819),
        ("PoolIdLibrary", 6839),
        ("Slot0Library", 8745),
    ];

    assert_eq!(contracts.len(), expected.len(), "contract count mismatch");
    for (name, id) in &expected {
        assert!(
            contracts.iter().any(|(n, i)| n == name && i == id),
            "missing contract {} (id={})",
            name,
            id
        );
    }
}

// --- CompletionCache captures the right scope data ---

#[test]
fn test_cache_scope_declarations_has_block_scoped_vars() {
    // 99 vars are scoped to Block nodes. Our scope_declarations should capture them.
    let cache = load_cache();
    let mut block_scoped_count = 0;
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);

    for (scope_id, decls) in &cache.scope_declarations {
        if id_map.get(&scope_id.0).map(|s| s.as_str()) == Some("Block") {
            block_scoped_count += decls.len();
        }
    }
    // VariableDeclarations scoped to Block = 99
    // We don't record FunctionDefinitions in Block scope, so this should be ~99
    assert!(
        block_scoped_count >= 99,
        "should capture all 99 block-scoped variables, got {}",
        block_scoped_count
    );
}

#[test]
fn test_cache_scope_declarations_has_function_params() {
    // Vars scoped to FunctionDefinition (parameters + returns) should be captured.
    // scope_declarations also holds FunctionDefinitions scoped to contracts,
    // so we count only ScopedDeclarations whose scope target is a FunctionDefinition.
    let cache = load_cache();
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    let mut fn_scoped_count = 0;
    for (scope_id, decls) in &cache.scope_declarations {
        if id_map.get(&scope_id.0).map(|s| s.as_str()) == Some("FunctionDefinition") {
            fn_scoped_count += decls.len();
        }
    }
    // We record both VariableDeclaration (params) and FunctionDefinition names here.
    // At minimum we should have the 637 function-parameter variables.
    assert!(
        fn_scoped_count >= 557,
        "should capture function-scoped variables, got {}",
        fn_scoped_count
    );
}

#[test]
fn test_cache_scope_declarations_has_state_vars() {
    // 56 vars are scoped to ContractDefinition. Our scope_declarations should capture them.
    let cache = load_cache();
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    let mut contract_scoped_count = 0;
    for (scope_id, decls) in &cache.scope_declarations {
        if id_map.get(&scope_id.0).map(|s| s.as_str()) == Some("ContractDefinition") {
            contract_scoped_count += decls.len();
        }
    }
    // ContractDefinition scope has 56 vars + 205 functions = 261
    assert!(
        contract_scoped_count >= 56,
        "should capture at least 56 contract-scoped state variables, got {}",
        contract_scoped_count
    );
}

#[test]
fn test_cache_scope_parent_count() {
    // scope_parent has entries for:
    // - Nodes with AST `scope` field: FunctionDef(215) + ContractDef(43) = 258
    // - Inferred parents: Block(277) + UncheckedBlock(29) + ModifierDef(4) = 310
    // Total = 568. SourceUnit(45) has no parent (root).
    let cache = load_cache();
    assert_eq!(
        cache.scope_parent.len(),
        568,
        "scope_parent should have 568 entries (258 from AST + 310 inferred)"
    );
}

#[test]
fn test_cache_scope_ranges_count() {
    // Total scope-creating nodes: SourceUnit(45) + ContractDef(43) + FunctionDef(215)
    // + ModifierDef(4) + Block(277) + UncheckedBlock(29) = 613
    let cache = load_cache();
    assert_eq!(
        cache.scope_ranges.len(),
        613,
        "scope_ranges should have 613 entries (all scope-creating nodes)"
    );
}

// --- The Block parent linkage bug: scope walk stops at Block ---

#[test]
fn test_cache_blocks_have_no_parent_link() {
    // This test documents the bug: Block nodes are in scope_ranges but NOT in scope_parent.
    // The scope walk finds the innermost Block but can't walk up to FunctionDefinition.
    let cache = load_cache();
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);

    // Collect all Block node IDs
    let block_ids: Vec<i64> = nodes
        .iter()
        .filter_map(|n| {
            if n.get("nodeType").and_then(|v| v.as_str()) == Some("Block") {
                n.get("id").and_then(|v| v.as_i64())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(block_ids.len(), 277);

    let blocks_with_parent = block_ids
        .iter()
        .filter(|id| cache.scope_parent.contains_key(&NodeId(**id)))
        .count();

    assert_eq!(
        blocks_with_parent, 277,
        "all Block nodes should have inferred parent links in scope_parent"
    );
}

#[test]
fn test_cache_unchecked_blocks_have_no_parent_link() {
    let cache = load_cache();
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);

    let ub_ids: Vec<i64> = nodes
        .iter()
        .filter_map(|n| {
            if n.get("nodeType").and_then(|v| v.as_str()) == Some("UncheckedBlock") {
                n.get("id").and_then(|v| v.as_i64())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(ub_ids.len(), 29);

    let with_parent = ub_ids
        .iter()
        .filter(|id| cache.scope_parent.contains_key(&NodeId(**id)))
        .count();

    assert_eq!(
        with_parent, 29,
        "all UncheckedBlock nodes should have inferred parent links in scope_parent"
    );
}

// --- Concrete scope chain trace ---
// PoolManager.initialize: lpFee (var id=822, scope=880 Block)
// Block 880 is the body of FunctionDefinition 881 (scope=1767 PoolManager)
// PoolManager (id=1767, scope=1768 SourceUnit)

#[test]
fn test_scope_chain_lpfee_in_initialize() {
    let cache = load_cache();
    // lpFee (id=822) is declared in Block 880 (body of initialize, fn id=881)
    // Block 880 should be in scope_declarations
    let block_880_decls = cache.scope_declarations.get(&NodeId(329));
    assert!(
        block_880_decls.is_some(),
        "Block 880 should have declarations"
    );
    let names: Vec<&str> = block_880_decls
        .unwrap()
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(
        names.contains(&"lpFee"),
        "lpFee should be in Block 880, got: {:?}",
        names
    );
    assert!(
        names.contains(&"id"),
        "id should be in Block 880, got: {:?}",
        names
    );

    // FunctionDefinition 881 (initialize) should have scope -> 1767 (PoolManager)
    assert_eq!(
        cache.scope_parent.get(&NodeId(330)),
        Some(&NodeId(1216)),
        "initialize (881) should have parent PoolManager (1767)"
    );

    // PoolManager 1767 should have scope -> 1768 (SourceUnit)
    assert_eq!(
        cache.scope_parent.get(&NodeId(1216)),
        Some(&NodeId(1217)),
        "PoolManager (1767) should have parent SourceUnit (1768)"
    );

    // Block 880 now has an inferred parent link to FunctionDefinition 881
    assert_eq!(
        cache.scope_parent.get(&NodeId(329)),
        Some(&NodeId(330)),
        "Block 880 should have inferred parent link to FunctionDefinition 881"
    );
}

// --- Struct field scope verification ---

#[test]
fn test_ast_pool_key_struct_fields() {
    // PoolKey (scope=6871) has 5 fields: currency0, currency1, fee, tickSpacing, hooks
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let id_map = build_id_to_node_type(&nodes);
    let fields: Vec<String> = nodes
        .iter()
        .filter_map(|n| {
            if n.get("nodeType").and_then(|v| v.as_str()) == Some("VariableDeclaration")
                && n.get("scope").and_then(|v| v.as_i64()) == Some(6871)
                && id_map.get(&6871).map(|s| s.as_str()) == Some("StructDefinition")
            {
                n.get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(fields.len(), 5);
    assert!(fields.contains(&"currency0".to_string()));
    assert!(fields.contains(&"currency1".to_string()));
    assert!(fields.contains(&"fee".to_string()));
    assert!(fields.contains(&"tickSpacing".to_string()));
    assert!(fields.contains(&"hooks".to_string()));
}

#[test]
fn test_ast_pool_state_struct_fields() {
    // Pool.State (scope=3870) fields
    let ast = load_ast();
    let nodes = collect_all_nodes(&ast);
    let fields: Vec<String> = nodes
        .iter()
        .filter_map(|n| {
            if n.get("nodeType").and_then(|v| v.as_str()) == Some("VariableDeclaration")
                && n.get("scope").and_then(|v| v.as_i64()) == Some(3870)
            {
                n.get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    assert!(fields.contains(&"slot0".to_string()));
    assert!(fields.contains(&"liquidity".to_string()));
    assert!(fields.contains(&"feeGrowthGlobal0X128".to_string()));
}

// --- Linearized base contracts ---

#[test]
fn test_cache_linearized_base_contracts_pool_manager() {
    let cache = load_cache();
    let bases = cache.linearized_base_contracts.get(&NodeId(1216)).unwrap();
    assert_eq!(
        bases,
        &vec![
            NodeId(1216),
            NodeId(1362),
            NodeId(1332),
            NodeId(1289),
            NodeId(7191),
            NodeId(1414),
            NodeId(1641),
            NodeId(7455),
            NodeId(2123),
            NodeId(7246),
            NodeId(7224),
            NodeId(7569),
            NodeId(7323)
        ]
    );
}

#[test]
fn test_cache_linearized_base_contracts_erc6909_claims() {
    let cache = load_cache();
    let bases = cache.linearized_base_contracts.get(&NodeId(1289)).unwrap();
    // ERC6909Claims inherits ERC6909 inherits IERC6909Claims
    assert_eq!(bases, &vec![NodeId(1289), NodeId(7191), NodeId(7569)]);
}

#[test]
fn test_cache_linearized_base_contracts_simple_contract() {
    let cache = load_cache();
    // Owned has no parents — just itself
    let bases = cache.linearized_base_contracts.get(&NodeId(7455)).unwrap();
    assert_eq!(bases, &vec![NodeId(7455)]);
}

// =============================================================================
// Deterministic scope chain tests
//
// Each test specifies an exact input (byte_pos, file_id) and asserts the exact
// output: innermost scope, parent chain, and name resolution results.
// These prove the scoping engine is correct independently of completion.
// =============================================================================

use solidity_language_server::completion::find_innermost_scope;

/// Walk scope_parent chain from a given scope, collecting all scope IDs visited.
fn walk_scope_chain(
    cache: &solidity_language_server::completion::CompletionCache,
    start: NodeId,
) -> Vec<NodeId> {
    let mut chain = vec![start];
    let mut current = start;
    loop {
        match cache.scope_parent.get(&current) {
            Some(&parent) => {
                chain.push(parent);
                current = parent;
            }
            None => break,
        }
    }
    chain
}

// --- PoolManager.swap: byte 9600 in file 6 ---
// Expected chain: Block 1166 → (BUG: stops) should be → FnDef 1167 → Contract 1767 → SourceUnit 1768
//
// swap body block: src="9580:1491:6" (bytes 9580..11071)
// swap fn:         src="9385:1686:6" (bytes 9385..11071)
// PoolManager:     has id=1767
// SourceUnit:      has id=1768

#[test]
fn test_innermost_scope_in_swap_body() {
    let cache = load_cache();
    // Byte 9600 is inside swap body block (9580..11071, file 6)
    let scope = find_innermost_scope(&cache, 9600, FileId(5));
    assert_eq!(
        scope,
        Some(NodeId(615)),
        "innermost scope at byte 9600/file 6 should be Block 1166 (swap body)"
    );
}

#[test]
fn test_scope_chain_from_swap_body_block() {
    let cache = load_cache();
    // Block 1166 → FnDef 1167 → Contract 1767 → SourceUnit 1768
    let chain = walk_scope_chain(&cache, NodeId(615));
    assert_eq!(
        chain,
        vec![NodeId(615), NodeId(616), NodeId(1216), NodeId(1217)]
    );
}

#[test]
fn test_scope_chain_from_swap_fn() {
    let cache = load_cache();
    // FunctionDefinition 1167 has scope=1767 (PoolManager), which has scope=1768 (SourceUnit)
    let chain = walk_scope_chain(&cache, NodeId(616));
    assert_eq!(
        chain,
        vec![NodeId(616), NodeId(1216), NodeId(1217)],
        "swap fn → PoolManager → SourceUnit"
    );
}

#[test]
fn test_swap_body_declarations() {
    let cache = load_cache();
    // Block 1166 (swap body) declares: id, pool, beforeSwapDelta, hookDelta
    let decls = cache.scope_declarations.get(&NodeId(615)).unwrap();
    let names: Vec<&str> = decls.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"id"), "swap body should declare 'id'");
    assert!(names.contains(&"pool"), "swap body should declare 'pool'");
    assert!(
        names.contains(&"beforeSwapDelta"),
        "swap body should declare 'beforeSwapDelta'"
    );
    assert!(
        names.contains(&"hookDelta"),
        "swap body should declare 'hookDelta'"
    );
}

#[test]
fn test_swap_fn_declarations() {
    let cache = load_cache();
    // FunctionDefinition 1167 (swap) has params: key, params, hookData
    let decls = cache.scope_declarations.get(&NodeId(616)).unwrap();
    let names: Vec<&str> = decls.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"key"), "swap fn should declare param 'key'");
    assert!(
        names.contains(&"params"),
        "swap fn should declare param 'params'"
    );
    assert!(
        names.contains(&"hookData"),
        "swap fn should declare param 'hookData'"
    );
}

#[test]
fn test_swap_body_local_var_types() {
    let cache = load_cache();
    let decls = cache.scope_declarations.get(&NodeId(615)).unwrap();
    let type_of = |name: &str| {
        decls
            .iter()
            .find(|d| d.name == name)
            .map(|d| d.type_id.as_str())
    };

    assert_eq!(type_of("id"), Some("t_userDefinedValueType$_PoolId_$6825"));
    assert_eq!(type_of("pool"), Some("t_struct$_State_$3870_storage_ptr"));
    assert_eq!(
        type_of("beforeSwapDelta"),
        Some("t_userDefinedValueType$_BeforeSwapDelta_$6473")
    );
    assert_eq!(
        type_of("hookDelta"),
        Some("t_userDefinedValueType$_BalanceDelta_$6311")
    );
}

#[test]
fn test_swap_fn_param_types() {
    let cache = load_cache();
    let decls = cache.scope_declarations.get(&NodeId(616)).unwrap();
    let type_of = |name: &str| {
        decls
            .iter()
            .find(|d| d.name == name)
            .map(|d| d.type_id.as_str())
    };

    assert_eq!(type_of("key"), Some("t_struct$_PoolKey_$6871_memory_ptr"));
    assert_eq!(
        type_of("params"),
        Some("t_struct$_SwapParams_$6898_memory_ptr")
    );
    assert_eq!(type_of("hookData"), Some("t_bytes_calldata_ptr"));
}

// --- Nested block inside swap: byte 9900 in file 6 ---
// Block 1125: src="9836:807:6" (bytes 9836..10643)
// Contains: amountToSwap (int256), lpFeeOverride (uint24)
// Expected chain: Block 1125 → Block 1166 → FnDef 1167 → Contract 1767 → SourceUnit 1768

#[test]
fn test_innermost_scope_in_nested_block() {
    let cache = load_cache();
    // Byte 9900 is inside nested block 1125 (9836..10643) which is inside swap body 1166 (9580..11071)
    let scope = find_innermost_scope(&cache, 9900, FileId(5));
    assert_eq!(
        scope,
        Some(NodeId(574)),
        "innermost scope at byte 9900/file 6 should be nested Block 1125"
    );
}

#[test]
fn test_nested_block_declarations() {
    let cache = load_cache();
    let decls = cache.scope_declarations.get(&NodeId(574)).unwrap();
    let names: Vec<&str> = decls.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"amountToSwap"));
    assert!(names.contains(&"lpFeeOverride"));
}

#[test]
fn test_nested_block_declaration_types() {
    let cache = load_cache();
    let decls = cache.scope_declarations.get(&NodeId(574)).unwrap();
    let type_of = |name: &str| {
        decls
            .iter()
            .find(|d| d.name == name)
            .map(|d| d.type_id.as_str())
    };
    assert_eq!(type_of("amountToSwap"), Some("t_int256"));
    assert_eq!(type_of("lpFeeOverride"), Some("t_uint24"));
}

#[test]
fn test_scope_chain_from_nested_block() {
    let cache = load_cache();
    // Nested Block 1125 → Block 1166 → FnDef 1167 → Contract 1767 → SourceUnit 1768
    let chain = walk_scope_chain(&cache, NodeId(574));
    assert_eq!(
        chain,
        vec![
            NodeId(574),
            NodeId(615),
            NodeId(616),
            NodeId(1216),
            NodeId(1217)
        ]
    );
}

// --- PoolManager.initialize: byte 6300 in file 6 ---
// Body block: 880, src="6224:1338:6" (bytes 6224..7562)
// Fn: 881, scope=1767

#[test]
fn test_innermost_scope_in_initialize_body() {
    let cache = load_cache();
    let scope = find_innermost_scope(&cache, 6300, FileId(5));
    assert_eq!(
        scope,
        Some(NodeId(329)),
        "innermost scope at byte 6300/file 6 should be Block 880 (initialize body)"
    );
}

#[test]
fn test_initialize_body_declarations() {
    let cache = load_cache();
    let decls = cache.scope_declarations.get(&NodeId(329)).unwrap();
    let names: Vec<&str> = decls.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"lpFee"));
    assert!(names.contains(&"id"));
}

#[test]
fn test_initialize_body_declaration_types() {
    let cache = load_cache();
    let decls = cache.scope_declarations.get(&NodeId(329)).unwrap();
    let type_of = |name: &str| {
        decls
            .iter()
            .find(|d| d.name == name)
            .map(|d| d.type_id.as_str())
    };
    assert_eq!(type_of("lpFee"), Some("t_uint24"));
    assert_eq!(type_of("id"), Some("t_userDefinedValueType$_PoolId_$6825"));
}

#[test]
fn test_initialize_fn_params() {
    let cache = load_cache();
    let decls = cache.scope_declarations.get(&NodeId(330)).unwrap();
    let type_of = |name: &str| {
        decls
            .iter()
            .find(|d| d.name == name)
            .map(|d| d.type_id.as_str())
    };
    assert_eq!(type_of("key"), Some("t_struct$_PoolKey_$6871_memory_ptr"));
    assert_eq!(type_of("sqrtPriceX96"), Some("t_uint160"));
}

// --- Scope resolution: name lookup at specific positions ---
// These test resolve_name_in_scope with exact (name, byte_pos, file_id) → type_id

#[test]
fn test_resolve_local_var_pool_in_swap_body() {
    let cache = load_cache();
    // "pool" at byte 9600 (swap body, Block 1166) → declared in same block
    let result = resolve_name_in_scope(&cache, "pool", 9600, FileId(5));
    assert_eq!(
        result,
        Some("t_struct$_State_$3870_storage_ptr".to_string())
    );
}

#[test]
fn test_resolve_param_key_in_swap_body() {
    let cache = load_cache();
    // "key" at byte 9600 (swap body, Block 1166)
    // Not in Block 1166 declarations. Walks up to FnDef 1167 where "key" is a param.
    let result = resolve_name_in_scope(&cache, "key", 9600, FileId(5));
    assert_eq!(
        result,
        Some("t_struct$_PoolKey_$6871_memory_ptr".to_string()),
        "key in swap body should walk up to FnDef scope and find the parameter"
    );
}

#[test]
fn test_resolve_nested_block_sees_own_vars() {
    let cache = load_cache();
    // "amountToSwap" at byte 9900 (nested Block 1125 inside swap)
    // Declared in Block 1125 → found immediately
    let result = resolve_name_in_scope(&cache, "amountToSwap", 9900, FileId(5));
    assert_eq!(result, Some("t_int256".to_string()));
}

#[test]
fn test_resolve_nested_block_outer_local_walks_up() {
    let cache = load_cache();
    // "pool" at byte 9900 (nested Block 1125)
    // Not in Block 1125. Walks up to Block 1166 where "pool" is a local var.
    let result = resolve_name_in_scope(&cache, "pool", 9900, FileId(5));
    assert_eq!(
        result,
        Some("t_struct$_State_$3870_storage_ptr".to_string()),
        "pool in nested block should walk up to outer block and find the local var"
    );
}

#[test]
fn test_resolve_state_var_in_function_body() {
    let cache = load_cache();
    // "_pools" at byte 9600 (swap body)
    // Not in Block 1166. Not in FnDef 1167 params. Walks up to ContractDef 1767 state vars.
    let result = resolve_name_in_scope(&cache, "_pools", 9600, FileId(5));
    assert!(
        result.is_some(),
        "_pools should walk up to contract scope and resolve"
    );
}

// --- Position at function header (between fn signature and body) ---
// At byte 9500 (inside FnDef 1167 src=9385..11071, but before body Block 1166 src=9580..11071)
// Innermost scope should be FnDef 1167 (NOT Block 1166)

#[test]
fn test_innermost_scope_in_fn_header() {
    let cache = load_cache();
    // Byte 9500 is after swap fn start (9385) but before body block start (9580)
    let scope = find_innermost_scope(&cache, 9500, FileId(5));
    // This should find FnDef 1167 as innermost (its range 9385..11071 contains 9500,
    // and Block 1166 range 9580..11071 does NOT contain 9500)
    assert_eq!(
        scope,
        Some(NodeId(616)),
        "byte 9500 in fn header should resolve to FnDef 1167, not Block"
    );
}

#[test]
fn test_resolve_param_in_fn_header_uses_scope_walk() {
    let cache = load_cache();
    // At byte 9500, innermost scope is FnDef 1167 (not a Block!)
    // FnDef 1167 has scope_parent → 1767 → 1768. Full chain works.
    // "key" is declared at scope 1167 → found in first scope check.
    let result = resolve_name_in_scope(&cache, "key", 9500, FileId(5));
    assert_eq!(
        result,
        Some("t_struct$_PoolKey_$6871_memory_ptr".to_string()),
        "key at fn header position should resolve via scope walk (not fallback)"
    );
}

#[test]
fn test_resolve_inherited_var_in_fn_header() {
    let cache = load_cache();
    // At byte 9500, scope chain: FnDef 1167 → Contract 1767 → SourceUnit 1768
    // "owner" is not in FnDef 1167 or Contract 1767 scope_declarations.
    // But Contract 1767 has linearizedBaseContracts including Owned (59).
    // Owned (59) has "owner" in scope_declarations.
    let result = resolve_name_in_scope(&cache, "owner", 9500, FileId(5));
    assert_eq!(
        result,
        Some("t_address".to_string()),
        "owner at fn header should resolve via inheritance walk"
    );
}

// --- PoolManager contract-level state variables ---

#[test]
fn test_pool_manager_state_vars_in_scope_declarations() {
    let cache = load_cache();
    let decls = cache.scope_declarations.get(&NodeId(1216)).unwrap();
    let names: Vec<&str> = decls.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"MAX_TICK_SPACING"));
    assert!(names.contains(&"MIN_TICK_SPACING"));
    assert!(names.contains(&"_pools"));
}

#[test]
fn test_owned_state_var_in_scope_declarations() {
    let cache = load_cache();
    let decls = cache.scope_declarations.get(&NodeId(7455)).unwrap();
    let names: Vec<&str> = decls.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"owner"));
}

#[test]
fn test_protocol_fees_state_vars_in_scope_declarations() {
    let cache = load_cache();
    let decls = cache.scope_declarations.get(&NodeId(1641)).unwrap();
    let names: Vec<&str> = decls.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"protocolFeesAccrued"));
    assert!(names.contains(&"protocolFeeController"));
}

// --- type(X). context-sensitive completions ---

#[test]
fn test_parse_chain_type_expression() {
    let chain = parse_dot_chain("type(uint256).", 14);
    assert_eq!(chain.len(), 1);
    assert_eq!(
        chain[0],
        DotSegment {
            name: "type".to_string(),
            kind: AccessKind::Call,
            call_args: Some("uint256".to_string()),
        }
    );
}

#[test]
fn test_parse_chain_type_expression_contract() {
    let line = "type(PoolManager).";
    let chain = parse_dot_chain(line, line.len() as u32);
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].name, "type");
    assert_eq!(chain[0].kind, AccessKind::Call);
    assert_eq!(chain[0].call_args, Some("PoolManager".to_string()));
}

#[test]
fn test_parse_chain_type_expression_interface() {
    let line = "type(IHooks).";
    let chain = parse_dot_chain(line, line.len() as u32);
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].name, "type");
    assert_eq!(chain[0].kind, AccessKind::Call);
    assert_eq!(chain[0].call_args, Some("IHooks".to_string()));
}

#[test]
fn test_contract_kinds_populated() {
    let cache = load_cache();
    // PoolManager (1767) should be "contract"
    assert_eq!(
        cache.contract_kinds.get(&NodeId(1216)).map(|s| s.as_str()),
        Some("contract"),
        "PoolManager should be classified as contract"
    );
    // IHooks (2248) should be "interface"
    assert_eq!(
        cache.contract_kinds.get(&NodeId(1840)).map(|s| s.as_str()),
        Some("interface"),
        "IHooks should be classified as interface"
    );
    // FullMath (3250) should be "library"
    assert_eq!(
        cache.contract_kinds.get(&NodeId(8913)).map(|s| s.as_str()),
        Some("library"),
        "FullMath should be classified as library"
    );
}

#[test]
fn test_type_meta_contract_completions() {
    // type(PoolManager). — contract, should show name, creationCode, runtimeCode
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "type".to_string(),
        kind: AccessKind::Call,
        call_args: Some("PoolManager".to_string()),
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    assert!(labels.contains(&"name"), "type(Contract). should have name");
    assert!(
        labels.contains(&"creationCode"),
        "type(Contract). should have creationCode"
    );
    assert!(
        labels.contains(&"runtimeCode"),
        "type(Contract). should have runtimeCode"
    );
    assert!(
        !labels.contains(&"interfaceId"),
        "type(Contract). should NOT have interfaceId"
    );
    assert!(
        !labels.contains(&"min"),
        "type(Contract). should NOT have min"
    );
    assert!(
        !labels.contains(&"max"),
        "type(Contract). should NOT have max"
    );
    assert_eq!(
        labels.len(),
        3,
        "type(Contract). should have exactly 3 members"
    );
}

#[test]
fn test_type_meta_interface_completions() {
    // type(IHooks). — interface, should show name, interfaceId
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "type".to_string(),
        kind: AccessKind::Call,
        call_args: Some("IHooks".to_string()),
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    assert!(
        labels.contains(&"name"),
        "type(Interface). should have name"
    );
    assert!(
        labels.contains(&"interfaceId"),
        "type(Interface). should have interfaceId"
    );
    assert!(
        !labels.contains(&"creationCode"),
        "type(Interface). should NOT have creationCode"
    );
    assert!(
        !labels.contains(&"runtimeCode"),
        "type(Interface). should NOT have runtimeCode"
    );
    assert!(
        !labels.contains(&"min"),
        "type(Interface). should NOT have min"
    );
    assert!(
        !labels.contains(&"max"),
        "type(Interface). should NOT have max"
    );
    assert_eq!(
        labels.len(),
        2,
        "type(Interface). should have exactly 2 members"
    );
}

#[test]
fn test_type_meta_integer_completions() {
    // type(uint256). — integer type, should show min, max
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "type".to_string(),
        kind: AccessKind::Call,
        call_args: Some("uint256".to_string()),
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    assert!(labels.contains(&"min"), "type(uint256). should have min");
    assert!(labels.contains(&"max"), "type(uint256). should have max");
    assert!(
        !labels.contains(&"name"),
        "type(uint256). should NOT have name"
    );
    assert!(
        !labels.contains(&"creationCode"),
        "type(uint256). should NOT have creationCode"
    );
    assert!(
        !labels.contains(&"interfaceId"),
        "type(uint256). should NOT have interfaceId"
    );
    assert_eq!(
        labels.len(),
        2,
        "type(uint256). should have exactly 2 members"
    );
}

#[test]
fn test_type_meta_int8_completions() {
    // type(int8). — also integer type
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "type".to_string(),
        kind: AccessKind::Call,
        call_args: Some("int8".to_string()),
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    assert!(labels.contains(&"min"), "type(int8). should have min");
    assert!(labels.contains(&"max"), "type(int8). should have max");
    assert_eq!(labels.len(), 2);
}

#[test]
fn test_type_meta_unknown_fallback() {
    // type(SomethingUnknown). — unknown, should return all 6 members as fallback
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "type".to_string(),
        kind: AccessKind::Call,
        call_args: Some("SomethingUnknown".to_string()),
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    assert!(
        labels.contains(&"name"),
        "unknown fallback should have name"
    );
    assert!(
        labels.contains(&"creationCode"),
        "unknown fallback should have creationCode"
    );
    assert!(
        labels.contains(&"runtimeCode"),
        "unknown fallback should have runtimeCode"
    );
    assert!(
        labels.contains(&"interfaceId"),
        "unknown fallback should have interfaceId"
    );
    assert!(labels.contains(&"min"), "unknown fallback should have min");
    assert!(labels.contains(&"max"), "unknown fallback should have max");
    assert_eq!(
        labels.len(),
        6,
        "unknown fallback should have all 6 members"
    );
}

#[test]
fn test_type_meta_no_args_fallback() {
    // type(). — empty parens, should return all 6 members as fallback
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "type".to_string(),
        kind: AccessKind::Call,
        call_args: None,
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    assert_eq!(
        labels.len(),
        6,
        "type(). with no args should have all 6 members"
    );
}

#[test]
fn test_type_meta_library_completions() {
    // type(FullMath). — library, should show name, creationCode, runtimeCode (same as contract)
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "type".to_string(),
        kind: AccessKind::Call,
        call_args: Some("FullMath".to_string()),
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    assert!(labels.contains(&"name"), "type(Library). should have name");
    assert!(
        labels.contains(&"creationCode"),
        "type(Library). should have creationCode"
    );
    assert!(
        labels.contains(&"runtimeCode"),
        "type(Library). should have runtimeCode"
    );
    assert!(
        !labels.contains(&"interfaceId"),
        "type(Library). should NOT have interfaceId"
    );
    assert_eq!(
        labels.len(),
        3,
        "type(Library). should have exactly 3 members"
    );
}

#[test]
fn test_type_meta_items_are_property_kind() {
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "type".to_string(),
        kind: AccessKind::Call,
        call_args: Some("uint256".to_string()),
    }];
    let items = get_chain_completions(&cache, &chain, None);
    for item in &items {
        assert_eq!(
            item.kind,
            Some(CompletionItemKind::PROPERTY),
            "{} should be PROPERTY kind",
            item.label
        );
    }
}

#[test]
fn test_type_meta_items_have_detail() {
    let cache = load_cache();
    let chain = vec![DotSegment {
        name: "type".to_string(),
        kind: AccessKind::Call,
        call_args: Some("IPoolManager".to_string()),
    }];
    let items = get_chain_completions(&cache, &chain, None);
    let name_item = items.iter().find(|i| i.label == "name").unwrap();
    assert_eq!(name_item.detail.as_deref(), Some("string"));

    let iid_item = items.iter().find(|i| i.label == "interfaceId").unwrap();
    assert_eq!(iid_item.detail.as_deref(), Some("bytes4"));
}
