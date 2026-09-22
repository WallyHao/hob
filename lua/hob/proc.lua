-- --- hob.proc ---
-- Subprocesses for flows.
--
-- One-shot calls run a command in a fresh context; `proc.open` returns a
-- session that keeps its working directory, environment overrides and profile
-- between commands, which is what a sequence of commands in one checkout
-- wants.
--
-- The result of a command is
-- `{ code, stdout, stderr, ok, duration_ms, truncated }`.

local hob = require("hob")

local proc = {}
local session = {}
session.__index = session

local ALLOWED = {
  open = { cwd = true, env = true, profile = true },
  exec = { inherit = true, stdin = true, timeout_ms = true, trim = true },
  shell = { inherit = true, stdin = true, timeout_ms = true, trim = true },
}

local function check(op, opts)
  for key in pairs(opts) do
    if not ALLOWED[op][key] then
      error("hob.proc." .. op .. ": unknown option `" .. key .. "`", 0)
    end
  end
end

local function options(opts)
  opts = opts or {}
  return {
    inherit = opts.inherit,
    stdin = opts.stdin,
    timeout_ms = opts.timeout_ms,
    trim = opts.trim,
  }
end

--- Run one program and return its result.
--
-- `opts.inherit` hands the terminal over instead of capturing output;
-- `opts.stdin` writes text to the child; `opts.timeout_ms` kills a child that
-- runs too long, and the result then carries code 124.
function proc.exec(argv, opts)
  check("exec", opts or {})
  local request = options(opts)
  request.argv = argv
  return hob.effect("proc", "exec", request)
end

--- Run one shell line through `sh -c`.
function proc.shell(line, opts)
  check("shell", opts or {})
  local request = options(opts)
  request.line = line
  return hob.effect("proc", "shell", request)
end

--- Resolve a program on `PATH`, or nil.
function proc.which(prog)
  return hob.effect("proc", "which", { prog = prog })
end

--- Open a session: `opts.cwd`, `opts.env`, `opts.profile`.
function proc.open(opts)
  opts = opts or {}
  check("open", opts)
  local id = hob.effect("proc", "open", {
    cwd = opts.cwd,
    env = opts.env,
    profile = opts.profile,
  })
  return setmetatable({ id = id }, session)
end

function session:exec(argv, opts)
  check("exec", opts or {})
  local request = options(opts)
  request.session = self.id
  request.argv = argv
  return hob.effect("proc", "exec", request)
end

function session:shell(line, opts)
  check("shell", opts or {})
  local request = options(opts)
  request.session = self.id
  request.line = line
  return hob.effect("proc", "shell", request)
end

function session:setenv(name, value)
  return hob.effect("proc", "setenv", { session = self.id, name = name, value = value })
end

function session:unset(name)
  return hob.effect("proc", "unset", { session = self.id, name = name })
end

function session:chdir(path)
  return hob.effect("proc", "chdir", { session = self.id, path = path })
end

function session:setup(text)
  return hob.effect("proc", "setup", { session = self.id, text = text })
end

function session:state()
  return hob.effect("proc", "state", { session = self.id })
end

function session:reset()
  return hob.effect("proc", "reset", { session = self.id })
end

function session:close()
  return hob.effect("proc", "close", { session = self.id })
end

return proc
