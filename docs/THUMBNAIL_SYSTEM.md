# Thumbnail System

Plato generates cover thumbnails in the background so that the library view stays responsive even on low‑power e‑ink devices. The subsystem lives in the `plato‑thumbnail` crate (or the `thumbnail` module of `plato‑core`).

## Architecture

```
┌───────────────────────────────────────────────┐
│                Library View                 │
│  (requests thumbnail for each book)       │
└──────────────────┬──────────────────────┘
                   │ request
                   ▼
┌───────────────────────────────────────────────┐
│          ThumbnailManager                 │
│  - holds a channel (Sender)               │
│  - maintains cache stats                    │
└──────────────────┬──────────────────────┘
                   │
         ┌─────────┴─────────┐
         │   Worker Pool        │
         │  (N worker threads) │
         └─────────┬─────────┘
                   │ render cover → pixmap
                   ▼
┌───────────────────────────────────────────────┐
│              LRU Cache                    │
│  - key: path (String)                     │
│  - value: (pixmap, last_accessed)        │
│  - max size configured per device           │
└───────────────────────────────────────────────┘
```

## Worker pool sizing

The optimal number of worker threads is chosen at startup:

```rust
pub fn optimal_worker_count() -> usize {
    if is_mobile_platform() {
        // Android: up to 4 cores
        num_cpus::get().min(4)
    } else {
        // Kobo: single‑core ARM, be gentle
        1
    }
}
```

The same logic picks the cache size (`optimal_cache_size()`). For Kobo devices the defaults are 1 worker and 50 MiB cache; for Android they are scaled to the number of CPU cores (max 4) and 200 MiB.

## Cache eviction

The cache is an LRU (least‑recently‑used) structure. When the total size exceeds the configured limit, the least‑recently‑accessed entry is dropped. Each cache entry stores the pixmap bytes and a timestamp; the timestamp is refreshed on every `get`.

## Configuration

Settings are stored in `Settings.thumbnail`:

| Field | Type | Default | Meaning |
|-------|------|---------|---------|
| `worker_count` | `usize` | `optimal_worker_count()` | Override worker thread count. |
| `cache_size` | `usize` | `optimal_cache_size()` | Max cache size in bytes. |
| `default_width` | `u32` | 200 | Width of generated thumbnails (pixels). |
| `default_height` | `u32` | 300 | Height of generated thumbnails (pixels). |

## Performance notes

- Thumbnails are generated only when requested; there is no eager pre‑generation.
- The worker threads block on I/O (reading image files) and CPU (scaling). On Kobo, only one thread is used to avoid starving the UI.
- The LRU cache is protected by a `Mutex`; contention is minimal because the UI only reads when a thumbnail is missing.
