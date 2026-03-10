-- Custom call hierarchy handlers: show target definition location
-- instead of the default fromRanges (call-site in the current file).
vim.lsp.handlers["callHierarchy/incomingCalls"] = function(_, result, ctx)
  if not result or vim.tbl_isempty(result) then
    vim.notify("No incoming calls found", vim.log.levels.INFO)
    return
  end
  local items = {}
  for _, call in ipairs(result) do
    local item = call.from
    local range = item.selectionRange
    table.insert(items, {
      filename = vim.uri_to_fname(item.uri),
      lnum = range.start.line + 1,
      col = range.start.character + 1,
      text = item.name .. (item.detail and (" " .. item.detail) or ""),
    })
  end
  vim.fn.setqflist({}, " ", { title = "Incoming Calls", items = items })
  vim.cmd("copen")
end

vim.lsp.handlers["callHierarchy/outgoingCalls"] = function(_, result, ctx)
  if not result or vim.tbl_isempty(result) then
    vim.notify("No outgoing calls found", vim.log.levels.INFO)
    return
  end
  local items = {}
  for _, call in ipairs(result) do
    local item = call.to
    local range = item.selectionRange
    table.insert(items, {
      filename = vim.uri_to_fname(item.uri),
      lnum = range.start.line + 1,
      col = range.start.character + 1,
      text = item.name .. (item.detail and (" " .. item.detail) or ""),
    })
  end
  vim.fn.setqflist({}, " ", { title = "Outgoing Calls", items = items })
  vim.cmd("copen")
end
