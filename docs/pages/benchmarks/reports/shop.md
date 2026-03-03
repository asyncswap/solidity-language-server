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
| [mmsaki](https://github.com/mmsaki/solidity-language-server/releases/tag/v0.1.28) | `0.1.28` |

---

## Summary

| Method | mmsaki |
|--------|--------|
| [initialize](#initialize) | 33.6ms ⚡ |
| [textDocument/diagnostic](#textdocumentdiagnostic) | 59.9ms ⚡ |
| [textDocument/semanticTokens/full/delta](#textdocumentsemantictokensfulldelta) | 1.6ms ⚡ |
| [textDocument/definition](#textdocumentdefinition) | 1.1ms ⚡ |
| [textDocument/declaration](#textdocumentdeclaration) | 0.2ms ⚡ |
| [textDocument/hover](#textdocumenthover) | 1.0ms ⚡ |
| [textDocument/references](#textdocumentreferences) | 1.1ms ⚡ |
| [textDocument/completion](#textdocumentcompletion) | 5.5ms ⚡ |
| [textDocument/signatureHelp](#textdocumentsignaturehelp) | 1.0ms ⚡ |
| [textDocument/rename](#textdocumentrename) | 2.0ms ⚡ |
| [textDocument/prepareRename](#textdocumentpreparerename) | 0.2ms ⚡ |
| [textDocument/documentSymbol](#textdocumentdocumentsymbol) | 1.3ms ⚡ |
| [textDocument/documentHighlight](#textdocumentdocumenthighlight) | 1.3ms ⚡ |
| [textDocument/documentLink](#textdocumentdocumentlink) | 0.2ms ⚡ |
| [textDocument/formatting](#textdocumentformatting) | 11.6ms ⚡ |
| [textDocument/foldingRange](#textdocumentfoldingrange) | 1.2ms ⚡ |
| [textDocument/selectionRange](#textdocumentselectionrange) | 1.0ms ⚡ |
| [textDocument/inlayHint](#textdocumentinlayhint) | 1.5ms ⚡ |
| [textDocument/semanticTokens/full](#textdocumentsemantictokensfull) | 1.6ms ⚡ |
| [textDocument/semanticTokens/range](#textdocumentsemantictokensrange) | 1.0ms ⚡ |
| [workspace/symbol](#workspacesymbol) | 1.1ms ⚡ |
| [workspace/willRenameFiles](#workspacewillrenamefiles) | 5.7ms ⚡ |
| [workspace/willCreateFiles](#workspacewillcreatefiles) | 0.1ms ⚡ |
| [workspace/willDeleteFiles](#workspacewilldeletefiles) | 6.3ms ⚡ |
| [workspace/executeCommand](#workspaceexecutecommand) | 0.1ms ⚡ |
| [textDocument/codeAction](#textdocumentcodeaction) | 0.9ms ⚡ |

### Scorecard

| Server | Wins | Out of |
|--------|------|--------|
| **mmsaki** | **26** | **26** |

---

## Results

### initialize

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 33.6ms ⚡ | **9.7 MB** | ok |

### textDocument/diagnostic

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 59.9ms ⚡ | **15.4 MB** | 2 diagnostics |

### textDocument/semanticTokens/full/delta

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.6ms ⚡ | **15.4 MB** | delta |

### textDocument/definition

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.1ms ⚡ | **15.3 MB** | `Shop.sol:69` |

### textDocument/declaration

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **15.4 MB** | `Shop.sol:69` |

### textDocument/hover

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.0ms ⚡ | **15.4 MB** | address payable public owner |

### textDocument/references

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.1ms ⚡ | **15.3 MB** | 11 references |

### textDocument/completion

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.5ms ⚡ | **15.5 MB** | 3 items (Order, addTax, getRefund) |

### textDocument/signatureHelp

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.0ms ⚡ | **15.4 MB** | function addTax(uint256 amount, uint16 tax, uint16... |

### textDocument/rename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.0ms ⚡ | **15.4 MB** | 11 edits in 1 files |

### textDocument/prepareRename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **15.4 MB** | ready (line 130) |

### textDocument/documentSymbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.3ms ⚡ | **15.4 MB** | 4 symbols |

### textDocument/documentHighlight

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.3ms ⚡ | **15.3 MB** | 11 highlights |

### textDocument/documentLink

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **15.4 MB** | 1 links |

### textDocument/formatting

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 11.6ms ⚡ | **15.3 MB** | 1 edits |

### textDocument/foldingRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.2ms ⚡ | **15.4 MB** | folding ranges |

### textDocument/selectionRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.0ms ⚡ | **15.4 MB** | selection ranges |

### textDocument/inlayHint

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.5ms ⚡ | **15.2 MB** | 24 hints (tax:, base:, buyer:) |

### textDocument/semanticTokens/full

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.6ms ⚡ | **15.5 MB** | 455 tokens |

### textDocument/semanticTokens/range

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.0ms ⚡ | **15.3 MB** | 160 tokens |

### workspace/symbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.1ms ⚡ | **15.5 MB** | 61 symbols |

### workspace/willRenameFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.7ms ⚡ | **15.4 MB** | null (valid) |

### workspace/willCreateFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.1ms ⚡ | **15.2 MB** | null (valid) |

### workspace/willDeleteFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.3ms ⚡ | **15.2 MB** | null (valid) |

### workspace/executeCommand

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.1ms ⚡ | **15.2 MB** | {"success":true} |

### textDocument/codeAction

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.9ms ⚡ | **15.4 MB** | Remove unused import |

---

*Benchmark run: 2026-03-03T21:02:10Z*
