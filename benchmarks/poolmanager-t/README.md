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
| [mmsaki](https://github.com/asyncswap/solidity-language-server/releases/tag/v0.1.37) | `0.1.37` |
| [forge](https://github.com/foundry-rs/foundry) | `1.9.0` |

---

## Summary

| Method | mmsaki | forge |
|--------|--------|-------|
| [initialize](#initialize) | 16.0ms ⚡ | 22.5ms |
| [textDocument/diagnostic](#textdocumentdiagnostic) | 0.4ms | 0.2ms ⚡ |
| [textDocument/semanticTokens/full/delta](#textdocumentsemantictokensfulldelta) | 10.8ms ⚡ | No such method textDocument/semanticToke... |
| [textDocument/definition](#textdocumentdefinition) | 60.3ms | 0.1ms ⚡ |
| [textDocument/declaration](#textdocumentdeclaration) | 8.2ms | 0.1ms ⚡ |
| [textDocument/hover](#textdocumenthover) | 6.9ms | 0.1ms ⚡ |
| [textDocument/references](#textdocumentreferences) | 23.7ms | 0.3ms ⚡ |
| [textDocument/completion](#textdocumentcompletion) | 50.9ms | 0.3ms ⚡ |
| [textDocument/signatureHelp](#textdocumentsignaturehelp) | 6.1ms | 0.1ms ⚡ |
| [textDocument/rename](#textdocumentrename) | 23.3ms | 0.7ms ⚡ |
| [textDocument/prepareRename](#textdocumentpreparerename) | 0.3ms ⚡ | cannot rename this symbol because worksp... |
| [textDocument/documentSymbol](#textdocumentdocumentsymbol) | 7.3ms | 0.7ms ⚡ |
| [textDocument/documentHighlight](#textdocumentdocumenthighlight) | 7.4ms | 0.1ms ⚡ |
| [textDocument/documentLink](#textdocumentdocumentlink) | 1.7ms | 0.1ms ⚡ |
| [textDocument/formatting](#textdocumentformatting) | 32.4ms ⚡ | 44.5ms |
| [textDocument/foldingRange](#textdocumentfoldingrange) | 10.1ms | 0.7ms ⚡ |
| [textDocument/selectionRange](#textdocumentselectionrange) | 8.8ms | 0.1ms ⚡ |
| [textDocument/inlayHint](#textdocumentinlayhint) | 9.2ms | 2.2ms ⚡ |
| [textDocument/semanticTokens/full](#textdocumentsemantictokensfull) | 14.0ms ⚡ | No such method textDocument/semanticToke... |
| [textDocument/semanticTokens/range](#textdocumentsemantictokensrange) | 10.3ms ⚡ | No such method textDocument/semanticToke... |
| [workspace/symbol](#workspacesymbol) | 72.1ms | 34.1ms ⚡ |
| [workspace/willRenameFiles](#workspacewillrenamefiles) | 227.7ms | 1.1ms ⚡ |
| [workspace/willCreateFiles](#workspacewillcreatefiles) | 0.2ms | 0.1ms ⚡ |
| [workspace/willDeleteFiles](#workspacewilldeletefiles) | 226.0ms | 0.4ms ⚡ |
| [workspace/executeCommand](#workspaceexecutecommand) | 0.1ms ⚡ | unknown command `solidity.reindex` |
| [textDocument/prepareCallHierarchy](#textdocumentpreparecallhierarchy) | 0.5ms | 0.1ms ⚡ |
| [textDocument/codeAction](#textdocumentcodeaction) | timeout | 0.1ms |
| [callHierarchy/incomingCalls](#callhierarchyincomingcalls) | 29.4ms ⚡ | 0.1ms |
| [callHierarchy/outgoingCalls](#callhierarchyoutgoingcalls) | 5.9ms | 0.2ms ⚡ |

### Scorecard

| Server | Wins | Out of |
|--------|------|--------|
| **forge** | **20** | **29** |
| mmsaki | 8 | 29 |

---

## Results

### initialize

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 16.0ms ⚡ | **9.8 MB** | ok |
| **forge** | 22.5ms | 28.5 MB | ok |

### textDocument/diagnostic

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.4ms | **10.6 MB** | {"message":"lsp server initialized.","ty... |
| **forge** | 0.2ms ⚡ | 28.6 MB | {"message":"solar initialized","type":3} |

### textDocument/semanticTokens/full/delta

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 10.8ms ⚡ | **10.5 MB** | delta |
| **forge** | - | 28.2 MB | No such method textDocument/semanticToke... |

### textDocument/definition

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 60.3ms | **67.8 MB** | `TickMath.sol:9` |
| **forge** | 0.1ms ⚡ | 197.9 MB | `TickMath.sol:9` |

### textDocument/declaration

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 8.2ms | **67.6 MB** | `TickMath.sol:9` |
| **forge** | 0.1ms ⚡ | 193.3 MB | `TickMath.sol:9` |

### textDocument/hover

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.9ms | **67.8 MB** | error PoolNotInitialized() |
| **forge** | 0.1ms ⚡ | 185.2 MB | error PoolNotInitialized() |

### textDocument/references

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 23.7ms | **68.7 MB** | 67 references |
| **forge** | 0.3ms ⚡ | 189.4 MB | 67 references |

### textDocument/completion

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 50.9ms | **66.7 MB** | 23 items (amount0, amount1, checkTicks) |
| **forge** | 0.3ms ⚡ | 200.1 MB | 2 items (amount0, amount1) |

### textDocument/signatureHelp

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 6.1ms | **66.5 MB** | function bound(uint256 x, uint256 min, uint256 max... |
| **forge** | 0.1ms ⚡ | 197.3 MB | function bound(uint256 x, uint256 min, uint256 max... |

### textDocument/rename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 23.3ms | **69.3 MB** | 9 edits in 1 files |
| **forge** | 0.7ms ⚡ | 197.2 MB | 9 edits in 1 files |

### textDocument/prepareRename

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.3ms ⚡ | **67.3 MB** | ready (line 116) |
| **forge** | - | 187.6 MB | cannot rename this symbol because worksp... |

### textDocument/documentSymbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 7.3ms | **10.3 MB** | 35 symbols |
| **forge** | 0.7ms ⚡ | 28.3 MB | 258 symbols |

### textDocument/documentHighlight

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 7.4ms | **69.0 MB** | [{"kind":2,"range":{"end":{"character":1... |
| **forge** | 0.1ms ⚡ | 192.1 MB | [{"kind":2,"range":{"end":{"character":1... |

### textDocument/documentLink

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 1.7ms | **67.0 MB** | 33 links |
| **forge** | 0.1ms ⚡ | 191.2 MB | 33 links |

### textDocument/formatting

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 32.4ms ⚡ | **10.4 MB** | 1 edits |
| **forge** | 44.5ms | 28.4 MB | 1 edits |

### textDocument/foldingRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 10.1ms | **10.4 MB** | [{"endCharacter":1,"endLine":1261,"start... |
| **forge** | 0.7ms ⚡ | 28.2 MB | [{"endCharacter":63,"endLine":35,"kind":... |

### textDocument/selectionRange

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 8.8ms | **10.3 MB** | [{"parent":{"parent":{"parent":{"parent"... |
| **forge** | 0.1ms ⚡ | 24.7 MB | [{"parent":{"parent":{"parent":{"parent"... |

### textDocument/inlayHint

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 9.2ms | **69.1 MB** | 1082 hints (name:, hooks:, _manager:) |
| **forge** | 2.2ms ⚡ | 186.7 MB | 1271 hints (name:, : address, hooks:) |

### textDocument/semanticTokens/full

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 14.0ms ⚡ | **10.3 MB** | 1512 tokens |
| **forge** | - | 28.2 MB | No such method textDocument/semanticToke... |

### textDocument/semanticTokens/range

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 10.3ms ⚡ | **10.5 MB** | 417 tokens |
| **forge** | - | 28.2 MB | No such method textDocument/semanticToke... |

### workspace/symbol

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 72.1ms | **67.4 MB** | 1797 symbols |
| **forge** | 34.1ms ⚡ | 185.3 MB | 15657 symbols |

### workspace/willRenameFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 227.7ms | **67.9 MB** | 12 edits in 12 files |
| **forge** | 1.1ms ⚡ | 180.4 MB | null (valid) |

### workspace/willCreateFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.2ms | **67.0 MB** | null (valid) |
| **forge** | 0.1ms ⚡ | 185.8 MB | null (valid) |

### workspace/willDeleteFiles

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 226.0ms | **66.9 MB** | {"changes":{"file:///Users/meek/develope... |
| **forge** | 0.4ms ⚡ | 201.6 MB | null (valid) |

### workspace/executeCommand

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.1ms ⚡ | **10.5 MB** | {"success":true} |
| **forge** | - | 28.7 MB | unknown command `solidity.reindex` |

### textDocument/prepareCallHierarchy

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 0.5ms | **67.9 MB** | 1 items: `TickMath` |
| **forge** | 0.1ms ⚡ | 197.1 MB | 1 items: `test_addLiquidity_succeedsIfInitialized` |

### textDocument/codeAction

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | - | 402.5 MB | timeout |
| **forge** | 0.1ms | 199.3 MB | [] |

### callHierarchy/incomingCalls

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 29.4ms ⚡ | **67.1 MB** | 63 items: `getGasCostOfGetTickAtSqrtPrice`, `boundTicks`, `MIN_SQRT_PRICE`, ... (+60) |
| **forge** | 0.1ms | 199.6 MB | 0 items |

### callHierarchy/outgoingCalls

| Server | p95 | RSS | Result |
|--------|-----|-----|--------|
| **mmsaki** | 5.9ms | **67.5 MB** | 4 items: `revertWith`, `revertWith`, `mostSignificantBit`, ... (+1) |
| **forge** | 0.2ms ⚡ | 185.8 MB | 4 items: `bound`, `expectEmit`, `modifyLiquidity`, ... (+1) |

---

*Benchmark run: 2026-11-03T23:08:06Z*
