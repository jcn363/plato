//! Caching strategies for PDF rendering and text extraction
//! 
//! This module implements LRU caching for:
//! - Rendered pages (pixmaps)
//! - Extracted text
//! - Page metadata
//! - Outlines

use anyhow::Result;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use crate::document::pdfpurr::PdfPurrPixmap;

/// Cache key for page-related data
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PageCacheKey {
    pub doc_id: String,
    pub page_index: i32,
}

impl PageCacheKey {
    pub fn new(doc_id: String, page_index: i32) -> Self {
        Self { doc_id, page_index }
    }
}

/// Cached rendered page pixmap
#[derive(Debug, Clone)]
pub struct CachedPage {
    pub pixmap: PdfPurrPixmap,
    pub timestamp: u64,
}

/// Cached text extraction result
#[derive(Debug, Clone)]
pub struct CachedText {
    pub text: String,
    pub timestamp: u64,
}

/// Cached page metadata
#[derive(Debug, Clone)]
pub struct CachedMetadata {
    pub dims: (f32, f32),
    pub timestamp: u64,
}

/// Main cache for PDF operations
pub struct PdfCache {
    rendered_pages: Arc<Mutex<LruCache<PageCacheKey, CachedPage>>>,
    extracted_text: Arc<Mutex<LruCache<PageCacheKey, CachedText>>>,
    metadata: Arc<Mutex<LruCache<PageCacheKey, CachedMetadata>>>,
    outlines: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl PdfCache {
    /// Create a new PDF cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity.max(1)).unwrap();
        
        Self {
            rendered_pages: Arc::new(Mutex::new(LruCache::new(capacity))),
            extracted_text: Arc::new(Mutex::new(LruCache::new(capacity))),
            metadata: Arc::new(Mutex::new(LruCache::new(capacity))),
            outlines: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get cached rendered page
    pub fn get_rendered_page(&self, key: &PageCacheKey) -> Option<CachedPage> {
        self.rendered_pages.lock().unwrap().get(key).cloned()
    }

    /// Cache rendered page
    pub fn put_rendered_page(&self, key: PageCacheKey, pixmap: PdfPurrPixmap) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.rendered_pages.lock().unwrap().put(key, CachedPage { pixmap, timestamp });
    }

    /// Get cached text
    pub fn get_extracted_text(&self, key: &PageCacheKey) -> Option<CachedText> {
        self.extracted_text.lock().unwrap().get(key).cloned()
    }

    /// Cache extracted text
    pub fn put_extracted_text(&self, key: PageCacheKey, text: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.extracted_text.lock().unwrap().put(key, CachedText { text, timestamp });
    }

    /// Get cached metadata
    pub fn get_metadata(&self, key: &PageCacheKey) -> Option<CachedMetadata> {
        self.metadata.lock().unwrap().get(key).cloned()
    }

    /// Cache metadata
    pub fn put_metadata(&self, key: PageCacheKey, dims: (f32, f32)) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.metadata.lock().unwrap().put(key, CachedMetadata { dims, timestamp });
    }

    /// Get cached outlines
    pub fn get_outlines(&self, doc_id: &str) -> Option<Vec<String>> {
        self.outlines.lock().unwrap().get(doc_id).cloned()
    }

    /// Cache outlines
    pub fn put_outlines(&self, doc_id: String, outlines: Vec<String>) {
        self.outlines.lock().unwrap().insert(doc_id, outlines);
    }

    /// Clear all caches
    pub fn clear(&self) {
        self.rendered_pages.lock().unwrap().clear();
        self.extracted_text.lock().unwrap().clear();
        self.metadata.lock().unwrap().clear();
        self.outlines.lock().unwrap().clear();
    }

    /// Clear cache for a specific document
    pub fn clear_document(&self, doc_id: &str) {
        let mut rendered = self.rendered_pages.lock().unwrap();
        let mut text = self.extracted_text.lock().unwrap();
        let mut meta = self.metadata.lock().unwrap();
        let mut outlines = self.outlines.lock().unwrap();

        // LruCache doesn't have retain, so we collect keys to remove
        let rendered_keys: Vec<_> = rendered.iter().map(|(k, _)| k.clone()).collect();
        for key in rendered_keys {
            if key.doc_id == doc_id {
                rendered.pop(&key);
            }
        }

        let text_keys: Vec<_> = text.iter().map(|(k, _)| k.clone()).collect();
        for key in text_keys {
            if key.doc_id == doc_id {
                text.pop(&key);
            }
        }

        let meta_keys: Vec<_> = meta.iter().map(|(k, _)| k.clone()).collect();
        for key in meta_keys {
            if key.doc_id == doc_id {
                meta.pop(&key);
            }
        }

        outlines.remove(doc_id);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            rendered_pages: self.rendered_pages.lock().unwrap().len(),
            extracted_text: self.extracted_text.lock().unwrap().len(),
            metadata: self.metadata.lock().unwrap().len(),
            outlines: self.outlines.lock().unwrap().len(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub rendered_pages: usize,
    pub extracted_text: usize,
    pub metadata: usize,
    pub outlines: usize,
}

impl Default for PdfCache {
    fn default() -> Self {
        Self::new(32) // Default cache 32 pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_put_get() {
        let cache = PdfCache::new(4);
        let key = PageCacheKey::new("test.pdf".to_string(), 0);
        
        // Test metadata caching
        cache.put_metadata(key.clone(), (600.0, 800.0));
        let cached = cache.get_metadata(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().dims, (600.0, 800.0));
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = PdfCache::new(2);
        
        // Fill cache beyond capacity
        for i in 0..3 {
            let key = PageCacheKey::new("test.pdf".to_string(), i);
            cache.put_metadata(key, (600.0, 800.0));
        }
        
        // First entry should be evicted
        let key0 = PageCacheKey::new("test.pdf".to_string(), 0);
        let key1 = PageCacheKey::new("test.pdf".to_string(), 1);
        let key2 = PageCacheKey::new("test.pdf".to_string(), 2);
        
        assert!(cache.get_metadata(&key0).is_none());
        assert!(cache.get_metadata(&key1).is_some());
        assert!(cache.get_metadata(&key2).is_some());
    }

    #[test]
    fn test_clear_document() {
        let cache = PdfCache::new(10);
        
        // Add entries for two documents
        for i in 0..3 {
            let key1 = PageCacheKey::new("doc1.pdf".to_string(), i);
            let key2 = PageCacheKey::new("doc2.pdf".to_string(), i);
            cache.put_metadata(key1, (600.0, 800.0));
            cache.put_metadata(key2, (600.0, 800.0));
        }
        
        cache.clear_document("doc1.pdf");
        
        // doc1 entries should be gone
        assert!(cache.get_metadata(&PageCacheKey::new("doc1.pdf".to_string(), 0)).is_none());
        
        // doc2 entries should remain
        assert!(cache.get_metadata(&PageCacheKey::new("doc2.pdf".to_string(), 0)).is_some());
    }
}
