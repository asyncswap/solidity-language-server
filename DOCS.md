## Installation

Solidity lsp server using foundry's build process.

**Install from crates.io**

```
cargo install solidity-language-server
```

**Build from source**

```sh
cargo build --release
```

## Getting Started

### Neovim

If you have neovim 0.11+ installed add these to your config

```lua
-- lsp/solidity-language-server.lua
return {
  name = "Solidity Language Server",
  cmd = { "solidity-language-server", "--stdio" },
  filetypes = { "solidity" },
  root_markers = { "foundry.toml", ".git" },
  on_attach = function(_, _)
    vim.api.nvim_create_autocmd("BufWritePost", {
      pattern = { "*.sol" },
      callback = function()
        vim.lsp.buf.format()
      end,
    })
  end,
  settings = {
    ["solidity-language-server"] = {
      inlayHints = {
        -- Show parameter name hints on function/event/struct calls.
        parameters = true,
      },
      lint = {
        -- Master toggle for forge lint diagnostics.
        enabled = true,
        -- Filter lints by severity. Empty = all severities.
        -- Values: "high", "med", "low", "info", "gas", "code-size"
        severity = {},
        -- Run only specific lint rules by ID. Empty = all rules.
        -- Values: "incorrect-shift", "unchecked-call", "erc20-unchecked-transfer",
        --   "divide-before-multiply", "unsafe-typecast", "pascal-case-struct",
        --   "mixed-case-function", "mixed-case-variable", "screaming-snake-case-const",
        --   "screaming-snake-case-immutable", "unused-import", "unaliased-plain-import",
        --   "named-struct-fields", "unsafe-cheatcode", "asm-keccak256", "custom-errors",
        --   "unwrapped-modifier-logic"
        only = {},
        -- Suppress specific lint rule IDs from diagnostics.
        exclude = {},
      },
      fileOperations = {
        -- Auto-generate scaffold for new .sol files.
        -- Set to false to disable scaffold generation.
        templateOnCreate = true,
        -- Auto-update imports via workspace/willRenameFiles.
        updateImportsOnRename = true,
        -- Auto-remove imports via workspace/willDeleteFiles.
        updateImportsOnDelete = true,
      },
    },
  },
}
-- init.lua
vim.lsp.enable("solidity-language-server")
```

### VSCode

You can add the following to VSCode (or cursor) using a lsp-proxy extension see comment [here](https://github.com/foundry-rs/foundry/pull/11187#issuecomment-3148743488):

```json
[
  {
    "languageId": "solidity",
    "command": "solidity-language-server",
    "fileExtensions": [
      ".sol"
    ]
  }
]
```

### Zed

Add the following to your Zed settings (`settings.json`):

```json
{
  "lsp": {
    "solidity": {
      "binary": {
        "path": "solidity-language-server", // or path to the binary
        "arguments": []
      }
    }
  }
}
```

## Settings

Settings are passed via `initializationOptions` or `didChangeConfiguration`. All settings are optional — defaults are shown below.

### Neovim

```lua
-- lsp/solidity_lsp.lua
return {
  cmd = { "solidity-language-server", "--stdio" },
  filetypes = { "solidity" },
  root_markers = { "foundry.toml", ".git" },
  settings = {
    ["solidity-language-server"] = {
      inlayHints = {
        -- Show parameter name hints on function/event/struct calls.
        parameters = true,
      },
      lint = {
        -- Master toggle for forge lint diagnostics.
        enabled = true,
        -- Filter lints by severity. Empty = all severities.
        -- Values: "high", "med", "low", "info", "gas", "code-size"
        severity = {},
        -- Run only specific lint rules by ID. Empty = all rules.
        -- Values: "incorrect-shift", "unchecked-call", "erc20-unchecked-transfer",
        --   "divide-before-multiply", "unsafe-typecast", "pascal-case-struct",
        --   "mixed-case-function", "mixed-case-variable", "screaming-snake-case-const",
        --   "screaming-snake-case-immutable", "unused-import", "unaliased-plain-import",
        --   "named-struct-fields", "unsafe-cheatcode", "asm-keccak256", "custom-errors",
        --   "unwrapped-modifier-logic"
        only = {},
        -- Suppress specific lint rule IDs from diagnostics.
        exclude = {},
      },
      fileOperations = {
        -- Auto-generate scaffold for new .sol files.
        -- Set to false to disable scaffold generation.
        templateOnCreate = true,
        -- Auto-update imports via workspace/willRenameFiles.
        updateImportsOnRename = true,
        -- Auto-remove imports via workspace/willDeleteFiles.
        updateImportsOnDelete = true,
      },
      projectIndex = {
        -- Build a full project index at startup for cross-file references.
        fullProjectScan = true,
        -- Persistent cache mode: "v2"
        cacheMode = "v2",
        -- Aggressive scoped dirty-sync reindex on save.
        -- Falls back to full reindex if scoped compile fails.
        incrementalEditReindex = false,
      },
    },
  },
}
```

### Helix

```toml
# languages.toml
[language-server.solidity-language-server.config]
inlayHints.parameters = true
lint.enabled = true
lint.severity = ["high", "med"]
lint.exclude = ["pascal-case-struct"]
fileOperations.templateOnCreate = true
fileOperations.updateImportsOnRename = true
fileOperations.updateImportsOnDelete = true
projectIndex.fullProjectScan = true
projectIndex.cacheMode = "v2"
projectIndex.incrementalEditReindex = false
```

### VSCode / Cursor

```json
{
  "solidity-language-server.inlayHints.parameters": true,
  "solidity-language-server.lint.enabled": true,
  "solidity-language-server.lint.severity": ["high", "med"],
  "solidity-language-server.lint.only": [],
  "solidity-language-server.lint.exclude": ["pascal-case-struct"],
  "solidity-language-server.fileOperations.templateOnCreate": true,
  "solidity-language-server.fileOperations.updateImportsOnRename": true,
  "solidity-language-server.fileOperations.updateImportsOnDelete": true,
  "solidity-language-server.projectIndex.fullProjectScan": true,
  "solidity-language-server.projectIndex.cacheMode": "v2",
  "solidity-language-server.projectIndex.incrementalEditReindex": false
}
```

### Zed

```json
{
  "lsp": {
    "solidity": {
      "binary": {
        "path": "solidity-language-server"
      },
      "settings": {
        "inlayHints": {
          "parameters": true
        },
        "lint": {
          "enabled": true,
          "severity": ["high", "med"],
          "exclude": ["pascal-case-struct"]
        },
        "fileOperations": {
          "templateOnCreate": true,
          "updateImportsOnRename": true,
          "updateImportsOnDelete": true
        },
        "projectIndex": {
          "fullProjectScan": true,
          "cacheMode": "v2",
          "incrementalEditReindex": false
        }
      }
    }
  }
}
```

## Debugging

### Neovim

Lsp logs are stored in `~/.local/state/nvim/lsp.log`

To clear lsp logs run:

```bash
> -f ~/.local/state/nvim/lsp.log
```

To monitor logs in real time run:

```bash
tail -f ~/.local/state/nvim/lsp.log
```

Enable traces in neovim to view full traces in logs:

```sh
# for [info] traces
:lua vim.lsp.set_log_level("info")
# for [debug] traces
:lua vim.lsp.set_log_level("trace")
```

## FAQ

### Renaming a `.sol` file doesn't update imports in Neovim

When you rename a file through a file explorer plugin (e.g. oil.nvim), the server should automatically update import paths in other files. If this isn't working, check the following:

**1. Verify workspace folders are set correctly**

Open a `.sol` file and run:

```vim
:lua for _, c in ipairs(vim.lsp.get_clients()) do print(c.name, vim.inspect(c.workspace_folders)) end
```

You should see an **absolute** `file://` URI like:

```
Solidity Language Server { { name = "/Users/you/project", uri = "file:///Users/you/project" } }
```

If you see `file://.` or a relative path, the problem is `root_dir` in your LSP config. **Do not** use `root_dir = vim.fs.root(0, { ... })` — it evaluates at config load time when buffer 0 may not be a `.sol` file, producing a bad value. Use `root_markers` instead, which Neovim resolves lazily per buffer:

```lua
-- lsp/solidity-language-server.lua
return {
  cmd = { "solidity-language-server" },
  filetypes = { "solidity" },
  root_markers = { "foundry.toml", ".git" },  -- use this, not root_dir
}
```

**2. Verify your file explorer sends `willRenameFiles`**

Not all file explorer plugins send LSP file operation requests. For oil.nvim, make sure `lsp_file_methods` is enabled:

```lua
require("oil").setup({
  lsp_file_methods = {
    enabled = true,
    timeout_ms = 1000,
    autosave_changes = "unmodified",
  },
})
```

**3. Check the LSP log for server responses**

```bash
tail -f ~/.local/state/nvim/lsp.log
```

You should see `workspace/willRenameFiles` followed by `willRenameFiles: N edit(s) across M file(s)`. If you see `willRenameFiles: no import edits needed`, the server couldn't find matching imports — the project index may not have finished building yet (wait for the "Indexing project" spinner to complete before renaming).

**4. Save all modified buffers after rename**

`workspace/willRenameFiles` applies import updates to open buffers. Your editor may not auto-save those buffers to disk.
After a rename, run:

```vim
:wa
```

This writes all modified files, so external tools (e.g. Foundry) read the updated imports from disk.
