return {
  name = "Solidity Language Server",
  cmd = { "solidity-language-server", "--stdio" },
  filetypes = { "solidity" },
  root_markers = { "foundry.toml", ".git" },
  settings = {
    ["solidity-language-server"] = {
      inlayHints = {
        -- Show parameter name hints on function/event/struct calls.
        parameters = true,
        -- Show gas cost hints on functions annotated with
        -- `/// @custom:lsp-enable gas-estimates`.
        gasEstimates = true,
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
        fullProjectScan = true,
        -- Persistent cache mode: "v2"
        cacheMode = "v2",
        -- Aggressive scoped dirty-sync: reindex only reverse-import affected files.
        -- Falls back to full reindex if scoped compile fails.
        incrementalEditReindex = false,
      },
    },
  },
  on_attach = function(client, bufnr)
    vim.lsp.inlay_hint.enable(true, { bufnr = bufnr })

    -- autoformat
    vim.api.nvim_create_autocmd("BufWritePost", {
      callback = function()
        vim.lsp.buf.format()
      end,
    })

    -- completions autotrigger
    vim.lsp.completion.enable(true, client.id, bufnr, {
      autotrigger = true,
      convert = function(item)
        return { abbr = item.label:gsub('%b()', '') }
      end,
    })

    -- completion trigger list
    for _, char in ipairs({ "(", ",", "[" }) do
      vim.keymap.set("i", char, function()
        vim.api.nvim_feedkeys(char, "n", false)
        vim.defer_fn(vim.lsp.buf.signature_help, 50)
      end, { buffer = bufnr })
    end
  end,
}
