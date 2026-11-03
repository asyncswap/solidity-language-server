# forge lsp vs solidity-language-server

Benchmarked against `example` — `Shop.sol` on 2026-11-03.

> **Caveat:** `example/sample/A.t.sol` imports `forge-std`, which is not installed in `example/`.
> Because `foundry.toml` sets `src = "."`, that one file breaks the whole solc build, so
> solidity-language-server has no AST and returns `null` for every AST-backed method.
> forge (solar) skips the bad import and keeps working. Rows marked ❌ for
> solidity-language-server reflect this, not normal behaviour.

## Settings

| Setting | Value |
|---------|-------|
| File | `Shop.sol` |
| Position | line 130, col 26 |
| Iterations | 10 (2 warmup) |
| Timeout | 10s |
| Index timeout | 30s |

## Servers

| Server | Version |
|--------|---------|
| mmsaki release | `0.1.37+commit.55a37d1` |
| [forge](https://github.com/foundry-rs/foundry) | `1.9.0` |

---

## Summary

p50 latency. ⚡ = faster *valid* response. ❌ = empty/null or error.

| Method | mmsaki release | forge |
|--------|----------------|-------|
| initialize | 14.8ms ⚡ | 19.8ms |
| textDocument/diagnostic ¹ | 0.2ms | 0.15ms |
| textDocument/definition | 6.9ms ❌ null | 0.11ms ⚡ |
| textDocument/declaration | 1.8ms ❌ null | 0.07ms ⚡ |
| textDocument/hover | 0.2ms ❌ null | 0.09ms ⚡ |
| textDocument/references | 2.6ms | 0.08ms ⚡ |
| textDocument/completion | 6.5ms ❌ empty | 0.08ms ⚡ |
| textDocument/signatureHelp | 0.23ms ❌ null | 0.07ms ⚡ |
| textDocument/rename | 0.18ms ❌ null | ❌ error ² |
| textDocument/prepareRename | 0.3ms ⚡ | ❌ error ² |
| textDocument/documentSymbol | 1.9ms | 0.3ms ⚡ |
| textDocument/documentHighlight | 1.65ms | 0.06ms ⚡ |
| textDocument/documentLink | 0.2ms ❌ null | 0.06ms ⚡ |
| textDocument/formatting | 27.5ms ⚡ | 41.1ms |
| textDocument/semanticTokens/full | 2.0ms ⚡ | ❌ unsupported |
| workspace/symbol | 0.14ms ❌ empty | 0.64ms ⚡ |

¹ Both servers' recorded response is a startup log message, not diagnostics — not comparable.
² `cannot rename this symbol because workspace indexing may omit source files`

### Scorecard

| Server | Wins | Out of |
|--------|------|--------|
| mmsaki release | 4 | 15 |
| **forge** | **10** | **15** |

Verify: 5/15 expectations failed (all mmsaki release: definition, declaration, completion, documentLink, workspace/symbol).

### Memory (peak RSS)

| Server | Min | Max |
|--------|-----|-----|
| mmsaki release | 10.9 MB | 20.8 MB |
| forge | 25.7 MB | 33.6 MB |

---

## Feature gaps observed

| | mmsaki release | forge |
|---|---|---|
| Survives one unresolved import | ❌ all AST features go dark | ✅ |
| Semantic tokens | ✅ | ❌ method not found |
| Rename / prepareRename | ✅ (when AST available) | ❌ refused |
| Formatting | ✅ (identical output) | ✅ |
