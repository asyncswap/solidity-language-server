use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Type wrapper for Solidity compiler diagnostic error codes.
///
/// Solc and forge diagnostics carry numeric codes like `2072`, `1878`, and
/// `7359`. Wrapping the integer avoids mixing error-code keys with unrelated
/// numeric IDs in maps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ErrorCode(pub u32);

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

    /// The file ID as a [`SolcFileId`], for use as a HashMap key when
    /// interacting with the `id_to_path_map`.
    pub fn file_id_str(&self) -> SolcFileId {
        SolcFileId::new(self.file_id.0.to_string())
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

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for ErrorCode {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl std::borrow::Borrow<u32> for ErrorCode {
    fn borrow(&self) -> &u32 {
        &self.0
    }
}

// ── Typed string wrappers ──────────────────────────────────────────────────
//
// These newtypes replace bare `String` keys in HashMaps so readers can
// instantly tell what a value represents. They serialize as plain strings
// for JSON cache backwards-compatibility.

/// An absolute file path from the AST (`absolutePath` field) or filesystem.
///
/// Used as the key in [`CachedBuild::nodes`], values in
/// [`CachedBuild::path_to_abs`], and keys in [`HintIndex`].
///
/// # Examples
/// ```ignore
/// AbsPath::new("src/PoolManager.sol")            // AST absolutePath
/// AbsPath::new("/Users/me/project/src/Foo.sol")   // filesystem abs path
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AbsPath(String);

impl AbsPath {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Consume and return the inner `String`.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for AbsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for AbsPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for AbsPath {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<std::path::Path> for AbsPath {
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(&self.0)
    }
}

impl From<String> for AbsPath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AbsPath {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl std::borrow::Borrow<str> for AbsPath {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// A project-relative Solidity source path (e.g. `src/Foo.sol`, `lib/X.sol`).
///
/// This wraps keys used in maps like `path_to_abs`, where solc emits paths
/// relative to the project root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RelPath(String);

impl RelPath {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for RelPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for RelPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for RelPath {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for RelPath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for RelPath {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl std::borrow::Borrow<str> for RelPath {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// Composite key for gas estimates: `"<path>:<contract_name>"`.
///
/// Used as the key type in `GasIndex` to avoid mixing this domain-specific
/// composite string with unrelated plain strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GasKey(String);

impl GasKey {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn from_parts(path: &str, contract_name: &str) -> Self {
        Self(format!("{path}:{contract_name}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    /// Split into `(path, contract_name)` using the last `:` separator.
    pub fn split(&self) -> Option<(&str, &str)> {
        self.0.rsplit_once(':')
    }

    pub fn path(&self) -> Option<&str> {
        self.split().map(|(path, _)| path)
    }

    pub fn contract_name(&self) -> Option<&str> {
        self.split().map(|(_, name)| name)
    }
}

impl std::fmt::Display for GasKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for GasKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for GasKey {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for GasKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for GasKey {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl std::borrow::Borrow<str> for GasKey {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// Identifier/symbol name used for completion indexes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SymbolName(String);

impl SymbolName {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for SymbolName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for SymbolName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for SymbolName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SymbolName {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl std::borrow::Borrow<str> for SymbolName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<String> for SymbolName {
    fn borrow(&self) -> &String {
        &self.0
    }
}

/// Solidity `typeIdentifier` string used in completion resolution.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TypeIdentifier(String);

impl TypeIdentifier {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for TypeIdentifier {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TypeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for TypeIdentifier {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TypeIdentifier {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl std::borrow::Borrow<str> for TypeIdentifier {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<String> for TypeIdentifier {
    fn borrow(&self) -> &String {
        &self.0
    }
}

/// A solc source-file ID in string form (e.g. `"0"`, `"34"`, `"127"`).
///
/// The compiler assigns each input file a numeric ID that appears as the
/// third component of `src` strings (`"offset:length:fileId"`). This
/// newtype wraps the stringified form used as keys in
/// [`CachedBuild::id_to_path_map`] and the `id_remap` table during
/// incremental merges.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SolcFileId(String);

impl SolcFileId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for SolcFileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for SolcFileId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for SolcFileId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for SolcFileId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SolcFileId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl std::borrow::Borrow<str> for SolcFileId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// A raw `"offset:length:fileId"` source-location string from the AST.
///
/// This is the **unparsed** form stored on [`NodeInfo::src`] and used as
/// keys in [`ExternalRefs`]. For the parsed representation with typed
/// fields, see [`SourceLoc::parse`].
///
/// # Examples
/// ```ignore
/// SrcLocation::new("2068:10:33")
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SrcLocation(String);

impl SrcLocation {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn into_inner(self) -> String {
        self.0
    }
    /// Parse into a structured [`SourceLoc`].
    pub fn parse(&self) -> Option<SourceLoc> {
        SourceLoc::parse(&self.0)
    }
}

impl std::fmt::Display for SrcLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for SrcLocation {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for SrcLocation {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<&str> for SrcLocation {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<SrcLocation> for &str {
    fn eq(&self, other: &SrcLocation) -> bool {
        *self == other.0
    }
}

impl From<String> for SrcLocation {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SrcLocation {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl std::borrow::Borrow<str> for SrcLocation {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// An LSP document URI string (e.g. `"file:///Users/me/project/src/Foo.sol"`).
///
/// Used as keys in [`ForgeLsp::ast_cache`], [`ForgeLsp::text_cache`],
/// [`ForgeLsp::completion_cache`], [`SemanticTokenCache`], and
/// [`ForgeLsp::did_save_workers`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentUri(String);

impl DocumentUri {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for DocumentUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for DocumentUri {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for DocumentUri {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for DocumentUri {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DocumentUri {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<&String> for DocumentUri {
    fn from(s: &String) -> Self {
        Self(s.clone())
    }
}

impl std::borrow::Borrow<str> for DocumentUri {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<String> for DocumentUri {
    fn borrow(&self) -> &String {
        &self.0
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

// ── Path interner ──────────────────────────────────────────────────────────
//
// Project-wide, append-only table that assigns canonical [`FileId`] values
// from file paths.  Every [`CachedBuild`] translates solc's arbitrary
// per-compilation file IDs into canonical IDs at construction time, so all
// builds share the same ID space and merges never produce file-ID conflicts.

/// Project-wide path interner.
///
/// Assigns deterministic [`FileId`] values based on file paths.  Once a
/// path is interned it keeps its ID for the lifetime of the session.
///
/// The interner is append-only: new paths get monotonically increasing IDs
/// and existing paths keep theirs.  This means canonical IDs are stable
/// across compilations — a property that solc's own file IDs lack.
///
/// Lives on `ForgeLsp` behind `Arc<RwLock<PathInterner>>` so every
/// `CachedBuild` (per-file or project-wide) can share it.
#[derive(Debug, Clone)]
pub struct PathInterner {
    /// Canonical ID → file path.
    paths: Vec<String>,
    /// File path → canonical ID (reverse lookup).
    path_to_id: HashMap<String, u64>,
}

impl PathInterner {
    /// Create an empty interner.
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            path_to_id: HashMap::new(),
        }
    }

    /// Get or assign a canonical [`FileId`] for `path`.
    ///
    /// If `path` was interned before, the same ID is returned.
    /// Otherwise a new ID is allocated (one higher than the current max).
    pub fn intern(&mut self, path: &str) -> FileId {
        if let Some(&id) = self.path_to_id.get(path) {
            return FileId(id);
        }
        let id = self.paths.len() as u64;
        self.paths.push(path.to_owned());
        self.path_to_id.insert(path.to_owned(), id);
        FileId(id)
    }

    /// Look up the file path for a canonical ID.
    pub fn resolve(&self, id: FileId) -> Option<&str> {
        self.paths.get(id.0 as usize).map(|s| s.as_str())
    }

    /// Number of interned paths.
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Whether the interner is empty.
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Build a [`SolcFileId`]-keyed remap table that translates solc's
    /// per-compilation file IDs into canonical IDs.
    ///
    /// `solc_id_to_path` is the `source_id_to_path` map from a single
    /// solc invocation (e.g. `{ "0": "/abs/src/Foo.sol", "3": "/abs/lib/Bar.sol" }`).
    ///
    /// Returns a map from solc file ID → canonical [`FileId`] that can be
    /// used to rewrite `src` strings during [`CachedBuild::new()`].
    pub fn build_remap(
        &mut self,
        solc_id_to_path: &HashMap<SolcFileId, String>,
    ) -> HashMap<u64, FileId> {
        let mut remap = HashMap::with_capacity(solc_id_to_path.len());
        for (solc_id, path) in solc_id_to_path {
            let solc_num: u64 = solc_id.as_str().parse().unwrap_or(u64::MAX);
            let canonical = self.intern(path);
            remap.insert(solc_num, canonical);
        }
        remap
    }

    /// Build a canonical `id_to_path_map` from the interner's current state.
    ///
    /// Returns `HashMap<SolcFileId, String>` in the same shape as the
    /// existing `CachedBuild.id_to_path_map`, but using canonical IDs.
    pub fn to_id_to_path_map(&self) -> HashMap<SolcFileId, String> {
        self.paths
            .iter()
            .enumerate()
            .map(|(i, path)| (SolcFileId::new(i.to_string()), path.clone()))
            .collect()
    }
}

impl Default for PathInterner {
    fn default() -> Self {
        Self::new()
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
        assert_eq!(loc.file_id_str(), SolcFileId::new("3"));
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

    // ── PathInterner tests ─────────────────────────────────────────────

    #[test]
    fn test_path_interner_basic() {
        let mut interner = PathInterner::new();
        assert!(interner.is_empty());

        let id_a = interner.intern("src/Foo.sol");
        let id_b = interner.intern("src/Bar.sol");
        assert_ne!(id_a, id_b);
        assert_eq!(interner.len(), 2);

        // Same path returns the same ID.
        let id_a2 = interner.intern("src/Foo.sol");
        assert_eq!(id_a, id_a2);
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_path_interner_resolve() {
        let mut interner = PathInterner::new();
        let id = interner.intern("/abs/src/Pool.sol");
        assert_eq!(interner.resolve(id), Some("/abs/src/Pool.sol"));
        assert_eq!(interner.resolve(FileId(999)), None);
    }

    #[test]
    fn test_path_interner_monotonic_ids() {
        let mut interner = PathInterner::new();
        let a = interner.intern("a.sol");
        let b = interner.intern("b.sol");
        let c = interner.intern("c.sol");
        assert_eq!(a, FileId(0));
        assert_eq!(b, FileId(1));
        assert_eq!(c, FileId(2));
    }

    #[test]
    fn test_path_interner_build_remap() {
        let mut interner = PathInterner::new();
        // Pre-intern one path from a previous compilation.
        interner.intern("/abs/src/Foo.sol");

        // Simulate solc output where file IDs are different.
        let mut solc_map = HashMap::new();
        solc_map.insert(SolcFileId::new("0"), "/abs/src/Bar.sol".to_string());
        solc_map.insert(SolcFileId::new("1"), "/abs/src/Foo.sol".to_string());

        let remap = interner.build_remap(&solc_map);

        // Foo.sol was already interned as canonical 0.
        assert_eq!(remap[&1], FileId(0));
        // Bar.sol is new, gets canonical 1.
        assert_eq!(remap[&0], FileId(1));
    }

    #[test]
    fn test_path_interner_to_id_to_path_map() {
        let mut interner = PathInterner::new();
        interner.intern("src/A.sol");
        interner.intern("src/B.sol");

        let map = interner.to_id_to_path_map();
        assert_eq!(map.get("0").map(|s| s.as_str()), Some("src/A.sol"));
        assert_eq!(map.get("1").map(|s| s.as_str()), Some("src/B.sol"));
    }
}
