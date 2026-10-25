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
| [mmsaki](https://github.com/asyncswap/solidity-language-server/releases/tag/v0.1.30) | `0.1.30` |

---

## Summary

| Method | mmsaki |
|--------|--------|
| [initialize](#initialize) | 22.4ms ⚡ |
| [textDocument/diagnostic](#textdocumentdiagnostic) | 2.8ms ⚡ |
| [textDocument/semanticTokens/full/delta](#textdocumentsemantictokensfulldelta) | 11.1ms ⚡ |
| [textDocument/definition](#textdocumentdefinition) | 8.3ms ⚡ |
| [textDocument/declaration](#textdocumentdeclaration) | 1.6ms ⚡ |
| [textDocument/hover](#textdocumenthover) | 7.3ms ⚡ |
| [textDocument/references](#textdocumentreferences) | 5.3ms ⚡ |
| [textDocument/completion](#textdocumentcompletion) | 59.9ms ⚡ |
| [textDocument/signatureHelp](#textdocumentsignaturehelp) | 6.2ms ⚡ |
| [textDocument/rename](#textdocumentrename) | 15.1ms ⚡ |
| [textDocument/prepareRename](#textdocumentpreparerename) | 0.2ms ⚡ |
| [textDocument/documentSymbol](#textdocumentdocumentsymbol) | 7.7ms ⚡ |
| [textDocument/documentHighlight](#textdocumentdocumenthighlight) | 8.4ms ⚡ |
| [textDocument/documentLink](#textdocumentdocumentlink) | 2.6ms ⚡ |
| [textDocument/formatting](#textdocumentformatting) | 43.4ms ⚡ |
| [textDocument/foldingRange](#textdocumentfoldingrange) | 10.4ms ⚡ |
| [textDocument/selectionRange](#textdocumentselectionrange) | 6.6ms ⚡ |
| [textDocument/inlayHint](#textdocumentinlayhint) | 11.4ms ⚡ |
| [textDocument/semanticTokens/full](#textdocumentsemantictokensfull) | 18.5ms ⚡ |
| [textDocument/semanticTokens/range](#textdocumentsemantictokensrange) | 8.9ms ⚡ |
| [workspace/symbol](#workspacesymbol) | 7.2ms ⚡ |
| [workspace/willRenameFiles](#workspacewillrenamefiles) | 240.6ms ⚡ |
| [workspace/willCreateFiles](#workspacewillcreatefiles) | 1.6ms ⚡ |
| [workspace/willDeleteFiles](#workspacewilldeletefiles) | 230.5ms ⚡ |
| [workspace/executeCommand](#workspaceexecutecommand) | 5.3ms ⚡ |
| [textDocument/codeAction](#textdocumentcodeaction) | 28.7ms ⚡ |

### Scorecard

| Server | Wins | Out of |
|--------|------|--------|
| **mmsaki** | **26** | **26** |

---

## Results

### initialize

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 22.4ms ⚡ | **9.8 MB** | ok |

### textDocument/diagnostic

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.8ms ⚡ | **10.5 MB** | 0 diagnostics |

### textDocument/semanticTokens/full/delta

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 11.1ms ⚡ | **10.3 MB** | delta |

### textDocument/definition

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 8.3ms ⚡ | **262.0 MB** | `TickMath.sol:9` |

### textDocument/declaration

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.6ms ⚡ | **262.9 MB** | `TickMath.sol:9` |

### textDocument/hover

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 7.3ms ⚡ | **261.7 MB** | error PoolNotInitialized() |

### textDocument/references

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.3ms ⚡ | **261.2 MB** | 7 references |

### textDocument/completion

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 59.9ms ⚡ | **261.0 MB** | 23 items (amount0, amount1, checkTicks) |

### textDocument/signatureHelp

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.2ms ⚡ | **260.7 MB** | function bound(uint256 x, uint256 min, uint256 max... |

### textDocument/rename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 15.1ms ⚡ | **260.5 MB** | 9 edits in 1 files |

### textDocument/prepareRename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **262.2 MB** | ready (line 116) |

### textDocument/documentSymbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 7.7ms ⚡ | **10.3 MB** | 35 symbols |

### textDocument/documentHighlight

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 8.4ms ⚡ | **260.0 MB** | `[{"kind":2,"range":{"end":{"character":1...` |

### textDocument/documentLink

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.6ms ⚡ | **261.7 MB** | 33 links |

### textDocument/formatting

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 43.4ms ⚡ | **10.3 MB** | 1 edits |

### textDocument/foldingRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 10.4ms ⚡ | **10.5 MB** | `[{"endCharacter":1,"endLine":1261,"start...` |

### textDocument/selectionRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.6ms ⚡ | **10.3 MB** | `[{"parent":{"parent":{"parent":{"parent"...` |

### textDocument/inlayHint

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 11.4ms ⚡ | **259.9 MB** | 1082 hints (name:, hooks:, _manager:) |

### textDocument/semanticTokens/full

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 18.5ms ⚡ | **10.4 MB** | 1512 tokens |

### textDocument/semanticTokens/range

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 8.9ms ⚡ | **10.3 MB** | 417 tokens |

### workspace/symbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 7.2ms ⚡ | **261.4 MB** | 90 symbols |

### workspace/willRenameFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 240.6ms ⚡ | **260.9 MB** | 12 edits in 12 files |

### workspace/willCreateFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.6ms ⚡ | **262.2 MB** | null (valid) |

### workspace/willDeleteFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 230.5ms ⚡ | **260.4 MB** | `{"changes":{"file:///Users/meek/develope...` |

### workspace/executeCommand

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.3ms ⚡ | **10.3 MB** | `{"success":true}` |

### textDocument/codeAction

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 28.7ms ⚡ | **262.0 MB** | `[{"diagnostics":[{"code":"unused-import"...` |

---

*Benchmark run: 2026-03-07T21:12:22Z*
