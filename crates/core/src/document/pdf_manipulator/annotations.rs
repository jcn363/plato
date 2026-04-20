//! PDF Annotation Export Module
//!
//! Provides functionality for exporting PDF annotations to a new document.
//! Supports creating annotation copies with their content and positioning.

use super::mupdf;
use anyhow::{format_err, Error};
use std::path::{Path, PathBuf};

/// PDF annotation information
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PdfAnnotation {
    pub page: usize,
    pub annot_type: String,
    pub contents: String,
    pub rect: Option<(f32, f32, f32, f32)>,
    pub color: Option<(u8, u8, u8)>,
}

/// PDF annotation exporter
///
/// Exports annotations from a source PDF to a new output document.
pub struct PdfAnnotationExporter {
    source_doc: mupdf::Document,
    output_doc: mupdf::Document,
    file_path: PathBuf,
    total_pages: usize,
}

impl PdfAnnotationExporter {
    /// Create a new annotation exporter
    pub fn new(source_path: &Path, output_path: &Path) -> Result<PdfAnnotationExporter, Error> {
        let ctx = mupdf::MuPdfContext::new()?;

        let source_doc = ctx
            .open_document(source_path)
            .ok_or_else(|| format_err!("Failed to open source PDF: {}", source_path.display()))?;

        let total_pages = source_doc.page_count() as usize;

        let output_doc = ctx
            .new_pdf_document()
            .ok_or_else(|| format_err!("Failed to create output PDF"))?;

        Ok(PdfAnnotationExporter {
            source_doc,
            output_doc,
            file_path: output_path.to_path_buf(),
            total_pages,
        })
    }

    /// Get the total page count
    pub fn page_count(&self) -> usize {
        self.total_pages
    }

    /// Add an annotation to the output document
    pub fn add_annotation(&mut self, annot: PdfAnnotation) -> Result<(), Error> {
        if annot.page >= self.total_pages {
            return Err(format_err!("Page {} does not exist", annot.page + 1));
        }

        let page = self
            .source_doc
            .load_page(annot.page as i32)
            .map_err(|_| format_err!("Failed to load page {}", annot.page + 1))?;

        if mupdf::create_annot(self.source_doc.ctx(), page.as_ptr(), &annot.annot_type).is_some() {
            let pdf_annot =
                mupdf::create_annot(self.source_doc.ctx(), page.as_ptr(), &annot.annot_type);
            if let Some(pdf_annot) = pdf_annot {
                if !annot.contents.is_empty() {
                    pdf_annot.set_contents(&annot.contents);
                }

                if let Some((x0, y0, x1, y1)) = annot.rect {
                    pdf_annot.set_rect(mupdf::FzRect { x0, y0, x1, y1 });
                }
            }
        }

        Ok(())
    }

    /// Save the output document with annotations
    pub fn save(&self) -> Result<PathBuf, Error> {
        let opts = mupdf::FzWriteOptions::default();
        self.output_doc.save(&self.file_path, &opts, "pdf");

        Ok(self.file_path.clone())
    }
}
