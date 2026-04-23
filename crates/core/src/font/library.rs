use crate::font::face::Font as NewFont;
use crate::font::skrifa_wrapper;
use anyhow::Result;
use std::path::Path;
use std::rc::Rc;

/// Simple font library using skrifa for font loading.
pub struct FontLibrary;

impl FontLibrary {
    pub fn new() -> Result<Self> {
        Ok(FontLibrary)
    }

    pub fn new_library() -> Result<Self> {
        Self::new()
    }

    pub fn new_face<P: AsRef<Path>>(&self, path: P, index: i32) -> Result<skrifa_wrapper::Face> {
        let data = std::fs::read(path.as_ref())?;
        skrifa_wrapper::Face::from_memory(data, index as u32)
    }

    pub fn new_memory_face(&self, data: &[u8], index: i32) -> Result<skrifa_wrapper::Face> {
        if data.is_empty() {
            return Err(anyhow::format_err!("Font data cannot be empty"));
        }
        skrifa_wrapper::Face::from_memory(data.to_vec(), index as u32)
    }
}

pub struct FontOpener(Rc<FontLibrary>);

impl FontOpener {
    pub fn new() -> Result<Self> {
        FontLibrary::new().map(|lib| FontOpener(Rc::new(lib)))
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<NewFont> {
        let face = self.0.new_face(path.as_ref(), 0)?;
        Ok(NewFont::new(face))
    }

    pub fn open_memory(&self, buf: &[u8]) -> Result<NewFont> {
        if buf.is_empty() {
            return Err(anyhow::format_err!("Font data cannot be empty"));
        }
        let face = self.0.new_memory_face(buf, 0)?;
        Ok(NewFont::new(face))
    }
}
