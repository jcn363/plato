use crate::font::freetype_sys::FtError;
use std::fmt;

#[derive(Debug)]
pub struct FreetypeError(pub FtError);

impl fmt::Display for FreetypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FreeType error: {}", self.0)
    }
}

impl std::error::Error for FreetypeError {}

impl From<FtError> for FreetypeError {
    fn from(code: FtError) -> Self {
        FreetypeError(code)
    }
}
