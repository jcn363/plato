//! Smart Collection Generation Engine
//!
//! Provides automated collection management for library organization:
//! - Auto-categorization by reading status (Reading, Want to Read, Finished)
//! - Smart collections by genre/category
//! - Author-based collections
//! - Series collections
//! - Recently added
//! - Recently read

use crate::library::Library;
use anyhow::Error;
use chrono::{Duration, Local};
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;

/// Smart collection type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionType {
    ReadingStatus,
    Author,
    Genre,
    Series,
    RecentlyAdded,
    RecentlyRead,
    All,
}

/// Generate smart collections by reading status
pub fn generate_by_reading_status(
    library: &Library,
) -> Result<BTreeMap<String, Vec<String>>, Error> {
    let mut collections: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let currently_reading = collections.entry("Currently Reading".into()).or_default();
    let want_to_read = collections.entry("Want to Read".into()).or_default();
    let finished = collections.entry("Finished".into()).or_default();
    let not_started = collections.entry("Not Started".into()).or_default();

    for (fp, info) in &library.db {
        let fingerprint = fp.to_string();

        if let Some(reader_info) = &info.reader_info {
            let progress = reader_info.progress.as_ref();

            if reader_info.want_to_read {
                want_to_read.push(fingerprint);
                continue;
            }

            if reader_info.finished {
                finished.push(fingerprint);
                continue;
            }

            if let Some(progress) = progress {
                if *progress > 0.0 && *progress < 1.0 {
                    currently_reading.push(fingerprint);
                    continue;
                }
            }
        }

        not_started.push(fingerprint);
    }

    collections.retain(|_, v| !v.is_empty());
    Ok(collections)
}

/// Generate smart collections by author
pub fn generate_by_author(library: &Library) -> Result<BTreeMap<String, Vec<String>>, Error> {
    let mut collections: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let unknown = collections.entry("Unknown Author".into()).or_default();

    for (fp, info) in &library.db {
        let fingerprint = fp.to_string();

        if info.author.is_empty() {
            unknown.push(fingerprint);
        } else {
            let author = info.author.split(',').next().unwrap_or(&info.author).trim();
            collections
                .entry(author.to_string())
                .or_default()
                .push(fingerprint);
        }
    }

    collections.retain(|_, v| !v.is_empty());
    Ok(collections)
}

/// Generate smart collections by genre/category
pub fn generate_by_genre(library: &Library) -> Result<BTreeMap<String, Vec<String>>, Error> {
    let mut collections: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (fp, info) in &library.db {
        let fingerprint = fp.to_string();

        if info.categories.is_empty() {
            if !info.series.is_empty() {
                collections
                    .entry(info.series.clone())
                    .or_default()
                    .push(fingerprint);
            } else {
                collections
                    .entry("Uncategorized".into())
                    .or_default()
                    .push(fingerprint);
            }
        } else {
            for category in &info.categories {
                collections
                    .entry(category.clone())
                    .or_default()
                    .push(fingerprint.clone());
            }
        }
    }

    collections.retain(|_, v| !v.is_empty());
    Ok(collections)
}

/// Generate smart collections by series
pub fn generate_by_series(library: &Library) -> Result<BTreeMap<String, Vec<String>>, Error> {
    let mut collections: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let no_series = collections.entry("Standalone Books".into()).or_default();

    for (fp, info) in &library.db {
        let fingerprint = fp.to_string();

        if info.series.is_empty() {
            no_series.push(fingerprint);
        } else {
            let volume = info.volume.as_deref().unwrap_or("");
            let series_name = if volume.is_empty() {
                info.series.clone()
            } else {
                format!("{} ({})", info.series, volume)
            };
            collections
                .entry(series_name)
                .or_default()
                .push(fingerprint);
        }
    }

    collections.retain(|_, v| !v.is_empty());
    Ok(collections)
}

/// Generate recently added books collection
pub fn generate_recently_added(
    library: &Library,
    days: i64,
) -> Result<BTreeMap<String, Vec<String>>, Error> {
    let mut collections: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let recently_added = collections
        .entry(format!("Added in Last {} Days", days))
        .or_default();

    let cutoff = (Local::now() - Duration::days(days)).naive_local();

    for (fp, info) in &library.db {
        if info.added >= cutoff {
            recently_added.push(fp.to_string());
        }
    }

    collections.retain(|_, v| !v.is_empty());
    Ok(collections)
}

/// Generate recently read books collection
pub fn generate_recently_read(
    library: &Library,
    days: i64,
) -> Result<BTreeMap<String, Vec<String>>, Error> {
    let mut collections: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let recently_read = collections
        .entry(format!("Read in Last {} Days", days))
        .or_default();

    let cutoff = (Local::now() - Duration::days(days)).naive_local();

    for (fp, info) in &library.db {
        if let Some(reader_info) = &info.reader_info {
            if let Some(opened) = reader_info.opened {
                if opened >= cutoff {
                    recently_read.push(fp.to_string());
                }
            }
        }
    }

    collections.retain(|_, v| !v.is_empty());
    Ok(collections)
}

/// Generate all smart collections at once
pub fn generate_all(
    library: &Library,
    recent_days: i64,
) -> Result<FxHashMap<String, Vec<String>>, Error> {
    let mut all_collections: FxHashMap<String, Vec<String>> = FxHashMap::default();

    let reading_status = generate_by_reading_status(library)?;
    for (name, books) in reading_status {
        all_collections.insert(name, books);
    }

    let by_author = generate_by_author(library)?;
    for (name, books) in by_author {
        all_collections.insert(format!("Author: {}", name), books);
    }

    let by_genre = generate_by_genre(library)?;
    for (name, books) in by_genre {
        all_collections.insert(format!("Genre: {}", name), books);
    }

    let by_series = generate_by_series(library)?;
    for (name, books) in by_series {
        all_collections.insert(format!("Series: {}", name), books);
    }

    let recent_added = generate_recently_added(library, recent_days)?;
    for (name, books) in recent_added {
        all_collections.insert(name, books);
    }

    let recent_read = generate_recently_read(library, recent_days)?;
    for (name, books) in recent_read {
        all_collections.insert(name, books);
    }

    Ok(all_collections)
}

/// Legacy function for backward compatibility
pub fn generate_smart_collections(
    library: &Library,
) -> Result<BTreeMap<String, Vec<String>>, Error> {
    generate_by_reading_status(library)
}

/// Group books by first letter of author (legacy compatibility)
pub fn generate_by_author_letter(
    library: &Library,
) -> Result<BTreeMap<String, Vec<String>>, Error> {
    let mut collections: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (fp, info) in &library.db {
        let key = if info.author.is_empty() {
            "Unknown".into()
        } else {
            info.author
                .chars()
                .next()
                .map(|c| c.to_ascii_uppercase().to_string())
                .unwrap_or_else(|| "Unknown".into())
        };
        collections.entry(key).or_default().push(fp.to_string());
    }

    collections.retain(|_, v| !v.is_empty());
    Ok(collections)
}

/// Find duplicate books by exact title match and file size
pub fn find_duplicates(library: &Library) -> Result<Vec<Vec<String>>, Error> {
    let mut potential_duplicates: Vec<Vec<String>> = Vec::new();
    let books: Vec<_> = library.db.iter().collect();

    let mut title_groups: FxHashMap<String, Vec<String>> = FxHashMap::default();

    for (fp, info) in &books {
        let normalized = info.title.to_lowercase().trim().to_string();
        if !normalized.is_empty() {
            title_groups
                .entry(normalized)
                .or_default()
                .push(fp.to_string());
        }
    }

    for (_title, fingerprints) in title_groups {
        if fingerprints.len() > 1 {
            potential_duplicates.push(fingerprints);
        }
    }

    let mut size_groups: FxHashMap<u64, Vec<String>> = FxHashMap::default();
    for (fp, info) in &books {
        if info.file.size > 0 {
            size_groups
                .entry(info.file.size)
                .or_default()
                .push(fp.to_string());
        }
    }

    for (_size, fingerprints) in size_groups {
        if fingerprints.len() > 1 {
            let already_exists = potential_duplicates
                .iter()
                .any(|g| g.iter().any(|fp| fingerprints.contains(fp)));

            if !already_exists {
                potential_duplicates.push(fingerprints);
            }
        }
    }

    Ok(potential_duplicates)
}
