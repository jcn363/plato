//! OCR module for scanned PDFs (Desktop/Android only)

use anyhow::Result;
use std::path::Path;

#[cfg(feature = "ocr")]
use pdfpurr::ocr::tesseract_engine::TesseractEngine;

/// OCR manager
pub struct OcrManager {
    language: String,
}

impl OcrManager {
    pub fn new() -> Self {
        Self {
            language: "eng".to_string(),
        }
    }

    pub fn with_language(lang: &str) -> Self {
        Self {
            language: lang.to_string(),
        }
    }

    #[cfg(feature = "ocr")]
    pub fn ocr_page<P: AsRef<Path>>(&self, pdf_path: P, page_num: usize) -> Result<String> {
        let mut doc = pdfpurr::Document::open(pdf_path.as_ref())?;
        let engine = TesseractEngine::new(&self.language, None);
        let config = pdfpurr::ocr::OcrConfig::default();
        doc.ocr_page(page_num, &engine, &config)?;
        // After OCR, extract the text from the page
        let text = doc
            .extract_page_text(page_num)
            .map_err(|e| anyhow::anyhow!("Failed to extract text: {}", e))?;
        Ok(text)
    }
}
