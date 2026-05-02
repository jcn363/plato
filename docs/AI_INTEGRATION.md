# AI Integration

Plato includes an optional AI layer that provides text embeddings and can be used for semantic search, smart recommendations, or an in‑reader chatbot. The implementation lives in the `plato‑ai` crate and is controlled by a simple provider abstraction.

## Provider abstraction

The central trait is `Provider` (defined in `crates/ai/src/traits.rs`). It requires a single method:

```rust
fn embed(&self, text: &str) -> PlatoResult<Vec<f32>>;
```

Three implementations ship out of the box:

| Provider | Variant | Notes |
|----------|---------|-------|
| `OllamaProvider` | Local Ollama server (default) | Expects a running `ollama serve` with the model already pulled. |
| `LocalProvider` | Pure‑Rust backend via `candle` | Uses a quantized model (e.g., `phi3‑mini`) that is loaded at startup. |
| `OpenAiProvider` / `ClaudeProvider` | Remote cloud APIs | Require an API key and a network connection. |

## Configuration

Settings are stored in `Settings.ai` (see `crates/core/src/settings/`). The relevant fields are:

- `provider` – one of `"ollama"`, `"local"`, `"openai"`, `"claude"`
- `model` – model name understood by the chosen provider (e.g., `"phi3‑mini"`, `"gpt‑4o"`)
- `endpoint` – optional override (defaults: `http://localhost:11434` for Ollama, `https://api.openai.com/v1`, etc.)
- `api_key` – required for cloud providers.

The UI exposes a simple toggle button that cycles through the available providers, and another that cycles through preset models.

## Cache

A thread‑safe LRU cache (`crates/ai/src/cache.rs`) stores recent embeddings so that repeated queries (e.g., “What is this book about?”) do not hit the model again. The cache size is configurable via `Settings.ai.cache_size`.

## Adding a new provider

1. Create a struct that implements `Provider`.
2. Add a new variant to the `ProviderType` enum in `crates/ai/src/traits.rs`.
3. Update `Provider::new()` to construct your provider when the variant is selected.
4. Add a preset button entry in the UI (see `crates/core/src/view/ai_chat.rs`).

## Limitations

- Only embedding generation is currently exposed; chat‑completion endpoints are stubbed.
- On Kobo devices the feature is disabled (no network, no CPU power).
- Android TTS is stubbed; AI works only on desktop and Android if the native back‑end can be compiled.
