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
| [mmsaki](https://github.com/mmsaki/solidity-language-server/releases/tag/v0.1.30) | `0.1.30` |

---

## Summary

| Method | mmsaki |
|--------|--------|
| [initialize](#initialize) | 19.3ms ⚡ |
| [textDocument/diagnostic](#textdocumentdiagnostic) | 2.7ms ⚡ |
| [textDocument/semanticTokens/full/delta](#textdocumentsemantictokensfulldelta) | 5.0ms ⚡ |
| [textDocument/definition](#textdocumentdefinition) | 2.9ms ⚡ |
| [textDocument/declaration](#textdocumentdeclaration) | 0.5ms ⚡ |
| [textDocument/hover](#textdocumenthover) | 3.2ms ⚡ |
| [textDocument/references](#textdocumentreferences) | 2.5ms ⚡ |
| [textDocument/completion](#textdocumentcompletion) | 11.8ms ⚡ |
| [textDocument/signatureHelp](#textdocumentsignaturehelp) | 2.9ms ⚡ |
| [textDocument/rename](#textdocumentrename) | 5.0ms ⚡ |
| [textDocument/prepareRename](#textdocumentpreparerename) | 0.2ms ⚡ |
| [textDocument/documentSymbol](#textdocumentdocumentsymbol) | 10.2ms ⚡ |
| [textDocument/documentHighlight](#textdocumentdocumenthighlight) | 3.1ms ⚡ |
| [textDocument/documentLink](#textdocumentdocumentlink) | 0.6ms ⚡ |
| [textDocument/formatting](#textdocumentformatting) | 70.1ms ⚡ |
| [textDocument/foldingRange](#textdocumentfoldingrange) | 5.0ms ⚡ |
| [textDocument/selectionRange](#textdocumentselectionrange) | 8.9ms ⚡ |
| [textDocument/inlayHint](#textdocumentinlayhint) | 3.2ms ⚡ |
| [textDocument/semanticTokens/full](#textdocumentsemantictokensfull) | 9.9ms ⚡ |
| [textDocument/semanticTokens/range](#textdocumentsemantictokensrange) | 5.2ms ⚡ |
| [workspace/symbol](#workspacesymbol) | 3.2ms ⚡ |
| [workspace/willRenameFiles](#workspacewillrenamefiles) | 232.9ms ⚡ |
| [workspace/willCreateFiles](#workspacewillcreatefiles) | 1.6ms ⚡ |
| [workspace/willDeleteFiles](#workspacewilldeletefiles) | 232.9ms ⚡ |
| [workspace/executeCommand](#workspaceexecutecommand) | 0.1ms ⚡ |
| [textDocument/codeAction](#textdocumentcodeaction) | 0.1ms |

### Scorecard

| Server | Wins | Out of |
|--------|------|--------|
| **mmsaki** | **25** | **26** |

---

## Results

### initialize

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 19.3ms ⚡ | **9.8 MB** | ok |

### textDocument/diagnostic

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.7ms ⚡ | **10.3 MB** | 0 diagnostics |

### textDocument/semanticTokens/full/delta

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.0ms ⚡ | **10.3 MB** | delta |

### textDocument/definition

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.9ms ⚡ | **29.7 MB** | `TickMath.sol:9` |

### textDocument/declaration

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.5ms ⚡ | **29.0 MB** | `TickMath.sol:9` |

### textDocument/hover

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 3.2ms ⚡ | **29.6 MB** | function modifyLiquidity(struct Pool.State storage... |

### textDocument/references

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.5ms ⚡ | **28.5 MB** | 21 references |

### textDocument/completion

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 11.8ms ⚡ | **29.3 MB** | 28 items (slot0, feeGrowthGlobal0X128, feeGrowthGlobal1X128) |

### textDocument/signatureHelp

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 2.9ms ⚡ | **29.2 MB** | function getTickAtSqrtPrice(uint160 sqrtPriceX96) ... |

### textDocument/rename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.0ms ⚡ | **28.4 MB** | 13 edits in 1 files |

### textDocument/prepareRename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms ⚡ | **29.1 MB** | ready (line 102) |

### textDocument/documentSymbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 10.2ms ⚡ | **10.3 MB** | 16 symbols |

### textDocument/documentHighlight

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 3.1ms ⚡ | **28.6 MB** | [{"kind":2,"range":{"end":{"character":1... |

### textDocument/documentLink

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.6ms ⚡ | **28.8 MB** | 14 links |

### textDocument/formatting

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 70.1ms ⚡ | **10.3 MB** | 1 edits |

### textDocument/foldingRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.0ms ⚡ | **10.3 MB** | [{"endCharacter":1,"endLine":612,"startC... |

### textDocument/selectionRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 8.9ms ⚡ | **10.2 MB** | [{"parent":{"parent":{"parent":{"parent"... |

### textDocument/inlayHint

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 3.2ms ⚡ | **29.3 MB** | 114 hints (value1:, value2:, value:) |

### textDocument/semanticTokens/full

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 9.9ms ⚡ | **10.3 MB** | 697 tokens |

### textDocument/semanticTokens/range

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.2ms ⚡ | **10.3 MB** | 274 tokens |

### workspace/symbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 3.2ms ⚡ | **28.7 MB** | 68 symbols |

### workspace/willRenameFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 232.9ms ⚡ | **29.5 MB** | 12 edits in 12 files |

### workspace/willCreateFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.6ms ⚡ | **29.5 MB** | null (valid) |

### workspace/willDeleteFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 232.9ms ⚡ | **28.4 MB** | {"changes":{"file:///Users/meek/develope... |

### workspace/executeCommand

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.1ms ⚡ | **10.3 MB** | {"success":true} |

### textDocument/codeAction

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.1ms | 29.0 MB | null |

---

*Benchmark run: 2026-03-07T21:21:47Z*
