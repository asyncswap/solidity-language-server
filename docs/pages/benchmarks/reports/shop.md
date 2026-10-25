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
| [mmsaki](https://github.com/asyncswap/solidity-language-server/releases/tag/v0.1.30) | `0.1.30` |

---

## Summary

| Method | mmsaki |
|--------|--------|
| [initialize](#initialize) | 19.6ms ⚡ |
| [textDocument/diagnostic](#textdocumentdiagnostic) | 2.5ms ⚡ |
| [textDocument/semanticTokens/full/delta](#textdocumentsemantictokensfulldelta) | 2.2ms ⚡ |
| [textDocument/definition](#textdocumentdefinition) | 1.3ms ⚡ |
| [textDocument/declaration](#textdocumentdeclaration) | 0.3ms ⚡ |
| [textDocument/hover](#textdocumenthover) | 1.2ms ⚡ |
| [textDocument/references](#textdocumentreferences) | 0.7ms ⚡ |
| [textDocument/completion](#textdocumentcompletion) | 7.1ms ⚡ |
| [textDocument/signatureHelp](#textdocumentsignaturehelp) | 1.2ms ⚡ |
| [textDocument/rename](#textdocumentrename) | 2.3ms ⚡ |
| [textDocument/prepareRename](#textdocumentpreparerename) | 0.2ms ⚡ |
| [textDocument/documentSymbol](#textdocumentdocumentsymbol) | 1.9ms ⚡ |
| [textDocument/documentHighlight](#textdocumentdocumenthighlight) | 1.8ms ⚡ |
| [textDocument/documentLink](#textdocumentdocumentlink) | 0.2ms ⚡ |
| [textDocument/formatting](#textdocumentformatting) | 12.4ms ⚡ |
| [textDocument/foldingRange](#textdocumentfoldingrange) | 2.2ms ⚡ |
| [textDocument/selectionRange](#textdocumentselectionrange) | 2.1ms ⚡ |
| [textDocument/inlayHint](#textdocumentinlayhint) | 1.5ms ⚡ |
| [textDocument/semanticTokens/full](#textdocumentsemantictokensfull) | 2.1ms ⚡ |
| [textDocument/semanticTokens/range](#textdocumentsemantictokensrange) | 1.7ms ⚡ |
| [workspace/symbol](#workspacesymbol) | 1.4ms ⚡ |
| [workspace/willRenameFiles](#workspacewillrenamefiles) | 8.6ms ⚡ |
| [workspace/willCreateFiles](#workspacewillcreatefiles) | 0.1ms ⚡ |
| [workspace/willDeleteFiles](#workspacewilldeletefiles) | 7.6ms ⚡ |
| [workspace/executeCommand](#workspaceexecutecommand) | 0.3ms ⚡ |
| [textDocument/codeAction](#textdocumentcodeaction) | 1.1ms ⚡ |

### Scorecard

| Server | Wins | Out of |
|--------|------|--------|
| **mmsaki** | **26** | **26** |

---

## Results

### initialize

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 19.6ms ⚡ | **9.7 MB** | ok |

### textDocument/diagnostic

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.5ms ⚡ | **10.2 MB** | 0 diagnostics |

### textDocument/semanticTokens/full/delta

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.2ms ⚡ | **10.1 MB** | delta |

### textDocument/definition

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.3ms ⚡ | **15.3 MB** | `Shop.sol:41` |

### textDocument/declaration

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.3ms ⚡ | **15.2 MB** | `Shop.sol:41` |

### textDocument/hover

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.2ms ⚡ | **15.1 MB** | function addTax(uint256 amount, uint16 tax, uint16... |

### textDocument/references

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.7ms ⚡ | **15.2 MB** | 2 references |

### textDocument/completion

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 7.1ms ⚡ | **15.3 MB** | 2 items (addTax, getRefund) |

### textDocument/signatureHelp

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.2ms ⚡ | **15.2 MB** | function addTax(uint256 amount, uint16 tax, uint16... |

### textDocument/rename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.3ms ⚡ | **15.2 MB** | 2 edits in 1 files |

### textDocument/prepareRename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **15.2 MB** | ready (line 136) |

### textDocument/documentSymbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.9ms ⚡ | **10.1 MB** | 4 symbols |

### textDocument/documentHighlight

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.8ms ⚡ | **15.2 MB** | `[{"kind":3,"range":{"end":{"character":1...` |

### textDocument/documentLink

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **15.2 MB** | 1 links |

### textDocument/formatting

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 12.4ms ⚡ | **10.0 MB** | 1 edits |

### textDocument/foldingRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.2ms ⚡ | **10.1 MB** | `[{"endCharacter":1,"endLine":53,"startCh...` |

### textDocument/selectionRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.1ms ⚡ | **10.0 MB** | `[{"parent":{"parent":{"parent":{"parent"...` |

### textDocument/inlayHint

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.5ms ⚡ | **15.1 MB** | 22 hints (tax:, base:, buyer:) |

### textDocument/semanticTokens/full

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.1ms ⚡ | **10.0 MB** | 455 tokens |

### textDocument/semanticTokens/range

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.7ms ⚡ | **10.1 MB** | 160 tokens |

### workspace/symbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.4ms ⚡ | **15.2 MB** | 61 symbols |

### workspace/willRenameFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 8.6ms ⚡ | **15.2 MB** | null (valid) |

### workspace/willCreateFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.1ms ⚡ | **10.1 MB** | null (valid) |

### workspace/willDeleteFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 7.6ms ⚡ | **15.0 MB** | null (valid) |

### workspace/executeCommand

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.3ms ⚡ | **10.1 MB** | `{"success":true}` |

### textDocument/codeAction

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.1ms ⚡ | **15.1 MB** | `[{"diagnostics":[{"code":"unused-import"...` |

---

*Benchmark run: 2026-03-07T20:47:44Z*
