-- --- hob.term ---
-- Terminal interaction for flows.
--
-- Printing is an effect like everything else, so the engine records it, and a
-- preview can skip or redirect it, rather than a flow writing straight to the
-- terminal. Styling is named rather than spelled in escape codes: the engine
-- owns the terminal and decides whether colour makes sense.

local hob = require("hob")

local term = {}

--- Print one line for the user.
--
-- `opts.style` is one of "error", "warn", "info", "success", "muted" or
-- "title". `opts.stream` is "stdout" (the default) or "stderr".
function term.print(text, opts)
  opts = opts or {}
  return hob.effect("term", "print", {
    text = text,
    style = opts.style,
    stream = opts.stream,
  })
end

return term
