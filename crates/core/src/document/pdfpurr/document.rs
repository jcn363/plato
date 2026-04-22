use anyhow::{Context, Result};
use std::path::Path;

use pdfpurr::Document as PdfPurrDocument;

/// PDF document wrapper using PDFPurr.
pub struct Document {
    inner: PdfPurrDocument,
}

impl Document {
    /// Opens a PDF file from the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let inner = PdfPurrDocument::open(path_ref)
            .with_context(|| format!("Failed to open PDF: {}", path_ref.display()))?;
        Ok(Document { inner })
    }

    /// Opens a PDF from memory buffer.
    pub fn open_memory(magic: &str, buf: &[u8]) -> Result<Self> {
        let inner = PdfPurrDocument::from_bytes(buf)
            .with_context(|| "Failed to open PDF from memory")?;
        Ok(Document { inner })
    }

    /// Opens a password-protected PDF from memory.
    pub fn open_memory_with_password(magic: &str, buf: &[u8], password: &[u8]) -> Result<Self> {
        let inner = PdfPurrDocument::from_bytes_with_password(buf, password)
            .with_context(|| "Failed to open password-protected PDF")?;
        Ok(Document { inner })
    }

    /// Returns the number of pages in the document.
    pub fn page_count(&self) -> i32 {
        self.inner.page_count().unwrap_or(0) as i32
    }

    /// Loads a page by index (0-based).
    pub fn load_page(&self, index: i32) -> Result<super::Page> {
        let page = self.inner.get_page(index as usize)
            .with_context(|| format!("Failed to load page {}", index))?;
        Ok(super::Page::new(page))
    }

    /// Loads the document outline (table of contents).
    pub fn load_outline(&self) -> Option<super::Outline> {
        let outlines = self.inner.outlines();
        if outlines.is_empty() {
            None
        } else {
            Some(super::Outline::new(outlines))
        }
    }

    /// Looks up metadata by key.
    pub fn lookup_metadata(&self, key: &str) -> Option<String> {
        let meta = self.inner.metadata();
        match key.to_lowercase().as_str() {
            "title" => meta.title.clone(),
            "author" => meta.author.clone(),
            "subject" => meta.subject.clone(),
            "keywords" => meta.keywords.clone(),
            "creator" => meta.creator.clone(),
            "producer" => meta.producer.clone(),
            _ => None,
        }
    }

    /// Returns the document title.
    pub fn title(&self) -> Option<String> {
        self.inner.metadata().title.clone()
    }

    /// Returns the document author.
    pub fn author(&self) -> Option<String> {
        self.inner.metadata().author.clone()
    }

    /// Checks if the document is password-protected.
    pub fn needs_password(&self) -> bool {
        self.inner.metadata().encrypted
    }

    /// Checks if the document is reflowable (EPUB/XHTML).
    /// PDF documents are not reflowable.
    pub fn is_reflowable(&self) -> bool {
        false
    }

    /// Sets the document layout dimensions.
    pub fn layout(&mut self, width: f32, height: f32) {
        // PDFPurr doesn't have a direct layout method
        // This is a no-op for PDF documents
    }
}
