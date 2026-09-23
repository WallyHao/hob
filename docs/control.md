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
| `--trace-full` | With `--trace`, record content fields as they are |
| `--timeout SECS` | Stop the whole run after SECS seconds, with exit 124 |
| `--json` | Print `list`, `which` and `doctor` as one JSON object; other verbs ignore it |
| `--max-calls N` | Refuse the run's N+1'th provider call, model listings included |
| `--max-tokens N` | Refuse a provider call once N reported tokens were spent |
| `--color WHEN` | `term.print` styling: `auto` (terminal only), `always`, `never` |
| `-v`, `-vv` | Show `debug` log lines, then `trace` ones; more `v`s mean `-vv` |
| `-q`, `--quiet` | Only warnings and errors |
| `-y`, `--yes` | Answer every question with its default |
| `--` | Everything after it belongs to the flow |

`--dry-run` and `--step` cannot be combined; `-q` and `-v` cannot be combined.
An unknown flag before the command name is a usage error (exit 2).

`--json` shapes the verbs that describe the installation: `list`, `which`,
`doctor` and `trust --list` print one object instead of text. The success output
of the other verbs ignores the flag. Once the command name is parsed, a failure
is one `{"error": "..."}` object on stderr instead of an `error:` line, so a
script reads JSON from stdout or stderr and nothing else; a usage error raised
before the command name (the flags themselves did not parse) stays text.

`--color auto` -- the default -- spells a style only when the line's own stream
is a terminal, and honours `NO_COLOR` (set to anything: no colour) and
`CLICOLOR_FORCE` (set, and not `0`: colour even into a pipe). The flag wins over
both: `--color always` colours a pipe, `--color never` stays plain. A flow names
a style, never an escape code, so a command that pipes its output somewhere else
is not the one that has to think about it.

## Budgets

`--max-calls N` and `--max-tokens N` cap what a run spends on model calls, so a
flow that loops cannot quietly turn into a bill. One call is one HTTP request to
a provider, retries and model listings included. Tokens are the `total_tokens`
sum of every answer: a call can overshoot the token cap, since only its answer
says what it cost, but the next one is refused. Both default to no cap; `0`
means the same. A refusal is an ordinary failure (exit 1) that names the cap and
the flag that raises it.

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
| `agent.ask` / `:send` with a schema | the schema's smallest valid example (`minLength`, `minItems` and `minimum` respected), with `meta.attempts = 0` |
| `agent.ask` / `:send` without a schema | `""` |
| `agent.list` | `{}` (no models) |
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
`dry-run` or `declined`, plus the error message when there was one. Request and
outcome records share an `id`; every line also carries `t`, milliseconds since
the run started, so the shape of a slow flow is visible without wall-clock
timestamps.

The file is created fresh per run: appending would mix runs without a boundary
between them. Arguments and errors are recorded after the same secret masking a
preview applies, so a key cannot end up at rest in the trace. Fields that carry
user content — file bodies, process input, model prompts and messages, injected
chat turns and log text — are recorded as their size, so a trace can be kept or
shared without the data in it. `--trace-full` records them as they are while
still masking known credentials. A write
failure mid-run prints a warning and abandons the trace rather than failing the
flow; a trace file that cannot be created at all fails the run before it starts.

## Interrupt

Ctrl-C ends the run immediately. The handler lives on its own thread, so it works
while the main thread is blocked in a read, a wait or a request; it first kills
every live process group with `SIGKILL`, then exits 130. `SIGTERM` takes the
same path and exits 143, and `--timeout SECS` is the same kill on a timer,
exiting 124 the way `timeout(1)` does. Commands run in their
own process group, so a command that started a tree (`sh -c 'long & wait'`) is
killed whole and cannot leave orphans behind, and the same group kill backs
`timeout_ms`. A trace is already flushed line by line, so an interrupt does not
lose it.
