/// Text extraction wrapper for PDFPurr.
/// PDFPurr provides text extraction with positioning information.

use pdfpurr::TextRun;

/// Text page containing blocks of text.
pub struct TextPage {
    text: String,
    runs: Vec<TextRun>,
}

impl TextPage {
    pub fn new(text: String) -> Self {
        TextPage {
            text,
            runs: Vec::new(),
        }
    }

    pub fn with_runs(text: String, runs: Vec<TextRun>) -> Self {
        TextPage { text, runs }
    }

    /// Returns an iterator over text blocks.
    pub fn blocks(&self) -> TextBlockIter {
        TextBlockIter::new(&self.text)
    }
}

/// Iterator over text blocks.
pub struct TextBlockIter {
    lines: Vec<String>,
    index: usize,
}

impl TextBlockIter {
    fn new(text: &str) -> Self {
        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        TextBlockIter {
            lines,
            index: 0,
        }
    }
}

impl Iterator for TextBlockIter {
    type Item = TextBlock;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.lines.len() {
            let line = self.lines[self.index].clone();
            self.index += 1;
            Some(TextBlock::new(line))
        } else {
            None
        }
    }
}

/// Text block (paragraph or similar grouping).
pub struct TextBlock {
    text: String,
}

impl TextBlock {
    pub fn new(text: String) -> Self {
        TextBlock { text }
    }

    /// Returns the bounding box of this block.
    pub fn bbox(&self) -> crate::geom::Boundary {
        // PDFPurr doesn't provide exact bounding boxes in the simple API
        // This would need to be extracted from TextRun data
        crate::geom::Boundary {
            min: crate::geom::Vec2::zero(),
            max: crate::geom::Vec2::zero(),
        }
    }

    /// Returns the block kind (text, image, etc.).
    pub fn kind(&self) -> i32 {
        0 // Text block
    }

    /// Returns an iterator over lines in this block.
    pub fn lines(&self) -> TextLineIter {
        TextLineIter::new(&self.text)
    }
}

/// Iterator over text lines.
pub struct TextLineIter {
    words: Vec<String>,
    index: usize,
}

impl TextLineIter {
    fn new(text: &str) -> Self {
        let words: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        TextLineIter {
            words,
            index: 0,
        }
    }
}

impl Iterator for TextLineIter {
    type Item = TextLine;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.words.len() {
            let word = self.words[self.index].clone();
            self.index += 1;
            Some(TextLine::new(word))
        } else {
            None
        }
    }
}

/// Text line.
pub struct TextLine {
    text: String,
}

impl TextLine {
    pub fn new(text: String) -> Self {
        TextLine { text }
    }

    /// Returns the bounding box of this line.
    pub fn bbox(&self) -> crate::geom::Boundary {
        crate::geom::Boundary {
            min: crate::geom::Vec2::zero(),
            max: crate::geom::Vec2::zero(),
        }
    }

    /// Returns an iterator over characters in this line.
    pub fn chars(&self) -> TextCharIter {
        TextCharIter::new(&self.text)
    }
}

/// Iterator over text characters.
pub struct TextCharIter {
    chars: Vec<char>,
    index: usize,
}

impl TextCharIter {
    fn new(text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        TextCharIter {
            chars,
            index: 0,
        }
    }
}

impl Iterator for TextCharIter {
    type Item = TextChar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.chars.len() {
            let c = self.chars[self.index];
            self.index += 1;
            Some(TextChar::new(c))
        } else {
            None
        }
    }
}

/// Text character with position information.
pub struct TextChar {
    char_code: u32,
}

impl TextChar {
    pub fn new(c: char) -> Self {
        TextChar {
            char_code: c as u32,
        }
    }

    pub fn char_code(&self) -> i32 {
        self.char_code as i32
    }

    pub fn quad(&self) -> super::FzQuad {
        super::FzQuad::default()
    }
}

/// Placeholder for FzQuad type.
#[derive(Debug, Clone, Default)]
pub struct FzQuad {
    pub ul: super::FzPoint,
    pub ur: super::FzPoint,
    pub ll: super::FzPoint,
    pub lr: super::FzPoint,
}

/// Placeholder for FzPoint type.
#[derive(Debug, Clone, Default)]
pub struct FzPoint {
    pub x: f32,
    pub y: f32,
}

/// Block kind constants
pub const FZ_PAGE_BLOCK_IMAGE: i32 = 1;
pub const FZ_PAGE_BLOCK_TEXT: i32 = 2;

/// Text option flags
pub const FZ_TEXT_PRESERVE_IMAGES: i32 = 1;
