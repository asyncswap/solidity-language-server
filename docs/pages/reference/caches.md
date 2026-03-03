# Caches

This page explains the cache architecture used by the language server today.

## Cache layers

The server uses two layers:

- Memory cache: live `CachedBuild`/AST state used to answer requests immediately.
- Disk cache: warm-start data under `.solidity-language-server/` so restart latency is lower.

## On-disk layout

Current primary schema is v2:

- `.solidity-language-server/solidity-lsp-schema-v2.json` (metadata)
- `.solidity-language-server/reference-index-v2/*.json` (per-file shards)

The cache directory now auto-generates:

- `.solidity-language-server/.gitignore`

with:

```gitignore [.gitignore]
*
```

so cache artifacts are not accidentally committed.

## What v2 stores

Metadata (`solidity-lsp-schema-v2.json`) includes:

- `schema_version`
- `project_root`
- `config_fingerprint` — Keccak256 of `lsp_version`, `solc_version`, `remappings`, `evm_version`, `sources_dir`, `libs`
- `file_hashes`
- `file_hash_history`
- `path_to_abs`
- `id_to_path_map`
- `external_refs`
- `node_shards`

Each shard file stores one source file’s node entries.

## Cache invalidation

The `config_fingerprint` field is a Keccak256 hash of the inputs that affect the reference index. If any of these change, the entire on-disk cache is treated as a miss and a clean rebuild is triggered automatically:

| Input | Why it matters |
|---|---|
| `lsp_version` | Any server upgrade rebuilds from scratch — no stale data from an old binary |
| `solc_version` | Different compiler → different AST node IDs |
| `remappings` | Affects which files are resolved and compiled |
| `evm_version` | Can change which code paths solc parses |
| `sources_dir` / `libs` | Determines the set of source files in scope |

Optimizer settings (`optimizer`, `optimizer_runs`, `via_ir`) are intentionally excluded — they affect bytecode output but not the AST shape or node IDs that the reference index is built from.

In addition to the fingerprint, each file has an independent Keccak256 content hash. A fingerprint match but content change for individual files triggers a scoped reconcile for those files only, without discarding the whole cache.

## Freshness model

### Startup (warm load)

On startup, the server tries to load v2, validates project/config/hash compatibility, and reuses matching files.  
If files changed, it performs scoped reconcile for changed/affected files, merges results, and writes back.

### During editing

On successful saves/builds:

- touched files are upserted quickly into v2 shards,
- sync is debounced + single-flight to avoid overlapping work during save bursts.

This keeps reference/goto data fresh without full-project recompiles on every save.

## Commands

The server exposes two `workspace/executeCommand` commands for cache management. Both are advertised during `initialize` so any LSP client can invoke them.

### `solidity.clearCache`

Deletes the entire `.solidity-language-server/` directory on disk and wipes the in-memory AST cache for the current project root. The next file save or open triggers a clean rebuild from scratch. Use this when the cache is corrupt, after a major foundry config change, or when you want to verify a fresh index.

**nvim:**
```lua
vim.lsp.buf.execute_command({ command = "solidity.clearCache" })
```

**VS Code / Cursor** (from the command palette or an extension):
```ts
vscode.commands.executeCommand("solidity.clearCache");
```

**Helix** (`:lsp-execute-command solidity.clearCache` — requires Helix ≥ 24.03 with execute-command support)

Returns `{ "success": true }` on success, or a JSON-RPC `InternalError` if the directory could not be removed.

### `solidity.reindex`

Evicts the in-memory AST cache entry for the current project root and sets the dirty flag so the background cache-sync worker triggers a fresh project index build. The on-disk cache is left intact, so the warm-load on reindex will be fast. Use this when go-to-definition or find-references feels stale but you do not want to throw away the disk cache.

**nvim:**
```lua
vim.lsp.buf.execute_command({ command = "solidity.reindex" })
```

**VS Code / Cursor:**
```ts
vscode.commands.executeCommand("solidity.reindex");
```

Returns `{ "success": true }`.

### Comparison

| | `solidity.clearCache` | `solidity.reindex` |
|---|---|---|
| Deletes `.solidity-language-server/` on disk | Yes | No |
| Clears in-memory AST cache | Yes | Yes |
| Triggers background reindex | Yes (from scratch) | Yes (warm from disk) |
| Speed of next index build | Slow (full recompile) | Fast (warm load) |
| Use when | Cache corrupt / config changed | Index feels stale |

## Settings that affect cache behavior

Under `projectIndex`:

- `cacheMode`: `v2`
- `incrementalEditReindex`: enable scoped affected-file reindex path
- `incrementalEditReindexThreshold`: ratio gate for scoped vs full reindex
- `fullProjectScan`: optional broader startup indexing mode

See setup schema in [Setup Overview](/setup).
