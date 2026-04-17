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

/// OpenType feature specification wrapper.
#[derive(Debug, Clone, Copy)]
pub struct FeatureWrapper {
    pub tag: u32,
    pub value: u32,
}

/// Create a feature from a string specification.
pub fn feature_from_string(s: &str) -> Option<FeatureWrapper> {
    let (tag_str, value_str) = if let Some(eq_pos) = s.find('=') {
        (&s[..eq_pos], &s[eq_pos + 1..])
    } else {
        (s, "1")
    };

    if tag_str.len() != 4 {
        return None;
    }

    let bytes = tag_str.as_bytes();
    let tag = ((bytes[0] as u32) << 24)
        | ((bytes[1] as u32) << 16)
        | ((bytes[2] as u32) << 8)
        | bytes[3] as u32;

    let value = value_str.parse::<u32>().ok()?;

    Some(FeatureWrapper { tag, value })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_from_string() {
        let f = feature_from_string("liga").unwrap();
        assert_eq!(f.tag, 0x6c696761); // "liga"
        assert_eq!(f.value, 1);

        let f = feature_from_string("dlig=0").unwrap();
        assert_eq!(f.tag, 0x646c6967); // "dlig"
        assert_eq!(f.value, 0);
    }
}
