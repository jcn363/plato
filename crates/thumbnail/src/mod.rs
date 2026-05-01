pub mod cache;
pub mod error;
pub mod manager;
pub mod request;
pub mod worker;

pub use cache::ThumbnailCache;
pub use error::{ThumbnailError, ThumbnailResult};
pub use manager::ThumbnailManager;
pub use request::ThumbnailRequest;

// Re-export thumbnail constants from canonical source in consts::thumbnail
// per Single Source of Truth rule.
pub use crate::consts::thumbnail::{
    ANDROID_CACHE_SIZE, ANDROID_WORKER_COUNT, DEFAULT_CACHE_SIZE, DEFAULT_WORKER_COUNT,
    ELIPSA_CACHE_SIZE, ELIPSA_WORKER_COUNT, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH,
};

/// Maximum allowed worker threads for Kobo devices
pub const MAX_WORKER_COUNT: usize = 4;

/// Maximum allowed worker threads for Android devices (8-core CPUs)
pub const ANDROID_MAX_WORKER_COUNT: usize = 6;

/// Minimum allowed worker threads
pub const MIN_WORKER_COUNT: usize = 1;

/// Maximum allowed cache size in memory for Kobo
pub const MAX_CACHE_SIZE: usize = 50;

/// Maximum allowed cache size for Android devices
pub const ANDROID_MAX_CACHE_SIZE: usize = 100;

/// Minimum allowed cache size
pub const MIN_CACHE_SIZE: usize = 5;

use crate::device::{is_android, is_elipsa};

/// Get the optimal worker count for the current device
pub fn optimal_worker_count() -> usize {
    if is_elipsa() {
        ELIPSA_WORKER_COUNT
    } else if is_android() {
        ANDROID_WORKER_COUNT
    } else {
        DEFAULT_WORKER_COUNT
    }
}

/// Get the optimal cache size for the current device
pub fn optimal_cache_size() -> usize {
    if is_elipsa() {
        ELIPSA_CACHE_SIZE
    } else if is_android() {
        ANDROID_CACHE_SIZE
    } else {
        DEFAULT_CACHE_SIZE
    }
}
