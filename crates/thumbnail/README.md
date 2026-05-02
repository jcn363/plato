# Plato‑Thumbnail

The `plato‑thumbnail` crate provides background thumbnail generation for the Plato e‑reader. It manages a pool of worker threads that load cover images, scale them to the required size, and cache the results in an LRU cache that is optimized for the limited memory of e‑ink devices.

## Public API

| Symbol | Description |
|--------|-------------|
| `ThumbnailManager` | Coordinates the worker pool and the cache. |
| `ThumbnailConfig` | Runtime configuration: worker count, cache size, default dimensions. |
| `request_thumbnail` | Enqueue a thumbnail generation request. |
| `get_thumbnail` | Retrieve a cached pixmap, if available. |
| `cache_stats` | Return hit/miss counts for monitoring. |

The worker pool auto‑scales according to the device (Kobo vs Android) and respects the limits defined in `ThumbnailConfig`.

## Example usage

```rust
use plato_thumbnail::{ThumbnailManager, ThumbnailConfig};

let cfg = ThumbnailConfig::default();
let mut mgr = ThumbnailManager::new(cfg);
mgr.request_thumbnail("/path/to/cover.jpg").await;
let pixmap = mgr.get_thumbnail("/path/to/cover.jpg");
```
