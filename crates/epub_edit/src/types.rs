#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// Metadata extracted from an EPUB file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubMetadata {
    /// The title of the EPUB.
    pub title: String,
    /// The author of the EPUB.
    pub author: String,
    /// The language code of the EPUB (e.g., "en").
    pub language: String,
    /// The unique identifier of the EPUB.
    pub identifier: String,
    /// The publisher of the EPUB, if available.
    pub publisher: Option<String>,
    /// The publication date of the EPUB, if available.
    pub date: Option<String>,
    /// A description of the EPUB, if available.
    pub description: Option<String>,
}

impl Default for EpubMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            language: String::from("en"),
            identifier: String::new(),
            publisher: None,
            date: None,
            description: None,
        }
    }
}

/// A chapter in an EPUB document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubChapter {
    /// The unique ID of the chapter.
    pub id: String,
    /// The relative path to the chapter file.
    pub href: String,
    /// The title of the chapter.
    pub title: String,
    /// The HTML content of the chapter.
    pub content: String,
}

/// An action that can be undone/redone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UndoAction {
    /// A metadata change action.
    Metadata(EpubMetadata),
    /// A chapter content change action with index and old content.
    Chapter(usize, String),
    /// A chapter rename action with index and old title.
    RenameChapter(usize, String),
    /// A chapter reorder action with from and to indices.
    ReorderChapters(usize, usize),
}

/// Options for search and replace operations.
#[derive(Debug, Clone, Copy, Default)]
pub struct SearchOptions {
    /// Whether to use regex for matching.
    pub use_regex: bool,
    /// Whether the search is case-sensitive.
    pub case_sensitive: bool,
    /// Whether to match whole words only.
    pub whole_word: bool,
}

/// A validation issue found in an EPUB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// The index of the chapter with the issue.
    pub chapter_index: usize,
    /// The title of the chapter with the issue.
    pub chapter_title: String,
    /// The type of validation issue.
    pub issue_type: String,
    /// A descriptive message about the issue.
    pub message: String,
    /// The location in the chapter where the issue was found, if available.
    pub location: Option<String>,
}

/// Result of validating an EPUB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// All validation issues found.
    pub issues: Vec<ValidationIssue>,
    /// The total number of chapters in the EPUB.
    pub total_chapters: usize,
    /// The number of chapters that have issues.
    pub chapters_with_issues: usize,
}

/// A spelling error found in an EPUB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellError {
    /// The index of the chapter containing the error.
    pub chapter_index: usize,
    /// The title of the chapter containing the error.
    pub chapter_title: String,
    /// The misspelled word.
    pub word: String,
    /// The position of the word in the chapter.
    pub position: usize,
    /// Suggested corrections for the word.
    pub suggestions: Vec<String>,
}

/// Result of spell checking an EPUB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellCheckResult {
    /// All spelling errors found.
    pub errors: Vec<SpellError>,
    /// The total number of words checked.
    pub total_words: usize,
    /// The number of chapters that were checked.
    pub chapters_checked: usize,
}

/// Statistics for a chapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterStatistics {
    /// The index of the chapter.
    pub chapter_index: usize,
    /// The title of the chapter.
    pub chapter_title: String,
    /// The number of words in the chapter.
    pub word_count: usize,
    /// The number of characters in the chapter.
    pub character_count: usize,
    /// The number of paragraphs in the chapter.
    pub paragraph_count: usize,
}

/// Information about an image in an EPUB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// The index of the chapter containing the image.
    pub chapter_index: usize,
    /// The title of the chapter containing the image.
    pub chapter_title: String,
    /// The source path of the image.
    pub src: String,
    /// The alt text of the image, if available.
    pub alt: Option<String>,
}

/// Information about CSS in an EPUB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSInfo {
    /// The index of the chapter referencing the CSS.
    pub chapter_index: usize,
    /// The title of the chapter referencing the CSS.
    pub chapter_title: String,
    /// The href of the CSS file.
    pub href: String,
    /// The media type of the CSS, if available.
    pub media_type: Option<String>,
}

/// A bookmark in an EPUB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// The index of the chapter containing the bookmark.
    pub chapter_index: usize,
    /// The title of the chapter containing the bookmark.
    pub chapter_title: String,
    /// The position in the chapter where the bookmark is set.
    pub position: usize,
    /// An optional note associated with the bookmark.
    pub note: Option<String>,
}
