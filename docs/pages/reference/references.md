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

`NodeInfo` includes fields like `src`, `name_location`, and `referenced_declaration`.

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
- Deduplicate by `(uri, start, end)` and return.

This is why you get both local and cross-file references in one response when caches are available.

## How target resolution works

Inside `references.rs`, resolution follows this order:

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

## Cross-file behavior: stable identity, not unstable node IDs

Node IDs are not stable across independent builds.  
Cross-file references therefore do **not** share raw node IDs between builds.

Instead, the server derives a stable identity:

- declaration file absolute path (`def_abs_path`)
- declaration byte offset (`def_byte_offset`)

Then each other cached build re-resolves that location locally via `byte_to_id(&build.nodes, def_abs_path, def_byte_offset)` — note that `byte_to_id` takes the unwrapped `nodes` map from the build, not a full `CachedBuild` — and collects matching references in that build.

One important detail: resolution prefers `name_location` over `src` for declarations when available, so cross-file matching lands on the symbol name itself rather than a broader declaration span.

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
