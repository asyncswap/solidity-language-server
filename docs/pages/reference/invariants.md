# Invariant Checks

Rules that must hold for every response, whatever the input. They are checked in two places: a randomized test suite, and a runtime checker built into the server.

## Runtime checker

```sh
solidity-language-server --stdio --check-invariants
```

With the flag, `InvariantService` (`src/invariants.rs`) wraps the server and validates each response. A violation is printed to stderr as `[invariant] <method> ...` and the response becomes a JSON-RPC internal error, so the client or `lsp-bench` sees the failure. Without the flag the wrapper is not installed and costs nothing.

The checker tracks open buffers from `didOpen`, `didChange` and `didClose`, so it knows what the client sees without reaching into the server.

| Method | Rules |
|---|---|
| `documentSymbol` | Ranges are inside the document; `selectionRange` is inside `range`, including children |
| `selectionRange` | One result per position; each parent contains its child; ranges are inside the document |
| `foldingRange` | Start is not after end; end is inside the document |
| `semanticTokens/full`, `/range` | Data is a multiple of 5 integers; types are in the legend; tokens do not overlap and stay on their line |
| `semanticTokens/full/delta` | Edit `start` and `deleteCount` align to whole tokens |
| `documentHighlight`, `documentLink` | Ranges are inside the document |
| `formatting`, `rename`, `willRenameFiles`, `willCreateFiles`, `willDeleteFiles` | Edits to one file do not overlap |
| `definition`, `declaration`, `implementation`, `references` | Ranges are not inverted; a `LocationLink` selection is inside its target |
| `prepareCallHierarchy`, `incomingCalls`, `outgoingCalls` | `selectionRange` is inside `range`; `fromRanges` are not inverted |

### Known limits

- It checks structure, not whether the answer is right. A well-formed reference to the wrong function passes.
- Locations from solc's AST are not checked against document bounds, because they can legitimately describe text older than the buffer.
- Responses are not checked if the document changed while the handler ran. A violation is also dismissed if the response fits the text before the latest edit, since the handler may have started before that edit reached its cache.
- Documents with a bare `\r` skip bounds checks. See [Line endings](#line-endings).

## Randomized tests

`tests/invariants.rs` uses `proptest` to generate adversarial text and damaged copies of `example/*.sol`:

- Position conversion matches an independent line model and round trips, in UTF-8 and UTF-16.
- Any position lands on a character boundary inside its line.
- `apply_text_edits` matches a splice model.
- Semantic tokens pass the checker in both encodings, a delta applied to the old tokens gives the new ones, and range tokens match full tokens on those lines.
- Tree-sitter handlers never panic on broken source and always pass the checker.

Failing inputs shrink to a minimal case and are saved to `tests/invariants.regressions`. Commit that file with the fix.

`tests/check_invariants_server.rs` runs the real binary with the flag over multibyte CRLF source in both encodings.

## Position encoding

The negotiated encoding is stored once per process, so tests can't switch it. Functions that need testing in both modes take it as a parameter: `byte_offset_to_position_with`, `position_to_byte_offset_with` and `semantic_tokens_full_with`.

## Line endings

`byte_offset_to_position` and `position_to_byte_offset` follow the LSP specification: `\n`, `\r\n` and `\r` end a line, and U+2028 and U+2029 do not. A column past the end of a line clamps to before its line ending.

Tree-sitter only breaks lines on `\n`, so features built on tree-sitter rows report different lines in a file with a bare `\r`.
