-- --- gitcommit.git ---
-- Read a bounded, exact picture of the working tree before planning commits.
local M = {}

function M.call(args)
  local argv = { "git" }
  for _, arg in ipairs(args) do argv[#argv + 1] = arg end
  local result = hob.proc.exec(argv, { trim = false })
  hob.assert(not result.truncated, "Git output exceeded hob's capture limit")
  return result
end

function M.must(args)
  local result = M.call(args)
  hob.assert(result.ok, "git " .. args[1] .. ": " .. result.stderr)
  return result.stdout
end

local function entry(status, path, old)
  hob.assert(not path:find("[\r\n]"), "paths with newlines are not supported")
  local paths = { path }
  if old then paths[#paths + 1] = old end
  local item = { status = status, path = path, paths = paths }
  if status == "??" then
    local stat = hob.file.stat(path)
    hob.assert(stat and stat.kind == "file" and stat.size <= 16384,
      "untracked file needs manual handling: " .. path)
    local ok, body = pcall(hob.file.read, path)
    hob.assert(ok, "untracked file is not readable UTF-8: " .. path)
    item.content = body
    item.signature = status .. path .. M.must({ "hash-object", "--", path })
    item.detail = "untracked " .. path .. "\n" .. body
  else
    local args = { "diff", "--no-ext-diff", "--binary", "--" }
    for _, name in ipairs(paths) do args[#args + 1] = name end
    local diff = M.must(args)
    hob.assert(#diff <= 32768, "diff too large for one file: " .. path)
    item.signature = status .. path .. (old or "") .. diff
    item.detail = diff
  end
  return item
end

function M.scan()
  hob.assert(M.must({ "rev-parse", "--is-inside-work-tree" }):match("true"), "not a Git work tree")
  local head = M.must({ "rev-parse", "HEAD" })
  local raw = M.must({ "status", "--porcelain=v1", "-z", "--untracked-files=all" })
  local rows, pos, total = {}, 1, 0
  while pos <= #raw do
    local stop = raw:find("\0", pos, true)
    hob.assert(stop, "malformed git status output")
    local record = raw:sub(pos, stop - 1)
    pos = stop + 1
    local status, path = record:sub(1, 2), record:sub(4)
    hob.assert(status:sub(1, 1) == " " or status == "??", "staged or conflicted changes exist; handle them first")
    hob.assert(status ~= "!!" and path ~= "", "unsupported git status entry")
    local old
    if status:find("[RC]") then
      local next_stop = raw:find("\0", pos, true)
      hob.assert(next_stop, "malformed rename in git status")
      old = raw:sub(pos, next_stop - 1)
      pos = next_stop + 1
    end
    local row = entry(status, path, old)
    total = total + #row.detail
    hob.assert(#rows < 40 and total <= 160000, "diff too large; split the work before running git-commit")
    row.id = #rows + 1
    rows[#rows + 1] = row
  end
  return head, rows
end

function M.unchanged(head, original, done)
  hob.assert(M.must({ "rev-parse", "HEAD" }) == head, "HEAD changed while planning; stop and rerun")
  local _, current = M.scan()
  local expected = {}
  for _, row in ipairs(original) do
    if not done[row.id] then expected[row.path] = row.signature end
  end
  hob.assert(#current == (function() local n = 0; for _ in pairs(expected) do n = n + 1 end; return n end)(),
    "working tree changed while planning; stop and rerun")
  for _, row in ipairs(current) do
    hob.assert(expected[row.path] == row.signature, "working tree changed: " .. row.path)
  end
end

return M
