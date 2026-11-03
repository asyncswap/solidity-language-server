# Solidity LSP Competition

Benchmarked against `v4-core` — `src/libraries/Pool.sol`.

## Settings

| Setting | Value |
|---------|-------|
| File | `src/libraries/Pool.sol` |
| Position | line 102, col 15 |
| Iterations | 10 (2 warmup) |
| Timeout | 10s |

## Servers

| Server | Version |
|--------|---------|
| [mmsaki](https://github.com/asyncswap/solidity-language-server/releases/tag/v0.1.37) | `0.1.37` |
| [forge](https://github.com/foundry-rs/foundry) | `1.9.0` |

---

## Summary

| Method | mmsaki | forge |
|--------|--------|-------|
| [initialize](#initialize) | 21.0ms ⚡ | 24.2ms |
| [textDocument/diagnostic](#textdocumentdiagnostic) | 0.2ms ⚡ | 0.2ms |
| [textDocument/definition](#textdocumentdefinition) | 13.3ms | 0.1ms ⚡ |
| [textDocument/declaration](#textdocumentdeclaration) | 2.7ms | 0.1ms ⚡ |
| [textDocument/hover](#textdocumenthover) | 2.4ms | 0.1ms ⚡ |
| [textDocument/references](#textdocumentreferences) | 17.4ms | 0.4ms ⚡ |
| [textDocument/completion](#textdocumentcompletion) | 10.9ms | 0.2ms ⚡ |
| [textDocument/signatureHelp](#textdocumentsignaturehelp) | 2.1ms | 0.1ms ⚡ |
| [textDocument/rename](#textdocumentrename) | 9.0ms | 0.9ms ⚡ |
| [textDocument/prepareRename](#textdocumentpreparerename) | 0.3ms ⚡ | 0.7ms |
| [textDocument/documentSymbol](#textdocumentdocumentsymbol) | 11.2ms | 0.5ms ⚡ |
| [textDocument/documentHighlight](#textdocumentdocumenthighlight) | 2.7ms | 0.1ms ⚡ |
| [textDocument/documentLink](#textdocumentdocumentlink) | 0.5ms | 0.2ms ⚡ |
| [textDocument/formatting](#textdocumentformatting) | 30.7ms ⚡ | 42.1ms |
| [textDocument/semanticTokens/full](#textdocumentsemantictokensfull) | 8.6ms ⚡ | No such method textDocument/semanticToke... |
| [workspace/symbol](#workspacesymbol) | 71.8ms | 34.0ms ⚡ |

### Scorecard

| Server | Wins | Out of |
|--------|------|--------|
| **forge** | **11** | **16** |
| mmsaki | 5 | 16 |

---

## Results

### initialize

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 21.0ms ⚡ | **11.7 MB** | ok |
| **forge** | 24.2ms | 28.4 MB | ok |

### textDocument/diagnostic

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **13.1 MB** | {"message":"lsp server initialized.","ty... |
| **forge** | 0.2ms | 28.7 MB | {"message":"solar initialized","type":3} |

### textDocument/definition

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 13.3ms | **72.0 MB** | `TickMath.sol:9` |
| **forge** | 0.1ms ⚡ | 191.0 MB | `TickMath.sol:9` |

### textDocument/declaration

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.7ms | **72.0 MB** | `TickMath.sol:9` |
| **forge** | 0.1ms ⚡ | 195.3 MB | `TickMath.sol:9` |

### textDocument/hover

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.4ms | **71.4 MB** | function modifyLiquidity(struct Pool.State storage... |
| **forge** | 0.1ms ⚡ | 189.1 MB | function modifyLiquidity(State storage self, Modif... |

### textDocument/references

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 17.4ms | **71.6 MB** | 193 references |
| **forge** | 0.4ms ⚡ | 184.7 MB | 193 references |

### textDocument/completion

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 10.9ms | **75.0 MB** | 28 items (slot0, feeGrowthGlobal0X128, feeGrowthGlobal1X128) |
| **forge** | 0.2ms ⚡ | 198.0 MB | 18 items (checkPoolInitialized, clearTick, crossTick) |

### textDocument/signatureHelp

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.1ms | **71.3 MB** | function getTickAtSqrtPrice(uint160 sqrtPriceX96) ... |
| **forge** | 0.1ms ⚡ | 184.5 MB | function getTickAtSqrtPrice(uint160 sqrtPriceX96) ... |

### textDocument/rename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 9.0ms | **72.4 MB** | 13 edits in 1 files |
| **forge** | 0.9ms ⚡ | 194.3 MB | 13 edits in 1 files |

### textDocument/prepareRename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.3ms ⚡ | **71.5 MB** | ready (line 149) |
| **forge** | 0.7ms | 187.8 MB | ready (line 149) |

### textDocument/documentSymbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 11.2ms | **11.2 MB** | 16 symbols |
| **forge** | 0.5ms ⚡ | 26.1 MB | 159 symbols |

### textDocument/documentHighlight

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.7ms | **72.6 MB** | [{"kind":3,"range":{"end":{"character":2... |
| **forge** | 0.1ms ⚡ | 198.6 MB | [{"kind":3,"range":{"end":{"character":2... |

### textDocument/documentLink

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.5ms | **72.0 MB** | 14 links |
| **forge** | 0.2ms ⚡ | 195.6 MB | 14 links |

### textDocument/formatting

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 30.7ms ⚡ | **11.2 MB** | 1 edits |
| **forge** | 42.1ms | 28.5 MB | 1 edits |

### textDocument/semanticTokens/full

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 8.6ms ⚡ | **11.2 MB** | 697 tokens |
| **forge** | - | 27.9 MB | No such method textDocument/semanticToke... |

### workspace/symbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 71.8ms | **74.1 MB** | 1797 symbols |
| **forge** | 34.0ms ⚡ | 194.9 MB | 15657 symbols |

---

*Benchmark run: 2026-11-03T22:22:13Z*
