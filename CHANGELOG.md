# Changelog

## Unreleased

### Features

- Project-wide `PathInterner` for canonical file IDs — solc assigns file IDs sequentially based on input order, causing cross-compilation reference bugs; the interner assigns deterministic IDs so all builds share the same ID space (#185)
- `textDocument/hover` — show AST node ID (`NodeId: \`<id>\``) in hover tooltips for debugging; references show ``NodeId: \`<id>\` referencedDeclaration: \`<decl_id>\` ``
- `textDocument/references` — qualifier-aware reference resolution; "Find All References" on a container (contract/library/interface) now includes qualified type paths where the container appears as a prefix (e.g., `Pool` in `Pool.State`) (#187)
- `textDocument/definition` — qualifier segment goto; clicking the qualifier in a qualified type path (e.g., `Pool` in `Pool.State`) navigates to the container declaration instead of the struct/enum (#187)
- `CachedBuild` gains `qualifier_refs` field (`HashMap<NodeId, Vec<NodeId>>`) — maps container declaration IDs to `IdentifierPath` node IDs used as qualifiers; built at cache time by `build_qualifier_refs()` and merged across builds via `merge_missing_from()` (#187)
- `NodeInfo` gains `scope` field — the containing declaration's node ID from the AST `scope` property; used to resolve qualified type path containers (#187)
- `dedup_locations()` removes both exact and contained-range duplicates from reference results, preventing qualified type paths from producing duplicate entries (#187)
- Two-phase project index — phase 1 compiles src-closure for fast time-to-first-reference, phase 2 compiles full closure (src + test + script) in background (#189)
- Parallel sub-cache compilation — library sub-projects now build concurrently via `JoinSet` instead of sequentially; Asyncswap benchmark: 53.5s to 15.8s (3.4x speedup) (#190)
- `solidity/subCachesLoaded` progress token — emitted when all library sub-caches are built and loaded, allowing benchmarks and editors to detect full pipeline completion (#190)

### Performance

- Remove `evm.gasEstimates` from solc output selection — gas estimation consumed 88% of compile time (SmarDex 510-file project: 56s to 6s). Removed `gas.rs`, `GasKey`, `gas_index`, gas inlay hints, gas hover, and `gasEstimates` setting (#189, #190)

### Fixes

- Fix stale-offset duplicate references after editing — "Find All References" no longer produces bogus duplicates (e.g. 734 instead of 729 results) when the project cache has stale byte offsets for the current file; `goto_references_for_target()` now excludes the current file from the project-cache scan since the fresh file-level build already covers it (#185)
- `CompletionCache` file IDs are now canonicalized through the `PathInterner` remap, preventing scope-range mismatches across compilations (#185)
- Removed dead `remap_src_file_id` and `remap_node_info_file_ids` functions; simplified `merge_scoped_cached_build()` by removing ~50 lines of `id_remap` machinery (#185)
- Sub-cache loading flag was never reset to `false` after completion — now correctly cleared (#190)

### Tests

- 302 tests, 0 failures, 0 warnings

### Benchmarks

| Project | Files | Before | After | Speedup |
|---|---|---|---|---|
| SmarDex | 510 | ~56s | 14.1s | 4x |
| Asyncswap | 108 + 1108 lib | ~57s | 21.1s | 2.7x |

## v0.1.29

### Features

- `textDocument/codeAction` — JSON-driven quickfix engine; handles `unused-import` forge-lint diagnostic with a "Remove unused import" action (#168)
- `workspace/executeCommand` — `solidity.clearCache` deletes the on-disk cache and wipes in-memory AST, forcing a clean rebuild; `solidity.reindex` evicts in-memory AST and triggers a fast background reindex from the warm disk cache (#175)
- `textDocument/completion` — import path completions inside import strings (#173)
- Non-blocking `didSave` with per-URI watch channel serialisation — rapid saves collapse instead of stacking (#171)

### Fixes

- Skip redundant solc rebuild on save when file content is unchanged — eliminates ~5s recompilation from format-on-save loops on large projects (#181)
- Wrap `collect_import_pragmas` in `spawn_blocking` — the transitive import FS crawl no longer blocks the tokio async runtime, preventing request timeouts on first save of large files (#181)
- `textDocument/references` now returns results for state variables and other declarations (was returning null) — 11 refs for `Shop.sol`, 67 for `PoolManager.t.sol`
- Guarantee full rebuild when reindex races with a running didSave worker (#178)
- Tree-sitter import/assembly string completion guards (#177)
- Use `project_cache_key()` in executeCommand and wake reindex worker correctly (#176)

### Benchmarks

- `Shop.sol`: 26/26 methods, references 11 results, rename 11 edits, codeAction working
- `PoolManager.t.sol`: 26/26 methods, references 67 results, 1082 inlay hints at 9.3ms

## v0.1.28

### Features

- Drop v1 cache path; v2 cache is now the only supported mode
- Auto-create `.gitignore` in the cache directory on first write
- Cache fingerprint now includes `lsp_version` so upgrading the server binary automatically invalidates stale caches

### Fixes

- Pull settings via `workspace/configuration` when `initializationOptions` is absent — fixes Neovim which sends settings only via pull requests, not as `initializationOptions` (#164)
- Use import-closure for project index: BFS from source/test/script roots via forge remappings compiles only files the project actually imports (~510 vs 1788 for v4-core), cross-file `referencedDeclaration` IDs now populated correctly (#165)
- Resolve solc version from transitive import pragmas: intersects pragma constraints across the full import graph so a wildcard pragma file importing an exact-version file picks a compatible solc, fixing empty ASTs and broken goto-definition (#162)
- Use tree-sitter for import parsing in pragma graph walk, correctly handling multi-line import directives
- Correct `textDocument/rename` for aliased imports (`import {A as B}` and `import './X.sol' as Y`) (#167)
- Fix config profile fallback and lint parsing
- Incremental reindex safety with threshold gating and scoped cache merge; warm-load reconciles changed files and persists cache aggressively (#156)

### Docs

- Add `--stdio` to Neovim `cmd` snippets in `DOCS.md`
- Add `projectIndex` settings to all editor config examples
- Add missing `willCreate`/`didCreate`/`willDelete`/`didDelete` capabilities to `neovim-plugin-lsp.lua`
- Fix `CONTRIBUTING.md`: correct fixture name, test count (600), and project structure
- Fix stale benchmark config references across profiling and completions docs

## v0.1.27

### Features

- `projectIndex.fullProjectScan` setting to enable full project source indexing at startup (#152)
- Persistent reference cache with atomic writes and single-flight sync (#153)
- Auto-create `.gitignore` in cache directory (#154)
- Incremental reindex safety with threshold gating and scoped cache merge (#156)
- Drop v1 cache path; v2 cache is now the only supported mode

### Fixes

- Pull settings via `workspace/configuration` when `initializationOptions` is absent (#164)
- Resolve solc version from transitive import pragmas (#162)
- Use import-closure for project index with full cross-file references (#165)
- Correct `textDocument/rename` for aliased imports (`import {A as B}` and `import './X.sol' as Y`) (#167)

### Tests

- 600 total tests, 0 warnings

## v0.1.26

### Features

- File operation behaviors are configurable via:
  - `fileOperations.templateOnCreate`
  - `fileOperations.updateImportsOnRename`
  - `fileOperations.updateImportsOnDelete`
- Default file-operation settings are enabled.
- Template/scaffolding naming standardized to `templateOnCreate`.

### Fixes

- Improve file creation scaffolding flow to avoid missing scaffold content on new files.
- Fix duplicate/incorrect scaffold insertion timing during create-file lifecycle.
- Improve auto-import completion behavior for top-level symbols and import edit attachment.

### Benchmarks

- Added/updated benchmark coverage for file-operation lifecycle flows (`willCreateFiles`,
  `willRenameFiles`, `willDeleteFiles`) and auto-import scenarios.

## v0.1.25

### Performance

- Replace clone-then-strip with build-filtered-map in `walk_and_extract()` (#132)
  - `build_filtered_decl()` / `build_filtered_contract()` iterate borrowed node fields and only
    clone fields that pass the STRIP_FIELDS filter, skipping heavy subtrees (`body`, `modifiers`,
    `value`, `overrides`, etc.)
  - Eliminates 234 MB of transient allocations (629→395 MB total, -37%)
  - RSS: 310 MB → 254 MB (down from 394 MB pre-optimization)
- Pre-size HashMaps with `with_capacity()` in `cache_ids()`, `extract_decl_nodes()`,
  `build_completion_cache()`, `build_hint_index()`, `build_constructor_index()` (#132)
- Remove dead `goto_references()` and `goto_references_with_index()` functions (#132)
- Gate `SolcOutput` / `SourceEntry` behind `#[cfg(test)]` (#132)

### Fixes

- Fix cross-file references contamination: use `nameLocation` instead of `src` in
  `resolve_target_location()` so "Find All References" on `IPoolManager manager` returns
  references to `manager`, not to the `IPoolManager` interface (#131)
- Fix non-deterministic hover on inherited contracts: `byte_to_id()` now prefers
  nodes with `referencedDeclaration` when two nodes share the same span length (#131)

### Memory

| State | RSS | vs v0.1.24 |
|---|---|---|
| v0.1.24 baseline | 230 MB | — |
| Before optimization | 394 MB | +164 MB |
| **v0.1.25** | **254 MB** | **+24 MB** |

DHAT profiling (poolmanager-t-full.json, 95 files):

| Metric | v0.1.24 | v0.1.25 | Delta |
|---|---|---|---|
| Total allocated | 629 MB | 395 MB | -37% |
| Peak (t-gmax) | 277 MB | 243 MB | -12% |
| Retained (t-end) | 60 MB | 60 MB | unchanged |

### Tests

- 458 total tests, 0 warnings

### Benchmarks

Updated for Shop.sol (all competitors), Pool.sol (v0.1.25 vs v0.1.24), and
PoolManager.t.sol (v0.1.25 vs v0.1.24).

## v0.1.24

### Features

- Project-wide source indexing for cross-file references (#115, #119)
- Semantic tokens range and delta support
- LSP settings configuration (#112)
- Benchmark configs with server registry and didChange snapshots (#121)

### Performance

- `textDocument/documentLink` returns only import links, not every identifier (#122)
- Drop optimizer and conditionally exclude gasEstimates from solc input (#117)

### Fixes

- Handle `{value: ...}` / `{gas: ...}` modifier calls in inlay hints and signature help (#125, #116)
- Correct signatureHelp cursor positions to inside function call parens
- Remap all tests from forge to solc fixture (#123)
- CI: checkout submodules so fixture-based tests can find v4-core

### Tests

- 466 total tests, 0 warnings

### Benchmark (v4-core Pool.sol)

| Method | p95 |
|--------|-----|
| initialize | 12.9ms |
| completion | 0.4ms |
| hover | 23.1ms |
| definition | 11.1ms |
| references | 19.4ms |
| rename | 21.2ms |
| inlayHint | 2.8ms |
| signatureHelp | 11.7ms |
| semanticTokens/full | 3.7ms |

Scorecard: **15/18 wins** vs solc, nomicfoundation, juanfranblanco, qiuxiang

## v0.1.23

### Features

- `textDocument/signatureHelp` — shows function signature and active parameter while typing (#110)
- Opt-in gas estimates via `@custom:lsp-enable gas-estimates` NatSpec tag (#109)

## v0.1.22

### Improvements

- Use `svm-rs` as a library for solc version management (#106, #105)
  - `svm::installed_versions()` for discovering installed solc versions
  - `svm::version_binary()` for resolving solc binary paths
  - `svm::install()` for auto-installing missing versions (async, native)
  - No longer shells out to the `svm` CLI or manually walks the filesystem

## v0.1.21

### Features

- Auto-detect solc version from `pragma solidity` and resolve matching binary (#93, #95)
  - Parses pragma constraints: exact (`0.8.26`), caret (`^0.8.0`), gte (`>=0.8.0`), range (`>=0.6.2 <0.9.0`)
  - Scans svm-rs and solc-select install directories for matching versions
  - Auto-installs missing versions via `svm-rs` library
  - Cross-platform support (macOS, Linux, Windows)
  - Cached version list (scanned once per session)
- Solc version resolution respects both pragma and foundry.toml (#103)
  - Exact pragmas (`=0.7.6`) always honoured — foundry.toml cannot override
  - Wildcard pragmas (`^0.8.0`) use foundry.toml version if it satisfies the constraint
  - No pragma falls back to foundry.toml, then system solc
- Foundry config support for compiler settings (#103)
  - Reads `via_ir`, `optimizer`, `optimizer_runs`, `evm_version` from `foundry.toml`
  - Passes settings to solc standard JSON (`viaIR`, `optimizer`, `evmVersion`)
  - Reads `ignored_error_codes` to suppress matching diagnostics
  - Fixes "Stack too deep" errors for projects requiring `via_ir` (e.g. EkuboProtocol/evm-contracts)
- Callsite parameter documentation on hover (#103)
  - Hovering over arguments in function/event calls shows `@param` doc from the called definition
  - Uses tree-sitter on the live buffer to find enclosing call and argument index
  - Resolves via `HintIndex` (exact offset or `(name, arg_count)` fallback) for param name and `decl_id`
  - Looks up `@param` doc from `DocIndex` or raw NatSpec with `@inheritdoc` resolution
- Gas estimates in hover, inlay hints, and code lens (#91, #94)
  - `GasIndex` built from solc contract output (creation + external/internal costs)
  - Hover shows gas cost for functions and deploy cost for contracts
  - Fire icon (🔥) with formatted numbers (e.g. `125,432`)
- Use solc directly for AST + diagnostics, 11x faster on large projects (#90)
- Use solc userdoc/devdoc for hover documentation (#99)
  - `DocIndex` built from solc contract output at cache time with pre-resolved `@inheritdoc`
  - Hover on parameters and return values shows `@param`/`@return` docs from parent function
  - Works at both declaration site and any usage inside the function body
  - Structured rendering: notice, `@dev` details, params table, returns table
  - Typed selectors: `FuncSelector`, `EventSelector`, `Selector` enum, `MethodId` newtype
  - Replaces raw `String` selectors throughout gas, hover, completion, and inlay hints

### Refactor

- Gas inlay hints use tree-sitter positions from the live buffer (#96)
  - Fixes hints drifting to wrong positions during editing
  - Function gas hints support opening/closing brace placement (`FN_GAS_HINT_POSITION` constant)
  - Contract deploy hints show `codeDepositCost` when `totalCost` is infinite
  - Libraries and interfaces now show deploy cost hints
- Remove code lens — gas info is covered by inlay hints and hover (#96)

### Fixes

- Improve natspec tag formatting in hover (#98)
  - `@dev` now renders with a bold `**@dev**` header above italic content
  - `@custom:` tags and other unknown `@` tags render with bold label and italic content
- Bound `foundry.toml` search at git repo root (#89)
- Hover works when file has compilation errors (#92)

### Tests

- 423 total tests, 0 warnings

## v0.1.20

### Features

- Tree-sitter enhanced goto definition (#66, #79)
- Inlay hints v2 — tree-sitter positions + AST semantics (#61, #81)
- Respect `foundry.toml` lint ignore and `lint_on_build` config (#84, #87)
- Introduce `NodeId`/`FileId` newtypes and shared `SourceLoc` parser (#86)

### Fixes

- Goto definition returns wrong result after unsaved edits (#83)
- Bound `foundry.toml` search at git repo root (#89)

## v0.1.19

### Refactor

- Rewrite `textDocument/documentSymbol` and `workspace/symbol` to use tree-sitter instead of Forge AST (#77, #78)
  - Symbols no longer depend on `forge build` — works on any Solidity file immediately
  - `documentSymbol` reads from text_cache with disk fallback
  - `workspace/symbol` scans open files only
- Clean up semantic tokens to avoid overriding tree-sitter highlights (#78)
  - Remove modifiers, builtin types, pragmas, variables, and member expressions from LSP tokens
  - Prevents `@lsp.typemod.*` priority 126-127 from overriding tree-sitter colors

### Performance

- `textDocument/documentSymbol` 3.2x faster (3.24ms → 1.02ms)
- `workspace/symbol` 6.4x faster (6.08ms → 0.95ms)

### Features

- Add `textDocument/semanticTokens/full` via tree-sitter (#75, #76)

### Notes

#### Symbol kinds

The `documentSymbol` response returns hierarchical symbols with the following kind mappings:

| Solidity construct         | LSP SymbolKind     |
|----------------------------|--------------------|
| `contract`                 | CLASS              |
| `interface`                | INTERFACE          |
| `library`                  | NAMESPACE          |
| `function`                 | FUNCTION           |
| `constructor`              | CONSTRUCTOR        |
| `fallback` / `receive`     | FUNCTION           |
| `state variable`           | FIELD              |
| `event`                    | EVENT              |
| `error`                    | EVENT              |
| `modifier`                 | METHOD             |
| `struct`                   | STRUCT             |
| `struct member`            | FIELD              |
| `enum`                     | ENUM               |
| `enum value`               | ENUM_MEMBER        |
| `using ... for`            | PROPERTY           |
| `type ... is ...`          | TYPE_PARAMETER     |
| `pragma`                   | STRING             |
| `import`                   | MODULE             |

Functions include a detail string with parameters and return types (e.g. `(address to, uint256 amount) returns (bool)`).

Contracts, structs, and enums are returned as parent symbols with their members nested as children. Top-level declarations (pragma, import, free functions, free structs/enums) appear at root level.

#### Semantic tokens and tree-sitter coexistence

LSP semantic tokens in Neovim have higher priority (125-127) than tree-sitter highlights (100). When both emit tokens for the same range, LSP wins. This causes problems when `@lsp.typemod.*` groups fall back to the generic `@lsp` highlight with undesirable colors.

The approach taken here: only emit semantic tokens where the LSP adds value that tree-sitter cannot provide. Let tree-sitter handle syntax it already highlights well (builtins, variables, member access, pragmas, modifiers). The LSP focuses on declaration identifiers, parameters, type references, and call targets where semantic knowledge matters.

## v0.1.18

### Features

- Context-sensitive `type(X).` completions (#70)
  - Typing `type(ContractName).` now shows `creationCode`, `runtimeCode`, `name`, `interfaceId`
  - Typing `type(IntegerType).` shows `min`, `max`

### Fixes

- Skip using-for completions on contract/library/interface names (#71, #72)
  - `Lock.` no longer returns Pool and SafeCast library functions from `using Pool for *` and `using SafeCast for *`
  - Using-for completions now only appear when completing on a value of a matching type, not on a contract/library/interface name
- Fix SIMD chunk selection skipping past target column in `position_to_byte_offset` (#73, #74)
  - The SIMD-accelerated position calculation introduced in #68 could pick a chunk boundary past the target column on long lines, returning the wrong byte offset
  - Go-to-definition on `AlreadyUnlocked` in PoolManager.sol resolved to `revertWith` in CustomRevert.sol instead of the correct `error AlreadyUnlocked()` in IPoolManager.sol

### Performance

- SIMD-accelerated position calculation via `lintspec-core` TextIndex (#68)
  - `position_to_byte_offset` and `byte_offset_to_position` now use a single SIMD pass over 128-byte chunks instead of a full linear scan
  - Short linear walk (at most 128 bytes) from the nearest chunk to the exact position

### Refactor

- Rewrite position conversion to use `lintspec-core` `compute_indices` and `TextIndex` (#64, #68)
- Simplify conversion functions, use builtin traits and constructors
- Improved identifier validation for Solidity keywords

## v0.1.17

### Fixes

- Simplify diagnostic path matching to fix silent drop (#63)

## v0.1.16

### Features

- Scope-aware completion with inheritance resolution (#57)
  - Completions are now filtered by the current scope (contract, function, block)
  - Inherited members from parent contracts are resolved and included
  - Replaces the previous `fast`/`full` completion mode split with a single unified engine

### Fixes

- Use relative path to filter out diagnostics (#55)
  - Build diagnostic filtering now correctly matches files using relative paths
  - Fixes cases where diagnostics from dependency files were incorrectly included

### Deprecations

- `--completion-mode` flag is deprecated (#59)
  - The `fast`/`full` split is no longer needed — scope-aware completions are always active

### New Contributors

- [@libkakashi](https://github.com/libkakashi) — chore: add Zed editor setup section in docs (#60)

### Other

- Refactor build module: simplify diagnostic filtering, extract path comparison helper
- Add Zed editor setup section in docs (#60)
- 272 total tests, 0 warnings

## v0.1.15

### Fixes

- AST cache now updates when build has warnings but no errors (#41)
  - The `build_succeeded` check used `diagnostics.is_empty()` which blocked cache updates for files with unused variables or other warnings
  - Changed to only block on `DiagnosticSeverity::ERROR`, so warnings pass through
- Cross-file rename reads from in-memory editor buffers instead of disk (#50)
  - `rename_symbol` accepts `text_buffers` parameter reflecting unsaved editor state
  - No more disk writes behind the editor's back
- Full `WorkspaceEdit` returned to client for all files (#50)
  - Previously split edits between client (current file) and server-side `fs::write` (other files)
  - Now the complete edit set is returned to the client
- `nameLocations` index fallback in references (#50)
  - Nodes without `nameLocations` array now correctly fall through to `nameLocation` or `src`
- Stale AST range correction during rename (#50)
  - `find_identifier_on_line` scans the current line to correct shifted column positions after unsaved edits
- All LSP handlers read source from `text_cache` instead of `std::fs::read` (#50)
- Respect `includeDeclaration` in `textDocument/references` (#49)
- Use cached AST for `workspace/symbol` instead of rebuilding (#46)
- Clear caches on `did_close` to free memory (#45)
- Encoding-aware UTF-16 position conversion (#39)
- Remove document version when publishing diagnostics (#40)
- Pass `--ignore-eip-3860` and `--ignored-error-codes 5574` to forge build (#11)

### Features

- Announce full version string in LSP `initialize` response (#51)
  - e.g. `0.1.15+commit.abc1234.macos.aarch64`

### Tests

- 10 new regression tests for bugs fixed in #41 and #50
  - `tests/build.rs`: warning-only builds succeed, error builds fail, empty diagnostics succeed
  - `tests/rename.rs`: nameLocations fallback, text_buffers usage, cross-file WorkspaceEdit, stale AST correction, identifier extraction
- Solidity fixture files in `example/` for rename tests (A.sol, B.sol, C.sol, Counter.sol)
- 186 total tests, 0 warnings

### New Contributors

- [@beeb](https://github.com/beeb) — fix: remove document version when publishing diagnostics (#40), filed issues #32, #33, #34, #35, #36, #37, #38, #41

### Other

- `rustfmt.toml` and `cargo fmt` across codebase (#42)
- Benchmark config updated with all implemented LSP methods

## v0.1.14

### Fixes

- `textDocument/definition` and `textDocument/declaration` now return proper range width instead of zero-width ranges (#30)
  - `goto_bytes()` returns `(file_path, byte_offset, length)` — extracts the length field from `nameLocation` or `src`
  - `goto_declaration()` computes `end` from `byte_offset + length`, so editors correctly highlight the target symbol
  - Previously `start == end` in the returned `Location`, making it impossible for the editor to highlight the target

### Tests

- 4 new goto range-length tests: `Hooks` (len 5), `Pool` (len 4), `SafeCast` (len 8), Yul external reference (nonzero)
- 168 total tests, 0 warnings

## v0.1.13

### Features

- `textDocument/documentLink` — every reference in a file is a clickable link
- Import paths link to the imported file (resolves `absolutePath` from AST)
- All nodes with `referencedDeclaration` link to their definition via `id_to_location`
- Uses pre-indexed `CachedBuild.nodes` — no extra AST traversal at request time

### Fixes

- `--version` now shows commit hash when installed from crates.io (reads `.cargo_vcs_info.json` as fallback)

### Tests

- 11 document link tests (CI-safe, real fixture data from `pool-manager-ast.json`)
- 164 total tests, 0 warnings

## v0.1.12

### Features

- Cross-file "Find All References" — scans all cached AST builds to find usages across files that don't share a build scope
- Cross-file "Rename" — renames symbols across all cached builds, not just the current file's dependency tree
- `CachedBuild` struct — pre-computes `cache_ids()` once per cache insert instead of N+1 times per request

### Performance

- `cache_ids()` no longer called at request time — all node indexing happens on file save
- `get_or_fetch_build()` deduplicates cache-miss logic across goto, references, rename, hover, and document symbol handlers

### Tests

- 12 new cross-file reference tests (CI-safe, hardcoded AST values from fixture)
- 153 total tests, 0 warnings

## v0.1.11

### Features

- `--version` / `-V` flag with full build metadata: version, commit hash, OS, architecture
- GPG-signed release checksums for binary verification
- `CONTRIBUTING.md` with project structure and development workflow

### Improvements

- Remove redundant timestamp from tracing log output
- Add `build.rs` to embed git commit hash at compile time
- Add `public-key.asc` for release signature verification
- Updated README with CLI usage examples, all flags, and verification instructions

## v0.1.10

### Features

- `textDocument/hover` — show Solidity signatures, NatSpec documentation, and selectors on hover
- Signature generation for functions, contracts, structs, enums, errors, events, modifiers, variables, UDVTs
- NatSpec formatting: `@notice`, `@param`, `@return`, `@dev` rendered as structured markdown
- Display `functionSelector`, `errorSelector`, `eventSelector` from AST in hover output
- `@inheritdoc` resolution via `functionSelector` matching between implementation and parent interface — correctly handles overloaded functions
- 25 hover tests against Uniswap v4 PoolManager AST

## v0.1.8

### Features

- Full completion engine with chain resolution, using-for directives, and type casts (~1400 lines, 86 tests)
- `--completion-mode` flag: `fast` (default) pre-built completions, `full` per-request scope filtering
- Dot-completion for structs, contracts, libraries, magic globals (`msg`, `block`, `tx`, `abi`, `type`)
- Chain completions through function return types, mappings, type casts
- `using-for` directive support (e.g. `PoolKey.toId()`, `BalanceDelta.amount0()`)
- Method identifier completions with 4-byte selectors and full signatures
- Keyword, global function, ether/time unit completions

### Performance

- Arc-based zero-copy AST cache — eliminates 7MB+ deep clones per handler request
- Non-blocking completion cache — returns static completions immediately while cache builds in background
- `document_symbol` uses `ast_cache` instead of shelling out to `forge ast` on every request
- Removed blocking `log_message` from completion handler to fix cancel+re-trigger lag

### Yul

- Yul `externalReferences` support for goto-definition and find-references

## v0.1.7

- Yul externalReferences support for goto-definition and find-references
- Completion engine with chain resolution, using-for, and type casts

## v0.1.6

- Fix rename in tests
- Fix: ignore bytecode size warnings for all sol files
- Enable goto definition for import statement strings
- Handle ImportDirective nodes in goto definition
- Add absolute_path field to NodeInfo struct

## v0.1.4

- Fix: only update AST cache when build succeeds
- Fix: preserve AST cache on file changes to keep go-to-definition working during errors
