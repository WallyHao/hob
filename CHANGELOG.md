# Changelog

All notable changes to hob are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Commands resolved from three layers -- project `.hob/commands`, the user
  config, builtins -- with namespaces, help lines and argument ranges.
- `hob trust` and a user-side trust store: a project's commands run only after
  the root was allowed, and a shadowed user command is never used as a fallback.
- The Lua API (`hob.agent`, `hob.file`, `hob.proc`, `hob.term`, `hob.logs`,
  `hob.json`, `hob.tmpl`) over one effect bridge, with `--dry-run` and `--step`
  classified in Rust.
- Providers: an OpenAI-compatible dialect and the Anthropic Messages dialect,
  with retries, timeouts and a key read only from the provider's own variable.
- `hob doctor` and `hob list`/`which --json` for scripts.
- Control flags: `--dry-run`, `--step`, `--trace`, `--trace-full`, `--timeout`,
  `--json`, `--max-calls`, `--max-tokens`.
- Conversation prompt budgets (`max_prompt_tokens`), per-run spend caps, and
  `finish_reason`/`truncated` in every answer's `meta`.
- Streaming answers (`stream = true`): the text arrives on stderr as it is
  generated, and the assembled answer is still returned.

### Security

- Previews, logs and traces mask credential-like values; the content fields of a
  trace record sizes instead of text unless `--trace-full` is passed.
- `--max-calls`/`--max-tokens` refuse runaway spending; `file.read` is capped at
  8 MiB by default; `file.write` replaces a file atomically.
