use crate::validation::validate_range;
use anyhow::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ThumbnailSettings {
    pub enabled: bool,
    pub worker_count: usize,
    pub cache_size: usize,
    pub thumbnail_width: u32,
    pub thumbnail_height: u32,
}

impl Default for ThumbnailSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            worker_count: crate::thumbnail::DEFAULT_WORKER_COUNT,
            cache_size: crate::thumbnail::DEFAULT_CACHE_SIZE,
            thumbnail_width: crate::thumbnail::THUMBNAIL_WIDTH,
            thumbnail_height: crate::thumbnail::THUMBNAIL_HEIGHT,
        }
    }
}

impl ThumbnailSettings {
    /// Validates thumbnail settings are within acceptable ranges
    ///
    /// # Validation Rules
    /// - worker_count: 1 to 8 (reasonable thread count)
    /// - cache_size: 10 to 10000 (reasonable cache size)
    /// - thumbnail_width: 50 to 800 pixels
    /// - thumbnail_height: 50 to 800 pixels
    pub fn validate(&self) -> Result<(), Error> {
        // Worker count must be reasonable
        validate_range(self.worker_count, 1, 8, "thumbnail.worker_count")?;

        // Cache size must be reasonable (10 to 10000 entries)
        validate_range(self.cache_size, 10, 10000, "thumbnail.cache_size")?;

        // Thumbnail dimensions must be reasonable
        validate_range(self.thumbnail_width, 50, 800, "thumbnail.thumbnail_width")?;
        validate_range(self.thumbnail_height, 50, 800, "thumbnail.thumbnail_height")?;

        Ok(())
    }
}
