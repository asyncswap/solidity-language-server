use serde::{Deserialize, Serialize};

/// Type wrapper for AST node IDs.
///
/// Every node in the Solidity compiler's JSON AST has a unique numeric `id`.
/// Wrapping it prevents accidental mixups with [`FileId`] or plain integers.
///
/// Signed because solc uses negative IDs for built-in symbols (e.g. `-1` for
/// `abi`, `-15` for `msg`, `-18` for `require`, `-28` for `this`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub i64);

/// Newtype wrapper for source file IDs.
///
/// The compiler assigns each input file a numeric ID that appears in `src`
/// strings (`"offset:length:fileId"`). Wrapping it prevents accidental
/// mixups with [`NodeId`] or plain integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FileId(pub u64);

/// A parsed `"offset:length:fileId"` source location from the AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLoc {
    /// Byte offset in the source file.
    pub offset: usize,
    /// Byte length of the source range.
    pub length: usize,
    /// ID of the source file this location belongs to.
    pub file_id: FileId,
}

impl SourceLoc {
    /// Parse a `"offset:length:fileId"` string.
    ///
    /// Returns `None` if the format is invalid or any component fails to parse.
    pub fn parse(src: &str) -> Option<Self> {
        let mut parts = src.split(':');
        let offset = parts.next()?.parse::<usize>().ok()?;
        let length = parts.next()?.parse::<usize>().ok()?;
        let file_id = parts.next()?.parse::<u64>().ok()?;
        // Reject if there are extra parts
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            offset,
            length,
            file_id: FileId(file_id),
        })
    }

    /// End byte offset (`offset + length`).
    pub fn end(&self) -> usize {
        self.offset + self.length
    }

    /// The file ID as a string, for use as a HashMap key when interacting
    /// with the `source_id_to_path` map (which uses string keys).
    pub fn file_id_str(&self) -> String {
        self.file_id.0.to_string()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for FileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── Selector types ─────────────────────────────────────────────────────────

/// 4-byte function selector (`keccak256(signature)[0..4]`).
///
/// Used for external/public functions, public state variable getters,
/// and custom errors. Stored as 8-char lowercase hex without `0x` prefix,
/// matching the format solc uses in AST `functionSelector` / `errorSelector`
/// fields and in `evm.methodIdentifiers` values.
///
/// # Examples
/// ```ignore
/// FuncSelector::new("f3cd914c")   // PoolManager.swap
/// FuncSelector::new("8da5cb5b")   // Ownable.owner
/// FuncSelector::new("0d89438e")   // DelegateCallNotAllowed error
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FuncSelector(String);

impl FuncSelector {
    /// Wrap a raw 8-char hex string (no `0x` prefix).
    pub fn new(hex: impl Into<String>) -> Self {
        Self(hex.into())
    }

    /// The raw hex string (no `0x` prefix), e.g. `"f3cd914c"`.
    pub fn as_hex(&self) -> &str {
        &self.0
    }

    /// Display with `0x` prefix, e.g. `"0xf3cd914c"`.
    pub fn to_prefixed(&self) -> String {
        format!("0x{}", self.0)
    }
}

impl std::fmt::Display for FuncSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 32-byte event topic (`keccak256(signature)`).
///
/// Used for events. Stored as 64-char lowercase hex without `0x` prefix,
/// matching the format solc uses in the AST `eventSelector` field.
///
/// # Examples
/// ```ignore
/// EventSelector::new("8be0079c...") // OwnershipTransferred
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventSelector(String);

impl EventSelector {
    /// Wrap a raw 64-char hex string (no `0x` prefix).
    pub fn new(hex: impl Into<String>) -> Self {
        Self(hex.into())
    }

    /// The raw hex string (no `0x` prefix).
    pub fn as_hex(&self) -> &str {
        &self.0
    }

    /// Display with `0x` prefix.
    pub fn to_prefixed(&self) -> String {
        format!("0x{}", self.0)
    }
}

impl std::fmt::Display for EventSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A selector extracted from an AST declaration node.
///
/// Unifies [`FuncSelector`] (functions, errors, public variables) and
/// [`EventSelector`] (events) into a single enum so callers can handle
/// both with one match.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Selector {
    /// 4-byte selector for functions, public variables, and errors.
    Func(FuncSelector),
    /// 32-byte topic hash for events.
    Event(EventSelector),
}

impl Selector {
    /// The raw hex string (no `0x` prefix).
    pub fn as_hex(&self) -> &str {
        match self {
            Selector::Func(s) => s.as_hex(),
            Selector::Event(s) => s.as_hex(),
        }
    }

    /// Display with `0x` prefix.
    pub fn to_prefixed(&self) -> String {
        match self {
            Selector::Func(s) => s.to_prefixed(),
            Selector::Event(s) => s.to_prefixed(),
        }
    }
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selector::Func(s) => write!(f, "{s}"),
            Selector::Event(s) => write!(f, "{s}"),
        }
    }
}

/// Canonical ABI method signature from `evm.methodIdentifiers`.
///
/// This is the full ABI-encoded signature string like
/// `"swap((address,address,uint24,int24,address),(bool,int256,uint160),bytes)"`.
/// Unlike Solidity source signatures (which use struct names like `PoolKey`),
/// these use fully-expanded tuple types. They are also the keys used in
/// solc's `userdoc` and `devdoc` output.
///
/// Paired with a [`FuncSelector`] via `evm.methodIdentifiers`:
/// `{ "swap(...)": "f3cd914c" }`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodId(String);

impl MethodId {
    /// Wrap a canonical ABI signature string.
    pub fn new(sig: impl Into<String>) -> Self {
        Self(sig.into())
    }

    /// The canonical signature, e.g. `"swap((address,...),bytes)"`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The function/error name (text before the first `(`).
    pub fn name(&self) -> &str {
        self.0.split('(').next().unwrap_or(&self.0)
    }
}

impl std::fmt::Display for MethodId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_loc_parse_valid() {
        let loc = SourceLoc::parse("100:50:3").unwrap();
        assert_eq!(loc.offset, 100);
        assert_eq!(loc.length, 50);
        assert_eq!(loc.file_id, FileId(3));
        assert_eq!(loc.end(), 150);
        assert_eq!(loc.file_id_str(), "3");
    }

    #[test]
    fn test_source_loc_parse_zero() {
        let loc = SourceLoc::parse("0:0:0").unwrap();
        assert_eq!(loc.offset, 0);
        assert_eq!(loc.length, 0);
        assert_eq!(loc.file_id, FileId(0));
    }

    #[test]
    fn test_source_loc_parse_invalid_format() {
        assert!(SourceLoc::parse("").is_none());
        assert!(SourceLoc::parse("100").is_none());
        assert!(SourceLoc::parse("100:50").is_none());
        assert!(SourceLoc::parse("abc:50:3").is_none());
        assert!(SourceLoc::parse("100:abc:3").is_none());
        assert!(SourceLoc::parse("100:50:abc").is_none());
    }

    #[test]
    fn test_source_loc_parse_rejects_extra_parts() {
        assert!(SourceLoc::parse("100:50:3:extra").is_none());
    }

    #[test]
    fn test_node_id_equality() {
        assert_eq!(NodeId(42), NodeId(42));
        assert_ne!(NodeId(42), NodeId(43));
    }

    #[test]
    fn test_file_id_equality() {
        assert_eq!(FileId(1), FileId(1));
        assert_ne!(FileId(1), FileId(2));
    }

    #[test]
    fn test_node_id_file_id_are_different_types() {
        // This test documents the compile-time guarantee.
        // NodeId(1) and FileId(1) are different types — they cannot be
        // compared or used interchangeably.
        let _n: NodeId = NodeId(1);
        let _f: FileId = FileId(1);
        // If you uncomment the following line, it won't compile:
        // assert_ne!(_n, _f);
    }

    // ── Selector type tests ────────────────────────────────────────────

    #[test]
    fn test_func_selector_display() {
        let sel = FuncSelector::new("f3cd914c");
        assert_eq!(sel.as_hex(), "f3cd914c");
        assert_eq!(sel.to_prefixed(), "0xf3cd914c");
        assert_eq!(format!("{sel}"), "f3cd914c");
    }

    #[test]
    fn test_func_selector_equality() {
        assert_eq!(FuncSelector::new("f3cd914c"), FuncSelector::new("f3cd914c"));
        assert_ne!(FuncSelector::new("f3cd914c"), FuncSelector::new("8da5cb5b"));
    }

    #[test]
    fn test_event_selector_display() {
        let sel =
            EventSelector::new("8be0079c5114abcdef1234567890abcdef1234567890abcdef1234567890abcd");
        assert_eq!(sel.as_hex().len(), 64);
        assert!(sel.to_prefixed().starts_with("0x"));
    }

    #[test]
    fn test_selector_enum_variants() {
        let func = Selector::Func(FuncSelector::new("f3cd914c"));
        let event = Selector::Event(EventSelector::new("a".repeat(64)));

        assert_eq!(func.as_hex(), "f3cd914c");
        assert_eq!(func.to_prefixed(), "0xf3cd914c");
        assert_eq!(event.as_hex().len(), 64);
    }

    #[test]
    fn test_method_id() {
        let mid = MethodId::new(
            "swap((address,address,uint24,int24,address),(bool,int256,uint160),bytes)",
        );
        assert_eq!(mid.name(), "swap");
        assert!(mid.as_str().starts_with("swap("));
    }

    #[test]
    fn test_method_id_no_params() {
        let mid = MethodId::new("settle()");
        assert_eq!(mid.name(), "settle");
    }

    #[test]
    fn test_func_selector_hashmap_key() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(FuncSelector::new("f3cd914c"), "swap");
        map.insert(FuncSelector::new("8da5cb5b"), "owner");
        assert_eq!(map.get(&FuncSelector::new("f3cd914c")), Some(&"swap"));
        assert_eq!(map.get(&FuncSelector::new("8da5cb5b")), Some(&"owner"));
    }
}
