use crate::goto;
use crate::goto::CachedBuild;
use crate::references;
use crate::types::SourceLoc;
use std::collections::HashMap;
use tower_lsp::lsp_types::{Location, Position, Range, TextEdit, Url, WorkspaceEdit};

/// Search a specific line for an identifier and return its exact range.
/// Used to correct stale AST ranges when the buffer has been edited but
/// not saved (e.g. after a previous rename).
fn find_identifier_on_line(source_bytes: &[u8], line: u32, identifier: &str) -> Option<Range> {
    let text = String::from_utf8_lossy(source_bytes);
    let target_line = text.lines().nth(line as usize)?;
    // Find all occurrences of the identifier on this line, bounded by
    // non-identifier characters so we don't match substrings.
    let ident_bytes = identifier.as_bytes();
    let mut search_start = 0;
    while let Some(offset) = target_line[search_start..].find(identifier) {
        let col = search_start + offset;
        let before_ok = col == 0 || {
            let b = target_line.as_bytes()[col - 1];
            !b.is_ascii_alphanumeric() && b != b'_'
        };
        let after_ok = col + ident_bytes.len() >= target_line.len() || {
            let b = target_line.as_bytes()[col + ident_bytes.len()];
            !b.is_ascii_alphanumeric() && b != b'_'
        };
        if before_ok && after_ok {
            // Compute encoding-aware column positions
            let line_start_byte: usize = text
                .lines()
                .take(line as usize)
                .map(|l| l.len() + 1) // +1 for newline
                .sum();
            let start = crate::utils::byte_offset_to_position(&text, line_start_byte + col);
            let end = crate::utils::byte_offset_to_position(
                &text,
                line_start_byte + col + ident_bytes.len(),
            );
            return Some(Range { start, end });
        }
        search_start = col + 1;
    }
    None
}

fn get_text_at_range(source_bytes: &[u8], range: &Range) -> Option<String> {
    let start_byte = goto::pos_to_bytes(source_bytes, range.start);
    let end_byte = goto::pos_to_bytes(source_bytes, range.end);
    if end_byte > source_bytes.len() {
        return None;
    }
    String::from_utf8(source_bytes[start_byte..end_byte].to_vec()).ok()
}

fn get_name_location_index(
    build: &CachedBuild,
    file_uri: &Url,
    position: Position,
    source_bytes: &[u8],
) -> Option<usize> {
    let path = file_uri.to_file_path().ok()?;
    let path_str = path.to_str()?;
    let abs_path = build.path_to_abs.get(path_str)?;
    let byte_position = goto::pos_to_bytes(source_bytes, position);
    let node_id = references::byte_to_id(&build.nodes, abs_path, byte_position)?;
    let file_nodes = build.nodes.get(abs_path)?;
    let node_info = file_nodes.get(&node_id)?;

    if !node_info.name_locations.is_empty() {
        for (i, name_loc) in node_info.name_locations.iter().enumerate() {
            if let Some(loc) = SourceLoc::parse(name_loc)
                && loc.offset <= byte_position
                && byte_position < loc.end()
            {
                return Some(i);
            }
        }
    }
    None
}

pub fn get_identifier_at_position(source_bytes: &[u8], position: Position) -> Option<String> {
    let text = String::from_utf8_lossy(source_bytes);
    let abs_offset = crate::utils::position_to_byte_offset(&text, position);
    let lines: Vec<&str> = text.lines().collect();
    let line = lines.get(position.line as usize)?;
    // Compute byte offset within this line
    let line_start = text
        .as_bytes()
        .iter()
        .take(abs_offset)
        .enumerate()
        .rev()
        .find(|&(_, &b)| b == b'\n')
        .map(|(i, _)| i + 1)
        .unwrap_or(0);
    let col_byte = abs_offset - line_start;
    if col_byte > line.len() {
        return None;
    }
    let mut start = col_byte;
    let mut end = col_byte;

    while start > 0
        && (line.as_bytes()[start - 1].is_ascii_alphanumeric()
            || line.as_bytes()[start - 1] == b'_')
    {
        start -= 1;
    }
    while end < line.len()
        && (line.as_bytes()[end].is_ascii_alphanumeric() || line.as_bytes()[end] == b'_')
    {
        end += 1;
    }

    if start == end {
        return None;
    }
    if line.as_bytes()[start].is_ascii_digit() {
        return None;
    }

    Some(line[start..end].to_string())
}

pub fn get_identifier_range(source_bytes: &[u8], position: Position) -> Option<Range> {
    let text = String::from_utf8_lossy(source_bytes);
    let abs_offset = crate::utils::position_to_byte_offset(&text, position);
    let lines: Vec<&str> = text.lines().collect();
    let line = lines.get(position.line as usize)?;
    // Compute byte offset of line start and cursor column within line
    let line_start = text
        .as_bytes()
        .iter()
        .take(abs_offset)
        .enumerate()
        .rev()
        .find(|&(_, &b)| b == b'\n')
        .map(|(i, _)| i + 1)
        .unwrap_or(0);
    let col_byte = abs_offset - line_start;
    if col_byte > line.len() {
        return None;
    }
    let mut start = col_byte;
    let mut end = col_byte;

    while start > 0
        && (line.as_bytes()[start - 1].is_ascii_alphanumeric()
            || line.as_bytes()[start - 1] == b'_')
    {
        start -= 1;
    }
    while end < line.len()
        && (line.as_bytes()[end].is_ascii_alphanumeric() || line.as_bytes()[end] == b'_')
    {
        end += 1;
    }

    if start == end {
        return None;
    }
    if line.as_bytes()[start].is_ascii_digit() {
        return None;
    }

    // Convert byte offsets back to encoding-aware positions
    let start = crate::utils::byte_offset_to_position(&text, line_start + start);
    let end = crate::utils::byte_offset_to_position(&text, line_start + end);

    Some(Range { start, end })
}

/// Check whether `cursor_byte` falls on an alias local name inside an
/// `import_directive` in the tree-sitter parse tree, e.g. `MyTest` in
/// `import {Test as MyTest} from "./A.sol"` or `AFile` in
/// `import "./A.sol" as AFile`.
///
/// These names have `nameLocation: "-1:-1:-1"` in the solc AST, so
/// `byte_to_id` lands on the enclosing `ImportDirective` node and
/// `goto_references_cached` cannot find them.  Tree-sitter gives us their
/// exact byte range.
///
/// Returns `Some(identifier_text)` if the cursor is on such a name.
fn ts_alias_local_name_at_cursor(source_bytes: &[u8], cursor_byte: usize) -> Option<String> {
    let source_str = std::str::from_utf8(source_bytes).ok()?;
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_solidity::LANGUAGE.into())
        .ok()?;
    let tree = parser.parse(source_str, None)?;

    fn find_alias(node: tree_sitter::Node, source: &str, cursor_byte: usize) -> Option<String> {
        if node.kind() == "import_directive" {
            // Walk children looking for:
            //   identifier AS identifier  (symbol alias: `{Test as MyTest}`)
            //   AS identifier             (unit alias:   `"./A.sol" as AFile`)
            let count = node.child_count();
            let mut i = 0;
            while i < count {
                let child = node.child(i as u32)?;
                if child.kind() == "as" {
                    // The identifier immediately after `as` is the local alias name
                    if let Some(next) = node.child((i + 1) as u32) {
                        if next.kind() == "identifier"
                            && next.start_byte() <= cursor_byte
                            && cursor_byte < next.end_byte()
                        {
                            return Some(source[next.start_byte()..next.end_byte()].to_string());
                        }
                    }
                }
                i += 1;
            }
            return None;
        }
        // Recurse
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i as u32) {
                if let Some(result) = find_alias(child, source, cursor_byte) {
                    return Some(result);
                }
            }
        }
        None
    }

    find_alias(tree.root_node(), source_str, cursor_byte)
}

/// Collect all tree-sitter `identifier` node ranges in `source_bytes` whose
/// text exactly matches `name` (whole-word), returning them as LSP `Location`s
/// for `file_uri`.
///
/// Used as a fallback for alias local names that have no solc AST node.
fn ts_collect_identifier_locations(
    source_bytes: &[u8],
    file_uri: &Url,
    name: &str,
) -> Vec<Location> {
    let Some(source_str) = std::str::from_utf8(source_bytes).ok() else {
        return vec![];
    };
    let mut parser = tree_sitter::Parser::new();
    if parser
        .set_language(&tree_sitter_solidity::LANGUAGE.into())
        .is_err()
    {
        return vec![];
    }
    let Some(tree) = parser.parse(source_str, None) else {
        return vec![];
    };

    let mut locations = Vec::new();

    fn collect(
        node: tree_sitter::Node,
        source: &str,
        source_bytes: &[u8],
        name: &str,
        file_uri: &Url,
        out: &mut Vec<Location>,
    ) {
        if node.kind() == "identifier" {
            let text = &source[node.start_byte()..node.end_byte()];
            if text == name {
                if let (Some(start), Some(end)) = (
                    goto::bytes_to_pos(source_bytes, node.start_byte()),
                    goto::bytes_to_pos(source_bytes, node.end_byte()),
                ) {
                    out.push(Location {
                        uri: file_uri.clone(),
                        range: Range { start, end },
                    });
                }
            }
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i as u32) {
                collect(child, source, source_bytes, name, file_uri, out);
            }
        }
    }

    collect(
        tree.root_node(),
        source_str,
        source_bytes,
        name,
        file_uri,
        &mut locations,
    );
    locations
}

/// Deduplication map: URI → (start_line, start_col, end_line, end_col) → TextEdit.
type RenameEdits = HashMap<Url, HashMap<(u32, u32, u32, u32), TextEdit>>;

pub fn rename_symbol(
    build: &CachedBuild,
    file_uri: &Url,
    position: Position,
    source_bytes: &[u8],
    new_name: String,
    other_builds: &[&CachedBuild],
    text_buffers: &HashMap<String, Vec<u8>>,
) -> Option<WorkspaceEdit> {
    let original_identifier = get_identifier_at_position(source_bytes, position)?;

    // Check early whether the cursor is on an import alias local name, e.g.
    //   `MyTest` in `import {Test as MyTest} from "./A.sol"`
    //   `AFile`  in `import "./A.sol" as AFile`
    //
    // These names have `nameLocation: "-1:-1:-1"` in the solc AST (symbol
    // aliases) or their usages carry no back-reference to the import
    // declaration node (unit aliases).  The AST-based reference engine cannot
    // collect their occurrences correctly, so we handle them here with a pure
    // tree-sitter text-match pass and skip the AST path entirely.
    let cursor_byte = goto::pos_to_bytes(source_bytes, position);
    if let Some(alias_name) = ts_alias_local_name_at_cursor(source_bytes, cursor_byte) {
        if alias_name == original_identifier {
            // Import alias local names (`MyTest` in `import {Test as MyTest}`,
            // `AFile` in `import "./A.sol" as AFile`) are scoped to the file
            // that declares them.  Scan only the current file — either from the
            // in-memory buffer (unsaved edits) or from disk — not every open
            // document, which would produce false positives for identifiers that
            // happen to share the same spelling in unrelated files.
            let file_bytes = text_buffers
                .get(file_uri.as_str())
                .cloned()
                .unwrap_or_else(|| source_bytes.to_vec());
            let locations = ts_collect_identifier_locations(&file_bytes, file_uri, &alias_name);
            return build_workspace_edit(
                locations,
                &alias_name,
                new_name,
                text_buffers,
                source_bytes,
                file_uri,
            );
        }
    }

    let name_location_index = get_name_location_index(build, file_uri, position, source_bytes);
    let mut locations = references::goto_references_cached(
        build,
        file_uri,
        position,
        source_bytes,
        name_location_index,
        true, // rename always includes the declaration
    );

    // Cross-file: scan other cached ASTs for the same target definition.
    // Exclude the current file — it was already covered by the file-level
    // build above (which has fresh byte offsets from the editor buffer).
    let current_abs = file_uri
        .to_file_path()
        .ok()
        .and_then(|p| p.to_str().map(String::from));
    if let Some((def_abs_path, def_byte_offset)) =
        references::resolve_target_location(build, file_uri, position, source_bytes)
    {
        for other_build in other_builds {
            let other_locations = references::goto_references_for_target(
                other_build,
                &def_abs_path,
                def_byte_offset,
                name_location_index,
                true, // rename always includes the declaration
                current_abs.as_deref(),
            );
            locations.extend(other_locations);
        }
    }

    build_workspace_edit(
        locations,
        &original_identifier,
        new_name,
        text_buffers,
        source_bytes,
        file_uri,
    )
}

/// Deduplicate `locations`, build `TextEdit`s replacing `original_identifier`
/// with `new_name`, and return a `WorkspaceEdit`.  Returns `None` if no edits
/// can be produced.
fn build_workspace_edit(
    mut locations: Vec<Location>,
    original_identifier: &str,
    new_name: String,
    text_buffers: &HashMap<String, Vec<u8>>,
    _source_bytes: &[u8],
    _file_uri: &Url,
) -> Option<WorkspaceEdit> {
    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    locations.retain(|loc| {
        seen.insert((
            loc.uri.clone(),
            loc.range.start.line,
            loc.range.start.character,
            loc.range.end.line,
            loc.range.end.character,
        ))
    });

    if locations.is_empty() {
        return None;
    }
    let mut changes: RenameEdits = HashMap::new();
    for location in locations {
        // Read the file content, preferring in-memory text buffers (which
        // reflect unsaved editor changes) over reading from disk.
        let file_source_bytes = if let Some(buf) = text_buffers.get(location.uri.as_str()) {
            buf.clone()
        } else {
            let absolute_path = match location.uri.to_file_path() {
                Ok(p) => p,
                Err(_) => continue,
            };
            match std::fs::read(&absolute_path) {
                Ok(b) => b,
                Err(_) => continue,
            }
        };
        let text_at_range = get_text_at_range(&file_source_bytes, &location.range);
        let actual_range = if text_at_range.as_deref() == Some(original_identifier) {
            // Range matches the buffer — use it directly.
            location.range
        } else if text_at_range.is_some_and(|t| !t.is_empty()) {
            // The range resolves to a non-empty, different identifier.  This
            // happens when the AST returned an alias's referent (e.g. `Test`
            // when we are renaming `MyTest`) — the range is not stale, it just
            // points to the wrong symbol.  Skip without attempting a line scan,
            // which would incorrectly find `original_identifier` elsewhere on
            // the same line.
            continue;
        } else {
            // Range is out-of-bounds or empty — the AST range is stale (buffer
            // was edited but not saved since the last build).  Search the same
            // line for the identifier and correct the range.
            match find_identifier_on_line(
                &file_source_bytes,
                location.range.start.line,
                original_identifier,
            ) {
                Some(corrected) => corrected,
                None => continue,
            }
        };
        let text_edit = TextEdit {
            range: actual_range,
            new_text: new_name.clone(),
        };
        let key = (
            actual_range.start.line,
            actual_range.start.character,
            actual_range.end.line,
            actual_range.end.character,
        );
        changes
            .entry(location.uri)
            .or_default()
            .insert(key, text_edit);
    }
    let changes_vec: HashMap<Url, Vec<TextEdit>> = changes
        .into_iter()
        .map(|(uri, edits_map)| (uri, edits_map.into_values().collect()))
        .collect();
    Some(WorkspaceEdit {
        changes: Some(changes_vec),
        document_changes: None,
        change_annotations: None,
    })
}
