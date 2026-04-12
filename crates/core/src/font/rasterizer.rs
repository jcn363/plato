use crate::font::freetype::Face;
use crate::font::freetype_sys::{FT_LOAD_RENDER, FT_LOAD_NO_HINTING};
use anyhow::Result;

pub struct Rasterizer<'a> {
    face: &'a Face,
}

impl<'a> Rasterizer<'a> {
    pub fn new(face: &'a Face) -> Self {
        Rasterizer { face }
    }

    pub fn load_glyph(&self, glyph_index: u32) -> Result<()> {
        self.face.load_glyph(glyph_index, FT_LOAD_RENDER | FT_LOAD_NO_HINTING)
    }

    pub fn bitmap(&self) -> &crate::font::freetype_sys::FtBitmap {
        &self.face.glyph().bitmap
    }
}
