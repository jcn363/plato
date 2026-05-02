# Plato‑AI

The `plato-ai` crate implements a lightweight LLM‑powered embedding engine and a provider abstraction used by the main application to fetch embeddings and generate text.  It supports local and cloud back‑ends via a simple trait interface, and ships with a built‑in cache to keep network traffic low.

## Public API

| Symbol | Description |
|--------|-------------|
| `Provider` | Trait that defines the set of operations an LLM provider must expose.
| `Embeddings` | Functions to convert text to vector embeddings.
| `Config` | Runtime configuration (model name, endpoint, API key, cache size).
| `Cache` | Simple in‑memory LRU cache used by the providers.

Implementations are available for:

* `OllamaProvider` – local Ollama server.
* `LocalProvider` – any locally running model via the `candle` backend.
* `ClaudeProvider`, `OpenAiProvider` – optional remote providers.

The crate is intentionally tiny and can be used as a drop‑in dependency in other services.

## Optional features

None – the crate is featureless and pulls in the required back‑ends via `cargo`.

## Example usage

```rust
use plato_ai::{LocalProvider, Embeddings, Config};

let mut cfg = Config::default();
cfg.model = "phi3-mini".into();
let provider = LocalProvider::new(cfg).expect("failed to init model");
let vec = provider.embed("Hello world");
```