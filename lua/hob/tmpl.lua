-- --- hob.tmpl ---
-- Prompt templates.
--
-- `render` is pure: it substitutes `{name}` placeholders from a table and
-- raises when a name has no value, because a prompt that silently loses a
-- variable is worse than one that fails.

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

return tmpl
