//! Search and replace operations for EPUB content.

use regex::Regex;
use std::fs;

use crate::editor::EpubEditorCore;
use crate::types::SearchOptions;

impl EpubEditorCore {
    /// Replaces all occurrences of a query string in all chapters.
    ///
    /// # Arguments
    ///
    /// * `query` - The search string to replace
    /// * `replacement` - The replacement string
    /// * `options` - Search options (case sensitivity, regex, whole word)
    ///
    /// # Errors
    ///
    /// Returns an error if writing any chapter content to file fails.
    ///
    /// # Returns
    ///
    /// The total number of replacements made across all chapters.
    pub fn replace_all_in_all_chapters(
        &mut self,
        query: &str,
        replacement: &str,
        options: SearchOptions,
    ) -> anyhow::Result<usize> {
        let mut total_replacements = 0;

        for index in 0..self.chapters.len() {
            let replacements = self.replace_in_chapter(index, query, replacement, options)?;
            total_replacements += replacements;
        }

        Ok(total_replacements)
    }

    /// Searches for a query string within a specific chapter.
    ///
    /// # Arguments
    ///
    /// * `index` - The chapter index to search
    /// * `query` - The search string
    /// * `options` - Search options (case sensitivity, regex, whole word)
    ///
    /// # Returns
    ///
    /// A vector of (start, end) byte positions for each match.
    #[must_use]
    pub fn search_in_chapter(
        &self,
        index: usize,
        query: &str,
        options: SearchOptions,
    ) -> Vec<(usize, usize)> {
        if index >= self.chapters.len() || query.is_empty() {
            return Vec::new();
        }
        let content = &self.chapters[index].content;
        let search_content = if options.case_sensitive {
            content
        } else {
            &content.to_lowercase()
        };
        let search_query = if options.case_sensitive {
            query
        } else {
            &query.to_lowercase()
        };
        let mut matches = Vec::new();
        let mut start = 0;

        if options.use_regex {
            if let Ok(re) = Regex::new(search_query) {
                for mat in re.find_iter(search_content) {
                    matches.push((mat.start(), mat.end()));
                }
            }
        } else {
            while let Some(pos) = search_content[start..].find(search_query) {
                let abs_pos = start + pos;
                if options.whole_word {
                    let before = if abs_pos > 0 {
                        search_content.chars().nth(abs_pos - 1)
                    } else {
                        None
                    };
                    let after = search_content.chars().nth(abs_pos + search_query.len());
                    let is_word_boundary = before.is_none_or(|c| !c.is_alphanumeric())
                        && after.is_none_or(|c| !c.is_alphanumeric());
                    if is_word_boundary {
                        matches.push((abs_pos, abs_pos + search_query.len()));
                    }
                } else {
                    matches.push((abs_pos, abs_pos + search_query.len()));
                }
                start = abs_pos + 1;
            }
        }
        matches
    }

    /// Replaces occurrences of a search string in a specific chapter.
    ///
    /// # Arguments
    ///
    /// * `index` - The chapter index to modify
    /// * `search` - The search string to replace
    /// * `replace` - The replacement string
    /// * `options` - Search options (case sensitivity, regex, whole word)
    ///
    /// # Errors
    ///
    /// Returns an error if writing the updated chapter content to file fails.
    ///
    /// # Returns
    ///
    /// The number of replacements made.
    pub fn replace_in_chapter(
        &mut self,
        index: usize,
        search: &str,
        replace: &str,
        options: SearchOptions,
    ) -> anyhow::Result<usize> {
        if index >= self.chapters.len() || search.is_empty() {
            return Ok(0);
        }
        let old_content = self.chapters[index].content.clone();
        let (search_content, search_query) = if options.case_sensitive {
            (old_content.clone(), search.to_string())
        } else {
            (old_content.to_lowercase(), search.to_lowercase())
        };

        let count = if options.use_regex {
            if let Ok(re) = Regex::new(&search_query) {
                re.find_iter(&search_content).count()
            } else {
                return Ok(0);
            }
        } else if options.whole_word {
            let mut c = 0;
            let mut start = 0;
            while let Some(pos) = search_content[start..].find(&search_query) {
                let abs_pos = start + pos;
                let before = if abs_pos > 0 {
                    search_content.chars().nth(abs_pos - 1)
                } else {
                    None
                };
                let after = search_content.chars().nth(abs_pos + search_query.len());
                let is_word_boundary = before.is_none_or(|c| !c.is_alphanumeric())
                    && after.is_none_or(|c| !c.is_alphanumeric());
                if is_word_boundary {
                    c += 1;
                }
                start = abs_pos + 1;
            }
            c
        } else {
            search_content.matches(&search_query).count()
        };

        if count == 0 {
            return Ok(0);
        }

        let original_content = self.chapters[index].content.clone();
        let new_content = if options.use_regex {
            if let Ok(re) = Regex::new(&search_query) {
                re.replace_all(&old_content, replace).to_string()
            } else {
                old_content
            }
        } else {
            old_content.replace(search, replace)
        };

        self.undo_stack
            .push(crate::types::UndoAction::Chapter(index, original_content));
        self.redo_stack.clear();
        self.chapters[index].content = new_content.clone();
        let chapter = &self.chapters[index];
        let file_path = self.temp_dir.join(&chapter.href);
        fs::write(&file_path, &new_content)?;
        Ok(count)
    }

    /// Replaces all occurrences of a search string with a replacement string in all chapters.
    ///
    /// # Errors
    ///
    /// Returns an error if writing any chapter content to file fails during replacement.
    pub fn replace_all_in_document(
        &mut self,
        search: &str,
        replace: &str,
        options: SearchOptions,
    ) -> anyhow::Result<usize> {
        if search.is_empty() {
            return Ok(0);
        }
        let mut total = 0;
        for i in 0..self.chapters.len() {
            let count = self.replace_in_chapter(i, search, replace, options)?;
            total += count;
        }
        Ok(total)
    }

    /// Searches for a query string across all chapters.
    ///
    /// # Arguments
    ///
    /// * `query` - The search string
    /// * `options` - Search options (case sensitivity, regex, whole word)
    ///
    /// # Returns
    ///
    /// A vector of (`chapter_index`, matches) tuples for chapters containing matches.
    #[must_use]
    pub fn search_all_chapters(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> Vec<(usize, Vec<(usize, usize)>)> {
        if query.is_empty() {
            return Vec::new();
        }
        let mut results = Vec::new();
        for i in 0..self.chapters.len() {
            let matches = self.search_in_chapter(i, query, options);
            if !matches.is_empty() {
                results.push((i, matches));
            }
        }
        results
    }
}
