-- --- hob.file ---
-- File access for flows.
--
-- A flow cannot reach a file any other way: `io` is gone from the sandbox and
-- every call here is an effect. That is what makes a preview able to say what a
-- flow would touch before it touches it.

local hob = require("hob")

local file = {}

--- Read a UTF-8 file.
--
-- `opts.optional` returns `nil` when the file does not exist instead of
-- failing, for the common "read it if it is there" case.
function file.read(path, opts)
  opts = opts or {}
  return hob.effect("file", "read", { path = path, optional = opts.optional })
end

--- Write text, creating parent directories as needed.
--
-- `opts.append` adds to the end of the file instead of replacing it.
function file.write(path, text, opts)
  opts = opts or {}
  return hob.effect("file", "write", {
    path = path,
    text = text,
    append = opts.append,
  })
end

--- Size, mtime and kind of a path, or `nil` when it does not exist.
--
-- `kind` is "file", "dir" or "link"; `mtime` is seconds since the epoch.
function file.stat(path)
  return hob.effect("file", "stat", { path = path })
end

--- Entry names directly under a directory, sorted, not recursive.
function file.list(path)
  return hob.effect("file", "list", { path = path })
end

return file
