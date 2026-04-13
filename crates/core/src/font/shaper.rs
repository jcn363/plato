use crate::font::harfbuzz::{Buffer, Font};

pub struct Shaper(Buffer);

impl Shaper {
    pub fn new() -> Self {
        Shaper(Buffer::new())
    }

    pub fn shape(&mut self, font: &Font, features: &[crate::font::harfbuzz_sys::HbFeature]) {
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

    pub fn guess_segment_properties(&mut self) {
        self.0.guess_segment_properties();
    }

    pub fn script(&self) -> crate::font::harfbuzz_sys::HbScript {
        self.0.script()
    }

    pub fn add_utf8(&mut self, text: &str, offset: usize, len: usize) {
        self.0.add_utf8(text, offset, len);
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
