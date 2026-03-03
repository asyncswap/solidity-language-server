use crate::goto::{CachedBuild, bytes_to_pos};
use crate::types::SourceLoc;
use crate::utils;
use tower_lsp::lsp_types::{DocumentLink, Position, Range, Url};
use tree_sitter::Parser;

/// Extract document links for import directives in the current file.
///
/// Each `ImportDirective` node produces a clickable link over the import
/// path string that targets the imported file. Other identifier links
/// are handled by `textDocument/definition`.
pub fn document_links(
    build: &CachedBuild,
    file_uri: &Url,
    source_bytes: &[u8],
) -> Vec<DocumentLink> {
    let mut links = Vec::new();

    let file_path = match file_uri.to_file_path() {
        Ok(p) => p,
        Err(_) => return links,
    };
    let file_path_str = match file_path.to_str() {
        Some(s) => s,
        None => return links,
    };

    let abs_path = match build.path_to_abs.get(file_path_str) {
        Some(a) => a.as_str(),
        None => return links,
    };

    let file_nodes = match build.nodes.get(abs_path) {
        Some(n) => n,
        None => return links,
    };

    for (_id, node_info) in file_nodes.iter() {
        if node_info.node_type.as_deref() == Some("ImportDirective")
            && let Some(link) = import_link(node_info, source_bytes)
        {
            links.push(link);
        }
    }

    links.sort_by(|a, b| {
        a.range
            .start
            .line
            .cmp(&b.range.start.line)
            .then(a.range.start.character.cmp(&b.range.start.character))
    });

    links
}

/// Find the LSP Range of the import path string inside an ImportDirective.
///
/// Returns the range covering just the text between the quotes in the
/// import statement. Used by both `document_links` (for clickable links)
/// and `file_operations::rename_imports` (for path rewriting).
pub fn import_path_range(node_info: &crate::goto::NodeInfo, source_bytes: &[u8]) -> Option<Range> {
    let src_loc = SourceLoc::parse(&node_info.src)?;
    let (start_byte, length) = (src_loc.offset, src_loc.length);
    let end_byte = start_byte + length;

    if end_byte > source_bytes.len() || end_byte < 3 {
        return None;
    }

    // Walk backwards: `;` then closing quote then file string then opening quote
    let close_quote = end_byte - 2;
    let open_quote = (start_byte..close_quote)
        .rev()
        .find(|&i| source_bytes[i] == b'"' || source_bytes[i] == b'\'')?;

    let start_pos = bytes_to_pos(source_bytes, open_quote + 1)?;
    let end_pos = bytes_to_pos(source_bytes, close_quote)?;

    Some(Range {
        start: start_pos,
        end: end_pos,
    })
}

/// Build a document link for an ImportDirective node.
/// The link covers the quoted import path and targets the resolved file.
fn import_link(node_info: &crate::goto::NodeInfo, source_bytes: &[u8]) -> Option<DocumentLink> {
    let absolute_path = node_info.absolute_path.as_deref()?;
    let range = import_path_range(node_info, source_bytes)?;

    let target_path = std::path::Path::new(absolute_path);
    let full_path = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(target_path)
    };
    let target_uri = Url::from_file_path(&full_path).ok()?;

    Some(DocumentLink {
        range,
        target: Some(target_uri),
        tooltip: Some(absolute_path.to_string()),
        data: None,
    })
}

/// An import found by tree-sitter: the quoted path string and its LSP range
/// (covering only the text between the quotes).
pub struct TsImport {
    /// The import path string (without quotes), e.g. `./Extsload.sol`.
    pub path: String,
    /// LSP range covering the path text between quotes.
    pub inner_range: Range,
}

/// Parse `source_bytes` with tree-sitter and return all import paths with
/// their ranges.  This is independent of the solc AST and always reflects
/// the **current** source content, making it safe to use when the AST is
/// stale or unavailable (e.g. after a failed re-index).
pub fn ts_find_imports(source_bytes: &[u8]) -> Vec<TsImport> {
    let source = match std::str::from_utf8(source_bytes) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_solidity::LANGUAGE.into())
        .is_err()
    {
        return vec![];
    }
    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return vec![],
    };

    let mut imports = Vec::new();
    collect_imports(tree.root_node(), source_bytes, &mut imports);
    imports
}

/// Returns the inner LSP [`Range`] of the assembly-flags string the cursor is
/// inside (e.g. `assembly ("memory-safe")`), or `None` if the cursor is not
/// inside an assembly flags string.
///
/// Works for both complete syntax and unclosed strings that tree-sitter
/// cannot fully parse (produces an `ERROR` node).
pub fn ts_cursor_in_assembly_flags(source_bytes: &[u8], position: Position) -> Option<Range> {
    let source_str = std::str::from_utf8(source_bytes).unwrap_or("");
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_solidity::LANGUAGE.into())
        .is_err()
    {
        return None;
    }
    let tree = parser.parse(source_str, None)?;
    find_assembly_flags_range(tree.root_node(), source_bytes, source_str, position)
}

fn find_assembly_flags_range(
    node: tree_sitter::Node,
    source_bytes: &[u8],
    source_str: &str,
    position: Position,
) -> Option<Range> {
    // Fully parsed: assembly_statement > assembly_flags > string
    if node.kind() == "assembly_flags" {
        for i in 0..node.named_child_count() {
            if let Some(child) = node.named_child(i as u32) {
                if child.kind() == "string" {
                    let start = child.start_byte();
                    let end = child.end_byte().min(source_bytes.len());
                    if end >= start + 2 {
                        let inner_start = start + 1;
                        let inner_end = end - 1;
                        let s = utils::byte_offset_to_position(source_str, inner_start);
                        let e = utils::byte_offset_to_position(source_str, inner_end);
                        let r = Range { start: s, end: e };
                        if position >= r.start && position <= r.end {
                            return Some(r);
                        }
                    }
                }
            }
        }
    }

    // Incomplete/unclosed: ERROR node containing `assembly` `(` `"` siblings
    if node.kind() == "ERROR" {
        let mut saw_assembly = false;
        let mut saw_lparen = false;
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i as u32) {
                match child.kind() {
                    "assembly" => {
                        saw_assembly = true;
                    }
                    "(" if saw_assembly => {
                        saw_lparen = true;
                    }
                    "\"" | "'" if saw_assembly && saw_lparen => {
                        let q = source_bytes[child.start_byte()];
                        let inner_start = child.start_byte() + 1;
                        let inner_end = find_closing_quote(source_bytes, inner_start, q);
                        let s = utils::byte_offset_to_position(source_str, inner_start);
                        let e = utils::byte_offset_to_position(source_str, inner_end);
                        let r = Range { start: s, end: e };
                        if position >= r.start && position <= r.end {
                            return Some(r);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if let Some(r) = find_assembly_flags_range(child, source_bytes, source_str, position) {
                return Some(r);
            }
        }
    }
    None
}

/// Returns the inner LSP [`Range`] of the import string the cursor is inside,
/// or `None` if the cursor is not inside an import string.
///
/// "Inside" means `inner_range.start <= position <= inner_range.end` (the
/// range does **not** include the surrounding quotes).
pub fn ts_cursor_in_import_string(source_bytes: &[u8], position: Position) -> Option<Range> {
    ts_find_imports(source_bytes)
        .into_iter()
        .find(|imp| {
            let r = &imp.inner_range;
            position >= r.start && position <= r.end
        })
        .map(|imp| imp.inner_range)
}

/// Recursively walk the tree-sitter CST to find `import_directive` nodes, plus
/// `ERROR` nodes that begin with the `import` keyword (handles unclosed strings
/// such as `import {} from "` which tree-sitter cannot fully parse).
fn collect_imports(node: tree_sitter::Node, source_bytes: &[u8], out: &mut Vec<TsImport>) {
    let source_str = std::str::from_utf8(source_bytes).unwrap_or("");

    if node.kind() == "import_directive" {
        // Walk all children (not just named) to find either:
        //   a) a `string` node — the complete import path
        //   b) an `ERROR` child containing a bare `"` — an unclosed/malformed
        //      import string that tree-sitter couldn't fully parse (e.g.
        //      `import "forge-std/` followed by more lines greedy-consumed into
        //      the same import_directive)
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i as u32) {
                if child.kind() == "string" {
                    push_string_node(child, source_bytes, source_str, out);
                    return;
                }
                if child.kind() == "ERROR" {
                    // Look for a bare opening quote inside the ERROR node
                    for j in 0..child.child_count() {
                        if let Some(gc) = child.child(j as u32) {
                            if gc.kind() == "\"" || gc.kind() == "'" {
                                let q = source_bytes[gc.start_byte()];
                                let inner_start = gc.start_byte() + 1;
                                let inner_end = find_closing_quote(source_bytes, inner_start, q);
                                let path =
                                    String::from_utf8_lossy(&source_bytes[inner_start..inner_end])
                                        .to_string();
                                let start_pos =
                                    utils::byte_offset_to_position(source_str, inner_start);
                                let end_pos = utils::byte_offset_to_position(source_str, inner_end);
                                out.push(TsImport {
                                    path,
                                    inner_range: Range {
                                        start: start_pos,
                                        end: end_pos,
                                    },
                                });
                                return;
                            }
                        }
                    }
                }
            }
        }
        return;
    }

    // Handle ERROR nodes that start with `import` — this is what tree-sitter
    // produces for incomplete/unclosed import strings like:
    //   import {} from "
    //   import {} from "whi
    // In this case the source is unparseable so there is no `import_directive`;
    // instead we get an ERROR node whose first token is the `import` keyword.
    // We look for a bare `"` or `'` child token and treat everything from that
    // quote to end-of-source as the (open-ended) inner range.
    if node.kind() == "ERROR" {
        let mut has_import = false;
        let mut quote_byte: Option<usize> = None;
        let mut quote_ch: Option<u8> = None;
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i as u32) {
                let kind = child.kind();
                if kind == "import" {
                    has_import = true;
                }
                if has_import && (kind == "\"" || kind == "'") {
                    // bare opening quote with no matching close
                    let q = source_bytes[child.start_byte()];
                    quote_byte = Some(child.start_byte() + 1); // byte after the quote
                    quote_ch = Some(q);
                    break;
                }
                // Also handle a complete `string` node inside the ERROR
                if has_import && kind == "string" {
                    push_string_node(child, source_bytes, source_str, out);
                    quote_byte = None; // handled
                    break;
                }
            }
        }
        if let (Some(inner_start), Some(q)) = (quote_byte, quote_ch) {
            // Find the end: either the matching closing quote or end-of-line.
            let inner_end = find_closing_quote(source_bytes, inner_start, q);
            let path = String::from_utf8_lossy(&source_bytes[inner_start..inner_end]).to_string();
            let start_pos = utils::byte_offset_to_position(source_str, inner_start);
            let end_pos = utils::byte_offset_to_position(source_str, inner_end);
            out.push(TsImport {
                path,
                inner_range: Range {
                    start: start_pos,
                    end: end_pos,
                },
            });
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            collect_imports(child, source_bytes, out);
        }
    }
}

/// Push a `TsImport` for a tree-sitter `string` node (which includes its quotes).
/// Handles both complete (`"path"`) and malformed/unclosed strings where
/// tree-sitter may have consumed content past a newline.
///
/// The inner range is always clamped to the current line so that a greedy
/// parse of an unclosed string doesn't bleed into the next line.
fn push_string_node(
    node: tree_sitter::Node,
    source_bytes: &[u8],
    source_str: &str,
    out: &mut Vec<TsImport>,
) {
    let start = node.start_byte();
    let raw_end = node.end_byte().min(source_bytes.len());
    if raw_end < start + 1 {
        return;
    }
    let inner_start = start + 1;
    // Clamp end to end-of-line so a greedy unclosed string doesn't bleed
    // across newlines. Also strip the closing quote if present.
    let eol = find_closing_quote(source_bytes, inner_start, b'\n');
    let closing_quote = source_bytes
        .get(inner_start..eol)
        .and_then(|s| s.iter().position(|&b| b == source_bytes[start]))
        .map(|p| inner_start + p);
    let inner_end = closing_quote.unwrap_or(eol);

    let path = String::from_utf8_lossy(&source_bytes[inner_start..inner_end]).to_string();
    let start_pos = utils::byte_offset_to_position(source_str, inner_start);
    let end_pos = utils::byte_offset_to_position(source_str, inner_end);
    out.push(TsImport {
        path,
        inner_range: Range {
            start: start_pos,
            end: end_pos,
        },
    });
}

/// Find the byte offset of the closing `quote_char` starting at `from`, or
/// return the end of the current line (or end-of-source) if none is found.
fn find_closing_quote(source_bytes: &[u8], from: usize, quote_char: u8) -> usize {
    for i in from..source_bytes.len() {
        let b = source_bytes[i];
        if b == quote_char {
            return i;
        }
        // Stop at newline — an import string can't span lines.
        if b == b'\n' || b == b'\r' {
            return i;
        }
    }
    source_bytes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(line: u32, character: u32) -> Position {
        Position { line, character }
    }

    #[test]
    fn ts_cursor_in_import_string_inside() {
        // Cursor at col 12 is between the quotes of `"./Foo.sol"`
        let src = b"import \"./Foo.sol\";";
        let range = ts_cursor_in_import_string(src, pos(0, 12));
        assert!(range.is_some(), "expected Some inside import string");
        let r = range.unwrap();
        // inner_range should start right after the opening quote (col 8)
        assert_eq!(r.start.character, 8);
        // and end right before the closing quote (col 17)
        assert_eq!(r.end.character, 17);
    }

    #[test]
    fn ts_cursor_in_import_string_outside_semicolon() {
        // Cursor past the closing quote
        let src = b"import \"./Foo.sol\";";
        let range = ts_cursor_in_import_string(src, pos(0, 19));
        assert!(range.is_none(), "should be None past closing quote");
    }

    #[test]
    fn ts_cursor_in_import_string_non_import_literal() {
        // A regular string literal should NOT match
        let src = b"string memory s = \"hello\";";
        let range = ts_cursor_in_import_string(src, pos(0, 20));
        assert!(range.is_none(), "should be None for non-import string");
    }

    #[test]
    fn ts_cursor_in_import_string_from_style() {
        // Named import: `import {Foo} from "./Foo.sol";`
        let src = b"import {Foo} from \"./Foo.sol\";";
        // col 19 is just after the opening `"`
        let range = ts_cursor_in_import_string(src, pos(0, 19));
        assert!(range.is_some(), "expected Some for from-style import");
    }

    #[test]
    fn ts_cursor_in_import_string_empty_string() {
        // `import ""` — cursor right after the opening quote (col 8)
        let src = b"import \"\"";
        let range = ts_cursor_in_import_string(src, pos(0, 8));
        assert!(range.is_some(), "expected Some for empty import string");
        let r = range.unwrap();
        assert_eq!(r.start.character, 8);
        assert_eq!(r.end.character, 8); // inner range is zero-width
    }

    #[test]
    fn ts_cursor_in_import_string_unclosed() {
        // `import {} from "` — unclosed, cursor right after the quote (col 16)
        let src = b"import {} from \"";
        let range = ts_cursor_in_import_string(src, pos(0, 16));
        assert!(range.is_some(), "expected Some for unclosed import string");
        let r = range.unwrap();
        assert_eq!(r.start.character, 16); // byte after the `"`
    }

    #[test]
    fn ts_cursor_in_import_string_unclosed_mid() {
        // `import {} from "whi` — cursor at col 19
        let src = b"import {} from \"whi";
        let range = ts_cursor_in_import_string(src, pos(0, 19));
        assert!(range.is_some(), "expected Some mid unclosed import string");
    }

    #[test]
    fn ts_cursor_in_import_string_non_import_unclosed() {
        // `string memory s = "hello` — unclosed non-import string should NOT match
        let src = b"string memory s = \"hello";
        let range = ts_cursor_in_import_string(src, pos(0, 20));
        assert!(
            range.is_none(),
            "should be None for unclosed non-import string"
        );
    }

    #[test]
    fn ts_cursor_in_import_string_bare_unclosed() {
        // `import "` — bare import with no from, unclosed
        let src = b"import \"";
        let range = ts_cursor_in_import_string(src, pos(0, 8));
        assert!(range.is_some(), "expected Some for bare unclosed import");
        let r = range.unwrap();
        assert_eq!(r.start.character, 8);
    }

    #[test]
    fn ts_cursor_in_import_string_bare_unclosed_mid() {
        // `import "forge-std/` — mid-path unclosed
        let src = b"import \"forge-std/";
        // cursor at end (col 18)
        let range = ts_cursor_in_import_string(src, pos(0, 18));
        assert!(
            range.is_some(),
            "expected Some for `import \"forge-std/` (unclosed mid-path)"
        );
        let r = range.unwrap();
        assert_eq!(r.start.character, 8); // after opening quote
        assert_eq!(r.end.character, 18); // end of source
    }

    #[test]
    fn ts_cursor_in_import_string_assembly_flags() {
        // `assembly ("memory-safe") {}` must NOT trigger import completions.
        let src = b"contract A { function f() internal { assembly (\"memory-safe\") {} } }";
        let range = ts_cursor_in_import_string(src, pos(0, 50));
        assert!(
            range.is_none(),
            "assembly dialect string must not trigger import completions"
        );
    }

    #[test]
    fn ts_cursor_in_import_string_revert_string() {
        // `revert("some error")` — string_literal inside a call, not an import.
        let src = b"contract A { function f() public { revert(\"err\"); } }";
        let range = ts_cursor_in_import_string(src, pos(0, 43));
        assert!(
            range.is_none(),
            "revert string must not trigger import completions"
        );
    }

    // --- assembly_flags tests ---

    #[test]
    fn ts_cursor_in_assembly_flags_complete() {
        // `assembly ("memory-safe") {}` — cursor inside the string
        let src = b"contract A { function f() internal { assembly (\"memory-safe\") {} } }";
        let range = ts_cursor_in_assembly_flags(src, pos(0, 50));
        assert!(
            range.is_some(),
            "expected Some inside assembly flags string"
        );
    }

    #[test]
    fn ts_cursor_in_assembly_flags_unclosed() {
        // `assembly ("` — unclosed, cursor right after the quote
        let src = b"contract A { function f() internal { assembly (\"";
        let range = ts_cursor_in_assembly_flags(src, pos(0, 48));
        assert!(
            range.is_some(),
            "expected Some for unclosed assembly flags string"
        );
    }

    #[test]
    fn ts_cursor_in_assembly_flags_not_import() {
        // Must not match a plain import string
        let src = b"import \"./Foo.sol\";";
        let range = ts_cursor_in_assembly_flags(src, pos(0, 12));
        assert!(
            range.is_none(),
            "import string must not match assembly_flags"
        );
    }

    #[test]
    fn ts_cursor_in_assembly_flags_not_revert() {
        // Must not match a revert string
        let src = b"contract A { function f() public { revert(\"err\"); } }";
        let range = ts_cursor_in_assembly_flags(src, pos(0, 43));
        assert!(
            range.is_none(),
            "revert string must not match assembly_flags"
        );
    }
}
