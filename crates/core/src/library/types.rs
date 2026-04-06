use crate::helpers::{load_json, Fingerprint, Fp};
use crate::metadata::{Info, ReaderInfo, SortMethod};
use crate::settings::{ImportSettings, LibraryMode};
use crate::{log_error, log_warn};
use anyhow::{bail, Error};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use indexmap::IndexMap;
use std::fs::{self, File};
use std::io::{Error as IoError, ErrorKind};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

pub const METADATA_FILENAME: &str = ".metadata.json";
pub const FAT32_EPOCH_FILENAME: &str = ".fat32-epoch";
pub const READING_STATES_DIRNAME: &str = ".reading-states";
pub const THUMBNAIL_PREVIEWS_DIRNAME: &str = ".thumbnail-previews";

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
}

impl Library {
    pub fn new<P: AsRef<Path>>(home: P, mode: LibraryMode) -> Result<Self, Error> {
        if let Err(e) = fs::create_dir(&home) {
            if e.kind() != ErrorKind::AlreadyExists {
                bail!(e);
            }
        }

        let path = home.as_ref().join(METADATA_FILENAME);
        let mut db;
        if mode == LibraryMode::Database {
            match load_json::<IndexMap<Fp, Info, FxBuildHasher>, _>(&path) {
                Err(e) => {
                    if e.downcast_ref::<IoError>().map(|e| e.kind()) != Some(ErrorKind::NotFound) {
                        bail!(e);
                    } else {
                        db = IndexMap::with_capacity_and_hasher(0, FxBuildHasher::default());
                    }
                }
                Ok(v) => db = v,
            }
        } else {
            db = IndexMap::with_capacity_and_hasher(0, FxBuildHasher::default());
        }

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
                            log_warn!("Unknown fingerprint: {}.", fp);
                        }
                    } else {
                        reading_states.insert(fp, reader_info);
                    }
                }
            }
        }

        let path = home.as_ref().join(THUMBNAIL_PREVIEWS_DIRNAME);
        if !path.exists() {
            fs::create_dir(&path).ok();
        }

        let paths = if mode == LibraryMode::Database {
            db.iter()
                .map(|(fp, info)| (info.file.path.clone(), *fp))
                .collect()
        } else {
            FxHashMap::default()
        };

        let path = home.as_ref().join(FAT32_EPOCH_FILENAME);
        if !path.exists() {
            let file = File::create(&path)?;
            file.set_modified(std::time::UNIX_EPOCH + Duration::from_secs(315_532_800))?;
        }

        let fat32_epoch = path.metadata()?.modified()?;

        let sort_method = SortMethod::Opened;

        Ok(Library {
            home: home.as_ref().to_path_buf(),
            mode,
            db,
            paths,
            reading_states,
            modified_reading_states: FxHashSet::default(),
            has_db_changed: false,
            fat32_epoch,
            sort_method,
            reverse_order: sort_method.reverse_order(),
            show_hidden: false,
            import_settings: ImportSettings::default(),
        })
    }

    pub fn with_import_settings<P: AsRef<Path>>(
        home: P,
        mode: LibraryMode,
        import_settings: ImportSettings,
    ) -> Result<Self, Error> {
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
                .unwrap_or_else(|| Fp(0))
        })
    }
}
