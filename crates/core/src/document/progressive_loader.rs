use crate::document::pdfpurr::Document as PdfPurrDocument;
use crate::framebuffer::Pixmap;
use crate::{log_error, log_info, log_warn};
use anyhow::{format_err, Error};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};

// Re-export from canonical source in consts::system per Single Source of Truth rule.
use crate::consts::system::{PAGE_CACHE_SIZE_MB, PRELOAD_AHEAD_PAGES, PRELOAD_BEHIND_PAGES};

#[derive(Debug, Clone)]
pub struct PageInfo {
    pub index: usize,
    pub width: i32,
    pub height: i32,
    pub loaded: bool,
    pub last_access: Instant,
}

#[derive(Debug)]
pub struct CachedPage {
    pub info: PageInfo,
    pub pixmap: Option<Pixmap>,
}

pub struct ProgressiveDocLoader {
    doc: Option<PdfPurrDocument>,
    _path: PathBuf,
    total_pages: usize,
    current_page: usize,
    is_linearized: bool,
    page_cache: RwLock<HashMap<usize, CachedPage>>,
    access_order: RwLock<VecDeque<usize>>,
    cache_size_bytes: RwLock<usize>,
    last_loading_error: Mutex<Option<String>>,
    enable_progressive: bool,
}

impl ProgressiveDocLoader {
    pub fn new(path: &Path) -> Result<Self, Error> {
        let doc =
            PdfPurrDocument::open(path).map_err(|e| format_err!("Failed to open PDF: {}", e))?;

        let total_pages = doc.page_count();
        let is_linearized = false;

        let loader = ProgressiveDocLoader {
            doc: Some(doc),
            _path: path.to_path_buf(),
            total_pages,
            current_page: 0,
            is_linearized,
            page_cache: RwLock::new(HashMap::new()),
            access_order: RwLock::new(VecDeque::new()),
            cache_size_bytes: RwLock::new(0),
            last_loading_error: Mutex::new(None),
            enable_progressive: true,
        };

        if !is_linearized {
            log_warn!(
                "Note: {} is not linearized. Progressive loading limited.",
                path.display()
            );
        }

        Ok(loader)
    }

    pub fn total_pages(&self) -> usize {
        self.total_pages
    }

    pub fn is_linearized(&self) -> bool {
        self.is_linearized
    }

    pub fn current_page(&self) -> usize {
        self.current_page
    }

    pub fn set_current_page(&mut self, page: usize) {
        if page < self.total_pages {
            self.current_page = page;
            self.preload_nearby_pages(page);
        }
    }

    fn get_cache_size_bytes(&self) -> usize {
        *self
            .cache_size_bytes
            .read()
            .expect("cache_size_bytes lock poisoned")
    }

    fn evict_lru_page(&self) -> Option<usize> {
        let mut access = self
            .access_order
            .write()
            .expect("access_order lock poisoned");
        let mut cache = self.page_cache.write().expect("page_cache lock poisoned");

        while let Some(&page_idx) = access.front() {
            if let Some(cached) = cache.get(&page_idx) {
                if cached.info.loaded {
                    if let Some(pixmap) = &cached.pixmap {
                        let size = pixmap.data.len();
                        let mut cached_bytes = self
                            .cache_size_bytes
                            .write()
                            .expect("cache_size_bytes lock poisoned");
                        *cached_bytes = cached_bytes.saturating_sub(size);
                    }
                }
                access.pop_front();
                cache.remove(&page_idx);
                return Some(page_idx);
            } else {
                access.pop_front();
            }
        }
        None
    }

    fn ensure_cache_space(&self, required_bytes: usize) {
        let max_cache = PAGE_CACHE_SIZE_MB * 1024 * 1024;

        while self.get_cache_size_bytes() + required_bytes > max_cache {
            if self.evict_lru_page().is_none() {
                break;
            }
        }
    }

    fn preload_nearby_pages(&self, current: usize) {
        if !self.enable_progressive {
            return;
        }

        let ahead_start = (current + 1).min(self.total_pages);
        let ahead_end = (current + 1 + PRELOAD_AHEAD_PAGES).min(self.total_pages);

        for page_idx in ahead_start..ahead_end {
            let _ = self.load_page_thumbnail(page_idx);
        }

        let behind_start = current.saturating_sub(PRELOAD_BEHIND_PAGES);
        let behind_end = current;

        for page_idx in behind_start..behind_end {
            let _ = self.load_page_thumbnail(page_idx);
        }
    }

    pub fn load_page_thumbnail(&self, page_idx: usize) -> Result<Pixmap, Error> {
        if page_idx >= self.total_pages {
            return Err(format_err!("Page index out of range"));
        }

        {
            let cache = self.page_cache.read().expect("page_cache lock poisoned");
            if let Some(cached) = cache.get(&page_idx) {
                if cached.info.loaded {
                    if let Some(ref pixmap) = cached.pixmap {
                        let mut access = self
                            .access_order
                            .write()
                            .expect("access_order lock poisoned");
                        access.retain(|&x| x != page_idx);
                        access.push_back(page_idx);
                        return Ok(pixmap.clone());
                    }
                }
            }
        }

        let size_bytes = (800 * 1200) as usize;
        self.ensure_cache_space(size_bytes);

        let doc = self
            .doc
            .as_ref()
            .ok_or_else(|| format_err!("Document not loaded"))?;
        let page = doc.load_page(page_idx as i32)?;

        let matrix = 800.0 / 600.0;
        let colorspace = crate::document::pdfpurr::PixmapFormat::Grayscale;

        let pdfpurr_pixmap = page.render_pixmap(matrix, colorspace, 0)?;

        let data_len = pdfpurr_pixmap.data().len();
        let pixmap = Pixmap {
            width: 800,
            height: 1200,
            samples: 1,
            data: pdfpurr_pixmap.data().to_vec(),
            update_flag: false,
        };

        let page_info = PageInfo {
            index: page_idx,
            width: 800,
            height: 1200,
            loaded: true,
            last_access: Instant::now(),
        };

        *self
            .cache_size_bytes
            .write()
            .expect("cache_size_bytes lock poisoned") += data_len;

        {
            let mut cache = self.page_cache.write().expect("page_cache lock poisoned");
            cache.insert(
                page_idx,
                CachedPage {
                    info: page_info,
                    pixmap: Some(pixmap.clone()),
                },
            );
        }

        {
            let mut access = self
                .access_order
                .write()
                .expect("access_order lock poisoned");
            access.push_back(page_idx);
        }

        Ok(pixmap)
    }

    pub fn preload_page(&self, page_idx: usize) {
        if let Err(e) = self.load_page_thumbnail(page_idx) {
            log_error!("Failed to preload page {}: {}", page_idx, e);
        }
    }

    pub fn get_page_count(&self) -> usize {
        self.total_pages
    }

    pub fn get_memory_usage(&self) -> usize {
        self.get_cache_size_bytes()
    }

    pub fn clear_cache(&self) {
        let mut cache = self.page_cache.write().expect("page_cache lock poisoned");
        let mut access = self
            .access_order
            .write()
            .expect("access_order lock poisoned");
        let mut size = self
            .cache_size_bytes
            .write()
            .expect("cache_size_bytes lock poisoned");

        cache.clear();
        access.clear();
        *size = 0;

        log_info!("Page cache cleared");
    }

    pub fn is_progressive_available(&self) -> bool {
        self.is_linearized
    }

    pub fn get_load_warning(&self) -> Option<String> {
        self.last_loading_error
            .lock()
            .expect("last_loading_error lock poisoned")
            .clone()
    }

    pub fn estimate_load_time(&self, page_idx: usize) -> Duration {
        if self.is_linearized {
            let distance = page_idx.abs_diff(self.current_page);
            Duration::from_millis(distance as u64 * 50)
        } else {
            Duration::from_millis(page_idx as u64 * 100)
        }
    }
}

pub struct ProgressiveLoaderOptions {
    pub enable_preloading: bool,
    pub max_cache_mb: usize,
    pub preload_ahead: usize,
    pub preload_behind: usize,
}

impl Default for ProgressiveLoaderOptions {
    fn default() -> Self {
        ProgressiveLoaderOptions {
            enable_preloading: true,
            max_cache_mb: PAGE_CACHE_SIZE_MB,
            preload_ahead: PRELOAD_AHEAD_PAGES,
            preload_behind: PRELOAD_BEHIND_PAGES,
        }
    }
}

pub fn open_progressive(path: &Path) -> Result<ProgressiveDocLoader, Error> {
    let metadata = std::fs::metadata(path)?;
    let file_size_mb = metadata.len() / (1024 * 1024);

    if file_size_mb > 100 {
        log_warn!(
            "Warning: Large file ({}MB). Loading may be slow.",
            file_size_mb
        );
    }

    ProgressiveDocLoader::new(path)
}

pub fn optimize_for_kobo(path: &Path) -> Result<ProgressiveDocLoader, Error> {
    let loader = ProgressiveDocLoader::new(path)?;

    if !loader.is_linearized() {
        log_info!("Note: Converting to linearized view for better performance...");
    }

    loader.preload_page(0);

    Ok(loader)
}
