# AGENTS.md

hob: a lightweight, scriptable CLI for AI chores. One Rust crate (`src/`) that
embeds Lua 5.4 flows (`lua/hob/`). `docs/` (commands, control, lua-api,
providers, budgets) is the spec set; `README.md` is the front door.

## Commands

- `just check` is the full gate, in fixed order: `lint typecheck size test
  budget`. Run at least the affected recipes before claiming a change is done.
- `just format` rewrites files (`cargo fmt`, then `cargo clippy --fix`).
- Focused test: `cargo test --all-features <filter>` (add `--test <file>` for
  one suite). `just test` takes no filter: extra words are read as recipe names
  and fail.
- `just bench` needs `valgrind` plus the `iai-callgrind-runner` version locked
  in `Cargo.lock`. `just env` installs it and the other tools; do not run that
  recipe unprompted, it installs into the user's toolchain over the network.
- Builds use the toolchain pinned in `rust-toolchain.toml` (1.98.1).
- `.github/workflows/` runs the same `just check` on every push; run it locally
  too, since an agent does not push.

## Hard rules

- Every tracked `.rs`/`.lua` file stays within 120 code lines (blank and comment
  lines excluded); `just size` enforces it. Split files, never raise the limit.
- Budgets in `docs/budgets.md` are ratchets: lower freely; raise only in the
  diff that earns it, with the measured number in the commit; never to silence a
  regression. `scripts/check_*.sh` are the executable truth (the heap overrides
  are `HOB_CLI_HEAP_BUDGET_KIB`/`HOB_FLOW_HEAP_BUDGET_KIB`, not the doc's
  `HOB_HEAP_BUDGET_KIB`).
- `unsafe_code` is forbidden; clippy pedantic with `-D warnings`; normal deps
  capped at 150; `openssl-sys`/`native-tls` banned (rustls only).
- Conventional Commits scoped by area: `feat(store): ...`, `test(bench): ...`,
  `docs: ...`.
- Comments are English/ASCII with a `// --- <name> ---` header (Rust) or
  `-- --- <name> ---` (Lua), and explain why, not what.

## Architecture

- `src/cli` parses and dispatches -> `src/driver` runs a flow (its gate
  implements `--dry-run`/`--step`) -> `src/effect` is the Lua<->Rust wire ->
  `src/exec` performs effects and `src/provider` speaks wire dialects.
  `src/store` resolves command names: project `.hob/commands` > user config >
  `src/builtin`. Project commands run only after `hob trust` recorded the root
  in the user-side `<config>/trust.json`; never put the trust store in a
  project.
- `src/lib.rs` exposes `provider`, `run` and a few constants tests use; the rest
  is `pub(crate)`. Tests call `hob::run` with injected writers.
- Rust owns all effects. `lua/hob/*.lua` are thin wrappers over the one
  `hob.effect` primitive, embedded with `include_str!` (`src/lua/embedded.rs`):
  edit the `.lua` files directly, there is no codegen or install step.
- Read-vs-write is classified in Rust (`src/exec/safety.rs`), so a flow cannot
  talk the engine out of refusing a write during a preview.
- New OpenAI-compatible provider: one entry in `src/provider/registry.rs` plus
  the key-variable test. New wire dialect: a `Protocol` variant and a module
  under `src/provider/wire/`. Keys only ever come from the env var a spec names.
- `docs/lua-api.md` naming rules are a contract: 3-4 letter module names, names
  within a module differ by at most one letter, wire `op` == function name. Keep
  the docs in step with behavior.

## Tests

- Integration tests run the real binary (`CARGO_BIN_EXE_hob`) via helpers in
  `tests/common/` (`Sandbox`, `Flow`, wiremock `mock`) and set `HOB_CONFIG_DIR`
  to a temp tree; never the real `~/.config/hob`.
- `tests/allocations.rs` must stay exactly one test: its counting global
  allocator is polluted by parallel tests in the same binary.
- Agent tests use wiremock and run the subprocess through `spawn_blocking`.
- `benches/` are iai-callgrind harnesses, deliberately excluded from `just test`;
  run them only via `just bench` (save a baseline with
  `cargo bench --bench startup -- --save-baseline main`).
