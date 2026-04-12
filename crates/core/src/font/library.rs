use std::rc::Rc;
use std::path::Path;
use crate::font::freetype::{Library, Face};
use anyhow::Result;

pub struct FontLibrary(Library);

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
