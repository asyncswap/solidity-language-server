//! Randomized checks of rules that must hold for any input, not just examples.
//!
//! Failing inputs shrink to a minimal case and are saved next to this file in
//! `invariants.regressions`, so commit that file with the fix.

use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use solidity_language_server::file_operations::apply_text_edits;
use solidity_language_server::invariants::check_response;
use solidity_language_server::utils::{
    PositionEncoding, byte_offset_to_position, byte_offset_to_position_with,
    position_to_byte_offset_with,
};
use solidity_language_server::{folding, highlight, links, rename, selection};
use solidity_language_server::{goto, semantic_tokens, symbols};
use tower_lsp::lsp_types::{Position, Range, SemanticToken, TextEdit};

const ENCODINGS: [PositionEncoding; 2] = [PositionEncoding::Utf8, PositionEncoding::Utf16];

/// Fragments that exercise line endings, multibyte text and Solidity syntax.
const PIECES: &[&str] = &[
    "a",
    "x_1",
    " ",
    "\t",
    "\n",
    "\r\n",
    "\r",
    "é",
    "€",
    "😀",
    "\u{2028}",
    "{",
    "}",
    "(",
    ")",
    ";",
    "\"",
    "=",
    ".",
    "//",
    "/*",
    "*/",
    "import ",
    "contract ",
    "function ",
    "assembly ",
    "using ",
];

const SEEDS: &[&str] = &[
    include_str!("../example/A.sol"),
    include_str!("../example/Alias.sol"),
    include_str!("../example/B.sol"),
    include_str!("../example/ScopedLocals.sol"),
    include_str!("../example/Shop.sol"),
    include_str!("../example/StringCompletions.sol"),
];

fn text() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(PIECES), 0..200).prop_map(|pieces| pieces.concat())
}

fn floor(source: &str, index: usize) -> usize {
    let mut index = index % (source.len() + 1);
    while !source.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// A real file damaged the way typing damages it: fragments inserted, spans
/// deleted and duplicated, so most inputs do not parse cleanly.
fn solidity() -> impl Strategy<Value = String> {
    let edit = (0..3u8, any::<usize>(), 0..60usize, any::<usize>());
    (
        prop::sample::select(SEEDS),
        prop::collection::vec(edit, 1..6),
        prop::sample::select(PIECES),
    )
        .prop_map(|(seed, edits, piece)| {
            let mut source = seed.to_string();
            for (kind, at, len, to) in edits {
                let start = floor(&source, at);
                let end = floor(&source, (start + len).min(source.len()));
                let end = end.max(start);
                match kind {
                    0 => source.replace_range(start..end, ""),
                    1 => source.insert_str(start, piece),
                    _ => {
                        let chunk = source[start..end].to_string();
                        source.insert_str(floor(&source, to), &chunk);
                    }
                }
            }
            source
        })
}

/// Independent model of LSP line endings: `(line, start byte, end byte)`.
fn model_lines(source: &str) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = source.char_indices().peekable();
    while let Some((index, c)) = chars.next() {
        if c == '\r' && chars.peek().is_some_and(|(_, next)| *next == '\n') {
            let (newline, _) = chars.next().unwrap();
            lines.push((start, index));
            start = newline + 1;
        } else if c == '\n' || c == '\r' {
            lines.push((start, index));
            start = index + 1;
        }
    }
    lines.push((start, source.len()));
    lines
}

fn units(text: &str, encoding: PositionEncoding) -> u32 {
    match encoding {
        PositionEncoding::Utf8 => text.len() as u32,
        PositionEncoding::Utf16 => text.encode_utf16().count() as u32,
    }
}

fn is_inside_crlf(source: &str, offset: usize) -> bool {
    offset > 0
        && source.as_bytes()[offset - 1] == b'\r'
        && source.as_bytes().get(offset) == Some(&b'\n')
}

fn assert_clean(
    method: &str,
    params: Option<serde_json::Value>,
    result: impl serde::Serialize,
    source: &str,
) {
    let result = serde_json::to_value(result).unwrap();
    let violations = check_response(
        method,
        params.as_ref(),
        &result,
        Some(source),
        PositionEncoding::default(),
    );
    assert!(
        violations.is_empty(),
        "{method}: {violations:?}\nsource: {source:?}"
    );
}

fn has_bare_cr(source: &str) -> bool {
    let bytes = source.as_bytes();
    (0..bytes.len()).any(|i| bytes[i] == b'\r' && bytes.get(i + 1) != Some(&b'\n'))
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 512,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn positions_match_the_line_model(source in text()) {
        let lines = model_lines(&source);
        for encoding in ENCODINGS {
            for (line, &(start, end)) in lines.iter().enumerate() {
                let mut boundaries: Vec<usize> = source[start..end].char_indices().map(|(i, _)| start + i).collect();
                boundaries.push(end);
                for offset in boundaries {
                    let expected = Position::new(line as u32, units(&source[start..offset], encoding));
                    prop_assert_eq!(byte_offset_to_position_with(&source, offset, encoding), expected);
                }
            }
        }
    }

    #[test]
    fn offsets_round_trip_through_positions(source in text()) {
        for encoding in ENCODINGS {
            for offset in (0..=source.len()).filter(|&i| source.is_char_boundary(i)) {
                if is_inside_crlf(&source, offset) {
                    continue;
                }
                let position = byte_offset_to_position_with(&source, offset, encoding);
                prop_assert_eq!(position_to_byte_offset_with(&source, position, encoding), offset);
            }
        }
    }

    #[test]
    fn any_position_lands_on_a_boundary_within_its_line(source in text(), line in 0u32..80, character in 0u32..300) {
        let lines = model_lines(&source);
        for encoding in ENCODINGS {
            let offset = position_to_byte_offset_with(&source, Position::new(line, character), encoding);
            prop_assert!(source.is_char_boundary(offset));
            match lines.get(line as usize) {
                Some(&(start, end)) => prop_assert!((start..=end).contains(&offset)),
                None => prop_assert_eq!(offset, source.len()),
            }
        }
    }

    #[test]
    fn any_byte_offset_converts_without_panicking(source in text(), offset in 0usize..2000) {
        for encoding in ENCODINGS {
            let position = byte_offset_to_position_with(&source, offset, encoding);
            prop_assert!(position.line as usize <= model_lines(&source).len());
        }
    }

    #[test]
    fn text_edits_match_a_splice_model(
        source in text(),
        cuts in prop::collection::vec((any::<usize>(), any::<usize>(), "[a-z]{0,3}"), 0..6),
    ) {
        // Disjoint, in-order byte spans. Spans inside a CRLF have no position.
        let mut spans: Vec<(usize, usize, String)> = Vec::new();
        let mut floor_at = 0;
        let mut sorted = cuts;
        sorted.sort_by_key(|(at, _, _)| *at % (source.len() + 1));
        for (at, len, new_text) in sorted {
            let start = floor(&source, at).max(floor_at);
            let end = floor(&source, (start + len % 12).min(source.len())).max(start);
            if !source.is_char_boundary(start) || is_inside_crlf(&source, start) || is_inside_crlf(&source, end) {
                continue;
            }
            floor_at = end;
            spans.push((start, end, new_text));
        }

        let mut expected = String::new();
        let mut cursor = 0;
        for (start, end, new_text) in &spans {
            expected.push_str(&source[cursor..*start]);
            expected.push_str(new_text);
            cursor = *end;
        }
        expected.push_str(&source[cursor..]);

        let edits: Vec<TextEdit> = spans
            .iter()
            .map(|(start, end, new_text)| TextEdit {
                range: Range::new(byte_offset_to_position(&source, *start), byte_offset_to_position(&source, *end)),
                new_text: new_text.clone(),
            })
            .collect();
        prop_assert_eq!(apply_text_edits(&source, &edits), expected);
    }

    #[test]
    fn semantic_tokens_are_valid_in_both_encodings(source in solidity()) {
        for encoding in ENCODINGS {
            let tokens = semantic_tokens::semantic_tokens_full_with(&source, encoding);
            let result = serde_json::to_value(&tokens).unwrap();
            let violations = check_response("textDocument/semanticTokens/full", None, &result, Some(&source), encoding);
            prop_assert!(violations.is_empty(), "{:?}\nsource: {:?}", violations, source);
        }
    }

    #[test]
    fn applying_a_token_delta_reproduces_the_new_tokens(old in solidity(), new in solidity()) {
        let old = semantic_tokens::semantic_tokens_full(&old).data;
        let new = semantic_tokens::semantic_tokens_full(&new).data;
        let mut patched: Vec<SemanticToken> = old.clone();
        for edit in semantic_tokens::compute_delta(&old, &new) {
            let start = (edit.start / 5) as usize;
            let delete = (edit.delete_count / 5) as usize;
            patched.splice(start..start + delete, edit.data.unwrap_or_default());
        }
        prop_assert_eq!(patched, new);
    }

    #[test]
    fn range_tokens_cover_the_full_tokens_on_those_lines(source in solidity(), first in 0u32..60, span in 0u32..30) {
        let decode = |data: &[SemanticToken]| {
            let (mut line, mut col) = (0, 0);
            data.iter().map(|t| {
                if t.delta_line > 0 { line += t.delta_line; col = t.delta_start; } else { col += t.delta_start; }
                (line, col, t.length, t.token_type)
            }).collect::<Vec<_>>()
        };
        let last = first + span;
        let full = decode(&semantic_tokens::semantic_tokens_full(&source).data);
        let range = decode(&semantic_tokens::semantic_tokens_range(&source, first, last).data);
        for token in full.iter().filter(|t| (first..=last).contains(&t.0)) {
            prop_assert!(range.contains(token), "missing {:?}", token);
        }
    }

    /// Everything computed from the live buffer: never panics, always valid.
    #[test]
    fn buffer_handlers_survive_broken_source(
        source in solidity(),
        positions in prop::collection::vec((0u32..300, 0u32..120), 1..5),
    ) {
        prop_assume!(!has_bare_cr(&source));
        let positions: Vec<Position> = positions.into_iter().map(|(l, c)| Position::new(l, c)).collect();
        let bytes = source.as_bytes();

        assert_clean("textDocument/foldingRange", None, folding::folding_ranges(&source), &source);
        assert_clean("textDocument/documentSymbol", None, symbols::extract_document_symbols(&source), &source);
        assert_clean(
            "textDocument/selectionRange",
            Some(serde_json::json!({ "positions": positions })),
            selection::selection_ranges(&source, &positions),
            &source,
        );
        links::ts_find_imports(bytes);
        for &position in &positions {
            assert_clean("textDocument/documentHighlight", None, highlight::document_highlights(&source, position), &source);
            rename::get_identifier_at_position(bytes, position);
            rename::get_identifier_range(bytes, position);
            goto::cursor_context(&source, position);
            links::ts_cursor_in_import_string(bytes, position);
            links::ts_cursor_in_assembly_flags(bytes, position);
        }
    }
}

#[test]
fn inserts_survive_next_to_a_replacement_at_the_same_start() {
    let edit = |start, end, new_text: &str| TextEdit {
        range: Range::new(Position::new(0, start), Position::new(0, end)),
        new_text: new_text.to_string(),
    };
    let edits = [edit(1, 3, "X"), edit(1, 1, "a"), edit(1, 1, "b")];
    assert_eq!(apply_text_edits("hello", &edits), "habXlo");
}

#[test]
fn wider_edit_wins_when_replacements_share_a_start() {
    let edit = |end, new_text: &str| TextEdit {
        range: Range::new(Position::new(0, 0), Position::new(0, end)),
        new_text: new_text.to_string(),
    };
    assert_eq!(
        apply_text_edits("hello", &[edit(1, "a"), edit(3, "b")]),
        "blo"
    );
}
