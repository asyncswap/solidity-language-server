use solidity_language_server::utils::{
    byte_offset_to_position, is_valid_solidity_identifier, position_to_byte_offset,
};
use tower_lsp::lsp_types::Position;

#[test]
fn test_byte_offset_to_position_unix_newlines() {
    let source = "line1\nline2\nline3\n";
    assert_eq!(byte_offset_to_position(source, 0), Position::new(0, 0)); // 'l' in line1
    assert_eq!(byte_offset_to_position(source, 5), Position::new(0, 5)); // '\n'
    assert_eq!(byte_offset_to_position(source, 6), Position::new(1, 0)); // 'l' in line2
    assert_eq!(byte_offset_to_position(source, 11), Position::new(1, 5)); // '\n'
    assert_eq!(byte_offset_to_position(source, 12), Position::new(2, 0)); // 'l' in line3
}

#[test]
fn test_byte_offset_to_position_windows_newlines() {
    let source = "line1\r\nline2\r\nline3\r\n";
    assert_eq!(byte_offset_to_position(source, 0), Position::new(0, 0));
    assert_eq!(byte_offset_to_position(source, 5), Position::new(0, 5));
    assert_eq!(byte_offset_to_position(source, 7), Position::new(1, 0)); // skips \r\n
    assert_eq!(byte_offset_to_position(source, 12), Position::new(1, 5));
    assert_eq!(byte_offset_to_position(source, 14), Position::new(2, 0));
}

#[test]
fn test_byte_offset_to_position_no_newlines() {
    let source = "justoneline";
    assert_eq!(byte_offset_to_position(source, 0), Position::new(0, 0));
    assert_eq!(byte_offset_to_position(source, 5), Position::new(0, 5));
    assert_eq!(byte_offset_to_position(source, 11), Position::new(0, 11));
}

#[test]
fn test_byte_offset_to_position_offset_out_of_bounds() {
    let source = "short\nfile";
    let offset = source.len() + 10;
    assert_eq!(byte_offset_to_position(source, offset), Position::new(1, 4));
}

#[test]
fn test_byte_offset_to_position_empty_source() {
    let source = "";
    assert_eq!(byte_offset_to_position(source, 0), Position::new(0, 0));
    assert_eq!(byte_offset_to_position(source, 10), Position::new(0, 0));
}

#[test]
fn test_position_to_byte_offset_basic() {
    let source = "line1\nline2\nline3\n";
    assert_eq!(position_to_byte_offset(source, Position::new(0, 0)), 0); // 'l'
    assert_eq!(position_to_byte_offset(source, Position::new(0, 5)), 5); // '\n'
    assert_eq!(position_to_byte_offset(source, Position::new(1, 0)), 6); // 'l' in line2
    assert_eq!(position_to_byte_offset(source, Position::new(1, 3)), 9); // 'e' in line2
    assert_eq!(position_to_byte_offset(source, Position::new(2, 0)), 12); // 'l' in line3
}

#[test]
fn test_position_to_byte_offset_out_of_bounds() {
    let source = "line1\nline2\n";
    assert_eq!(
        position_to_byte_offset(source, Position::new(10, 10)),
        source.len()
    );
}

#[test]
fn test_position_to_byte_offset_empty() {
    let source = "";
    assert_eq!(position_to_byte_offset(source, Position::new(0, 0)), 0);
}

#[test]
fn test_is_valid_solidity_identifier() {
    assert!(is_valid_solidity_identifier("validName"));
    assert!(is_valid_solidity_identifier("_valid"));
    assert!(is_valid_solidity_identifier("a"));
    assert!(is_valid_solidity_identifier("name123"));
    assert!(is_valid_solidity_identifier("_"));
    assert!(is_valid_solidity_identifier("a_b_c"));
    assert!(is_valid_solidity_identifier("$token"));
    assert!(is_valid_solidity_identifier("$"));
    assert!(is_valid_solidity_identifier("my$var"));

    assert!(!is_valid_solidity_identifier(""));
    assert!(!is_valid_solidity_identifier("123invalid"));
    assert!(!is_valid_solidity_identifier("invalid-name"));
    assert!(!is_valid_solidity_identifier("invalid name"));
    assert!(!is_valid_solidity_identifier("invalid.name"));
}

#[test]
fn test_rejects_active_keywords() {
    let keywords = [
        "abstract",
        "address",
        "anonymous",
        "as",
        "assembly",
        "bool",
        "break",
        "bytes",
        "calldata",
        "catch",
        "constant",
        "constructor",
        "continue",
        "contract",
        "delete",
        "do",
        "else",
        "emit",
        "enum",
        "event",
        "external",
        "fallback",
        "false",
        "fixed",
        "for",
        "function",
        "hex",
        "if",
        "immutable",
        "import",
        "indexed",
        "interface",
        "internal",
        "is",
        "library",
        "mapping",
        "memory",
        "modifier",
        "new",
        "override",
        "payable",
        "pragma",
        "private",
        "public",
        "pure",
        "receive",
        "return",
        "returns",
        "storage",
        "string",
        "struct",
        "true",
        "try",
        "type",
        "ufixed",
        "unchecked",
        "unicode",
        "using",
        "view",
        "virtual",
        "while",
    ];
    for kw in keywords {
        assert!(
            !is_valid_solidity_identifier(kw),
            "{kw:?} should be rejected"
        );
    }
}

#[test]
fn test_rejects_reserved_keywords() {
    let reserved = [
        "after",
        "alias",
        "apply",
        "auto",
        "byte",
        "case",
        "copyof",
        "default",
        "define",
        "final",
        "implements",
        "in",
        "inline",
        "let",
        "macro",
        "match",
        "mutable",
        "null",
        "of",
        "partial",
        "promise",
        "reference",
        "relocatable",
        "sealed",
        "sizeof",
        "static",
        "supports",
        "switch",
        "typedef",
        "typeof",
        "var",
    ];
    for kw in reserved {
        assert!(
            !is_valid_solidity_identifier(kw),
            "{kw:?} should be rejected"
        );
    }
}

#[test]
fn test_rejects_numeric_type_keywords() {
    // int / uint bare
    assert!(!is_valid_solidity_identifier("int"));
    assert!(!is_valid_solidity_identifier("uint"));

    // int<N> / uint<N> for valid sizes
    for n in (8..=256).step_by(8) {
        let int_kw = format!("int{n}");
        let uint_kw = format!("uint{n}");
        assert!(
            !is_valid_solidity_identifier(&int_kw),
            "{int_kw:?} should be rejected"
        );
        assert!(
            !is_valid_solidity_identifier(&uint_kw),
            "{uint_kw:?} should be rejected"
        );
    }

    // bytes1..bytes32
    for n in 1..=32 {
        let bytes_kw = format!("bytes{n}");
        assert!(
            !is_valid_solidity_identifier(&bytes_kw),
            "{bytes_kw:?} should be rejected"
        );
    }
}

#[test]
fn test_allows_identifier_keywords() {
    // These 7 keywords are explicitly allowed as identifiers per SolidityParser.g4
    let allowed = [
        "from",
        "error",
        "revert",
        "global",
        "transient",
        "layout",
        "at",
    ];
    for kw in allowed {
        assert!(
            is_valid_solidity_identifier(kw),
            "{kw:?} should be allowed as identifier"
        );
    }
}

#[test]
fn test_numeric_type_edge_cases() {
    // Invalid sizes should still be valid identifiers (not keywords)
    assert!(is_valid_solidity_identifier("int7"));
    assert!(is_valid_solidity_identifier("int257"));
    assert!(is_valid_solidity_identifier("int0"));
    assert!(is_valid_solidity_identifier("uint3"));
    assert!(is_valid_solidity_identifier("uint512"));
    assert!(is_valid_solidity_identifier("bytes0"));
    assert!(is_valid_solidity_identifier("bytes33"));
    assert!(is_valid_solidity_identifier("bytes64"));
}

// ---------------------------------------------------------------------------
// UTF-16 encoding-aware tests (default encoding = UTF-16)
//
// Characters used:
//   '█' = U+2588  -> 3 bytes UTF-8, 1 UTF-16 code unit
//   '░' = U+2591  -> 3 bytes UTF-8, 1 UTF-16 code unit
//   '😀' = U+1F600 -> 4 bytes UTF-8, 2 UTF-16 code units (surrogate pair)
//   'é' = U+00E9  -> 2 bytes UTF-8, 1 UTF-16 code unit
// ---------------------------------------------------------------------------

#[test]
fn test_byte_offset_to_position_utf16_bmp_chars() {
    // "a█b" — 'a' is 1 byte, '█' is 3 bytes (U+2588), 'b' is 1 byte
    // Total: 5 bytes
    // UTF-16 columns: a=0, █=1, b=2
    let source = "a█b";
    assert_eq!(source.len(), 5);

    assert_eq!(byte_offset_to_position(source, 0), Position::new(0, 0)); // before 'a'
    assert_eq!(byte_offset_to_position(source, 1), Position::new(0, 1)); // before '█' (byte 1)
    assert_eq!(byte_offset_to_position(source, 4), Position::new(0, 2)); // before 'b' (byte 4)
    assert_eq!(byte_offset_to_position(source, 5), Position::new(0, 3)); // end
}

#[test]
fn test_byte_offset_to_position_utf16_surrogate_pair() {
    // "a😀b" — 'a' is 1 byte, '😀' is 4 bytes (U+1F600), 'b' is 1 byte
    // Total: 6 bytes
    // UTF-16: a=col 0, 😀=col 1 (takes 2 UTF-16 code units), b=col 3
    let source = "a😀b";
    assert_eq!(source.len(), 6);

    assert_eq!(byte_offset_to_position(source, 0), Position::new(0, 0)); // before 'a'
    assert_eq!(byte_offset_to_position(source, 1), Position::new(0, 1)); // before '😀'
    assert_eq!(byte_offset_to_position(source, 5), Position::new(0, 3)); // before 'b' (col 1+2=3)
    assert_eq!(byte_offset_to_position(source, 6), Position::new(0, 4)); // end
}

#[test]
fn test_byte_offset_to_position_utf16_mixed_multibyte() {
    // "é█😀x" — é=2B, █=3B, 😀=4B, x=1B -> 10 bytes total
    // UTF-16 columns: é=0(1 unit), █=1(1 unit), 😀=2(2 units), x=4
    let source = "é█😀x";
    assert_eq!(source.len(), 10);

    assert_eq!(byte_offset_to_position(source, 0), Position::new(0, 0)); // before 'é'
    assert_eq!(byte_offset_to_position(source, 2), Position::new(0, 1)); // before '█'
    assert_eq!(byte_offset_to_position(source, 5), Position::new(0, 2)); // before '😀'
    assert_eq!(byte_offset_to_position(source, 9), Position::new(0, 4)); // before 'x'
    assert_eq!(byte_offset_to_position(source, 10), Position::new(0, 5)); // end
}

#[test]
fn test_byte_offset_to_position_utf16_multiline() {
    // Line 0: "█░\n" — 3+3+1 = 7 bytes; UTF-16 cols: █=0, ░=1
    // Line 1: "a😀\n" — 1+4+1 = 6 bytes; UTF-16 cols: a=0, 😀=1
    // Line 2: "z"
    let source = "█░\na😀\nz";
    assert_eq!(source.len(), 14);

    // Line 0
    assert_eq!(byte_offset_to_position(source, 0), Position::new(0, 0)); // '█'
    assert_eq!(byte_offset_to_position(source, 3), Position::new(0, 1)); // '░'
    assert_eq!(byte_offset_to_position(source, 6), Position::new(0, 2)); // '\n'
    // Line 1
    assert_eq!(byte_offset_to_position(source, 7), Position::new(1, 0)); // 'a'
    assert_eq!(byte_offset_to_position(source, 8), Position::new(1, 1)); // '😀'
    assert_eq!(byte_offset_to_position(source, 12), Position::new(1, 3)); // '\n'
    // Line 2
    assert_eq!(byte_offset_to_position(source, 13), Position::new(2, 0)); // 'z'
}

#[test]
fn test_position_to_byte_offset_utf16_bmp_chars() {
    let source = "a█b";
    assert_eq!(position_to_byte_offset(source, Position::new(0, 0)), 0); // 'a'
    assert_eq!(position_to_byte_offset(source, Position::new(0, 1)), 1); // '█' starts at byte 1
    assert_eq!(position_to_byte_offset(source, Position::new(0, 2)), 4); // 'b' starts at byte 4
}

#[test]
fn test_position_to_byte_offset_utf16_surrogate_pair() {
    // "a😀b" — UTF-16 cols: a=0, 😀=1(2 units), b=3
    let source = "a😀b";
    assert_eq!(position_to_byte_offset(source, Position::new(0, 0)), 0); // 'a'
    assert_eq!(position_to_byte_offset(source, Position::new(0, 1)), 1); // '😀' at byte 1
    assert_eq!(position_to_byte_offset(source, Position::new(0, 3)), 5); // 'b' at byte 5
}

#[test]
fn test_position_to_byte_offset_utf16_multiline() {
    let source = "█░\na😀\nz";
    // Line 0: █=col0, ░=col1
    assert_eq!(position_to_byte_offset(source, Position::new(0, 0)), 0);
    assert_eq!(position_to_byte_offset(source, Position::new(0, 1)), 3);
    // Line 1: a=col0, 😀=col1, end=col3
    assert_eq!(position_to_byte_offset(source, Position::new(1, 0)), 7);
    assert_eq!(position_to_byte_offset(source, Position::new(1, 1)), 8);
    assert_eq!(position_to_byte_offset(source, Position::new(1, 3)), 12);
    // Line 2: z=col0
    assert_eq!(position_to_byte_offset(source, Position::new(2, 0)), 13);
}

#[test]
fn test_roundtrip_utf16_byte_to_pos_to_byte() {
    // Verify byte_offset -> position -> byte_offset roundtrips for every char boundary
    let source = "ab😀é█cd\nef";
    let char_boundaries: Vec<usize> = source.char_indices().map(|(i, _)| i).collect();
    for &byte_off in &char_boundaries {
        let pos = byte_offset_to_position(source, byte_off);
        let recovered = position_to_byte_offset(source, pos);
        assert_eq!(
            recovered, byte_off,
            "roundtrip failed for byte_off={byte_off}: pos=({},{}) -> {recovered}",
            pos.line, pos.character
        );
    }
}

#[test]
fn test_position_past_end_of_line_stops_before_the_line_ending() {
    let pos = Position::new(0, 99);
    assert_eq!(position_to_byte_offset("ab\ncd", pos), 2);
    assert_eq!(position_to_byte_offset("ab\r\ncd", pos), 2);
    assert_eq!(position_to_byte_offset("ab\rcd", pos), 2);
}

#[test]
fn test_bare_carriage_return_ends_a_line() {
    assert_eq!(byte_offset_to_position("a\rb", 2), Position::new(1, 0));
    assert_eq!(byte_offset_to_position("a\rb", 3), Position::new(1, 1));
    assert_eq!(position_to_byte_offset("a\rb", Position::new(1, 0)), 2);
}

#[test]
fn test_unicode_line_separators_do_not_end_a_line() {
    for separator in ['\u{2028}', '\u{2029}'] {
        let source = format!("a{separator}b");
        let end = source.len();
        assert_eq!(byte_offset_to_position(&source, end), Position::new(0, 3));
        assert_eq!(position_to_byte_offset(&source, Position::new(0, 2)), 4);
    }
}

#[test]
fn test_offset_inside_crlf_reports_the_end_of_the_line() {
    assert_eq!(byte_offset_to_position("ab\r\ncd", 3), Position::new(0, 2));
}

#[test]
fn test_offset_inside_a_character_clamps_to_its_start() {
    assert_eq!(byte_offset_to_position("a😀b", 3), Position::new(0, 1));
}

#[test]
fn test_column_inside_surrogate_pair_rounds_up() {
    assert_eq!(position_to_byte_offset("😀b", Position::new(0, 1)), 4);
}
