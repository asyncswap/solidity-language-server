#![allow(deprecated)]

use tower_lsp::lsp_types::{DocumentSymbol, Location, Range, SymbolInformation, SymbolKind, Url};
use tree_sitter::{Node, Parser};

// ── Document symbols (hierarchical, single file) ───────────────────────────

/// Extract hierarchical document symbols from Solidity source using tree-sitter.
pub fn extract_document_symbols(source: &str) -> Vec<DocumentSymbol> {
    let tree = match parse(source) {
        Some(t) => t,
        None => return vec![],
    };
    collect_top_level(tree.root_node(), source)
}

fn collect_top_level(node: Node, source: &str) -> Vec<DocumentSymbol> {
    named_children(node)
        .filter_map(|child| match child.kind() {
            "pragma_directive" => Some(text_symbol(child, source, SymbolKind::STRING)),
            "import_directive" => Some(import_symbol(child, source)),
            "contract_declaration" => contract_symbol(child, source, SymbolKind::CLASS),
            "interface_declaration" => contract_symbol(child, source, SymbolKind::INTERFACE),
            "library_declaration" => contract_symbol(child, source, SymbolKind::NAMESPACE),
            "struct_declaration" => struct_symbol(child, source),
            "enum_declaration" => enum_symbol(child, source),
            "function_definition" => function_symbol(child, source),
            "event_definition" | "error_declaration" => id_symbol(child, source, SymbolKind::EVENT),
            "state_variable_declaration" => id_symbol(child, source, SymbolKind::FIELD),
            "user_defined_type_definition" => id_symbol(child, source, SymbolKind::TYPE_PARAMETER),
            _ => None,
        })
        .collect()
}

fn collect_contract_members(body: Node, source: &str) -> Vec<DocumentSymbol> {
    named_children(body)
        .filter_map(|child| match child.kind() {
            "function_definition" => function_symbol(child, source),
            "constructor_definition" => {
                Some(leaf("constructor", SymbolKind::CONSTRUCTOR, child, source))
            }
            "fallback_receive_definition" => Some(leaf(
                &fallback_or_receive(child, source),
                SymbolKind::FUNCTION,
                child,
                source,
            )),
            "state_variable_declaration" => id_symbol(child, source, SymbolKind::FIELD),
            "event_definition" | "error_declaration" => id_symbol(child, source, SymbolKind::EVENT),
            "modifier_definition" => id_symbol(child, source, SymbolKind::METHOD),
            "struct_declaration" => struct_symbol(child, source),
            "enum_declaration" => enum_symbol(child, source),
            "using_directive" => Some(text_symbol(child, source, SymbolKind::PROPERTY)),
            "user_defined_type_definition" => id_symbol(child, source, SymbolKind::TYPE_PARAMETER),
            _ => None,
        })
        .collect()
}

// ── Symbol builders ────────────────────────────────────────────────────────

fn contract_symbol(node: Node, source: &str, kind: SymbolKind) -> Option<DocumentSymbol> {
    let name = child_id_text(node, source)?;
    let children = find_child(node, "contract_body")
        .map(|body| collect_contract_members(body, source))
        .filter(|c| !c.is_empty());

    Some(DocumentSymbol {
        name: name.into(),
        detail: None,
        kind,
        range: range(node, source),
        selection_range: child_id_range(node, source)?,
        children,
        tags: None,
        deprecated: None,
    })
}

fn function_symbol(node: Node, source: &str) -> Option<DocumentSymbol> {
    let name = child_id_text(node, source)?;
    Some(DocumentSymbol {
        name: name.into(),
        detail: Some(function_detail(node, source)),
        kind: SymbolKind::FUNCTION,
        range: range(node, source),
        selection_range: child_id_range(node, source)?,
        children: None,
        tags: None,
        deprecated: None,
    })
}

fn struct_symbol(node: Node, source: &str) -> Option<DocumentSymbol> {
    let name = child_id_text(node, source)?;
    let children = find_child(node, "struct_body")
        .map(|body| {
            named_children(body)
                .filter(|c| c.kind() == "struct_member")
                .filter_map(|c| id_symbol(c, source, SymbolKind::FIELD))
                .collect::<Vec<_>>()
        })
        .filter(|c| !c.is_empty());

    Some(DocumentSymbol {
        name: name.into(),
        detail: None,
        kind: SymbolKind::STRUCT,
        range: range(node, source),
        selection_range: child_id_range(node, source)?,
        children,
        tags: None,
        deprecated: None,
    })
}

fn enum_symbol(node: Node, source: &str) -> Option<DocumentSymbol> {
    let name = child_id_text(node, source)?;
    let children = find_child(node, "enum_body")
        .map(|body| {
            named_children(body)
                .filter(|c| c.kind() == "enum_value")
                .map(|c| leaf(&source[c.byte_range()], SymbolKind::ENUM_MEMBER, c, source))
                .collect::<Vec<_>>()
        })
        .filter(|c| !c.is_empty());

    Some(DocumentSymbol {
        name: name.into(),
        detail: None,
        kind: SymbolKind::ENUM,
        range: range(node, source),
        selection_range: child_id_range(node, source)?,
        children,
        tags: None,
        deprecated: None,
    })
}

/// Symbol whose name comes from its first `identifier` child.
fn id_symbol(node: Node, source: &str, kind: SymbolKind) -> Option<DocumentSymbol> {
    let name = child_id_text(node, source)?;
    Some(DocumentSymbol {
        name: name.into(),
        detail: None,
        kind,
        range: range(node, source),
        selection_range: child_id_range(node, source).unwrap_or_else(|| range(node, source)),
        children: None,
        tags: None,
        deprecated: None,
    })
}

/// Symbol whose name is the full node text (pragmas, using directives).
fn text_symbol(node: Node, source: &str, kind: SymbolKind) -> DocumentSymbol {
    let text = source[node.byte_range()].trim_end_matches(';').trim();
    leaf(text, kind, node, source)
}

fn import_symbol(node: Node, source: &str) -> DocumentSymbol {
    let name = find_child(node, "string")
        .map(|s| format!("import {}", &source[s.byte_range()]))
        .unwrap_or_else(|| {
            source[node.byte_range()]
                .trim_end_matches(';')
                .trim()
                .into()
        });
    leaf(&name, SymbolKind::MODULE, node, source)
}

/// Leaf symbol with no children — range equals selection_range.
fn leaf(name: &str, kind: SymbolKind, node: Node, source: &str) -> DocumentSymbol {
    DocumentSymbol {
        name: name.into(),
        detail: None,
        kind,
        range: range(node, source),
        selection_range: range(node, source),
        children: None,
        tags: None,
        deprecated: None,
    }
}

fn function_detail(node: Node, source: &str) -> String {
    let params: Vec<&str> = named_children(node)
        .filter(|c| c.kind() == "parameter")
        .map(|c| source[c.byte_range()].trim())
        .collect();

    let returns: Vec<&str> = find_child(node, "return_type_definition")
        .map(|ret| {
            named_children(ret)
                .filter(|c| c.kind() == "parameter")
                .map(|c| source[c.byte_range()].trim())
                .collect()
        })
        .unwrap_or_default();

    let mut sig = format!("({})", params.join(", "));
    if !returns.is_empty() {
        sig.push_str(&format!(" returns ({})", returns.join(", ")));
    }
    sig
}

// ── Workspace symbols (flat, multi-file) ───────────────────────────────────

/// Extract flat workspace symbols from multiple files.
pub fn extract_workspace_symbols(files: &[(Url, String)]) -> Vec<SymbolInformation> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_solidity::LANGUAGE.into())
        .expect("failed to load Solidity grammar");

    let mut symbols = Vec::new();
    for (uri, source) in files {
        if let Some(tree) = parser.parse(source, None) {
            collect_workspace_symbols(tree.root_node(), source, uri, None, &mut symbols);
        }
    }
    symbols
}

fn collect_workspace_symbols(
    node: Node,
    source: &str,
    uri: &Url,
    container: Option<&str>,
    out: &mut Vec<SymbolInformation>,
) {
    for child in named_children(node) {
        match child.kind() {
            // Containers: recurse into body
            "contract_declaration" | "interface_declaration" | "library_declaration" => {
                let kind = match child.kind() {
                    "interface_declaration" => SymbolKind::INTERFACE,
                    "library_declaration" => SymbolKind::NAMESPACE,
                    _ => SymbolKind::CLASS,
                };
                if let Some(name) = child_id_text(child, source) {
                    push_info(out, name, kind, child, source, uri, container);
                    if let Some(body) = find_child(child, "contract_body") {
                        collect_workspace_symbols(body, source, uri, Some(name), out);
                    }
                }
            }
            "struct_declaration" => {
                if let Some(name) = child_id_text(child, source) {
                    push_info(out, name, SymbolKind::STRUCT, child, source, uri, container);
                    if let Some(body) = find_child(child, "struct_body") {
                        collect_workspace_symbols(body, source, uri, Some(name), out);
                    }
                }
            }
            "enum_declaration" => {
                if let Some(name) = child_id_text(child, source) {
                    push_info(out, name, SymbolKind::ENUM, child, source, uri, container);
                    if let Some(body) = find_child(child, "enum_body") {
                        collect_workspace_symbols(body, source, uri, Some(name), out);
                    }
                }
            }
            // Leaves
            "function_definition" => {
                push_id(out, child, source, SymbolKind::FUNCTION, uri, container)
            }
            "constructor_definition" => push_info(
                out,
                "constructor",
                SymbolKind::CONSTRUCTOR,
                child,
                source,
                uri,
                container,
            ),
            "state_variable_declaration" | "struct_member" => {
                push_id(out, child, source, SymbolKind::FIELD, uri, container)
            }
            "event_definition" | "error_declaration" => {
                push_id(out, child, source, SymbolKind::EVENT, uri, container)
            }
            "modifier_definition" => {
                push_id(out, child, source, SymbolKind::METHOD, uri, container)
            }
            "enum_value" => push_info(
                out,
                &source[child.byte_range()],
                SymbolKind::ENUM_MEMBER,
                child,
                source,
                uri,
                container,
            ),
            "user_defined_type_definition" => push_id(
                out,
                child,
                source,
                SymbolKind::TYPE_PARAMETER,
                uri,
                container,
            ),
            _ => {}
        }
    }
}

fn push_id(
    out: &mut Vec<SymbolInformation>,
    node: Node,
    source: &str,
    kind: SymbolKind,
    uri: &Url,
    container: Option<&str>,
) {
    if let Some(name) = child_id_text(node, source) {
        push_info(out, name, kind, node, source, uri, container);
    }
}

fn push_info(
    out: &mut Vec<SymbolInformation>,
    name: &str,
    kind: SymbolKind,
    node: Node,
    source: &str,
    uri: &Url,
    container: Option<&str>,
) {
    out.push(SymbolInformation {
        name: name.into(),
        kind,
        tags: None,
        deprecated: None,
        location: Location {
            uri: uri.clone(),
            range: range(node, source),
        },
        container_name: container.map(Into::into),
    });
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn parse(source: &str) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_solidity::LANGUAGE.into())
        .expect("failed to load Solidity grammar");
    parser.parse(source, None)
}

/// Tree-sitter reports columns in bytes; LSP expects them in the negotiated
/// encoding (UTF-16 unless the client opted into UTF-8). Convert through
/// `byte_column_to_position`, which measures only the current line's prefix —
/// converting via absolute byte offsets here would rescan the file per node and
/// make whole-tree walks quadratic.
fn range(node: Node, source: &str) -> Range {
    let s = node.start_position();
    let e = node.end_position();
    Range {
        start: crate::utils::byte_column_to_position(source, s.row, s.column, node.start_byte()),
        end: crate::utils::byte_column_to_position(source, e.row, e.column, node.end_byte()),
    }
}

fn named_children(node: Node) -> impl Iterator<Item = Node> {
    let mut cursor = node.walk();
    let children: Vec<Node> = node
        .children(&mut cursor)
        .filter(|c| c.is_named())
        .collect();
    children.into_iter()
}

fn child_id_text<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|c| c.kind() == "identifier" && c.is_named())
        .map(|c| &source[c.byte_range()])
}

fn child_id_range(node: Node, source: &str) -> Option<Range> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|c| c.kind() == "identifier" && c.is_named())
        .map(|c| range(c, source))
}

fn find_child<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|c| c.kind() == kind)
}

fn fallback_or_receive(node: Node, source: &str) -> String {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|c| !c.is_named() && matches!(&source[c.byte_range()], "fallback" | "receive"))
        .map(|c| source[c.byte_range()].into())
        .unwrap_or_else(|| "fallback".into())
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        assert!(extract_document_symbols("").is_empty());
    }

    #[test]
    fn test_simple_contract() {
        let source = r#"
pragma solidity ^0.8.0;

contract Counter {
    uint256 public count;
    function increment() public { count += 1; }
    function getCount() public view returns (uint256) { return count; }
}
"#;
        let symbols = extract_document_symbols(source);
        assert!(symbols.len() >= 2);

        let contract = symbols
            .iter()
            .find(|s| s.kind == SymbolKind::CLASS)
            .unwrap();
        assert_eq!(contract.name, "Counter");

        let children = contract.children.as_ref().unwrap();
        assert!(
            children
                .iter()
                .any(|c| c.name == "count" && c.kind == SymbolKind::FIELD)
        );
        assert!(
            children
                .iter()
                .any(|c| c.name == "increment" && c.kind == SymbolKind::FUNCTION)
        );
        assert!(
            children
                .iter()
                .any(|c| c.name == "getCount" && c.kind == SymbolKind::FUNCTION)
        );
    }

    #[test]
    fn test_struct_with_members() {
        let source = "contract Foo { struct Info { string name; uint256 value; } }";
        let symbols = extract_document_symbols(source);
        let members = symbols[0]
            .children
            .as_ref()
            .unwrap()
            .iter()
            .find(|c| c.kind == SymbolKind::STRUCT)
            .unwrap()
            .children
            .as_ref()
            .unwrap();
        assert_eq!(members.len(), 2);
        assert!(members.iter().any(|m| m.name == "name"));
        assert!(members.iter().any(|m| m.name == "value"));
    }

    #[test]
    fn test_enum_with_values() {
        let source = "contract Foo { enum Status { Active, Paused, Stopped } }";
        let symbols = extract_document_symbols(source);
        let members = symbols[0]
            .children
            .as_ref()
            .unwrap()
            .iter()
            .find(|c| c.kind == SymbolKind::ENUM)
            .unwrap()
            .children
            .as_ref()
            .unwrap();
        assert_eq!(members.len(), 3);
        assert!(members.iter().any(|m| m.name == "Active"));
        assert!(members.iter().any(|m| m.name == "Paused"));
        assert!(members.iter().any(|m| m.name == "Stopped"));
    }

    #[test]
    fn test_all_member_types() {
        let source = r#"
contract Token {
    event Transfer(address from, address to, uint256 value);
    error Unauthorized();
    uint256 public totalSupply;
    modifier onlyOwner() { _; }
    constructor() {}
    function transfer(address to, uint256 amount) external returns (bool) { return true; }
    fallback() external payable {}
    receive() external payable {}
    type Price is uint256;
}
"#;
        let children = extract_document_symbols(source)
            .into_iter()
            .find(|s| s.kind == SymbolKind::CLASS)
            .unwrap()
            .children
            .unwrap();

        assert!(
            children
                .iter()
                .any(|c| c.name == "Transfer" && c.kind == SymbolKind::EVENT)
        );
        assert!(
            children
                .iter()
                .any(|c| c.name == "Unauthorized" && c.kind == SymbolKind::EVENT)
        );
        assert!(
            children
                .iter()
                .any(|c| c.name == "totalSupply" && c.kind == SymbolKind::FIELD)
        );
        assert!(
            children
                .iter()
                .any(|c| c.name == "onlyOwner" && c.kind == SymbolKind::METHOD)
        );
        assert!(
            children
                .iter()
                .any(|c| c.name == "constructor" && c.kind == SymbolKind::CONSTRUCTOR)
        );
        assert!(
            children
                .iter()
                .any(|c| c.name == "transfer" && c.kind == SymbolKind::FUNCTION)
        );
        assert!(
            children
                .iter()
                .any(|c| c.name == "fallback" && c.kind == SymbolKind::FUNCTION)
        );
        assert!(
            children
                .iter()
                .any(|c| c.name == "receive" && c.kind == SymbolKind::FUNCTION)
        );
        assert!(
            children
                .iter()
                .any(|c| c.name == "Price" && c.kind == SymbolKind::TYPE_PARAMETER)
        );
    }

    #[test]
    fn test_interface_and_library() {
        let source = r#"
interface IToken { function transfer(address to, uint256 amount) external returns (bool); }
library SafeMath { function add(uint256 a, uint256 b) internal pure returns (uint256) { return a + b; } }
"#;
        let symbols = extract_document_symbols(source);
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "IToken" && s.kind == SymbolKind::INTERFACE)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "SafeMath" && s.kind == SymbolKind::NAMESPACE)
        );
    }

    #[test]
    fn test_workspace_symbols() {
        let uri = Url::parse("file:///test.sol").unwrap();
        let source = "contract Foo { uint256 public bar; function baz() public {} }";
        let symbols = extract_workspace_symbols(&[(uri, source.into())]);
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "Foo" && s.kind == SymbolKind::CLASS)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "bar" && s.container_name.as_deref() == Some("Foo"))
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "baz" && s.container_name.as_deref() == Some("Foo"))
        );
    }

    #[test]
    fn test_shop_sol() {
        let source = std::fs::read_to_string("example/Shop.sol").unwrap();
        let symbols = extract_document_symbols(&source);
        // Should find both Transaction library and Shop contract
        assert!(symbols.iter().any(|s| s.name == "Transaction"));
        let shop = symbols.iter().find(|s| s.name == "Shop").unwrap();
        let children = shop.children.as_ref().unwrap();
        assert!(children.iter().any(|c| c.name == "buy"));
        assert!(children.iter().any(|c| c.name == "withdraw"));
        assert!(children.iter().any(|c| c.name == "refund"));
    }

    #[test]
    fn test_function_detail() {
        let source = "contract Foo { function bar(uint256 x, address y) public pure returns (bool) { return true; } }";
        let func = extract_document_symbols(source)[0]
            .children
            .as_ref()
            .unwrap()
            .iter()
            .find(|c| c.name == "bar")
            .unwrap()
            .clone();
        let detail = func.detail.unwrap();
        assert!(detail.contains("uint256 x"));
        assert!(detail.contains("address y"));
        assert!(detail.contains("returns"));
    }

    /// Columns are reported in UTF-16 code units (the LSP default), not bytes,
    /// so a non-ASCII literal earlier on the line must not shift later symbols.
    #[test]
    fn test_symbol_columns_are_utf16_not_bytes() {
        // `é` is 2 bytes / 1 UTF-16 unit, `🚀` is 4 bytes / 2 units, so the
        // byte column of `priceTotal` runs 3 ahead of its UTF-16 column.
        let line = r#"    string greeting = unicode"héllo 🚀"; uint256 priceTotal;"#;
        let source = format!("contract C {{\n{line}\n}}");

        let char_idx = line.find("priceTotal").unwrap();
        let utf16_col = line[..char_idx].encode_utf16().count() as u32;
        let byte_col = line[..char_idx].len() as u32;
        assert_eq!(
            byte_col,
            utf16_col + 3,
            "fixture no longer exercises the gap"
        );

        let members = extract_document_symbols(&source)[0]
            .children
            .clone()
            .unwrap();
        let var = members.iter().find(|c| c.name == "priceTotal").unwrap();

        assert_eq!(var.selection_range.start.line, 1);
        assert_eq!(var.selection_range.start.character, utf16_col);
    }

    /// Rows stay in tree-sitter's line model. `TextIndex` treats U+2028 as a
    /// line terminator and tree-sitter does not, so deriving the row from an
    /// absolute byte offset would report the symbol on the following line.
    #[test]
    fn test_symbol_line_uses_tree_sitter_line_model() {
        let line = "    string a = unicode\"x\u{2028}y\"; uint256 counter;";
        let source = format!("contract C {{\n{line}\n}}");
        let char_idx = line.find("counter").unwrap();
        let utf16_col = line[..char_idx].encode_utf16().count() as u32;

        let members = extract_document_symbols(&source)[0]
            .children
            .clone()
            .unwrap();
        let var = members.iter().find(|c| c.name == "counter").unwrap();

        assert_eq!(var.selection_range.start.line, 1, "U+2028 split the line");
        assert_eq!(var.selection_range.start.character, utf16_col);
    }

    /// A pure-ASCII line must be unaffected by the conversion.
    #[test]
    fn test_symbol_columns_unchanged_for_ascii() {
        let line = "    string plain = \"ascii\"; uint256 counter;";
        let source = format!("contract C {{\n{line}\n}}");
        let expected = line.find("counter").unwrap() as u32;

        let members = extract_document_symbols(&source)[0]
            .children
            .clone()
            .unwrap();
        let var = members.iter().find(|c| c.name == "counter").unwrap();

        assert_eq!(var.selection_range.start.character, expected);
    }

    /// Workspace symbols share the same conversion path. Their location spans
    /// the whole declaration, so anchor on where that declaration starts.
    #[test]
    fn test_workspace_symbol_columns_are_utf16() {
        let line = r#"    string label = unicode"状态"; uint256 counterA;"#;
        let source = format!("contract C {{\n{line}\n}}");

        // `状态` is 6 bytes / 2 UTF-16 units, so the declaration's byte column
        // runs 4 ahead of its UTF-16 column.
        let decl_idx = line.find("uint256 counterA").unwrap();
        let utf16_col = line[..decl_idx].encode_utf16().count() as u32;
        assert_eq!(
            line[..decl_idx].len() as u32,
            utf16_col + 4,
            "fixture no longer exercises the gap"
        );

        let uri = Url::parse("file:///tmp/C.sol").unwrap();
        let symbols = extract_workspace_symbols(&[(uri, source)]);
        let var = symbols.iter().find(|s| s.name == "counterA").unwrap();

        assert_eq!(var.location.range.start.line, 1);
        assert_eq!(var.location.range.start.character, utf16_col);
    }
}
