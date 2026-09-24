--- Suggest and review one or more Git commits from the working tree.
--- usage: hob git-commit [model]
--- args: 0..1
-- --- git-commit ---
local git = require("gitcommit.git")
local plan = require("gitcommit.plan")

local function paths(group, rows)
  local args = {}
  for _, id in ipairs(group.ids) do
    for _, path in ipairs(rows[id].paths) do args[#args + 1] = path end
  end
  return args
end

local function with_paths(prefix, names)
  local args = {}
  for _, part in ipairs(prefix) do args[#args + 1] = part end
  args[#args + 1] = "--"
  for _, path in ipairs(names) do args[#args + 1] = path end
  return args
end

local function staged_paths_match(names)
  local raw = git.must({ "diff", "--cached", "--name-only", "--no-renames", "-z" })
  local expected, actual = {}, {}
  for _, name in ipairs(names) do expected[name] = true end
  for name in raw:gmatch("([^\0]+)\0") do actual[name] = true end
  for name in pairs(expected) do hob.assert(actual[name], "staging missed " .. name) end
  for name in pairs(actual) do hob.assert(expected[name], "unexpected staged path: " .. name) end
  hob.assert(raw ~= "", "nothing was staged")
end

local function commit(group, rows)
  local names = paths(group, rows)
  git.must(with_paths({ "add", "-A" }, names))
  staged_paths_match(names)
  local check = git.call({ "diff", "--cached", "--check" })
  if not check.ok then
    git.must(with_paths({ "reset", "-q", "HEAD" }, names))
    hob.abort("staged diff has whitespace errors: " .. check.stdout .. check.stderr)
  end
  hob.term.print(git.must({ "diff", "--cached", "--stat" }))
  if not hob.term.allow("Create this commit: " .. group.title .. "?", { default = false }) then
    git.must(with_paths({ "reset", "-q", "HEAD" }, names))
    return false
  end
  local args = { "commit", "-m", group.title }
  if group.body ~= "" then args[#args + 1] = "-m"; args[#args + 1] = group.body end
  local result = git.call(args)
  hob.assert(result.ok, "git commit failed; staged changes remain for inspection: " .. result.stderr)
  hob.term.print(result.stdout)
  return true
end

local head, rows = git.scan()
if #rows == 0 then hob.term.print("Nothing to commit."); return end
hob.term.print("Found " .. #rows .. " changed files:")
for _, row in ipairs(rows) do hob.term.print("  [" .. row.id .. "] " .. row.path) end
if not hob.term.allow("Send these diffs to the configured model?", { default = false,
  detail = "The diff may contain secrets. Declining makes no Git changes." }) then return end
local note = hob.term.input{ prompt = "Context for the commit plan (optional): ", default = "" }
local proposed
while true do
  git.unchanged(head, rows, {})
  proposed = plan.ask(rows, note, hob.args[1])
  plan.show(proposed, rows)
  local action = hob.term.input{ prompt = "[c]ommit, [r]evise, [q]uit? ", default = "q" }
  if action == "c" then break end
  if action ~= "r" then hob.term.print("Cancelled; no commits created."); return end
  local extra = hob.term.input{ prompt = "Add context or corrections: " }
  if extra ~= "" then note = note .. "\n" .. extra end
end
local done = {}
for number, group in ipairs(proposed.groups) do
  git.unchanged(head, rows, done)
  hob.term.print("Review commit " .. number .. "/" .. #proposed.groups .. ": " .. group.title)
  if not commit(group, rows) then
    hob.term.print("Stopped; earlier commits remain, uncommitted files are unchanged.")
    return
  end
  for _, id in ipairs(group.ids) do done[id] = true end
  head = git.must({ "rev-parse", "HEAD" })
end
hob.term.print("Done: " .. #proposed.groups .. " commit(s) created.")
