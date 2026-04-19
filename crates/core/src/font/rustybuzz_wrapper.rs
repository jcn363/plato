use rustybuzz::{Direction, UnicodeBuffer};

/// Text shaping buffer wrapper.
pub struct Buffer {
    unicode_buffer: UnicodeBuffer,
}

impl Buffer {
    /// Create a new shaping buffer.
    pub fn new() -> Self {
        Buffer {
            unicode_buffer: UnicodeBuffer::new(),
        }
    }

    /// Add UTF-8 text to the buffer.
    pub fn add_utf8(&mut self, text: &str, _item_offset: usize, _item_length: usize) {
        self.unicode_buffer.push_str(text);
    }

    /// Set the text direction.
    pub fn set_direction(&mut self, direction: Direction) {
        self.unicode_buffer.set_direction(direction);
    }

    /// Guess segment properties.
    pub fn guess_segment_properties(&mut self) {
        self.unicode_buffer.guess_segment_properties();
    }

    /// Get the detected script.
    pub fn script(&self) -> rustybuzz::Script {
        self.unicode_buffer.script()
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.unicode_buffer = UnicodeBuffer::new();
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}
