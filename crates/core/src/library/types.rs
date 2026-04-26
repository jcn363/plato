//! Library data structures and core types
//!
//! This module defines the core Library struct and related types including:
//! - Library database management
//! - Book metadata handling
//! - Reading state tracking
//! - Concurrent caching with DashMap for fast lookups during indexing
//!
//! ## Dependencies
//!
//! - `dashmap` - For concurrent hash map operations
//! - `rustc-hash` - For fast hashing (FxHashMap, FxHashSet, FxBuildHasher)
//! - `indexmap` - For ordered collections (IndexMap)

use crate::helpers::{load_json, Fingerprint, Fp};
use crate::log_error;
use crate::metadata::{Info, ReaderInfo, SortMethod};
use crate::settings::{ImportSettings, LibraryMode};
use anyhow::{bail, Error};
use dashmap::DashMap;
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use std::fs::{self, File};
use std::io::{Error as IoError, ErrorKind};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

// Import validate_library_path from validation module
use crate::validation::validate_library_path;

// Re-export library constants from canonical source in consts::library
// per Single Source of Truth rule.
pub use crate::consts::library::{
    FAT32_EPOCH_FILENAME, METADATA_FILENAME, READING_STATES_DIRNAME, THUMBNAIL_PREVIEWS_DIRNAME,
};

pub struct Library {
    pub home: PathBuf,
    pub mode: LibraryMode,
    pub db: IndexMap<Fp, Info, FxBuildHasher>,
    pub paths: FxHashMap<PathBuf, Fp>,
    pub reading_states: FxHashMap<Fp, ReaderInfo>,
    pub modified_reading_states: FxHashSet<Fp>,
    pub has_db_changed: bool,
    pub fat32_epoch: SystemTime,
    pub sort_method: SortMethod,
    pub reverse_order: bool,
    pub show_hidden: bool,
    pub import_settings: ImportSettings,
    /// Concurrent cache for fast lookups during indexing
    pub concurrent_cache: Arc<DashMap<String, Info>>,
}

impl Library {
    pub fn new<P: AsRef<Path>>(home: P, mode: LibraryMode) -> Result<Self, Error> {
        // Validate home path before any operations
        validate_library_path(&home)?;

        Self::create_home_dir(&home)?;

        let mut db = Self::load_database(&home, mode)?;
        let fat32_epoch = Self::ensure_fat32_epoch_file(&home)?;
        let import_settings = ImportSettings::default();
        let paths = Self::build_paths_map(&db, mode);
        let reading_states = Self::load_reading_states(&home, mode, &mut db)?;
        Self::create_thumbnail_previews_dir(&home);

        Ok(Library {
            home: home.as_ref().to_path_buf(),
            mode,
            db,
            paths,
            reading_states,
            modified_reading_states: FxHashSet::default(),
            has_db_changed: false,
            fat32_epoch,
            sort_method: SortMethod::Opened,
            reverse_order: false,
            show_hidden: false,
            import_settings,
            concurrent_cache: Arc::new(DashMap::new()),
        })
    }

    fn create_home_dir<P: AsRef<Path>>(home: P) -> Result<(), Error> {
        if let Err(e) = fs::create_dir(&home) {
            if e.kind() != ErrorKind::AlreadyExists {
                bail!(e);
            }
        }
        Ok(())
    }

    fn load_database<P: AsRef<Path>>(
        home: P,
        mode: LibraryMode,
    ) -> Result<IndexMap<Fp, Info, FxBuildHasher>, Error> {
        let path = home.as_ref().join(METADATA_FILENAME);
        if mode == LibraryMode::Database {
            match load_json::<IndexMap<Fp, Info, FxBuildHasher>, _>(&path) {
                Err(e) => {
                    if e.downcast_ref::<IoError>().map(|e| e.kind()) != Some(ErrorKind::NotFound) {
                        bail!(e);
                    } else {
                        Ok(IndexMap::with_capacity_and_hasher(0, FxBuildHasher))
                    }
                }
                Ok(v) => Ok(v),
            }
        } else {
            Ok(IndexMap::with_capacity_and_hasher(0, FxBuildHasher))
        }
    }

    fn load_reading_states<P: AsRef<Path>>(
        home: P,
        mode: LibraryMode,
        db: &mut IndexMap<Fp, Info, FxBuildHasher>,
    ) -> Result<FxHashMap<Fp, ReaderInfo>, Error> {
        let mut reading_states = FxHashMap::default();

        let path = home.as_ref().join(READING_STATES_DIRNAME);
        if let Err(e) = fs::create_dir(&path) {
            if e.kind() != ErrorKind::AlreadyExists {
                bail!(e);
            }
        }

        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(fp) = path
                .file_stem()
                .and_then(|v| v.to_str())
                .and_then(|v| Fp::from_str(v).ok())
            {
                if let Ok(reader_info) =
                    load_json(path).map_err(|e| log_error!("Can't load reading state: {:#}.", e))
                {
                    if mode == LibraryMode::Database {
                        if let Some(info) = db.get_mut(&fp) {
                            info.reader = Some(reader_info);
                        } else {
                            log_error!("Unknown fingerprint: {}.", fp);
                        }
                    } else {
                        reading_states.insert(fp, reader_info);
                    }
                }
            }
        }

        Ok(reading_states)
    }

    fn create_thumbnail_previews_dir<P: AsRef<Path>>(home: P) {
        let path = home.as_ref().join(THUMBNAIL_PREVIEWS_DIRNAME);
        if !path.exists() {
            fs::create_dir(&path).ok();
        }
    }

    fn build_paths_map(
        db: &IndexMap<Fp, Info, FxBuildHasher>,
        mode: LibraryMode,
    ) -> FxHashMap<PathBuf, Fp> {
        if mode == LibraryMode::Database {
            db.iter()
                .map(|(fp, info)| (info.file.path.clone(), *fp))
                .collect()
        } else {
            FxHashMap::default()
        }
    }

    fn ensure_fat32_epoch_file<P: AsRef<Path>>(home: P) -> Result<SystemTime, Error> {
        let path = home.as_ref().join(FAT32_EPOCH_FILENAME);
        if !path.exists() {
            let file = File::create(&path)?;
            file.set_modified(std::time::UNIX_EPOCH + Duration::from_secs(315_532_800))?;
        }
        Ok(path.metadata()?.modified()?)
    }

    pub fn with_import_settings<P: AsRef<Path>>(
        home: P,
        mode: LibraryMode,
        import_settings: ImportSettings,
    ) -> Result<Self, Error> {
        // Validate home path before creating library
        validate_library_path(&home)?;

        let mut library = Self::new(home, mode)?;
        library.import_settings = import_settings;
        Ok(library)
    }

    pub fn reading_state_path(&self, fp: Fp) -> PathBuf {
        self.home
            .join(READING_STATES_DIRNAME)
            .join(format!("{}.json", fp))
    }

    pub fn thumbnail_preview_path(&self, fp: Fp) -> PathBuf {
        self.home
            .join(THUMBNAIL_PREVIEWS_DIRNAME)
            .join(format!("{}.png", fp))
    }

    pub fn get_fingerprint(&self, path: &Path) -> Fp {
        self.paths.get(path).cloned().unwrap_or_else(|| {
            self.home
                .join(path)
                .metadata()
                .ok()
                .and_then(|md| md.fingerprint(self.fat32_epoch).ok())
                .unwrap_or(Fp(0))
        })
    }

    /// Returns the number of books in the library
    pub fn len(&self) -> usize {
        self.db.len()
    }

    /// Returns true if the library contains no books
    pub fn is_empty(&self) -> bool {
        self.db.is_empty()
    }

    /// Returns an iterator over the books in the library
    pub fn iter(&self) -> impl Iterator<Item = (&Fp, &Info)> {
        self.db.iter()
    }
}
