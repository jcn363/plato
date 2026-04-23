use crate::metadata::sorter;
use crate::metadata::{Info, ReaderInfo, SimpleStatus, SortMethod};
use crate::settings::LibraryMode;
use levenshtein::levenshtein;
use regex::Regex;
use std::fs;
use std::path::Path;

use super::types::{Library, THUMBNAIL_PREVIEWS_DIRNAME};

impl Library {
    /// Fuzzy search for books by title or author using Levenshtein distance
    /// Returns books with similarity score above threshold (default: 0.7)
    pub fn fuzzy_search(&self, query: &str, threshold: Option<f64>) -> Vec<(String, Info)> {
        if query.is_empty() {
            return Vec::new();
        }
        let mut threshold = threshold.unwrap_or(0.7);
        if !(0.0..=1.0).contains(&threshold) {
            eprintln!("Fuzzy search threshold must be between 0.0 and 1.0, using default 0.7");
            threshold = 0.7;
        }
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        if self.mode == LibraryMode::Filesystem {
            return results;
        }

        for (fp, info) in &self.db {
            let title_lower = info.title.to_lowercase();
            let author_lower = info.author.to_lowercase();

            // Calculate similarity scores
            let title_distance = levenshtein(&query_lower, &title_lower);
            let title_similarity =
                1.0 - (title_distance as f64 / query_lower.len().max(title_lower.len()) as f64);

            let author_distance = levenshtein(&query_lower, &author_lower);
            let author_similarity =
                1.0 - (author_distance as f64 / query_lower.len().max(author_lower.len()) as f64);

            // Use the higher similarity score
            let max_similarity = title_similarity.max(author_similarity);

            if max_similarity >= threshold {
                results.push((fp.to_string(), info.clone()));
            }
        }

        results
    }

    /// Advanced regex search for books by title, author, or other metadata
    /// Returns books matching the regex pattern
    pub fn regex_search(&self, pattern: &str) -> Result<Vec<(String, Info)>, regex::Error> {
        if pattern.is_empty() {
            return Ok(Vec::new());
        }
        let regex = Regex::new(pattern)?;
        let mut results = Vec::new();

        if self.mode == LibraryMode::Filesystem {
            return Ok(results);
        }

        for (fp, info) in &self.db {
            // Search in title
            if regex.is_match(&info.title) {
                results.push((fp.to_string(), info.clone()));
                continue;
            }

            // Search in author
            if regex.is_match(&info.author) {
                results.push((fp.to_string(), info.clone()));
                continue;
            }

            // Search in series
            if regex.is_match(&info.series) {
                results.push((fp.to_string(), info.clone()));
                continue;
            }
        }

        Ok(results)
    }

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
                    let reader_info = self.reading_states.entry(fp).or_default();
                    reader_info.finished = status == SimpleStatus::Finished;
                    self.modified_reading_states.insert(fp);
                }
            }
        }
    }
}
