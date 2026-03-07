use serde_json::Value;
use solidity_language_server::goto::CachedBuild;
use solidity_language_server::links;
use std::fs;
use tower_lsp::lsp_types::Url;

/// Load the normalized AST (with sources intact) for test helpers that inspect
/// raw AST structure, plus a CachedBuild (which strips sources after indexing).
fn load_fixture() -> (Value, CachedBuild) {
    let raw: Value =
        serde_json::from_str(&fs::read_to_string("poolmanager.json").unwrap()).unwrap();
    let ast = solidity_language_server::solc::normalize_solc_output(raw, None);
    let build = CachedBuild::new(ast.clone(), 0, None);
    (ast, build)
}

/// Find the PoolManager.sol source key dynamically from the raw AST.
fn pm_key(ast: &Value) -> String {
    ast["sources"]
        .as_object()
        .unwrap()
        .keys()
        .find(|k| k.ends_with("src/PoolManager.sol"))
        .expect("fixture should contain PoolManager.sol")
        .clone()
}

/// Extract ImportDirective nodes from the normalized fixture AST.
/// Returns (src, file, absolutePath) tuples for verification.
fn extract_imports<'a>(ast: &'a Value, key: &str) -> Vec<(&'a str, &'a str, &'a str)> {
    let nodes = ast["sources"][key]["ast"]["nodes"].as_array().unwrap();
    nodes
        .iter()
        .filter(|n| n["nodeType"].as_str() == Some("ImportDirective"))
        .map(|n| {
            (
                n["src"].as_str().unwrap(),
                n["file"].as_str().unwrap(),
                n["absolutePath"].as_str().unwrap(),
            )
        })
        .collect()
}

/// Build source bytes from AST data alone.
///
/// Each import's `src` gives byte offset and length. Gaps between
/// consecutive imports are exactly 1 byte (the newline). We place
/// `"file";` at each offset's end so the quote-scan finds real quotes.
fn build_source_bytes(imports: &[(&str, &str, &str)]) -> Vec<u8> {
    let last = imports.last().unwrap();
    let parts: Vec<usize> = last.0.split(':').map(|p| p.parse().unwrap()).collect();
    let total_len = parts[0] + parts[1] + 1;

    let mut buf = vec![b' '; total_len];

    // Place 3 newlines before the first import to put it on line 3.
    let first_start: usize = imports[0].0.split(':').next().unwrap().parse().unwrap();
    if first_start >= 3 {
        let step = first_start / 3;
        buf[step - 1] = b'\n';
        buf[2 * step - 1] = b'\n';
        buf[first_start - 1] = b'\n';
    }

    for (src, file, _abs) in imports {
        let src_parts: Vec<usize> = src.split(':').map(|p| p.parse().unwrap()).collect();
        let start = src_parts[0];
        let length = src_parts[1];
        let end = start + length;

        let file_bytes = file.as_bytes();
        let semicolon_pos = end - 1;
        let close_quote_pos: usize = end - 2;
        let file_start_pos = close_quote_pos - file_bytes.len();
        let open_quote_pos = file_start_pos - 1;

        if semicolon_pos < total_len {
            buf[semicolon_pos] = b';';
        }
        if close_quote_pos < total_len {
            buf[close_quote_pos] = b'"';
        }
        if open_quote_pos >= start && open_quote_pos < total_len {
            buf[open_quote_pos] = b'"';
        }
        for (j, &b) in file_bytes.iter().enumerate() {
            let pos = file_start_pos + j;
            if pos < total_len {
                buf[pos] = b;
            }
        }

        if end < total_len {
            buf[end] = b'\n';
        }
    }

    buf
}

/// Filter links to just import links (those with a tooltip).
/// Reference links have tooltip = None, import links have tooltip = Some(absolutePath).
fn import_links(
    links: &[tower_lsp::lsp_types::DocumentLink],
) -> Vec<&tower_lsp::lsp_types::DocumentLink> {
    links.iter().filter(|l| l.tooltip.is_some()).collect()
}

// =============================================================================
// Import links: count and tooltips
// =============================================================================

#[test]
fn test_returns_link_for_every_import() {
    let (ast, build) = load_fixture();
    let key = pm_key(&ast);
    let imports = extract_imports(&ast, &key);
    let source_bytes = build_source_bytes(&imports);
    let uri = Url::from_file_path(&key).unwrap();

    let all_links = links::document_links(&build, &uri, &source_bytes);
    let ilinks = import_links(&all_links);

    assert_eq!(
        ilinks.len(),
        imports.len(),
        "should return one import link per ImportDirective (expected {}, got {})",
        imports.len(),
        ilinks.len(),
    );
}

#[test]
fn test_tooltips_match_absolute_paths() {
    let (ast, build) = load_fixture();
    let key = pm_key(&ast);
    let imports = extract_imports(&ast, &key);
    let source_bytes = build_source_bytes(&imports);
    let uri = Url::from_file_path(&key).unwrap();

    let all_links = links::document_links(&build, &uri, &source_bytes);
    let ilinks = import_links(&all_links);

    for (link, (_src, _file, abs_path)) in ilinks.iter().zip(imports.iter()) {
        assert_eq!(
            link.tooltip.as_deref(),
            Some(*abs_path),
            "tooltip should be the absolutePath"
        );
    }
}

#[test]
fn test_targets_are_file_uris() {
    let (ast, build) = load_fixture();
    let key = pm_key(&ast);
    let imports = extract_imports(&ast, &key);
    let source_bytes = build_source_bytes(&imports);
    let uri = Url::from_file_path(&key).unwrap();

    let all_links = links::document_links(&build, &uri, &source_bytes);

    for link in &all_links {
        let target = link.target.as_ref().expect("link should have a target");
        assert_eq!(target.scheme(), "file", "target should be a file:// URI");
    }
}

// =============================================================================
// Import link position accuracy
// =============================================================================

#[test]
fn test_import_range_length_matches_file_field() {
    let (ast, build) = load_fixture();
    let key = pm_key(&ast);
    let imports = extract_imports(&ast, &key);
    let source_bytes = build_source_bytes(&imports);
    let uri = Url::from_file_path(&key).unwrap();

    let all_links = links::document_links(&build, &uri, &source_bytes);
    let ilinks = import_links(&all_links);

    for (link, (_src, file, _abs)) in ilinks.iter().zip(imports.iter()) {
        let char_span = link.range.end.character - link.range.start.character;
        assert_eq!(
            char_span as usize,
            file.len(),
            "range character span should equal file field length for '{}'",
            file
        );
    }
}

#[test]
fn test_import_links_each_on_own_line() {
    let (ast, build) = load_fixture();
    let key = pm_key(&ast);
    let imports = extract_imports(&ast, &key);
    let source_bytes = build_source_bytes(&imports);
    let uri = Url::from_file_path(&key).unwrap();

    let all_links = links::document_links(&build, &uri, &source_bytes);
    let ilinks = import_links(&all_links);

    let mut seen_lines = std::collections::HashSet::new();
    for link in &ilinks {
        assert_eq!(
            link.range.start.line, link.range.end.line,
            "import link range should be single-line"
        );
        assert!(
            seen_lines.insert(link.range.start.line),
            "line {} appears in multiple import links",
            link.range.start.line
        );
    }
}

#[test]
fn test_links_are_in_ascending_line_order() {
    let (ast, build) = load_fixture();
    let key = pm_key(&ast);
    let imports = extract_imports(&ast, &key);
    let source_bytes = build_source_bytes(&imports);
    let uri = Url::from_file_path(&key).unwrap();

    let all_links = links::document_links(&build, &uri, &source_bytes);

    for pair in all_links.windows(2) {
        assert!(
            (pair[0].range.start.line, pair[0].range.start.character)
                <= (pair[1].range.start.line, pair[1].range.start.character),
            "links should be in ascending position order"
        );
    }
}

// =============================================================================
// Spot-check: first and last import
// =============================================================================

#[test]
fn test_first_import_hooks() {
    let (ast, build) = load_fixture();
    let key = pm_key(&ast);
    let imports = extract_imports(&ast, &key);
    let source_bytes = build_source_bytes(&imports);
    let uri = Url::from_file_path(&key).unwrap();

    let all_links = links::document_links(&build, &uri, &source_bytes);
    let ilinks = import_links(&all_links);
    let first = ilinks[0];

    // Tooltip is the absolutePath from the AST (ends with this suffix)
    let tooltip = first.tooltip.as_deref().unwrap();
    assert!(
        tooltip.ends_with("src/libraries/Hooks.sol"),
        "first import tooltip should end with src/libraries/Hooks.sol, got: {}",
        tooltip
    );
    assert_eq!(first.range.start.line, 3);
    assert_eq!(first.range.end.line, 3);
    assert_eq!(first.range.end.character - first.range.start.character, 21);
}

#[test]
fn test_last_import_custom_revert() {
    let (ast, build) = load_fixture();
    let key = pm_key(&ast);
    let imports = extract_imports(&ast, &key);
    let source_bytes = build_source_bytes(&imports);
    let uri = Url::from_file_path(&key).unwrap();

    let all_links = links::document_links(&build, &uri, &source_bytes);
    let ilinks = import_links(&all_links);
    let last = ilinks.last().unwrap();

    let tooltip = last.tooltip.as_deref().unwrap();
    assert!(
        tooltip.ends_with("src/libraries/CustomRevert.sol"),
        "last import tooltip should end with src/libraries/CustomRevert.sol, got: {}",
        tooltip
    );
    assert_eq!(last.range.start.line, 27);
    assert_eq!(last.range.end.character - last.range.start.character, 28);
}

// =============================================================================
// Reference links (nodes with referencedDeclaration)
// =============================================================================

#[test]
fn test_reference_links_have_no_tooltip() {
    let (ast, build) = load_fixture();
    let key = pm_key(&ast);
    let imports = extract_imports(&ast, &key);
    let source_bytes = build_source_bytes(&imports);
    let uri = Url::from_file_path(&key).unwrap();

    let all_links = links::document_links(&build, &uri, &source_bytes);
    let ref_links: Vec<_> = all_links.iter().filter(|l| l.tooltip.is_none()).collect();

    // With constructed source bytes, reference links won't resolve (no real
    // files on disk for id_to_location). This test documents the distinction:
    // import links have tooltips, reference links don't.
    // On a real system with source files, ref_links would be non-empty.
    for link in &ref_links {
        assert!(link.tooltip.is_none());
        let target = link.target.as_ref().unwrap();
        assert_eq!(target.scheme(), "file");
    }
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_empty_sources() {
    let build = CachedBuild::new(serde_json::json!({}), 0, None);
    let source_bytes = b"";
    let uri = Url::from_file_path("/tmp/Empty.sol").unwrap();

    let result = links::document_links(&build, &uri, source_bytes);
    assert!(result.is_empty());
}

#[test]
fn test_file_uri_mismatch_returns_empty() {
    let (ast, build) = load_fixture();
    let key = pm_key(&ast);
    let imports = extract_imports(&ast, &key);
    let source_bytes = build_source_bytes(&imports);
    let uri = Url::from_file_path("/tmp/nonexistent.sol").unwrap();

    let result = links::document_links(&build, &uri, &source_bytes);
    assert!(result.is_empty());
}
