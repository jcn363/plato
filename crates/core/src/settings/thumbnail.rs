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
