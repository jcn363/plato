use crate::document::file_kind;
use crate::helpers::walkdir_visible;
use crate::log_error;
use crate::settings::ExternalStorageSettings;
use anyhow::Error;
use std::fs;
use std::path::PathBuf;

use super::types::Library;

impl Library {
    pub fn import_from_external(
        &mut self,
        settings: &ExternalStorageSettings,
    ) -> Result<usize, Error> {
        if !settings.enabled {
            return Ok(0);
        }

        let external_path = &settings.path;
        if !external_path.exists() {
            return Ok(0);
        }

        let mut imported = 0;
        let allowed_kinds = &self.import_settings.allowed_kinds;

        for entry in walkdir_visible(external_path) {
            let path = entry.path();
            if path.is_file() {
                if let Some(kind) = file_kind(path) {
                    if allowed_kinds.contains(&kind) {
                        if let Some(filename) = path.file_name() {
                            let dest = self.home.join(filename);
                            if !dest.exists() {
                                if let Err(e) = fs::copy(path, &dest) {
                                    log_error!("Failed to copy {}: {}", path.display(), e);
                                } else {
                                    imported += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        if imported > 0 {
            self.has_db_changed = true;
        }

        Ok(imported)
    }

    pub fn list_external_files(&self, settings: &ExternalStorageSettings) -> Vec<PathBuf> {
        if !settings.enabled {
            return Vec::new();
        }

        let external_path = &settings.path;
        if !external_path.exists() {
            return Vec::new();
        }

        let allowed_kinds = &self.import_settings.allowed_kinds;

        // Pre-allocate with estimated capacity to reduce reallocations
        let mut files = Vec::with_capacity(64);

        for entry in walkdir_visible(external_path) {
            let path = entry.path();
            if path.is_file() {
                if let Some(kind) = file_kind(path) {
                    if allowed_kinds.contains(&kind) {
                        if let Some(filename) = path.file_name() {
                            let dest = self.home.join(filename);
                            if !dest.exists() {
                                files.push(path.to_path_buf());
                            }
                        }
                    }
                }
            }
        }

        files
    }
}
