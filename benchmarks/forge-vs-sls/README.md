# Solidity LSP Competition

Benchmarked against `example` — `Shop.sol`.

## Settings

| Setting | Value |
|---------|-------|
| File | `Shop.sol` |
| Position | line 130, col 26 |
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
| [initialize](#initialize) | 12.5ms ⚡ | 19.0ms |
| [textDocument/diagnostic](#textdocumentdiagnostic) | 0.2ms ⚡ | 0.2ms |
| [textDocument/definition](#textdocumentdefinition) | 6.4ms | 0.1ms ⚡ |
| [textDocument/declaration](#textdocumentdeclaration) | 1.3ms | 0.1ms ⚡ |
| [textDocument/hover](#textdocumenthover) | 1.0ms | 0.1ms ⚡ |
| [textDocument/references](#textdocumentreferences) | 2.5ms | 0.1ms ⚡ |
| [textDocument/completion](#textdocumentcompletion) | 13.7ms | 0.1ms ⚡ |
| [textDocument/signatureHelp](#textdocumentsignaturehelp) | 0.9ms | 0.1ms ⚡ |
| [textDocument/rename](#textdocumentrename) | 2.5ms ⚡ | cannot rename this symbol because worksp... |
| [textDocument/prepareRename](#textdocumentpreparerename) | 0.2ms ⚡ | cannot rename this symbol because worksp... |
| [textDocument/documentSymbol](#textdocumentdocumentsymbol) | 2.1ms | 0.3ms ⚡ |
| [textDocument/documentHighlight](#textdocumentdocumenthighlight) | 1.3ms | 0.1ms ⚡ |
| [textDocument/documentLink](#textdocumentdocumentlink) | 0.2ms | 0.1ms ⚡ |
| [textDocument/formatting](#textdocumentformatting) | 18.1ms ⚡ | 27.3ms |
| [textDocument/semanticTokens/full](#textdocumentsemantictokensfull) | 2.1ms ⚡ | No such method textDocument/semanticToke... |
| [workspace/symbol](#workspacesymbol) | 4.3ms | 0.9ms ⚡ |

### Scorecard

| Server | Wins | Out of |
|--------|------|--------|
| **forge** | **10** | **16** |
| mmsaki | 6 | 16 |

---

## Results

### initialize

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 12.5ms ⚡ | **10.9 MB** | ok |
| **forge** | 19.0ms | 25.1 MB | ok |

### textDocument/diagnostic

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **12.0 MB** | {"message":"lsp server initialized.","ty... |
| **forge** | 0.2ms | 25.3 MB | {"message":"solar initialized","type":3} |

### textDocument/definition

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.4ms | **20.4 MB** | `Shop.sol:41` |
| **forge** | 0.1ms ⚡ | 33.3 MB | `Shop.sol:41` |

### textDocument/declaration

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.3ms | **20.3 MB** | `Shop.sol:41` |
| **forge** | 0.1ms ⚡ | 33.2 MB | `Shop.sol:41` |

### textDocument/hover

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.0ms | **20.4 MB** | function addTax(uint256 amount, uint16 tax, uint16... |
| **forge** | 0.1ms ⚡ | 33.2 MB | function addTax(uint256 amount, uint16 tax, uint16... |

### textDocument/references

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.5ms | **20.4 MB** | 2 references |
| **forge** | 0.1ms ⚡ | 33.3 MB | 2 references |

### textDocument/completion

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 13.7ms | **20.3 MB** | 2 items (addTax, getRefund) |
| **forge** | 0.1ms ⚡ | 33.1 MB | 2 items (addTax, getRefund) |

### textDocument/signatureHelp

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.9ms | **20.3 MB** | function addTax(uint256 amount, uint16 tax, uint16... |
| **forge** | 0.1ms ⚡ | 33.4 MB | function addTax(uint16 tax, uint16 base) internal ... |

### textDocument/rename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.5ms ⚡ | **20.3 MB** | 2 edits in 1 files |
| **forge** | - | 33.5 MB | cannot rename this symbol because worksp... |

### textDocument/prepareRename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **20.5 MB** | ready (line 136) |
| **forge** | - | 33.3 MB | cannot rename this symbol because worksp... |

### textDocument/documentSymbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.1ms | **10.9 MB** | 4 symbols |
| **forge** | 0.3ms ⚡ | 25.2 MB | 103 symbols |

### textDocument/documentHighlight

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.3ms | **20.4 MB** | [{"kind":3,"range":{"end":{"character":1... |
| **forge** | 0.1ms ⚡ | 33.3 MB | [{"kind":3,"range":{"end":{"character":1... |

### textDocument/documentLink

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms | **20.4 MB** | 1 links |
| **forge** | 0.1ms ⚡ | 33.4 MB | 1 links |

### textDocument/formatting

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 18.1ms ⚡ | **10.6 MB** | 1 edits |
| **forge** | 27.3ms | 25.2 MB | 1 edits |

### textDocument/semanticTokens/full

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.1ms ⚡ | **11.8 MB** | 455 tokens |
| **forge** | - | 25.2 MB | No such method textDocument/semanticToke... |

### workspace/symbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 4.3ms | **20.5 MB** | 182 symbols |
| **forge** | 0.9ms ⚡ | 33.4 MB | 253 symbols |

---

*Benchmark run: 2026-11-03T22:18:14Z*
