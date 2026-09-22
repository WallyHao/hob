-- --- hob ---
-- The effect primitive every other module is built on.
--
-- A flow never touches the world directly. It calls `hob.effect`, which yields
-- the request to the engine and blocks until the engine resumes the coroutine
-- with the result. Because the yield happens in plain Lua, the engine needs no
-- yield machinery of its own and the whole driver stays synchronous.

local hob = {}

-- Key the engine uses to report a failure it refused to perform. Must match
-- `ABORT_KEY` in `src/effect/mod.rs`.
local ABORT_KEY = "__hob_abort"

--- Perform one effect and return its result.
--
-- A failure aborts the flow with the engine's message. Pass `fallible = true`
-- to get `nil, err` back instead, for the rare case where a flow wants to
-- survive a failure.
--
-- The engine classifies each effect for `--dry-run` and `--step`; a flow cannot
-- mark its own writes as reads.
function hob.effect(ns, op, cmd, opts)
  opts = opts or {}
  local request = { ns = ns, op = op, cmd = cmd or {} }
  if opts.fallible then
    request["try"] = true
  end
  if opts.fallible then
    -- A fallible call resumes with `(nil, message)`, so every result has to be
    -- returned rather than captured into a single value.
    return coroutine.yield(request)
  end
  local result = coroutine.yield(request)
  if type(result) == "table" and result[ABORT_KEY] ~= nil then
    error(result[ABORT_KEY], 0)
  end
  return result
end

--- End the flow with a message rather than a traceback.
function hob.abort(message, code)
  return hob.effect("term", "abort", { message = message, code = code })
end

--- Abort unless `condition` holds.
function hob.assert(condition, message)
  if not condition then
    hob.abort(message)
  end
  return condition
end

--- Arguments of the invocation, published by the engine before the body runs.
hob.args = {}

return hob
