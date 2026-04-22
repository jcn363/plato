use anyhow::{Context, Result};
use pdfpurr::Page as PdfPurrPage;

/// PDF page wrapper using PDFPurr.
pub struct Page {
    inner: PdfPurrPage,
}

impl Page {
    pub fn new(inner: PdfPurrPage) -> Self {
        Page { inner }
    }

    /// Returns the page dimensions (width, height).
    pub fn dims(&self) -> (f32, f32) {
        let media_box = self.inner.media_box();
        (media_box.width, media_box.height)
    }

    /// Returns the page width.
    pub fn width(&self) -> f32 {
        self.dims().0
    }

    /// Returns the page height.
    pub fn height(&self) -> f32 {
        self.dims().1
    }

    /// Converts the page to a text page for text extraction.
    pub fn to_text_page(&self) -> Result<super::TextPage> {
        let text = self.inner.extract_text()
            .with_context(|| "Failed to extract text from page")?;
        Ok(super::TextPage::new(text))
    }

    /// Loads links from the page.
    pub fn load_links(&self) -> Option<super::Link> {
        let annotations = self.inner.annotations();
        let links: Vec<_> = annotations
            .iter()
            .filter(|a| a.annotation_type == "Link")
            .cloned()
            .collect();
        
        if links.is_empty() {
            None
        } else {
            Some(super::Link::new(links))
        }
    }

    /// Renders the page to a pixmap.
    pub fn render_pixmap(&self, scale: f32, format: super::PixmapFormat, _flags: i32) -> Result<super::Pixmap> {
        use pdfpurr::{Renderer, RenderOptions};
        
        let renderer = Renderer::new(&self.inner.document(), RenderOptions {
            dpi: (72.0 * scale) as f64,
            ..Default::default()
        });
        
        let pixmap = renderer.render_page(self.inner.index())
            .with_context(|| "Failed to render page")?;
        
        Ok(super::Pixmap::new(pixmap, format))
    }
}
