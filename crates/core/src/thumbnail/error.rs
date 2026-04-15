use thiserror::Error;

pub type ThumbnailResult<T> = Result<T, ThumbnailError>;

#[derive(Debug, Error)]
pub enum ThumbnailError {
    #[error("Invalid file path: {path}")]
    InvalidPath { path: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Invalid thumbnail dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Document processing failed: {source}")]
    DocumentProcessing {
        #[from]
        source: anyhow::Error,
        path: String,
    },

    #[error("Failed to save thumbnail: {path}")]
    SaveFailed { path: String },

    #[error("Cache operation failed: {message}")]
    Cache { message: String },

    #[error("Thread pool error: {message}")]
    ThreadPool { message: String },

    #[error("Channel communication failed")]
    Channel,

    #[error("Resource limit exceeded: {resource}")]
    ResourceLimit { resource: String },

    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
        path: Option<String>,
    },
}

impl ThumbnailError {
    pub fn invalid_path<S: Into<String>>(path: S) -> Self {
        Self::InvalidPath { path: path.into() }
    }

    pub fn file_not_found<S: Into<String>>(path: S) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    pub fn invalid_dimensions(width: u32, height: u32) -> Self {
        Self::InvalidDimensions { width, height }
    }

    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    pub fn save_failed<S: Into<String>>(path: S) -> Self {
        Self::SaveFailed { path: path.into() }
    }

    pub fn cache<S: Into<String>>(message: S) -> Self {
        Self::Cache {
            message: message.into(),
        }
    }

    pub fn thread_pool<S: Into<String>>(message: S) -> Self {
        Self::ThreadPool {
            message: message.into(),
        }
    }

    pub fn resource_limit<S: Into<String>>(resource: S) -> Self {
        Self::ResourceLimit {
            resource: resource.into(),
        }
    }

    pub fn document_processing<S: Into<String>>(source: anyhow::Error, path: S) -> Self {
        Self::DocumentProcessing {
            source,
            path: path.into(),
        }
    }
}
