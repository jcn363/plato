use plato_error::PlatoError;
use std::error;

#[derive(Debug)]
pub enum DictError {
    Plato(PlatoError),
    InvalidCharacter(char, Option<usize>, Option<usize>),
    MissingColumnInIndex(usize),
    InvalidFileFormat(String, Option<String>),
    MemoryError,
    WordNotFound(String),
    DeflateError(flate2::DecompressError),
}

impl std::fmt::Display for DictError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DictError::Plato(e) => e.fmt(f),
            DictError::InvalidCharacter(ch, line, pos) => {
                let mut ret = write!(f, "Invalid character {}", ch);
                if let Some(ln) = line {
                    ret = write!(f, " on line {}", ln);
                }
                if let Some(pos) = pos {
                    ret = write!(f, " at position {}", pos);
                }
                ret
            }
            DictError::MissingColumnInIndex(lnum) => write!(f, "line {}: not enough columns", lnum),
            DictError::InvalidFileFormat(ex, path) => {
                write!(f, "{}{}", path.as_deref().unwrap_or(""), ex)
            }
            DictError::MemoryError => write!(f, "not enough memory"),
            DictError::WordNotFound(word) => write!(f, "Word not found: {}", word),
            DictError::DeflateError(err) => write!(f, "Deflate error: {:?}", err),
        }
    }
}

impl error::Error for DictError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            DictError::Plato(e) => e.source(),
            _ => None,
        }
    }
}

impl From<PlatoError> for DictError {
    fn from(e: PlatoError) -> Self {
        DictError::Plato(e)
    }
}
impl From<std::io::Error> for DictError {
    fn from(err: std::io::Error) -> Self {
        DictError::Plato(PlatoError::Io(err))
    }
}
impl From<std::string::FromUtf8Error> for DictError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        DictError::Plato(PlatoError::Utf8Error(err))
    }
}
impl From<flate2::DecompressError> for DictError {
    fn from(err: flate2::DecompressError) -> Self {
        DictError::DeflateError(err)
    }
}
