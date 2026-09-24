# hob

A lightweight, scriptable CLI for AI chores.

hob is one Rust binary that runs Lua 5.4 flows. Rust owns every effect --
network calls, files, processes, the terminal -- so a flow cannot talk the
engine out of refusing a write: `--dry-run`, `--step`, traces and budgets are
enforced in Rust, and a command is a file rather than a plugin.

## Quick start

Rust 1.98.1, pinned in `rust-toolchain.toml`:

```sh
cargo build --release
```

A command is a Lua file. Put one in a project and let hob find it:

```sh
mkdir -p .hob/commands
cat > .hob/commands/hello.lua <<'LUA'
--- Greet someone with one model call.

local answer = hob.agent.ask{
  model = "deepseek-chat",
  prompt = "Say hello to " .. (hob.args[1] or "the world") .. " in one line.",
}
hob.term.print(answer)
LUA

hob trust          # allow this project's commands, once
hob hello wally    # DEEPSEEK_API_KEY must be set
```

`hob list` shows what is installed; `hob run flow.lua` runs a file directly;
`hob doctor` reports paths, commands and key variables.

## Example: review Git commits

This checkout includes `.hob/commands/git-commit.lua` and its modules in
`.hob/lib/gitcommit/`. From this checkout, run `hob trust` once, configure a
provider and default model as described in `docs/providers.md`, then run:

```sh
hob git-commit                 # use the configured default model
hob git-commit deepseek-chat   # or name a model explicitly
```

The command reads the working-tree diff and untracked text files, asks before
sending them to the model, and proposes one or more commits. You can add context,
revise the plan, or cancel. Every commit needs another confirmation after its
files are staged; declining leaves remaining changes uncommitted. To use it in
another project, copy the command into that project's `.hob/commands/` and the
`gitcommit/` directory into `.hob/lib/`, then trust that project.

The first version groups whole files. It requires an empty Git index and an
existing `HEAD`; it refuses conflicts, non-text or large untracked files, and
large diffs. It reads at most 40 files and 160 KB of diff content. The built-in
`--dry-run` cannot preview this command because `hob` currently treats all
subprocess calls, including `git diff`, as effects; use the command's own review
and cancellation prompts. `--yes` keeps their safe default (cancel).

## Documentation

| Subject | File |
| --- | --- |
| Commands and the three layers | `docs/commands.md` |
| Control flags, previews, traces, budgets | `docs/control.md` |
| Lua API (`agent`, `file`, `proc`, `term`, `logs`, `json`, `tmpl`) | `docs/lua-api.md` |
| Providers, dialects, keys | `docs/providers.md` |
| Size, heap and dependency gates | `docs/budgets.md` |

## Platform support

Linux only. hob is built and tested on Linux, and CI runs the full gate on
`ubuntu-latest`; there is no macOS or Windows build. The maintainer has neither
a development nor a test environment for them, so another platform would be
unverified code shipped on trust, and the time it would take is better spent on
the engine. The Unix-shaped parts (process groups, signals) are not
Linux-specific by design, but verification, not design, is what is missing.

## Development

`just check` is the gate: formatting, clippy, the 120-line source limit, the
test suite and the budget scripts. `just format` rewrites files. `just install`
builds the release binary and copies it to `$PREFIX/bin/hob` (default
`$HOME/.local/bin`, or pass a directory: `just install /usr/local`). The
benchmark recipe additionally needs `iai-callgrind-runner`, which `just env`
installs at the version locked in `Cargo.lock`.

## License

MIT, see `LICENSE`.
