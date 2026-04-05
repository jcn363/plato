use crate::document::mupdf_sys::*;
use std::ptr;

/// Safe wrapper around an MuPDF text page with RAII cleanup.
pub struct TextPage {
    pub(crate) ctx: *mut FzContext,
    pub(crate) text_page: *mut FzTextPage,
}

impl TextPage {
    /// Get the raw FFI text page pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzTextPage {
        self.text_page
    }

    /// Get the first text block.
    pub fn first_block(&self) -> Option<TextBlock> {
        unsafe {
            let block = (*self.text_page).first_block;
            if block.is_null() {
                None
            } else {
                Some(TextBlock {
                    ctx: self.ctx,
                    block,
                })
            }
        }
    }

    /// Iterate over all text blocks.
    pub fn blocks(&self) -> TextBlockIter {
        TextBlockIter {
            ctx: self.ctx,
            current: unsafe { (*self.text_page).first_block },
        }
    }
}

impl Drop for TextPage {
    fn drop(&mut self) {
        if !self.text_page.is_null() {
            unsafe { fz_drop_stext_page(self.ctx, self.text_page) }
        }
    }
}

/// Iterator over text blocks in a text page.
pub struct TextBlockIter {
    ctx: *mut FzContext,
    current: *mut FzTextBlock,
}

impl Iterator for TextBlockIter {
    type Item = TextBlock;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let block = TextBlock {
                ctx: self.ctx,
                block: self.current,
            };
            self.current = unsafe { (*self.current).next };
            Some(block)
        }
    }
}

/// Safe wrapper around an MuPDF text block. Not owned — borrows from TextPage.
pub struct TextBlock {
    ctx: *mut FzContext,
    block: *mut FzTextBlock,
}

impl TextBlock {
    /// Get the raw FFI text block pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzTextBlock {
        self.block
    }

    /// Get the block kind (text, image, struct, vector, grid).
    #[inline]
    pub fn kind(&self) -> libc::c_int {
        unsafe { (*self.block).kind }
    }

    /// Get the block's bounding box.
    #[inline]
    pub fn bbox(&self) -> FzRect {
        unsafe { (*self.block).bbox }
    }

    /// Get the first text line if this is a text block.
    pub fn first_line(&self) -> Option<TextLine> {
        unsafe {
            if (*self.block).kind != FZ_PAGE_BLOCK_TEXT {
                return None;
            }
            let line = (*self.block).u.text.first_line;
            if line.is_null() {
                None
            } else {
                Some(TextLine {
                    ctx: self.ctx,
                    line,
                })
            }
        }
    }

    /// Iterate over all text lines in this block (only valid for text blocks).
    pub fn lines(&self) -> TextLineIter {
        let first = unsafe {
            if (*self.block).kind == FZ_PAGE_BLOCK_TEXT {
                (*self.block).u.text.first_line
            } else {
                ptr::null_mut()
            }
        };
        TextLineIter {
            ctx: self.ctx,
            current: first,
        }
    }
}

/// Iterator over text lines in a text block.
pub struct TextLineIter {
    ctx: *mut FzContext,
    current: *mut FzTextLine,
}

impl Iterator for TextLineIter {
    type Item = TextLine;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let line = TextLine {
                ctx: self.ctx,
                line: self.current,
            };
            self.current = unsafe { (*self.current).next };
            Some(line)
        }
    }
}

/// Safe wrapper around an MuPDF text line. Not owned — borrows from TextBlock.
pub struct TextLine {
    ctx: *mut FzContext,
    line: *mut FzTextLine,
}

impl TextLine {
    /// Get the raw FFI text line pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzTextLine {
        self.line
    }

    /// Get the line's bounding box.
    #[inline]
    pub fn bbox(&self) -> FzRect {
        unsafe { (*self.line).bbox }
    }

    /// Get the first text character in this line.
    pub fn first_char(&self) -> Option<TextChar> {
        unsafe {
            let chr = (*self.line).first_char;
            if chr.is_null() {
                None
            } else {
                Some(TextChar {
                    ctx: self.ctx,
                    text_char: chr,
                })
            }
        }
    }

    /// Iterate over all characters in this line.
    pub fn chars(&self) -> TextCharIter {
        TextCharIter {
            ctx: self.ctx,
            current: unsafe { (*self.line).first_char },
        }
    }
}

/// Iterator over text characters in a text line.
pub struct TextCharIter {
    ctx: *mut FzContext,
    current: *mut FzTextChar,
}

impl Iterator for TextCharIter {
    type Item = TextChar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let text_char = TextChar {
                ctx: self.ctx,
                text_char: self.current,
            };
            self.current = unsafe { (*self.current).next };
            Some(text_char)
        }
    }
}

/// Safe wrapper around an MuPDF text character. Not owned — borrows from TextLine.
pub struct TextChar {
    #[allow(dead_code)]
    ctx: *mut FzContext,
    text_char: *mut FzTextChar,
}

impl TextChar {
    /// Get the raw FFI text char pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzTextChar {
        self.text_char
    }

    /// Get the character code.
    #[inline]
    pub fn char_code(&self) -> libc::c_int {
        unsafe { (*self.text_char).c }
    }

    /// Get the character's quad (bounding quadrilateral).
    #[inline]
    pub fn quad(&self) -> FzQuad {
        unsafe { (*self.text_char).quad }
    }
}
