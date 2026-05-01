use crate::document::file_kind;
use crate::helpers::{Fingerprint, Fp};
use crate::metadata::{FileInfo, Info};
use crate::settings::LibraryMode;
use crate::validation::{validate_filename, validate_path};
use anyhow::{format_err, Context, Error};
use chrono::Local;
use std::fs;
use std::fs::{File, Metadata as FsMetadata};
use std::path::{Path, PathBuf};

use super::types::Library;

impl Library {
    pub fn rename<P: AsRef<Path>>(&mut self, path: P, file_name: &str) -> Result<(), Error> {
        // Validate inputs before any operations
        validate_path(&path, "rename source path")?;
        validate_filename(file_name, "rename destination name")?;

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

        fs::rename(&src, &dest).with_context(|| format!("rename from {:?} to {:?}", src, dest))?;

        let new_path = dest.strip_prefix(&self.home)?;
        self.paths.insert(new_path.to_path_buf(), fp);
        if self.mode == LibraryMode::Database {
            if let Some(info) = self.db.get_mut(&fp) {
                info.file.path = new_path.to_path_buf();
                self.has_db_changed = true;
            }
        }

        Ok(())
    }

    pub fn remove<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        // Validate path before any operations
        validate_path(&path, "remove path")?;

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
        if paths.is_empty() {
            return Ok(());
        }
        let mut failed = 0usize;
        for path in paths {
            if let Err(e) = self.remove(path) {
                crate::log_error!("remove_batch: failed to remove {}: {:#}", path.display(), e);
                failed += 1;
            }
        }
        if failed > 0 {
            Err(anyhow::format_err!(
                "{}/{} files could not be removed (see log for details)",
                failed,
                paths.len()
            ))
        } else {
            Ok(())
        }
    }

    pub fn copy_to<P: AsRef<Path>>(&mut self, path: P, other: &mut Library) -> Result<(), Error> {
        // Validate path before any operations
        validate_path(&path, "copy source path")?;

        let src = self.home.join(path.as_ref());
        self.validate_source_exists(&src, path.as_ref())?;

        let md = src.metadata()?;
        let fp = self.get_fingerprint(path.as_ref());

        let dest = self.prepare_destination(path.as_ref(), other)?;

        self.copy_file(&src, &dest, &md)?;
        self.copy_metadata(&fp, other)?;
        self.copy_reader_info(&fp, &dest, &md, other)?;

        other.modified_reading_states.insert(fp);
        Ok(())
    }

    fn validate_source_exists<P: AsRef<Path>>(&self, src: &Path, path: P) -> Result<(), Error> {
        if !src.exists() {
            return Err(format_err!(
                "can't copy non-existing file {}",
                path.as_ref().display()
            ));
        }
        Ok(())
    }

    fn prepare_destination<P: AsRef<Path>>(
        &self,
        path: P,
        other: &Library,
    ) -> Result<PathBuf, Error> {
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
        Ok(dest)
    }

    fn copy_file(&self, src: &Path, dest: &Path, md: &FsMetadata) -> Result<(), Error> {
        fs::copy(src, dest)?;
        let fdest = File::open(dest)
            .with_context(|| format!("can't open destination file {}", dest.display()))?;
        fdest.set_modified(md.modified()?)?;
        Ok(())
    }

    fn copy_metadata(&self, fp: &Fp, other: &Library) -> Result<(), Error> {
        let rsp_src = self.reading_state_path(*fp);
        fs::copy(&rsp_src, other.reading_state_path(*fp)).ok();

        let tpp_src = self.thumbnail_preview_path(*fp);
        fs::copy(&tpp_src, other.thumbnail_preview_path(*fp)).ok();
        Ok(())
    }

    fn copy_reader_info(
        &self,
        fp: &Fp,
        dest: &Path,
        md: &FsMetadata,
        other: &mut Library,
    ) -> Result<(), Error> {
        if other.mode == LibraryMode::Database {
            self.copy_to_database(fp, dest, md, other)?;
        } else {
            self.copy_to_filesystem(fp, other)?;
        }
        Ok(())
    }

    fn copy_to_database(
        &self,
        fp: &Fp,
        dest: &Path,
        md: &FsMetadata,
        other: &mut Library,
    ) -> Result<(), Error> {
        let info = self.db.get(fp).cloned().or_else(|| {
            self.reading_states
                .get(fp)
                .cloned()
                .map(|reader_info| Info {
                    file: FileInfo {
                        size: md.len(),
                        kind: file_kind(dest).unwrap_or_default(),
                        ..Default::default()
                    },
                    reader: Some(reader_info),
                    ..Default::default()
                })
        });
        if let Some(mut info) = info {
            let dest_path = dest.strip_prefix(&other.home)?;
            info.file.path = dest_path.to_path_buf();
            other.db.insert(*fp, info);
            other.paths.insert(dest_path.to_path_buf(), *fp);
            other.has_db_changed = true;
        }
        Ok(())
    }

    fn copy_to_filesystem(&self, fp: &Fp, other: &mut Library) -> Result<(), Error> {
        let reader_info = self
            .reading_states
            .get(fp)
            .cloned()
            .or_else(|| self.db.get(fp).cloned().and_then(|info| info.reader));
        if let Some(reader_info) = reader_info {
            other.reading_states.insert(*fp, reader_info);
        }
        Ok(())
    }

    pub fn move_to<P: AsRef<Path>>(&mut self, path: P, other: &mut Library) -> Result<(), Error> {
        // Validate path before any operations
        validate_path(&path, "move source path")?;

        let src = self.home.join(path.as_ref());
        self.validate_source_exists(&src, path.as_ref())?;

        let md = src.metadata()?;
        let fp = self.get_fingerprint(path.as_ref());

        let dest = self.prepare_destination(path.as_ref(), other)?;

        self.move_file(&src, &dest)?;
        self.move_metadata(&fp, other)?;
        self.move_reader_info(path.as_ref(), &fp, &dest, &md, other)?;
        self.move_modified_state(&fp, other)?;

        Ok(())
    }

    fn move_file(&self, src: &Path, dest: &Path) -> Result<(), Error> {
        fs::rename(src, dest)?;
        Ok(())
    }

    fn move_metadata(&self, fp: &Fp, other: &Library) -> Result<(), Error> {
        let rsp_src = self.reading_state_path(*fp);
        fs::rename(&rsp_src, other.reading_state_path(*fp)).ok();

        let tpp_src = self.thumbnail_preview_path(*fp);
        fs::rename(&tpp_src, other.thumbnail_preview_path(*fp)).ok();
        Ok(())
    }

    fn move_reader_info<P: AsRef<Path>>(
        &mut self,
        path: P,
        fp: &Fp,
        dest: &Path,
        md: &FsMetadata,
        other: &mut Library,
    ) -> Result<(), Error> {
        if other.mode == LibraryMode::Database {
            self.move_to_database(path, fp, dest, md, other)?;
        } else {
            self.move_to_filesystem(fp, other)?;
        }
        Ok(())
    }

    fn move_to_database<P: AsRef<Path>>(
        &mut self,
        path: P,
        fp: &Fp,
        dest: &Path,
        md: &FsMetadata,
        other: &mut Library,
    ) -> Result<(), Error> {
        let info = self.db.shift_remove(fp).or_else(|| {
            self.reading_states.remove(fp).map(|reader_info| Info {
                file: FileInfo {
                    size: md.len(),
                    kind: file_kind(dest).unwrap_or_default(),
                    ..Default::default()
                },
                reader: Some(reader_info),
                ..Default::default()
            })
        });
        if let Some(mut info) = info {
            let dest_path = dest.strip_prefix(&other.home)?;
            info.file.path = dest_path.to_path_buf();
            other.db.insert(*fp, info);
            self.paths.remove(path.as_ref());
            other.paths.insert(dest_path.to_path_buf(), *fp);
            self.has_db_changed = true;
            other.has_db_changed = true;
        }
        Ok(())
    }

    fn move_to_filesystem(&mut self, fp: &Fp, other: &mut Library) -> Result<(), Error> {
        let reader_info = self
            .reading_states
            .remove(fp)
            .or_else(|| self.db.shift_remove(fp).and_then(|info| info.reader));
        if let Some(reader_info) = reader_info {
            other.reading_states.insert(*fp, reader_info);
        }
        Ok(())
    }

    fn move_modified_state(&mut self, fp: &Fp, other: &mut Library) -> Result<(), Error> {
        if self.modified_reading_states.remove(fp) {
            other.modified_reading_states.insert(*fp);
        }
        Ok(())
    }

    pub fn move_batch(&mut self, paths: &[PathBuf], other: &mut Library) -> Result<(), Error> {
        if paths.is_empty() {
            return Ok(());
        }
        let mut failed = 0usize;
        for path in paths {
            if let Err(e) = self.move_to(path, other) {
                crate::log_error!("move_batch: failed to move {}: {:#}", path.display(), e);
                failed += 1;
            }
        }
        if failed > 0 {
            Err(anyhow::format_err!(
                "{}/{} files could not be moved (see log for details)",
                failed,
                paths.len()
            ))
        } else {
            Ok(())
        }
    }
}
