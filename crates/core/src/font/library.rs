use crate::font::face::Font as NewFont;
use crate::font::freetype::{Face, Library};
use anyhow::Result;
use std::path::Path;
use std::rc::Rc;

pub struct FontLibrary(pub(crate) Library);

impl FontLibrary {
    pub fn new() -> Result<Self> {
        Library::new().map(FontLibrary).map_err(|e| e.into())
    }

    pub fn new_library() -> Result<Self> {
        Self::new()
    }

    pub fn new_face<P: AsRef<Path>>(&self, path: P, index: i32) -> Result<Face> {
        Face::from_path(self.library(), path.as_ref(), index)
    }

    pub fn new_memory_face(&self, data: &[u8], index: i32) -> Result<Face> {
        Face::from_memory(self.library(), data, index)
    }

    pub fn as_ptr(&self) -> *mut crate::font::freetype_sys::FtLibrary {
        self.0.as_ptr()
    }

    pub fn library(&self) -> &Library {
        &self.0
    }
}

pub struct FontOpener(Rc<FontLibrary>);

impl FontOpener {
    pub fn new() -> Result<Self> {
        FontLibrary::new().map(|lib| FontOpener(Rc::new(lib)))
    }

    pub fn library(&self) -> &Library {
        self.0.library()
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<NewFont> {
        let face = Face::from_path(self.0.library(), path.as_ref(), 0)?;
        Ok(NewFont::new(face))
    }

    pub fn open_memory(&self, buf: &[u8]) -> Result<NewFont> {
        let face = Face::from_memory(self.0.library(), buf, 0)?;
        Ok(NewFont::new(face))
    }
}
