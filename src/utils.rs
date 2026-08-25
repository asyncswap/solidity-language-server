use lintspec_core::textindex::{TextIndex, compute_indices};
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

/// Convert a tree-sitter node boundary into a [`Position`], without rescanning
/// the file.
///
/// [`byte_offset_to_position`] walks from byte 0 on every call, which is fine
/// once per request but quadratic when a whole syntax tree is converted node by
/// node. Tree-sitter already reports `row` and a `byte_column` measured from the
/// start of that row, so the line begins at `byte_offset - byte_column` and only
/// that prefix has to be measured.
///
/// Taking `row` from tree-sitter also keeps the result in the same line model as
/// the tree being walked; `TextIndex` treats U+2028/U+2029 as line terminators
/// and tree-sitter does not, so deriving the row here instead would put symbols
/// on a different line than the node they came from.
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

/// Convert a byte offset in `source` to a [`Position`] whose column unit depends
/// on the negotiated [`PositionEncoding`].
pub fn byte_offset_to_position(source: &str, byte_offset: usize) -> Position {
    if source.is_empty() {
        return Position::new(0, 0);
    }

    let idx = if byte_offset >= source.len() {
        // Offset is at or past the end of source — walk the entire string.
        // `compute_indices` only handles offsets that fall within the source.
        let mut ti = TextIndex::ZERO;
        let mut chars = source.chars().peekable();
        while let Some(c) = chars.next() {
            ti.advance(c, chars.peek());
        }
        ti
    } else {
        // SIMD-accelerated lookup for offsets within the source.
        let indices = compute_indices(source, &[byte_offset]);
        match indices.first() {
            Some(ti) => *ti,
            None => return Position::new(0, 0),
        }
    };

    Position {
        line: idx.line,
        character: match encoding() {
            PositionEncoding::Utf8 => idx.col_utf8,
            PositionEncoding::Utf16 => idx.col_utf16,
        },
    }
}

/// Convert an LSP [`Position`] position back to a byte offset, where
/// `character` is interpreted according to the negotiated [`PositionEncoding`].
///
/// Uses a single SIMD-accelerated pass with [`compute_indices`] to build a
/// coarse index of the file at 128-byte intervals, then does a short linear
/// walk (at most 128 bytes) with [`TextIndex::advance`] to find the exact
/// byte offset.
pub fn position_to_byte_offset(source: &str, pos: Position) -> usize {
    if source.is_empty() {
        return 0;
    }

    let enc = encoding();

    // 1. Build chunk offsets at 128-byte intervals across the source.
    let chunk_offsets: Vec<usize> = (0..source.len()).step_by(128).collect();

    // 2. Single SIMD-accelerated pass — compute TextIndex for every chunk.
    let chunk_indices = compute_indices(source, &chunk_offsets);

    // 3. Find the last chunk that is still at or before the target position.
    //    Multiple chunks can fall on the same line, so we must also check
    //    the column to avoid starting past the target.
    let start = chunk_indices
        .iter()
        .take_while(|ti| {
            if ti.line < pos.line {
                return true;
            }
            if ti.line == pos.line {
                let col = match enc {
                    PositionEncoding::Utf8 => ti.col_utf8,
                    PositionEncoding::Utf16 => ti.col_utf16,
                };
                return col <= pos.character;
            }
            false
        })
        .last()
        .copied()
        .unwrap_or(TextIndex::ZERO);

    // 4. Linear walk from `start` (at most ~128 bytes) to the exact position.
    let mut idx = start;
    let mut chars = source[idx.utf8..].chars().peekable();

    while let Some(c) = chars.next() {
        let col = match enc {
            PositionEncoding::Utf8 => idx.col_utf8,
            PositionEncoding::Utf16 => idx.col_utf16,
        };
        if idx.line >= pos.line && col >= pos.character {
            return idx.utf8;
        }
        if idx.line == pos.line && c == '\n' {
            return idx.utf8; // clamp to end of line
        }
        idx.advance(c, chars.peek());
    }
    source.len() // position past end of source
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
