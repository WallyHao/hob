# Commands

A command is a Lua flow the CLI can run by name. `hob run path/to/file.lua`
stays available for files and CI; a bare word is looked up in the registry.

## The registry

| Layer | Location | Precedence |
| --- | --- | --- |
| project | `<project>/.hob/commands/*.lua` | highest |
| user | `$HOB_CONFIG_DIR`, else `$XDG_CONFIG_HOME/hob`, else `~/.config/hob`, plus `/commands/*.lua` | lower |
| builtin | compiled into the binary | lowest |

The project root is the nearest ancestor of the working directory holding
`.hob` or `.git`, so a command works from any subdirectory of a checkout. A
name defined in two layers is not merged: the project file wins and the user
file is shadowed, which `hob list` and `hob which` show.

`HOB_CONFIG_DIR` exists so tests and throwaway profiles can run against a
temporary configuration.

## Loading

Discovery never executes a command. A file contributes a name, a path and its
`---` header: the first line that is not a known key is the summary `hob list`
shows, and `--- usage:` and `--- args:` say how a call should look. The source is
read only when the command runs, and it runs through the same driver as
`hob run`: the flow sees `hob.command` (the name) and `hob.args` (everything
after the name, flags included).

Names are `[a-z][a-z0-9-]*` segments joined by `/`, at most 32 characters in
total: `foo/bar.lua` is the command `foo/bar`, so a subdirectory is a namespace
rather than a second naming scheme. A symlinked directory is not searched, so
discovery cannot loop; a symlinked file is a command like any other. Files whose
names do not match, or that take a reserved verb as their name (`run`, `list`,
`new`, `rm`, `which`), are not commands; `hob list` reports them under `ignored`
rather than hiding them.

## Verbs

| Verb | Behavior | Exit |
| --- | --- | --- |
| `hob <name> [args...]` | run the effective command | flow's code |
| `hob run <file.lua> [args...]` | run a file, no discovery | flow's code |
| `hob list` | effective commands, origin, summary; then ignored files | 0 |
| `hob which <name>` | every layer that defines the name, effective first | 0, 1 unknown |
| `hob new <name> [--user\|--local]` | write a template; project when inside one, the user directory otherwise | 0, 1 exists, 2 bad flags |
| `hob rm <name>` | delete the effective file | 0, 1 unknown |

`rm` only ever deletes a file the registry resolved: it takes a name, never a
path, and cannot touch a builtin. Removing a project command that shadows a
user command prints the path that becomes effective again.

Control flags (`--dry-run`, `--step`, `-v`, `-q`, `--yes`) may follow the
command name; `--` ends them and hands everything after it to the flow. They are
described in `docs/control.md`.

Unknown names exit 2 with a nearest-name suggestion (edit distance at most 2),
or a `hob run` hint when the word looks like a path.

## Help and arguments

`hob <name> --help` prints what the header says -- summary, usage line,
argument range and path -- and does not run the flow. `--help` is the command's
own only when it is the only argument after the name, so a flow that takes a
`--help` of its own still receives it.

The header may also declare how many arguments a call takes: `--- args: 1`,
`--- args: 2+` or `--- args: 0..2`. A call outside the range exits 2 with the
usage line, and an unparseable `args:` line is ignored rather than guessed at.

## Shared libraries

A command may `require` a shared module: `require("util.text")` resolves to
`<project>/.hob/lib/util/text.lua`, then to the user's `lib/`, so a project
library wins the way a project command does. Module names are lowercase words
joined by dots; a slash or `..` is not a module name, so a library cannot climb
out of its directory. The embedded `hob.*` modules are loaded first, so a
library adds modules rather than replacing the stdlib.

## Builtins

`doctor` is compiled into the binary. It prints the version, the configuration
and project directories, how many commands are effective in each layer, and
which provider key variables are set -- by name only, never by value. A file in
either layer shadows it like any other name, and `rm` refuses to delete it.

## Not in this layer

Deferred until a real flow needs it: a compiled-command cache. The registry is
the filesystem; there is no manifest.
