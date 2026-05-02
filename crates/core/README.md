# Plato Core

The `plato-core` crate implements the core logic of the Plato e‑reader. It contains all device abstractions, rendering and UI infrastructure, library management, settings, document handling and the shared types used by the binary and the plugins.

## Exposed API

The public surface of the crate is split into a few logical modules:

| Module | Purpose |
|--------|---------|
| `device` | Trait `Device` and concrete `KoboDevice`, `MockDevice`, `AndroidDevice` implementations.
| `battery` | Low‑level battery access and `Battery` trait.
| `frontlight` | Abstract front‑light control.
| `framebuffer` | Framebuffer abstractions for Kobo, desktop and simulator.
| `document` | `Document` trait for PDFs/EPUBs/HTMLs, along with helpers.
| `library` | Library catalog operations: scan, query, maintenance and hygiene.
| `settings` | Config management, validation and the `ConfigManager` type.
| `theme` / `mobile_theme` | Colour themes, mobile dark‑mode helpers and touch settings.
| `view` | View tree, event bubbling and gesture handling.
| `tts` | Optional TTS engines – enabled only if the `tts` feature is activated.
| `thumbnail` | Background thumbnail worker pool and LRU cache.

All public types are re‑exported from `plato_core::` for easy access.

## Optional features

* `tts` – provides the `tts` module exposing `TtsEngine`.
* `android` – enables Android‑specific compilation.

```
# Cargo.toml
[dependencies]
plato-core = { path = "../core" }
```

For users running the binary, most of the functionality is used internally; the crates that expose a stable API are `plato` (the binary) and `plato-core`.

For contributing, see the repository’s CONTRIBUTING guide and run `cargo test` and `cargo clippy` to maintain the quality standards specified in AGENTS.md.
