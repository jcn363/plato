use crate::font::harfbuzz::{Font, Buffer};
use crate::font::harfbuzz_sys::HbFeature;

pub struct Shaper(Buffer);

impl Shaper {
    pub fn new() -> Self {
        Shaper(Buffer::new())
    }

    pub fn shape(&mut self, font: &Font, text: &str, features: &[HbFeature]) {
        self.0.clear();
        self.0.add_utf8(text, 0, text.len());
        self.0.guess_segment_properties();
        self.0.shape(font, features);
    }

    pub fn create_buffer() -> Buffer {
        Buffer::new()
    }

    pub fn destroy_buffer(buffer: Buffer) {
        drop(buffer);
    }

    pub fn set_direction(&mut self, direction: crate::font::harfbuzz_sys::HbDirection) {
        self.0.set_direction(direction);
    }

    pub fn length(&self) -> u32 {
        self.0.length()
    }

    pub fn glyph_infos(&self) -> Vec<crate::font::harfbuzz_sys::HbGlyphInfo> {
        self.0.glyph_infos()
    }

    pub fn glyph_positions(&self) -> Vec<crate::font::harfbuzz_sys::HbGlyphPosition> {
        self.0.glyph_positions()
    }
    
    pub fn buffer(&self) -> &Buffer {
        &self.0
    }
}
