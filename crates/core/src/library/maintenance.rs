use crate::helpers::{load_json, save_json, Fingerprint, Fp};
use crate::settings::LibraryMode;
use crate::{log_error, log_warn};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use super::types::{
    Library, METADATA_FILENAME, READING_STATES_DIRNAME, THUMBNAIL_PREVIEWS_DIRNAME,
};

impl Library {
    pub fn clean_up(&mut self) {
        if self.mode == LibraryMode::Filesystem {
            return;
        }

        let fps = self.collect_valid_fingerprints();
        self.cleanup_reading_states(&fps);
        self.cleanup_orphaned_files(&fps);
    }

    fn collect_valid_fingerprints(&self) -> rustc_hash::FxHashSet<Fp> {
        walkdir::WalkDir::new(&self.home)
            .min_depth(1)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                if entry.file_type().is_dir() {
                    None
                } else {
                    entry
                        .metadata()
                        .ok()
                        .and_then(|md| md.fingerprint(self.fat32_epoch).ok())
                }
            })
            .collect()
    }

    fn cleanup_reading_states(&mut self, fps: &rustc_hash::FxHashSet<Fp>) {
        self.reading_states.retain(|fp, _| {
            if fps.contains(fp) {
                true
            } else {
                log_warn!("Remove reading state for {}.", fp);
                false
            }
        });
        self.modified_reading_states.retain(|fp| fps.contains(fp));
    }

    fn cleanup_orphaned_files(&self, fps: &rustc_hash::FxHashSet<Fp>) {
        let reading_states_dir = self.home.join(READING_STATES_DIRNAME);
        let thumbnail_previews_dir = self.home.join(THUMBNAIL_PREVIEWS_DIRNAME);
        let reading_entries = fs::read_dir(&reading_states_dir).ok().into_iter().flatten();
        let thumbnail_entries = fs::read_dir(&thumbnail_previews_dir)
            .ok()
            .into_iter()
            .flatten();
        for entry in reading_entries.chain(thumbnail_entries) {
            let Ok(entry) = entry else {
                continue;
            };
            if let Some(fp) = entry
                .path()
                .file_stem()
                .and_then(|v| v.to_str())
                .and_then(|v| crate::helpers::Fp::from_str(v).ok())
            {
                if !fps.contains(&fp) {
                    fs::remove_file(entry.path()).ok();
                }
            }
        }
    }

    pub fn reload(&mut self) {
        self.reload_database();
        self.reload_reading_states();
        self.rebuild_paths();
    }

    fn reload_database(&mut self) {
        if self.mode == LibraryMode::Database {
            let path = self.home.join(METADATA_FILENAME);

            match load_json(&path) {
                Err(e) => {
                    log_error!("Can't reload database: {:#}.", e);
                    return;
                }
                Ok(v) => {
                    self.db = v;
                    self.has_db_changed = false;
                }
            }
        }
    }

    fn reload_reading_states(&mut self) {
        let path = self.home.join(READING_STATES_DIRNAME);

        self.modified_reading_states.clear();
        if self.mode == LibraryMode::Filesystem {
            self.reading_states.clear();
        }

        let Ok(dir_entries) = fs::read_dir(&path) else {
            return;
        };
        for entry in dir_entries {
            let Ok(entry) = entry else {
                continue;
            };
            let path = entry.path();
            if let Some(fp) = path
                .file_stem()
                .and_then(|v| v.to_str())
                .and_then(|v| crate::helpers::Fp::from_str(v).ok())
            {
                self.load_reading_state(path, fp);
            }
        }
    }

    fn load_reading_state(&mut self, path: PathBuf, fp: Fp) {
        if let Ok(reader_info) =
            load_json(path).map_err(|e| log_error!("Can't load reading state: {:#}.", e))
        {
            if self.mode == LibraryMode::Database {
                if let Some(info) = self.db.get_mut(&fp) {
                    info.reader = Some(reader_info);
                } else {
                    log_warn!("Unknown fingerprint: {}.", fp);
                }
            } else {
                self.reading_states.insert(fp, reader_info);
            }
        }
    }

    fn rebuild_paths(&mut self) {
        if self.mode == LibraryMode::Database {
            self.paths = self
                .db
                .iter()
                .map(|(fp, info)| (info.file.path.clone(), *fp))
                .collect();
        }
    }

    pub fn flush(&mut self) {
        for fp in &self.modified_reading_states {
            let reader_info = if self.mode == LibraryMode::Database {
                self.db.get(fp).and_then(|info| info.reader.as_ref())
            } else {
                self.reading_states.get(fp)
            };
            if let Some(reader_info) = reader_info {
                save_json(reader_info, self.reading_state_path(*fp))
                    .map_err(|e| log_error!("Can't save reading state: {:#}.", e))
                    .ok();
            }
        }

        self.modified_reading_states.clear();

        if self.has_db_changed {
            save_json(&self.db, self.home.join(METADATA_FILENAME))
                .map_err(|e| log_error!("Can't save database: {:#}.", e))
                .ok();
            self.has_db_changed = false;
        }
    }

    pub fn is_database_empty(&self) -> Option<bool> {
        if self.mode == LibraryMode::Database {
            Some(self.db.is_empty())
        } else {
            None
        }
    }

    pub fn compute_statistics(&self) -> crate::settings::LibraryStatistics {
        let mut total_reading_time = 0u64;
        let mut finished_books = 0usize;
        let mut total_progress = 0f32;

        for info in self.db.values() {
            if let Some(reader) = info.reader.as_ref() {
                total_reading_time += reader.reading_time_seconds;
                total_progress += reader.progress();
                if reader.finished {
                    finished_books += 1;
                }
            }
        }

        let total_books = self.db.len();
        let average_progress = if total_books > 0 {
            total_progress / total_books as f32
        } else {
            0.0
        };

        crate::settings::LibraryStatistics {
            total_books,
            finished_books,
            total_reading_time,
            current_streak: 0,
            longest_streak: 0,
            average_progress,
        }
    }
}
