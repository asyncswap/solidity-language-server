use serde_json::Value;
use std::sync::OnceLock;
use tower_lsp::lsp_types::{Position, PositionEncodingKind};

// ---------------------------------------------------------------------------
// Position Encoding
// ---------------------------------------------------------------------------

static ENCODING: OnceLock<PositionEncoding> = OnceLock::new();

/// Store the negotiated encoding.  Called exactly once from the LSP
/// `initialize` handler.  Subsequent calls are silently ignored.
pub fn set_encoding(enc: PositionEncoding) {
    let _ = ENCODING.set(enc);
}

/// Read the negotiated encoding (falls back to UTF-16 if never set).
pub fn encoding() -> PositionEncoding {
    ENCODING.get().copied().unwrap_or_default()
}

/// How the LSP client counts column offsets within a line.
///
/// Set once during `initialize()` via [`set_encoding`] and read implicitly by
/// [`byte_offset_to_position`] and [`position_to_byte_offset`].  All other
/// modules are encoding-agnostic — they never need to know or pass this value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PositionEncoding {
    /// Column = number of bytes from the start of the line (UTF-8 code units).
    Utf8,
    /// Column = number of UTF-16 code units from the start of the line.
    /// This is the **mandatory default** per the LSP specification.
    #[default]
    Utf16,
}

impl PositionEncoding {
    /// Pick the best encoding from the set the client advertises.
    ///
    /// Preference: UTF-8 if supported, otherwise UTF-16 (the mandatory fallback).
    pub fn negotiate(client_encodings: Option<&[PositionEncodingKind]>) -> Self {
        let Some(encodings) = client_encodings else {
            return Self::default();
        };
        if encodings.contains(&PositionEncodingKind::UTF8) {
            PositionEncoding::Utf8
        } else {
            PositionEncoding::Utf16
        }
    }
}

impl From<PositionEncoding> for PositionEncodingKind {
    fn from(value: PositionEncoding) -> Self {
        match value {
            PositionEncoding::Utf8 => PositionEncodingKind::UTF8,
            PositionEncoding::Utf16 => PositionEncodingKind::UTF16,
        }
    }
}

// ---------------------------------------------------------------------------
// Byte-offset to LSP Position conversion
// ---------------------------------------------------------------------------

/// Length of `text` in the column unit of `encoding`.
pub fn encoded_len(text: &str, encoding: PositionEncoding) -> u32 {
    match encoding {
        PositionEncoding::Utf8 => text.len() as u32,
        PositionEncoding::Utf16 => text.encode_utf16().count() as u32,
    }
}

/// Convert a tree-sitter node boundary into a [`Position`], without rescanning
/// the file.
///
/// [`byte_offset_to_position`] walks from byte 0 on every call, which is fine
/// once per request but quadratic when a whole syntax tree is converted node by
/// node. Tree-sitter already reports `row` and a `byte_column` measured from the
/// start of that row, so the line begins at `byte_offset - byte_column` and only
/// that prefix has to be measured.
///
/// `row` stays in tree-sitter's line model, which only breaks on `\n`. That
/// matches [`byte_offset_to_position`] except in files with a bare `\r`.
pub fn byte_column_to_position(
    source: &str,
    row: usize,
    byte_column: usize,
    byte_offset: usize,
) -> Position {
    let column = match encoding() {
        PositionEncoding::Utf8 => byte_column as u32,
        PositionEncoding::Utf16 => {
            let line_start = byte_offset.saturating_sub(byte_column);
            source
                .get(line_start..byte_offset)
                .map_or(byte_column as u32, |prefix| {
                    prefix.encode_utf16().count() as u32
                })
        }
    };
    Position::new(row as u32, column)
}

/// Whether the byte at `index` ends a line. A `\r` followed by `\n` does not:
/// the pair ends at the `\n`. LSP line endings are `\n`, `\r\n` and `\r` only,
/// so U+2028 and U+2029 are ordinary characters here.
fn ends_line(bytes: &[u8], index: usize) -> bool {
    match bytes[index] {
        b'\n' => true,
        b'\r' => bytes.get(index + 1) != Some(&b'\n'),
        _ => false,
    }
}

/// Convert a byte offset in `source` to a [`Position`] whose column unit depends
/// on the negotiated [`PositionEncoding`].
pub fn byte_offset_to_position(source: &str, byte_offset: usize) -> Position {
    byte_offset_to_position_with(source, byte_offset, encoding())
}

/// [`byte_offset_to_position`] with an explicit encoding.
///
/// Offsets past the end clamp to the end, and offsets inside a character clamp
/// to its start. An offset between `\r` and `\n` reports the end of that line.
pub fn byte_offset_to_position_with(
    source: &str,
    byte_offset: usize,
    encoding: PositionEncoding,
) -> Position {
    let mut offset = byte_offset.min(source.len());
    while !source.is_char_boundary(offset) {
        offset -= 1;
    }

    let bytes = source.as_bytes();
    let prefix = &bytes[..offset];

    // Counting `\n` first keeps the common case (no `\r`) a single fast pass.
    let mut line = prefix.iter().filter(|&&b| b == b'\n').count();
    if prefix.contains(&b'\r') {
        line += (0..offset)
            .filter(|&i| bytes[i] == b'\r' && ends_line(bytes, i))
            .count();
    }
    let line_start = (0..offset)
        .rev()
        .find(|&i| ends_line(bytes, i))
        .map_or(0, |i| i + 1);

    let text = &source[line_start..offset];
    let text = if bytes.get(offset) == Some(&b'\n') {
        text.strip_suffix('\r').unwrap_or(text)
    } else {
        text
    };

    Position::new(line as u32, encoded_len(text, encoding))
}

/// Convert an LSP [`Position`] position back to a byte offset, where
/// `character` is interpreted according to the negotiated [`PositionEncoding`].
pub fn position_to_byte_offset(source: &str, pos: Position) -> usize {
    position_to_byte_offset_with(source, pos, encoding())
}

/// [`position_to_byte_offset`] with an explicit encoding.
///
/// Per the LSP spec, a column past the end of the line clamps to the end of the
/// line, before its line ending. A line past the end clamps to the end of the
/// source. A column inside a UTF-8 or UTF-16 sequence rounds up to the next
/// character, so the result is always a character boundary.
pub fn position_to_byte_offset_with(
    source: &str,
    pos: Position,
    encoding: PositionEncoding,
) -> usize {
    let bytes = source.as_bytes();

    let mut line_start = 0;
    for _ in 0..pos.line {
        match (line_start..bytes.len()).find(|&i| ends_line(bytes, i)) {
            Some(end) => line_start = end + 1,
            None => return source.len(),
        }
    }

    let mut column = 0;
    for (index, c) in source[line_start..].char_indices() {
        if column >= pos.character || matches!(c, '\n' | '\r') {
            return line_start + index;
        }
        column += match encoding {
            PositionEncoding::Utf8 => c.len_utf8() as u32,
            PositionEncoding::Utf16 => c.len_utf16() as u32,
        };
    }
    source.len()
}

// ---------------------------------------------------------------------------
// Identifier validation
// ---------------------------------------------------------------------------

/// Check whether `name` is a valid Solidity identifier
pub fn is_valid_solidity_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' && first != '$' {
        return false;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$') {
        return false;
    }
    if SOLIDITY_KEYWORDS.contains(&name) {
        return false;
    }
    if is_numeric_type_keyword(name) {
        return false;
    }
    true
}

/// Keywords that are not allowed as identifiers in Solidity.
///
/// The grammar permits only 7 keywords as identifiers:
/// `from`, `error`, `revert`, `global`, `transient`, `layout`, `at`.
/// Everything else listed in the lexer is blacklisted here.
const SOLIDITY_KEYWORDS: &[&str] = &[
    // Active keywords
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
    // Reserved keywords (future use)
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

/// Check whether `name` is a numeric-type keyword: `int<N>`, `uint<N>`, or `bytes<N>`.
fn is_numeric_type_keyword(name: &str) -> bool {
    if let Some(suffix) = name
        .strip_prefix("uint")
        .or_else(|| name.strip_prefix("int"))
    {
        if suffix.is_empty() {
            return true;
        }
        let Ok(n) = suffix.parse::<u16>() else {
            return false;
        };
        return (8..=256).contains(&n) && n % 8 == 0;
    }
    if let Some(suffix) = name.strip_prefix("bytes") {
        // bare "bytes" is in SOLIDITY_KEYWORDS; only "bytes1"–"bytes32" are handled here
        let Ok(n) = suffix.parse::<u16>() else {
            return false;
        };
        return (1..=32).contains(&n);
    }
    false
}

// ---------------------------------------------------------------------------
// JSON AST helpers
// ---------------------------------------------------------------------------

/// Push a JSON value onto `stack` if the field `key` is an object or array.
///
/// Used during AST walks that iterate over child nodes without full
/// typed deserialization. Objects are pushed as-is; arrays are flattened
/// so each element is pushed individually.
pub fn push_if_node_or_array<'a>(tree: &'a Value, key: &str, stack: &mut Vec<&'a Value>) {
    if let Some(value) = tree.get(key) {
        match value {
            Value::Array(arr) => stack.extend(arr),
            Value::Object(_) => stack.push(value),
            _ => {}
        }
    }
}
