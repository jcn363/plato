use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid character: {0}")]
    InvalidCharacter(char, Option<usize>, Option<usize>),
    #[error("Missing column in index: {0}")]
    MissingColumnInIndex(usize),
    #[error("Invalid file format: {0}")]
    InvalidFileFormat(String, Option<String>),
    #[error("Memory error")]
    MemoryError,
    #[error("Word not found: {0}")]
    WordNotFound(String),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Deflate error: {0}")]
    DeflateError(String),
    #[error("Format error: {0}")]
    Format(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("AI error: {0}")]
    Ai(#[from] anyhow::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Battery error: {0}")]
    Battery(String),
    #[error("Document error: {0}")]
    Document(String),
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("Pdf error: {0}")]
    Pdf(String),
    #[error("Unknown error")]
    Unknown,
}

pub type PlatoResult<T> = Result<T, PlatoError>;
