pub mod cache;
pub mod error;
pub mod manager;
pub mod request;
pub mod worker;

pub use cache::ThumbnailCache;
pub use error::{ThumbnailError, ThumbnailResult};
pub use manager::ThumbnailManager;
pub use request::ThumbnailRequest;

/// Default number of worker threads for thumbnail generation
pub const DEFAULT_WORKER_COUNT: usize = 2;

/// Default maximum number of thumbnails to cache in memory
pub const DEFAULT_CACHE_SIZE: usize = 20;

/// Default thumbnail width in pixels
pub const THUMBNAIL_WIDTH: u32 = 240;

/// Default thumbnail height in pixels
pub const THUMBNAIL_HEIGHT: u32 = 320;

/// Maximum allowed worker threads for Kobo devices
pub const MAX_WORKER_COUNT: usize = 4;

/// Minimum allowed worker threads
pub const MIN_WORKER_COUNT: usize = 1;

/// Maximum allowed cache size in memory
pub const MAX_CACHE_SIZE: usize = 50;

/// Minimum allowed cache size
pub const MIN_CACHE_SIZE: usize = 5;
