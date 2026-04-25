//! PDFPurr page wrapper

use anyhow::Result;
use pdfpurr::rendering::{RenderOptions, Renderer};
use pdfpurr::Document as PdfPurrDoc;
use std::sync::Arc;

use super::outline::Link;
use super::text::TextPage;
use super::types::{FzPoint, FzQuad, FzRect, PdfPurrPixmap, PixmapFormat};
use crate::document::cache::{PageCacheKey, PdfCache};

/// Wrapper around PDFPurr page
pub struct Page<'a> {
    doc: &'a PdfPurrDoc,
    index: usize,
    cache_key: PageCacheKey,
    cache: Option<Arc<PdfCache>>,
    lopdf_doc: Option<&'a lopdf::Document>,
}

impl<'a> Page<'a> {
    pub fn new(
        doc: &'a PdfPurrDoc,
        index: usize,
        cache_key: PageCacheKey,
        cache: Option<Arc<PdfCache>>,
        lopdf_doc: Option<&'a lopdf::Document>,
    ) -> Self {
        Self {
            doc,
            index,
            cache_key,
            cache,
            lopdf_doc,
        }
    }

    pub fn to_text_page(&self, _options: Option<&()>) -> Option<TextPage> {
        // Extract text runs from PDFPurr
        self.doc
            .extract_text_runs(self.index)
            .ok()
            .map(TextPage::new)
    }

    pub fn load_links(&self) -> Option<Link> {
        // Extract links from the page's annotation dictionary using lopdf
        if let Some(lopdf_doc) = self.lopdf_doc {
            let pages = lopdf_doc.get_pages();
            let page_id = pages.get(&(self.index as u32 + 1)).copied()?;
            
            let mut annotations = Vec::new();
            
            if let Ok(lopdf_annots) = lopdf_doc.get_page_annotations(page_id) {
                for annot in lopdf_annots {
                    // Check if this is a Link annotation
                    let subtype = annot.get_deref(b"Subtype", lopdf_doc)
                        .and_then(|obj| obj.as_name())
                        .map(|name| String::from_utf8_lossy(name).to_string())
                        .unwrap_or_default();
                    
                    if subtype == "Link" {
                        // Extract bounding box
                        let rect = annot.get_deref(b"Rect", lopdf_doc)
                            .and_then(|obj| obj.as_array())
                            .and_then(|arr| {
                                let coords: Vec<f64> = arr.iter()
                                    .filter_map(|obj| obj.as_i64().ok())
                                    .map(|i| i as f64)
                                    .collect();
                                if coords.len() == 4 {
                                    Ok([coords[0], coords[1], coords[2], coords[3]])
                                } else {
                                    Err(lopdf::Error::Syntax("Invalid rect coordinates".to_string()))
                                }
                            })
                            .unwrap_or([0.0, 0.0, 0.0, 0.0]);
                        
                        // Extract link URI or destination
                        let uri = annot.get_deref(b"A", lopdf_doc)
                            .and_then(|obj| obj.as_dict())
                            .map(|dict| {
                                // Try URI action
                                if let Ok(uri_obj) = dict.get(b"URI") {
                                    uri_obj.as_str().ok().map(|s| String::from_utf8_lossy(s).to_string())
                                } else {
                                    // Try GoTo action (destination)
                                    dict.get(b"D").ok().and_then(|dest| {
                                        // Convert destination to page reference
                                        if let Ok(dest_arr) = dest.as_array() {
                                            if let Some(page_ref) = dest_arr.first() {
                                                page_ref.as_i64().ok().map(|page_num| {
                                                    format!("#page{}", page_num + 1)
                                                })
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    })
                                }
                            })
                            .unwrap_or(None);
                        
                        // Create PDFPurr Annotation
                        annotations.push(pdfpurr::structure::Annotation {
                            subtype: "Link".to_string(),
                            rect,
                            contents: None,
                            flags: 0,
                            color: None,
                            author: None,
                            modified_date: None,
                            uri,
                            quad_points: Vec::new(),
                        });
                    }
                }
            }
            
            Some(Link::new(annotations, 0))
        } else {
            // Fallback to empty link list if lopdf document not available
            Some(Link::new(Vec::new(), 0))
        }
    }

    pub fn first_annot(&self) -> Option<()> {
        // Extract annotations from the page's annotation dictionary using lopdf
        if let Some(lopdf_doc) = self.lopdf_doc {
            let pages = lopdf_doc.get_pages();
            let page_id = pages.get(&(self.index as u32 + 1)).copied()?;
            
            if let Ok(annotations) = lopdf_doc.get_page_annotations(page_id) {
                // Return Some(()) if there are any annotations
                if !annotations.is_empty() {
                    Some(())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn render_pixmap(
        &self,
        _matrix: f32,
        _color_space: PixmapFormat,
        _flags: i32,
    ) -> Result<PdfPurrPixmap> {
        // Check cache first
        if let Some(ref cache) = self.cache {
            if let Some(cached) = cache.get_rendered_page(&self.cache_key) {
                return Ok(cached.pixmap);
            }
        }

        let options = RenderOptions {
            dpi: 72.0 * _matrix as f64,
            background: [255, 255, 255, 255],
        };
        let renderer = Renderer::new(self.doc, options);
        let pixmap = renderer
            .render_page(self.index)
            .map_err(|e| anyhow::format_err!("Failed to render page: {}", e))?;
        // PDFPurr returns tiny-skia::pixmap::Pixmap (0.11.4), but we need tiny_skia::Pixmap (0.12.0)
        // Convert by creating a new Pixmap with the same data using tiny-skia 0.12.0 API
        let width = pixmap.width();
        let height = pixmap.height();
        let data = pixmap.data();
        let mut converted = tiny_skia::Pixmap::new(width, height)
            .ok_or_else(|| anyhow::format_err!("Failed to create pixmap"))?;
        // Convert u8 data to PremultipliedColorU8 for tiny-skia 0.12.0
        let colors: Vec<tiny_skia::PremultipliedColorU8> = data
            .chunks(4)
            .map(|chunk| {
                if chunk.len() == 4 {
                    tiny_skia::PremultipliedColorU8::from_rgba(
                        chunk[0], chunk[1], chunk[2], chunk[3],
                    )
                    .unwrap_or_else(|| {
                        // Fallback to black (0, 0, 0, 255) - always valid
                        tiny_skia::PremultipliedColorU8::from_rgba(0, 0, 0, 255).unwrap()
                    })
                } else {
                    // Fallback to black (0, 0, 0, 255) - always valid
                    tiny_skia::PremultipliedColorU8::from_rgba(0, 0, 0, 255).unwrap()
                }
            })
            .collect();
        converted.pixels_mut().copy_from_slice(&colors);

        let result = PdfPurrPixmap::new(converted);

        // Cache the result
        if let Some(ref cache) = self.cache {
            cache.put_rendered_page(self.cache_key.clone(), result.clone());
        }

        Ok(result)
    }

    pub fn dims(&self) -> (f32, f32) {
        // Check cache first
        if let Some(ref cache) = self.cache {
            if let Some(cached) = cache.get_metadata(&self.cache_key) {
                return cached.dims;
            }
        }

        // Get page dimensions from PDFPurr
        // PDFPurr stores page dimensions in the page dictionary
        // For now, use default dimensions - this should be improved
        // by accessing the PDF's MediaBox directly
        let dims = (600.0, 800.0);

        // Cache the dimensions
        if let Some(ref cache) = self.cache {
            cache.put_metadata(self.cache_key.clone(), dims);
        }

        dims
    }

    pub fn width(&self) -> f32 {
        self.dims().0
    }

    pub fn height(&self) -> f32 {
        self.dims().1
    }

    pub fn media_box(&self) -> FzRect {
        let (width, height) = self.dims();
        FzRect {
            x0: 0.0,
            y0: 0.0,
            x1: width,
            y1: height,
        }
    }

    pub fn search(&self, needle: &str) -> Option<Vec<FzQuad>> {
        if needle.is_empty() {
            return None;
        }
        // Basic search implementation using PDFPurr text extraction
        let text_runs = self.doc.extract_text_runs(self.index).ok()?;
        let text: String = text_runs.iter().map(|r| r.text.as_str()).collect();

        if text.contains(needle) {
            // Return page-level quad if text is found
            // Full implementation would need character-level position tracking
            Some(vec![FzQuad {
                ul: FzPoint { x: 0.0, y: 0.0 },
                ur: FzPoint { x: 600.0, y: 0.0 },
                ll: FzPoint { x: 0.0, y: 800.0 },
                lr: FzPoint { x: 600.0, y: 800.0 },
            }])
        } else {
            Some(Vec::new())
        }
    }

    pub fn images(&self) -> Option<Vec<FzRect>> {
        // PDFPurr doesn't have a direct image extraction API in version 0.4.0
        // This would require accessing the PDF's XObject dictionary directly
        // This is a Phase 4 feature - for now return empty list
        Some(Vec::new())
    }

    pub fn char_count(&self) -> usize {
        self.to_text_page(None).map(|tp| tp.chars()).unwrap_or(0)
    }
}
