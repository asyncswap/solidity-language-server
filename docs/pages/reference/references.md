# References

## What this page covers

This page documents how `textDocument/references` works in the current implementation:

- how the target symbol is resolved from the cursor,
- how same-file and cross-file references are collected,
- how Yul `externalReferences` are included,
- what is covered by tests today, and what still needs explicit tests.

If you are looking for import-string navigation (for example `import "./Pool.sol"`), that path belongs to go-to-definition and is documented in [`goto.md`](./goto).

## The working model

`references` is built around one core data structure: `CachedBuild`.

`CachedBuild` is a snapshot built from successful compiler output on disk. Internally it stores `HashMap`s such as:

- `nodes: HashMap<AbsPath, HashMap<NodeId, NodeInfo>>`
- `id_to_path_map: HashMap<SolcFileId, String>`
- `external_refs: HashMap<SrcLocation, NodeId>`

`NodeInfo` includes fields like `src`, `name_location`, `referenced_declaration`, and `scope`.

At request time, the server uses this snapshot to resolve references quickly, then merges results from other cached builds to get cross-file coverage.

## Caches and freshness

References use two cache layers:

- in-memory `ast_cache`/`CachedBuild` entries for fast request-time lookups,
- on-disk project cache (`.solidity-language-server/solidity-lsp-schema-v2.json` + shard files) for warm starts.

Current cache behavior (v0.1.29):

- Warm-load reconcile: if source hashes changed, affected files are recompiled and merged before cache write-back.
- Per-save upsert: after successful save/build, touched files are upserted into cache v2 quickly (350ms debounce).
- Single-flight/debounced sync: bursty save events do not spawn overlapping cache jobs (700ms debounce for full sync).
- Scoped incremental reindex is gated by `projectIndex.incrementalEditReindexThreshold`; when the affected ratio is too high, the server falls back to full reindex for correctness.

Practical result: references can be partial right at startup, then become complete as reconcile finishes, while later saves keep cache freshness without full-project recompiles on every edit.

For full cache architecture details, see [Caches](/reference/caches).

## Request flow in practice

In `src/lsp.rs`, the `references` handler does the following:

- Load source bytes and get (or build) `CachedBuild` for the URI.
- Collect current-build references.
- Derive stable target location `(def_abs_path, def_byte_offset)`.
- Scan other cached builds for cross-file references to the same target.
- Deduplicate via `dedup_locations()` — removes exact `(uri, start, end)` duplicates and contained-range duplicates (when one range strictly contains another on the same URI, keep only the narrower range). This prevents qualified type paths like `IPoolManager.ModifyLiquidityParams` from producing two entries per usage site.

This is why you get both local and cross-file references in one response when caches are available.

## How target resolution works

Inside `references.rs`, resolution follows this order:

- Check for qualifier cursor: if the cursor is on the first segment of a multi-segment `IdentifierPath` (e.g., `Pool` in `Pool.State`), `resolve_qualifier_target()` resolves the container via `referencedDeclaration → scope` and dispatches to `collect_qualifier_references()`.
- Try Yul resolution first (`externalReferences` mapping).
- Fall back to AST span match (smallest containing node).
- Normalize to declaration target: follow `referencedDeclaration` when present, else use the node id directly.

That resolved node ID becomes the target for reference collection.

## Yul references are first-class

Yul identifiers inside `assembly {}` do not have normal Solidity node IDs in the same way usage sites do. The implementation bridges this via `InlineAssembly.externalReferences`, which maps Yul `src` ranges to Solidity declaration IDs.

During references:

- cursor-on-Yul is resolved through `external_refs` before normal AST span matching,
- Yul usage locations are also appended back into the result set by matching `decl_id`.

This is why references work inside inline assembly rather than only in high-level Solidity syntax.

## Qualifier references

When the cursor is on the qualifier segment of a qualified type path (e.g., `Pool` in `Pool.State`), the server resolves references for the container (contract/library/interface) rather than the struct/enum member.

**Detection:** `resolve_qualifier_target()` checks that the node is an `IdentifierPath` with `name_locations.len() > 1` and that the cursor falls within `name_locations[0]` (the first segment).

**Resolution chain:** The function follows `referencedDeclaration` (which points to the struct/enum) to find that declaration node, then reads its `scope` field to get the container's node ID.

**Collection:** `collect_qualifier_references()` merges two sources:
- Direct references to the container (imports, expression-position usages) via the normal `all_references` index.
- Qualifier references from the `qualifier_refs` index — `IdentifierPath` nodes where the container appears as the first segment. These are emitted using `nameLocations[0]` to produce the correct narrow range.

**Index:** `CachedBuild.qualifier_refs` (`HashMap<NodeId, Vec<NodeId>>`) is built at cache time by `build_qualifier_refs()`. It scans all multi-segment `IdentifierPath` nodes, follows `referencedDeclaration → scope`, and maps the container ID to the `IdentifierPath` node ID.

**Cross-file:** `goto_references_for_target()` also checks the `qualifier_refs` index when the target is a container, so cross-file scans (used by both the references handler and the rename handler) include qualifier usages from other builds.

**Merge:** `merge_missing_from()` merges `qualifier_refs` entries from other builds so the combined index is complete.

## Canonical file IDs via PathInterner

Solc assigns file IDs sequentially based on input order — the same file gets different IDs in different compilations. This caused cross-compilation reference bugs where `src` strings (format `offset:length:fileId`) from one build could not be compared to another.

The server uses a project-wide `PathInterner` (stored on `ForgeLsp` behind `Arc<RwLock<PathInterner>>`) to assign deterministic canonical file IDs. During `CachedBuild::new()`, all `src` strings in `NodeInfo` fields (`src`, `name_location`, `name_locations`, `member_location`) are rewritten via `remap_src_canonical()` to use the interner's IDs instead of solc's.

Sub-caches (library sub-projects) pass `None` for the interner since they have isolated ID spaces and are matched by file path + byte offset, not by file ID.

The `CompletionCache` is also canonicalized: `build_completion_cache()` accepts a `file_id_remap` to translate `path_to_file_id` and `ScopeRange.file_id` entries through the same canonical mapping.

## Cross-file behavior: stable identity, not unstable node IDs

Node IDs are not stable across independent builds.  
Cross-file references therefore do **not** share raw node IDs between builds.

Instead, the server derives a stable identity:

- declaration file absolute path (`def_abs_path`)
- declaration byte offset (`def_byte_offset`)

Then each other cached build re-resolves that location locally via `byte_to_id(&build.nodes, def_abs_path, def_byte_offset)` — note that `byte_to_id` takes the unwrapped `nodes` map from the build, not a full `CachedBuild` — and collects matching references in that build.

One important detail: resolution prefers `name_location` over `src` for declarations when available, so cross-file matching lands on the symbol name itself rather than a broader declaration span.

## Stale-offset exclusion

After editing a file, the project-level cache may still hold stale AST byte offsets for that file from the previous compilation. When `id_to_location_with_index` converts those stale offsets to line/column positions using the current file content from disk, the shifted offsets produce wrong positions. These wrong positions don't deduplicate against the correct positions from the fresh file-level build, causing duplicate references to appear.

To prevent this, `goto_references_for_target()` accepts an `exclude_abs_path: Option<&str>` parameter. When the references handler scans the project-level cache for cross-file results, it passes the current file's absolute path as the exclusion — skipping all nodes (including Yul external refs) that belong to the current file. The fresh file-level build already provides correct references for the current file with up-to-date byte offsets.

The same exclusion pattern is applied in `rename.rs` for cross-file rename scans.

## includeDeclaration behavior

`includeDeclaration` from the LSP request is honored directly:

- when `true`, declaration location is included,
- when `false`, only usage locations are returned.

This flag is applied in both the current-build pass and cross-file passes.

## What this implementation does not try to do

- It does not treat import string literals as references. Import path navigation is handled by go-to-definition logic.
- It does not guarantee cross-file completeness when another file has no cached build yet. Cross-file scanning only runs over available entries in `ast_cache`.

## Test coverage and confidence

Current tests give good coverage for the core reference architecture:

- `tests/cross_file_references.rs` covers stable cross-file target resolution using `(path, byte_offset)`.
- `tests/rename.rs` includes `goto_references_cached` behavior that relies on `nameLocation` fallback.
- `tests/yul_external_references.rs` covers Yul external-reference indexing and goto/reference mapping behavior.

This gives strong confidence in:

- target resolution correctness,
- cross-file re-resolution strategy,
- Yul assembly integration.

### Recommended explicit additions

The following are good direct test additions for long-term safety:

- `lsp.rs` handler-level test for full merge/dedup behavior across multiple cached builds.
- a direct test for `includeDeclaration = false` through the full request path.
- a mixed Solidity + Yul reference scenario validated end-to-end through the LSP method boundary.
