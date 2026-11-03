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
| [initialize](#initialize) | 14.9ms ⚡ | 20.0ms |
| [textDocument/diagnostic](#textdocumentdiagnostic) | 0.1ms ⚡ | 0.2ms |
| [textDocument/definition](#textdocumentdefinition) | 6.5ms | 0.1ms ⚡ |
| [textDocument/declaration](#textdocumentdeclaration) | 1.2ms | 0.1ms ⚡ |
| [textDocument/hover](#textdocumenthover) | 1.1ms | 0.1ms ⚡ |
| [textDocument/references](#textdocumentreferences) | 2.5ms | 0.1ms ⚡ |
| [textDocument/completion](#textdocumentcompletion) | 5.6ms | 0.1ms ⚡ |
| [textDocument/signatureHelp](#textdocumentsignaturehelp) | 0.9ms | 0.1ms ⚡ |
| [textDocument/rename](#textdocumentrename) | 2.4ms ⚡ | cannot rename this symbol because worksp... |
| [textDocument/prepareRename](#textdocumentpreparerename) | 0.2ms ⚡ | cannot rename this symbol because worksp... |
| [textDocument/documentSymbol](#textdocumentdocumentsymbol) | 1.9ms | 0.3ms ⚡ |
| [textDocument/documentHighlight](#textdocumentdocumenthighlight) | 1.4ms | 0.1ms ⚡ |
| [textDocument/documentLink](#textdocumentdocumentlink) | 0.1ms | 0.1ms ⚡ |
| [textDocument/formatting](#textdocumentformatting) | 16.1ms ⚡ | 26.4ms |
| [textDocument/semanticTokens/full](#textdocumentsemantictokensfull) | 2.2ms ⚡ | No such method textDocument/semanticToke... |
| [workspace/symbol](#workspacesymbol) | 0.2ms | 1.6ms ⚡ |

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
| **mmsaki** | 14.9ms ⚡ | **11.5 MB** | ok |
| **forge** | 20.0ms | 25.2 MB | ok |

### textDocument/diagnostic

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.1ms ⚡ | **12.5 MB** | {"message":"lsp server initialized.","ty... |
| **forge** | 0.2ms | 25.3 MB | {"message":"solar initialized","type":3} |

### textDocument/definition

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.5ms | **20.1 MB** | `Shop.sol:41` |
| **forge** | 0.1ms ⚡ | 33.2 MB | `Shop.sol:41` |

### textDocument/declaration

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.2ms | **20.2 MB** | `Shop.sol:41` |
| **forge** | 0.1ms ⚡ | 33.5 MB | `Shop.sol:41` |

### textDocument/hover

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.1ms | **20.3 MB** | function addTax(uint256 amount, uint16 tax, uint16... |
| **forge** | 0.1ms ⚡ | 33.4 MB | function addTax(uint256 amount, uint16 tax, uint16... |

### textDocument/references

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.5ms | **20.3 MB** | 2 references |
| **forge** | 0.1ms ⚡ | 33.1 MB | 2 references |

### textDocument/completion

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.6ms | **20.2 MB** | 0 items |
| **forge** | 0.1ms ⚡ | 33.2 MB | 2 items (addTax, getRefund) |

### textDocument/signatureHelp

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.9ms | **20.2 MB** | function addTax(uint256 amount, uint16 tax, uint16... |
| **forge** | 0.1ms ⚡ | 33.4 MB | function addTax(uint16 tax, uint16 base) internal ... |

### textDocument/rename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.4ms ⚡ | **20.3 MB** | 2 edits in 1 files |
| **forge** | - | 33.5 MB | cannot rename this symbol because worksp... |

### textDocument/prepareRename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **20.2 MB** | ready (line 136) |
| **forge** | - | 33.4 MB | cannot rename this symbol because worksp... |

### textDocument/documentSymbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.9ms | **10.8 MB** | 4 symbols |
| **forge** | 0.3ms ⚡ | 25.3 MB | 103 symbols |

### textDocument/documentHighlight

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.4ms | **20.3 MB** | [{"kind":3,"range":{"end":{"character":1... |
| **forge** | 0.1ms ⚡ | 33.5 MB | [{"kind":3,"range":{"end":{"character":1... |

### textDocument/documentLink

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.1ms | **20.2 MB** | 1 links |
| **forge** | 0.1ms ⚡ | 33.3 MB | 1 links |

### textDocument/formatting

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 16.1ms ⚡ | **10.8 MB** | 1 edits |
| **forge** | 26.4ms | 25.2 MB | 1 edits |

### textDocument/semanticTokens/full

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.2ms ⚡ | **10.7 MB** | 455 tokens |
| **forge** | - | 25.1 MB | No such method textDocument/semanticToke... |

### workspace/symbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms | 20.3 MB | 0 symbols |
| **forge** | 1.6ms ⚡ | **33.2 MB** | 253 symbols |

---

*Benchmark run: 2026-11-03T22:09:07Z*
