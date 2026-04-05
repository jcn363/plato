use crate::document::file_kind;
use crate::helpers::Fingerprint;
use crate::metadata::{FileInfo, Info};
use crate::settings::LibraryMode;
use anyhow::format_err;
use anyhow::Error;
use chrono::Local;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use super::types::Library;

impl Library {
    pub fn rename<P: AsRef<Path>>(&mut self, path: P, file_name: &str) -> Result<(), Error> {
        let src = self.home.join(path.as_ref());

        let fp = self
            .paths
            .remove(path.as_ref())
            .or_else(|| {
                src.metadata()
                    .ok()
                    .and_then(|md| md.fingerprint(self.fat32_epoch).ok())
            })
            .ok_or_else(|| format_err!("can't get fingerprint of {}", path.as_ref().display()))?;

        let mut dest = src.clone();
        dest.set_file_name(file_name);
        fs::rename(&src, &dest)?;

        if self.mode == LibraryMode::Database {
            let new_path = dest.strip_prefix(&self.home)?;
            self.paths.insert(new_path.to_path_buf(), fp);
            if let Some(info) = self.db.get_mut(&fp) {
                info.file.path = new_path.to_path_buf();
                self.has_db_changed = true;
            }
        }

        Ok(())
    }

    pub fn remove<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let full_path = self.home.join(path.as_ref());

        let fp = self
            .paths
            .get(path.as_ref())
            .cloned()
            .or_else(|| {
                full_path
                    .metadata()
                    .ok()
                    .and_then(|md| md.fingerprint(self.fat32_epoch).ok())
            })
            .ok_or_else(|| format_err!("can't get fingerprint of {}", path.as_ref().display()))?;

        if full_path.exists() {
            fs::remove_file(&full_path)?;
        }

        if let Some(parent) = full_path.parent() {
            if parent != self.home {
                fs::remove_dir(parent).ok();
            }
        }

        self.db.shift_remove(&fp);
        self.paths.remove(path.as_ref());
        self.reading_states.remove(&fp);
        self.modified_reading_states.insert(fp);
        self.has_db_changed = true;

        Ok(())
    }

    pub fn remove_batch(&mut self, paths: &[PathBuf]) -> Result<(), Error> {
        for path in paths {
            self.remove(path)?;
        }
        Ok(())
    }

    pub fn copy_to<P: AsRef<Path>>(&mut self, path: P, other: &mut Library) -> Result<(), Error> {
        let src = self.home.join(path.as_ref());

        if !src.exists() {
            return Err(format_err!(
                "can't copy non-existing file {}",
                path.as_ref().display()
            ));
        }

        let md = src.metadata()?;
        let fp = self
            .paths
            .get(path.as_ref())
            .cloned()
            .or_else(|| md.fingerprint(self.fat32_epoch).ok())
            .ok_or_else(|| format_err!("can't get fingerprint of {}", path.as_ref().display()))?;

        let mut dest = other.home.join(path.as_ref());
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        if dest.exists() {
            let prefix = Local::now().format("%Y%m%d_%H%M%S ");
            let name = dest
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| prefix.to_string() + name)
                .ok_or_else(|| format_err!("can't compute new name for {}", dest.display()))?;
            dest.set_file_name(name);
        }

        fs::copy(&src, &dest)?;
        {
            let fdest = File::open(&dest)?;
            fdest.set_modified(md.modified()?)?;
        }

        let rsp_src = self.reading_state_path(fp);
        fs::copy(&rsp_src, &other.reading_state_path(fp)).ok();

        let tpp_src = self.thumbnail_preview_path(fp);
        fs::copy(&tpp_src, &other.thumbnail_preview_path(fp)).ok();

        if other.mode == LibraryMode::Database {
            let info = self.db.get(&fp).cloned().or_else(|| {
                self.reading_states
                    .get(&fp)
                    .cloned()
                    .map(|reader_info| Info {
                        file: FileInfo {
                            size: md.len(),
                            kind: file_kind(&dest).unwrap_or_default(),
                            ..Default::default()
                        },
                        reader: Some(reader_info),
                        ..Default::default()
                    })
            });
            if let Some(mut info) = info {
                let dest_path = dest.strip_prefix(&other.home)?;
                info.file.path = dest_path.to_path_buf();
                other.db.insert(fp, info);
                other.paths.insert(dest_path.to_path_buf(), fp);
                other.has_db_changed = true;
            }
        } else {
            let reader_info = self
                .reading_states
                .get(&fp)
                .cloned()
                .or_else(|| self.db.get(&fp).cloned().and_then(|info| info.reader));
            if let Some(reader_info) = reader_info {
                other.reading_states.insert(fp, reader_info);
            }
        }

        other.modified_reading_states.insert(fp);

        Ok(())
    }

    pub fn move_to<P: AsRef<Path>>(&mut self, path: P, other: &mut Library) -> Result<(), Error> {
        let src = self.home.join(path.as_ref());

        if !src.exists() {
            return Err(format_err!(
                "can't move non-existing file {}",
                path.as_ref().display()
            ));
        }

        let md = src.metadata()?;
        let fp = self
            .paths
            .get(path.as_ref())
            .cloned()
            .or_else(|| md.fingerprint(self.fat32_epoch).ok())
            .ok_or_else(|| format_err!("can't get fingerprint of {}", path.as_ref().display()))?;

        let src = self.home.join(path.as_ref());
        let mut dest = other.home.join(path.as_ref());
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        if dest.exists() {
            let prefix = Local::now().format("%Y%m%d_%H%M%S ");
            let name = dest
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| prefix.to_string() + name)
                .ok_or_else(|| format_err!("can't compute new name for {}", dest.display()))?;
            dest.set_file_name(name);
        }

        fs::rename(&src, &dest)?;

        let rsp_src = self.reading_state_path(fp);
        fs::rename(&rsp_src, &other.reading_state_path(fp)).ok();

        let tpp_src = self.thumbnail_preview_path(fp);
        fs::rename(&tpp_src, &other.thumbnail_preview_path(fp)).ok();

        if other.mode == LibraryMode::Database {
            let info = self.db.shift_remove(&fp).or_else(|| {
                self.reading_states.remove(&fp).map(|reader_info| Info {
                    file: FileInfo {
                        size: md.len(),
                        kind: file_kind(&dest).unwrap_or_default(),
                        ..Default::default()
                    },
                    reader: Some(reader_info),
                    ..Default::default()
                })
            });
            if let Some(mut info) = info {
                let dest_path = dest.strip_prefix(&other.home)?;
                info.file.path = dest_path.to_path_buf();
                other.db.insert(fp, info);
                self.paths.remove(path.as_ref());
                other.paths.insert(dest_path.to_path_buf(), fp);
                self.has_db_changed = true;
                other.has_db_changed = true;
            }
        } else {
            let reader_info = self
                .reading_states
                .remove(&fp)
                .or_else(|| self.db.shift_remove(&fp).and_then(|info| info.reader));
            if let Some(reader_info) = reader_info {
                other.reading_states.insert(fp, reader_info);
            }
        }

        if self.modified_reading_states.remove(&fp) {
            other.modified_reading_states.insert(fp);
        }

        Ok(())
    }

    pub fn move_batch(&mut self, paths: &[PathBuf], other: &mut Library) -> Result<(), Error> {
        for path in paths {
            self.move_to(path, other)?;
        }
        Ok(())
    }
}
