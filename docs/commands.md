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

## Trust

A project's commands are code, so a checkout may not run them until the user
says so once. `hob trust` records the project root in `<config>/trust.json`;
`hob trust --revoke` takes it back and `hob trust --list` prints the roots. The
store lives in the user's configuration, never in the project, so a repository
cannot bless itself; roots are canonicalized, so a command reached through a
relative path or a symlink compares equal to the one that was trusted.

Project command entry points and project libraries are gated. `hob run <file.lua>`
is an explicit decision to execute that file, not to trust the surrounding
project's libraries. The user layer and builtins are the user's own
configuration. `hob list` marks a project command `(untrusted)`, `hob doctor`
reports the current project's state, and `hob <name> --help` stays available because it reads the
header instead of running anything. A refused command exits 1 and names the
project and the command that fixes it; it does not fall back to a shadowed user
command, because running a different file under the name would be worse than
refusing.

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
`new`, `rm`, `trust`, `which`), are not commands; `hob list` reports them under
`ignored` rather than hiding them.

## Verbs

| Verb | Behavior | Exit |
| --- | --- | --- |
| `hob <name> [args...]` | run the effective command | flow's code |
| `hob run <file.lua> [args...]` | run a file, no discovery | flow's code |
| `hob list` | effective commands, origin, summary; then ignored files | 0 |
| `hob which <name>` | every layer that defines the name, effective first | 0, 1 unknown |
| `hob new <name> [--user\|--local]` | write a template; project when inside one, the user directory otherwise | 0, 1 exists, 2 bad flags |
| `hob rm <name>` | delete the effective file | 0, 1 unknown |
| `hob trust [--revoke\|--list]` | trust the current project so its commands may run | 0, 1 not a project, 2 bad flags |

`rm` only ever deletes a file the registry resolved: it takes a name, never a
path, and cannot touch a builtin. Removing a project command that shadows a
user command prints the path that becomes effective again. `list`, `which` and
`doctor` also answer as one JSON object under `--json` (`docs/control.md`).

Control flags (`--dry-run`, `--step`, `--trace`, `--timeout`, `-v`, `-q`,
`--yes`) may follow the command name; `--` ends them and hands everything after
it to the flow. They are described in `docs/control.md`.

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
`<project>/.hob/lib/util/text.lua` when the project is trusted, then to the user's
`lib/`. An untrusted project's library directory is excluded even for user
commands and explicit `hob run` flows; a matching user library remains available.
Existing search roots are authorized and canonicalized once before the flow
starts; directories created later are not added to the search. In a project, an
unreadable or malformed user trust store fails the run before executing Lua.
Trust changes take effect on the next run.

Module names are lowercase words joined by dots; a slash or `..` is not a module
name. A library root may be a symlink only when its target remains within its
owning project or user configuration directory. A module's resolved path must
remain inside that library root, including through directory symlinks. An
existing link that resolves outside this boundary, or a dangling module-file
link, fails explicitly instead of falling back to a lower-priority library.
Links within the boundary are supported. These checks do not protect against
concurrent filesystem replacement by another process.

The embedded `hob.*` modules are loaded first, so a
library adds modules rather than replacing the stdlib.

## Builtins

`doctor` is compiled into the binary. It prints the version, the configuration
and project directories, how many commands are effective in each layer, and
which provider key variables are set -- by name only, never by value. A file in
either layer shadows it like any other name, and `rm` refuses to delete it.

## Not in this layer

A compiled-command cache was measured and dropped: compiling the flow is a
rounding error next to starting the VM (about 4% of a 2.4M-instruction cold
start, `docs/budgets.md`), so a cache would add invalidation and binary-chunk
risk for a fraction of a millisecond. The registry is the filesystem; there is
no manifest.
