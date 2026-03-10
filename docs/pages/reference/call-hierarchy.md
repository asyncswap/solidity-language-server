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

By default, Neovim renders results in the quickfix list using `fromRanges` but can pair them with the wrong file name, which is confusing.

The recommended handlers below jump to the **call-site expression** (`fromRanges`) with just the function name in the quickfix text. For outgoing calls, `fromRanges` belong to the caller item file (`ctx.params.item.uri`), not the callee definition URI:

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

With `base_function_implementation` equivalence, incoming calls for `PoolManager.swap` also includes callers that reference `IPoolManager.swap` (the interface declaration), since they are equivalent functions.

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

### No separate call index

Call hierarchy queries are derived from the same `nodes` index that powers `textDocument/references`. There is no separate call-site index or pre-built call graph. Every AST node with a `referenced_declaration` is a potential call edge.

This approach works uniformly on both fresh builds (`CachedBuild::new()`) and warm-loaded builds (`from_reference_index()`) because the `nodes` index is always populated.

### Call edge resolution via span containment

The caller/callee relationship is resolved at query time via **span containment**: for each reference node whose `referenced_declaration` matches the target, the server finds the smallest enclosing `FunctionDefinition` or `ModifierDefinition` — that is the "caller".

For outgoing calls, the same principle works in reverse: find all reference nodes whose `src` falls inside the caller's span and whose `referenced_declaration` points to a callable.

### Equivalence via `base_function_implementation`

When `base_function_implementation` is populated, incoming calls expand the target to include all equivalent IDs (interface <-> implementation). This means:

- Incoming calls for `PoolManager.swap` includes callers referencing `IPoolManager.swap`
- Incoming calls for `IPoolManager.swap` includes callers referencing `PoolManager.swap`

The `base_function_implementation` index is bidirectional: built from `NodeInfo.base_functions`, it maps both interface -> implementation and implementation -> interface. See [Implementation](/reference/implementation) for details.

### fromRanges

Call-site ranges use the reference node's `src` directly — the full expression span at the call site. This gives narrow, precise ranges for:
- **Direct identifier calls** (e.g., `foo()`): the identifier span
- **Member access calls** (e.g., `pool.swap()`): the member access expression span

### Container aggregation

When querying a contract/interface/library:
- `incomingCalls(contract)` = union of incoming calls to all its callables
- `outgoingCalls(contract)` = union of outgoing calls from all its callables

## Cross-build resolution

Node IDs are per-compilation — the same numeric ID can refer to completely different functions in different builds. The server uses a two-tier strategy to safely resolve targets across builds.

### `verify_node_identity()`

O(1) identity proof that a `NodeId` in a specific build refers to the expected source entity. Checks three properties:

1. **File** — the node must exist in the expected file within this build
2. **Position** — the node's `name_location` byte offset must match
3. **Name** — the source text at `name_location` must match the expected name

If all three match, the node is guaranteed to be the same source entity regardless of which compilation produced the build.

### `resolve_target_in_build()`

Two-tier resolution strategy used by both incoming and outgoing call handlers:

1. **Fast path (O(1))**: `verify_node_identity()` — if the original numeric ID exists in this build and passes identity verification, accept it immediately.
2. **Slow path (O(n))**: `byte_to_id()` — if the ID doesn't exist or fails verification (e.g., sub-cache with a different function at the same numeric ID), re-resolve by byte offset using span containment.

Returns the resolved node IDs (empty if the build doesn't contain the target file).

### Why this matters

Without identity verification, a bare `find_node_info(&build.nodes, node_id)` across all builds would silently match the wrong function. For example, node ID 616 = `swap` in the file build, but node ID 616 = a completely different library function in a sub-cache. `resolve_target_in_build()` prevents this class of bug.

### Deduplication

When the same function appears in multiple builds (file build + project build both contain `PoolManager.swap`), the results have different `NodeId`s but the same source position. Results are deduplicated by **source position** (`selectionRange.start`), never by node ID.

## Key files

| File | Role |
|---|---|
| `src/call_hierarchy.rs` | Core module: `verify_node_identity()`, `resolve_target_in_build()`, `incoming_calls()`, `outgoing_calls()`, `resolve_callable_at_position()`, LSP conversion helpers |
| `src/goto.rs` | `CachedBuild` struct (field: `base_function_implementation`), construction |
| `src/references.rs` | `byte_to_id()` — span containment node lookup used by the slow path |
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

### incomingCalls

1. Extract `nodeId` from the `CallHierarchyItem.data`
2. Resolve target identity: extract `(abs_path, name, name_offset)` from the item's URI and `selectionRange`
3. Expand target IDs via `base_function_implementation` to include equivalent interface/implementation IDs
4. For each build:
   a. `resolve_target_in_build()` to get build-local target IDs
   b. `incoming_calls(nodes, &target_ids)` — scan `nodes` for references matching any target ID, resolve enclosing callables via span containment
   c. Build `CallHierarchyIncomingCall` items within the build loop (not after — prevents node ID leaks across builds)
5. Dedup by `selectionRange.start`

### outgoingCalls

1. Extract `nodeId` from the `CallHierarchyItem.data`
2. Resolve target identity: extract `(abs_path, name, name_offset)` from the item's URI and `selectionRange`
3. For each build:
   a. `resolve_target_in_build()` to get the build-local caller ID
   b. `outgoing_calls(nodes, caller_id)` — find all reference nodes inside the caller's span pointing to callables
   c. Build `CallHierarchyOutgoingCall` items within the build loop
4. Sort by source position (server-side, not client-side)
5. Dedup by `selectionRange.start`

## Known limitations

- **Cross-file callers from test files**: Incoming calls only include callers from files in the current build's scope. If a test file calls a function but isn't part of the file-level import closure, it requires a project-level build to appear. Use `waitForProgressToken: "solidity/projectIndexFull"` in benchmarks to ensure full coverage.
- **Yul internal functions**: Calls within `assembly {}` blocks to Yul-internal functions are not tracked. Only calls to Solidity-level callables (via `externalReferences`) are visible.
