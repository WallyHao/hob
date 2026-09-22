-- --- hob.agent ---
-- Model calls.
--
-- `agent.ask` is one round trip; `agent.open` returns a conversation whose
-- turns live in the engine, so history cannot be edited by accident. Tools are
-- data: `meta.tool_calls` says what the model asked for and the flow decides
-- what to run. There is no automatic loop.
--
-- Streaming, a mock provider and `thinking` are not implemented yet; passing
-- them is an error rather than a silent no-op.

local hob = require("hob")

local agent = {}
local chat = {}
chat.__index = chat

local ALLOWED = {
  provider = true,
  model = true,
  system = true,
  prompt = true,
  messages = true,
  schema = true,
  temperature = true,
  max_tokens = true,
  max_attempts = true,
  effort = true,
  tools = true,
}

local function check(opts)
  for key in pairs(opts) do
    if not ALLOWED[key] then
      error("hob.agent: unknown option `" .. key .. "`", 0)
    end
  end
end

--- One round trip. Returns `answer, meta`.
--
-- Pass `prompt` or `messages`, `system` to frame them, and `schema` to require
-- JSON of a shape: the engine then validates the answer and, up to
-- `max_attempts`, asks again. `meta` is
-- `{ provider, model, attempts, usage, tool_calls?, reasoning? }`.
function agent.ask(opts)
  opts = opts or {}
  check(opts)
  local result = hob.effect("agent", "ask", opts)
  return result.answer, result.meta
end

--- Open a conversation and return a handle.
--
-- The options are the ones `ask` takes, minus `prompt` and `messages`; they
-- become the defaults for every `send`.
function agent.open(opts)
  opts = opts or {}
  check(opts)
  local id = hob.effect("agent", "open", opts)
  return setmetatable({ id = id }, chat)
end

--- The model ids a provider lists.
function agent.list(opts)
  opts = opts or {}
  check(opts)
  return hob.effect("agent", "list", { provider = opts.provider })
end

--- One turn. Returns `answer, meta`, like `ask`.
function chat:send(opts)
  opts = opts or {}
  check(opts)
  local request = { session = self.id }
  for key, value in pairs(opts) do
    request[key] = value
  end
  local result = hob.effect("agent", "send", request)
  return result.answer, result.meta
end

--- Inject a message (a tool result, a correction) without calling the model.
function chat:push(message)
  return hob.effect("agent", "push", { session = self.id, message = message })
end

--- The committed turns, oldest first.
function chat:turns()
  return hob.effect("agent", "turns", { session = self.id })
end

--- Token accounting so far.
function chat:usage()
  return hob.effect("agent", "usage", { session = self.id })
end

--- Forget the turns; the system prompt and the accounting stay.
function chat:reset()
  return hob.effect("agent", "reset", { session = self.id })
end

--- Drop the conversation.
function chat:close()
  return hob.effect("agent", "close", { session = self.id })
end

return agent
