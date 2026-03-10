-- Custom call hierarchy handlers: jump to the call-site expression
-- (fromRanges) rather than the caller/callee function definition.
vim.lsp.handlers["callHierarchy/incomingCalls"] = function(_, result, ctx)
  if not result or vim.tbl_isempty(result) then
    vim.notify("No incoming calls found", vim.log.levels.INFO)
    return
  end
  local items = {}
  for _, call in ipairs(result) do
    local caller = call.from
    local filename = vim.uri_to_fname(caller.uri)
    -- Each fromRange is a call-site expression inside the caller.
    for _, range in ipairs(call.fromRanges or {}) do
      table.insert(items, {
        filename = filename,
        lnum = range.start.line + 1,
        col = range.start.character + 1,
        text = caller.name,
      })
    end
  end
  vim.fn.setqflist({}, " ", { title = "Incoming Calls", items = items })
  vim.cmd("copen")
end

vim.lsp.handlers["callHierarchy/outgoingCalls"] = function(_, result, ctx)
  if not result or vim.tbl_isempty(result) then
    vim.notify("No outgoing calls found", vim.log.levels.INFO)
    return
  end
  -- fromRanges are in the caller item file, not the callee definition file.
  local caller_uri = ctx.params and ctx.params.item and ctx.params.item.uri
  local caller_file = caller_uri and vim.uri_to_fname(caller_uri)
    or vim.api.nvim_buf_get_name(ctx.bufnr)
  local items = {}
  for _, call in ipairs(result) do
    local callee = call.to
    for _, range in ipairs(call.fromRanges or {}) do
      table.insert(items, {
        filename = caller_file,
        lnum = range.start.line + 1,
        col = range.start.character + 1,
        text = callee.name,
      })
    end
  end
  vim.fn.setqflist({}, " ", { title = "Outgoing Calls", items = items })
  vim.cmd("copen")
end
