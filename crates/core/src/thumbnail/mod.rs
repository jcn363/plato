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
    DEFAULT_CACHE_SIZE, DEFAULT_WORKER_COUNT, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH,
};

/// Maximum allowed worker threads for Kobo devices
pub const MAX_WORKER_COUNT: usize = 4;

/// Minimum allowed worker threads
pub const MIN_WORKER_COUNT: usize = 1;

/// Maximum allowed cache size in memory
pub const MAX_CACHE_SIZE: usize = 50;

/// Minimum allowed cache size
pub const MIN_CACHE_SIZE: usize = 5;
