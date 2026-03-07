# Completions

## What this page covers

This page explains the current completion implementation:

- how the completion request is routed in `lsp.rs`,
- what data structures power completion in `completion.rs`,
- how dot-chain/type/member resolution works,
- how auto-import tail candidates are appended,
- what is covered by tests today.

## Terms used in this page

- **`CompletionCache`**: a set of hash maps/vectors built from compiler output and reused per request.
- **local cache**: completion cache keyed by current URI (`completion_cache[uri]`).
- **root cache**: project-level cache from root build (`ast_cache[root_uri].completion_cache`), used for global tail candidates.
- **general completions**: non-dot completion items (keywords, globals, named symbols, units).
- **dot completions**: member completions for expressions ending with `.`.
- **tail candidates**: top-level importable symbols appended at the end of non-dot completion results (with optional import edits).

## Request flow in `lsp.rs`

At `textDocument/completion`:

- Read source text from `text_cache` (disk fallback when needed).
- Load URI-local cache and root cache (if present).
- Use local cache first, root cache as fallback.
- Resolve `file_id` from `path_to_file_id` for scope-aware lookup.
- For non-dot requests, build tail candidates from top-level importables.
- Produce response via the completion handler and return.

Important behavior:

- If trigger character is `.`, tail candidates are disabled.
- If no cache is available, the server asynchronously hydrates from `CachedBuild` when possible.

## The working data model

`CompletionCache` is intentionally map-heavy so request-time work is mostly lookups:

- `name_to_type`: `SymbolName` -> `TypeIdentifier`
- `type_to_node`: `TypeIdentifier` -> declaration node id
- `node_members`: declaration node id -> member completion items
- `method_identifiers`: contract node id -> method signature completions (+ selector label details)
- `function_return_types`: `(contract_node_id, fn_name)` -> return `typeIdentifier` (for `foo().`)
- `using_for`: `TypeIdentifier` -> extension methods
- `using_for_wildcard`: methods from `using X for *`
- `scope_declarations`, `scope_parent`, `scope_ranges`: scope-aware lookup context
- `linearized_base_contracts`: inheritance traversal for scope resolution
- `top_level_importables_by_name`, `top_level_importables_by_file`: import-on-completion support (`SymbolName` / `RelPath` keyed)

This is built in `build_completion_cache(...)` from `.sources` AST and optional `.contracts` method identifiers.

## Completion behavior by mode

### Non-dot mode

Non-dot requests return prebuilt general completions and then append tail candidates (if any).

General completions include:

- named AST symbols,
- Solidity keywords,
- magic globals,
- units and type helpers.

Tail candidates are appended last so local/scope-aware symbols stay prioritized.

### Dot mode

Dot requests parse the expression chain before the cursor (`parse_dot_chain`) and resolve segment-by-segment (`get_chain_completions`):

- **Plain segment**: resolve symbol to type.
- **Call segment**: use `function_return_types` to continue on return type.
- **Index segment**: peel mapping/array value type and continue.

Final member set is composed from:

- `node_members`,
- `method_identifiers`,
- `using_for` matches,
- `using_for_wildcard`.

## Scope-aware name resolution

When resolving a symbol in context, completion walks:

- Start at the innermost scope from `(file_id, byte position)`.
- Walk outward through `scope_parent`.
- Include inherited contracts via `linearized_base_contracts`.
- Fall back to global maps (`name_to_type`, `name_to_node_id`) when needed.

This is why completion can prioritize locals/params while still finding inherited members.

## Auto-import tail candidates

For non-dot mode, root cache can provide importable top-level symbols not declared in the current file.

Each tail candidate can carry `additionalTextEdits` to insert an import when selected.  
Candidate extraction intentionally only includes directly declared top-level importables (contracts, structs, enums, UDVTs, free functions, constants) and excludes re-export aliases.

## Why method identifiers are separate

AST member lists are useful but do not always carry full external signature detail in completion shape.  
`method_identifiers` from `.contracts` adds:

- canonical signature text,
- selector metadata in label details,
- better external/public method display for contract and interface completions.

## Import path completions

When the cursor is inside an import string literal (e.g. `import "|"`), the server provides file-path completions instead of symbol completions.

### How it works

At request time, `textDocument/completion` detects whether the cursor is inside a string literal inside an `import_directive` node using tree-sitter. If so, it switches to import-path mode:

- The partial path typed so far is extracted from the string literal.
- The server resolves the project root and remapping table from `foundry.toml`.
- Candidate paths are collected from the filesystem relative to the current file (for relative imports) and from configured remapping prefixes (e.g. `@openzeppelin/` → `lib/openzeppelin/src/`).
- Each candidate is returned as a `CompletionItem` with `kind: Module` and a label that represents the relative or remapped import path.

This mode is entirely separate from symbol completions: `CompletionCache` is not consulted, and tail candidates are not appended.

### Important boundary

Import path completions only apply inside import string literals. They do not apply to:

- identifier expressions (use regular symbol completions),
- string literals in other contexts (no completions are offered).

## Current limitations / tradeoffs

- Completion quality depends on cache freshness; background cache hydration is best-effort.
- Dot-chain resolution for very complex expressions is intentionally heuristic, not a full type-check pass.
- Tail candidate import edits are only added in non-dot mode by design.
- Import path completions are filesystem-based; they do not filter to files that actually export the symbol you need.

## Test coverage and confidence

`tests/completion.rs` is extensive and covers:

- scope declarations and scope parent behavior,
- AST extraction of declarations by kind,
- inheritance-aware scope resolution,
- type parsing helpers (`extract_node_id_from_type`, mapping value extraction),
- dot-chain parsing and chain resolution behavior,
- using-for and wildcard method inclusion,
- top-level importable extraction and tail-candidate behavior.

This gives strong confidence in cache construction and request-time resolution logic.

### Recommended explicit additions

High-value direct additions:

- request-level test in `lsp.rs` validating local-cache-first, root-fallback behavior,
- request-level test asserting tail candidates are suppressed on `.` trigger,
- end-to-end test validating `additionalTextEdits` import insertion behavior through the completion response.
