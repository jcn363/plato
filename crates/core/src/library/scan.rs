use crate::document::file_kind;
use crate::helpers::{Fingerprint, IsHidden};
use crate::log_info;
use crate::metadata::BookQuery;
use crate::metadata::{extract_metadata_from_document, sort, FileInfo, Info};
use crate::settings::{ImportSettings, LibraryMode};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use walkdir::WalkDir;

use super::types::Library;
use super::types::READING_STATES_DIRNAME;
use super::types::THUMBNAIL_PREVIEWS_DIRNAME;

impl Library {
    pub fn list<P: AsRef<Path>>(
        &self,
        prefix: P,
        query: Option<&BookQuery>,
        skip_files: bool,
    ) -> (Vec<Info>, std::collections::BTreeSet<PathBuf>) {
        let mut dirs = std::collections::BTreeSet::new();
        let mut files = Vec::new();

        match self.mode {
            LibraryMode::Database => {
                let relat_prefix = prefix
                    .as_ref()
                    .strip_prefix(&self.home)
                    .unwrap_or_else(|_| prefix.as_ref());
                for (_, info) in self.db.iter() {
                    if let Ok(relat) = info.file.path.strip_prefix(relat_prefix) {
                        let mut compos = relat.components();
                        let mut first = compos.next();
                        if compos.next().is_none() {
                            first = None;
                        }
                        if let Some(child) = first {
                            dirs.insert(prefix.as_ref().join(child.as_os_str()));
                        }
                        if skip_files {
                            continue;
                        }
                        if query.map_or(true, |q| q.is_match(info)) {
                            files.push(info.clone());
                        }
                    }
                }
            }
            LibraryMode::Filesystem => {
                if !prefix.as_ref().is_dir() {
                    return (files, dirs);
                }

                let max_depth = if query.is_some() { usize::MAX } else { 1 };

                for entry in WalkDir::new(prefix.as_ref())
                    .min_depth(1)
                    .max_depth(max_depth)
                    .into_iter()
                    .filter_entry(|e| self.show_hidden || !e.is_hidden())
                {
                    let Ok(entry) = entry else {
                        continue;
                    };
                    let path = entry.path();

                    if path.is_dir() {
                        if entry.depth() == 1 {
                            dirs.insert(path.to_path_buf());
                        }
                    } else {
                        let relat = path.strip_prefix(&self.home).unwrap_or(path);
                        if skip_files
                            || query.map_or(false, |q| {
                                relat.to_str().map_or(true, |s| !q.is_simple_match(s))
                            })
                        {
                            continue;
                        }

                        let kind = file_kind(&path).unwrap_or_default();
                        let Ok(md) = entry.metadata() else {
                            continue;
                        };
                        let size = md.len();
                        let Ok(fp) = md.fingerprint(self.fat32_epoch) else {
                            continue;
                        };
                        let file = FileInfo {
                            path: relat.to_path_buf(),
                            kind,
                            size,
                        };
                        let secs = (*fp >> 32) as i64;
                        let nsecs = ((*fp & ((1 << 32) - 1)) % 1_000_000_000) as u32;
                        let Some(added) =
                            chrono::DateTime::from_timestamp(secs, nsecs).map(|dt| dt.naive_utc())
                        else {
                            continue;
                        };
                        let info = Info {
                            file,
                            added,
                            reader: self.reading_states.get(&fp).cloned(),
                            ..Default::default()
                        };

                        files.push(info);
                    }
                }

                sort(&mut files, self.sort_method, self.reverse_order);
            }
        }

        (files, dirs)
    }

    pub fn find_next_file(&self, current_path: &Path, query: Option<&BookQuery>) -> Option<Info> {
        let parent = current_path.parent()?;
        let (files, _) = self.list(parent, query, false);

        for (i, info) in files.iter().enumerate() {
            if &info.file.path == current_path {
                if i + 1 < files.len() {
                    return Some(files[i + 1].clone());
                }
                break;
            }
        }
        None
    }

    pub fn import(&mut self, settings: &ImportSettings) {
        if self.mode == LibraryMode::Filesystem {
            return;
        }

        for entry in WalkDir::new(&self.home)
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| !e.is_hidden())
        {
            let Ok(entry) = entry else {
                continue;
            };
            if entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path();
            let relat = path.strip_prefix(&self.home).unwrap_or(path);
            let Ok(md) = entry.metadata() else {
                continue;
            };
            let Ok(fp) = md.fingerprint(self.fat32_epoch) else {
                continue;
            };

            if self.db.contains_key(&fp) {
                if relat != self.db[&fp].file.path {
                    log_info!(
                        "Update path for {}: {} → {}.",
                        fp,
                        self.db[&fp].file.path.display(),
                        relat.display()
                    );
                    self.paths.remove(&self.db[&fp].file.path);
                    self.paths.insert(relat.to_path_buf(), fp);
                    self.db[&fp].file.path = relat.to_path_buf();
                    self.has_db_changed = true;
                }
            } else if let Some(fp2) = self.paths.get(relat).cloned() {
                log_info!(
                    "Update fingerprint for {}: {} → {}.",
                    relat.display(),
                    fp2,
                    fp
                );
                let Some(mut info) = self.db.swap_remove(&fp2) else {
                    continue;
                };
                if settings.sync_metadata && settings.metadata_kinds.contains(&info.file.kind) {
                    extract_metadata_from_document(&self.home, &mut info);
                }
                self.db.insert(fp, info);
                self.db[&fp].file.size = md.len();
                self.paths.insert(relat.to_path_buf(), fp);
                let rp1 = self.reading_state_path(fp2);
                let rp2 = self.reading_state_path(fp);
                fs::rename(rp1, rp2).ok();
                let tpp = self.thumbnail_preview_path(fp2);
                if tpp.exists() {
                    fs::remove_file(tpp).ok();
                }
                self.has_db_changed = true;
            } else {
                let fp1 = self
                    .fat32_epoch
                    .checked_sub(std::time::Duration::from_secs(1))
                    .and_then(|epoch| md.fingerprint(epoch).ok())
                    .unwrap_or(fp);
                let fp2 = self
                    .fat32_epoch
                    .checked_add(std::time::Duration::from_secs(1))
                    .and_then(|epoch| md.fingerprint(epoch).ok())
                    .unwrap_or(fp);

                let nfp = if fp1 != fp && self.db.contains_key(&fp1) {
                    Some(fp1)
                } else if fp2 != fp && self.db.contains_key(&fp2) {
                    Some(fp2)
                } else {
                    None
                };

                if let Some(nfp) = nfp {
                    log_info!(
                        "Update fingerprint for {}: {} → {}.",
                        self.db[&nfp].file.path.display(),
                        nfp,
                        fp
                    );
                    let Some(info) = self.db.swap_remove(&nfp) else {
                        continue;
                    };
                    self.db.insert(fp, info);
                    let rp1 = self.reading_state_path(nfp);
                    let rp2 = self.reading_state_path(fp);
                    fs::rename(rp1, rp2).ok();
                    let tp1 = self.thumbnail_preview_path(nfp);
                    let tp2 = self.thumbnail_preview_path(fp);
                    fs::rename(tp1, tp2).ok();
                    if relat != self.db[&fp].file.path {
                        log_info!(
                            "Update path for {}: {} → {}.",
                            fp,
                            self.db[&fp].file.path.display(),
                            relat.display()
                        );
                        self.paths.remove(&self.db[&fp].file.path);
                        self.paths.insert(relat.to_path_buf(), fp);
                        self.db[&fp].file.path = relat.to_path_buf();
                    }
                } else {
                    let kind = file_kind(&path).unwrap_or_default();
                    if !settings.allowed_kinds.contains(&kind) {
                        continue;
                    }
                    log_info!("Add new entry: {}, {}.", fp, relat.display());
                    let size = md.len();
                    let file = FileInfo {
                        path: relat.to_path_buf(),
                        kind,
                        size,
                    };
                    let mut info = Info {
                        file,
                        ..Default::default()
                    };
                    if settings.metadata_kinds.contains(&info.file.kind) {
                        extract_metadata_from_document(&self.home, &mut info);
                    }
                    self.db.insert(fp, info);
                    self.paths.insert(relat.to_path_buf(), fp);
                }

                self.has_db_changed = true;
            }
        }

        let home = &self.home;
        let len = self.db.len();

        self.db.retain(|fp, info| {
            let path = home.join(&info.file.path);
            if path.exists() {
                true
            } else {
                log_info!("Remove entry: {}, {}.", fp, info.file.path.display());
                false
            }
        });

        if self.db.len() != len {
            self.has_db_changed = true;
            let db = &self.db;
            self.paths.retain(|_, fp| db.contains_key(fp));
            self.modified_reading_states
                .retain(|fp| db.contains_key(fp));

            let reading_states_dir = home.join(READING_STATES_DIRNAME);
            let thumbnail_previews_dir = home.join(THUMBNAIL_PREVIEWS_DIRNAME);
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
                    if !self.db.contains_key(&fp) {
                        fs::remove_file(entry.path()).ok();
                    }
                }
            }
        }
    }

    pub fn add_document(&mut self, info: Info) {
        let path = self.home.join(&info.file.path);
        let Ok(md) = path.metadata() else {
            return;
        };
        let Ok(fp) = md.fingerprint(self.fat32_epoch) else {
            return;
        };

        if info.reader.is_some() {
            self.modified_reading_states.insert(fp);
        }

        if self.mode == LibraryMode::Database {
            self.paths.insert(info.file.path.clone(), fp);
            self.db.insert(fp, info);
            self.has_db_changed = true;
        } else {
            if let Some(reader_info) = info.reader {
                self.reading_states.insert(fp, reader_info);
            }
        }
    }
}
