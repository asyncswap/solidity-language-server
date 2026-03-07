use serde_json::Value;
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    Documentation, Hover, HoverContents, MarkupContent, MarkupKind, ParameterInformation,
    ParameterLabel, Position, SignatureHelp, SignatureInformation, Url,
};

use crate::gas::{self, GasIndex};
#[cfg(test)]
use crate::goto::CHILD_KEYS;
use crate::goto::pos_to_bytes;
use crate::references::{byte_to_decl_via_external_refs, byte_to_id};
#[cfg(test)]
use crate::types::NodeId;
use crate::types::{EventSelector, FuncSelector, MethodId, Selector};

// ── DocIndex — pre-built userdoc/devdoc lookup ─────────────────────────────

/// Merged documentation from solc userdoc + devdoc for a single declaration.
#[derive(Debug, Clone, Default)]
pub struct DocEntry {
    /// `@notice` from userdoc.
    pub notice: Option<String>,
    /// `@dev` / `details` from devdoc.
    pub details: Option<String>,
    /// `@param` descriptions from devdoc, keyed by parameter name.
    pub params: Vec<(String, String)>,
    /// `@return` descriptions from devdoc, keyed by return name.
    pub returns: Vec<(String, String)>,
    /// `@title` from devdoc (contract-level only).
    pub title: Option<String>,
    /// `@author` from devdoc (contract-level only).
    pub author: Option<String>,
}

/// Key for looking up documentation in the [`DocIndex`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocKey {
    /// 4-byte selector for functions, public variables, and errors.
    Func(FuncSelector),
    /// 32-byte topic hash for events.
    Event(EventSelector),
    /// Contract-level docs, keyed by `"path:Name"`.
    Contract(String),
    /// State variable docs, keyed by `"path:ContractName:varName"`.
    StateVar(String),
    /// Fallback for methods without a selector (shouldn't happen, but safe).
    Method(String),
}

/// Pre-built documentation index from solc contract output.
///
/// Keyed by [`DocKey`] for type-safe lookup from AST nodes.
pub type DocIndex = HashMap<DocKey, DocEntry>;

/// Build a documentation index from normalized AST output.
///
/// Iterates over `contracts[path][name]` and merges userdoc + devdoc
/// into `DocEntry` values keyed for fast lookup from AST nodes.
pub fn build_doc_index(ast_data: &Value) -> DocIndex {
    let mut index = DocIndex::new();

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
            let userdoc = contract.get("userdoc");
            let devdoc = contract.get("devdoc");
            let method_ids = contract
                .get("evm")
                .and_then(|e| e.get("methodIdentifiers"))
                .and_then(|m| m.as_object());

            // Build canonical_sig → selector for userdoc/devdoc key lookups
            let sig_to_selector: HashMap<&str, &str> = method_ids
                .map(|mi| {
                    mi.iter()
                        .filter_map(|(sig, sel)| sel.as_str().map(|s| (sig.as_str(), s)))
                        .collect()
                })
                .unwrap_or_default();

            // ── Contract-level docs ──
            let mut contract_entry = DocEntry::default();
            if let Some(ud) = userdoc {
                contract_entry.notice = ud
                    .get("notice")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
            if let Some(dd) = devdoc {
                contract_entry.title = dd
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                contract_entry.details = dd
                    .get("details")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                contract_entry.author = dd
                    .get("author")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
            if contract_entry.notice.is_some()
                || contract_entry.title.is_some()
                || contract_entry.details.is_some()
            {
                let key = DocKey::Contract(format!("{path}:{name}"));
                index.insert(key, contract_entry);
            }

            // ── Method docs (functions + public state variable getters) ──
            let ud_methods = userdoc
                .and_then(|u| u.get("methods"))
                .and_then(|m| m.as_object());
            let dd_methods = devdoc
                .and_then(|d| d.get("methods"))
                .and_then(|m| m.as_object());

            // Collect all canonical sigs from both userdoc and devdoc methods
            let mut all_sigs: Vec<&str> = Vec::new();
            if let Some(um) = ud_methods {
                all_sigs.extend(um.keys().map(|k| k.as_str()));
            }
            if let Some(dm) = dd_methods {
                for k in dm.keys() {
                    if !all_sigs.contains(&k.as_str()) {
                        all_sigs.push(k.as_str());
                    }
                }
            }

            for sig in &all_sigs {
                let mut entry = DocEntry::default();

                // userdoc notice
                if let Some(um) = ud_methods
                    && let Some(method) = um.get(*sig)
                {
                    entry.notice = method
                        .get("notice")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }

                // devdoc details + params + returns
                if let Some(dm) = dd_methods
                    && let Some(method) = dm.get(*sig)
                {
                    entry.details = method
                        .get("details")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    if let Some(params) = method.get("params").and_then(|p| p.as_object()) {
                        for (pname, pdesc) in params {
                            if let Some(desc) = pdesc.as_str() {
                                entry.params.push((pname.clone(), desc.to_string()));
                            }
                        }
                    }

                    if let Some(returns) = method.get("returns").and_then(|r| r.as_object()) {
                        for (rname, rdesc) in returns {
                            if let Some(desc) = rdesc.as_str() {
                                entry.returns.push((rname.clone(), desc.to_string()));
                            }
                        }
                    }
                }

                if entry.notice.is_none()
                    && entry.details.is_none()
                    && entry.params.is_empty()
                    && entry.returns.is_empty()
                {
                    continue;
                }

                // Key by selector (for AST node matching)
                if let Some(selector) = sig_to_selector.get(sig) {
                    let key = DocKey::Func(FuncSelector::new(*selector));
                    index.insert(key, entry);
                } else {
                    // No selector (shouldn't happen for methods, but be safe)
                    // Key by function name for fallback matching
                    let fn_name = sig.split('(').next().unwrap_or(sig);
                    let key = DocKey::Method(format!("{path}:{name}:{fn_name}"));
                    index.insert(key, entry);
                }
            }

            // ── Error docs ──
            let ud_errors = userdoc
                .and_then(|u| u.get("errors"))
                .and_then(|e| e.as_object());
            let dd_errors = devdoc
                .and_then(|d| d.get("errors"))
                .and_then(|e| e.as_object());

            let mut all_error_sigs: Vec<&str> = Vec::new();
            if let Some(ue) = ud_errors {
                all_error_sigs.extend(ue.keys().map(|k| k.as_str()));
            }
            if let Some(de) = dd_errors {
                for k in de.keys() {
                    if !all_error_sigs.contains(&k.as_str()) {
                        all_error_sigs.push(k.as_str());
                    }
                }
            }

            for sig in &all_error_sigs {
                let mut entry = DocEntry::default();

                // userdoc: errors are arrays of { notice }
                if let Some(ue) = ud_errors
                    && let Some(arr) = ue.get(*sig).and_then(|v| v.as_array())
                    && let Some(first) = arr.first()
                {
                    entry.notice = first
                        .get("notice")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }

                // devdoc: errors are also arrays
                if let Some(de) = dd_errors
                    && let Some(arr) = de.get(*sig).and_then(|v| v.as_array())
                    && let Some(first) = arr.first()
                {
                    entry.details = first
                        .get("details")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    if let Some(params) = first.get("params").and_then(|p| p.as_object()) {
                        for (pname, pdesc) in params {
                            if let Some(desc) = pdesc.as_str() {
                                entry.params.push((pname.clone(), desc.to_string()));
                            }
                        }
                    }
                }

                if entry.notice.is_none() && entry.details.is_none() && entry.params.is_empty() {
                    continue;
                }

                // Compute 4-byte error selector from the canonical signature
                // errorSelector = keccak256(sig)[0..4]
                let selector = FuncSelector::new(compute_selector(sig));
                index.insert(DocKey::Func(selector), entry);
            }

            // ── Event docs ──
            let ud_events = userdoc
                .and_then(|u| u.get("events"))
                .and_then(|e| e.as_object());
            let dd_events = devdoc
                .and_then(|d| d.get("events"))
                .and_then(|e| e.as_object());

            let mut all_event_sigs: Vec<&str> = Vec::new();
            if let Some(ue) = ud_events {
                all_event_sigs.extend(ue.keys().map(|k| k.as_str()));
            }
            if let Some(de) = dd_events {
                for k in de.keys() {
                    if !all_event_sigs.contains(&k.as_str()) {
                        all_event_sigs.push(k.as_str());
                    }
                }
            }

            for sig in &all_event_sigs {
                let mut entry = DocEntry::default();

                if let Some(ue) = ud_events
                    && let Some(ev) = ue.get(*sig)
                {
                    entry.notice = ev
                        .get("notice")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }

                if let Some(de) = dd_events
                    && let Some(ev) = de.get(*sig)
                {
                    entry.details = ev
                        .get("details")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    if let Some(params) = ev.get("params").and_then(|p| p.as_object()) {
                        for (pname, pdesc) in params {
                            if let Some(desc) = pdesc.as_str() {
                                entry.params.push((pname.clone(), desc.to_string()));
                            }
                        }
                    }
                }

                if entry.notice.is_none() && entry.details.is_none() && entry.params.is_empty() {
                    continue;
                }

                // Event topic = full keccak256 hash of canonical signature
                let topic = EventSelector::new(compute_event_topic(sig));
                index.insert(DocKey::Event(topic), entry);
            }

            // ── State variable docs (from devdoc) ──
            if let Some(dd) = devdoc
                && let Some(state_vars) = dd.get("stateVariables").and_then(|s| s.as_object())
            {
                for (var_name, var_doc) in state_vars {
                    let mut entry = DocEntry {
                        details: var_doc
                            .get("details")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        ..DocEntry::default()
                    };

                    if let Some(returns) = var_doc.get("return").and_then(|v| v.as_str()) {
                        entry.returns.push(("_0".to_string(), returns.to_string()));
                    }
                    if let Some(returns) = var_doc.get("returns").and_then(|r| r.as_object()) {
                        for (rname, rdesc) in returns {
                            if let Some(desc) = rdesc.as_str() {
                                entry.returns.push((rname.clone(), desc.to_string()));
                            }
                        }
                    }

                    if entry.details.is_some() || !entry.returns.is_empty() {
                        let key = DocKey::StateVar(format!("{path}:{name}:{var_name}"));
                        index.insert(key, entry);
                    }
                }
            }
        }
    }

    index
}

/// Compute a 4-byte function/error selector from a canonical ABI signature.
///
/// `keccak256("transfer(address,uint256)")` → first 4 bytes as hex.
fn compute_selector(sig: &str) -> String {
    use tiny_keccak::{Hasher, Keccak};
    let mut hasher = Keccak::v256();
    hasher.update(sig.as_bytes());
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    hex::encode(&output[..4])
}

/// Compute a full 32-byte event topic from a canonical ABI signature.
///
/// `keccak256("Transfer(address,address,uint256)")` → full hash as hex.
fn compute_event_topic(sig: &str) -> String {
    use tiny_keccak::{Hasher, Keccak};
    let mut hasher = Keccak::v256();
    hasher.update(sig.as_bytes());
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    hex::encode(output)
}

/// Look up documentation for an AST declaration node from the DocIndex.
///
/// Returns a cloned DocEntry since key construction is dynamic.
/// Typed version of `lookup_doc_entry` using `DeclNode` and `decl_index`.
///
/// Looks up documentation for an AST declaration node from the DocIndex
/// using typed field access instead of raw `Value` chains.
pub fn lookup_doc_entry_typed(
    doc_index: &DocIndex,
    decl: &crate::solc_ast::DeclNode,
    decl_index: &std::collections::HashMap<crate::types::NodeId, crate::solc_ast::DeclNode>,
    node_id_to_source_path: &std::collections::HashMap<crate::types::NodeId, crate::types::AbsPath>,
) -> Option<DocEntry> {
    use crate::solc_ast::DeclNode;

    match decl {
        DeclNode::FunctionDefinition(_) | DeclNode::VariableDeclaration(_) => {
            // Try by selector first
            if let Some(sel) = decl.selector() {
                let key = DocKey::Func(FuncSelector::new(sel));
                if let Some(entry) = doc_index.get(&key) {
                    return Some(entry.clone());
                }
            }

            // For state variables without selector, try statevar key
            if matches!(decl, DeclNode::VariableDeclaration(_))
                && let var_name = decl.name()
                && let Some(scope_id) = decl.scope()
                && let Some(scope_decl) = decl_index.get(&crate::types::NodeId(scope_id))
                && let Some(path) = node_id_to_source_path.get(&crate::types::NodeId(scope_id))
            {
                let contract_name = scope_decl.name();
                let key = DocKey::StateVar(format!("{path}:{contract_name}:{var_name}"));
                if let Some(entry) = doc_index.get(&key) {
                    return Some(entry.clone());
                }
            }

            // Fallback: try method by name
            let fn_name = decl.name();
            let scope_id = decl.scope()?;
            let scope_decl = decl_index.get(&crate::types::NodeId(scope_id))?;
            let contract_name = scope_decl.name();
            let path = node_id_to_source_path.get(&crate::types::NodeId(scope_id))?;
            let key = DocKey::Method(format!("{path}:{contract_name}:{fn_name}"));
            doc_index.get(&key).cloned()
        }
        DeclNode::ErrorDefinition(_) => {
            let sel = decl.selector()?;
            let key = DocKey::Func(FuncSelector::new(sel));
            doc_index.get(&key).cloned()
        }
        DeclNode::EventDefinition(_) => {
            let sel = decl.selector()?;
            let key = DocKey::Event(EventSelector::new(sel));
            doc_index.get(&key).cloned()
        }
        DeclNode::ContractDefinition(_) => {
            let contract_name = decl.name();
            let node_id = decl.id();
            let path = node_id_to_source_path.get(&crate::types::NodeId(node_id))?;
            let key = DocKey::Contract(format!("{path}:{contract_name}"));
            doc_index.get(&key).cloned()
        }
        _ => None,
    }
}

/// Look up documentation for a parameter from its parent function/error/event.
///
/// When hovering a `VariableDeclaration` that is a parameter or return value,
/// this walks up to the parent declaration (via `scope`) and extracts the
/// Typed version of `lookup_param_doc` using `DeclNode` and `decl_index`.
///
/// When hovering a parameter/return `VariableDeclaration`, this looks up
/// the parent declaration in `decl_index` and extracts `@param`/`@return` doc.
pub fn lookup_param_doc_typed(
    doc_index: &DocIndex,
    decl: &crate::solc_ast::DeclNode,
    decl_index: &std::collections::HashMap<crate::types::NodeId, crate::solc_ast::DeclNode>,
    node_id_to_source_path: &std::collections::HashMap<crate::types::NodeId, crate::types::AbsPath>,
) -> Option<String> {
    use crate::solc_ast::DeclNode;

    // Only VariableDeclarations can be parameters
    let var = match decl {
        DeclNode::VariableDeclaration(v) => v,
        _ => return None,
    };

    let param_name = &var.name;
    if param_name.is_empty() {
        return None;
    }

    // Walk up to the parent via scope
    let scope_id = var.scope?;
    let parent = decl_index.get(&crate::types::NodeId(scope_id))?;

    // Only handle function/error/event/modifier parents
    if !matches!(
        parent,
        DeclNode::FunctionDefinition(_)
            | DeclNode::ErrorDefinition(_)
            | DeclNode::EventDefinition(_)
            | DeclNode::ModifierDefinition(_)
    ) {
        return None;
    }

    // Determine if this param is a return value (only for functions)
    let is_return = if let Some(ret_params) = parent.return_parameters() {
        ret_params.parameters.iter().any(|p| p.id == var.id)
    } else {
        false
    };

    // Try DocIndex first (structured devdoc)
    if let Some(parent_doc) =
        lookup_doc_entry_typed(doc_index, parent, decl_index, node_id_to_source_path)
    {
        if is_return {
            for (rname, rdesc) in &parent_doc.returns {
                if rname == param_name {
                    return Some(rdesc.clone());
                }
            }
        } else {
            for (pname, pdesc) in &parent_doc.params {
                if pname == param_name {
                    return Some(pdesc.clone());
                }
            }
        }
    }

    // Fallback: parse raw AST documentation on the parent
    if let Some(doc_text) = parent.extract_doc_text() {
        let resolved = if doc_text.contains("@inheritdoc") {
            resolve_inheritdoc_typed(parent, &doc_text, decl_index)
        } else {
            None
        };
        let text = resolved.as_deref().unwrap_or(&doc_text);

        let tag = if is_return { "@return " } else { "@param " };
        for line in text.lines() {
            let trimmed = line.trim().trim_start_matches('*').trim();
            if let Some(rest) = trimmed.strip_prefix(tag) {
                if let Some((name, desc)) = rest.split_once(' ') {
                    if name == param_name {
                        return Some(desc.to_string());
                    }
                } else if rest == param_name {
                    return Some(String::new());
                }
            }
        }
    }

    None
}

/// Format a `DocEntry` as markdown for hover display.
pub fn format_doc_entry(entry: &DocEntry) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Title (contract-level)
    if let Some(title) = &entry.title {
        lines.push(format!("**{title}**"));
        lines.push(String::new());
    }

    // Notice (@notice)
    if let Some(notice) = &entry.notice {
        lines.push(notice.clone());
    }

    // Author
    if let Some(author) = &entry.author {
        lines.push(format!("*@author {author}*"));
    }

    // Details (@dev)
    if let Some(details) = &entry.details {
        lines.push(String::new());
        lines.push("**@dev**".to_string());
        lines.push(format!("*{details}*"));
    }

    // Parameters (@param)
    if !entry.params.is_empty() {
        lines.push(String::new());
        lines.push("**Parameters:**".to_string());
        for (name, desc) in &entry.params {
            lines.push(format!("- `{name}` — {desc}"));
        }
    }

    // Returns (@return)
    if !entry.returns.is_empty() {
        lines.push(String::new());
        lines.push("**Returns:**".to_string());
        for (name, desc) in &entry.returns {
            if name.starts_with('_') && name.len() <= 3 {
                // Unnamed return (e.g. "_0") — just show description
                lines.push(format!("- {desc}"));
            } else {
                lines.push(format!("- `{name}` — {desc}"));
            }
        }
    }

    lines.join("\n")
}

/// Find the raw AST node with the given id by walking all sources.
///
/// O(N) DFS walk — only used in tests. Production code uses the O(1)
/// `CachedBuild.id_index` HashMap instead.
#[cfg(test)]
fn find_node_by_id(sources: &Value, target_id: NodeId) -> Option<&Value> {
    let sources_obj = sources.as_object()?;
    for (_path, source_data) in sources_obj {
        let ast = source_data.get("ast")?;

        // Check root
        if ast.get("id").and_then(|v| v.as_i64()) == Some(target_id.0) {
            return Some(ast);
        }

        let mut stack = vec![ast];
        while let Some(node) = stack.pop() {
            if node.get("id").and_then(|v| v.as_i64()) == Some(target_id.0) {
                return Some(node);
            }
            for key in CHILD_KEYS {
                if let Some(value) = node.get(key) {
                    match value {
                        Value::Array(arr) => stack.extend(arr.iter()),
                        Value::Object(_) => stack.push(value),
                        _ => {}
                    }
                }
            }
        }
    }
    None
}

/// Extract documentation text from a node.
/// Handles both object form `{text: "..."}` and plain string form.
pub fn extract_documentation(node: &Value) -> Option<String> {
    let doc = node.get("documentation")?;
    match doc {
        Value::Object(_) => doc
            .get("text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

/// Extract the selector from a declaration node.
///
/// Returns a [`Selector`] — either a 4-byte [`FuncSelector`] (for functions,
/// public variables, and errors) or a 32-byte [`EventSelector`] (for events).
pub fn extract_selector(node: &Value) -> Option<Selector> {
    let node_type = node.get("nodeType").and_then(|v| v.as_str())?;
    match node_type {
        "FunctionDefinition" | "VariableDeclaration" => node
            .get("functionSelector")
            .and_then(|v| v.as_str())
            .map(|s| Selector::Func(FuncSelector::new(s))),
        "ErrorDefinition" => node
            .get("errorSelector")
            .and_then(|v| v.as_str())
            .map(|s| Selector::Func(FuncSelector::new(s))),
        "EventDefinition" => node
            .get("eventSelector")
            .and_then(|v| v.as_str())
            .map(|s| Selector::Event(EventSelector::new(s))),
        _ => None,
    }
}

/// Resolve `@inheritdoc ParentName` by matching selectors in the parent
/// contract's typed `nodes` array using `DeclNode` and `decl_index`.
pub fn resolve_inheritdoc_typed(
    decl: &crate::solc_ast::DeclNode,
    doc_text: &str,
    decl_index: &std::collections::HashMap<crate::types::NodeId, crate::solc_ast::DeclNode>,
) -> Option<String> {
    use crate::solc_ast::DeclNode;

    // Parse "@inheritdoc ParentName"
    let parent_name = doc_text
        .lines()
        .find_map(|line| {
            let trimmed = line.trim().trim_start_matches('*').trim();
            trimmed.strip_prefix("@inheritdoc ")
        })?
        .trim();

    // Get the selector from the implementation function
    let impl_selector = decl.extract_typed_selector()?;

    // Get the scope (containing contract id)
    let scope_id = decl.scope()?;

    // Find the scope contract in decl_index
    let scope_decl = decl_index.get(&crate::types::NodeId(scope_id))?;
    let scope_contract = match scope_decl {
        DeclNode::ContractDefinition(c) => c,
        _ => return None,
    };

    // Find the parent contract in baseContracts by name
    let parent_id = scope_contract.base_contracts.iter().find_map(|base| {
        if base.base_name.name == parent_name {
            base.base_name.referenced_declaration
        } else {
            None
        }
    })?;

    // Find the parent contract in decl_index
    let parent_decl = decl_index.get(&crate::types::NodeId(parent_id))?;
    let parent_contract = match parent_decl {
        DeclNode::ContractDefinition(c) => c,
        _ => return None,
    };

    // Search parent's children for matching selector
    for child in &parent_contract.nodes {
        if let Some(child_sel_str) = child.selector() {
            // Compare selectors — both are hex strings
            let child_matches = match &impl_selector {
                crate::types::Selector::Func(fs) => child_sel_str == fs.as_hex(),
                crate::types::Selector::Event(es) => child_sel_str == es.as_hex(),
            };
            if child_matches {
                return child.documentation_text();
            }
        }
    }

    None
}

/// Format NatSpec documentation as markdown.
/// Strips leading `@` tags and formats them nicely.
/// When `inherited_doc` is provided, it replaces `@inheritdoc` lines with the resolved content.
pub fn format_natspec(text: &str, inherited_doc: Option<&str>) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut in_params = false;
    let mut in_returns = false;

    for raw_line in text.lines() {
        let line = raw_line.trim().trim_start_matches('*').trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("@title ") {
            in_params = false;
            in_returns = false;
            lines.push(format!("**{rest}**"));
            lines.push(String::new());
        } else if let Some(rest) = line.strip_prefix("@notice ") {
            in_params = false;
            in_returns = false;
            lines.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("@dev ") {
            in_params = false;
            in_returns = false;
            lines.push(String::new());
            lines.push("**@dev**".to_string());
            lines.push(format!("*{rest}*"));
        } else if let Some(rest) = line.strip_prefix("@param ") {
            if !in_params {
                in_params = true;
                in_returns = false;
                lines.push(String::new());
                lines.push("**Parameters:**".to_string());
            }
            if let Some((name, desc)) = rest.split_once(' ') {
                lines.push(format!("- `{name}` — {desc}"));
            } else {
                lines.push(format!("- `{rest}`"));
            }
        } else if let Some(rest) = line.strip_prefix("@return ") {
            if !in_returns {
                in_returns = true;
                in_params = false;
                lines.push(String::new());
                lines.push("**Returns:**".to_string());
            }
            if let Some((name, desc)) = rest.split_once(' ') {
                lines.push(format!("- `{name}` — {desc}"));
            } else {
                lines.push(format!("- `{rest}`"));
            }
        } else if let Some(rest) = line.strip_prefix("@author ") {
            in_params = false;
            in_returns = false;
            lines.push(format!("*@author {rest}*"));
        } else if line.starts_with("@inheritdoc ") {
            // Resolve inherited docs if available
            if let Some(inherited) = inherited_doc {
                // Recursively format the inherited doc (it won't have another @inheritdoc)
                let formatted = format_natspec(inherited, None);
                if !formatted.is_empty() {
                    lines.push(formatted);
                }
            } else {
                let parent = line.strip_prefix("@inheritdoc ").unwrap_or("");
                lines.push(format!("*Inherits documentation from `{parent}`*"));
            }
        } else if line.starts_with('@') {
            // Any other tag (@custom:xyz, @dev, etc.)
            in_params = false;
            in_returns = false;
            if let Some((tag, rest)) = line.split_once(' ') {
                lines.push(String::new());
                lines.push(format!("**{tag}**"));
                lines.push(format!("*{rest}*"));
            } else {
                lines.push(String::new());
                lines.push(format!("**{line}**"));
            }
        } else {
            // Continuation line
            lines.push(line.to_string());
        }
    }

    lines.join("\n")
}

/// Format parameter list from a parameters node.
fn format_parameters(params_node: Option<&Value>) -> String {
    let params_node = match params_node {
        Some(v) => v,
        None => return String::new(),
    };
    let params = match params_node.get("parameters").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return String::new(),
    };

    let parts: Vec<String> = params
        .iter()
        .map(|p| {
            let type_str = p
                .get("typeDescriptions")
                .and_then(|v| v.get("typeString"))
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let storage = p
                .get("storageLocation")
                .and_then(|v| v.as_str())
                .unwrap_or("default");

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

/// Build a function/modifier signature string from a raw AST node.
pub(crate) fn build_function_signature(node: &Value) -> Option<String> {
    let node_type = node.get("nodeType").and_then(|v| v.as_str())?;
    let name = node.get("name").and_then(|v| v.as_str()).unwrap_or("");

    match node_type {
        "FunctionDefinition" => {
            let kind = node
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("function");
            let visibility = node
                .get("visibility")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let state_mutability = node
                .get("stateMutability")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let params = format_parameters(node.get("parameters"));
            let returns = format_parameters(node.get("returnParameters"));

            let mut sig = match kind {
                "constructor" => format!("constructor({params})"),
                "receive" => "receive() external payable".to_string(),
                "fallback" => format!("fallback({params})"),
                _ => format!("function {name}({params})"),
            };

            if !visibility.is_empty() && kind != "constructor" && kind != "receive" {
                sig.push_str(&format!(" {visibility}"));
            }
            if !state_mutability.is_empty() && state_mutability != "nonpayable" {
                sig.push_str(&format!(" {state_mutability}"));
            }
            if !returns.is_empty() {
                sig.push_str(&format!(" returns ({returns})"));
            }
            Some(sig)
        }
        "ModifierDefinition" => {
            let params = format_parameters(node.get("parameters"));
            Some(format!("modifier {name}({params})"))
        }
        "EventDefinition" => {
            let params = format_parameters(node.get("parameters"));
            Some(format!("event {name}({params})"))
        }
        "ErrorDefinition" => {
            let params = format_parameters(node.get("parameters"));
            Some(format!("error {name}({params})"))
        }
        "VariableDeclaration" => {
            let type_str = node
                .get("typeDescriptions")
                .and_then(|v| v.get("typeString"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let visibility = node
                .get("visibility")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let mutability = node
                .get("mutability")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let mut sig = type_str.to_string();
            if !visibility.is_empty() {
                sig.push_str(&format!(" {visibility}"));
            }
            if mutability == "constant" || mutability == "immutable" {
                sig.push_str(&format!(" {mutability}"));
            }
            sig.push_str(&format!(" {name}"));
            Some(sig)
        }
        "ContractDefinition" => {
            let contract_kind = node
                .get("contractKind")
                .and_then(|v| v.as_str())
                .unwrap_or("contract");

            let mut sig = format!("{contract_kind} {name}");

            // Add base contracts
            if let Some(bases) = node.get("baseContracts").and_then(|v| v.as_array())
                && !bases.is_empty()
            {
                let base_names: Vec<&str> = bases
                    .iter()
                    .filter_map(|b| {
                        b.get("baseName")
                            .and_then(|bn| bn.get("name"))
                            .and_then(|n| n.as_str())
                    })
                    .collect();
                if !base_names.is_empty() {
                    sig.push_str(&format!(" is {}", base_names.join(", ")));
                }
            }
            Some(sig)
        }
        "StructDefinition" => {
            let mut sig = format!("struct {name} {{\n");
            if let Some(members) = node.get("members").and_then(|v| v.as_array()) {
                for member in members {
                    let mname = member.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                    let mtype = member
                        .get("typeDescriptions")
                        .and_then(|v| v.get("typeString"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    sig.push_str(&format!("    {mtype} {mname};\n"));
                }
            }
            sig.push('}');
            Some(sig)
        }
        "EnumDefinition" => {
            let mut sig = format!("enum {name} {{\n");
            if let Some(members) = node.get("members").and_then(|v| v.as_array()) {
                let names: Vec<&str> = members
                    .iter()
                    .filter_map(|m| m.get("name").and_then(|v| v.as_str()))
                    .collect();
                for n in &names {
                    sig.push_str(&format!("    {n},\n"));
                }
            }
            sig.push('}');
            Some(sig)
        }
        "UserDefinedValueTypeDefinition" => {
            let underlying = node
                .get("underlyingType")
                .and_then(|v| v.get("typeDescriptions"))
                .and_then(|v| v.get("typeString"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            Some(format!("type {name} is {underlying}"))
        }
        _ => None,
    }
}

// ── Signature Help ─────────────────────────────────────────────────────────

/// Find a mapping `VariableDeclaration` by name in the typed `decl_index`.
///
/// Returns the `VariableDeclaration` whose `name` matches and whose
/// `type_name` is `TypeName::Mapping`.
fn find_mapping_decl_typed<'a>(
    decl_index: &'a std::collections::HashMap<crate::types::NodeId, crate::solc_ast::DeclNode>,
    name: &str,
) -> Option<&'a crate::solc_ast::VariableDeclaration> {
    use crate::solc_ast::DeclNode;

    decl_index.values().find_map(|decl| match decl {
        DeclNode::VariableDeclaration(v)
            if v.name == name
                && matches!(
                    v.type_name.as_ref(),
                    Some(crate::solc_ast::TypeName::Mapping(_))
                ) =>
        {
            Some(v)
        }
        _ => None,
    })
}

/// Typed version of `mapping_signature_help` using `DeclNode`.
///
/// Builds signature help for `name[key]` from a typed `VariableDeclaration`
/// with a `Mapping` type name, avoiding the O(N) CHILD_KEYS DFS walk.
fn mapping_signature_help_typed(
    decl_index: &std::collections::HashMap<crate::types::NodeId, crate::solc_ast::DeclNode>,
    name: &str,
) -> Option<SignatureHelp> {
    use crate::solc_ast::TypeName;

    let decl = find_mapping_decl_typed(decl_index, name)?;
    let mapping = match decl.type_name.as_ref()? {
        TypeName::Mapping(m) => m,
        _ => return None,
    };

    // Key type string from the key's TypeDescriptions
    let key_type = crate::solc_ast::type_name_to_str(&mapping.key_type);

    // Named mapping keys (Solidity >= 0.8.18)
    let key_name = mapping.key_name.as_deref().filter(|s| !s.is_empty());

    let param_str = if let Some(kn) = key_name {
        format!("{} {}", key_type, kn)
    } else {
        key_type.to_string()
    };

    let sig_label = format!("{}[{}]", name, param_str);

    let param_start = name.len() + 1; // after `[`
    let param_end = param_start + param_str.len();

    let key_param_name = key_name.unwrap_or("");
    let var_name = &decl.name;

    let param_info = ParameterInformation {
        label: ParameterLabel::LabelOffsets([param_start as u32, param_end as u32]),
        documentation: if !key_param_name.is_empty() {
            Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("`{}` — key for `{}`", key_param_name, var_name),
            }))
        } else {
            None
        },
    };

    // Value type for function-level documentation
    let value_type = crate::solc_ast::type_name_to_str(&mapping.value_type);
    let sig_doc = Some(format!("@returns `{}`", value_type));

    Some(SignatureHelp {
        signatures: vec![SignatureInformation {
            label: sig_label,
            documentation: sig_doc.map(|doc| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc,
                })
            }),
            parameters: Some(vec![param_info]),
            active_parameter: Some(0),
        }],
        active_signature: Some(0),
        active_parameter: Some(0),
    })
}

/// Produce signature help for the call at the given position.
///
/// Uses tree-sitter on the live buffer to find the enclosing call and argument
/// index, then resolves the declaration via `HintIndex` to build the signature
/// with parameter label offsets and `@param` documentation.
///
/// Also handles mapping index access (`name[key]`), showing the key type.
pub fn signature_help(
    cached_build: &crate::goto::CachedBuild,
    source_bytes: &[u8],
    position: Position,
) -> Option<SignatureHelp> {
    let hint_index = &cached_build.hint_index;
    let doc_index = &cached_build.doc_index;
    let di = &cached_build.decl_index;
    let id_to_path = &cached_build.node_id_to_source_path;

    let source_str = String::from_utf8_lossy(source_bytes);
    let tree = crate::inlay_hints::ts_parse(&source_str)?;
    let byte_pos = pos_to_bytes(source_bytes, position);

    // Find the enclosing call and which argument the cursor is on
    let ctx =
        crate::inlay_hints::ts_find_call_for_signature(tree.root_node(), &source_str, byte_pos)?;

    // Mapping index access: use typed decl_index
    if ctx.is_index_access {
        return mapping_signature_help_typed(di, ctx.name);
    }

    // Try all hint lookups to resolve the callsite declaration and get skip count
    let (decl_id, skip) = hint_index.values().find_map(|lookup| {
        lookup.resolve_callsite_with_skip(ctx.call_start_byte, ctx.name, ctx.arg_count)
    })?;

    // Typed DeclNode — O(1) from decl_index
    let typed_decl = di.get(&decl_id)?;

    // Build the signature label
    let sig_label = typed_decl.build_signature()?;

    // Build individual parameter strings for offset calculation
    let param_strings = typed_decl.param_strings();

    // Look up @param docs from DocIndex
    let doc_entry = lookup_doc_entry_typed(doc_index, typed_decl, di, id_to_path);

    // Calculate parameter label offsets within the signature string
    // The signature looks like: "function name(uint256 amount, uint16 tax) ..."
    // We need to find the byte offset of each parameter within the label
    let params_start = sig_label.find('(')? + 1;
    let mut param_infos = Vec::new();
    let mut offset = params_start;

    for (i, param_str) in param_strings.iter().enumerate() {
        let start = offset;
        let end = start + param_str.len();

        // Find @param doc for this parameter
        let param_name = typed_decl
            .parameters()
            .and_then(|pl| pl.parameters.get(i))
            .map(|p| p.name.as_str())
            .unwrap_or("");

        let param_doc = doc_entry.as_ref().and_then(|entry| {
            entry
                .params
                .iter()
                .find(|(name, _)| name == param_name)
                .map(|(_, desc)| desc.clone())
        });

        param_infos.push(ParameterInformation {
            label: ParameterLabel::LabelOffsets([start as u32, end as u32]),
            documentation: param_doc.map(|doc| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc,
                })
            }),
        });

        // +2 for ", " separator
        offset = end + 2;
    }

    // Build notice/dev documentation for the signature
    let sig_doc = doc_entry.as_ref().and_then(|entry| {
        let mut parts = Vec::new();
        if let Some(notice) = &entry.notice {
            parts.push(notice.clone());
        }
        if let Some(details) = &entry.details {
            parts.push(format!("*{}*", details));
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n\n"))
        }
    });

    // Adjust activeParameter for using-for (skip=1 means first param is self)
    let active_param = (ctx.arg_index + skip) as u32;

    Some(SignatureHelp {
        signatures: vec![SignatureInformation {
            label: sig_label,
            documentation: sig_doc.map(|doc| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc,
                })
            }),
            parameters: Some(param_infos),
            active_parameter: Some(active_param),
        }],
        active_signature: Some(0),
        active_parameter: Some(active_param),
    })
}

/// Check if the source text has the gas sentinel comment above a declaration.
///
/// Looks at the lines preceding the declaration's byte offset in the source
/// for a comment containing `@lsp-enable gas-estimates`.
fn source_has_gas_sentinel(source: &str, src_field: &str) -> bool {
    let offset = src_field
        .split(':')
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    // Look at the text before this declaration for the sentinel
    let preceding = &source[..offset.min(source.len())];
    // Check last few lines before the declaration
    for line in preceding.lines().rev().take(10) {
        let trimmed = line.trim();
        if trimmed.contains(gas::GAS_SENTINEL) {
            return true;
        }
        // Stop if we hit a non-comment, non-empty line
        if !trimmed.is_empty()
            && !trimmed.starts_with("///")
            && !trimmed.starts_with("//")
            && !trimmed.starts_with('*')
            && !trimmed.starts_with("/*")
        {
            break;
        }
    }
    false
}

/// Typed version of `gas_hover_for_function` using `DeclNode`.
fn gas_hover_for_function_typed(
    decl: &crate::solc_ast::DeclNode,
    gas_index: &GasIndex,
    decl_index: &std::collections::HashMap<crate::types::NodeId, crate::solc_ast::DeclNode>,
    node_id_to_source_path: &std::collections::HashMap<crate::types::NodeId, crate::types::AbsPath>,
) -> Option<String> {
    use crate::solc_ast::DeclNode;

    let func = match decl {
        DeclNode::FunctionDefinition(f) => f,
        _ => return None,
    };

    // Try by selector first (external/public functions)
    if let Some(sel) = &func.function_selector
        && let Some((_contract, cost)) = gas::gas_by_selector(gas_index, &FuncSelector::new(sel))
    {
        return Some(format!("Gas: `{}`", gas::format_gas(cost)));
    }

    // Try by name (internal functions)
    let contract_key =
        gas::resolve_contract_key_typed(decl, gas_index, decl_index, node_id_to_source_path)?;
    let contract_gas = gas_index.get(&contract_key)?;

    let prefix = format!("{}(", func.name);
    for (sig, cost) in &contract_gas.internal {
        if sig.starts_with(&prefix) {
            return Some(format!("Gas: `{}`", gas::format_gas(cost)));
        }
    }

    None
}

/// Typed version of `gas_hover_for_contract` using `DeclNode`.
fn gas_hover_for_contract_typed(
    decl: &crate::solc_ast::DeclNode,
    gas_index: &GasIndex,
    decl_index: &std::collections::HashMap<crate::types::NodeId, crate::solc_ast::DeclNode>,
    node_id_to_source_path: &std::collections::HashMap<crate::types::NodeId, crate::types::AbsPath>,
) -> Option<String> {
    use crate::solc_ast::DeclNode;

    if !matches!(decl, DeclNode::ContractDefinition(_)) {
        return None;
    }

    let contract_key =
        gas::resolve_contract_key_typed(decl, gas_index, decl_index, node_id_to_source_path)?;
    let contract_gas = gas_index.get(&contract_key)?;

    let mut lines = Vec::new();

    if !contract_gas.creation.is_empty() {
        lines.push("**Deploy Cost**".to_string());
        if let Some(cost) = contract_gas.creation.get("totalCost") {
            lines.push(format!("- Total: `{}`", gas::format_gas(cost)));
        }
        if let Some(cost) = contract_gas.creation.get("codeDepositCost") {
            lines.push(format!("- Code deposit: `{}`", gas::format_gas(cost)));
        }
        if let Some(cost) = contract_gas.creation.get("executionCost") {
            lines.push(format!("- Execution: `{}`", gas::format_gas(cost)));
        }
    }

    if !contract_gas.external_by_sig.is_empty() {
        lines.push(String::new());
        lines.push("**Function Gas**".to_string());

        let mut fns: Vec<(&MethodId, &String)> = contract_gas.external_by_sig.iter().collect();
        fns.sort_by_key(|(k, _)| k.as_str().to_string());

        for (sig, cost) in fns {
            lines.push(format!("- `{}`: `{}`", sig.name(), gas::format_gas(cost)));
        }
    }

    if lines.is_empty() {
        return None;
    }

    Some(lines.join("\n"))
}

/// Produce hover information for the symbol at the given position.
pub fn hover_info(
    cached_build: &crate::goto::CachedBuild,
    file_uri: &Url,
    position: Position,
    source_bytes: &[u8],
) -> Option<Hover> {
    let nodes = &cached_build.nodes;
    let path_to_abs = &cached_build.path_to_abs;
    let external_refs = &cached_build.external_refs;
    let id_to_path = &cached_build.id_to_path_map;
    let gas_index = &cached_build.gas_index;
    let doc_index = &cached_build.doc_index;
    let hint_index = &cached_build.hint_index;

    // Resolve the file path
    let file_path = file_uri.to_file_path().ok()?;
    let file_path_str = file_path.to_str()?;

    // Find the absolute path for this file
    let abs_path = path_to_abs
        .iter()
        .find(|(k, _)| file_path_str.ends_with(k.as_str()))
        .map(|(_, v)| v.clone())?;

    let byte_pos = pos_to_bytes(source_bytes, position);

    // Resolve: first try Yul external refs, then normal node lookup
    let node_id = byte_to_decl_via_external_refs(external_refs, id_to_path, &abs_path, byte_pos)
        .or_else(|| byte_to_id(nodes, &abs_path, byte_pos))?;

    // Get the NodeInfo for this node
    let node_info = nodes
        .values()
        .find_map(|file_nodes| file_nodes.get(&node_id))?;

    // Follow referenced_declaration to the declaration node
    let decl_id = node_info.referenced_declaration.unwrap_or(node_id);

    // Typed DeclNode — O(1) from decl_index
    let typed_decl = cached_build.decl_index.get(&decl_id);

    // Build hover content
    let mut parts: Vec<String> = Vec::new();

    // Signature in a code block
    if let Some(sig) = typed_decl.and_then(|d| d.build_signature()) {
        parts.push(format!("```solidity\n{sig}\n```"));
    } else if let Some(d) = typed_decl {
        // Fallback: show type description for any node
        if let Some(ts) = d.type_string() {
            parts.push(format!("```solidity\n{ts} {}\n```", d.name()));
        }
    }

    // Selector (function, error, or event)
    if let Some(selector) = typed_decl.and_then(|d| d.extract_typed_selector()) {
        parts.push(format!("Selector: `{}`", selector.to_prefixed()));
    }

    // Node ID for debugging: show the cursor-hit node and the resolved
    // declaration (if different, e.g. when hovering a reference).
    if node_id == decl_id {
        parts.push(format!("NodeId: `{}`", decl_id.0));
    } else {
        parts.push(format!(
            "NodeId: `{}` referencedDeclaration: `{}`",
            node_id.0, decl_id.0
        ));
    }

    let di = &cached_build.decl_index;
    let id_to_path = &cached_build.node_id_to_source_path;

    // Gas estimates — only shown when `@lsp-enable gas-estimates` is present
    if !gas_index.is_empty() {
        let source_str = String::from_utf8_lossy(source_bytes);
        if let Some(d) = typed_decl
            && source_has_gas_sentinel(&source_str, d.src())
        {
            if let Some(gas_text) =
                typed_decl.and_then(|d| gas_hover_for_function_typed(d, gas_index, di, id_to_path))
            {
                parts.push(gas_text);
            } else if let Some(gas_text) =
                typed_decl.and_then(|d| gas_hover_for_contract_typed(d, gas_index, di, id_to_path))
            {
                parts.push(gas_text);
            }
        }
    }

    // Documentation
    if let Some(doc_entry) =
        typed_decl.and_then(|d| lookup_doc_entry_typed(doc_index, d, di, id_to_path))
    {
        let formatted = format_doc_entry(&doc_entry);
        if !formatted.is_empty() {
            parts.push(format!("---\n{formatted}"));
        }
    } else if let Some(doc_text) = typed_decl.and_then(|d| d.extract_doc_text()) {
        let inherited_doc = typed_decl.and_then(|d| resolve_inheritdoc_typed(d, &doc_text, di));
        let formatted = format_natspec(&doc_text, inherited_doc.as_deref());
        if !formatted.is_empty() {
            parts.push(format!("---\n{formatted}"));
        }
    } else if let Some(param_doc) =
        typed_decl.and_then(|d| lookup_param_doc_typed(doc_index, d, di, id_to_path))
    {
        // Parameter/return value — show the @param/@return description from parent
        if !param_doc.is_empty() {
            parts.push(format!("---\n{param_doc}"));
        }
    }

    // Call-site parameter doc: when the hovered node is used as an argument
    // in a function call, show the @param doc from the called function's definition.
    // Uses tree-sitter on the live buffer to find the enclosing call and argument
    // index, then resolves via HintIndex for the param name and declaration id.
    if let Some(hint_lookup) = hint_index.get(&abs_path) {
        let source_str = String::from_utf8_lossy(source_bytes);
        if let Some(tree) = crate::inlay_hints::ts_parse(&source_str)
            && let Some(ctx) =
                crate::inlay_hints::ts_find_call_at_byte(tree.root_node(), &source_str, byte_pos)
            && let Some(resolved) = hint_lookup.resolve_callsite_param(
                ctx.call_start_byte,
                ctx.name,
                ctx.arg_count,
                ctx.arg_index,
            )
        {
            // Look up @param doc via typed DeclNode
            let typed_fn = di.get(&resolved.decl_id);
            let param_doc = typed_fn.and_then(|fn_decl| {
                // Try DocIndex first (structured devdoc)
                if let Some(doc_entry) = lookup_doc_entry_typed(doc_index, fn_decl, di, id_to_path)
                {
                    for (pname, pdesc) in &doc_entry.params {
                        if pname == &resolved.param_name {
                            return Some(pdesc.clone());
                        }
                    }
                }
                // Fallback: parse typed NatSpec on the function definition
                if let Some(doc_text) = fn_decl.extract_doc_text() {
                    let resolved_doc = if doc_text.contains("@inheritdoc") {
                        resolve_inheritdoc_typed(fn_decl, &doc_text, di)
                    } else {
                        None
                    };
                    let text = resolved_doc.as_deref().unwrap_or(&doc_text);
                    for line in text.lines() {
                        let trimmed = line.trim().trim_start_matches('*').trim();
                        if let Some(rest) = trimmed.strip_prefix("@param ")
                            && let Some((name, desc)) = rest.split_once(' ')
                            && name == resolved.param_name
                        {
                            return Some(desc.to_string());
                        }
                    }
                }
                None
            });
            if let Some(desc) = param_doc
                && !desc.is_empty()
            {
                parts.push(format!("**@param `{}`** — {desc}", resolved.param_name));
            }
        }
    }

    if parts.is_empty() {
        return None;
    }

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: parts.join("\n\n"),
        }),
        range: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_test_ast() -> Value {
        let data = std::fs::read_to_string("poolmanager.json").expect("test fixture");
        let raw: Value = serde_json::from_str(&data).expect("valid json");
        crate::solc::normalize_solc_output(raw, None)
    }

    #[test]
    fn test_find_node_by_id_pool_manager() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        let node = find_node_by_id(sources, NodeId(1216)).unwrap();
        assert_eq!(
            node.get("name").and_then(|v| v.as_str()),
            Some("PoolManager")
        );
        assert_eq!(
            node.get("nodeType").and_then(|v| v.as_str()),
            Some("ContractDefinition")
        );
    }

    #[test]
    fn test_find_node_by_id_initialize() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // IPoolManager.initialize has the full docs
        let node = find_node_by_id(sources, NodeId(2003)).unwrap();
        assert_eq!(
            node.get("name").and_then(|v| v.as_str()),
            Some("initialize")
        );
    }

    #[test]
    fn test_extract_documentation_object() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // IPoolManager.initialize (id=2003) has full NatSpec
        let node = find_node_by_id(sources, NodeId(2003)).unwrap();
        let doc = extract_documentation(node).unwrap();
        assert!(doc.contains("@notice"));
        assert!(doc.contains("@param key"));
    }

    #[test]
    fn test_extract_documentation_none() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // PoolKey struct (id=6871) — check if it has docs
        let node = find_node_by_id(sources, NodeId(6871)).unwrap();
        // PoolKey may or may not have docs, just verify no crash
        let _ = extract_documentation(node);
    }

    #[test]
    fn test_format_natspec_notice_and_params() {
        let text = "@notice Initialize the state for a given pool ID\n @param key The pool key\n @param sqrtPriceX96 The initial square root price\n @return tick The initial tick";
        let formatted = format_natspec(text, None);
        assert!(formatted.contains("Initialize the state"));
        assert!(formatted.contains("**Parameters:**"));
        assert!(formatted.contains("`key`"));
        assert!(formatted.contains("**Returns:**"));
        assert!(formatted.contains("`tick`"));
    }

    #[test]
    fn test_format_natspec_inheritdoc() {
        let text = "@inheritdoc IPoolManager";
        let formatted = format_natspec(text, None);
        assert!(formatted.contains("Inherits documentation from `IPoolManager`"));
    }

    #[test]
    fn test_format_natspec_dev() {
        let text = "@notice Do something\n @dev This is an implementation detail";
        let formatted = format_natspec(text, None);
        assert!(formatted.contains("Do something"));
        assert!(formatted.contains("**@dev**"));
        assert!(formatted.contains("*This is an implementation detail*"));
    }

    #[test]
    fn test_format_natspec_custom_tag() {
        let text = "@notice Do something\n @custom:security Non-reentrant";
        let formatted = format_natspec(text, None);
        assert!(formatted.contains("Do something"));
        assert!(formatted.contains("**@custom:security**"));
        assert!(formatted.contains("*Non-reentrant*"));
    }

    #[test]
    fn test_build_function_signature_initialize() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        let node = find_node_by_id(sources, NodeId(2003)).unwrap();
        let sig = build_function_signature(node).unwrap();
        assert!(sig.starts_with("function initialize("));
        assert!(sig.contains("returns"));
    }

    #[test]
    fn test_build_signature_contract() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        let node = find_node_by_id(sources, NodeId(1216)).unwrap();
        let sig = build_function_signature(node).unwrap();
        assert!(sig.contains("contract PoolManager"));
        assert!(sig.contains(" is "));
    }

    #[test]
    fn test_build_signature_struct() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        let node = find_node_by_id(sources, NodeId(6871)).unwrap();
        let sig = build_function_signature(node).unwrap();
        assert!(sig.starts_with("struct PoolKey"));
        assert!(sig.contains('{'));
    }

    #[test]
    fn test_build_signature_error() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // Find an ErrorDefinition
        let node = find_node_by_id(sources, NodeId(1372)).unwrap();
        assert_eq!(
            node.get("nodeType").and_then(|v| v.as_str()),
            Some("ErrorDefinition")
        );
        let sig = build_function_signature(node).unwrap();
        assert!(sig.starts_with("error "));
    }

    #[test]
    fn test_build_signature_event() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // Find an EventDefinition
        let node = find_node_by_id(sources, NodeId(7404)).unwrap();
        assert_eq!(
            node.get("nodeType").and_then(|v| v.as_str()),
            Some("EventDefinition")
        );
        let sig = build_function_signature(node).unwrap();
        assert!(sig.starts_with("event "));
    }

    #[test]
    fn test_build_signature_variable() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // Find a VariableDeclaration with documentation — check a state var
        // PoolManager has state variables, find one
        let pm = find_node_by_id(sources, NodeId(1216)).unwrap();
        if let Some(nodes) = pm.get("nodes").and_then(|v| v.as_array()) {
            for node in nodes {
                if node.get("nodeType").and_then(|v| v.as_str()) == Some("VariableDeclaration") {
                    let sig = build_function_signature(node);
                    assert!(sig.is_some());
                    break;
                }
            }
        }
    }

    #[test]
    fn test_pool_manager_has_documentation() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // Owned contract (id=7455) has NatSpec
        let node = find_node_by_id(sources, NodeId(7455)).unwrap();
        let doc = extract_documentation(node).unwrap();
        assert!(doc.contains("@notice"));
    }

    #[test]
    fn test_format_parameters_empty() {
        let result = format_parameters(None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_parameters_with_data() {
        let params: Value = serde_json::json!({
            "parameters": [
                {
                    "name": "key",
                    "typeDescriptions": { "typeString": "struct PoolKey" },
                    "storageLocation": "memory"
                },
                {
                    "name": "sqrtPriceX96",
                    "typeDescriptions": { "typeString": "uint160" },
                    "storageLocation": "default"
                }
            ]
        });
        let result = format_parameters(Some(&params));
        assert!(result.contains("struct PoolKey memory key"));
        assert!(result.contains("uint160 sqrtPriceX96"));
    }

    // --- Selector tests ---

    #[test]
    fn test_extract_selector_function() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // PoolManager.swap (id=616) has functionSelector "f3cd914c"
        let node = find_node_by_id(sources, NodeId(616)).unwrap();
        let selector = extract_selector(node).unwrap();
        assert_eq!(selector, Selector::Func(FuncSelector::new("f3cd914c")));
        assert_eq!(selector.as_hex(), "f3cd914c");
    }

    #[test]
    fn test_extract_selector_error() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // DelegateCallNotAllowed (id=1372) has errorSelector
        let node = find_node_by_id(sources, NodeId(1372)).unwrap();
        let selector = extract_selector(node).unwrap();
        assert_eq!(selector, Selector::Func(FuncSelector::new("0d89438e")));
        assert_eq!(selector.as_hex(), "0d89438e");
    }

    #[test]
    fn test_extract_selector_event() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // OwnershipTransferred (id=7404) has eventSelector
        let node = find_node_by_id(sources, NodeId(7404)).unwrap();
        let selector = extract_selector(node).unwrap();
        assert!(matches!(selector, Selector::Event(_)));
        assert_eq!(selector.as_hex().len(), 64); // 32-byte keccak hash
    }

    #[test]
    fn test_extract_selector_public_variable() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // owner (id=7406) is public, has functionSelector
        let node = find_node_by_id(sources, NodeId(7406)).unwrap();
        let selector = extract_selector(node).unwrap();
        assert_eq!(selector, Selector::Func(FuncSelector::new("8da5cb5b")));
    }

    #[test]
    fn test_extract_selector_internal_function_none() {
        let ast = load_test_ast();
        let sources = ast.get("sources").unwrap();
        // Pool.swap (id=5021) is internal, no selector
        let node = find_node_by_id(sources, NodeId(5021)).unwrap();
        assert!(extract_selector(node).is_none());
    }

    // --- @inheritdoc resolution tests ---

    #[test]
    fn test_resolve_inheritdoc_swap() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);
        // PoolManager.swap (id=616) has "@inheritdoc IPoolManager"
        let decl = build.decl_index.get(&NodeId(616)).unwrap();
        let doc_text = decl.extract_doc_text().unwrap();
        assert!(doc_text.contains("@inheritdoc"));

        let resolved = resolve_inheritdoc_typed(decl, &doc_text, &build.decl_index).unwrap();
        assert!(resolved.contains("@notice"));
        assert!(resolved.contains("Swap against the given pool"));
    }

    #[test]
    fn test_resolve_inheritdoc_initialize() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);
        // PoolManager.initialize (id=330) has "@inheritdoc IPoolManager"
        let decl = build.decl_index.get(&NodeId(330)).unwrap();
        let doc_text = decl.extract_doc_text().unwrap();

        let resolved = resolve_inheritdoc_typed(decl, &doc_text, &build.decl_index).unwrap();
        assert!(resolved.contains("Initialize the state"));
        assert!(resolved.contains("@param key"));
    }

    #[test]
    fn test_resolve_inheritdoc_extsload_overload() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);

        // extsload(bytes32) — id=1306, selector "1e2eaeaf"
        let decl = build.decl_index.get(&NodeId(1306)).unwrap();
        let doc_text = decl.extract_doc_text().unwrap();
        let resolved = resolve_inheritdoc_typed(decl, &doc_text, &build.decl_index).unwrap();
        assert!(resolved.contains("granular pool state"));
        assert!(resolved.contains("@param slot"));

        // extsload(bytes32, uint256) — id=1319, selector "35fd631a"
        let decl2 = build.decl_index.get(&NodeId(1319)).unwrap();
        let doc_text2 = decl2.extract_doc_text().unwrap();
        let resolved2 = resolve_inheritdoc_typed(decl2, &doc_text2, &build.decl_index).unwrap();
        assert!(resolved2.contains("@param startSlot"));

        // extsload(bytes32[]) — id=1331, selector "dbd035ff"
        let decl3 = build.decl_index.get(&NodeId(1331)).unwrap();
        let doc_text3 = decl3.extract_doc_text().unwrap();
        let resolved3 = resolve_inheritdoc_typed(decl3, &doc_text3, &build.decl_index).unwrap();
        assert!(resolved3.contains("sparse pool state"));
    }

    #[test]
    fn test_resolve_inheritdoc_formats_in_hover() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);
        // PoolManager.swap with @inheritdoc — verify format_natspec resolves it
        let decl = build.decl_index.get(&NodeId(616)).unwrap();
        let doc_text = decl.extract_doc_text().unwrap();
        let inherited = resolve_inheritdoc_typed(decl, &doc_text, &build.decl_index);
        let formatted = format_natspec(&doc_text, inherited.as_deref());
        assert!(!formatted.contains("@inheritdoc"));
        assert!(formatted.contains("Swap against the given pool"));
        assert!(formatted.contains("**Parameters:**"));
    }

    // --- Parameter/return doc tests ---

    #[test]
    fn test_param_doc_error_parameter() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);

        // PriceLimitAlreadyExceeded.sqrtPriceCurrentX96 (id=3821)
        let decl = build.decl_index.get(&NodeId(3821)).unwrap();
        assert_eq!(decl.name(), "sqrtPriceCurrentX96");

        let doc = lookup_param_doc_typed(
            &build.doc_index,
            decl,
            &build.decl_index,
            &build.node_id_to_source_path,
        )
        .unwrap();
        assert!(
            doc.contains("invalid"),
            "should describe the invalid price: {doc}"
        );
    }

    #[test]
    fn test_param_doc_error_second_parameter() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);

        // PriceLimitAlreadyExceeded.sqrtPriceLimitX96 (id=3823)
        let decl = build.decl_index.get(&NodeId(3823)).unwrap();
        let doc = lookup_param_doc_typed(
            &build.doc_index,
            decl,
            &build.decl_index,
            &build.node_id_to_source_path,
        )
        .unwrap();
        assert!(
            doc.contains("surpassed price limit"),
            "should describe the surpassed limit: {doc}"
        );
    }

    #[test]
    fn test_param_doc_function_return_value() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);

        // Pool.modifyLiquidity return param "delta" (id=4055)
        let decl = build.decl_index.get(&NodeId(4055)).unwrap();
        assert_eq!(decl.name(), "delta");

        let doc = lookup_param_doc_typed(
            &build.doc_index,
            decl,
            &build.decl_index,
            &build.node_id_to_source_path,
        )
        .unwrap();
        assert!(
            doc.contains("deltas of the token balances"),
            "should have return doc: {doc}"
        );
    }

    #[test]
    fn test_param_doc_function_input_parameter() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);

        // Pool.modifyLiquidity input param "params" — find by scanning decl_index
        // The function id=4371 has a parameter named "params"
        let params_decl = build
            .decl_index
            .values()
            .find(|d| d.name() == "params" && d.scope() == Some(4371))
            .unwrap();

        let doc = lookup_param_doc_typed(
            &build.doc_index,
            params_decl,
            &build.decl_index,
            &build.node_id_to_source_path,
        )
        .unwrap();
        assert!(
            doc.contains("position details"),
            "should have param doc: {doc}"
        );
    }

    #[test]
    fn test_param_doc_inherited_function_via_docindex() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);

        // PoolManager.swap `key` param (id=478) — parent has @inheritdoc IPoolManager
        let decl = build.decl_index.get(&NodeId(478)).unwrap();
        assert_eq!(decl.name(), "key");

        let doc = lookup_param_doc_typed(
            &build.doc_index,
            decl,
            &build.decl_index,
            &build.node_id_to_source_path,
        )
        .unwrap();
        assert!(
            doc.contains("pool to swap"),
            "should have inherited param doc: {doc}"
        );
    }

    #[test]
    fn test_param_doc_non_parameter_returns_none() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);

        // PoolManager contract (id=1216) is not a parameter
        let decl = build.decl_index.get(&NodeId(1216)).unwrap();
        assert!(
            lookup_param_doc_typed(
                &build.doc_index,
                decl,
                &build.decl_index,
                &build.node_id_to_source_path,
            )
            .is_none()
        );
    }

    // ── DocIndex integration tests (poolmanager.json) ──

    fn load_solc_fixture() -> Value {
        let data = std::fs::read_to_string("poolmanager.json").expect("test fixture");
        let raw: Value = serde_json::from_str(&data).expect("valid json");
        crate::solc::normalize_solc_output(raw, None)
    }

    #[test]
    fn test_doc_index_is_not_empty() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);
        assert!(!index.is_empty(), "DocIndex should contain entries");
    }

    #[test]
    fn test_doc_index_has_contract_entries() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // PoolManager has both title and notice
        let pm_keys: Vec<_> = index
            .keys()
            .filter(|k| matches!(k, DocKey::Contract(s) if s.contains("PoolManager")))
            .collect();
        assert!(
            !pm_keys.is_empty(),
            "should have a Contract entry for PoolManager"
        );

        let pm_key = DocKey::Contract(
            "/Users/meek/developer/mmsaki/solidity-language-server/v4-core/src/PoolManager.sol:PoolManager".to_string(),
        );
        let entry = index.get(&pm_key).expect("PoolManager contract entry");
        assert_eq!(entry.title.as_deref(), Some("PoolManager"));
        assert_eq!(
            entry.notice.as_deref(),
            Some("Holds the state for all pools")
        );
    }

    #[test]
    fn test_doc_index_has_function_by_selector() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // initialize selector = 6276cbbe
        let init_key = DocKey::Func(FuncSelector::new("6276cbbe"));
        let entry = index
            .get(&init_key)
            .expect("should have initialize by selector");
        assert_eq!(
            entry.notice.as_deref(),
            Some("Initialize the state for a given pool ID")
        );
        assert!(
            entry
                .details
                .as_deref()
                .unwrap_or("")
                .contains("MAX_SWAP_FEE"),
            "devdoc details should mention MAX_SWAP_FEE"
        );
        // params: key, sqrtPriceX96
        let param_names: Vec<&str> = entry.params.iter().map(|(n, _)| n.as_str()).collect();
        assert!(param_names.contains(&"key"), "should have param 'key'");
        assert!(
            param_names.contains(&"sqrtPriceX96"),
            "should have param 'sqrtPriceX96'"
        );
        // returns: tick
        let return_names: Vec<&str> = entry.returns.iter().map(|(n, _)| n.as_str()).collect();
        assert!(return_names.contains(&"tick"), "should have return 'tick'");
    }

    #[test]
    fn test_doc_index_swap_by_selector() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // swap selector = f3cd914c
        let swap_key = DocKey::Func(FuncSelector::new("f3cd914c"));
        let entry = index.get(&swap_key).expect("should have swap by selector");
        assert!(
            entry
                .notice
                .as_deref()
                .unwrap_or("")
                .contains("Swap against the given pool"),
            "swap notice should describe swapping"
        );
        // devdoc params: key, params, hookData
        assert!(
            !entry.params.is_empty(),
            "swap should have param documentation"
        );
    }

    #[test]
    fn test_doc_index_settle_by_selector() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // settle() selector = 11da60b4
        let key = DocKey::Func(FuncSelector::new("11da60b4"));
        let entry = index.get(&key).expect("should have settle by selector");
        assert!(
            entry.notice.is_some(),
            "settle should have a notice from userdoc"
        );
    }

    #[test]
    fn test_doc_index_has_error_entries() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // AlreadyUnlocked() → keccak256("AlreadyUnlocked()")[0..4]
        let selector = compute_selector("AlreadyUnlocked()");
        let key = DocKey::Func(FuncSelector::new(&selector));
        let entry = index.get(&key).expect("should have AlreadyUnlocked error");
        assert!(
            entry
                .notice
                .as_deref()
                .unwrap_or("")
                .contains("already unlocked"),
            "AlreadyUnlocked notice: {:?}",
            entry.notice
        );
    }

    #[test]
    fn test_doc_index_error_with_params() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // CurrenciesOutOfOrderOrEqual(address,address) has a notice
        let selector = compute_selector("CurrenciesOutOfOrderOrEqual(address,address)");
        let key = DocKey::Func(FuncSelector::new(&selector));
        let entry = index
            .get(&key)
            .expect("should have CurrenciesOutOfOrderOrEqual error");
        assert!(entry.notice.is_some(), "error should have notice");
    }

    #[test]
    fn test_doc_index_has_event_entries() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // Count event entries
        let event_count = index
            .keys()
            .filter(|k| matches!(k, DocKey::Event(_)))
            .count();
        assert!(event_count > 0, "should have event entries in the DocIndex");
    }

    #[test]
    fn test_doc_index_swap_event() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // Swap event topic = keccak256("Swap(bytes32,address,int128,int128,uint160,uint128,int24,uint24)")
        let topic =
            compute_event_topic("Swap(bytes32,address,int128,int128,uint160,uint128,int24,uint24)");
        let key = DocKey::Event(EventSelector::new(&topic));
        let entry = index.get(&key).expect("should have Swap event");

        // userdoc notice
        assert!(
            entry
                .notice
                .as_deref()
                .unwrap_or("")
                .contains("swaps between currency0 and currency1"),
            "Swap event notice: {:?}",
            entry.notice
        );

        // devdoc params (amount0, amount1, id, sender, sqrtPriceX96, etc.)
        let param_names: Vec<&str> = entry.params.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            param_names.contains(&"amount0"),
            "should have param 'amount0'"
        );
        assert!(
            param_names.contains(&"sender"),
            "should have param 'sender'"
        );
        assert!(param_names.contains(&"id"), "should have param 'id'");
    }

    #[test]
    fn test_doc_index_initialize_event() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        let topic = compute_event_topic(
            "Initialize(bytes32,address,address,uint24,int24,address,uint160,int24)",
        );
        let key = DocKey::Event(EventSelector::new(&topic));
        let entry = index.get(&key).expect("should have Initialize event");
        assert!(
            !entry.params.is_empty(),
            "Initialize event should have param docs"
        );
    }

    #[test]
    fn test_doc_index_no_state_variables_for_pool_manager() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // PoolManager has no devdoc.stateVariables, so no StateVar keys for it
        let sv_count = index
            .keys()
            .filter(|k| matches!(k, DocKey::StateVar(s) if s.contains("PoolManager")))
            .count();
        assert_eq!(
            sv_count, 0,
            "PoolManager should have no state variable doc entries"
        );
    }

    #[test]
    fn test_doc_index_multiple_contracts() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // Should have contract entries for multiple contracts (ERC6909, Extsload, IPoolManager, etc.)
        let contract_count = index
            .keys()
            .filter(|k| matches!(k, DocKey::Contract(_)))
            .count();
        assert!(
            contract_count >= 5,
            "should have at least 5 contract-level entries, got {contract_count}"
        );
    }

    #[test]
    fn test_doc_index_func_key_count() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        let func_count = index
            .keys()
            .filter(|k| matches!(k, DocKey::Func(_)))
            .count();
        // We have methods + errors keyed by selector across all 43 contracts
        assert!(
            func_count >= 30,
            "should have at least 30 Func entries (methods + errors), got {func_count}"
        );
    }

    #[test]
    fn test_doc_index_format_initialize_entry() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        let key = DocKey::Func(FuncSelector::new("6276cbbe"));
        let entry = index.get(&key).expect("initialize entry");
        let formatted = format_doc_entry(entry);

        assert!(
            formatted.contains("Initialize the state for a given pool ID"),
            "formatted should include notice"
        );
        assert!(
            formatted.contains("**@dev**"),
            "formatted should include dev section"
        );
        assert!(
            formatted.contains("**Parameters:**"),
            "formatted should include parameters"
        );
        assert!(
            formatted.contains("`key`"),
            "formatted should include key param"
        );
        assert!(
            formatted.contains("**Returns:**"),
            "formatted should include returns"
        );
        assert!(
            formatted.contains("`tick`"),
            "formatted should include tick return"
        );
    }

    #[test]
    fn test_doc_index_format_contract_entry() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        let key = DocKey::Contract(
            "/Users/meek/developer/mmsaki/solidity-language-server/v4-core/src/PoolManager.sol:PoolManager".to_string(),
        );
        let entry = index.get(&key).expect("PoolManager contract entry");
        let formatted = format_doc_entry(entry);

        assert!(
            formatted.contains("**PoolManager**"),
            "should include bold title"
        );
        assert!(
            formatted.contains("Holds the state for all pools"),
            "should include notice"
        );
    }

    #[test]
    fn test_doc_index_inherited_docs_resolved() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // Both PoolManager and IPoolManager define methods with the same selector.
        // The last one written wins (PoolManager overwrites IPoolManager for same selector).
        // Either way, swap(f3cd914c) should have full docs, not just "@inheritdoc".
        let key = DocKey::Func(FuncSelector::new("f3cd914c"));
        let entry = index.get(&key).expect("swap entry");
        // The notice should be the resolved text, not "@inheritdoc IPoolManager"
        let notice = entry.notice.as_deref().unwrap_or("");
        assert!(
            !notice.contains("@inheritdoc"),
            "userdoc/devdoc should have resolved inherited docs, not raw @inheritdoc"
        );
    }

    #[test]
    fn test_compute_selector_known_values() {
        // keccak256("AlreadyUnlocked()") first 4 bytes
        let sel = compute_selector("AlreadyUnlocked()");
        assert_eq!(sel.len(), 8, "selector should be 8 hex chars");

        // Verify against a known selector from evm.methodIdentifiers
        let init_sel =
            compute_selector("initialize((address,address,uint24,int24,address),uint160)");
        assert_eq!(
            init_sel, "6276cbbe",
            "computed initialize selector should match evm.methodIdentifiers"
        );
    }

    #[test]
    fn test_compute_event_topic_length() {
        let topic =
            compute_event_topic("Swap(bytes32,address,int128,int128,uint160,uint128,int24,uint24)");
        assert_eq!(
            topic.len(),
            64,
            "event topic should be 64 hex chars (32 bytes)"
        );
    }

    #[test]
    fn test_doc_index_error_count_poolmanager() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // PoolManager userdoc has 14 errors. Check that they're all indexed.
        // Compute selectors for all 14 error signatures and verify they exist.
        let error_sigs = [
            "AlreadyUnlocked()",
            "CurrenciesOutOfOrderOrEqual(address,address)",
            "CurrencyNotSettled()",
            "InvalidCaller()",
            "ManagerLocked()",
            "MustClearExactPositiveDelta()",
            "NonzeroNativeValue()",
            "PoolNotInitialized()",
            "ProtocolFeeCurrencySynced()",
            "ProtocolFeeTooLarge(uint24)",
            "SwapAmountCannotBeZero()",
            "TickSpacingTooLarge(int24)",
            "TickSpacingTooSmall(int24)",
            "UnauthorizedDynamicLPFeeUpdate()",
        ];
        let mut found = 0;
        for sig in &error_sigs {
            let selector = compute_selector(sig);
            let key = DocKey::Func(FuncSelector::new(&selector));
            if index.contains_key(&key) {
                found += 1;
            }
        }
        assert_eq!(
            found,
            error_sigs.len(),
            "all 14 PoolManager errors should be in the DocIndex"
        );
    }

    #[test]
    fn test_doc_index_extsload_overloads_have_different_selectors() {
        let ast = load_solc_fixture();
        let index = build_doc_index(&ast);

        // Three extsload overloads should each have their own selector entry
        // extsload(bytes32) = 1e2eaeaf
        // extsload(bytes32,uint256) = 35fd631a
        // extsload(bytes32[]) = dbd035ff
        let sel1 = DocKey::Func(FuncSelector::new("1e2eaeaf"));
        let sel2 = DocKey::Func(FuncSelector::new("35fd631a"));
        let sel3 = DocKey::Func(FuncSelector::new("dbd035ff"));

        assert!(index.contains_key(&sel1), "extsload(bytes32) should exist");
        assert!(
            index.contains_key(&sel2),
            "extsload(bytes32,uint256) should exist"
        );
        assert!(
            index.contains_key(&sel3),
            "extsload(bytes32[]) should exist"
        );
    }

    #[test]
    fn test_signature_help_parameter_offsets() {
        // Simulate a signature like: "function addTax(uint256 amount, uint16 tax, uint16 base)"
        let label = "function addTax(uint256 amount, uint16 tax, uint16 base)";
        let param_strings = vec![
            "uint256 amount".to_string(),
            "uint16 tax".to_string(),
            "uint16 base".to_string(),
        ];

        let params_start = label.find('(').unwrap() + 1;
        let mut offsets = Vec::new();
        let mut offset = params_start;
        for param_str in &param_strings {
            let start = offset;
            let end = start + param_str.len();
            offsets.push((start, end));
            offset = end + 2; // ", "
        }

        // Verify the offsets correctly slice the label
        assert_eq!(&label[offsets[0].0..offsets[0].1], "uint256 amount");
        assert_eq!(&label[offsets[1].0..offsets[1].1], "uint16 tax");
        assert_eq!(&label[offsets[2].0..offsets[2].1], "uint16 base");
    }

    // ── Typed mapping signature help tests ────────────

    #[test]
    fn find_mapping_decl_typed_pools() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);
        let decl = find_mapping_decl_typed(&build.decl_index, "_pools").unwrap();
        assert_eq!(decl.name, "_pools");
        assert!(matches!(
            decl.type_name.as_ref(),
            Some(crate::solc_ast::TypeName::Mapping(_))
        ));
    }

    #[test]
    fn find_mapping_decl_typed_not_found() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);
        assert!(find_mapping_decl_typed(&build.decl_index, "nonexistent").is_none());
    }

    #[test]
    fn mapping_signature_help_typed_pools() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);
        let help = mapping_signature_help_typed(&build.decl_index, "_pools").unwrap();
        assert!(help.signatures[0].label.contains("_pools"));
        let params = help.signatures[0].parameters.as_ref().unwrap();
        assert!(!params.is_empty());
    }

    #[test]
    fn mapping_signature_help_typed_protocol_fees() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);
        let help = mapping_signature_help_typed(&build.decl_index, "protocolFeesAccrued").unwrap();
        assert!(help.signatures[0].label.contains("protocolFeesAccrued"));
    }

    #[test]
    fn mapping_signature_help_typed_non_mapping() {
        let ast = load_test_ast();
        let build = crate::goto::CachedBuild::new(ast, 0, None);
        assert!(mapping_signature_help_typed(&build.decl_index, "owner").is_none());
    }
}
