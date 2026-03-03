# Solidity LSP Competition

Benchmarked against `v4-core` — `test/PoolManager.t.sol`.

## Settings

| Setting | Value |
|---------|-------|
| File | `test/PoolManager.t.sol` |
| Position | line 116, col 51 |
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
| [initialize](#initialize) | 23.2ms ⚡ |
| [textDocument/diagnostic](#textdocumentdiagnostic) | 4.8s ⚡ |
| [textDocument/semanticTokens/full/delta](#textdocumentsemantictokensfulldelta) | 9.7ms ⚡ |
| [textDocument/definition](#textdocumentdefinition) | 6.3ms ⚡ |
| [textDocument/declaration](#textdocumentdeclaration) | 0.7ms ⚡ |
| [textDocument/hover](#textdocumenthover) | 6.3ms ⚡ |
| [textDocument/references](#textdocumentreferences) | 8.6ms ⚡ |
| [textDocument/completion](#textdocumentcompletion) | 52.9ms ⚡ |
| [textDocument/signatureHelp](#textdocumentsignaturehelp) | 5.8ms ⚡ |
| [textDocument/rename](#textdocumentrename) | 12.5ms ⚡ |
| [textDocument/prepareRename](#textdocumentpreparerename) | 0.2ms ⚡ |
| [textDocument/documentSymbol](#textdocumentdocumentsymbol) | 6.2ms ⚡ |
| [textDocument/documentHighlight](#textdocumentdocumenthighlight) | 7.2ms ⚡ |
| [textDocument/documentLink](#textdocumentdocumentlink) | 1.7ms ⚡ |
| [textDocument/formatting](#textdocumentformatting) | 18.8ms ⚡ |
| [textDocument/foldingRange](#textdocumentfoldingrange) | 7.3ms ⚡ |
| [textDocument/selectionRange](#textdocumentselectionrange) | 5.8ms ⚡ |
| [textDocument/inlayHint](#textdocumentinlayhint) | 9.3ms ⚡ |
| [textDocument/semanticTokens/full](#textdocumentsemantictokensfull) | 9.9ms ⚡ |
| [textDocument/semanticTokens/range](#textdocumentsemantictokensrange) | 6.6ms ⚡ |
| [workspace/symbol](#workspacesymbol) | 6.0ms ⚡ |
| [workspace/willRenameFiles](#workspacewillrenamefiles) | 242.3ms ⚡ |
| [workspace/willCreateFiles](#workspacewillcreatefiles) | 0.3ms ⚡ |
| [workspace/willDeleteFiles](#workspacewilldeletefiles) | 225.7ms ⚡ |
| [workspace/executeCommand](#workspaceexecutecommand) | 0.1ms ⚡ |
| [textDocument/codeAction](#textdocumentcodeaction) | 27.7ms ⚡ |

### Scorecard

| Server | Wins | Out of |
|--------|------|--------|
| **mmsaki** | **26** | **26** |

---

## Results

### initialize

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 23.2ms ⚡ | **9.9 MB** | ok |

### textDocument/diagnostic

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 4.8s ⚡ | **265.0 MB** | 15 diagnostics |

### textDocument/semanticTokens/full/delta

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 9.7ms ⚡ | **263.9 MB** | delta |

### textDocument/definition

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.3ms ⚡ | **262.5 MB** | `TickMath.sol:9` |

### textDocument/declaration

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.7ms ⚡ | **263.1 MB** | `TickMath.sol:9` |

### textDocument/hover

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.3ms ⚡ | **262.3 MB** | error PoolNotInitialized() |

### textDocument/references

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 8.6ms ⚡ | **263.0 MB** | 67 references |

### textDocument/completion

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 52.9ms ⚡ | **262.8 MB** | 23 items (amount0, amount1, checkTicks) |

### textDocument/signatureHelp

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.8ms ⚡ | **263.3 MB** | function bound(uint256 x, uint256 min, uint256 max... |

### textDocument/rename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 12.5ms ⚡ | **263.4 MB** | 9 edits in 1 files |

### textDocument/prepareRename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **263.8 MB** | ready (line 116) |

### textDocument/documentSymbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.2ms ⚡ | **263.6 MB** | 35 symbols |

### textDocument/documentHighlight

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 7.2ms ⚡ | **263.6 MB** | 9 highlights |

### textDocument/documentLink

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.7ms ⚡ | **262.4 MB** | 33 links |

### textDocument/formatting

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 18.8ms ⚡ | **263.4 MB** | 1 edits |

### textDocument/foldingRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 7.3ms ⚡ | **262.3 MB** | folding ranges |

### textDocument/selectionRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.8ms ⚡ | **263.6 MB** | selection ranges |

### textDocument/inlayHint

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 9.3ms ⚡ | **264.5 MB** | 1082 hints (name:, hooks:, _manager:) |

### textDocument/semanticTokens/full

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 9.9ms ⚡ | **263.2 MB** | 1512 tokens |

### textDocument/semanticTokens/range

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.6ms ⚡ | **262.4 MB** | 417 tokens |

### workspace/symbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.0ms ⚡ | **262.8 MB** | 90 symbols |

### workspace/willRenameFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 242.3ms ⚡ | **263.0 MB** | 12 edits in 12 files |

### workspace/willCreateFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.3ms ⚡ | **263.6 MB** | null (valid) |

### workspace/willDeleteFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 225.7ms ⚡ | **262.2 MB** | import updates |

### workspace/executeCommand

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.1ms ⚡ | **263.7 MB** | {"success":true} |

### textDocument/codeAction

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 27.7ms ⚡ | **263.1 MB** | Remove unused import |

---

*Benchmark run: 2026-03-03T21:08:25Z*
