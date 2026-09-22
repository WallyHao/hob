# Control

The flags that shape a run rather than name a command. They may appear before or
after the command name, because `hob git-commit --dry-run` is how the flag is
naturally typed; everything after `--` goes to the flow untouched, so a command
that has a flag of its own can still receive it.

| Flag | What it does |
| --- | --- |
| `--dry-run` | Perform reads, refuse everything else, print what was refused |
| `--step` | Ask before every effect |
| `--trace FILE` | Write a JSONL record of every effect (overwrites) |
| `-v`, `-vv`, `-vvv` | Show `debug`, then `trace` log lines |
| `-q`, `--quiet` | Only warnings and errors |
| `-y`, `--yes` | Answer every question with its default |
| `--` | Everything after it belongs to the flow |

`--dry-run` and `--step` cannot be combined; `-q` and `-v` cannot be combined.
An unknown flag before the command name is a usage error (exit 2).

## What a preview performs

Every effect is classified by the engine, not by the flow: a flow cannot mark its
own writes as reads. `--dry-run` performs the effects that only observe or touch
in-memory state — file and template reads, `proc.which`, session bookkeeping,
`agent` history, `term.print`, `logs` — and refuses the rest.

A refused effect resumes the flow with the conservative answer, shaped like a
success so the preview can walk the whole path:

| Effect | Stand-in value |
| --- | --- |
| `file.write` | `nil` |
| `proc.exec` / `proc.shell` | `{ code = 0, stdout = "", stderr = "", ok = true, duration_ms = 0, truncated = false }` |
| `agent.ask` / `:send` with a schema | the schema's smallest valid example, with `meta.attempts = 0` |
| `agent.ask` / `:send` without a schema | `""` |
| `term.input` | `default` or `""` |
| `term.allow` | `default` or `false` |
| `term.select` / `term.choose` | the default labels' values |

A flow that branches on one of these may take a surprising path, which is why
every refusal is printed as `dry-run: <effect>`.

## Step

`--step` prints the effect and asks `perform? [Y/n]` on stderr. An empty line
answers yes; `n` refuses it, so the effect is skipped with the stand-in value
above and the line `declined: <effect>` is printed; end of input fails the run
instead of answering. `--yes` skips the questions and performs everything, which
is what makes a step run usable in a pipeline.

## What is printed

One line per effect, never the wire arguments: a prompt is reported as a size and
a command is clipped. Values of environment variables named like credentials
(`*_KEY`, `*_TOKEN`, `*_SECRET`, `*_PASSWORD`, `*_CREDENTIAL`) are masked in
that line, so a preview cannot print a key. A flow cannot read the environment,
but a value it obtained another way may still be one.

## Trace

`--trace FILE` writes one JSON object per line for every effect: a `request`
record carrying the arguments, then an `outcome` record with `ok`, `error`,
`dry-run` or `declined`, plus the error message when there was one. Every line
carries `t`, milliseconds since the run started, so the shape of a slow flow is
visible without wall-clock timestamps.

The file is created fresh per run: appending would mix runs without a boundary
between them. Arguments are recorded, with the same secret masking a preview
applies, so a key cannot end up at rest in the trace. A write failure mid-run
prints a warning and abandons the trace rather than failing the flow; a trace
file that cannot be created at all fails the run before it starts.
