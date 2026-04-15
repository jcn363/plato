use std::path::PathBuf;

use crate::framebuffer::{Framebuffer, Pixmap};
use crate::thumbnail::ThumbnailResult;

/// Thread-safe LRU cache for thumbnail pixmaps
pub struct ThumbnailCache {
    cache: lru::LruCache<PathBuf, Pixmap>,
    max_size: usize,
}

impl ThumbnailCache {
    /// Creates a new thumbnail cache with the specified maximum size
    pub fn new(max_size: usize) -> ThumbnailResult<Self> {
        if max_size == 0 {
            return Err(crate::thumbnail::error::ThumbnailError::cache(
                "cache size cannot be zero",
            ));
        }

        if max_size > 1000 {
            return Err(crate::thumbnail::error::ThumbnailError::cache(
                "cache size too large",
            ));
        }

        Ok(Self {
            cache: lru::LruCache::new(std::num::NonZeroUsize::new(max_size).unwrap()),
            max_size,
        })
    }

    /// Gets a cached thumbnail pixmap for the given file path
    pub fn get(&mut self, path: &PathBuf) -> Option<Pixmap> {
        self.cache.get(path).cloned()
    }

    /// Inserts a thumbnail pixmap into the cache
    pub fn put(&mut self, path: PathBuf, pixmap: Pixmap) -> ThumbnailResult<()> {
        // Validate pixmap size to prevent memory issues
        if pixmap.width() == 0 || pixmap.height() == 0 {
            return Err(crate::thumbnail::error::ThumbnailError::cache(
                "invalid pixmap dimensions",
            ));
        }

        // Estimate memory usage and reject if too large
        let estimated_bytes = pixmap.width() as usize * pixmap.height() as usize * 4; // RGBA
        const MAX_PIXMAP_SIZE: usize = 10 * 1024 * 1024; // 10MB per pixmap
        if estimated_bytes > MAX_PIXMAP_SIZE {
            return Err(crate::thumbnail::error::ThumbnailError::resource_limit(
                "pixmap too large for cache",
            ));
        }

        self.cache.put(path, pixmap);
        Ok(())
    }

    /// Removes a thumbnail from the cache
    pub fn remove(&mut self, path: &PathBuf) -> Option<Pixmap> {
        self.cache.pop(path)
    }

    /// Clears all cached thumbnails
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Returns the current number of cached items
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns true if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Returns the maximum cache size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Returns cache statistics for monitoring
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            current_size: self.cache.len(),
            max_size: self.max_size,
            utilization_ratio: self.cache.len() as f64 / self.max_size as f64,
        }
    }
}

impl Drop for ThumbnailCache {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub current_size: usize,
    pub max_size: usize,
    pub utilization_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Pixmap;

    fn create_test_pixmap(width: u32, height: u32) -> Pixmap {
        // Create a simple test pixmap
        Pixmap::new(width, height, 4).expect("Failed to create test pixmap")
    }

    #[test]
    fn test_cache_new() {
        let cache = ThumbnailCache::new(10);
        assert!(cache.is_ok());
        let cache = cache.unwrap();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.max_size(), 10);
    }

    #[test]
    fn test_cache_new_zero_size() {
        let cache = ThumbnailCache::new(0);
        assert!(cache.is_err());
    }

    #[test]
    fn test_cache_put_and_get() {
        let mut cache = ThumbnailCache::new(5).unwrap();
        let path = PathBuf::from("test.png");
        let pixmap = create_test_pixmap(100, 100);

        assert!(cache.put(path.clone(), pixmap.clone()).is_ok());
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get(&path);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().width(), pixmap.width());
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = ThumbnailCache::new(2).unwrap();
        let path1 = PathBuf::from("test1.png");
        let path2 = PathBuf::from("test2.png");
        let path3 = PathBuf::from("test3.png");

        let pixmap = create_test_pixmap(100, 100);

        // Fill cache to capacity
        assert!(cache.put(path1.clone(), pixmap.clone()).is_ok());
        assert!(cache.put(path2.clone(), pixmap.clone()).is_ok());
        assert_eq!(cache.len(), 2);

        // Add third item, should evict first
        assert!(cache.put(path3.clone(), pixmap.clone()).is_ok());
        assert_eq!(cache.len(), 2);

        // First item should be evicted
        assert!(cache.get(&path1).is_none());
        // Second and third should be present
        assert!(cache.get(&path2).is_some());
        assert!(cache.get(&path3).is_some());
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = ThumbnailCache::new(5).unwrap();
        let path = PathBuf::from("test.png");
        let pixmap = create_test_pixmap(100, 100);

        assert!(cache.put(path.clone(), pixmap.clone()).is_ok());
        assert_eq!(cache.len(), 1);

        let removed = cache.remove(&path);
        assert!(removed.is_some());
        assert_eq!(cache.len(), 0);
        assert!(cache.get(&path).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = ThumbnailCache::new(5).unwrap();
        let pixmap = create_test_pixmap(100, 100);

        for i in 0..3 {
            let path = PathBuf::from(format!("test{}.png", i));
            assert!(cache.put(path, pixmap.clone()).is_ok());
        }
        assert_eq!(cache.len(), 3);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = ThumbnailCache::new(10).unwrap();
        let pixmap = create_test_pixmap(100, 100);

        let stats = cache.stats();
        assert_eq!(stats.current_size, 0);
        assert_eq!(stats.max_size, 10);
        assert_eq!(stats.utilization_ratio, 0.0);

        for i in 0..5 {
            let path = PathBuf::from(format!("test{}.png", i));
            assert!(cache.put(path, pixmap.clone()).is_ok());
        }

        let stats = cache.stats();
        assert_eq!(stats.current_size, 5);
        assert_eq!(stats.max_size, 10);
        assert_eq!(stats.utilization_ratio, 0.5);
    }
}
