use crate::framebuffer::Pixmap as FramebufferPixmap;

/// Pixmap format.
#[derive(Debug, Clone, Copy)]
pub enum PixmapFormat {
    Gray,
    Rgb,
    Rgba,
}

/// Pixmap wrapper for PDFPurr rendering output.
pub struct Pixmap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub samples: usize,
    pub format: PixmapFormat,
}

impl Pixmap {
    pub fn new(data: Vec<u8>, format: PixmapFormat) -> Self {
        // PDFPurr renders to RGBA, we need to extract dimensions
        // For now, assume a default size
        let width = 0;
        let height = 0;
        let samples = match format {
            PixmapFormat::Gray => 1,
            PixmapFormat::Rgb => 3,
            PixmapFormat::Rgba => 4,
        };
        
        Pixmap {
            width,
            height,
            data,
            samples,
            format,
        }
    }

    /// Converts to framebuffer pixmap.
    pub fn to_framebuffer(&self) -> FramebufferPixmap {
        FramebufferPixmap {
            width: self.width,
            height: self.height,
            samples: self.samples,
            data: self.data.clone(),
        }
    }
}
