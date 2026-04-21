//! Font Cache Module
//!
//! This module provides font glyph caching functionality for text rendering.

use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Font cache entry
#[derive(Debug, Clone)]
pub struct FontCacheEntry {
    pub glyph_id: u32,
    pub pixmap: Option<Arc<Vec<u8>>>,
    pub width: f32,
    pub height: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance: f32,
    pub last_accessed: std::time::Instant,
}

/// Font cache configuration
#[derive(Debug, Clone)]
pub struct FontCacheConfig {
    pub max_entries: usize,
    pub max_memory_mb: usize,
    pub cleanup_threshold: f32,
    pub ttl_seconds: u64,
}

impl Default for FontCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            max_memory_mb: 100,
            cleanup_threshold: 0.8,
            ttl_seconds: 3600,
        }
    }
}

/// Font glyph cache
pub struct FontCache {
    cache: FxHashMap<u32, FontCacheEntry>,
    config: FontCacheConfig,
    current_memory: usize,
    access_order: Vec<u32>,
    cache_hits: u64,
    cache_misses: u64,
    eviction_count: u64,
}

impl FontCache {
    /// Create a new font cache
    pub fn new(config: FontCacheConfig) -> Self {
        let max_entries = config.max_entries;
        Self {
            cache: FxHashMap::default(),
            config,
            current_memory: 0,
            access_order: Vec::with_capacity(max_entries),
            cache_hits: 0,
            cache_misses: 0,
            eviction_count: 0,
        }
    }

    /// Get a cached glyph
    ///
    /// Updates hit/miss counters for performance tracking.
    pub fn get(&mut self, glyph_id: u32) -> Option<&FontCacheEntry> {
        if self.cache.contains_key(&glyph_id) {
            self.cache_hits += 1;
            self.update_access_order(glyph_id);
            self.cache.get_mut(&glyph_id).map(|entry| {
                entry.last_accessed = std::time::Instant::now();
                &*entry
            })
        } else {
            self.cache_misses += 1;
            None
        }
    }

    /// Insert a glyph into the cache
    pub fn insert(&mut self, glyph_id: u32, entry: FontCacheEntry) {
        // Check if we need to cleanup
        if self.should_cleanup() {
            self.cleanup();
        }

        let memory_size = self.calculate_memory_size(&entry);

        // Remove existing entry if present
        if let Some(old_entry) = self.cache.remove(&glyph_id) {
            self.current_memory -= self.calculate_memory_size(&old_entry);
        }

        self.current_memory += memory_size;
        self.cache.insert(glyph_id, entry);
        self.update_access_order(glyph_id);
    }

    /// Remove a glyph from the cache
    pub fn remove(&mut self, glyph_id: u32) -> Option<FontCacheEntry> {
        if let Some(entry) = self.cache.remove(&glyph_id) {
            self.current_memory -= self.calculate_memory_size(&entry);
            self.remove_from_access_order(glyph_id);
            Some(entry)
        } else {
            None
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.eviction_count += self.cache.len() as u64;
        self.cache.clear();
        self.current_memory = 0;
        self.access_order.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_lookups = self.cache_hits + self.cache_misses;
        let hit_rate = if total_lookups > 0 {
            self.cache_hits as f32 / total_lookups as f32
        } else {
            0.0
        };

        CacheStats {
            entries: self.cache.len(),
            memory_mb: self.current_memory / (1024 * 1024),
            hit_rate,
            eviction_count: self.eviction_count as usize,
        }
    }

    /// Check if cleanup is needed
    fn should_cleanup(&self) -> bool {
        self.cache.len() > self.config.max_entries
            || self.current_memory > (self.config.max_memory_mb * 1024 * 1024)
    }

    /// Cleanup old entries
    fn cleanup(&mut self) {
        let now = std::time::Instant::now();
        let mut to_remove = Vec::with_capacity(self.cache.len() / 4);

        // Remove expired entries
        for (glyph_id, entry) in &self.cache {
            if now.duration_since(entry.last_accessed).as_secs() > self.config.ttl_seconds {
                to_remove.push(*glyph_id);
            }
        }

        // Remove entries if still over limits
        if self.cache.len() - to_remove.len() > self.config.max_entries {
            let excess = self.cache.len() - to_remove.len() - self.config.max_entries;
            let oldest_entries = self
                .access_order
                .iter()
                .take(excess)
                .cloned()
                .collect::<Vec<_>>();
            to_remove.extend(oldest_entries);
        }

        // Actually remove the entries
        let removed_count = to_remove.len() as u64;
        for glyph_id in to_remove {
            if let Some(entry) = self.cache.remove(&glyph_id) {
                self.current_memory -= self.calculate_memory_size(&entry);
                self.remove_from_access_order(glyph_id);
            }
        }
        self.eviction_count += removed_count;
    }

    /// Calculate memory size of an entry
    fn calculate_memory_size(&self, entry: &FontCacheEntry) -> usize {
        let pixmap_size = entry.pixmap.as_ref().map(|p| p.len()).unwrap_or(0);
        std::mem::size_of::<FontCacheEntry>() + pixmap_size
    }

    /// Update access order
    fn update_access_order(&mut self, glyph_id: u32) {
        if let Some(pos) = self.access_order.iter().position(|&id| id == glyph_id) {
            self.access_order.remove(pos);
        }
        self.access_order.push(glyph_id);
    }

    /// Remove from access order
    fn remove_from_access_order(&mut self, glyph_id: u32) {
        if let Some(pos) = self.access_order.iter().position(|&id| id == glyph_id) {
            self.access_order.remove(pos);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub memory_mb: usize,
    pub hit_rate: f32,
    pub eviction_count: usize,
}

/// Utility functions for font caching
pub mod utils {
    use super::*;

    /// Create a font cache with default settings
    pub fn create_default_cache() -> FontCache {
        FontCache::new(FontCacheConfig::default())
    }

    /// Create a font cache entry
    pub fn create_cache_entry(
        glyph_id: u32,
        width: f32,
        height: f32,
        advance: f32,
    ) -> FontCacheEntry {
        FontCacheEntry {
            glyph_id,
            pixmap: None,
            width,
            height,
            bearing_x: 0.0,
            bearing_y: height * 0.8,
            advance,
            last_accessed: std::time::Instant::now(),
        }
    }

    /// Estimate cache memory usage
    pub fn estimate_cache_memory_usage(entries: usize, average_glyph_size: usize) -> usize {
        entries * (std::mem::size_of::<FontCacheEntry>() + average_glyph_size)
    }

    /// Check if a glyph should be cached
    pub fn should_cache_glyph(glyph_id: u32, text_frequency: f32) -> bool {
        // Cache frequently used glyphs
        text_frequency > 0.1 || glyph_id < 128 // ASCII characters
    }
}
