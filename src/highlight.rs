use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightKind, Position, Range};
use tree_sitter::{Node, Parser};

/// Return all highlights for the identifier under the cursor.
///
/// Walks the tree-sitter parse tree to find every occurrence of the same
/// identifier text and classifies each as Read or Write based on its
/// syntactic context.
pub fn document_highlights(source: &str, position: Position) -> Vec<DocumentHighlight> {
    let tree = match parse(source) {
        Some(t) => t,
        None => return vec![],
    };

    let root = tree.root_node();

    // Find the identifier node at the cursor position.
    let target = match find_identifier_at(root, source, position) {
        Some(node) => node,
        None => return vec![],
    };

    let name = &source[target.byte_range()];

    // Collect every identifier in the file with the same text.
    let mut highlights = Vec::new();
    collect_matching_identifiers(root, source, name, &mut highlights);
    highlights
}

/// Find the identifier node at the given cursor position.
///
/// Descends to the deepest named node at the position, then walks up to
/// find the nearest `identifier` node.
fn find_identifier_at<'a>(root: Node<'a>, _source: &str, position: Position) -> Option<Node<'a>> {
    let point = tree_sitter::Point {
        row: position.line as usize,
        column: position.character as usize,
    };

    let node = root.descendant_for_point_range(point, point)?;

    // If we landed directly on an identifier, use it.
    if node.kind() == "identifier" {
        return Some(node);
    }

    // Check if the node text at this position is a keyword-like identifier
    // that tree-sitter doesn't classify as "identifier" (e.g., type names
    // in some contexts). Walk up a couple of levels.
    let mut current = node;
    for _ in 0..3 {
        if current.kind() == "identifier" {
            return Some(current);
        }
        current = current.parent()?;
    }

    // If the deepest node is a short anonymous node (like a keyword token),
    // check if it overlaps with an identifier sibling at the same position.
    // This handles cases where tree-sitter places the cursor on a non-named
    // token adjacent to an identifier.
    let parent = node.parent()?;
    let mut cursor = parent.walk();
    parent
        .children(&mut cursor)
        .find(|child| child.kind() == "identifier" && contains_point(*child, point))
}

/// Check if a node's range contains the given point.
fn contains_point(node: Node, point: tree_sitter::Point) -> bool {
    node.start_position() <= point && point <= node.end_position()
}

/// Recursively collect all identifier nodes matching `name`, classifying
/// each as Read or Write.
fn collect_matching_identifiers(
    node: Node,
    source: &str,
    name: &str,
    out: &mut Vec<DocumentHighlight>,
) {
    if node.kind() == "identifier" && &source[node.byte_range()] == name {
        let kind = classify_highlight(node, source);
        out.push(DocumentHighlight {
            range: range(node),
            kind: Some(kind),
        });
        return; // identifiers have no children
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_matching_identifiers(child, source, name, out);
    }
}

/// Determine whether an identifier occurrence is a Write or Read.
///
/// Write contexts:
/// - Declaration name (function, contract, struct, enum, event, error, modifier,
///   state variable, local variable, parameter, constructor)
/// - Left-hand side of an assignment expression (`=`, `+=`, `-=`, etc.)
/// - Increment/decrement expressions (`++`, `--`)
///
/// Everything else is Read.
///
/// Note: in tree-sitter-solidity, identifiers inside statements are wrapped
/// in `expression` nodes:
///   `count += 1` → augmented_assignment_expression > expression > identifier
///   `count++`    → update_expression > expression > identifier
///   `x = 5`      → assignment_expression > expression > identifier
/// So we check both the parent and grandparent to classify correctly.
fn classify_highlight(node: Node, _source: &str) -> DocumentHighlightKind {
    let parent = match node.parent() {
        Some(p) => p,
        None => return DocumentHighlightKind::READ,
    };

    // First, check if the immediate parent is a declaration context
    // (identifiers in declarations are direct children, not wrapped in expression).
    match parent.kind() {
        // ── Declaration sites (the name being declared) ────────────────
        "function_definition"
        | "constructor_definition"
        | "modifier_definition"
        | "contract_declaration"
        | "interface_declaration"
        | "library_declaration"
        | "struct_declaration"
        | "enum_declaration"
        | "event_definition"
        | "error_declaration"
        | "user_defined_type_definition"
        | "state_variable_declaration"
        | "struct_member" => {
            if is_first_identifier(parent, node) {
                return DocumentHighlightKind::WRITE;
            }
            return DocumentHighlightKind::READ;
        }

        // Local variable declaration: `uint256 x` inside variable_declaration
        "variable_declaration" => {
            if is_first_identifier(parent, node) {
                return DocumentHighlightKind::WRITE;
            }
            return DocumentHighlightKind::READ;
        }

        // Parameters: `function foo(uint256 x)` — the name is a Write
        "parameter" | "event_parameter" | "error_parameter" => {
            if is_first_identifier(parent, node) {
                return DocumentHighlightKind::WRITE;
            }
            return DocumentHighlightKind::READ;
        }

        _ => {}
    }

    // For expression-wrapped identifiers, check the grandparent.
    // Tree structure: grandparent > expression(parent) > identifier(node)
    if parent.kind() == "expression"
        && let Some(grandparent) = parent.parent()
    {
        return classify_expression_context(grandparent, parent);
    }

    DocumentHighlightKind::READ
}

/// Classify an identifier that is wrapped in an `expression` node.
/// `grandparent` is the node above `expression`, `expr_node` is the
/// `expression` wrapping the identifier.
fn classify_expression_context(grandparent: Node, expr_node: Node) -> DocumentHighlightKind {
    match grandparent.kind() {
        // `x = 5` → assignment_expression > expression(lhs) > identifier
        "assignment_expression" => {
            if is_lhs_of_assignment(grandparent, expr_node) {
                DocumentHighlightKind::WRITE
            } else {
                DocumentHighlightKind::READ
            }
        }

        // `count += 1` → augmented_assignment_expression > expression(lhs) > identifier
        "augmented_assignment_expression" => {
            if is_lhs_of_assignment(grandparent, expr_node) {
                DocumentHighlightKind::WRITE
            } else {
                DocumentHighlightKind::READ
            }
        }

        // `count++`, `++count` → update_expression > expression > identifier
        "update_expression" => DocumentHighlightKind::WRITE,

        // `delete x` → delete_expression > expression > identifier
        "delete_expression" | "delete_statement" => DocumentHighlightKind::WRITE,

        // Tuple destructuring: (a, b) = func()
        // The tuple_expression wraps expressions that are LHS of assignment
        "tuple_expression" => {
            if let Some(great_grandparent) = grandparent.parent()
                && let Some(ggp) = great_grandparent.parent()
                && (ggp.kind() == "assignment_expression"
                    || ggp.kind() == "augmented_assignment_expression")
                && is_lhs_of_assignment(ggp, great_grandparent)
            {
                return DocumentHighlightKind::WRITE;
            }
            DocumentHighlightKind::READ
        }

        _ => DocumentHighlightKind::READ,
    }
}

/// Check if `node` is the first `identifier` child of `parent`.
fn is_first_identifier(parent: Node, node: Node) -> bool {
    let mut cursor = parent.walk();
    for child in parent.children(&mut cursor) {
        if child.kind() == "identifier" {
            return child.id() == node.id();
        }
    }
    false
}

/// Check if `node` is on the left-hand side of an assignment expression.
///
/// In tree-sitter-solidity, assignment_expression has the structure:
///   assignment_expression -> lhs operator rhs
/// The LHS is the first named child.
fn is_lhs_of_assignment(assignment: Node, node: Node) -> bool {
    let mut cursor = assignment.walk();
    for child in assignment.children(&mut cursor) {
        if child.is_named() {
            // The first named child is the LHS.
            // Check if `node` is the LHS itself or is contained within it.
            return child.id() == node.id()
                || (child.start_byte() <= node.start_byte()
                    && node.end_byte() <= child.end_byte());
        }
    }
    false
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn parse(source: &str) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_solidity::LANGUAGE.into())
        .expect("failed to load Solidity grammar");
    parser.parse(source, None)
}

fn range(node: Node) -> Range {
    let s = node.start_position();
    let e = node.end_position();
    Range {
        start: Position::new(s.row as u32, s.column as u32),
        end: Position::new(e.row as u32, e.column as u32),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: return highlights as (line, col, kind) tuples for easy assertion.
    fn highlights_at(source: &str, line: u32, col: u32) -> Vec<(u32, u32, DocumentHighlightKind)> {
        let result = document_highlights(source, Position::new(line, col));
        result
            .into_iter()
            .map(|h| (h.range.start.line, h.range.start.character, h.kind.unwrap()))
            .collect()
    }

    #[test]
    fn test_empty_source() {
        assert!(document_highlights("", Position::new(0, 0)).is_empty());
    }

    #[test]
    fn test_no_identifier_at_position() {
        let source = "pragma solidity ^0.8.0;";
        let result = document_highlights(source, Position::new(0, 0));
        // "pragma" is a keyword, not an identifier — may or may not match
        // depending on tree-sitter grammar. Either empty or non-empty is fine.
        let _ = result;
    }

    #[test]
    fn test_state_variable_read_write() {
        let source = r#"contract Foo {
    uint256 public count;
    function inc() public {
        count += 1;
    }
    function get() public view returns (uint256) {
        return count;
    }
}"#;
        // Click on "count" at the declaration (line 1, col 23)
        let highlights = highlights_at(source, 1, 23);
        assert!(
            highlights.len() == 3,
            "expected 3 highlights for 'count', got {}: {:?}",
            highlights.len(),
            highlights
        );

        // Declaration should be Write
        let decl = highlights.iter().find(|h| h.0 == 1);
        assert_eq!(
            decl.map(|h| h.2),
            Some(DocumentHighlightKind::WRITE),
            "declaration should be Write"
        );

        // `count += 1` should be Write
        let assign = highlights.iter().find(|h| h.0 == 3);
        assert_eq!(
            assign.map(|h| h.2),
            Some(DocumentHighlightKind::WRITE),
            "`count += 1` should be Write"
        );

        // `return count` should be Read
        let read = highlights.iter().find(|h| h.0 == 6);
        assert_eq!(
            read.map(|h| h.2),
            Some(DocumentHighlightKind::READ),
            "`return count` should be Read"
        );
    }

    #[test]
    fn test_function_name_highlights() {
        let source = r#"contract Foo {
    function bar() public {}
    function baz() public {
        bar();
    }
}"#;
        // Click on "bar" at its definition (line 1)
        let highlights = highlights_at(source, 1, 13);
        assert_eq!(highlights.len(), 2, "expected 2 highlights for 'bar'");

        // Definition is Write
        assert_eq!(highlights[0].2, DocumentHighlightKind::WRITE);
        // Call is Read
        assert_eq!(highlights[1].2, DocumentHighlightKind::READ);
    }

    #[test]
    fn test_parameter_highlights() {
        let source = r#"contract Foo {
    function add(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }
}"#;
        // Click on "a" at parameter declaration (line 1, col 25)
        let highlights = highlights_at(source, 1, 25);
        assert_eq!(highlights.len(), 2, "expected 2 highlights for 'a'");
        // Parameter declaration is Write
        assert_eq!(highlights[0].2, DocumentHighlightKind::WRITE);
        // Usage in `return a + b` is Read
        assert_eq!(highlights[1].2, DocumentHighlightKind::READ);
    }

    #[test]
    fn test_local_variable_highlights() {
        let source = r#"contract Foo {
    function bar() public {
        uint256 x = 1;
        uint256 y = x + 1;
        x = y;
    }
}"#;
        // Click on "x" at declaration (line 2)
        let highlights = highlights_at(source, 2, 16);
        assert_eq!(
            highlights.len(),
            3,
            "expected 3 highlights for 'x': {:?}",
            highlights
        );
        // Declaration: Write
        assert_eq!(highlights[0].2, DocumentHighlightKind::WRITE);
        // `x + 1`: Read
        assert_eq!(highlights[1].2, DocumentHighlightKind::READ);
        // `x = y`: Write (LHS of assignment)
        assert_eq!(highlights[2].2, DocumentHighlightKind::WRITE);
    }

    #[test]
    fn test_contract_name_highlights() {
        let source = r#"contract Foo {
    Foo public self;
}"#;
        let highlights = highlights_at(source, 0, 9);
        assert!(
            !highlights.is_empty(),
            "expected at least 1 highlight for contract name 'Foo'"
        );
        // Contract declaration name is Write
        assert_eq!(highlights[0].2, DocumentHighlightKind::WRITE);
    }

    #[test]
    fn test_struct_name_and_members() {
        let source = r#"contract Foo {
    struct Info {
        string name;
        uint256 value;
    }
    Info public info;
}"#;
        // Click on "Info" at struct declaration (line 1)
        let highlights = highlights_at(source, 1, 11);
        assert!(
            highlights.len() >= 2,
            "expected at least 2 highlights for 'Info'"
        );
        // Struct declaration is Write
        assert_eq!(highlights[0].2, DocumentHighlightKind::WRITE);
    }

    #[test]
    fn test_event_name_highlights() {
        let source = r#"contract Foo {
    event Transfer(address from, address to, uint256 value);
    function send() public {
        emit Transfer(msg.sender, address(0), 100);
    }
}"#;
        // Click on "Transfer" at event definition (line 1)
        let highlights = highlights_at(source, 1, 10);
        assert_eq!(highlights.len(), 2, "expected 2 highlights for 'Transfer'");
        assert_eq!(highlights[0].2, DocumentHighlightKind::WRITE);
        assert_eq!(highlights[1].2, DocumentHighlightKind::READ);
    }

    #[test]
    fn test_no_cross_name_pollution() {
        let source = r#"contract Foo {
    uint256 public x;
    uint256 public y;
    function bar() public {
        x = y;
    }
}"#;
        // Click on "x" — should NOT highlight "y"
        let highlights = highlights_at(source, 1, 23);
        for h in &highlights {
            let text = source;
            let line: &str = text.lines().nth(h.0 as usize).unwrap();
            assert!(
                line.contains("x"),
                "highlight on line {} should contain 'x': '{}'",
                h.0,
                line
            );
        }
    }

    #[test]
    fn test_enum_name_highlights() {
        let source = r#"contract Foo {
    enum Status { Active, Paused }
    Status public status;
}"#;
        let highlights = highlights_at(source, 1, 9);
        assert!(
            highlights.len() >= 2,
            "expected at least 2 highlights for 'Status'"
        );
        assert_eq!(highlights[0].2, DocumentHighlightKind::WRITE);
    }

    #[test]
    fn test_modifier_name_highlights() {
        let source = r#"contract Foo {
    address public owner;
    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }
    function bar() public onlyOwner {}
}"#;
        let highlights = highlights_at(source, 2, 13);
        assert_eq!(highlights.len(), 2, "expected 2 highlights for 'onlyOwner'");
        assert_eq!(highlights[0].2, DocumentHighlightKind::WRITE);
        assert_eq!(highlights[1].2, DocumentHighlightKind::READ);
    }

    #[test]
    fn test_shop_sol() {
        let source = std::fs::read_to_string("example/Shop.sol").unwrap();
        // "PRICE" is declared at line 68 (0-indexed) and used in buy()
        let highlights = document_highlights(&source, Position::new(68, 22));
        assert!(
            highlights.len() >= 2,
            "Shop.sol 'PRICE' should have at least 2 highlights, got {}",
            highlights.len()
        );

        // The declaration (line 68) should be Write
        let decl = highlights.iter().find(|h| h.range.start.line == 68);
        assert_eq!(
            decl.map(|h| h.kind),
            Some(Some(DocumentHighlightKind::WRITE))
        );
    }

    #[test]
    fn test_increment_is_write() {
        let source = r#"contract Foo {
    uint256 public x;
    function inc() public {
        x++;
    }
}"#;
        // `x` is at column 19: "    uint256 public x;"
        let highlights = highlights_at(source, 1, 19);
        assert!(
            highlights.len() >= 2,
            "expected at least 2 highlights for 'x', got {}: {:?}",
            highlights.len(),
            highlights
        );
        let inc = highlights.iter().find(|h| h.0 == 3);
        assert_eq!(
            inc.map(|h| h.2),
            Some(DocumentHighlightKind::WRITE),
            "`x++` should be Write, all highlights: {:?}",
            highlights
        );
    }

    #[test]
    fn test_cursor_on_usage_finds_all() {
        let source = r#"contract Foo {
    uint256 public count;
    function inc() public {
        count += 1;
    }
}"#;
        // Click on "count" at the usage site (line 3), not the declaration
        let highlights_from_usage = highlights_at(source, 3, 8);
        let highlights_from_decl = highlights_at(source, 1, 23);
        assert_eq!(
            highlights_from_usage.len(),
            highlights_from_decl.len(),
            "clicking on usage vs declaration should find the same set"
        );
    }
}
