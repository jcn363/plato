use crate::font::rustybuzz_wrapper::Buffer;
use rustybuzz::Direction;

pub struct Shaper(Buffer);

impl Shaper {
    pub fn new() -> Self {
        Shaper(Buffer::new())
    }

    pub fn create_buffer() -> Buffer {
        Buffer::new()
    }

    pub fn destroy_buffer(_buffer: Buffer) {
        // Buffer is dropped automatically
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.0.set_direction(direction);
    }

    pub fn guess_segment_properties(&mut self) {
        self.0.guess_segment_properties();
    }

    pub fn script(&self) -> rustybuzz::Script {
        self.0.script()
    }

    pub fn add_utf8(&mut self, text: &str, offset: usize, len: usize) {
        if text.is_empty() {
            return;
        }
        self.0.add_utf8(text, offset, len);
    }

    pub fn buffer(&self) -> &Buffer {
        &self.0
    }
}

impl Default for Shaper {
    fn default() -> Self {
        Self::new()
    }
}
