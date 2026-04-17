use crate::thumbnail::cache::ThumbnailCache;
use crate::thumbnail::error::{ThumbnailError, ThumbnailResult};
use crate::thumbnail::request::ThumbnailRequest;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};

/// Configuration for thumbnail generation
#[derive(Debug, Clone)]
pub struct ThumbnailConfig {
    pub worker_count: usize,
    pub cache_size: usize,
    pub thumbnail_width: u32,
    pub thumbnail_height: u32,
    pub enabled: bool,
}

impl Default for ThumbnailConfig {
    fn default() -> Self {
        Self {
            worker_count: crate::thumbnail::DEFAULT_WORKER_COUNT,
            cache_size: crate::thumbnail::DEFAULT_CACHE_SIZE,
            thumbnail_width: crate::thumbnail::THUMBNAIL_WIDTH,
            thumbnail_height: crate::thumbnail::THUMBNAIL_HEIGHT,
            enabled: true,
        }
    }
}

impl ThumbnailConfig {
    /// Creates a new thumbnail configuration with validation
    pub fn new(
        worker_count: usize,
        cache_size: usize,
        thumbnail_width: u32,
        thumbnail_height: u32,
        enabled: bool,
    ) -> ThumbnailResult<Self> {
        // Validate worker count
        if worker_count < crate::thumbnail::MIN_WORKER_COUNT
            || worker_count > crate::thumbnail::MAX_WORKER_COUNT
        {
            return Err(ThumbnailError::configuration(format!(
                "worker count must be between {} and {}",
                crate::thumbnail::MIN_WORKER_COUNT,
                crate::thumbnail::MAX_WORKER_COUNT
            )));
        }

        // Validate cache size
        if cache_size < crate::thumbnail::MIN_CACHE_SIZE
            || cache_size > crate::thumbnail::MAX_CACHE_SIZE
        {
            return Err(ThumbnailError::configuration(format!(
                "cache size must be between {} and {}",
                crate::thumbnail::MIN_CACHE_SIZE,
                crate::thumbnail::MAX_CACHE_SIZE
            )));
        }

        // Validate thumbnail dimensions
        if thumbnail_width == 0 || thumbnail_height == 0 {
            return Err(ThumbnailError::configuration(
                "thumbnail dimensions cannot be zero",
            ));
        }

        const MAX_DIMENSION: u32 = 1000;
        if thumbnail_width > MAX_DIMENSION || thumbnail_height > MAX_DIMENSION {
            return Err(ThumbnailError::configuration(format!(
                "thumbnail dimensions must be <= {} pixels",
                MAX_DIMENSION
            )));
        }

        Ok(Self {
            worker_count,
            cache_size,
            thumbnail_width,
            thumbnail_height,
            enabled,
        })
    }

    /// Gets thumbnail dimensions as a tuple
    pub fn dimensions(&self) -> (u32, u32) {
        (self.thumbnail_width, self.thumbnail_height)
    }
}

/// Manages lazy thumbnail generation with worker pool and caching
pub struct ThumbnailManager {
    config: ThumbnailConfig,
    cache: Arc<Mutex<ThumbnailCache>>,
    pending_requests: Arc<DashMap<PathBuf, ()>>,
    request_sender: Sender<ThumbnailRequest>,
}

impl ThumbnailManager {
    /// Creates a new thumbnail manager with the given configuration
    pub fn new(config: ThumbnailConfig) -> ThumbnailResult<Self> {
        if !config.enabled {
            return Err(ThumbnailError::configuration(
                "thumbnail generation is disabled",
            ));
        }

        // Validate configuration
        let validated_config = ThumbnailConfig::new(
            config.worker_count,
            config.cache_size,
            config.thumbnail_width,
            config.thumbnail_height,
            config.enabled,
        )?;

        // Create cache
        let cache = Arc::new(Mutex::new(ThumbnailCache::new(
            validated_config.cache_size,
        )?));

        // Create communication channels
        let (request_sender, _request_receiver) = mpsc::channel::<ThumbnailRequest>();

        Ok(Self {
            config: validated_config,
            cache,
            request_sender,
            pending_requests: Arc::new(DashMap::new()),
        })
    }

    /// Requests a thumbnail for the given file path
    /// Returns Some(PathBuf) if thumbnail is available (cached or on disk)
    /// Returns None if thumbnail generation is in progress
    pub fn request_thumbnail(&self, file_path: &Path) -> ThumbnailResult<Option<PathBuf>> {
        // Validate input
        if file_path.as_os_str().is_empty() {
            return Err(ThumbnailError::invalid_path("empty file path"));
        }

        let file_path = file_path.to_path_buf();

        // Check if already pending
        if self.pending_requests.contains_key(&file_path) {
            return Ok(None);
        }

        let thumbnail_path = self.compute_thumbnail_path(&file_path)?;

        // Check if thumbnail exists on disk
        if thumbnail_path.exists() {
            return Ok(Some(thumbnail_path));
        }

        // Check cache
        if let Ok(mut cache) = self.cache.lock() {
            if let Some(_pixmap) = cache.get(&thumbnail_path) {
                return Ok(Some(thumbnail_path));
            }
        }

        // Add to pending requests
        self.pending_requests.insert(file_path.clone(), ());

        // Create and submit request
        let (response_tx, _response_rx) = mpsc::channel::<ThumbnailResult<PathBuf>>();
        let request = ThumbnailRequest::new(
            file_path.clone(),
            thumbnail_path.clone(),
            self.config.dimensions(),
            response_tx,
        );

        if let Err(_) = self.request_sender.send(request) {
            // Remove from pending if submission failed
            self.pending_requests.remove(&file_path);
            return Err(ThumbnailError::Channel);
        }

        // TODO: wait for the result or use async
        Ok(None)
    }

    /// Computes the thumbnail path for a given file path
    fn compute_thumbnail_path(&self, file_path: &Path) -> ThumbnailResult<PathBuf> {
        // TODO: Use library.thumbnail_preview_path() when integrated
        // For now, create a simple path
        let file_name = file_path
            .file_stem()
            .ok_or_else(|| ThumbnailError::invalid_path("invalid file name"))?;

        let thumbnail_name = format!("{}.png", file_name.to_string_lossy());
        Ok(file_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(".thumbnail-previews")
            .join(thumbnail_name))
    }

    /// Gets cache statistics for monitoring
    pub fn cache_stats(&self) -> crate::thumbnail::cache::CacheStats {
        self.cache.lock().expect("cache lock poisoned").stats()
    }

    /// Gets the number of pending requests
    pub fn pending_count(&self) -> usize {
        self.pending_requests.len()
    }

    /// Gets the current configuration
    pub fn config(&self) -> &ThumbnailConfig {
        &self.config
    }

    /// Clears all cached thumbnails
    pub fn clear_cache(&mut self) -> ThumbnailResult<()> {
        self.cache.lock().expect("cache lock poisoned").clear();
        Ok(())
    }

    /// Cancels a pending thumbnail request
    pub fn cancel_request(&self, file_path: &Path) -> bool {
        self.pending_requests.remove(file_path).is_some()
    }
}

impl Drop for ThumbnailManager {
    fn drop(&mut self) {
        // Cancel all pending requests
        self.pending_requests.clear();

        // Clear cache
        let _ = self.clear_cache();

        // Worker pool will be dropped automatically
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = ThumbnailConfig::default();
        assert_eq!(config.worker_count, crate::thumbnail::DEFAULT_WORKER_COUNT);
        assert_eq!(config.cache_size, crate::thumbnail::DEFAULT_CACHE_SIZE);
        assert!(config.enabled);
    }

    #[test]
    fn test_config_new_valid() {
        let config = ThumbnailConfig::new(2, 20, 240, 320, true);
        assert!(config.is_ok());
    }

    #[test]
    fn test_config_new_invalid_worker_count() {
        let config = ThumbnailConfig::new(0, 20, 240, 320, true);
        assert!(config.is_err());

        let config = ThumbnailConfig::new(10, 20, 240, 320, true);
        assert!(config.is_err());
    }

    #[test]
    fn test_config_new_invalid_cache_size() {
        let config = ThumbnailConfig::new(2, 0, 240, 320, true);
        assert!(config.is_err());

        let config = ThumbnailConfig::new(2, 100, 240, 320, true);
        assert!(config.is_err());
    }

    #[test]
    fn test_config_new_invalid_dimensions() {
        let config = ThumbnailConfig::new(2, 20, 0, 320, true);
        assert!(config.is_err());

        let config = ThumbnailConfig::new(2, 20, 240, 0, true);
        assert!(config.is_err());

        let config = ThumbnailConfig::new(2, 20, 2000, 320, true);
        assert!(config.is_err());
    }

    #[test]
    fn test_manager_new() {
        let config = ThumbnailConfig::default();
        let _manager = ThumbnailManager::new(config.clone()).expect("Failed to create manager");
    }

    #[test]
    fn test_manager_new_disabled() {
        let mut config = ThumbnailConfig::default();
        config.enabled = false;
        let manager = ThumbnailManager::new(config);
        assert!(manager.is_err());
    }

    #[test]
    fn test_request_thumbnail_invalid_path() {
        let config = ThumbnailConfig::default();
        let manager = ThumbnailManager::new(config).expect("Failed to create manager");
        let result = manager.request_thumbnail(Path::new(""));
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_thumbnail_path() {
        let config = ThumbnailConfig::default();
        let manager = ThumbnailManager::new(config).expect("Failed to create manager");
        let file_path = Path::new("/home/user/book.pdf");
        let thumbnail_path = manager
            .compute_thumbnail_path(file_path)
            .expect("Failed to compute thumbnail path");
        assert!(thumbnail_path.ends_with(".thumbnail-previews/book.png"));
    }

    #[test]
    fn test_cache_stats() {
        let config = ThumbnailConfig::default();
        let cache_size = config.cache_size;
        let manager = ThumbnailManager::new(config).expect("Failed to create manager");
        let stats = manager.cache_stats();
        assert_eq!(stats.current_size, 0);
        assert_eq!(stats.max_size, cache_size);
    }

    #[test]
    fn test_pending_count() {
        let config = ThumbnailConfig::default();
        let manager = ThumbnailManager::new(config).expect("Failed to create manager");
        assert_eq!(manager.pending_count(), 0);
    }
}
