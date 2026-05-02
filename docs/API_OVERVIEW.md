# Plato – Public API Overview

This document summarises the most important public types and functions exposed by the Plato workspace crates. It is aimed at developers who need to integrate with or extend the system.

## Core traits (plato‑core)

| Trait | Module | Purpose |
|------|--------|---------|
| `Device` | `device` | Hardware abstraction for Kobo/Android/desktop (model, dimensions, DPI, frontlight, rotation). |
| `Framebuffer` | `framebuffer` | Display output – draw pixels, update regions, set waveforms. |
| `Document` | `document` | Common interface for PDF, EPUB, HTML documents. |
| `Page` | `document` | Single page of a `Document` – text, links, pixmap. |
| `TtsEngine` | `tts` (feature `tts`) | Text‑to‑speech engine (desktop & Android implementations). |
| `Battery` | `battery` | Read battery capacity/status; caching for low‑power devices. |
| `Frontlight` | `frontlight` | Control front‑light intensity/temperature. |
| `LightSensor` | `lightsensor` | Read ambient light level (if available). |
| `Provider` | `ai` (crate `plato‑ai`) | LLM provider abstraction (`OllamaProvider`, `LocalProvider`, …). |

All traits are defined in `plato‑core` (or `plato‑ai`) and can be mocked using the types in `plato_core::test_mocks`.

## Key error types

| Type | Variants (selected) | Notes |
|------|-------------------|-------|
| `PlatoError` | `Io`, `InvalidCharacter`, `MissingColumnInIndex`, `InvalidFileFormat`, `MemoryError`, `WordNotFound`, `Utf8Error`, `DeflateError`, `Format`, `Database`, `Candle`, `Ai`, `Config`, `Battery`, `Document`, `Plugin`, `Pdf`, `Unknown` | Defined in `crates/error/src/error.rs`. Use `PlatoResult<T> = Result<T, PlatoError>`. |
| `DictError` | `Plato`, `InvalidCharacter`, `MissingColumnInIndex`, … | Internal to dictionary module; wraps `PlatoError`. |

## Helper functions

| Function | Path | Description |
|----------|------|-------------|
| `estimate_from_page_count` | `reading_time` | Estimate reading time from page count & speed. |
| `estimate_from_word_count` | `reading_time` | Estimate reading time from word count. |
| `format_duration` | `reading_time` | Format a `Duration` as a human‑readable string. |
| `optimal_worker_count` | `thumbnail` (or `plato_core::thumbnail`) | Get the best worker‑thread count for the current device. |
| `optimal_cache_size` | `thumbnail` | Get the best LRU cache size for the current device. |
| `walkdir_visible` | `helpers` | Directory walker that skips hidden files. |
| `validate_path` | `validation` | Validate a filesystem path (no traversal, exists). |
| `validate_range` | `validation` | Validate a numeric value is inside [min, max). |

## Example – using `Device` and `Framebuffer`

```rust
use plato_core::{Device, KoboDevice, Framebuffer, thumbnail::optimal_worker_count};

fn main() {
    let device = KoboDevice::new(&product, &model_number);
    let fb = /* obtain a framebuffer */;
    let workers = optimal_worker_count();
    println!("Device model: {:?}", device.model());
}
```

## Example – generating an embedding with `plato‑ai`

```rust
use plato_ai::{LocalProvider, Embeddings, Config};

let cfg = Config::default();
let provider = LocalProvider::new(cfg).expect("model init failed");
let vec = provider.embed("Hello, Plato!");
println!("Embedding len: {}", vec.len());
```

## Module re‑exports

The most common types are re‑exported from `plato_core`:

```rust
pub use device::{Device, KoboDevice, Model, Orientation, CURRENT_DEVICE};
pub use framebuffer::Framebuffer;
pub use document::Document;
pub use thumbnail::{ThumbnailManager, ThumbnailConfig};
pub use tts::TtsEngine; // with feature `tts`
```

For a full list, see `crates/core/src/lib.rs`.
