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
    
    pub fn buffer(&self) -> &Buffer {
        &self.0
    }
}
