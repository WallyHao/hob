-- --- hob.logs ---
-- Diagnostics for flows.
--
-- Lines go through the engine so they stay ordered with the rest of the flow
-- and can appear in a trace. `info` and up are visible by default; `debug` and
-- `trace` need the engine's higher verbosity levels.

local hob = require("hob")

local logs = {}

local function emit(level, message)
  return hob.effect("logs", "write", { level = level, msg = message })
end

--- Fine-grained tracing.
function logs.trace(message)
  return emit("trace", message)
end

--- Developer-facing detail.
function logs.debug(message)
  return emit("debug", message)
end

--- Progress worth reading.
function logs.info(message)
  return emit("info", message)
end

--- Something recoverable went wrong.
function logs.warn(message)
  return emit("warn", message)
end

--- Something failed.
function logs.error(message)
  return emit("error", message)
end

return logs
