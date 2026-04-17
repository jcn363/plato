use crate::metadata::SortMethod;
use crate::validation::{validate_path, validate_string_length};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

use super::{FirstColumn, SecondColumn};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryMode {
    Database,
    Filesystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct LibrarySettings {
    pub name: String,
    pub path: PathBuf,
    pub mode: LibraryMode,
    pub sort_method: SortMethod,
    pub first_column: FirstColumn,
    pub second_column: SecondColumn,
    pub thumbnail_previews: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hooks: Vec<Hook>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Hook {
    pub path: PathBuf,
    pub program: PathBuf,
    pub sort_method: Option<SortMethod>,
    pub first_column: Option<FirstColumn>,
    pub second_column: Option<SecondColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct LibraryStatistics {
    pub total_books: usize,
    pub finished_books: usize,
    pub total_reading_time: u64,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub average_progress: f32,
}

impl Default for LibrarySettings {
    fn default() -> Self {
        LibrarySettings {
            name: "Unnamed".to_string(),
            path: env::current_dir()
                .ok()
                .unwrap_or_else(|| PathBuf::from("/")),
            mode: LibraryMode::Database,
            sort_method: SortMethod::Opened,
            first_column: FirstColumn::TitleAndAuthor,
            second_column: SecondColumn::Progress,
            thumbnail_previews: true,
            hooks: Vec::new(),
        }
    }
}

impl LibrarySettings {
    /// Validates library settings are within acceptable ranges
    ///
    /// # Validation Rules
    /// - name: 1 to 100 characters
    /// - path: non-empty, valid path format
    pub fn validate(&self) -> Result<(), Error> {
        // Validate library name
        validate_string_length(&self.name, "library.name", 1, 100)?;

        // Validate library path
        validate_path(&self.path, "library.path")?;

        Ok(())
    }
}

impl Default for LibraryStatistics {
    fn default() -> Self {
        LibraryStatistics {
            total_books: 0,
            finished_books: 0,
            total_reading_time: 0,
            current_streak: 0,
            longest_streak: 0,
            average_progress: 0.0,
        }
    }
}
