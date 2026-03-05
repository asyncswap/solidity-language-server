//! Gas estimate extraction from solc contract output.
//!
//! Builds lookup tables from `contracts[path][name].contract.evm.gasEstimates`
//! and `contracts[path][name].contract.evm.methodIdentifiers` so that hover,
//! inlay hints, and code lenses can display gas costs.

use serde_json::Value;
use std::collections::HashMap;

use crate::types::{FuncSelector, GasKey, MethodId};

/// Sentinel comment to enable gas estimates for a function.
/// Place `/// @custom:lsp-enable gas-estimates` above a function definition.
/// Also matches the shorter `/// lsp-enable gas-estimates` form.
pub const GAS_SENTINEL: &str = "lsp-enable gas-estimates";

/// Gas estimates for a single contract.
#[derive(Debug, Clone, Default)]
pub struct ContractGas {
    /// Deploy costs: `codeDepositCost`, `executionCost`, `totalCost`.
    pub creation: HashMap<String, String>,
    /// External function gas keyed by 4-byte selector.
    pub external_by_selector: HashMap<FuncSelector, String>,
    /// External function gas keyed by ABI signature (for display).
    pub external_by_sig: HashMap<MethodId, String>,
    /// Internal function gas: signature → gas cost.
    pub internal: HashMap<String, String>,
}

/// All gas estimates indexed by (source_path, contract_name).
pub type GasIndex = HashMap<GasKey, ContractGas>;

/// Build a gas index from normalized AST output.
///
/// The index key is `"path:ContractName"` (e.g. `"src/PoolManager.sol:PoolManager"`).
/// For external functions, gas is also indexed by 4-byte selector for fast lookup
/// from AST nodes that have `functionSelector`.
///
/// Expects the canonical shape: `contracts[path][name] = { abi, evm, ... }`.
pub fn build_gas_index(ast_data: &Value) -> GasIndex {
    let mut index = GasIndex::new();

    let contracts = match ast_data.get("contracts").and_then(|c| c.as_object()) {
        Some(c) => c,
        None => return index,
    };

    for (path, names) in contracts {
        let names_obj = match names.as_object() {
            Some(n) => n,
            None => continue,
        };

        for (name, contract) in names_obj {
            let evm = match contract.get("evm") {
                Some(e) => e,
                None => continue,
            };

            let gas_estimates = match evm.get("gasEstimates") {
                Some(g) => g,
                None => continue,
            };

            let mut contract_gas = ContractGas::default();

            // Creation costs
            if let Some(creation) = gas_estimates.get("creation").and_then(|c| c.as_object()) {
                for (key, value) in creation {
                    let cost = value.as_str().unwrap_or("").to_string();
                    contract_gas.creation.insert(key.clone(), cost);
                }
            }

            // External function gas — also build selector → gas mapping
            let method_ids = evm.get("methodIdentifiers").and_then(|m| m.as_object());

            if let Some(external) = gas_estimates.get("external").and_then(|e| e.as_object()) {
                // Build signature → selector reverse map
                let sig_to_selector: HashMap<&str, &str> = method_ids
                    .map(|mi| {
                        mi.iter()
                            .filter_map(|(sig, sel)| sel.as_str().map(|s| (sig.as_str(), s)))
                            .collect()
                    })
                    .unwrap_or_default();

                for (sig, value) in external {
                    let cost = value.as_str().unwrap_or("").to_string();
                    // Store by selector for fast AST node lookup
                    if let Some(selector) = sig_to_selector.get(sig.as_str()) {
                        contract_gas
                            .external_by_selector
                            .insert(FuncSelector::new(*selector), cost.clone());
                    }
                    // Also store by signature for display
                    contract_gas
                        .external_by_sig
                        .insert(MethodId::new(sig.clone()), cost);
                }
            }

            // Internal function gas
            if let Some(internal) = gas_estimates.get("internal").and_then(|i| i.as_object()) {
                for (sig, value) in internal {
                    let cost = value.as_str().unwrap_or("").to_string();
                    contract_gas.internal.insert(sig.clone(), cost);
                }
            }

            let key = GasKey::from_parts(path, name);
            index.insert(key, contract_gas);
        }
    }

    index
}

/// Look up gas cost for a function by its [`FuncSelector`] (external functions).
pub fn gas_by_selector<'a>(
    index: &'a GasIndex,
    selector: &FuncSelector,
) -> Option<(&'a str, &'a str)> {
    for (contract_key, gas) in index {
        if let Some(cost) = gas.external_by_selector.get(selector) {
            return Some((contract_key.as_str(), cost.as_str()));
        }
    }
    None
}

/// Look up gas cost for an internal function by name.
///
/// Matches if the gas estimate key starts with `name(`.
pub fn gas_by_name<'a>(index: &'a GasIndex, name: &str) -> Vec<(&'a str, &'a str, &'a str)> {
    let prefix = format!("{name}(");
    let mut results = Vec::new();
    for (contract_key, gas) in index {
        for (sig, cost) in &gas.internal {
            if sig.starts_with(&prefix) {
                results.push((contract_key.as_str(), sig.as_str(), cost.as_str()));
            }
        }
    }
    results
}

/// Look up creation/deploy gas for a contract.
pub fn gas_for_contract<'a>(
    index: &'a GasIndex,
    path: &str,
    name: &str,
) -> Option<&'a ContractGas> {
    let key = GasKey::from_parts(path, name);
    index.get(&key)
}

/// Resolve the gas index key for a declaration node.
///
/// Typed version of `resolve_contract_key` using `DeclNode` and typed indices.
///
/// Resolves a declaration node to its `"{abs_path}:{contract_name}"` gas index key
/// using typed field access instead of raw `Value` chains.
pub fn resolve_contract_key_typed(
    decl: &crate::solc_ast::DeclNode,
    index: &GasIndex,
    decl_index: &HashMap<crate::types::NodeId, crate::solc_ast::DeclNode>,
    node_id_to_source_path: &HashMap<crate::types::NodeId, crate::types::AbsPath>,
) -> Option<GasKey> {
    use crate::solc_ast::DeclNode;

    // Get the contract name and source unit scope
    let (contract_name, source_unit_scope) = match decl {
        DeclNode::ContractDefinition(c) => (c.name.as_str(), crate::types::NodeId(c.scope?)),
        _ => {
            // Walk up to containing contract via scope
            let scope_id = crate::types::NodeId(decl.scope()?);
            let scope_decl = decl_index.get(&scope_id)?;
            let contract = match scope_decl {
                DeclNode::ContractDefinition(c) => c,
                _ => return None,
            };
            (
                contract.name.as_str(),
                crate::types::NodeId(contract.scope?),
            )
        }
    };

    // Find the absolute path via node_id_to_source_path (O(1) lookup)
    let abs_path = node_id_to_source_path.get(&source_unit_scope)?;

    // Build the exact key
    let exact_key = GasKey::from_parts(abs_path.as_str(), contract_name);
    if index.contains_key(&exact_key) {
        return Some(exact_key);
    }

    // Fallback: match by suffix
    let file_name = std::path::Path::new(abs_path.as_str())
        .file_name()?
        .to_str()?;
    let suffix = format!("{file_name}:{contract_name}");
    index
        .keys()
        .find(|k| k.as_str().ends_with(&suffix))
        .cloned()
}

/// Format a gas cost for display.
/// Numbers get comma-separated (e.g. "6924600" → "6,924,600").
/// "infinite" stays as-is.
pub fn format_gas(cost: &str) -> String {
    if cost == "infinite" {
        return "infinite".to_string();
    }
    // Try to parse as number and format with commas
    if let Ok(n) = cost.parse::<u64>() {
        let s = n.to_string();
        let mut result = String::new();
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        result.chars().rev().collect()
    } else {
        cost.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Load poolmanager.json (raw solc output) and normalize to canonical shape.
    fn load_solc_fixture() -> Value {
        let data = std::fs::read_to_string("poolmanager.json").expect("test fixture");
        let raw: Value = serde_json::from_str(&data).expect("valid json");
        crate::solc::normalize_solc_output(raw, None)
    }

    #[test]
    fn test_format_gas_number() {
        assert_eq!(format_gas("109"), "109");
        assert_eq!(format_gas("2595"), "2,595");
        assert_eq!(format_gas("6924600"), "6,924,600");
        assert_eq!(format_gas("28088"), "28,088");
    }

    #[test]
    fn test_format_gas_infinite() {
        assert_eq!(format_gas("infinite"), "infinite");
    }

    #[test]
    fn test_format_gas_unknown() {
        assert_eq!(format_gas("unknown"), "unknown");
    }

    #[test]
    fn test_build_gas_index_empty() {
        let data = json!({});
        let index = build_gas_index(&data);
        assert!(index.is_empty());
    }

    #[test]
    fn test_build_gas_index_no_contracts() {
        let data = json!({ "sources": {}, "contracts": {} });
        let index = build_gas_index(&data);
        assert!(index.is_empty());
    }

    #[test]
    fn test_build_gas_index_basic() {
        let data = json!({
            "contracts": {
                "src/Foo.sol": {
                    "Foo": {
                        "evm": {
                            "gasEstimates": {
                                "creation": {
                                    "codeDepositCost": "200",
                                    "executionCost": "infinite",
                                    "totalCost": "infinite"
                                },
                                "external": {
                                    "bar(uint256)": "109"
                                },
                                "internal": {
                                    "_baz(uint256)": "50"
                                }
                            },
                            "methodIdentifiers": {
                                "bar(uint256)": "abcd1234"
                            }
                        }
                    }
                }
            }
        });

        let index = build_gas_index(&data);
        assert_eq!(index.len(), 1);

        let foo = index.get("src/Foo.sol:Foo").unwrap();

        // Creation
        assert_eq!(foo.creation.get("codeDepositCost").unwrap(), "200");
        assert_eq!(foo.creation.get("executionCost").unwrap(), "infinite");

        // External — by selector
        assert_eq!(
            foo.external_by_selector
                .get(&FuncSelector::new("abcd1234"))
                .unwrap(),
            "109"
        );
        // External — by signature
        assert_eq!(
            foo.external_by_sig
                .get(&MethodId::new("bar(uint256)"))
                .unwrap(),
            "109"
        );

        // Internal
        assert_eq!(foo.internal.get("_baz(uint256)").unwrap(), "50");
    }

    #[test]
    fn test_gas_by_selector() {
        let data = json!({
            "contracts": {
                "src/Foo.sol": {
                    "Foo": {
                        "evm": {
                            "gasEstimates": {
                                "external": { "bar(uint256)": "109" }
                            },
                            "methodIdentifiers": {
                                "bar(uint256)": "abcd1234"
                            }
                        }
                    }
                }
            }
        });

        let index = build_gas_index(&data);
        let (contract, cost) = gas_by_selector(&index, &FuncSelector::new("abcd1234")).unwrap();
        assert_eq!(contract, "src/Foo.sol:Foo");
        assert_eq!(cost, "109");
    }

    #[test]
    fn test_gas_by_name() {
        let data = json!({
            "contracts": {
                "src/Foo.sol": {
                    "Foo": {
                        "evm": {
                            "gasEstimates": {
                                "internal": {
                                    "_baz(uint256)": "50",
                                    "_baz(uint256,address)": "120"
                                }
                            }
                        }
                    }
                }
            }
        });

        let index = build_gas_index(&data);
        let results = gas_by_name(&index, "_baz");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_gas_for_contract() {
        let data = json!({
            "contracts": {
                "src/Foo.sol": {
                    "Foo": {
                        "evm": {
                            "gasEstimates": {
                                "creation": {
                                    "codeDepositCost": "6924600"
                                }
                            }
                        }
                    }
                }
            }
        });

        let index = build_gas_index(&data);
        let gas = gas_for_contract(&index, "src/Foo.sol", "Foo").unwrap();
        assert_eq!(gas.creation.get("codeDepositCost").unwrap(), "6924600");
    }

    #[test]
    fn test_build_gas_index_from_solc_fixture() {
        let ast = load_solc_fixture();
        let index = build_gas_index(&ast);

        // poolmanager.json has gas estimates for PoolManager
        assert!(!index.is_empty(), "solc fixture should have gas data");

        // Find PoolManager — keys have absolute paths
        let pm_key = index
            .keys()
            .find(|k| k.contains("PoolManager.sol:PoolManager"))
            .expect("should have PoolManager gas data");

        let pm = index.get(pm_key).unwrap();

        // Creation costs
        assert!(
            pm.creation.contains_key("codeDepositCost"),
            "should have codeDepositCost"
        );
        assert!(
            pm.creation.contains_key("executionCost"),
            "should have executionCost"
        );
        assert!(
            pm.creation.contains_key("totalCost"),
            "should have totalCost"
        );

        // External functions
        assert!(
            !pm.external_by_selector.is_empty(),
            "should have external function gas estimates"
        );

        // Internal functions
        assert!(
            !pm.internal.is_empty(),
            "should have internal function gas estimates"
        );
    }

    #[test]
    fn test_gas_by_selector_from_solc_fixture() {
        let ast = load_solc_fixture();
        let index = build_gas_index(&ast);

        // owner() has selector "8da5cb5b" (well-known)
        let result = gas_by_selector(&index, &FuncSelector::new("8da5cb5b"));
        assert!(result.is_some(), "should find owner() by selector");
        let (contract, cost) = result.unwrap();
        assert!(
            contract.contains("PoolManager"),
            "should be PoolManager contract"
        );
        assert!(!cost.is_empty(), "should have a gas cost");
    }

    #[test]
    fn test_gas_by_name_from_solc_fixture() {
        let ast = load_solc_fixture();
        let index = build_gas_index(&ast);

        // _getPool is an internal function in PoolManager
        let results = gas_by_name(&index, "_getPool");
        assert!(!results.is_empty(), "should find _getPool internal gas");
    }

    #[test]
    fn test_gas_for_contract_from_solc_fixture() {
        let ast = load_solc_fixture();
        let index = build_gas_index(&ast);

        // Find the PoolManager key
        let pm_key = index
            .keys()
            .find(|k| k.contains("PoolManager.sol:PoolManager"))
            .expect("should have PoolManager");

        // Parse the path and name from "path:Name"
        let parts: Vec<&str> = pm_key.rsplitn(2, ':').collect();
        let name = parts[0];
        let path = parts[1];

        let gas = gas_for_contract(&index, path, name);
        assert!(gas.is_some(), "should find PoolManager contract gas");
        assert_eq!(
            gas.unwrap().creation.get("executionCost").unwrap(),
            "infinite"
        );
    }
}
