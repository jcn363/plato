use crate::font::skrifa_wrapper::{self, Face};
use anyhow::Result;

/// Glyph rasterizer using skrifa.
pub struct Rasterizer<'a> {
    face: &'a Face,
    glyph_id: u32,
}

impl<'a> Rasterizer<'a> {
    pub fn new(face: &'a Face) -> Self {
        Rasterizer { face, glyph_id: 0 }
    }

    /// Load a glyph by index for rasterization.
    pub fn load_glyph(&mut self, glyph_index: u32) -> Result<()> {
        self.glyph_id = glyph_index;
        Ok(())
    }

    /// Get the rasterized glyph bitmap.
    pub fn bitmap(&self) -> Result<skrifa_wrapper::GlyphBitmap> {
        self.face.rasterize_glyph(self.glyph_id as u16, 0)
    }
}
