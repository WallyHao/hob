-- --- gitcommit.plan ---
-- Treat model output as a proposal; validate the complete file partition.
local M = {}

local schema = {
  type = "object", required = { "groups", "skip" }, additionalProperties = false,
  properties = {
    groups = { type = "array", minItems = 1, items = {
      type = "object", required = { "title", "body", "why", "ids" }, additionalProperties = false,
      properties = {
        title = { type = "string", minLength = 1, maxLength = 72 },
        body = { type = "string" }, why = { type = "string", minLength = 1 },
        ids = { type = "array", minItems = 1, items = { type = "integer", minimum = 1 } },
      },
    } },
    skip = { type = "array", items = { type = "integer", minimum = 1 } },
  },
}

local function valid(plan, rows)
  local seen = {}
  for _, group in ipairs(plan.groups) do
    hob.assert(not group.title:find("[\r\n\0]") and group.title:match("%S"), "invalid commit title")
    hob.assert(not group.body:find("\0"), "NUL in commit body")
    for _, id in ipairs(group.ids) do
      hob.assert(rows[id] and not seen[id], "duplicate or unknown file id: " .. tostring(id))
      seen[id] = true
    end
  end
  for _, id in ipairs(plan.skip) do
    hob.assert(rows[id] and not seen[id], "duplicate or unknown skipped id: " .. tostring(id))
    seen[id] = true
  end
  for id in ipairs(rows) do hob.assert(seen[id], "model omitted file id: " .. id) end
  return plan
end

function M.ask(rows, note, model)
  local parts = {
    "Plan Git commits from the changes below. Group by purpose, not file type.",
    "Use one group if the changes form one coherent change. A file id is indivisible.",
    "Use skip for files the user asks to exclude. Every id must occur exactly once",
    "across groups and skip. Do not invent paths. Write concise commit titles.",
    "User context: " .. (note ~= "" and note or "none"),
  }
  for _, row in ipairs(rows) do
    parts[#parts + 1] = "\nFILE " .. row.id .. " [" .. row.status .. "] " .. row.path .. "\n" .. row.detail
  end
  local answer = hob.agent.ask{
    model = model, prompt = table.concat(parts, "\n"), schema = schema,
    max_attempts = 2,
  }
  return valid(answer, rows)
end

function M.show(plan, rows)
  for number, group in ipairs(plan.groups) do
    hob.term.print("Commit " .. number .. ": " .. group.title)
    hob.term.print("  " .. group.why)
    for _, id in ipairs(group.ids) do hob.term.print("  [" .. id .. "] " .. rows[id].path) end
    if group.body ~= "" then hob.term.print("  Body: " .. group.body) end
  end
  if #plan.skip > 0 then
    hob.term.print("Not committing:")
    for _, id in ipairs(plan.skip) do hob.term.print("  [" .. id .. "] " .. rows[id].path) end
  end
end

return M
