// This crate contains the thumbnail subsystem.
// It re‑exports the core modules it depends on to keep the
// existing import paths (`crate::…`) working.

// Re‑export core pieces the thumbnail code uses.
pub use plato_core::buffer_pool;
pub use plato_core::consts::thumbnail as core_thumbnail;
pub use plato_core::device;
pub use plato_core::document;
pub use plato_core::framebuffer;
pub use plato_core::log_error;

// Public API of this crate – same API that existed in the old
// `crates/core/src/thumbnail/mod.rs`.
mod cache;
mod error;
mod manager;
mod request;
mod worker;

pub use cache::ThumbnailCache;
pub use error::{ThumbnailError, ThumbnailResult};
pub use manager::ThumbnailManager;
pub use request::ThumbnailRequest;

// Re‑export constants from the core crate for compatibility.
pub use core_thumbnail::{
    ANDROID_CACHE_SIZE, ANDROID_WORKER_COUNT, DEFAULT_CACHE_SIZE, DEFAULT_WORKER_COUNT,
    ELIPSA_CACHE_SIZE, ELIPSA_WORKER_COUNT, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH,
};

// Max worker configuration constants.
pub const MAX_WORKER_COUNT: usize = 4;
pub const ANDROID_MAX_WORKER_COUNT: usize = 6;
pub const MIN_WORKER_COUNT: usize = 1;
pub const MAX_CACHE_SIZE: usize = 50;
pub const ANDROID_MAX_CACHE_SIZE: usize = 100;
pub const MIN_CACHE_SIZE: usize = 5;

// Helper functions that used to live in `crates/core/src/thumbnail/mod.rs`.
use plato_core::device::{is_android, is_elipsa};

/// Get the optimal worker count for the current device
pub fn optimal_worker_count() -> usize {
    if is_elipsa() {
        core_thumbnail::ELIPSA_WORKER_COUNT
    } else if is_android() {
        core_thumbnail::ANDROID_WORKER_COUNT
    } else {
        core_thumbnail::DEFAULT_WORKER_COUNT
    }
}

/// Get the optimal cache size for the current device
pub fn optimal_cache_size() -> usize {
    if is_elipsa() {
        core_thumbnail::ELIPSA_CACHE_SIZE
    } else if is_android() {
        core_thumbnail::ANDROID_CACHE_SIZE
    } else {
        core_thumbnail::DEFAULT_CACHE_SIZE
    }
}
