# Solidity Language Server: Hover Documentation

## Problem

Solidity developers need to see documentation and type signatures when hovering over symbols. The compiler AST includes NatSpec documentation on declaration nodes, but the LSP needs to resolve cursor positions to declaration nodes and extract + format this information.

## AST Documentation Structure

### StructuredDocumentation (Object Form)

Most documented nodes use an object with a `text` field:

```json
{
  "id": 2411,
  "nodeType": "FunctionDefinition",
  "name": "initialize",
  "documentation": {
    "id": 2401,
    "nodeType": "StructuredDocumentation",
    "src": "5732:356:11",
    "text": "@notice Initialize the state for a given pool ID\n @param key The pool key\n @param sqrtPriceX96 The initial square root price\n @return tick The initial tick of the pool"
  }
}
```

### String Form (Inline Comments)

Rare — some `ExpressionStatement` nodes have plain string documentation:

```json
{
  "nodeType": "ExpressionStatement",
  "documentation": " Equivalent to:\n   amount1 = FullMath.mulDiv(liquidity, ...)"
}
```

### Node Types With Documentation

From Uniswap v4 PoolManager AST (273 documented nodes total):

| Node Type | Count |
|-----------|-------|
| FunctionDefinition | 156 |
| ContractDefinition | 42 |
| ErrorDefinition | 36 |
| VariableDeclaration | 23 |
| EventDefinition | 8 |
| StructDefinition | 4 |
| ModifierDefinition | 3 |
| ExpressionStatement | 1 |

## Resolution Approach

### Step 1: Cursor → Node ID

Reuse existing infrastructure from `references.rs`:

1. `byte_to_decl_via_external_refs()` — checks Yul external references first
2. `byte_to_id()` — finds the smallest-span node containing the cursor position

### Step 2: Follow referencedDeclaration

The node at the cursor is usually a usage (Identifier, MemberAccess, etc.) that has a `referencedDeclaration` pointing to the declaration node ID. Follow it to get the actual declaration.

```
Cursor on `pool.swap(...)` → Identifier(id=X, referencedDeclaration=1167) → FunctionDefinition(id=1167)
```

### Step 3: Find Raw AST Node

`NodeInfo` (from `cache_ids`) doesn't store documentation, type descriptions, parameters, etc. We need to walk the raw AST JSON to find the declaration node by ID.

`find_node_by_id(sources, target_id)` does a DFS over all source ASTs using `CHILD_KEYS`, same traversal pattern as `cache_ids`.

### Step 4: Extract Signature + Documentation

From the raw AST node, extract:
- **Signature**: Built from `nodeType`, `name`, `visibility`, `stateMutability`, `parameters`, `returnParameters`, `contractKind`, `members`, etc.
- **Documentation**: From `documentation.text` (object) or `documentation` (string)

### Step 5: Format as Markdown

Output structure:

````
```solidity
function swap(struct PoolKey memory key, struct SwapParams memory params, bytes hookData) external returns (BalanceDelta swapDelta)
```

---
Swap against the given pool

*Swapping on low liquidity pools may cause unexpected swap amounts...*

**Parameters:**
- `key` — The pool to swap in
- `params` — The parameters for swapping
- `hookData` — The data to pass through to the swap hooks

**Returns:**
- `swapDelta` — The balance delta of the address swapping
````

## `@inheritdoc` Resolution via Selector Matching

Many implementation functions use `@inheritdoc IParentContract` instead of repeating NatSpec. We resolve these by matching function selectors between implementation and interface.

### Why Selectors

Function selectors are unique per overload — `keccak256(signature)[:4]`. Matching by name alone fails for overloaded functions like `extsload(bytes32)` vs `extsload(bytes32, uint256)` vs `extsload(bytes32[])`. Selectors handle this correctly:

| Implementation | Selector | Interface Match |
|---------------|----------|-----------------|
| `Extsload.extsload(bytes32)` | `1e2eaeaf` | `IExtsload.extsload(bytes32)` |
| `Extsload.extsload(bytes32, uint256)` | `35fd631a` | `IExtsload.extsload(bytes32, uint256)` |
| `Extsload.extsload(bytes32[])` | `dbd035ff` | `IExtsload.extsload(bytes32[])` |

### Resolution Algorithm

1. **Parse** `@inheritdoc ParentName` from documentation text
2. **Get** the implementation function's `functionSelector` from the AST node
3. **Find scope** — the `scope` field gives the containing contract ID
4. **Walk `baseContracts`** of the scope contract, find the one where `baseName.name == ParentName`
5. **Follow** `baseName.referencedDeclaration` to get the parent contract node
6. **Match by selector** — iterate parent's `.nodes[]`, find the child with the same `functionSelector`
7. **Extract** the matched node's documentation

### AST Fields Used

```json
{
  "id": 1167,
  "name": "swap",
  "nodeType": "FunctionDefinition",
  "functionSelector": "f3cd914c",
  "scope": 1767,
  "documentation": { "text": "@inheritdoc IPoolManager" }
}
```

The `scope` (1767 = PoolManager) has:
```json
{
  "baseContracts": [
    { "baseName": { "name": "IPoolManager", "referencedDeclaration": 2531 } }
  ]
}
```

And IPoolManager (2531) has a child with the same selector:
```json
{
  "id": 2444,
  "name": "swap",
  "functionSelector": "f3cd914c",
  "documentation": { "text": "@notice Swap against the given pool..." }
}
```

### Fallback

If we can't resolve (no selector, parent not in AST, etc.), we show: *Inherits documentation from \`ParentName\`*

## Selectors in Hover Output

All three selector types from the AST are shown in hover:

| AST Field | Node Types | Length | Example |
|-----------|-----------|--------|---------|
| `functionSelector` | FunctionDefinition, public VariableDeclaration | 4 bytes | `0xf3cd914c` |
| `errorSelector` | ErrorDefinition | 4 bytes | `0x0d89438e` |
| `eventSelector` | EventDefinition | 32 bytes | `0x40e9cecb...` |

Internal/private functions have no selector (`functionSelector: null`).

### Hover Output Example (with resolved @inheritdoc)

````
```solidity
function swap(struct PoolKey memory key, struct SwapParams memory params, bytes hookData) external returns (BalanceDelta swapDelta)
```

Selector: `0xf3cd914c`

---
Swap against the given pool

*Swapping on low liquidity pools may cause unexpected swap amounts...*

**Parameters:**
- `key` — The pool to swap in
- `params` — The parameters for swapping
- `hookData` — The data to pass through to the swap hooks

**Returns:**
- `swapDelta` — The balance delta of the address swapping
````

## userdoc/devdoc DocIndex

Instead of only relying on raw AST `documentation` text (which requires manual `@inheritdoc` resolution), we now also consume solc's pre-resolved `userdoc` and `devdoc` from contract output. These are already requested in `outputSelection` and contain fully resolved documentation — `@inheritdoc` is handled by the compiler.

### What solc provides

**userdoc** — user-facing docs keyed by canonical ABI signature:
```json
{
  "notice": "Holds the state for all pools",
  "methods": {
    "swap((address,address,uint24,int24,address),(bool,int256,uint160),bytes)": {
      "notice": "Swap against the given pool"
    }
  },
  "events": { "Swap(bytes32,address,...)": { "notice": "..." } },
  "errors": { "AlreadyUnlocked()": [{ "notice": "..." }] }
}
```

**devdoc** — developer docs with params and returns:
```json
{
  "title": "PoolManager",
  "methods": {
    "swap(...)": {
      "details": "Swapping on low liquidity pools...",
      "params": { "key": "The pool to swap in", ... },
      "returns": { "swapDelta": "The balance delta..." }
    }
  },
  "stateVariables": { "MAX_TICK_SPACING": { "details": "..." } }
}
```

### DocIndex construction (`build_doc_index`)

At `CachedBuild::new()` time, we iterate `contracts[path][name]` and merge userdoc + devdoc into `DocEntry` values. Each entry is keyed by a typed `DocKey`:

| DocKey variant | Key source | Example |
|---------------|------------|---------|
| `DocKey::Func(FuncSelector)` | `evm.methodIdentifiers` for methods, `keccak256(sig)[0..4]` for errors | `Func("f3cd914c")` |
| `DocKey::Event(EventSelector)` | `keccak256(sig)` full hash | `Event("40e9cecb...")` |
| `DocKey::Contract(String)` | `"path:Name"` | `Contract("src/PoolManager.sol:PoolManager")` |
| `DocKey::StateVar(String)` | `"path:Contract:var"` | `StateVar("src/Pool.sol:Pool:MAX_TICK")` |
| `DocKey::Method(String)` | fallback `"path:Contract:fnName"` | `Method("src/Foo.sol:Foo:bar")` |

### Matching strategy

userdoc/devdoc keys use canonical ABI signatures (`swap((address,address,uint24,int24,address),...)`) while AST nodes use Solidity types (`struct PoolKey`). We bridge this via `evm.methodIdentifiers`:

```
methodIdentifiers: { "swap((address,...),bytes)": "f3cd914c" }
                                                      │
AST node: { functionSelector: "f3cd914c" } ───────────┘
```

For errors: we compute `keccak256(canonical_sig)[0..4]` to get the selector.
For events: we compute the full `keccak256(canonical_sig)` topic hash.

### Lookup order in `hover_info()`

1. **DocIndex** (`lookup_doc_entry`) — structured devdoc/userdoc, pre-resolved `@inheritdoc`
2. **AST documentation** (`extract_documentation` + `resolve_inheritdoc`) — raw NatSpec text
3. **Parameter docs** (`lookup_param_doc`) — walks up to parent, extracts `@param`/`@return`

## Parameter and Return Value Documentation

When hovering a parameter or return value — whether at its declaration site or any usage inside the function body — we show the `@param`/`@return` description from the parent function's documentation.

### How it works

1. **Cursor resolves to the declaration node** — when you hover `key` used inside the swap body, the AST `Identifier` node has `referencedDeclaration` pointing to the `VariableDeclaration` parameter. `hover_info()` follows this at line `let decl_id = node_info.referenced_declaration.unwrap_or(node_id)`, so `decl_node` is always the parameter's declaration regardless of hover location.

2. **Walk up via `scope`** — the parameter's `VariableDeclaration` has `scope` pointing to the parent `FunctionDefinition`, `ErrorDefinition`, or `EventDefinition`:
   ```json
   { "id": 1029, "name": "key", "nodeType": "VariableDeclaration", "scope": 1167 }
   ```
   Where `scope: 1167` is `PoolManager.swap`.

3. **Determine param kind** — check if the declaration ID appears in `returnParameters.parameters[]` (return value) or `parameters.parameters[]` (input param).

4. **Extract the doc** — two resolution paths:
   - **DocIndex path**: look up the parent function's `DocEntry` by selector → extract matching `params["key"]` or `returns["delta"]` entry. This handles `@inheritdoc` automatically because solc already resolved it.
   - **AST fallback**: parse the parent's raw `documentation.text`, resolve `@inheritdoc` if present, scan for `@param key ...` or `@return delta ...` lines.

### Examples

Hovering `sqrtPriceCurrentX96` in `error PriceLimitAlreadyExceeded(uint160 sqrtPriceCurrentX96, ...)`:
````
```solidity
uint160 sqrtPriceCurrentX96
```

---
The invalid, already surpassed sqrtPriceLimitX96
````

Hovering `key` anywhere inside `PoolManager.swap()` body (resolved from `@inheritdoc IPoolManager`):
````
```solidity
struct PoolKey memory key
```

---
The pool to swap in
````

Hovering `delta` in `returns (BalanceDelta delta, BalanceDelta feeDelta)`:
````
```solidity
BalanceDelta internal delta
```

---
the deltas of the token balances of the pool, from the liquidity change
````

## Typed Selectors

Raw hex selector strings are wrapped in newtypes defined in `src/types.rs`:

| Type | Width | Source | Example |
|------|-------|--------|---------|
| `FuncSelector` | 4 bytes (8 hex chars) | AST `functionSelector`, `errorSelector` | `FuncSelector::new("f3cd914c")` |
| `EventSelector` | 32 bytes (64 hex chars) | AST `eventSelector` | `EventSelector::new("8be0079c...")` |
| `Selector` | enum | `extract_selector()` return | `Selector::Func(...)`, `Selector::Event(...)` |
| `MethodId` | variable | `evm.methodIdentifiers` keys, userdoc/devdoc keys | `MethodId::new("swap((address,...),bytes)")` |

These prevent mixing up selectors with other strings and make the data flow self-documenting:
- `gas_by_selector()` takes `&FuncSelector` (not `&str`)
- `ContractGas.external_by_selector` is `HashMap<FuncSelector, String>` (not `HashMap<String, String>`)
- `DocKey::Func(FuncSelector)` vs `DocKey::Event(EventSelector)` — can't accidentally use a 4-byte selector to look up an event

Display helpers: `selector.as_hex()` → `"f3cd914c"`, `selector.to_prefixed()` → `"0xf3cd914c"`.

## NatSpec Tag Handling

| Tag | Rendering |
|-----|-----------|
| `@notice` | Plain text (main description) |
| `@dev` | Italic text |
| `@param name desc` | `- \`name\` — desc` under **Parameters:** header |
| `@return name desc` | `- \`name\` — desc` under **Returns:** header |
| `@inheritdoc Parent` | Resolved via selector matching, falls back to *Inherits documentation from \`Parent\`* |
| `@author` | Skipped (not useful in hover) |

## Signature Generation

Signatures are generated for all major node types:

| Node Type | Example |
|-----------|---------|
| FunctionDefinition | `function swap(...) external returns (...)` |
| ContractDefinition | `contract PoolManager is IPoolManager, ERC6909Claims, ...` |
| StructDefinition | `struct PoolKey { ... }` with member listing |
| EnumDefinition | `enum Direction { ... }` with member listing |
| EventDefinition | `event Transfer(address from, address to, uint256 amount)` |
| ErrorDefinition | `error InvalidCaller()` |
| ModifierDefinition | `modifier onlyOwner()` |
| VariableDeclaration | `mapping(PoolId => Pool.State) internal _pools` |
| UserDefinedValueTypeDefinition | `type PoolId is bytes32` |

## Exploration Commands

```sh
# All documented node types and counts
cat pool-manager-ast.json | jq '[.. | objects | select(.documentation != null) | .nodeType] | group_by(.) | map({type: .[0], count: length})'

# Function with full NatSpec
cat pool-manager-ast.json | jq '.. | objects | select(.name == "swap" and .nodeType == "FunctionDefinition") | {id, name, documentation: .documentation.text, visibility, stateMutability}'

# Check documentation format types (object vs string)
cat pool-manager-ast.json | jq '[.. | objects | select(.documentation != null) | .documentation | type] | unique'

# Find a specific node by ID
cat pool-manager-ast.json | jq '.. | objects | select(.id == 2411) | {id, name, nodeType, documentation}'

# All @inheritdoc functions with their selectors
cat pool-manager-ast.json | jq '.. | objects | select(has("documentation") and (.documentation | type == "object") and (.documentation.text | contains("@inheritdoc"))) | {id, name, functionSelector, documentation: .documentation.text, scope}'

# Check overloaded @inheritdoc functions
cat pool-manager-ast.json | jq -c '[.. | objects | select(has("documentation") and (.documentation | type == "object") and (.documentation.text | contains("@inheritdoc")))] | group_by(.name) | map(select(length > 1) | {name: .[0].name, count: length})'

# All selectors by type
cat pool-manager-ast.json | jq '.. | objects | select(.functionSelector != null) | {id, name, nodeType, functionSelector}'
cat pool-manager-ast.json | jq '.. | objects | select(.errorSelector != null) | {id, name, errorSelector}'
cat pool-manager-ast.json | jq '.. | objects | select(.eventSelector != null) | {id, name, eventSelector}'

# Verify selector matching between implementation and interface
cat pool-manager-ast.json | jq '.. | objects | select(.name == "extsload" and .nodeType == "FunctionDefinition") | {id, name, functionSelector, scope}'

# userdoc/devdoc from solc contract output (poolmanager.json)
PM='.contracts["/Users/meek/developer/mmsaki/solidity-language-server/v4-core/src/PoolManager.sol"]["PoolManager"]'

# List all userdoc method keys (canonical ABI signatures)
cat poolmanager.json | jq "$PM.userdoc.methods | keys"

# View userdoc notice for swap
cat poolmanager.json | jq "$PM.userdoc.methods[\"swap((address,address,uint24,int24,address),(bool,int256,uint160),bytes)\"]"

# View devdoc for swap (details, params, returns)
cat poolmanager.json | jq "$PM.devdoc.methods[\"swap((address,address,uint24,int24,address),(bool,int256,uint160),bytes)\"]"

# Contract-level userdoc/devdoc
cat poolmanager.json | jq "$PM.userdoc | {kind, notice}"
cat poolmanager.json | jq "$PM.devdoc | {kind, title, details, author}"

# methodIdentifiers (bridges ABI sigs to 4-byte selectors)
cat poolmanager.json | jq "$PM.evm.methodIdentifiers"

# All error docs (userdoc errors are arrays)
cat poolmanager.json | jq "$PM.userdoc.errors"

# All event docs with params
cat poolmanager.json | jq "$PM.devdoc.events"

# State variable docs
cat poolmanager.json | jq "$PM.devdoc.stateVariables"

# List all contracts in solc output
cat poolmanager.json | jq '[.contracts | to_entries[] | {path: .key, contracts: (.value | keys)}]'

# Parameter scope → parent function (pool-manager-ast.json)
cat pool-manager-ast.json | jq '.. | objects | select(.id == 1029) | {id, name, scope}'
# Then check the parent
cat pool-manager-ast.json | jq '.. | objects | select(.id == 1167) | {id, name, nodeType}'
```

## Foundry Config Support

The LSP reads compiler settings from `foundry.toml` and passes them to `solc --standard-json`. Without this, projects that require `via_ir = true` or specific optimizer/EVM settings fail to compile (e.g. "Stack too deep" errors).

### Settings read from `[profile.default]`

| foundry.toml key | solc standard JSON key | Default |
|------------------|----------------------|---------|
| `solc` / `solc_version` | (binary selection) | pragma-based |
| `remappings` | `settings.remappings` | `forge remappings` fallback |
| `via_ir` | `settings.viaIR` | `false` |
| `optimizer` | `settings.optimizer.enabled` | `false` |
| `optimizer_runs` | `settings.optimizer.runs` | `200` |
| `evm_version` | `settings.evmVersion` | solc default |
| `ignored_error_codes` | (diagnostic filtering) | `[5574, 3860]` hardcoded |

### Example: EkuboProtocol/evm-contracts

```toml
[profile.default]
solc = '0.8.33'
optimizer = true
optimizer_runs = 9999999
via_ir = true
evm_version = 'osaka'
ignored_error_codes = [2394, 6321, 3860, 5574, 2424, 8429, 4591]
```

Without `via_ir = true` and `optimizer = true`, solc fails with:
```
Compiler error (libsolidity/codegen/LValue.cpp:54): Stack too deep.
Try compiling with `--via-ir` (cli) or the equivalent `viaIR: true` (standard JSON)
while enabling the optimizer.
```

### Ignored error codes

Default suppressed codes (hardcoded):
- `5574` — contract code size exceeds limit
- `3860` — contract initcode size exceeds limit

User-configured codes from `ignored_error_codes` in `foundry.toml` are suppressed in addition to the defaults.

### Config reload

The config is loaded on `initialize` and reloaded automatically when `foundry.toml` changes (via file watcher). Settings take effect on the next compilation (file save/open).

## Known Limitations

### Multi-line `@dev` is flattened by solc

Solc's `devdoc.details` concatenates multi-line `@dev` content into a single string, stripping newlines. The raw AST `documentation.text` preserves newlines, but we prefer devdoc as the source of truth (it handles `@inheritdoc` resolution and structured `@param`/`@return` parsing).

**Source:**
```solidity
/// @dev Core uses a custom storage layout to avoid keccak's where possible.
///      For certain storage values, the pool id is used as a base offset and
///      we allocate the following relative offsets (starting from the pool id) as:
///        0: pool state
///        [FPL_OFFSET, FPL_OFFSET + 1]: fees per liquidity
```

**AST `documentation.text`** (preserves newlines):
```
@dev Core uses a custom storage layout...\n      For certain storage values...
```

**devdoc `details`** (flattened — this is what we display):
```
Core uses a custom storage layout...      For certain storage values...
```

This is a solc limitation, not ours. Users who want formatted multi-line output in hover should use separate `@dev` tags or accept the single-line rendering.

### Struct member NatSpec not captured

Solc does NOT populate `documentation` on struct member `VariableDeclaration` nodes — they are always `null` in the AST, even when `///` comments exist above each field. The struct-level `StructDefinition` node does have documentation, but individual member docs are lost. Neither `userdoc` nor `devdoc` contain struct member docs.

```solidity
struct PoolKey {
    /// @notice The lower currency of the pool
    Currency currency0;  // ← AST node has documentation: null
}
```

### Free functions have no devdoc

Free functions (defined at file level, outside any contract/library) exist in the AST but produce no `contracts` entry in solc output. This means:

- **No devdoc/userdoc** — these are per-contract, so free functions get none
- **No ABI or method identifiers**

Hover still works for free functions via the raw AST `documentation.text` fallback (Path B), but structured doc formatting is unavailable.

**Example:** `src/math/twamm.sol` in EkuboProtocol defines 7 free functions (`computeSaleRate`, `addSaleRateDelta`, etc.). Solc returns `contracts` output only for imported libraries (e.g. `FixedPointMathLib`), not for the free functions themselves.

```sh
# Verify: solc contracts output has no entry for free functions
cat solc-output.json | jq '.contracts["src/math/twamm.sol"]'
# → null (no contract/library defined in this file)
```

## Performance

- Uses `ast_cache` (Arc-based) — no forge calls on hover
- `DocIndex` is built once at `CachedBuild::new()` time
- `find_node_by_id` walks the AST once per hover request
- `lookup_doc_entry` is a HashMap lookup by selector — O(1)
- `lookup_param_doc` does one `find_node_by_id` for the parent + one `lookup_doc_entry`
- `resolve_inheritdoc` (fallback path) calls `find_node_by_id` up to 2 more times

## Files

| File | Purpose |
|------|---------|
| `src/hover.rs` | `hover_info()`, `find_node_by_id()`, `extract_documentation()`, `extract_selector()`, `resolve_inheritdoc()`, `format_natspec()`, `build_function_signature()`, `build_doc_index()`, `lookup_doc_entry()`, `lookup_param_doc()`, `format_doc_entry()`, `compute_selector()`, `compute_event_topic()` |
| `src/types.rs` | `FuncSelector`, `EventSelector`, `Selector`, `MethodId` |
| `src/goto.rs` | `CachedBuild` — stores `doc_index` field |
| `src/lsp.rs` | `hover` handler, passes `doc_index` to `hover_info()` |
| `src/completion.rs` | Uses `FuncSelector::to_prefixed()` for selector display |
