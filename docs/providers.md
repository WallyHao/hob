# Providers

## The shape of a provider

A provider is data, not code: `src/provider/spec.rs` holds `ProviderSpec`
(id, base URL, key variable, protocol), and `src/provider/registry.rs` lists
the built-ins. DeepSeek is the first entry.

Adding an OpenAI-compatible provider (Moonshot, Qwen, Zhipu, OpenRouter,
SiliconFlow, a local gateway) is one entry in `registry.rs` plus a test that
the entry names its own key variable. No client code changes, because they all
speak the same chat-completions dialect.

A provider with its own dialect (Anthropic Messages, Gemini) is the point
where a new `Protocol` variant and a module beside `client.rs` appear. The
seam exists; the abstraction is not built until the second dialect is real.

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

## Keys

hob reads keys from the environment and never writes them anywhere.

- Each provider names one variable (`DEEPSEEK_API_KEY` for DeepSeek).
- `Client::from_env` reads it at construction. There is no credential file,
  no `hob auth` store, and no key in any configuration hob may write later.
- A missing key fails with the variable's name and nothing else.
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

## Deliberately deferred

- Streaming (SSE) responses; the request shape already carries `stream`.
- Retry and per-request timeout policy inside the client; `agent.max_attempts`
  retries a failed call at the flow level.
- A provider entry in a configuration file; inline specs cover local gateways
  until then.
- `hob models` as a command; `Client::models` is the underlying call, exposed
  as `agent.list`.
