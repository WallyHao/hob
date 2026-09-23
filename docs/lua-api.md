# Lua API

Status: implemented, except where a section says otherwise. The names and
rules below are fixed; each module's section says what exists today.

## The layer model

Two layers, one bridge:

- **Rust owns effects**: anything that touches the world (network, files,
  processes, terminal) and anything that must not be reimplemented in Lua
  (schema validation, JSON, history trimming).
- **Lua owns the API**: modules under `lua/hob/` are thin wrappers that
  validate options and call one primitive, `hob.effect`. A flow never sees the
  wire format.

Pure helpers (`hob.json`, `hob.tmpl.render`) are bound directly or written in
Lua; they do not pay a coroutine round trip and need no trace or preview
semantics. Everything with a side effect goes through the bridge.

## Naming rules

1. Module names are lowercase, singular, 3-4 letters, no underscores; full
   words except the established contractions `proc` and `tmpl` and the acronym
   `json`.
2. Within a module, or within one object's methods, names differ in length by
   at most one letter.
3. The same verb means the same thing across modules: `open` constructs,
   `list` enumerates, `read`/`write` move text, `send` is one conversation
   turn, `reset`/`close` are lifecycle.
4. The wire `op` is the function name, so there is no second naming scheme.

## Modules

| Module | Meaning | Length |
| --- | --- | --- |
| `hob.agent` | model calls and conversations | 5 |
| `hob.file` | file access | 4 |
| `hob.proc` | subprocesses and sessions | 4 |
| `hob.term` | terminal interaction | 4 |
| `hob.logs` | diagnostics | 4 |
| `hob.json` | encode and decode | 4 |
| `hob.tmpl` | prompt templates | 4 |

## The bridge

`hob.effect(ns, op, cmd, opts) -> result`

- `opts.fallible = true` returns `nil, message` instead of aborting.
- An unknown `ns` or `op` is an error, never a silent no-op.

The engine classifies every operation itself (`docs/control.md`), so `--dry-run`
and `--step` cannot be talked out of refusing a write by a flow.

`hob.abort(message, code?)` ends the flow without a traceback.
`hob.assert(condition, message)` is `abort` when the condition is falsy.
`hob.args` is the invocation's arguments, `hob.command` its name; both are
published by the engine before the body runs.

## Core (implemented)

| Function | Length | What it does |
| --- | --- | --- |
| `hob.effect` | 6 | perform one effect |
| `hob.assert` | 6 | abort unless the condition holds |
| `hob.abort` | 5 | end the flow with a message and code |

## `hob.agent` (implemented)

| Function | Length | What it does |
| --- | --- | --- |
| `agent.ask{...}` | 3 | one round trip; `prompt` or `messages`, `system`, `schema`, `provider`, `model`, `temperature`, `max_tokens`, `max_attempts`, `max_prompt_tokens`, `stream`, `effort`, `tools`; returns `answer, meta` |
| `agent.open{...}` | 4 | open a conversation; returns the chat handle |
| `agent.list{...}` | 4 | model ids a provider lists |

`provider` is a registry id (`"deepseek"`), a name from `config.toml`, or an
inline table `{ id, base_url, api_key_env, protocol, headers }` for a local
gateway; the key still comes from the environment. `model` is required: ids
change, so the engine does not guess one. A `schema` makes the answer JSON that
is validated in Rust and, up to `max_attempts`, asked for again; the validator
enforces `type`, `const`, `enum`, `allOf`/`anyOf`/`oneOf`, `required`,
`properties`, `additionalProperties`, `items`, `minItems`/`maxItems`,
`minimum`/`maximum`/`exclusiveMinimum`/`exclusiveMaximum` and
`minLength`/`maxLength`, and ignores the rest (`pattern` and tuple-form `items`
among them), so a schema using those is weaker than it looks. `effort` is
forwarded as `reasoning_effort`, which the Anthropic dialect refuses because it
has no equivalent.

| Chat handle | Length | What it does |
| --- | --- | --- |
| `:send{...}` | 4 | one turn in the conversation |
| `:push(message)` | 4 | inject a message (tool result, correction) |
| `:turns()` | 5 | committed turns, oldest first |
| `:usage()` | 5 | token accounting |
| `:reset()` | 5 | forget the turns, keep the system prompt |
| `:close()` | 5 | release the handle |

`stream = true` makes the answer arrive as events: the text is shown on stderr
while it is generated, and the assembled answer and `meta` are still returned
whole, so a flow does not branch on how the answer travelled. Streaming cannot
be combined with `tools` yet: a streamed tool call cannot be assembled, and the
call is refused rather than silently dropping it.

`max_prompt_tokens` caps what a conversation sends: before a `:send`, the oldest
exchanges are dropped until the rest is estimated under the cap, and the newest
exchange is never dropped. `0` turns the cap off; the default is 32000. A send
reports how many messages went in `meta.trimmed`, and the model is told with a
marker where the history starts. `agent.ask` accepts the setting but has no
history to trim.

`meta` is `{provider, model, attempts, usage, finish_reason?, truncated?,
tool_calls?, reasoning?, trimmed?}`. `finish_reason` is the provider's own word
for why the model stopped, and `truncated` is true when that reason means the
output cap was reached, so a flow can tell a finished answer from a cut one
without knowing each dialect. `usage` sums every attempt, repaired answers
included. Tools
are data: the response carries `tool_calls` and the flow decides what to run;
there is no automatic loop. Each entry is `{id, name, arguments}`, with
`arguments` the raw JSON string the model produced.

## `hob.file` (implemented)

| Function | Length | What it does |
| --- | --- | --- |
| `file.read(path, opts?)` | 4 | UTF-8 read; `{optional = true}` gives `nil` when absent |
| `file.write(path, text, opts?)` | 5 | replace, creating parents; `{append = true}` appends |
| `file.stat(path)` | 4 | `{size, mtime, kind}` or `nil`; `kind` is `"file"`, `"dir"` or `"link"` |
| `file.list(path)` | 4 | entry names, not recursive, sorted |

## `hob.proc` (implemented)

| Function | Length | What it does |
| --- | --- | --- |
| `proc.open(opts?)` | 4 | open a session: `cwd`, `env`, `profile`, `env_clear` |
| `proc.exec(argv, opts?)` | 4 | run one program; `{inherit = true}` hands over the terminal |
| `proc.shell(line, opts?)` | 5 | run one shell line through `sh -c` |
| `proc.which(prog)` | 5 | resolve a program on `PATH` |

| Session | Length | What it does |
| --- | --- | --- |
| `:exec(argv, opts?)` | 4 | run a program; `stdin`, `timeout_ms`, `trim`, `inherit`, `env_clear` |
| `:shell(line, opts?)` | 5 | run a shell line |
| `:setenv(name, value)` | 6 | add or replace one environment override |
| `:unset(name)` | 5 | remove one override |
| `:chdir(path)` | 5 | move the session |
| `:setup(text)` | 5 | replace the profile sourced before each command |
| `:state()` | 5 | `{cwd, env, env_clear}` |
| `:reset()` | 5 | restore the opening context |
| `:close()` | 5 | drop the session |

`exec` results are `{code, stdout, stderr, ok, duration_ms, truncated}`.
`timeout_ms` kills the child and reports code 124, as `timeout(1)` does; each
stream keeps its first megabyte, and `truncated` says when more arrived.
`env_clear` starts a command with an empty environment, so a flow can keep the
process's secrets out of what it runs; the session's `env` overrides still
apply on top of it.
`profile` is shell text evaluated before each shell line, not before `exec`.
Every command becomes the head of its own process group, so a timeout or Ctrl-C
kills the tree it started rather than leaving orphans (`docs/control.md`).

## `hob.term` (implemented)

| Function | Length | What it does |
| --- | --- | --- |
| `term.print(text, opts?)` | 5 | one line; `style`, `stream` |
| `term.input{...}` | 5 | one line of input; `prompt`, `default`, `initial` |
| `term.allow(prompt, opts?)` | 5 | yes/no; `default`, `detail` |
| `term.select{...}` | 6 | pick one label; `prompt`, `options`, `default` |
| `term.choose{...}` | 6 | pick labels; `defaults`, `min`, `max` |

## `hob.logs` (implemented)

`trace` (5), `debug` (5), `info` (4), `warn` (4), `error` (5); `info` and up
are visible by default, `debug` needs `-v` and `trace` needs `-vv`.

## `hob.json` (implemented)

`encode` (6) turns a Lua value into a string; `decode` (6) turns a string into
a Lua value and raises on malformed input (`pcall` to catch). The hand-written
bridge keeps `nil` and empty tables meaningful instead of exposing mlua's
serde quirks.

## `hob.tmpl` (implemented)

`render(text, vars)` (6) substitutes `{name}` placeholders.
`fetch(name)` (5) resolves a template from the project `.hob/prompts/`, then
the user's `~/.config/hob/prompts/`. Built-in templates are reserved but empty
today, and a name that is absolute or climbs out of the directory is refused.

## Sandbox

`io`, `os`, `debug`, `loadfile`, `dofile`, `load`, `print` and `warn` are
unreachable; `package.searchers` keeps only the preload searcher plus the
library searcher, so `require` returns an embedded module or a file from
`.hob/lib` (then the user's `lib/`) and nothing else — a module name is not a
path (`docs/commands.md`). Flows are trusted local code, but a flow that cannot
read a file except through `hob.file` is what makes `--dry-run` meaningful.

## LangChain, kept and dropped

| LangChain | hob | Decision |
| --- | --- | --- |
| `ChatModel.invoke` | `agent.ask` | adopt, in the product's voice |
| `.stream()` | `stream` / `on_delta` | adopt as options |
| `.batch()` | -- | defer: the engine is single-threaded |
| `bind_tools` / ToolNode | `tools` + `meta.tool_calls` | adopt the data, reject the loop |
| `with_structured_output` | `schema` | adopt; validation and repair in Rust |
| `ChatMessageHistory` | `agent.open` + `:push` | adopt as an explicit object |
| `PromptTemplate` | `tmpl.render` | adopt as a pure function |
| `JsonOutputParser` | `json.decode` | adopt |
| Runnable / LCEL | Lua itself | reject |
| AgentExecutor | -- | reject: hob is not an autonomous runtime |
| VectorStore / Retriever | -- | defer: DeepSeek has no embeddings endpoint |
| Callbacks / LangSmith | `--trace` + `logs` | adopt as engine-level |
| `set_llm_cache` | -- | defer |

## Deferred

`file.glob`, `file.remove`, `term.edit`, `proc` streaming, `agent` streaming to
a callback (`on_delta`; today the text goes to stderr), streaming with `tools`,
`thinking` and `show_reasoning`, the mock provider, time helpers, raw HTTP,
caching, retrieval.
