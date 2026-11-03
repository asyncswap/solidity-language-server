use crate::utils::{self, PositionEncoding};
use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensEdit,
    SemanticTokensLegend,
};
use tree_sitter::{Node, Parser};

/// Inclusive line range for filtering (0-based).
#[derive(Clone, Copy, Debug)]
struct LineRange {
    start: u32,
    end: u32,
}

// ── Token type indices ─────────────────────────────────────────────────────

#[allow(dead_code)]
mod ty {
    pub const NAMESPACE: u32 = 0;
    pub const TYPE: u32 = 1;
    pub const CLASS: u32 = 2;
    pub const ENUM: u32 = 3;
    pub const INTERFACE: u32 = 4;
    pub const STRUCT: u32 = 5;
    pub const PARAMETER: u32 = 6;
    pub const VARIABLE: u32 = 7;
    pub const PROPERTY: u32 = 8;
    pub const ENUM_MEMBER: u32 = 9;
    pub const EVENT: u32 = 10;
    pub const FUNCTION: u32 = 11;
    pub const METHOD: u32 = 12;
    pub const KEYWORD: u32 = 13;
    pub const COMMENT: u32 = 14;
    pub const STRING: u32 = 15;
    pub const NUMBER: u32 = 16;
    pub const OPERATOR: u32 = 17;
}

/// Build the legend announced in `ServerCapabilities`.
pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::NAMESPACE,   // 0
            SemanticTokenType::TYPE,        // 1
            SemanticTokenType::CLASS,       // 2
            SemanticTokenType::ENUM,        // 3
            SemanticTokenType::INTERFACE,   // 4
            SemanticTokenType::STRUCT,      // 5
            SemanticTokenType::PARAMETER,   // 6
            SemanticTokenType::VARIABLE,    // 7
            SemanticTokenType::PROPERTY,    // 8
            SemanticTokenType::ENUM_MEMBER, // 9
            SemanticTokenType::EVENT,       // 10
            SemanticTokenType::FUNCTION,    // 11
            SemanticTokenType::METHOD,      // 12
            SemanticTokenType::KEYWORD,     // 13
            SemanticTokenType::COMMENT,     // 14
            SemanticTokenType::STRING,      // 15
            SemanticTokenType::NUMBER,      // 16
            SemanticTokenType::OPERATOR,    // 17
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::DEFINITION,
            SemanticTokenModifier::READONLY,
            SemanticTokenModifier::STATIC,
            SemanticTokenModifier::DEPRECATED,
            SemanticTokenModifier::ABSTRACT,
            SemanticTokenModifier::DOCUMENTATION,
            SemanticTokenModifier::DEFAULT_LIBRARY,
        ],
    }
}

// ── Public entry points ────────────────────────────────────────────────────

/// Parse `source` with tree-sitter and return semantic tokens for the entire file.
pub fn semantic_tokens_full(source: &str) -> SemanticTokens {
    semantic_tokens_full_with(source, utils::encoding())
}

/// [`semantic_tokens_full`] with an explicit position encoding.
pub fn semantic_tokens_full_with(source: &str, encoding: PositionEncoding) -> SemanticTokens {
    semantic_tokens_impl(source, None, encoding)
}

/// Parse `source` with tree-sitter and return semantic tokens only for
/// lines `start_line..=end_line` (0-based). Nodes outside the range are
/// pruned during the tree walk so we avoid visiting most of the AST on
/// large files.
pub fn semantic_tokens_range(source: &str, start_line: u32, end_line: u32) -> SemanticTokens {
    semantic_tokens_impl(
        source,
        Some(LineRange {
            start: start_line,
            end: end_line,
        }),
        utils::encoding(),
    )
}

fn semantic_tokens_impl(
    source: &str,
    range: Option<LineRange>,
    encoding: PositionEncoding,
) -> SemanticTokens {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_solidity::LANGUAGE.into())
        .expect("failed to load Solidity grammar");

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => {
            return SemanticTokens {
                result_id: None,
                data: vec![],
            };
        }
    };

    let mut tokens: Vec<RawToken> = Vec::new();
    collect(tree.root_node(), source, range, &mut tokens);
    tokens.sort_by(|a, b| a.line.cmp(&b.line).then(a.byte_col.cmp(&b.byte_col)));

    // Delta-encode
    let mut prev_line = 0u32;
    let mut prev_col = 0u32;
    let data = tokens
        .iter()
        .map(|t| {
            // Tree-sitter measures in bytes; clients measure in their encoding.
            let line_start = t.start_byte - t.byte_col;
            let col = utils::encoded_len(&source[line_start..t.start_byte], encoding);
            let length =
                utils::encoded_len(&source[t.start_byte..t.start_byte + t.byte_len], encoding);
            let dl = t.line - prev_line;
            let dc = if dl == 0 { col - prev_col } else { col };
            prev_line = t.line;
            prev_col = col;
            SemanticToken {
                delta_line: dl,
                delta_start: dc,
                length,
                token_type: t.token_type,
                token_modifiers_bitset: 0,
            }
        })
        .collect();

    SemanticTokens {
        result_id: None,
        data,
    }
}

// ── Delta support ──────────────────────────────────────────────────────────

/// Compare two delta-encoded token arrays and return the minimal set of edits
/// to transform `old` into `new`. Each `SemanticToken` is 5 × u32, so one
/// token = 5 data elements in the flat array the LSP client sees.
///
/// The algorithm finds the longest common prefix and suffix, then emits a
/// single edit that replaces the changed middle section.
pub fn compute_delta(old: &[SemanticToken], new: &[SemanticToken]) -> Vec<SemanticTokensEdit> {
    let old_len = old.len();
    let new_len = new.len();

    // Find longest common prefix
    let prefix = old
        .iter()
        .zip(new.iter())
        .take_while(|(a, b)| tok_eq(a, b))
        .count();

    // If everything matches, no edits needed
    if prefix == old_len && prefix == new_len {
        return vec![];
    }

    // Find longest common suffix (not overlapping with prefix)
    let max_suffix = old_len.min(new_len) - prefix;
    let suffix = (0..max_suffix)
        .take_while(|&i| tok_eq(&old[old_len - 1 - i], &new[new_len - 1 - i]))
        .count();

    // The changed region in old: [prefix .. old_len - suffix]
    // The replacement from new:  [prefix .. new_len - suffix]
    let delete_count = old_len - prefix - suffix;
    let insert = &new[prefix..new_len - suffix];

    // Each token is 5 u32s in the flat data array
    let start = (prefix * 5) as u32;
    let delete_data = (delete_count * 5) as u32;

    let data: Vec<SemanticToken> = insert.to_vec();

    vec![SemanticTokensEdit {
        start,
        delete_count: delete_data,
        data: if data.is_empty() { None } else { Some(data) },
    }]
}

/// Compare two `SemanticToken`s field-by-field.
fn tok_eq(a: &SemanticToken, b: &SemanticToken) -> bool {
    a.delta_line == b.delta_line
        && a.delta_start == b.delta_start
        && a.length == b.length
        && a.token_type == b.token_type
        && a.token_modifiers_bitset == b.token_modifiers_bitset
}

#[derive(Debug)]
struct RawToken {
    line: u32,
    byte_col: usize,
    start_byte: usize,
    byte_len: usize,
    token_type: u32,
}

// ── Tree walker ────────────────────────────────────────────────────────────

/// Returns `true` if the node overlaps with the requested line range.
/// When no range is set (`None`), every node is in range.
fn in_range(node: Node, range: Option<LineRange>) -> bool {
    let Some(r) = range else { return true };
    let node_start = node.start_position().row as u32;
    let node_end = node.end_position().row as u32;
    // Overlap check: node ends at-or-after range start AND starts at-or-before range end
    node_end >= r.start && node_start <= r.end
}

fn collect(node: Node, source: &str, range: Option<LineRange>, tokens: &mut Vec<RawToken>) {
    // Prune subtrees entirely outside the requested range
    if !in_range(node, range) {
        return;
    }

    if !node.is_named() {
        if let Some(tt) = classify_anon(node, source) {
            push(node, source, tt, tokens);
        }
        return;
    }

    match node.kind() {
        // Literals
        "comment" => push_multiline(node, source, ty::COMMENT, tokens),
        "number_literal" => push(node, source, ty::NUMBER, tokens),
        "string" | "string_literal" | "hex_string_literal" | "unicode_string_literal" => {
            push_multiline(node, source, ty::STRING, tokens)
        }
        "boolean_literal" => push(node, source, ty::KEYWORD, tokens),

        // Declarations — tag the identifier, walk body
        "contract_declaration" => {
            push_child_id(node, source, ty::CLASS, tokens);
            walk(node, source, range, tokens);
        }
        "interface_declaration" => {
            push_child_id(node, source, ty::INTERFACE, tokens);
            walk(node, source, range, tokens);
        }
        "library_declaration" => {
            push_child_id(node, source, ty::NAMESPACE, tokens);
            walk(node, source, range, tokens);
        }
        "struct_definition" | "struct_declaration" => {
            push_child_id(node, source, ty::STRUCT, tokens);
            walk(node, source, range, tokens);
        }
        "enum_definition" | "enum_declaration" => {
            push_child_id(node, source, ty::ENUM, tokens);
            walk(node, source, range, tokens);
        }
        "enum_value" => push(node, source, ty::ENUM_MEMBER, tokens),
        "type_alias" | "user_defined_type_definition" => {
            push_child_id(node, source, ty::TYPE, tokens);
            walk(node, source, range, tokens);
        }
        "event_definition" => {
            push_child_id(node, source, ty::EVENT, tokens);
            walk(node, source, range, tokens);
        }
        "error_declaration" => {
            push_child_id(node, source, ty::TYPE, tokens);
            walk(node, source, range, tokens);
        }
        "function_definition" => {
            push_child_id(node, source, ty::FUNCTION, tokens);
            walk(node, source, range, tokens);
        }
        "state_variable_declaration" => {
            push_child_id(node, source, ty::PROPERTY, tokens);
            walk(node, source, range, tokens);
        }

        // Parameters
        "parameter" | "event_parameter" | "error_parameter" => {
            push_child_id(node, source, ty::PARAMETER, tokens);
            walk(node, source, range, tokens);
        }

        // Inheritance
        "inheritance_specifier" => {
            push_child_id(node, source, ty::INTERFACE, tokens);
            walk(node, source, range, tokens);
        }

        // Type references
        "user_defined_type" => push(node, source, ty::TYPE, tokens),

        // Identifiers — only emit where LSP adds value over tree-sitter
        "identifier" => {
            if let Some(tt) = classify_id(node) {
                push(node, source, tt, tokens);
            }
        }

        // Visibility / state mutability
        "visibility" | "state_mutability" => push(node, source, ty::KEYWORD, tokens),

        // Let tree-sitter handle: pragmas, builtins, modifiers, members, variables
        "pragma_directive" | "solidity_pragma_token" | "solidity_version" => {}
        "type_name" | "primitive_type" if node.child_count() == 0 => {}
        "type_name" | "primitive_type" => walk(node, source, range, tokens),

        // Everything else
        _ => walk(node, source, range, tokens),
    }
}

// ── Identifier classification ──────────────────────────────────────────────

/// Only returns a token type for identifiers where LSP improves on tree-sitter.
fn classify_id(node: Node) -> Option<u32> {
    let parent = node.parent()?;
    match parent.kind() {
        "call_expression" => Some(ty::FUNCTION),
        "inheritance_specifier" => Some(ty::INTERFACE),
        "using_directive" => Some(ty::NAMESPACE),
        "user_defined_type" => Some(ty::TYPE),
        "import_directive" | "import_clause" | "source_import" => Some(ty::NAMESPACE),
        "enum_value" => Some(ty::ENUM_MEMBER),
        "expression" => match parent.parent().map(|gp| gp.kind()) {
            Some("revert_statement" | "emit_statement") => Some(ty::TYPE),
            _ => None,
        },
        _ => None,
    }
}

/// Classify anonymous nodes (keywords, operators, punctuation).
fn classify_anon(node: Node, source: &str) -> Option<u32> {
    let text = &source[node.byte_range()];
    match text {
        // Keywords
        "import" | "from" | "as" | "contract" | "interface" | "library" | "is" | "using"
        | "for" | "struct" | "enum" | "event" | "error" | "function" | "modifier"
        | "constructor" | "fallback" | "receive" | "mapping" | "if" | "else" | "while" | "do"
        | "break" | "continue" | "return" | "returns" | "emit" | "revert" | "try" | "catch"
        | "throw" | "new" | "delete" | "assembly" | "type" | "let" | "switch" | "case"
        | "default" | "leave" | "public" | "private" | "internal" | "external" | "pure"
        | "view" | "payable" | "constant" | "immutable" | "override" | "virtual" | "abstract"
        | "memory" | "storage" | "calldata" | "indexed" | "anonymous" | "unchecked" => {
            Some(ty::KEYWORD)
        }

        // Builtin types — let tree-sitter handle (@type.builtin)
        "address" | "bool" | "string" | "bytes" | "int" | "uint" | "fixed" | "ufixed" => None,
        _ if is_sized_type(text) => None,

        // Operators
        "+" | "-" | "*" | "/" | "%" | "**" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&"
        | "||" | "!" | "&" | "|" | "^" | "~" | "<<" | ">>" | "=" | "+=" | "-=" | "*=" | "/="
        | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | "=>" | "?" | ":" => Some(ty::OPERATOR),

        _ => None,
    }
}

/// Check for sized types like uint256, int128, bytes32.
fn is_sized_type(text: &str) -> bool {
    if let Some(rest) = text
        .strip_prefix("uint")
        .or_else(|| text.strip_prefix("int"))
    {
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
    } else if let Some(rest) = text.strip_prefix("bytes") {
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

// ── Token emitters ─────────────────────────────────────────────────────────

fn walk(node: Node, source: &str, range: Option<LineRange>, tokens: &mut Vec<RawToken>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect(child, source, range, tokens);
    }
}

fn push(node: Node, source: &str, token_type: u32, tokens: &mut Vec<RawToken>) {
    // A token may not span lines; error recovery can produce nodes that do.
    if node.start_position().row != node.end_position().row {
        push_multiline(node, source, token_type, tokens);
        return;
    }
    let len = node.end_byte() - node.start_byte();
    if len > 0 {
        tokens.push(RawToken {
            line: node.start_position().row as u32,
            byte_col: node.start_position().column,
            start_byte: node.start_byte(),
            byte_len: len,
            token_type,
        });
    }
}

fn push_multiline(node: Node, source: &str, token_type: u32, tokens: &mut Vec<RawToken>) {
    let text = &source[node.byte_range()];
    let start_line = node.start_position().row as u32;
    let start_col = node.start_position().column;
    let mut line_byte = node.start_byte();

    for (i, line) in text.split('\n').enumerate() {
        let col = if i == 0 { start_col } else { 0 };
        let ws = if i > 0 {
            line.len() - line.trim_start().len()
        } else {
            0
        };
        let len = line.trim_end().len().saturating_sub(ws);
        if len > 0 {
            tokens.push(RawToken {
                line: start_line + i as u32,
                byte_col: col + ws,
                start_byte: line_byte + ws,
                byte_len: len,
                token_type,
            });
        }
        line_byte += line.len() + 1;
    }
}

fn push_child_id(node: Node, source: &str, token_type: u32, tokens: &mut Vec<RawToken>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" && child.is_named() {
            push(child, source, token_type, tokens);
            return;
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Decode to `(line, column, text, type)` for readable assertions.
    fn decoded(source: &str) -> Vec<(u32, u32, String, u32)> {
        let lines: Vec<&str> = source.split('\n').collect();
        let (mut line, mut col) = (0, 0);
        semantic_tokens_full_with(source, PositionEncoding::Utf8)
            .data
            .iter()
            .map(|t| {
                if t.delta_line > 0 {
                    line += t.delta_line;
                    col = t.delta_start;
                } else {
                    col += t.delta_start;
                }
                let end = (col + t.length) as usize;
                let text = lines[line as usize][col as usize..end].to_string();
                (line, col, text, t.token_type)
            })
            .collect()
    }

    fn has_token(source: &str, text: &str, token_type: u32) -> bool {
        decoded(source)
            .iter()
            .any(|(_, _, t, k)| t == text && *k == token_type)
    }

    #[test]
    fn test_legend_matches_token_type_indices() {
        let legend = legend();
        let name = |index: u32| legend.token_types[index as usize].as_str();
        assert_eq!(name(ty::NAMESPACE), "namespace");
        assert_eq!(name(ty::CLASS), "class");
        assert_eq!(name(ty::INTERFACE), "interface");
        assert_eq!(name(ty::STRUCT), "struct");
        assert_eq!(name(ty::ENUM), "enum");
        assert_eq!(name(ty::ENUM_MEMBER), "enumMember");
        assert_eq!(name(ty::TYPE), "type");
        assert_eq!(name(ty::EVENT), "event");
        assert_eq!(name(ty::FUNCTION), "function");
        assert_eq!(name(ty::PROPERTY), "property");
        assert_eq!(name(ty::PARAMETER), "parameter");
        assert_eq!(name(ty::KEYWORD), "keyword");
        assert_eq!(name(ty::COMMENT), "comment");
        assert_eq!(name(ty::STRING), "string");
        assert_eq!(name(ty::NUMBER), "number");
    }

    #[test]
    fn test_every_construct_gets_its_token_type() {
        let source = r#"// note
interface IThing { function act() external; }
library Lib { function help() internal {} }
struct Point { uint256 x; }
enum Color { Red }
type Price is uint256;
contract Base {}
contract Child is Base {
    event Moved(uint256 amount);
    uint256 public total;
    string label = "hi";
    Point spot;
    function run(uint256 amount) public view returns (bool) {
        return true && amount > 7;
    }
}
"#;
        let cases = [
            ("// note", ty::COMMENT),
            ("IThing", ty::INTERFACE),
            ("Lib", ty::NAMESPACE),
            ("Point", ty::STRUCT),
            ("Color", ty::ENUM),
            ("Red", ty::ENUM_MEMBER),
            ("Price", ty::TYPE),
            ("Child", ty::CLASS),
            ("Moved", ty::EVENT),
            ("total", ty::PROPERTY),
            ("run", ty::FUNCTION),
            ("amount", ty::PARAMETER),
            ("\"hi\"", ty::STRING),
            ("7", ty::NUMBER),
            ("true", ty::KEYWORD),
            ("public", ty::KEYWORD),
            ("view", ty::KEYWORD),
            (">", ty::OPERATOR),
        ];
        for (text, token_type) in cases {
            assert!(
                has_token(source, text, token_type),
                "expected {text:?} as type {token_type}: {:?}",
                decoded(source)
            );
        }
    }

    #[test]
    fn test_multiline_comment_splits_per_line_without_indentation() {
        let tokens = decoded("/* one\n   two\n\n   */ contract C {}");
        let comments: Vec<_> = tokens.iter().filter(|t| t.3 == ty::COMMENT).collect();
        assert_eq!(comments[0].0, 0);
        assert_eq!(comments[0].2, "/* one");
        assert_eq!((comments[1].0, comments[1].1), (1, 3));
        assert_eq!(comments[1].2, "two");
        assert_eq!((comments[2].0, comments[2].1), (3, 3));
        assert_eq!(comments[2].2, "*/");
        assert_eq!(comments.len(), 3);
    }

    #[test]
    fn test_lengths_and_columns_use_the_encoding() {
        let source = "/* é😀 */ contract C {}";
        let tokens = |encoding| semantic_tokens_full_with(source, encoding).data;
        let (utf16, utf8) = (
            tokens(PositionEncoding::Utf16),
            tokens(PositionEncoding::Utf8),
        );
        // `/* é😀 */` is 9 UTF-16 units and 12 bytes.
        assert_eq!(utf16[0].length, 9);
        assert_eq!(utf8[0].length, 12);
        // The `contract` keyword starts one past the comment in either unit.
        assert_eq!(utf16[1].delta_start, 10);
        assert_eq!(utf8[1].delta_start, 13);
    }

    #[test]
    fn test_range_keeps_only_overlapping_lines() {
        let source = "contract A {}\ncontract B {}\ncontract C {}\n";
        let tokens = semantic_tokens_range(source, 1, 1).data;
        assert_eq!(tokens[0].delta_line, 1);
        assert!(tokens.iter().skip(1).all(|t| t.delta_line == 0));
    }

    #[test]
    fn test_delta_of_change_at_both_ends_keeps_nothing_shared() {
        let token = |n| SemanticToken {
            delta_line: n,
            delta_start: 0,
            length: 1,
            token_type: 0,
            token_modifiers_bitset: 0,
        };
        let edits = compute_delta(&[token(1), token(2)], &[token(3), token(4), token(5)]);
        assert_eq!((edits[0].start, edits[0].delete_count), (0, 10));
        assert_eq!(edits[0].data.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_empty_source() {
        assert!(semantic_tokens_full("").data.is_empty());
    }

    #[test]
    fn test_simple_contract() {
        let source = "contract Foo {}";
        let types: Vec<u32> = semantic_tokens_full(source)
            .data
            .iter()
            .map(|t| t.token_type)
            .collect();
        assert!(types.contains(&ty::KEYWORD));
        assert!(types.contains(&ty::CLASS));
    }

    #[test]
    fn test_function_definition() {
        let source = "contract Foo { function bar(uint256 x) public pure returns (uint256) { return x + 1; } }";
        let types: Vec<u32> = semantic_tokens_full(source)
            .data
            .iter()
            .map(|t| t.token_type)
            .collect();
        assert!(types.contains(&ty::FUNCTION));
        assert!(types.contains(&ty::PARAMETER));
    }

    #[test]
    fn test_delta_encoding() {
        let result = semantic_tokens_full("contract A {}\ncontract B {}");
        assert!(result.data.iter().any(|t| t.delta_line > 0));
    }

    #[test]
    fn test_revert_emits_type() {
        let source = "contract Foo { error Bad(); function f() internal { revert Bad(); } }";
        let result = semantic_tokens_full(source);
        // "Bad" in error declaration and "Bad" in revert should both be TYPE
        let type_tokens: Vec<_> = result
            .data
            .iter()
            .filter(|t| t.token_type == ty::TYPE)
            .collect();
        assert!(
            type_tokens.len() >= 2,
            "expected error name as TYPE in both declaration and revert"
        );
    }

    #[test]
    fn test_emit_emits_type() {
        let source = "contract Foo { event Ping(); function f() internal { emit Ping(); } }";
        let result = semantic_tokens_full(source);
        let type_count = result
            .data
            .iter()
            .filter(|t| t.token_type == ty::TYPE)
            .count();
        assert!(type_count >= 1, "expected event name as TYPE in emit");
    }

    #[test]
    fn test_shop_sol() {
        let source = std::fs::read_to_string("example/Shop.sol").unwrap();
        let result = semantic_tokens_full(&source);
        assert!(
            result.data.len() > 50,
            "Shop.sol should produce many tokens"
        );
    }

    // ── Range tests ────────────────────────────────────────────────────

    #[test]
    fn test_range_returns_subset() {
        // Three contracts on separate lines — requesting only line 1
        // should return fewer tokens than the full file.
        let source = "contract A {}\ncontract B {}\ncontract C {}";
        let full = semantic_tokens_full(source);
        let range = semantic_tokens_range(source, 1, 1);
        assert!(
            range.data.len() < full.data.len(),
            "range (line 1 only) should return fewer tokens than full: range={} full={}",
            range.data.len(),
            full.data.len(),
        );
    }

    #[test]
    fn test_range_delta_encoding_starts_fresh() {
        // When range starts at line 2, the first delta_line should still
        // be relative to position (0,0) per the LSP spec — the editor
        // adjusts based on the range start.
        let source = "contract A {}\ncontract B {}\ncontract C {}";
        let range = semantic_tokens_range(source, 2, 2);
        assert!(!range.data.is_empty(), "should have tokens on line 2");
        // First token on line 2 has delta_line == 2 (relative to 0,0)
        assert_eq!(range.data[0].delta_line, 2);
    }

    #[test]
    fn test_range_covers_multiline_node() {
        // A function spanning lines 1-3; requesting lines 1-3 should
        // include all tokens from the function body.
        let source = "\
contract Foo {
    function bar(uint256 x) public pure returns (uint256) {
        return x + 1;
    }
}";
        let full = semantic_tokens_full(source);
        let range = semantic_tokens_range(source, 0, 4);
        assert_eq!(
            range.data.len(),
            full.data.len(),
            "range covering entire file should match full"
        );
    }

    #[test]
    fn test_range_empty_when_outside() {
        let source = "contract Foo {}";
        // Line 0 has the contract; line 5 has nothing
        let range = semantic_tokens_range(source, 5, 10);
        assert!(
            range.data.is_empty(),
            "range beyond EOF should return no tokens"
        );
    }

    #[test]
    fn test_range_only_second_contract() {
        // Two contracts on separate lines; request only the second one
        let source = "contract A {}\ncontract B {}";
        let range = semantic_tokens_range(source, 1, 1);
        // Should get "contract" keyword + "B" class identifier = 2 tokens
        assert_eq!(
            range.data.len(),
            2,
            "expected keyword + class on line 1, got {}",
            range.data.len(),
        );
        // Both tokens should be on line 1
        assert_eq!(range.data[0].delta_line, 1);
    }

    // ── Delta tests ────────────────────────────────────────────────────

    fn mk_tok(dl: u32, ds: u32, len: u32, tt: u32) -> SemanticToken {
        SemanticToken {
            delta_line: dl,
            delta_start: ds,
            length: len,
            token_type: tt,
            token_modifiers_bitset: 0,
        }
    }

    #[test]
    fn test_delta_identical_returns_empty() {
        let tokens = vec![mk_tok(0, 0, 8, ty::KEYWORD), mk_tok(0, 9, 3, ty::CLASS)];
        let edits = compute_delta(&tokens, &tokens);
        assert!(edits.is_empty(), "identical tokens should produce no edits");
    }

    #[test]
    fn test_delta_append_token() {
        let old = vec![mk_tok(0, 0, 8, ty::KEYWORD)];
        let new = vec![mk_tok(0, 0, 8, ty::KEYWORD), mk_tok(0, 9, 3, ty::CLASS)];
        let edits = compute_delta(&old, &new);
        assert_eq!(edits.len(), 1);
        // start = 5 (after 1 token × 5 u32s), delete 0, insert the new token
        assert_eq!(edits[0].start, 5);
        assert_eq!(edits[0].delete_count, 0);
        assert_eq!(edits[0].data.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_delta_remove_token() {
        let old = vec![mk_tok(0, 0, 8, ty::KEYWORD), mk_tok(0, 9, 3, ty::CLASS)];
        let new = vec![mk_tok(0, 0, 8, ty::KEYWORD)];
        let edits = compute_delta(&old, &new);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].start, 5);
        assert_eq!(edits[0].delete_count, 5); // 1 token × 5 u32s
        assert!(edits[0].data.is_none());
    }

    #[test]
    fn test_delta_change_middle_token() {
        let old = vec![
            mk_tok(0, 0, 8, ty::KEYWORD),
            mk_tok(0, 9, 3, ty::CLASS),
            mk_tok(1, 0, 8, ty::KEYWORD),
        ];
        let new = vec![
            mk_tok(0, 0, 8, ty::KEYWORD),
            mk_tok(0, 9, 5, ty::FUNCTION), // changed
            mk_tok(1, 0, 8, ty::KEYWORD),
        ];
        let edits = compute_delta(&old, &new);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].start, 5); // skip prefix (1 token)
        assert_eq!(edits[0].delete_count, 5); // replace 1 token
        let inserted = edits[0].data.as_ref().unwrap();
        assert_eq!(inserted.len(), 1);
        assert_eq!(inserted[0].length, 5);
        assert_eq!(inserted[0].token_type, ty::FUNCTION);
    }

    #[test]
    fn test_delta_full_file_edit() {
        // Original and modified source
        let old = semantic_tokens_full("contract A {}");
        let new = semantic_tokens_full("contract B {}");
        let edits = compute_delta(&old.data, &new.data);
        // Only the class name token differs (A → B same length, but the delta
        // encoding is identical since both are 1-char identifiers at same position).
        // If token arrays are identical, edits should be empty.
        // If they differ, we get exactly one edit.
        assert!(edits.len() <= 1);
    }

    #[test]
    fn test_delta_completely_different() {
        let old = vec![mk_tok(0, 0, 8, ty::KEYWORD)];
        let new = vec![mk_tok(0, 0, 5, ty::FUNCTION), mk_tok(0, 6, 3, ty::TYPE)];
        let edits = compute_delta(&old, &new);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].start, 0); // no common prefix
        assert_eq!(edits[0].delete_count, 5); // delete all old (1 × 5)
        assert_eq!(edits[0].data.as_ref().unwrap().len(), 2); // insert all new
    }
}
