# Call Hierarchy

## Overview

The call hierarchy feature provides navigation of call graphs across Solidity contracts and libraries via three LSP methods:

- `textDocument/prepareCallHierarchy` — resolve the callable at the cursor
- `callHierarchy/incomingCalls` — find all callers of a function/modifier/contract
- `callHierarchy/outgoingCalls` — find all callees from a function/modifier/contract

## Editor usage

### Neovim

Neovim has built-in call hierarchy support. Add keymaps in your `LspAttach` callback:

```lua
keymap("<leader>i", vim.lsp.buf.incoming_calls, "Incoming calls")
keymap("<leader>o", vim.lsp.buf.outgoing_calls, "Outgoing calls")
```

By default, Neovim renders results in the quickfix list using `fromRanges` (call-site locations in the current file) rather than the target definition location. This can be confusing because the quickfix entry shows the target file name but with the call-site line number.

To show the target definition location instead, add custom handlers:

```lua [.config/nvim/plugin/lsp.lua]
// [!include ~/snippets/setup/neovim-call-hierarchy.lua]
```

#### Lspsaga

If you have [lspsaga.nvim](https://github.com/glepnir/lspsaga.nvim) installed, it provides a dedicated tree UI for call hierarchy that renders both the target and call-site information correctly:

```vim
:Lspsaga incoming_calls
:Lspsaga outgoing_calls
```

#### Telescope

Telescope has `lsp_outgoing_calls` and `lsp_incoming_calls` pickers, but note that Telescope sometimes does not display the target ranges correctly. Lspsaga displays call hierarchy results more reliably.

### VS Code

VS Code has native call hierarchy support. Right-click a function and select:

- **Show Call Hierarchy** (or `Shift+Alt+H`)
- Then switch between **Incoming Calls** and **Outgoing Calls** in the panel

### Other editors

Any editor that supports the LSP call hierarchy protocol will work. The server advertises `callHierarchyProvider: true` in its capabilities.

## What gets tracked

### Incoming calls

"Who calls this function?" — finds every function, modifier, or constructor that calls the target.

Example: incoming calls for `_getPool` in PoolManager.sol returns `modifyLiquidity`, `swap`, and `donate`.

### Outgoing calls

"What does this function call?" — finds every function and modifier invoked from within the target.

Example: outgoing calls from `swap` in PoolManager.sol returns `onlyWhenUnlocked`, `noDelegateCall`, `revertWith`, `toId`, `_getPool`, `checkPoolInitialized`, `beforeSwap`, `_swap`, `afterSwap`, and `_accountPoolBalanceDelta`.

### Supported callable types

| Type | Supported |
|---|---|
| Functions (regular, constructor, fallback, receive) | Yes |
| Modifiers | Yes |
| Contracts/Interfaces/Libraries (aggregate) | Yes |
| Yul internal functions | No |

### Skipped call types

These are intentionally excluded from the call graph:

- `structConstructorCall` — struct literal construction, not a function call
- `typeConversion` — e.g., `address(x)`, `uint256(y)`
- Event emits — not function calls
- Built-in functions — negative `referencedDeclaration` IDs (e.g., `require`, `assert`)

## Architecture

### Index building

The call hierarchy index is built at cache time in `CachedBuild::new()` by walking the raw solc AST JSON. Two data structures are produced:

- **`call_sites: CallSiteIndex`** — per-file list of `CallSite` records, each containing `caller_id`, `callee_id`, `call_src`, and `kind`
- **`container_callables: ContainerCallables`** — per-file map from container IDs (contracts/interfaces/libraries) to their callable IDs (functions/modifiers/constructors)

### Call edge sources

Edges are recorded from:

| AST node type | Filter | Edge |
|---|---|---|
| `FunctionCall` | `kind == "functionCall"` | caller → `referencedDeclaration` on `expression` child |
| `ModifierInvocation` | always | function → `referencedDeclaration` on `modifierName` child |

### fromRanges

Call-site ranges are narrow:
- **Direct identifier calls** (e.g., `foo()`): uses `expression.src`
- **Member access calls** (e.g., `pool.swap()`): uses `expression.memberLocation` (the `.swap` portion)

### Container aggregation

When querying a contract/interface/library:
- `incomingCalls(contract)` = union of incoming calls to all its callables
- `outgoingCalls(contract)` = union of outgoing calls from all its callables

### Canonicalization

The `call_src` strings from the raw AST use solc's per-compilation file IDs. After building the index, `canonicalize_call_sites()` rewrites all `call_src` file IDs using the `PathInterner` remap table so they match the canonical `id_to_path_map`.

## Key files

| File | Role |
|---|---|
| `src/call_hierarchy.rs` | Core module: data structures, index builder, query functions, LSP conversion helpers |
| `src/goto.rs` | `CachedBuild` struct (fields: `call_sites`, `container_callables`), construction, merge |
| `src/lsp.rs` | LSP handlers: `prepare_call_hierarchy`, `incoming_calls`, `outgoing_calls`; capability advertisement |

## Runtime flow

### prepareCallHierarchy

1. `byte_to_id()` finds the innermost AST node at the cursor
2. `resolve_callable_at_position()` checks:
   - Is the node itself a callable declaration? Return its ID
   - Does the node reference a callable via `referencedDeclaration`? Return that
   - Find the narrowest enclosing callable by span containment
3. Build a `CallHierarchyItem` from either `decl_index` (fresh build) or `nodes` index (warm cache fallback)
4. Store `nodeId` in the item's `data` field for use by incoming/outgoing handlers

### incomingCalls / outgoingCalls

1. Extract `nodeId` from the `CallHierarchyItem.data`
2. Check if it's a container (contract) or callable (function/modifier)
3. Query `call_sites` for matching edges, grouped by caller/callee
4. Build a `CallHierarchyItem` for each unique caller/callee
5. Return with `fromRanges` listing all call-site ranges

## Warm cache behavior

In warm-loaded project builds (`from_reference_index()`), `call_sites` and `container_callables` are empty (same as `decl_index`). The single-file build always has them populated. The `merge_missing_from()` and `merge_scoped_cached_build()` functions propagate call hierarchy data across builds.

## Known limitations

- **Interface/implementation boundary**: If test code calls `manager.swap(...)` where `manager` is typed as `IPoolManager`, the call references `IPoolManager.swap` (the interface declaration), not `PoolManager.swap` (the implementation). Incoming calls for `PoolManager.swap` won't include those callers. See [#193](https://github.com/mmsaki/solidity-language-server/issues/193) for the planned fix using `baseFunctions` equivalence.
- **Cross-file callers from test files**: Incoming calls only include callers from files in the current build's scope. If a test file calls a function but isn't part of the file-level import closure, it may require a project-level build to appear.
