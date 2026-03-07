## Features

- **Go to Definition** / **Go to Declaration** — jump to any symbol across files, including qualifier segments in qualified type paths (e.g., `Pool` in `Pool.State` navigates to the contract/library)
- **Find References** — all usages of a symbol across the project, including qualified type path references (e.g., `Pool` in `Pool.State`)
- **Rename** — project-wide symbol rename with prepare support, including qualifier usages in qualified type paths
- **Hover** — signatures, NatSpec docs, function/error/event selectors, `@inheritdoc` resolution, AST node ID for debugging
- **Completions** — scope-aware with two modes (fast cache vs full recomputation)
- **Document Links** — clickable imports, type names, function calls
- **Document Symbols** / **Workspace Symbols** — outline and search
- **Formatting** — via `forge fmt`
- **Diagnostics** — from `solc` and `forge lint`
- **Signature Help** — parameter info on function calls, event emits, and mapping access
- **Inlay Hints** — parameter names and gas estimates
- **File Operations** — `workspace/willCreateFiles` scaffolding + `workspace/willRenameFiles`/`workspace/willDeleteFiles` import edits + `workspace/didCreateFiles`/`workspace/didRenameFiles`/`workspace/didDeleteFiles` cache migration/re-index (`fileOperations.templateOnCreate`, `fileOperations.updateImportsOnRename`, `fileOperations.updateImportsOnDelete`)
- **Code Actions** — `textDocument/codeAction` quickfix engine; handles `unused-import` forge-lint diagnostic with "Remove unused import" action; JSON-driven rule table in `data/error_codes.json`
- **Execute Commands** — `solidity.clearCache` (wipe on-disk cache + in-memory AST, force clean rebuild) · `solidity.reindex` (evict in-memory AST, trigger background reindex from warm disk cache)
- **Save Performance** — content hash check skips redundant solc rebuilds when file is unchanged; `collect_import_pragmas` runs on blocking thread pool to avoid stalling the async runtime on large projects

See [FEATURES.md](FEATURES.md) for the full LSP feature set and roadmap.

### LSP Methods

**General**

- [x] `initialize` - Server initialization
- [x] `initialized` - Server initialized notification
- [x] `shutdown` - Server shutdown

**Text Synchronization**

- [x] `textDocument/didOpen` - Handle file opening
- [x] `textDocument/didChange` - Handle file content changes
- [x] `textDocument/didSave` - Handle file saving with diagnostics refresh
- [x] `textDocument/didClose` - Handle file closing
- [x] `textDocument/willSave` - File will save notification
- [ ] `textDocument/willSaveWaitUntil` - File will save wait until

**Diagnostics**

- [x] `textDocument/publishDiagnostics` - Publish compilation errors and warnings via `forge build`
- [x] `textDocument/publishDiagnostics` - Publish linting errors and warnings via `forge lint`

**Language Features**

- [x] `textDocument/definition` - Go to definition
- [x] `textDocument/declaration` - Go to declaration
- [x] `textDocument/references` - Find all references
- [x] `textDocument/documentSymbol` - Document symbol outline (contracts, functions, variables, events, structs, enums, etc.)
- [x] `textDocument/prepareRename` - Prepare rename validation
- [x] `textDocument/rename` - Rename symbols across files
- [x] `textDocument/formatting` - Document formatting
- [x] `textDocument/completion` - Code completion
- [x] `textDocument/hover` - Hover information
- [x] `textDocument/signatureHelp` - Function signature help (functions, events, mappings)
- [ ] `textDocument/typeDefinition` - Go to type definition
- [ ] `textDocument/implementation` - Go to implementation
- [x] `textDocument/documentHighlight` - Document highlighting (read/write classification)
- [x] `textDocument/codeAction` - Code actions (unused-import quickfix via forge-lint diagnostics)
- [ ] `textDocument/codeLens` - Code lens
- [x] `textDocument/documentLink` - Document links (clickable references and import paths)
- [ ] `textDocument/documentColor` - Color information
- [ ] `textDocument/colorPresentation` - Color presentation
- [ ] `textDocument/rangeFormatting` - Range formatting
- [ ] `textDocument/onTypeFormatting` - On-type formatting
- [x] `textDocument/foldingRange` - Folding ranges (contracts, functions, structs, enums, blocks, comments, imports)
- [x] `textDocument/selectionRange` - Selection ranges
- [x] `textDocument/inlayHint` - Inlay hints (parameter names, gas estimates)
- [x] `textDocument/semanticTokens` - Semantic tokens
- [x] `textDocument/semanticTokens/full` - Full semantic tokens
- [x] `textDocument/semanticTokens/range` - Range semantic tokens
- [x] `textDocument/semanticTokens/delta` - Delta semantic tokens

**Workspace Features**

- [x] `workspace/symbol` - Workspace-wide symbol search
- [x] `workspace/didChangeConfiguration` - Updates editor settings (inlay hints, lint options)
- [x] `workspace/didChangeWatchedFiles` - Acknowledges watched file changes (logs only)
- [x] `workspace/didChangeWorkspaceFolders` - Acknowledges workspace folder changes (logs only)
- [ ] `workspace/applyEdit` - Inbound handler not implemented (server uses outbound `workspace/applyEdit` to scaffold created files)
- [x] `workspace/executeCommand` - Execute workspace commands (`solidity.clearCache`, `solidity.reindex`)
- [x] `workspace/willCreateFiles` - File creation preview (scaffolding for `.sol`, `.t.sol`, `.s.sol`)
- [x] `workspace/didCreateFiles` - Post-create scaffold fallback + cache/index refresh
- [x] `workspace/willRenameFiles` - File rename preview (import path updates)
- [x] `workspace/didRenameFiles` - Post-rename cache migration + background re-index
- [x] `workspace/willDeleteFiles` - File deletion preview (removes imports to deleted files)
- [x] `workspace/didDeleteFiles` - Post-delete cache cleanup + background re-index

**Window Features**

- [ ] `window/showMessage` - Show message to user
- [ ] `window/showMessageRequest` - Show message request to user
- [x] `window/workDoneProgress` - Work done progress
