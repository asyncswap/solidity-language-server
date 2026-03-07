# Hover

## What this page covers

This page describes the current `textDocument/hover` implementation:

- how cursor position resolves to a declaration,
- how signature, selector, gas, and docs are assembled,
- how `@inheritdoc` and call-site `@param` docs are resolved,
- what is covered by tests today.

## Terms used in this page

- **`CachedBuild`**: build-time snapshot (from compiler output on disk) stored as hash maps for fast lookups.
- **`decl_index`**: `HashMap<node_id, DeclNode>` with typed declaration nodes used for O(1) declaration lookup.
- **`doc_index`**: `HashMap<DocKey, DocEntry>` built from `userdoc` + `devdoc` in compiler output.
- **`hint_index`**: prebuilt call-site index used to map a call argument back to `(decl_id, param_name)`.
- **selector**: 4-byte function/error selector or 32-byte event topic shown in hover when available.

## Why hover is built this way

Hover needs to work for multiple cases with one response:

- declaration hover (signature + docs),
- parameter hover (show matching `@param` text),
- argument-at-callsite hover (show docs for the parameter at that call position),
- Yul-linked declarations.

A single source is not enough for all of these. The implementation combines typed AST declarations (`decl_index`), doc metadata (`doc_index`), and call-site mapping (`hint_index`).

## Runtime flow

In `src/lsp.rs`, hover is a thin wrapper around:

- `hover::hover_info(&cached_build, &uri, position, &source_bytes)`

Inside `hover_info`, the flow is:

```mermaid
flowchart TD
  A["hover request"] --> B["resolve byte position"]
  B --> C["resolve node id (Yul external refs first, else AST span)"]
  C --> D["follow referencedDeclaration to declaration id"]
  D --> E["lookup typed declaration in decl_index"]
  E --> F["build markdown parts"]
  F --> G["signature / type fallback"]
  F --> H["selector"]
  F --> H2["NodeId (always)"]
  F --> I["optional gas block (sentinel gated)"]
  F --> J["docs from doc_index or NatSpec fallback"]
  F --> K["call-site @param doc via hint_index"]
  G --> Z["return Hover markdown"]
  H --> Z
  H2 --> Z
  I --> Z
  J --> Z
  K --> Z
```

## Cursor to declaration

Resolution order is:

- Try Yul bridge first using `externalReferences`.
- Fallback to AST span match using smallest containing node in the current file.
- Resolve final declaration id by following `referencedDeclaration` when present.

This ensures hover works for both high-level Solidity and inline assembly references.

## Signature and selector

For the resolved typed declaration (`DeclNode`):

- Signature is built with `build_signature()`.
- If no signature is available, hover falls back to `typeString + name`.
- Selector is shown when present:
  - function/public variable/error: 4-byte selector,
  - event: 32-byte topic hash.

## Documentation source priority

Documentation resolution fires exactly one of three branches, tried in order:

1. **`lookup_doc_entry_typed`** — look up the declaration in `doc_index` (compiler-derived `userdoc`/`devdoc`). Preferred because it contains structured docs and compiler-resolved inherited docs.
2. **`extract_doc_text` + NatSpec fallback** — if no `doc_index` entry, extract raw NatSpec from the declaration node. If the text contains `@inheritdoc`, attempt to resolve parent docs.
3. **`lookup_param_doc_typed`** — if neither of the above produces content, and the hover target is a parameter or return variable, look up the `@param`/`@return` description from the parent declaration's doc entry.

Only one of these three branches fires per hover response. The call-site `@param` detail (below) is additive and independent of this chain.

## `@inheritdoc` behavior

If raw doc text contains `@inheritdoc`, the implementation can resolve parent docs by:

- parsing parent name,
- reading the declaration selector,
- finding the parent in base contracts,
- matching by selector in the parent contract declarations.

When resolution succeeds, hover renders resolved content instead of raw `@inheritdoc` lines.

## Call-site parameter doc (argument hover)

Hover also adds parameter doc when the cursor is on an argument at a call site:

- Parse the live buffer and find enclosing call context + argument index.
- Resolve callsite semantics through the prebuilt call index.
- Resolve parameter identity `(decl_id, param_name)`.
- Fetch matching `@param` text from doc index first, then NatSpec fallback.

This allows hover to show meaningful parameter descriptions directly at usage sites.

## Output shape

Hover markdown is assembled from parts in this order:

- Signature block (or type fallback).
- Selector line when available.
- NodeId line for debugging (always shown).
- Documentation section from doc index or NatSpec fallback.
- Optional call-site `@param` detail when applicable.

Parts are separated with blank lines for readability.

## NodeId in hover

Hover always includes a `NodeId:` line showing the AST node ID for debugging purposes. Two formats:

- **Declaration hover:** ``NodeId: `2941` `` — shows the node's own ID.
- **Reference hover:** ``NodeId: `569` referencedDeclaration: `2941` `` — shows the cursor-hit node's ID and the declaration it resolves to.

This is useful for debugging cross-file resolution, verifying that PathInterner canonical IDs are correct, and understanding which AST node the server resolved from a cursor position.

## Test coverage and confidence

The hover module has broad direct tests in `src/hover.rs`:

- selector extraction (`function`, `error`, `event`, public variable, no-selector internal),
- `@inheritdoc` resolution and formatting behavior,
- `DocIndex` construction from real fixture (`poolmanager.json`),
- known selector lookup coverage (including overload cases),
- parameter doc extraction behavior.

This gives good confidence for:

- selector correctness,
- docs lookup priority,
- inherited-doc resolution,
- markdown formatting behavior.

### Recommended explicit additions

Useful end-to-end additions through LSP boundary:

- a request-level hover test verifying full assembled markdown ordering,
- a dedicated call-site argument hover test through `lsp.rs` (not just helper-level),
- a gas-sentinel hover test that checks both enabled and disabled paths through request flow.
