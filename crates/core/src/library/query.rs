use crate::metadata::sorter;
use crate::metadata::{Info, ReaderInfo, SimpleStatus, SortMethod};
use crate::settings::LibraryMode;
use std::fs;
use std::path::Path;

use super::types::{Library, THUMBNAIL_PREVIEWS_DIRNAME};

impl Library {
    pub fn sort(&mut self, sort_method: SortMethod, reverse_order: bool) {
        self.sort_method = sort_method;
        self.reverse_order = reverse_order;

        if self.mode == LibraryMode::Filesystem {
            return;
        }

        let sort_fn = sorter(sort_method);

        if reverse_order {
            self.db.sort_by(|_, a, _, b| sort_fn(a, b).reverse());
        } else {
            self.db.sort_by(|_, a, _, b| sort_fn(a, b));
        }
    }

    pub fn apply<F>(&mut self, f: F)
    where
        F: Fn(&Path, &mut Info),
    {
        if self.mode == LibraryMode::Filesystem {
            return;
        }

        for (_, info) in &mut self.db {
            f(&self.home, info);
        }

        self.has_db_changed = true;
    }

    pub fn sync_reader_info<P: AsRef<Path>>(&mut self, path: P, reader: &ReaderInfo) {
        let fp = self.get_fingerprint(path.as_ref());
        self.modified_reading_states.insert(fp);
        match self.mode {
            LibraryMode::Database => {
                if let Some(info) = self.db.get_mut(&fp) {
                    info.reader = Some(reader.clone());
                }
            }
            LibraryMode::Filesystem => {
                self.reading_states.insert(fp, reader.clone());
            }
        }
    }

    pub fn thumbnail_preview<P: AsRef<Path>>(&self, path: P) -> std::path::PathBuf {
        if path.as_ref().starts_with(THUMBNAIL_PREVIEWS_DIRNAME) {
            self.home.join(path.as_ref())
        } else {
            let fp = self.get_fingerprint(path.as_ref());
            self.thumbnail_preview_path(fp)
        }
    }

    pub fn set_status<P: AsRef<Path>>(&mut self, path: P, status: SimpleStatus) {
        let fp = self.get_fingerprint(path.as_ref());
        if self.mode == LibraryMode::Database {
            match status {
                SimpleStatus::New => {
                    if let Some(info) = self.db.get_mut(&fp) {
                        info.reader = None;
                    }
                    fs::remove_file(self.reading_state_path(fp)).ok();
                    self.modified_reading_states.remove(&fp);
                }
                SimpleStatus::Reading | SimpleStatus::Finished => {
                    if let Some(info) = self.db.get_mut(&fp) {
                        let reader_info = info
                            .reader
                            .get_or_insert_with(crate::metadata::ReaderInfo::default);
                        reader_info.finished = status == SimpleStatus::Finished;
                        self.modified_reading_states.insert(fp);
                    }
                }
            }
        } else {
            match status {
                SimpleStatus::New => {
                    self.reading_states.remove(&fp);
                    fs::remove_file(self.reading_state_path(fp)).ok();
                    self.modified_reading_states.remove(&fp);
                }
                SimpleStatus::Reading | SimpleStatus::Finished => {
                    let reader_info = self
                        .reading_states
                        .entry(fp)
                        .or_insert_with(ReaderInfo::default);
                    reader_info.finished = status == SimpleStatus::Finished;
                    self.modified_reading_states.insert(fp);
                }
            }
        }
    }
}
