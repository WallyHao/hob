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
-- --- hello ---
-- Greet someone with one model call.

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

## Documentation

| Subject | File |
| --- | --- |
| Commands and the three layers | `docs/commands.md` |
| Control flags, previews, traces, budgets | `docs/control.md` |
| Lua API (`agent`, `file`, `proc`, `term`, `logs`, `json`, `tmpl`) | `docs/lua-api.md` |
| Providers, dialects, keys | `docs/providers.md` |
| Size, heap and dependency gates | `docs/budgets.md` |

## Development

`just check` is the gate: formatting, clippy, the 120-line source limit, the
test suite and the budget scripts. `just format` rewrites files. The benchmark
recipe additionally needs `iai-callgrind-runner`, which `just env` installs at
the version locked in `Cargo.lock`.

## License

MIT, see `LICENSE`.
