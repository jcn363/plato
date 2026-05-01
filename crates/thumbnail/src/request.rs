use std::path::PathBuf;
use std::sync::mpsc::Sender;

use crate::thumbnail::error::ThumbnailResult;

#[derive(Debug, Clone)]
pub struct ThumbnailRequest {
    pub file_path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub dimensions: (u32, u32),
    pub response_tx: Sender<ThumbnailResult<PathBuf>>,
}

impl ThumbnailRequest {
    pub fn new(
        file_path: PathBuf,
        thumbnail_path: PathBuf,
        dimensions: (u32, u32),
        response_tx: Sender<ThumbnailResult<PathBuf>>,
    ) -> Self {
        Self {
            file_path,
            thumbnail_path,
            dimensions,
            response_tx,
        }
    }

    /// Validates the thumbnail request parameters
    pub fn validate(&self) -> ThumbnailResult<()> {
        // Validate file path
        if self.file_path.as_os_str().is_empty() {
            return Err(crate::thumbnail::error::ThumbnailError::invalid_path(
                "empty file path",
            ));
        }

        // Validate thumbnail path
        if self.thumbnail_path.as_os_str().is_empty() {
            return Err(crate::thumbnail::error::ThumbnailError::invalid_path(
                "empty thumbnail path",
            ));
        }

        // Validate dimensions
        let (width, height) = self.dimensions;
        if width == 0 || height == 0 {
            return Err(crate::thumbnail::error::ThumbnailError::invalid_dimensions(
                width, height,
            ));
        }

        // Validate reasonable dimension bounds
        const MAX_DIMENSION: u32 = 2000;
        if width > MAX_DIMENSION || height > MAX_DIMENSION {
            return Err(crate::thumbnail::error::ThumbnailError::invalid_dimensions(
                width, height,
            ));
        }

        Ok(())
    }
}
