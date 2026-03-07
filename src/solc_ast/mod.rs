//! Typed Solidity AST deserialized from solc's JSON output.
//!
//! This module provides Rust structs that mirror the AST node types emitted
//! by the Solidity compiler (`solc --standard-json`), plus a lightweight
//! declaration extraction function ([`extract_decl_nodes`]) that avoids
//! deserializing the full AST.
//!
//! # Design
//!
//! - All structs derive `serde::Deserialize` to parse directly from solc JSON.
//! - Fields use `Option<T>` liberally for cross-version compatibility.
//! - An `AstVisitor` trait (gated behind `#[cfg(test)]`) follows the official
//!   C++ `ASTConstVisitor` pattern for test traversal.

pub mod contracts;
pub mod enums;
pub mod events;
pub mod expressions;
pub mod functions;
pub mod source_units;
pub mod statements;
pub mod types;
pub mod variables;
#[cfg(test)]
pub mod visitor;
pub mod yul;

// Re-export everything for convenient access via `use crate::solc_ast::*`.
pub use contracts::*;
pub use enums::*;
pub use events::*;
pub use expressions::*;
pub use functions::*;
pub use source_units::*;
pub use statements::*;
pub use types::*;
pub use variables::*;
#[cfg(test)]
pub use visitor::*;
pub use yul::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Typed declaration node ────────────────────────────────────────────────

/// A reference to any declaration-level AST node.
///
/// This enum covers the node types that hover, goto, and references care
/// about: functions, variables, contracts, events, errors, structs, enums,
/// modifiers, and user-defined value types.
///
/// Built by [`extract_decl_nodes`] and stored in `CachedBuild` for O(1)
/// typed node lookup by ID.
#[derive(Clone, Debug)]
pub enum DeclNode {
    FunctionDefinition(FunctionDefinition),
    VariableDeclaration(VariableDeclaration),
    ContractDefinition(ContractDefinition),
    EventDefinition(EventDefinition),
    ErrorDefinition(ErrorDefinition),
    StructDefinition(StructDefinition),
    EnumDefinition(EnumDefinition),
    ModifierDefinition(ModifierDefinition),
    UserDefinedValueTypeDefinition(UserDefinedValueTypeDefinition),
}

impl DeclNode {
    /// Node ID of the declaration.
    pub fn id(&self) -> NodeID {
        match self {
            Self::FunctionDefinition(n) => n.id,
            Self::VariableDeclaration(n) => n.id,
            Self::ContractDefinition(n) => n.id,
            Self::EventDefinition(n) => n.id,
            Self::ErrorDefinition(n) => n.id,
            Self::StructDefinition(n) => n.id,
            Self::EnumDefinition(n) => n.id,
            Self::ModifierDefinition(n) => n.id,
            Self::UserDefinedValueTypeDefinition(n) => n.id,
        }
    }

    /// Name of the declaration.
    pub fn name(&self) -> &str {
        match self {
            Self::FunctionDefinition(n) => &n.name,
            Self::VariableDeclaration(n) => &n.name,
            Self::ContractDefinition(n) => &n.name,
            Self::EventDefinition(n) => &n.name,
            Self::ErrorDefinition(n) => &n.name,
            Self::StructDefinition(n) => &n.name,
            Self::EnumDefinition(n) => &n.name,
            Self::ModifierDefinition(n) => &n.name,
            Self::UserDefinedValueTypeDefinition(n) => &n.name,
        }
    }

    /// Source location string (`"offset:length:fileId"`).
    pub fn src(&self) -> &str {
        match self {
            Self::FunctionDefinition(n) => &n.src,
            Self::VariableDeclaration(n) => &n.src,
            Self::ContractDefinition(n) => &n.src,
            Self::EventDefinition(n) => &n.src,
            Self::ErrorDefinition(n) => &n.src,
            Self::StructDefinition(n) => &n.src,
            Self::EnumDefinition(n) => &n.src,
            Self::ModifierDefinition(n) => &n.src,
            Self::UserDefinedValueTypeDefinition(n) => &n.src,
        }
    }

    /// Scope (parent) node ID, if available.
    pub fn scope(&self) -> Option<NodeID> {
        match self {
            Self::FunctionDefinition(n) => n.scope,
            Self::VariableDeclaration(n) => n.scope,
            Self::ContractDefinition(n) => n.scope,
            Self::EventDefinition(_) => None,
            Self::ErrorDefinition(_) => None,
            Self::StructDefinition(n) => n.scope,
            Self::EnumDefinition(_) => None,
            Self::ModifierDefinition(_) => None,
            Self::UserDefinedValueTypeDefinition(_) => None,
        }
    }

    /// Documentation attached to the declaration.
    pub fn documentation(&self) -> Option<&Documentation> {
        match self {
            Self::FunctionDefinition(n) => n.documentation.as_ref(),
            Self::VariableDeclaration(n) => n.documentation.as_ref(),
            Self::ContractDefinition(n) => n.documentation.as_ref(),
            Self::EventDefinition(n) => n.documentation.as_ref(),
            Self::ErrorDefinition(n) => n.documentation.as_ref(),
            Self::StructDefinition(n) => n.documentation.as_ref(),
            Self::EnumDefinition(n) => n.documentation.as_ref(),
            Self::ModifierDefinition(n) => n.documentation.as_ref(),
            Self::UserDefinedValueTypeDefinition(_) => None,
        }
    }

    /// Function selector (4-byte hex for functions/errors, 32-byte for events).
    pub fn selector(&self) -> Option<&str> {
        match self {
            Self::FunctionDefinition(n) => n.function_selector.as_deref(),
            Self::VariableDeclaration(n) => n.function_selector.as_deref(),
            Self::EventDefinition(n) => n.event_selector.as_deref(),
            Self::ErrorDefinition(n) => n.error_selector.as_deref(),
            _ => None,
        }
    }

    /// Extract a typed [`Selector`] from this declaration.
    ///
    /// Returns `Selector::Func` for functions, public variables, and errors,
    /// or `Selector::Event` for events. Equivalent to `extract_selector(&Value)`.
    pub fn extract_typed_selector(&self) -> Option<crate::types::Selector> {
        match self {
            Self::FunctionDefinition(n) => n
                .function_selector
                .as_deref()
                .map(|s| crate::types::Selector::Func(crate::types::FuncSelector::new(s))),
            Self::VariableDeclaration(n) => n
                .function_selector
                .as_deref()
                .map(|s| crate::types::Selector::Func(crate::types::FuncSelector::new(s))),
            Self::ErrorDefinition(n) => n
                .error_selector
                .as_deref()
                .map(|s| crate::types::Selector::Func(crate::types::FuncSelector::new(s))),
            Self::EventDefinition(n) => n
                .event_selector
                .as_deref()
                .map(|s| crate::types::Selector::Event(crate::types::EventSelector::new(s))),
            _ => None,
        }
    }

    /// Extract documentation text from this declaration.
    ///
    /// Returns the NatSpec text string, equivalent to the raw `extract_documentation()`.
    pub fn extract_doc_text(&self) -> Option<String> {
        self.documentation().map(|doc| match doc {
            Documentation::String(s) => s.clone(),
            Documentation::Structured(s) => s.text.clone(),
        })
    }

    /// Build a Solidity-style signature string for this declaration.
    ///
    /// Typed equivalent of `build_function_signature(&Value)` in `hover.rs`.
    /// Uses direct field access instead of `.get("field").and_then(|v| v.as_str())` chains.
    pub fn build_signature(&self) -> Option<String> {
        match self {
            Self::FunctionDefinition(n) => {
                let params = format_params_typed(&n.parameters);
                let returns = format_params_typed(&n.return_parameters);

                let mut sig = match n.kind {
                    FunctionKind::Constructor => format!("constructor({params})"),
                    FunctionKind::Receive => "receive() external payable".to_string(),
                    FunctionKind::Fallback => format!("fallback({params})"),
                    _ => format!("function {}({params})", n.name),
                };

                // Add visibility (skip for constructor and receive)
                if !matches!(n.kind, FunctionKind::Constructor | FunctionKind::Receive)
                    && let Some(vis) = &n.visibility
                {
                    let vis_str = vis.to_string();
                    if !vis_str.is_empty() {
                        sig.push_str(&format!(" {vis_str}"));
                    }
                }

                // Add state mutability (skip "nonpayable")
                if !matches!(n.state_mutability, StateMutability::Nonpayable) {
                    sig.push_str(&format!(" {}", n.state_mutability));
                }

                if !returns.is_empty() {
                    sig.push_str(&format!(" returns ({returns})"));
                }
                Some(sig)
            }
            Self::ModifierDefinition(n) => {
                let params = format_params_typed(&n.parameters);
                Some(format!("modifier {}({params})", n.name))
            }
            Self::EventDefinition(n) => {
                let params = format_params_typed(&n.parameters);
                Some(format!("event {}({params})", n.name))
            }
            Self::ErrorDefinition(n) => {
                let params = format_params_typed(&n.parameters);
                Some(format!("error {}({params})", n.name))
            }
            Self::VariableDeclaration(n) => {
                let type_str = n
                    .type_descriptions
                    .type_string
                    .as_deref()
                    .unwrap_or("unknown");
                let vis = n
                    .visibility
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                let mut sig = type_str.to_string();
                if !vis.is_empty() {
                    sig.push_str(&format!(" {vis}"));
                }
                match &n.mutability {
                    Some(Mutability::Constant) => sig.push_str(" constant"),
                    Some(Mutability::Immutable) => sig.push_str(" immutable"),
                    _ => {}
                }
                sig.push_str(&format!(" {}", n.name));
                Some(sig)
            }
            Self::ContractDefinition(n) => {
                let mut sig = format!("{} {}", n.contract_kind, n.name);

                if !n.base_contracts.is_empty() {
                    let base_names: Vec<&str> = n
                        .base_contracts
                        .iter()
                        .map(|b| b.base_name.name.as_str())
                        .collect();
                    if !base_names.is_empty() {
                        sig.push_str(&format!(" is {}", base_names.join(", ")));
                    }
                }
                Some(sig)
            }
            Self::StructDefinition(n) => {
                let mut sig = format!("struct {} {{\n", n.name);
                for member in &n.members {
                    let mtype = member
                        .type_descriptions
                        .type_string
                        .as_deref()
                        .unwrap_or("?");
                    sig.push_str(&format!("    {mtype} {};\n", member.name));
                }
                sig.push('}');
                Some(sig)
            }
            Self::EnumDefinition(n) => {
                let mut sig = format!("enum {} {{\n", n.name);
                for member in &n.members {
                    sig.push_str(&format!("    {},\n", member.name));
                }
                sig.push('}');
                Some(sig)
            }
            Self::UserDefinedValueTypeDefinition(n) => {
                let underlying = type_name_to_str(&n.underlying_type);
                Some(format!("type {} is {underlying}", n.name))
            }
        }
    }

    /// Build individual parameter strings for signature help.
    ///
    /// Returns a vec of strings like `["uint256 amount", "uint16 tax"]`.
    /// Typed equivalent of `build_parameter_strings(&Value)` in `hover.rs`.
    pub fn param_strings(&self) -> Vec<String> {
        match self {
            Self::FunctionDefinition(n) => build_param_strings_typed(&n.parameters),
            Self::ModifierDefinition(n) => build_param_strings_typed(&n.parameters),
            Self::EventDefinition(n) => build_param_strings_typed(&n.parameters),
            Self::ErrorDefinition(n) => build_param_strings_typed(&n.parameters),
            _ => Vec::new(),
        }
    }

    /// Returns the `ParameterList` for this declaration's parameters, if any.
    pub fn parameters(&self) -> Option<&ParameterList> {
        match self {
            Self::FunctionDefinition(n) => Some(&n.parameters),
            Self::ModifierDefinition(n) => Some(&n.parameters),
            Self::EventDefinition(n) => Some(&n.parameters),
            Self::ErrorDefinition(n) => Some(&n.parameters),
            _ => None,
        }
    }

    /// Returns the `ParameterList` for this declaration's return parameters, if any.
    pub fn return_parameters(&self) -> Option<&ParameterList> {
        match self {
            Self::FunctionDefinition(n) => Some(&n.return_parameters),
            _ => None,
        }
    }

    /// Returns the node type string, matching the JSON `nodeType` field.
    pub fn node_type(&self) -> &'static str {
        match self {
            Self::FunctionDefinition(_) => "FunctionDefinition",
            Self::VariableDeclaration(_) => "VariableDeclaration",
            Self::ContractDefinition(_) => "ContractDefinition",
            Self::EventDefinition(_) => "EventDefinition",
            Self::ErrorDefinition(_) => "ErrorDefinition",
            Self::StructDefinition(_) => "StructDefinition",
            Self::EnumDefinition(_) => "EnumDefinition",
            Self::ModifierDefinition(_) => "ModifierDefinition",
            Self::UserDefinedValueTypeDefinition(_) => "UserDefinedValueTypeDefinition",
        }
    }

    /// Returns the type description string for this declaration, if available.
    pub fn type_string(&self) -> Option<&str> {
        match self {
            Self::VariableDeclaration(n) => n.type_descriptions.type_string.as_deref(),
            _ => None,
        }
    }

    /// Returns parameter/member names for inlay hint resolution.
    ///
    /// For functions, events, errors, modifiers: returns `parameters.parameters[].name`.
    /// For structs: returns `members[].name`.
    /// Typed equivalent of `get_parameter_names(&Value)` in `inlay_hints.rs`.
    pub fn param_names(&self) -> Option<Vec<String>> {
        match self {
            Self::FunctionDefinition(n) => Some(
                n.parameters
                    .parameters
                    .iter()
                    .map(|p| p.name.clone())
                    .collect(),
            ),
            Self::ModifierDefinition(n) => Some(
                n.parameters
                    .parameters
                    .iter()
                    .map(|p| p.name.clone())
                    .collect(),
            ),
            Self::EventDefinition(n) => Some(
                n.parameters
                    .parameters
                    .iter()
                    .map(|p| p.name.clone())
                    .collect(),
            ),
            Self::ErrorDefinition(n) => Some(
                n.parameters
                    .parameters
                    .iter()
                    .map(|p| p.name.clone())
                    .collect(),
            ),
            Self::StructDefinition(n) => Some(n.members.iter().map(|m| m.name.clone()).collect()),
            _ => None,
        }
    }

    /// For constructors, returns true if this is a constructor function.
    pub fn is_constructor(&self) -> bool {
        matches!(self, Self::FunctionDefinition(n) if matches!(n.kind, crate::solc_ast::enums::FunctionKind::Constructor))
    }
}

// ── Typed formatting helpers ──────────────────────────────────────────────────

/// Format a typed `ParameterList` into a comma-separated parameter string.
///
/// Typed equivalent of `format_parameters(Option<&Value>)` in `hover.rs`.
pub fn format_params_typed(params: &ParameterList) -> String {
    let parts: Vec<String> = params
        .parameters
        .iter()
        .map(|p| {
            let type_str = p.type_descriptions.type_string.as_deref().unwrap_or("?");
            let name = &p.name;
            let storage = p
                .storage_location
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "default".to_string());

            if name.is_empty() {
                type_str.to_string()
            } else if storage != "default" {
                format!("{type_str} {storage} {name}")
            } else {
                format!("{type_str} {name}")
            }
        })
        .collect();

    parts.join(", ")
}

/// Build individual parameter strings from a typed `ParameterList`.
///
/// Returns a vec of strings like `["uint256 amount", "uint16 tax"]`.
fn build_param_strings_typed(params: &ParameterList) -> Vec<String> {
    params
        .parameters
        .iter()
        .map(|p| {
            let type_str = p.type_descriptions.type_string.as_deref().unwrap_or("?");
            let name = &p.name;
            let storage = p
                .storage_location
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "default".to_string());

            if name.is_empty() {
                type_str.to_string()
            } else if storage != "default" {
                format!("{type_str} {storage} {name}")
            } else {
                format!("{type_str} {name}")
            }
        })
        .collect()
}

/// Extract a human-readable type string from a `TypeName` node.
pub fn type_name_to_str(tn: &TypeName) -> &str {
    match tn {
        TypeName::ElementaryTypeName(e) => e
            .type_descriptions
            .type_string
            .as_deref()
            .unwrap_or(&e.name),
        TypeName::UserDefinedTypeName(u) => u
            .type_descriptions
            .type_string
            .as_deref()
            .unwrap_or("unknown"),
        TypeName::FunctionTypeName(f) => f
            .type_descriptions
            .type_string
            .as_deref()
            .unwrap_or("function"),
        TypeName::Mapping(m) => m
            .type_descriptions
            .type_string
            .as_deref()
            .unwrap_or("mapping"),
        TypeName::ArrayTypeName(a) => a
            .type_descriptions
            .type_string
            .as_deref()
            .unwrap_or("array"),
    }
}

// ── Declaration-only extraction from raw Value ────────────────────────────

/// The 9 declaration `nodeType` strings we care about.
const DECL_NODE_TYPES: &[&str] = &[
    "FunctionDefinition",
    "VariableDeclaration",
    "ContractDefinition",
    "EventDefinition",
    "ErrorDefinition",
    "StructDefinition",
    "EnumDefinition",
    "ModifierDefinition",
    "UserDefinedValueTypeDefinition",
];

/// Fields to strip from declaration nodes before deserializing.
/// These contain large AST subtrees (function bodies, expressions, etc.)
/// that are never read through `DeclNode`.
const STRIP_FIELDS: &[&str] = &[
    "body",
    "modifiers",
    "value",
    "overrides",
    "baseFunctions",
    "baseModifiers",
    "nameLocation",
    "implemented",
    "isVirtual",
    "abstract",
    "contractDependencies",
    "usedErrors",
    "usedEvents",
    "fullyImplemented",
    "linearizedBaseContracts",
    "canonicalName",
    "constant",
    "indexed",
];

/// Fields to strip from children inside `ContractDefinition.nodes`.
/// Same as `STRIP_FIELDS` — each contract child also has bodies, values, etc.
const STRIP_CHILD_FIELDS: &[&str] = &[
    "body",
    "modifiers",
    "value",
    "overrides",
    "baseFunctions",
    "baseModifiers",
    "nameLocation",
    "implemented",
    "isVirtual",
    "constant",
    "indexed",
    "canonicalName",
];

/// Result of extracting declaration nodes from the raw sources Value.
pub struct ExtractedDecls {
    /// Declaration index: node ID → typed `DeclNode`.
    pub decl_index: HashMap<NodeID, DeclNode>,
    /// Reverse index: node ID → source file path.
    pub node_id_to_source_path: HashMap<NodeID, String>,
}

/// Extract declaration nodes directly from the raw `sources` section of solc output.
///
/// Instead of deserializing the entire typed AST (SourceUnit, all expressions,
/// statements, Yul blocks), this walks the raw JSON Value tree and only
/// deserializes nodes whose `nodeType` matches one of the 9 declaration types.
///
/// Heavy fields (`body`, `modifiers`, `value`, etc.) are stripped from the
/// Value **before** deserialization, so function bodies are never parsed.
///
/// For `ContractDefinition`, the `nodes` array is preserved (needed for
/// `resolve_inheritdoc_typed`) but each child node within it also has its
/// heavy fields stripped.
///
/// This eliminates:
/// - The full `SourceUnit` deserialization per source file
/// - The `ast_node.clone()` per source file (~80 MB for large projects)
/// - All transient expression/statement/yul parsing (~40 MB)
pub fn extract_decl_nodes(sources: &serde_json::Value) -> Option<ExtractedDecls> {
    let sources_obj = sources.as_object()?;
    // Pre-size based on source count. Typical project averages ~30 decl nodes
    // per source file and ~30 id-to-path entries per source file.
    let source_count = sources_obj.len();
    let mut decl_index = HashMap::with_capacity(source_count * 32);
    let mut id_to_path = HashMap::with_capacity(source_count * 32);

    for (path, source_data) in sources_obj {
        let ast_node = source_data.get("ast")?;

        // Record the source unit id
        if let Some(su_id) = ast_node.get("id").and_then(|v| v.as_i64()) {
            id_to_path.insert(su_id, path.clone());
        }

        // Walk the top-level `nodes` array of the SourceUnit
        if let Some(nodes) = ast_node.get("nodes").and_then(|v| v.as_array()) {
            for node in nodes {
                walk_and_extract(node, path, &mut decl_index, &mut id_to_path);
            }
        }
    }

    Some(ExtractedDecls {
        decl_index,
        node_id_to_source_path: id_to_path,
    })
}

/// Recursively walk a JSON node, extracting declaration nodes.
fn walk_and_extract(
    node: &serde_json::Value,
    source_path: &str,
    decl_index: &mut HashMap<NodeID, DeclNode>,
    id_to_path: &mut HashMap<NodeID, String>,
) {
    let obj = match node.as_object() {
        Some(o) => o,
        None => return,
    };

    let node_type = match obj.get("nodeType").and_then(|v| v.as_str()) {
        Some(nt) => nt,
        None => return,
    };

    let node_id = obj.get("id").and_then(|v| v.as_i64());

    // Record id → path for all nodes that have an id
    if let Some(id) = node_id {
        id_to_path.insert(id, source_path.to_string());
    }

    // Check if this is a declaration node type
    if DECL_NODE_TYPES.contains(&node_type)
        && let Some(id) = node_id
    {
        // Build a filtered Value from the borrowed node, copying only
        // the fields needed for deserialization (skips body, modifiers, value, etc.).
        // This avoids cloning the entire node (previously ~117 MB of transient churn).
        let node_value = if node_type == "ContractDefinition" {
            build_filtered_contract(obj)
        } else {
            build_filtered_decl(obj)
        };

        // Deserialize the filtered node into the typed struct
        if let Some(decl) = deserialize_decl_node(node_type, node_value) {
            decl_index.insert(id, decl);
        }
    }

    // Recurse into children — ContractDefinition has `nodes`, SourceUnit has `nodes`.
    // We also need to recurse into `parameters`, `returnParameters`, and `members`
    // to find nested VariableDeclaration nodes (function params, return params,
    // error/event params, struct members).
    if let Some(children) = obj.get("nodes").and_then(|v| v.as_array()) {
        for child in children {
            walk_and_extract(child, source_path, decl_index, id_to_path);
        }
    }

    // Walk into ParameterList.parameters arrays to capture individual
    // VariableDeclaration nodes for params/returns
    for param_key in &["parameters", "returnParameters"] {
        if let Some(param_list) = obj.get(*param_key).and_then(|v| v.as_object()) {
            // ParameterList has id and a `parameters` array
            if let Some(pl_id) = param_list.get("id").and_then(|v| v.as_i64()) {
                id_to_path.insert(pl_id, source_path.to_string());
            }
            if let Some(params) = param_list.get("parameters").and_then(|v| v.as_array()) {
                for param in params {
                    walk_and_extract(param, source_path, decl_index, id_to_path);
                }
            }
        }
    }

    // Walk into struct `members` to capture member VariableDeclarations
    if let Some(members) = obj.get("members").and_then(|v| v.as_array()) {
        for member in members {
            walk_and_extract(member, source_path, decl_index, id_to_path);
        }
    }
}

/// Build a filtered Value from a borrowed declaration node, cloning only the
/// fields needed for deserialization. Heavy fields (body, modifiers, value, etc.)
/// are never copied. This replaces the old clone-then-strip pattern that caused
/// ~117 MB of transient allocation churn.
fn build_filtered_decl(obj: &serde_json::Map<String, serde_json::Value>) -> serde_json::Value {
    let mut filtered = serde_json::Map::with_capacity(obj.len());
    for (key, value) in obj {
        if !STRIP_FIELDS.contains(&key.as_str()) {
            filtered.insert(key.clone(), value.clone());
        }
    }
    serde_json::Value::Object(filtered)
}

/// Build a filtered Value from a borrowed ContractDefinition node.
///
/// Preserves the `nodes` array (needed for `resolve_inheritdoc_typed`) but
/// filters heavy fields from each child within it. Contract-level heavy fields
/// are also skipped.
fn build_filtered_contract(obj: &serde_json::Map<String, serde_json::Value>) -> serde_json::Value {
    let mut filtered = serde_json::Map::with_capacity(obj.len());
    for (key, value) in obj {
        if STRIP_FIELDS.contains(&key.as_str()) {
            continue;
        }
        if key == "nodes" {
            // Filter heavy fields from each child node in the `nodes` array
            if let Some(arr) = value.as_array() {
                let filtered_nodes: Vec<serde_json::Value> = arr
                    .iter()
                    .map(|child| {
                        if let Some(child_obj) = child.as_object() {
                            let mut filtered_child =
                                serde_json::Map::with_capacity(child_obj.len());
                            for (ck, cv) in child_obj {
                                if !STRIP_CHILD_FIELDS.contains(&ck.as_str()) {
                                    filtered_child.insert(ck.clone(), cv.clone());
                                }
                            }
                            serde_json::Value::Object(filtered_child)
                        } else {
                            child.clone()
                        }
                    })
                    .collect();
                filtered.insert(key.clone(), serde_json::Value::Array(filtered_nodes));
            } else {
                filtered.insert(key.clone(), value.clone());
            }
        } else {
            filtered.insert(key.clone(), value.clone());
        }
    }
    serde_json::Value::Object(filtered)
}

/// Deserialize a stripped JSON Value into a `DeclNode` based on `nodeType`.
fn deserialize_decl_node(node_type: &str, value: serde_json::Value) -> Option<DeclNode> {
    match node_type {
        "FunctionDefinition" => serde_json::from_value::<FunctionDefinition>(value)
            .ok()
            .map(DeclNode::FunctionDefinition),
        "VariableDeclaration" => serde_json::from_value::<VariableDeclaration>(value)
            .ok()
            .map(DeclNode::VariableDeclaration),
        "ContractDefinition" => serde_json::from_value::<ContractDefinition>(value)
            .ok()
            .map(DeclNode::ContractDefinition),
        "EventDefinition" => serde_json::from_value::<EventDefinition>(value)
            .ok()
            .map(DeclNode::EventDefinition),
        "ErrorDefinition" => serde_json::from_value::<ErrorDefinition>(value)
            .ok()
            .map(DeclNode::ErrorDefinition),
        "StructDefinition" => serde_json::from_value::<StructDefinition>(value)
            .ok()
            .map(DeclNode::StructDefinition),
        "EnumDefinition" => serde_json::from_value::<EnumDefinition>(value)
            .ok()
            .map(DeclNode::EnumDefinition),
        "ModifierDefinition" => serde_json::from_value::<ModifierDefinition>(value)
            .ok()
            .map(DeclNode::ModifierDefinition),
        "UserDefinedValueTypeDefinition" => {
            serde_json::from_value::<UserDefinedValueTypeDefinition>(value)
                .ok()
                .map(DeclNode::UserDefinedValueTypeDefinition)
        }
        _ => None,
    }
}

// ── Core types ─────────────────────────────────────────────────────────────

/// AST node ID, matching solc's signed 64-bit integer.
pub type NodeID = i64;

/// Type description attached to expressions and some declarations.
///
/// Contains the compiler's resolved type information as strings.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TypeDescriptions {
    /// Human-readable type string, e.g. `"uint256"`, `"mapping(address => uint256)"`.
    pub type_string: Option<String>,
    /// Machine-readable type identifier, e.g. `"t_uint256"`, `"t_mapping$_t_address_$_t_uint256_$"`.
    pub type_identifier: Option<String>,
}

/// Structured documentation (NatSpec) attached to declarations.
///
/// In the AST JSON this appears either as a string or as a node with `id`, `src`, `text`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Documentation {
    /// Simple string (older solc versions or some contexts).
    String(String),
    /// Structured documentation node.
    Structured(StructuredDocumentation),
}

/// A `StructuredDocumentation` AST node.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StructuredDocumentation {
    pub id: NodeID,
    pub src: String,
    pub text: String,
}

/// External reference inside an `InlineAssembly` node.
///
/// Maps a Yul identifier to the Solidity declaration it refers to.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalReference {
    pub declaration: NodeID,
    #[serde(default)]
    pub is_offset: bool,
    #[serde(default)]
    pub is_slot: bool,
    pub src: String,
    #[serde(default)]
    pub suffix: Option<String>,
    pub value_size: Option<i64>,
}

/// An entry in a `UsingForDirective`'s `functionList`.
///
/// Either a plain `function` reference or a `definition` + `operator` pair
/// for user-defined operators.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UsingForFunction {
    /// Plain library function reference (when no operator).
    pub function: Option<IdentifierPath>,
    /// Function definition for a user-defined operator.
    pub definition: Option<IdentifierPath>,
    /// The operator being overloaded (e.g. `"+"`).
    pub operator: Option<String>,
}

/// An `IdentifierPath` AST node — a dotted name like `IERC20` or `MyLib.add`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IdentifierPath {
    pub id: NodeID,
    pub name: String,
    #[serde(default)]
    pub name_locations: Vec<String>,
    pub referenced_declaration: Option<NodeID>,
    pub src: String,
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    /// Top-level output from `solc --standard-json` after normalization.
    ///
    /// Only used in tests to deserialize full fixture files.
    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SolcOutput {
        #[serde(default)]
        pub sources: std::collections::HashMap<String, SourceEntry>,
        #[serde(default)]
        pub source_id_to_path: std::collections::HashMap<String, String>,
    }

    /// A single source file entry in the solc output.
    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct SourceEntry {
        pub id: i64,
        pub ast: super::SourceUnit,
    }
    use super::*;
    use std::path::Path;

    /// Load and deserialize the entire poolmanager.json fixture.
    fn load_fixture() -> SolcOutput {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("poolmanager.json");
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("failed to deserialize poolmanager.json: {e}"))
    }

    #[test]
    fn deserialize_poolmanager() {
        let output = load_fixture();

        // The fixture has 45 source files.
        assert_eq!(
            output.sources.len(),
            45,
            "expected 45 source files in poolmanager.json"
        );

        // Every source entry should have a valid AST with a non-empty src.
        for (path, entry) in &output.sources {
            assert!(entry.ast.id > 0, "bad AST id for {path}");
            assert!(!entry.ast.src.is_empty(), "empty src for {path}");
        }
    }

    #[test]
    fn deserialize_all_source_units() {
        let output = load_fixture();

        // Verify that we can access nodes in every source unit
        // and that the top-level contract count matches expectations.
        let mut contract_count = 0;
        for (_path, entry) in output.sources {
            for node in &entry.ast.nodes {
                if matches!(node, SourceUnitNode::ContractDefinition(_)) {
                    contract_count += 1;
                }
            }
        }
        assert_eq!(
            contract_count, 43,
            "expected 43 top-level ContractDefinitions"
        );
    }

    // ── Visitor tests ──────────────────────────────────────────────────

    /// A visitor that counts specific node types.
    struct NodeCounter {
        functions: usize,
        contracts: usize,
        events: usize,
        errors: usize,
        variables: usize,
        total_nodes: usize,
    }

    impl NodeCounter {
        fn new() -> Self {
            Self {
                functions: 0,
                contracts: 0,
                events: 0,
                errors: 0,
                variables: 0,
                total_nodes: 0,
            }
        }
    }

    impl AstVisitor for NodeCounter {
        fn visit_node(&mut self, _id: NodeID, _src: &str) -> bool {
            self.total_nodes += 1;
            true
        }

        fn visit_function_definition(&mut self, node: &FunctionDefinition) -> bool {
            self.functions += 1;
            self.visit_node(node.id, &node.src)
        }

        fn visit_contract_definition(&mut self, node: &ContractDefinition) -> bool {
            self.contracts += 1;
            self.visit_node(node.id, &node.src)
        }

        fn visit_event_definition(&mut self, node: &EventDefinition) -> bool {
            self.events += 1;
            self.visit_node(node.id, &node.src)
        }

        fn visit_error_definition(&mut self, node: &ErrorDefinition) -> bool {
            self.errors += 1;
            self.visit_node(node.id, &node.src)
        }

        fn visit_variable_declaration(&mut self, node: &VariableDeclaration) -> bool {
            self.variables += 1;
            self.visit_node(node.id, &node.src)
        }
    }

    #[test]
    fn visitor_counts_nodes() {
        let output = load_fixture();
        let mut counter = NodeCounter::new();

        for (_path, entry) in output.sources {
            entry.ast.accept(&mut counter);
        }

        assert_eq!(counter.contracts, 43, "expected 43 ContractDefinitions");
        assert_eq!(counter.functions, 215, "expected 215 FunctionDefinitions");
        assert_eq!(counter.events, 12, "expected 12 EventDefinitions");
        assert_eq!(counter.errors, 42, "expected 42 ErrorDefinitions");
        assert!(
            counter.variables > 0,
            "expected at least some VariableDeclarations"
        );
        assert!(
            counter.total_nodes > 0,
            "expected total_nodes to be non-zero"
        );

        // Sanity: total should be much larger than the sum of specific types.
        let specific_sum = counter.contracts + counter.functions + counter.events + counter.errors;
        assert!(
            counter.total_nodes > specific_sum,
            "total_nodes ({}) should be > sum of specific types ({specific_sum})",
            counter.total_nodes
        );
    }

    #[test]
    fn cached_build_populates_decl_index() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("poolmanager.json");
        let json = std::fs::read_to_string(&path).unwrap();
        let raw: serde_json::Value = serde_json::from_str(&json).unwrap();

        let build = crate::goto::CachedBuild::new(raw, 0, None);

        assert!(
            !build.decl_index.is_empty(),
            "decl_index should be populated"
        );

        // Count by variant
        let mut funcs = 0;
        let mut vars = 0;
        let mut contracts = 0;
        let mut events = 0;
        let mut errors = 0;
        for decl in build.decl_index.values() {
            match decl {
                DeclNode::FunctionDefinition(_) => funcs += 1,
                DeclNode::VariableDeclaration(_) => vars += 1,
                DeclNode::ContractDefinition(_) => contracts += 1,
                DeclNode::EventDefinition(_) => events += 1,
                DeclNode::ErrorDefinition(_) => errors += 1,
                _ => {}
            }
        }

        assert_eq!(
            contracts, 43,
            "expected 43 ContractDefinitions in decl_index"
        );
        assert_eq!(funcs, 215, "expected 215 FunctionDefinitions in decl_index");
        assert_eq!(events, 12, "expected 12 EventDefinitions in decl_index");
        assert_eq!(errors, 42, "expected 42 ErrorDefinitions in decl_index");
        assert!(vars > 0, "expected VariableDeclarations in decl_index");
    }

    /// Verify that `node_id_to_source_path` covers all declaration nodes.
    #[test]
    fn node_id_to_source_path_covers_decl_index() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("poolmanager.json");
        let json = std::fs::read_to_string(&path).unwrap();
        let raw: serde_json::Value = serde_json::from_str(&json).unwrap();

        let build = crate::goto::CachedBuild::new(raw, 0, None);

        assert!(
            !build.node_id_to_source_path.is_empty(),
            "node_id_to_source_path should be populated"
        );

        // Every contract, function, event, error at contract level should have a path
        let mut missing = Vec::new();
        for (id, decl) in &build.decl_index {
            // Skip parameter VariableDeclarations — they are nested deeper than
            // the 2-level walk in our path builder
            if matches!(decl, DeclNode::VariableDeclaration(v) if v.state_variable != Some(true)) {
                continue;
            }
            if !build.node_id_to_source_path.contains_key(id) {
                missing.push(format!(
                    "id={id} name={:?} type={}",
                    decl.name(),
                    decl.node_type()
                ));
            }
        }

        assert!(
            missing.is_empty(),
            "Declarations without source path ({}):\n{}",
            missing.len(),
            missing.join("\n"),
        );
    }
}
