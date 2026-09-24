# Providers

## The shape of a provider

A provider is data, not code: `src/provider/spec.rs` holds `ProviderSpec`
(id, base URL, key variable, protocol), and `src/provider/registry.rs` lists
the built-ins. DeepSeek is the only built-in today.

Adding an OpenAI-compatible provider (Moonshot, Qwen, Zhipu, OpenRouter,
SiliconFlow, a local gateway) is one entry in `registry.rs` plus a test that
the entry names its own key variable. No client code changes, because they all
speak the same chat-completions dialect.

A provider with its own dialect is a new `Protocol` variant plus a module under
`src/provider/wire/`. Anthropic Messages is the second dialect and the first
user of that seam.

## Dialects

`protocol` names the wire dialect: `openai` (the default) or `anthropic`. The
translation lives in `src/provider/wire/`, one module per dialect, so the client
stays HTTP and a third dialect is another module and a match arm.

The Messages dialect differs where the formats do: the system prompt is its own
field, content is a list of typed blocks, and a tool call carries its arguments
as an object rather than a JSON string. `max_tokens` is required by Anthropic,
so a request without one uses 4096; `effort` has no equivalent and is refused
rather than dropped. Tools are accepted in the chat-completions shape and
translated, while an already-translated tool (`input_schema`) passes through.

## Inline providers

A flow may pass a provider as a table instead of a registry id:

```lua
hob.agent.ask{
  provider = { id = "local", base_url = "http://127.0.0.1:8080", api_key_env = "LOCAL_KEY" },
  model = "qwen3",
  prompt = "hi",
}
```

That is how a local gateway (llama.cpp, vLLM, Ollama) or a test server is
reached without adding a registry entry. The key policy is unchanged: the
table names an environment variable, never a key. `headers` carries what a
gateway needs for routing. The registry holds no model, because model ids
change per request, so `agent` requires `model` rather than guessing one.

## The configuration file

`config.toml` in the user's configuration directory defines providers that are
not in the registry:

```toml
[defaults]
provider = "local"   # used when a call names no provider
model = "qwen3-8b"   # used when a call names no model

[providers.local]
base_url = "http://127.0.0.1:8080"
api_key_env = "LOCAL_KEY"
headers = { X-Route = "team" }
```

`protocol` is `openai` when absent. The registry wins for an id it knows, so a
`[providers.deepseek]` table cannot silently redirect the built-in definition;
the file only adds names. The key policy is unchanged: an entry names the
variable to read, never the key itself. A file that cannot be parsed fails the
call that needed it, with the path and the parser's message.

`[defaults]` is what a call with no `provider`/`model` option uses. Without it,
`provider` falls back to `deepseek` and a missing `model` is still an error: ids
change, so the engine does not guess one, it only follows what was written down.

A flow names a provider as a registry id, a name from the configuration file, or
an inline table; the first two are looked up in that order.

## Keys

hob reads keys from the environment and never writes them anywhere.

- Each provider names one variable (`DEEPSEEK_API_KEY` for DeepSeek).
- `Client::from_env` reads it at construction. There is no credential file,
  no `hob auth` store, and no key in any configuration hob may write later.
- A missing key fails with the provider's name and the variable to set, never
  a value.
- `Secret` is the only type that carries a key. It has no `Display`, no
  `Serialize` and a redacted `Debug`, so the key can only leave through
  `expose()` at the request boundary. Error bodies are scrubbed with the same
  value before they are shown.

Why not store keys: a secret at rest leaks through dotfiles repositories,
backups, screenshots and bug reports, and storing one well (permissions,
encryption, rotation) is a responsibility this tool should not take on. The
environment composes with direnv, `pass`, `op run` or a systemd unit, which is
where a developer already keeps secrets.

If a config-file fallback is ever added, it belongs in `secret::resolve` as a
second lookup source, so the policy stays in one place.

## Streaming

`stream = true` sends `stream: true` and reads the answer as server-sent
events; the text is shown as it arrives (stderr) and the assembled
`ChatResponse` is what the flow receives. The two dialects differ only in their
events: chat completions send `choices[0].delta` and a final `[DONE]`, Messages
sends `content_block_delta` and `message_delta`.

An OpenAI-compatible request also asks for token usage in the last event
(`stream_options.include_usage`), since servers only send it when asked; a
server that ignores the option streams fine but reports no tokens, so
`--max-tokens` cannot see that call's cost. Streaming with `tools` is refused
until every dialect can assemble a streamed tool call: the OpenAI decoder
already does, the Messages one does not. A retry happens only while nothing has
been shown, so a partial line is never printed twice.

## Deliberately deferred

- Retry policy inside the client; `agent.max_attempts` retries a failed call at
  the flow level. The client already applies connect and request timeouts.
- `hob models` as a command; `Client::models` is the underlying call, exposed
  as `agent.list`.
