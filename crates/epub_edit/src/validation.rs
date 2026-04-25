//! Content validation and spell checking for EPUB files.

use regex::Regex;
use std::sync::LazyLock;

static WORD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[a-zA-Z]+").expect("invalid regex"));
static HREF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"href="([^"]+)""#).expect("invalid regex"));
static IMG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<img[^>]+src="([^"]+)""#).expect("invalid regex"));
use crate::editor::EpubEditorCore;
use crate::types::{
    ChapterStatistics, SpellCheckResult, SpellError, ValidationIssue, ValidationResult,
};

pub(crate) struct ValidationHelpers;

impl ValidationHelpers {
    pub(crate) fn strip_html_tags(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;

        for ch in html.chars() {
            if ch == '<' {
                in_tag = true;
            } else if ch == '>' {
                in_tag = false;
            } else if !in_tag {
                result.push(ch);
            }
        }
        result
    }

    fn extract_words(text: &str) -> Vec<String> {
        WORD_RE
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn is_potential_misspelling(word: &str) -> bool {
        if word.len() < 3 {
            return false;
        }

        let common_words = [
            "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was",
            "one", "our", "out", "has", "his", "how", "its", "may", "new", "now", "old", "see",
            "two", "way", "who", "boy", "did", "get", "has", "him", "his", "how", "its", "let",
            "may", "new", "now", "off", "old", "one", "our", "out", "put", "say", "see", "she",
            "too", "use", "was", "way", "who", "with", "yes",
        ];

        if common_words.contains(&word.to_lowercase().as_str()) {
            return false;
        }

        false
    }
}

impl EpubEditorCore {
    /// Validates the EPUB content for common issues.
    ///
    /// Checks for HTML structure problems, broken links, and external image references.
    ///
    /// # Returns
    ///
    /// A validation result containing all found issues and statistics.
    #[must_use]
    pub fn validate_content(&self) -> ValidationResult {
        let mut issues = Vec::new();
        for (index, chapter) in self.chapters.iter().enumerate() {
            Self::validate_chapter_content(index, chapter, &mut issues);
        }
        let chapters_with_issues = issues
            .iter()
            .map(|i| i.chapter_index)
            .collect::<std::collections::HashSet<_>>()
            .len();
        ValidationResult {
            issues,
            total_chapters: self.chapters.len(),
            chapters_with_issues,
        }
    }

    /// Performs a basic spell check on all chapters.
    ///
    /// Strips HTML tags and identifies potential misspellings by filtering out common words.
    /// This is a simple heuristic check, not a full dictionary-based spell checker.
    ///
    /// # Returns
    ///
    /// A spell check result containing potential errors and statistics.
    #[must_use]
    pub fn spell_check(&self) -> SpellCheckResult {
        let mut errors = Vec::new();
        let mut total_words = 0;

        for (index, chapter) in self.chapters.iter().enumerate() {
            let content = ValidationHelpers::strip_html_tags(&chapter.content);
            let words = ValidationHelpers::extract_words(&content);
            total_words += words.len();

            for (pos, word) in words.iter().enumerate() {
                if ValidationHelpers::is_potential_misspelling(word) {
                    errors.push(SpellError {
                        chapter_index: index,
                        chapter_title: chapter.title.clone(),
                        word: word.clone(),
                        position: pos,
                        suggestions: Vec::new(),
                    });
                }
            }
        }

        SpellCheckResult {
            errors,
            total_words,
            chapters_checked: self.chapters.len(),
        }
    }

    /// Returns statistics for a specific chapter.
    ///
    /// # Arguments
    ///
    /// * `index` - The chapter index
    ///
    /// # Returns
    ///
    /// Chapter statistics including word count, character count, and paragraph count,
    /// or None if the index is out of bounds.
    #[must_use]
    pub fn get_chapter_statistics(&self, index: usize) -> Option<ChapterStatistics> {
        if index >= self.chapters.len() {
            return None;
        }
        let chapter = &self.chapters[index];
        let content = ValidationHelpers::strip_html_tags(&chapter.content);
        let words = ValidationHelpers::extract_words(&content);
        let paragraphs = content.split('\n').filter(|p| !p.trim().is_empty()).count();

        Some(ChapterStatistics {
            chapter_index: index,
            chapter_title: chapter.title.clone(),
            word_count: words.len(),
            character_count: content.chars().count(),
            paragraph_count: paragraphs,
        })
    }

    /// Returns statistics for all chapters.
    ///
    /// # Returns
    ///
    /// A vector of chapter statistics for all chapters.
    #[must_use]
    pub fn get_all_chapters_statistics(&self) -> Vec<ChapterStatistics> {
        self.chapters
            .iter()
            .enumerate()
            .filter_map(|(index, _)| self.get_chapter_statistics(index))
            .collect()
    }

    fn validate_chapter_content(
        index: usize,
        chapter: &crate::types::EpubChapter,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let content = &chapter.content;

        Self::check_html_structure(content, index, chapter, issues);
        Self::check_broken_links(content, index, chapter, issues);
        Self::check_missing_images(content, index, chapter, issues);
    }

    fn check_html_structure(
        content: &str,
        index: usize,
        chapter: &crate::types::EpubChapter,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let mut tag_stack = Vec::new();
        let mut in_tag = false;
        let mut current_tag = String::new();

        for (pos, ch) in content.char_indices() {
            if ch == '<' {
                in_tag = true;
                if !current_tag.is_empty() && !current_tag.starts_with('/') {
                    tag_stack.push((current_tag.clone(), pos));
                }
                current_tag.clear();
            } else if ch == '>' {
                in_tag = false;
                if current_tag.starts_with('/') {
                    if let Some((tag, _)) = tag_stack.pop() {
                        let closing_tag = current_tag.trim_start_matches('/');
                        if tag != closing_tag {
                            issues.push(ValidationIssue {
                                chapter_index: index,
                                chapter_title: chapter.title.clone(),
                                issue_type: "HTML Structure".to_string(),
                                message: format!(
                                    "Mismatched tags: expected </{}>, found </{}>",
                                    tag, closing_tag
                                ),
                                location: Some(format!("Position {}", pos)),
                            });
                        }
                    }
                }
                current_tag.clear();
            } else if in_tag {
                if ch == ' ' {
                    in_tag = false;
                } else {
                    current_tag.push(ch);
                }
            }
        }

        if !tag_stack.is_empty() {
            for (tag, pos) in tag_stack {
                issues.push(ValidationIssue {
                    chapter_index: index,
                    chapter_title: chapter.title.clone(),
                    issue_type: "HTML Structure".to_string(),
                    message: format!("Unclosed tag: <{}>", tag),
                    location: Some(format!("Position {}", pos)),
                });
            }
        }
    }

    fn check_broken_links(
        content: &str,
        index: usize,
        chapter: &crate::types::EpubChapter,
        issues: &mut Vec<ValidationIssue>,
    ) {
        for cap in HREF_RE.captures_iter(content) {
            if let Some(href_match) = cap.get(1) {
                let href = href_match.as_str();
                if href.is_empty() || href == "#" {
                    issues.push(ValidationIssue {
                        chapter_index: index,
                        chapter_title: chapter.title.clone(),
                        issue_type: "Broken Link".to_string(),
                        message: format!("Empty or invalid href: {}", href),
                        location: Some(format!("Position {}", href_match.start())),
                    });
                }
            }
        }
    }

    fn check_missing_images(
        content: &str,
        index: usize,
        chapter: &crate::types::EpubChapter,
        issues: &mut Vec<ValidationIssue>,
    ) {
        for cap in IMG_RE.captures_iter(content) {
            if let Some(src_match) = cap.get(1) {
                let src = src_match.as_str();
                if src.starts_with("http://") || src.starts_with("https://") {
                    issues.push(ValidationIssue {
                        chapter_index: index,
                        chapter_title: chapter.title.clone(),
                        issue_type: "External Image".to_string(),
                        message: format!("External image reference: {}", src),
                        location: Some(format!("Position {}", src_match.start())),
                    });
                }
            }
        }
    }
}
