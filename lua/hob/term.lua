-- --- hob.term ---
-- Terminal interaction for flows.
--
-- Everything here is an effect, so the engine records it, and a preview can
-- skip or redirect it, rather than a flow writing straight to the terminal.
-- Styling is named rather than spelled in escape codes: the engine owns the
-- terminal and decides whether colour makes sense.
--
-- Questions are written to stderr and answers are read from stdin, so stdout
-- stays the data a caller can pipe.

local hob = require("hob")

local term = {}

-- One table per function so a typo is an error here rather than a silently
-- ignored key that reaches the engine.
local ALLOWED = {
  print = { style = true, stream = true },
  input = { prompt = true, default = true, initial = true },
  allow = { prompt = true, default = true, detail = true },
  select = { prompt = true, options = true, default = true },
  choose = { prompt = true, options = true, defaults = true, min = true, max = true },
}

local function check(op, opts)
  for key in pairs(opts) do
    if not ALLOWED[op][key] then
      error("hob.term." .. op .. ": unknown option `" .. key .. "`", 0)
    end
  end
end

--- Print one line for the user.
--
-- `opts.style` is one of "error", "warn", "info", "success", "muted" or
-- "title". `opts.stream` is "stdout" (the default) or "stderr".
function term.print(text, opts)
  opts = opts or {}
  check("print", opts)
  return hob.effect("term", "print", {
    text = text,
    style = opts.style,
    stream = opts.stream,
  })
end

--- Ask for one line of input.
--
-- `opts.prompt` is the question, `opts.default` the answer used when the line
-- is empty. `opts.initial` is accepted as a second default; hob has no line
-- editor to pre-fill.
function term.input(opts)
  opts = opts or {}
  check("input", opts)
  return hob.effect("term", "input", {
    prompt = opts.prompt,
    default = opts.default,
    initial = opts.initial,
  })
end

--- Ask a yes/no question; an empty line takes `opts.default` (false without).
--
-- `opts.detail` is printed before the question, for the text a confirmation
-- should show: what is about to happen, not just "are you sure".
function term.allow(prompt, opts)
  opts = opts or {}
  check("allow", opts)
  return hob.effect("term", "allow", {
    prompt = prompt,
    default = opts.default,
    detail = opts.detail,
  })
end

--- Pick one option from a numbered list.
--
-- `opts.options` holds labels, or tables `{ label = ..., value = ...}` when the
-- label shown is not the value wanted. `opts.default` is the label used when
-- the line is empty.
function term.select(opts)
  opts = opts or {}
  check("select", opts)
  return hob.effect("term", "select", {
    prompt = opts.prompt,
    options = opts.options,
    default = opts.default,
  })
end

--- Pick several options from a numbered list.
--
-- `opts.defaults` are the labels used when the line is empty; `opts.min` and
-- `opts.max` bound how many picks are accepted.
function term.choose(opts)
  opts = opts or {}
  check("choose", opts)
  return hob.effect("term", "choose", {
    prompt = opts.prompt,
    options = opts.options,
    defaults = opts.defaults,
    min = opts.min,
    max = opts.max,
  })
end

return term
