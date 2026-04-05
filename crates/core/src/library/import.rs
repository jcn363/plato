use crate::document::file_kind;
use crate::log_error;
use crate::settings::ExternalStorageSettings;
use anyhow::Error;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

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

        for entry in WalkDir::new(external_path)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
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

        WalkDir::new(external_path)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter_map(|e| {
                let path = e.path();
                if let Some(kind) = file_kind(path) {
                    if allowed_kinds.contains(&kind) {
                        let dest = self.home.join(path.file_name()?);
                        if !dest.exists() {
                            return Some(path.to_path_buf());
                        }
                    }
                }
                None
            })
            .collect()
    }
}
