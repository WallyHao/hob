-- --- hob.tmpl ---
-- Prompt templates.
--
-- `render` is pure: it substitutes `{name}` placeholders from a table and
-- raises when a name has no value, because a prompt that silently loses a
-- variable is worse than one that fails.

local hob = require("hob")

local tmpl = {}

--- Substitute `{name}` placeholders in `text` from `vars`.
function tmpl.render(text, vars)
  vars = vars or {}
  return (text:gsub("{([%w_]+)}", function(name)
    local value = vars[name]
    if value == nil then
      error("tmpl.render: no value for {" .. name .. "}", 0)
    end
    return tostring(value)
  end))
end

--- Read a prompt template by name.
--
-- Names resolve against the project's `.hob/prompts/`, then the user's
-- `prompts/` directory, so a checkout can ship its own prompts and still fall
-- back to the ones kept next to the configuration.
function tmpl.fetch(name)
  return hob.effect("tmpl", "fetch", { name = name })
end

return tmpl
