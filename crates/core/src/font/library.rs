use std::rc::Rc;
use crate::font::freetype::Library;
use anyhow::Result;

pub struct FontLibrary(Library);

impl FontLibrary {
    pub fn new() -> Result<Self> {
        Library::new().map(FontLibrary).map_err(|e| e.into())
    }

    pub fn new_library() -> Result<Self> {
        Self::new()
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
}
