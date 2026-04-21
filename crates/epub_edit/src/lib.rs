use anyhow::{bail, format_err, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

static TITLE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<dc:title[^>]*>([^<]+)</dc:title>"#).unwrap());
static AUTHOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<dc:creator[^>]*>([^<]+)</dc:creator>"#).unwrap());
static LANGUAGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<dc:language[^>]*>([^<]+)</dc:language>"#).unwrap());
static IDENTIFIER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<dc:identifier[^>]*>([^<]+)</dc:identifier>"#).unwrap());
static PUBLISHER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<dc:publisher[^>]*>([^<]+)</dc:publisher>"#).unwrap());
static DATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<dc:date[^>]*>([^<]+)</dc:date>"#).unwrap());
static DESCRIPTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<dc:description[^>]*>([^<]+)</dc:description>"#).unwrap());
static ROOTFILE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"rootfile[^"]*"?([^"]+)"?"#).unwrap());
static ITEM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<item[^>]+href="([^"]+)"[^>]+id="([^"]+)"[^>]*>"#).unwrap());
static SPINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<itemref[^>]+idref="([^"]+)"[^>]*>"#).unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubMetadata {
    pub title: String,
    pub author: String,
    pub language: String,
    pub identifier: String,
    pub publisher: Option<String>,
    pub date: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubChapter {
    pub id: String,
    pub href: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UndoAction {
    Metadata(EpubMetadata),
    Chapter(usize, String),
    RenameChapter(usize, String),
    ReorderChapters(usize, usize),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchOptions {
    pub use_regex: bool,
    pub case_sensitive: bool,
    pub whole_word: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub chapter_index: usize,
    pub chapter_title: String,
    pub issue_type: String,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
    pub total_chapters: usize,
    pub chapters_with_issues: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellError {
    pub chapter_index: usize,
    pub chapter_title: String,
    pub word: String,
    pub position: usize,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellCheckResult {
    pub errors: Vec<SpellError>,
    pub total_words: usize,
    pub chapters_checked: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterStatistics {
    pub chapter_index: usize,
    pub chapter_title: String,
    pub word_count: usize,
    pub character_count: usize,
    pub paragraph_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub chapter_index: usize,
    pub chapter_title: String,
    pub src: String,
    pub alt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSInfo {
    pub chapter_index: usize,
    pub chapter_title: String,
    pub href: String,
    pub media_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub chapter_index: usize,
    pub chapter_title: String,
    pub position: usize,
    pub note: Option<String>,
}

const MAX_HISTORY_SIZE: usize = 50;

pub struct EpubEditorCore {
    pub epub_path: PathBuf,
    pub metadata: EpubMetadata,
    pub chapters: Vec<EpubChapter>,
    pub temp_dir: PathBuf,
    pub undo_stack: Vec<UndoAction>,
    pub redo_stack: Vec<UndoAction>,
    pub bookmarks: Vec<Bookmark>,
}

impl EpubEditorCore {
    /// Creates a new EpubEditorCore instance from an EPUB file path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The EPUB file cannot be opened
    /// * The ZIP archive cannot be read
    /// * Creating the temporary directory fails
    /// * Extracting the EPUB contents fails
    /// * Parsing metadata fails
    /// * Parsing content fails
    pub fn new(epub_path: &str) -> Result<Self> {
        let temp_dir = Self::create_temp_dir()?;
        let mut editor = Self {
            epub_path: PathBuf::from(epub_path),
            metadata: EpubMetadata::default(),
            chapters: Vec::new(),
            temp_dir,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            bookmarks: Vec::new(),
        };

        editor.extract()?;
        editor.parse_metadata()?;
        editor.parse_content()?;

        Ok(editor)
    }

    /// Creates a temporary directory for EPUB processing.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Creating the temporary directory fails
    /// * Removing an existing temporary directory fails
    fn create_temp_dir() -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir().join(format!("epub_editor_{}", uuid::Uuid::new_v4()));
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;
        Ok(temp_dir)
    }

    /// Extracts the EPUB archive to the temporary directory.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Opening the EPUB file fails
    /// * Reading the ZIP archive fails
    /// * Creating directories for extracted files fails
    /// * Writing extracted files fails
    fn extract(&self) -> Result<()> {
        let file = File::open(&self.epub_path).context("Failed to open EPUB file")?;
        let mut archive = ZipArchive::new(file).context("Failed to read ZIP archive")?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;

            // Prevent zip slip: reject entries containing path traversal components
            let name = file.name();
            if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
                return Err(format_err!("Zip entry contains path traversal: {}", name));
            }

            let outpath = self.temp_dir.join(name);

            if name.ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
        Ok(())
    }

    /// Parses metadata from the EPUB's OPF file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The container.xml file is missing
    /// * Reading the container.xml file fails
    /// * The rootfile path cannot be found in container.xml
    /// * The OPF file is missing at the specified path
    /// * Reading the OPF file fails
    fn parse_metadata(&mut self) -> Result<()> {
        let container_path = self.temp_dir.join("META-INF/container.xml");
        if !container_path.exists() {
            return Err(format_err!("META-INF/container.xml not found"));
        }

        let container_content = fs::read_to_string(&container_path)?;

        if let Some(caps) = ROOTFILE_RE.captures(&container_content) {
            let opf_path = caps
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or("OEBPS/content.opf");
            let opf_full_path = self.temp_dir.join(opf_path);

            if opf_full_path.exists() {
                let opf_content = fs::read_to_string(&opf_full_path)?;
                self.parse_opf_metadata(&opf_content);
            } else {
                return Err(format_err!("OPF file not found at {}", opf_path));
            }
        } else {
            return Err(format_err!("Could not find rootfile in container.xml"));
        }
        Ok(())
    }

    fn parse_opf_metadata(&mut self, opf_content: &str) {
        if let Some(caps) = TITLE_RE.captures(opf_content) {
            self.metadata.title = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
        }
        if let Some(caps) = AUTHOR_RE.captures(opf_content) {
            self.metadata.author = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
        }
        if let Some(caps) = LANGUAGE_RE.captures(opf_content) {
            self.metadata.language = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "en".to_string());
        }
        if let Some(caps) = IDENTIFIER_RE.captures(opf_content) {
            self.metadata.identifier = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
        }
        if let Some(caps) = PUBLISHER_RE.captures(opf_content) {
            self.metadata.publisher = caps.get(1).map(|m| m.as_str().to_string());
        }
        if let Some(caps) = DATE_RE.captures(opf_content) {
            self.metadata.date = caps.get(1).map(|m| m.as_str().to_string());
        }
        if let Some(caps) = DESCRIPTION_RE.captures(opf_content) {
            self.metadata.description = caps.get(1).map(|m| m.as_str().to_string());
        }
    }

    #[must_use]
    pub fn to_plato_metadata(&self) -> EpubMetadata {
        self.metadata.clone()
    }

    /// Parses the content structure from the EPUB's OPF file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The container.xml file is missing
    /// * Reading the container.xml file fails
    /// * The rootfile path cannot be found in container.xml
    /// * The OPF file is missing at the specified path
    /// * Reading the OPF file fails
    fn parse_content(&mut self) -> Result<()> {
        let container_path = self.temp_dir.join("META-INF/container.xml");
        let container_content = fs::read_to_string(&container_path)?;

        if let Some(caps) = ROOTFILE_RE.captures(&container_content) {
            let opf_path = caps
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or("OEBPS/content.opf");
            let opf_full_path = self.temp_dir.join(opf_path);

            if opf_full_path.exists() {
                let opf_content = fs::read_to_string(&opf_full_path)?;
                self.parse_opf_content(&opf_content, opf_path);
            }
        }
        Ok(())
    }

    fn parse_opf_content(&mut self, opf_content: &str, opf_dir: &str) {
        let mut item_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for caps in ITEM_RE.captures_iter(opf_content) {
            let href = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let id = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            item_map.insert(id, href);
        }

        let mut order: Vec<String> = Vec::new();
        for caps in SPINE_RE.captures_iter(opf_content) {
            let idref = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            order.push(idref);
        }

        let opf_parent = Path::new(opf_dir).parent().unwrap_or(Path::new(""));
        for idref in order {
            if let Some(href) = item_map.get(&idref) {
                let full_path = self.temp_dir.join(opf_parent).join(href);
                if full_path.exists() {
                    if let Ok(content) = fs::read_to_string(&full_path) {
                        self.chapters.push(EpubChapter {
                            id: idref.clone(),
                            href: href.clone(),
                            title: Self::extract_title(&content).unwrap_or_else(|| href.clone()),
                            content,
                        });
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn extract_title(html: &str) -> Option<String> {
        let title_regex = Regex::new(r"(?i)<title[^>]*>([^<]+)</title>").ok()?;
        let h1_regex = Regex::new(r"(?i)<h1[^>]*>([^<]+)</h1>").ok()?;

        if let Some(caps) = title_regex.captures(html) {
            return Some(
                caps.get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
            );
        }
        if let Some(caps) = h1_regex.captures(html) {
            return Some(
                caps.get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
            );
        }
        None
    }

    #[must_use]
    pub fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => ' ',
                _ => c,
            })
            .collect::<String>()
            .trim()
            .to_string()
    }

    pub fn set_metadata(&mut self, metadata: EpubMetadata) {
        self.undo_stack
            .push(UndoAction::Metadata(self.metadata.clone()));
        self.redo_stack.clear();
        self.metadata = metadata;
    }

    /// Updates the content of a chapter at the specified index.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Writing the updated chapter content to file fails
    pub fn update_chapter(&mut self, index: usize, content: String) -> Result<()> {
        if index < self.chapters.len() {
            let old_content = self.chapters[index].content.clone();
            self.undo_stack
                .push(UndoAction::Chapter(index, old_content));
            self.redo_stack.clear();
            self.chapters[index].content = content;

            let chapter = &self.chapters[index];
            let file_path = self.temp_dir.join(&chapter.href);
            fs::write(&file_path, &chapter.content)?;
        }
        Ok(())
    }

    /// Undoes the last action performed on the EPUB.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Writing the undone content to file fails
    pub fn undo(&mut self) -> Result<bool> {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                UndoAction::Metadata(old_meta) => {
                    self.redo_stack
                        .push(UndoAction::Metadata(self.metadata.clone()));
                    self.metadata = old_meta;
                }
                UndoAction::Chapter(index, old_content) => {
                    if index < self.chapters.len() {
                        let current = self.chapters[index].content.clone();
                        self.redo_stack.push(UndoAction::Chapter(index, current));
                        self.chapters[index].content = old_content;
                        let chapter = &self.chapters[index];
                        let file_path = self.temp_dir.join(&chapter.href);
                        fs::write(&file_path, &chapter.content)?;
                    }
                }
                UndoAction::RenameChapter(index, old_title) => {
                    if index < self.chapters.len() {
                        let current = self.chapters[index].title.clone();
                        self.redo_stack
                            .push(UndoAction::RenameChapter(index, current));
                        self.chapters[index].title = old_title;
                    }
                }
                UndoAction::ReorderChapters(from_index, to_index) => {
                    if from_index < self.chapters.len() && to_index < self.chapters.len() {
                        let chapter = self.chapters.remove(from_index);
                        self.chapters.insert(to_index, chapter);
                        self.redo_stack
                            .push(UndoAction::ReorderChapters(to_index, from_index));
                    }
                }
            }
            if self.redo_stack.len() > MAX_HISTORY_SIZE {
                self.redo_stack.remove(0);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Redoes the last undone action on the EPUB.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Writing the redone content to file fails
    pub fn redo(&mut self) -> Result<bool> {
        if let Some(action) = self.redo_stack.pop() {
            match action {
                UndoAction::Metadata(new_meta) => {
                    self.undo_stack
                        .push(UndoAction::Metadata(self.metadata.clone()));
                    self.metadata = new_meta;
                }
                UndoAction::Chapter(index, new_content) => {
                    if index < self.chapters.len() {
                        let current = self.chapters[index].content.clone();
                        self.undo_stack.push(UndoAction::Chapter(index, current));
                        self.chapters[index].content = new_content;
                        let chapter = &self.chapters[index];
                        let file_path = self.temp_dir.join(&chapter.href);
                        fs::write(&file_path, &chapter.content)?;
                    }
                }
                UndoAction::RenameChapter(index, new_title) => {
                    if index < self.chapters.len() {
                        let current = self.chapters[index].title.clone();
                        self.undo_stack
                            .push(UndoAction::RenameChapter(index, current));
                        self.chapters[index].title = new_title;
                    }
                }
                UndoAction::ReorderChapters(from_index, to_index) => {
                    if from_index < self.chapters.len() && to_index < self.chapters.len() {
                        let chapter = self.chapters.remove(from_index);
                        self.chapters.insert(to_index, chapter);
                        self.undo_stack
                            .push(UndoAction::ReorderChapters(to_index, from_index));
                    }
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn add_bookmark(&mut self, index: usize, position: usize, note: Option<String>) {
        if index < self.chapters.len() {
            let bookmark = Bookmark {
                chapter_index: index,
                chapter_title: self.chapters[index].title.clone(),
                position,
                note,
            };
            self.bookmarks.push(bookmark);
        }
    }

    pub fn remove_bookmark(&mut self, index: usize) {
        if index < self.bookmarks.len() {
            self.bookmarks.remove(index);
        }
    }

    #[must_use]
    pub fn list_bookmarks(&self) -> Vec<Bookmark> {
        self.bookmarks.clone()
    }

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
                    let is_word_boundary = before.map_or(true, |c| !c.is_alphanumeric())
                        && after.map_or(true, |c| !c.is_alphanumeric());
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

    pub fn replace_in_chapter(
        &mut self,
        index: usize,
        search: &str,
        replace: &str,
        options: SearchOptions,
    ) -> Result<usize> {
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
                let is_word_boundary = before.map_or(true, |c| !c.is_alphanumeric())
                    && after.map_or(true, |c| !c.is_alphanumeric());
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
            .push(UndoAction::Chapter(index, original_content));
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
    /// Returns an error if:
    /// * Writing any chapter content to file fails during replacement
    pub fn replace_all_in_document(
        &mut self,
        search: &str,
        replace: &str,
        options: SearchOptions,
    ) -> Result<usize> {
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

    #[must_use]
    pub fn spell_check(&self) -> SpellCheckResult {
        let mut errors = Vec::new();
        let mut total_words = 0;

        for (index, chapter) in self.chapters.iter().enumerate() {
            let content = Self::strip_html_tags(&chapter.content);
            let words = Self::extract_words(&content);
            total_words += words.len();

            for (pos, word) in words.iter().enumerate() {
                if Self::is_potential_misspelling(word) {
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

    #[must_use]
    pub fn get_chapter_statistics(&self, index: usize) -> Option<ChapterStatistics> {
        if index >= self.chapters.len() {
            return None;
        }
        let chapter = &self.chapters[index];
        let content = Self::strip_html_tags(&chapter.content);
        let words = Self::extract_words(&content);
        let paragraphs = content.split('\n').filter(|p| !p.trim().is_empty()).count();

        Some(ChapterStatistics {
            chapter_index: index,
            chapter_title: chapter.title.clone(),
            word_count: words.len(),
            character_count: content.chars().count(),
            paragraph_count: paragraphs,
        })
    }

    #[must_use]
    pub fn get_all_chapters_statistics(&self) -> Vec<ChapterStatistics> {
        self.chapters
            .iter()
            .enumerate()
            .filter_map(|(index, _)| self.get_chapter_statistics(index))
            .collect()
    }

    pub fn generate_table_of_contents(&self) -> String {
        let mut toc = String::new();
        toc.push_str("<nav epub:type=\"toc\">\n");
        toc.push_str("  <ol>\n");

        for (_index, chapter) in self.chapters.iter().enumerate() {
            toc.push_str(&format!(
                "    <li><a href=\"{}\">{}</a></li>\n",
                chapter.href, chapter.title
            ));
        }

        toc.push_str("  </ol>\n");
        toc.push_str("</nav>\n");
        toc
    }

    pub fn update_table_of_contents(&mut self) -> Result<()> {
        let toc_content = self.generate_table_of_contents();
        let toc_path = self.temp_dir.join("toc.xhtml");
        fs::write(&toc_path, toc_content).with_context(|| {
            format!(
                "Failed to write table of contents to {}",
                toc_path.display()
            )
        })?;
        Ok(())
    }

    #[must_use]
    pub fn list_images(&self) -> Vec<ImageInfo> {
        let mut images = Vec::new();
        let img_re = Regex::new(r#"<img[^>]+src=["']([^"']+)["'][^>]*>"#).unwrap();

        for (index, chapter) in self.chapters.iter().enumerate() {
            for cap in img_re.captures_iter(&chapter.content) {
                let src = cap
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let alt = Regex::new(r#"alt=["']([^"']*)["']"#)
                    .unwrap()
                    .captures(&cap[0])
                    .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

                images.push(ImageInfo {
                    chapter_index: index,
                    chapter_title: chapter.title.clone(),
                    src,
                    alt,
                });
            }
        }

        images
    }

    #[must_use]
    pub fn list_css(&self) -> Vec<CSSInfo> {
        let mut css_files = Vec::new();
        let link_re =
            Regex::new(r#"<link[^>]+rel=["']stylesheet["'][^>]*href=["']([^"']+)["'][^>]*>"#)
                .unwrap();
        let style_re = Regex::new(r#"<style[^>]*>(.*?)</style>"#).unwrap();

        for (index, chapter) in self.chapters.iter().enumerate() {
            for cap in link_re.captures_iter(&chapter.content) {
                let href = cap
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let media_type = Regex::new(r#"media=["']([^"']*)["']"#)
                    .unwrap()
                    .captures(&cap[0])
                    .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

                css_files.push(CSSInfo {
                    chapter_index: index,
                    chapter_title: chapter.title.clone(),
                    href,
                    media_type,
                });
            }

            for _cap in style_re.captures_iter(&chapter.content) {
                css_files.push(CSSInfo {
                    chapter_index: index,
                    chapter_title: chapter.title.clone(),
                    href: format!("inline-style-{}", index),
                    media_type: Some("inline".to_string()),
                });
            }
        }

        css_files
    }

    fn strip_html_tags(html: &str) -> String {
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
        let word_re = Regex::new(r"[a-zA-Z]+").unwrap();
        word_re
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

    pub fn rename_chapter(&mut self, index: usize, new_title: &str) -> Result<()> {
        if index >= self.chapters.len() {
            bail!("Chapter index out of bounds");
        }
        let old_title = self.chapters[index].title.clone();
        self.chapters[index].title = new_title.to_string();
        self.undo_stack
            .push(UndoAction::RenameChapter(index, old_title));
        self.redo_stack.clear();
        let chapter = &self.chapters[index];
        let file_path = self.temp_dir.join(&chapter.href);
        let content = chapter.content.clone();
        fs::write(&file_path, &content)?;
        Ok(())
    }

    pub fn delete_chapter(&mut self, index: usize) -> Result<()> {
        if index >= self.chapters.len() {
            bail!("Chapter index out of bounds");
        }
        let chapter = self.chapters.remove(index);
        self.undo_stack
            .push(UndoAction::Chapter(index, chapter.content.clone()));
        self.redo_stack.clear();
        let file_path = self.temp_dir.join(&chapter.href);
        fs::remove_file(&file_path)?;
        Ok(())
    }

    pub fn reorder_chapters(&mut self, from_index: usize, to_index: usize) -> Result<()> {
        if from_index >= self.chapters.len() || to_index >= self.chapters.len() {
            bail!("Chapter index out of bounds");
        }
        if from_index == to_index {
            return Ok(());
        }
        let chapter = self.chapters.remove(from_index);
        self.chapters.insert(to_index, chapter);
        self.undo_stack
            .push(UndoAction::ReorderChapters(from_index, to_index));
        self.redo_stack.clear();
        Ok(())
    }

    pub fn export_chapter(&self, index: usize, path: &Path) -> Result<()> {
        if index >= self.chapters.len() {
            bail!("Chapter index out of bounds");
        }
        let chapter = &self.chapters[index];
        let content = Self::strip_html_tags(&chapter.content);
        fs::write(path, &content)
            .with_context(|| format!("Failed to write chapter export to {}", path.display()))?;
        Ok(())
    }

    pub fn import_chapter(&mut self, index: usize, path: &Path) -> Result<()> {
        if index >= self.chapters.len() {
            bail!("Chapter index out of bounds");
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read chapter import from {}", path.display()))?;
        let old_content = self.chapters[index].content.clone();
        self.chapters[index].content = content;
        self.undo_stack
            .push(UndoAction::Chapter(index, old_content));
        self.redo_stack.clear();
        let chapter = &self.chapters[index];
        let file_path = self.temp_dir.join(&chapter.href);
        fs::write(&file_path, &chapter.content)?;
        Ok(())
    }

    fn validate_chapter_content(
        index: usize,
        chapter: &EpubChapter,
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
        chapter: &EpubChapter,
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
        chapter: &EpubChapter,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let href_re = Regex::new(r#"href="([^"]+)""#).unwrap();
        for mat in href_re.find_iter(content) {
            let href = &content[mat.start() + 6..mat.end() - 1];
            if href.is_empty() || href == "#" {
                issues.push(ValidationIssue {
                    chapter_index: index,
                    chapter_title: chapter.title.clone(),
                    issue_type: "Broken Link".to_string(),
                    message: format!("Empty or invalid href: {}", href),
                    location: Some(format!("Position {}", mat.start())),
                });
            }
        }
    }

    fn check_missing_images(
        content: &str,
        index: usize,
        chapter: &EpubChapter,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let img_re = Regex::new(r#"<img[^>]+src="([^"]+)""#).unwrap();
        for mat in img_re.find_iter(content) {
            let src = &content[mat.start() + mat.as_str().find("src=").unwrap() + 5..mat.end() - 1];
            if src.starts_with("http://") || src.starts_with("https://") {
                issues.push(ValidationIssue {
                    chapter_index: index,
                    chapter_title: chapter.title.clone(),
                    issue_type: "External Image".to_string(),
                    message: format!("External image reference: {}", src),
                    location: Some(format!("Position {}", mat.start())),
                });
            }
        }
    }

    /// Saves the EPUB file with all modifications applied.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Updating the OPF metadata fails
    /// * Creating the output EPUB file fails
    /// * Initializing the ZIP writer fails
    /// * Walking the temporary directory and adding files to the ZIP archive fails
    /// * Finalizing the ZIP archive fails
    pub fn save(&self) -> Result<()> {
        self.update_opf_metadata()?;
        let file =
            File::create(&self.epub_path).context("Failed to create EPUB file for saving")?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default().compression_method(CompressionMethod::DEFLATE);
        let mut buffer = Vec::new();
        self.walk_dir(&self.temp_dir, &mut zip, &options, &mut buffer)?;
        zip.finish()?;
        Ok(())
    }

    /// Updates the metadata in the EPUB's OPF file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The container.xml file is missing
    /// * Reading the container.xml file fails
    /// * The rootfile path cannot be found in container.xml
    /// * The OPF file is missing at the specified path
    /// * Reading the OPF file fails
    /// * Writing the updated OPF file fails
    fn update_opf_metadata(&self) -> Result<()> {
        let container_path = self.temp_dir.join("META-INF/container.xml");
        let container_content = fs::read_to_string(&container_path)?;
        let rootfile_regex =
            Regex::new(r#"rootfile[^"]*"?([^"]+)"?"#).expect("Invalid rootfile regex");

        if let Some(caps) = rootfile_regex.captures(&container_content) {
            let opf_path = caps
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or("OEBPS/content.opf");
            let opf_full_path = self.temp_dir.join(opf_path);

            if opf_full_path.exists() {
                let mut opf_content = fs::read_to_string(&opf_full_path)?;
                opf_content =
                    self.update_opf_field(&opf_content, "dc:title", &self.metadata.title)?;
                opf_content =
                    self.update_opf_field(&opf_content, "dc:creator", &self.metadata.author)?;
                opf_content =
                    self.update_opf_field(&opf_content, "dc:language", &self.metadata.language)?;
                opf_content = self.update_opf_field(
                    &opf_content,
                    "dc:identifier",
                    &self.metadata.identifier,
                )?;

                if let Some(ref publisher) = self.metadata.publisher {
                    opf_content = self.update_opf_field(&opf_content, "dc:publisher", publisher)?;
                }
                if let Some(ref date) = self.metadata.date {
                    opf_content = self.update_opf_field(&opf_content, "dc:date", date)?;
                }
                if let Some(ref description) = self.metadata.description {
                    opf_content =
                        self.update_opf_field(&opf_content, "dc:description", description)?;
                }
                fs::write(&opf_full_path, opf_content)?;
            }
        }
        Ok(())
    }

    /// Updates a specific field in the OPF XML content.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Creating the regex pattern for the field tag fails
    fn update_opf_field(&self, content: &str, tag: &str, value: &str) -> Result<String> {
        let regex_str = format!(r#"<{}[^>]*>[^<]*</{}>"#, tag, tag);
        let regex = Regex::new(&regex_str).map_err(|e| format_err!("Invalid regex: {}", e))?;
        let replace_str = format!("<{}>{}</{}>", tag, value, tag);
        Ok(regex.replace(content, &replace_str).to_string())
    }

    fn walk_dir<W: Write + io::Seek>(
        &self,
        dir: &Path,
        zip: &mut ZipWriter<W>,
        options: &FileOptions<'_, ()>,
        buffer: &mut Vec<u8>,
    ) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.strip_prefix(&self.temp_dir).unwrap_or(&path);
            if path.is_dir() {
                zip.add_directory(name.to_str().unwrap_or(""), *options)?;
                self.walk_dir(&path, zip, options, buffer)?;
            } else {
                let mut file = File::open(&path)?;
                buffer.clear();
                file.read_to_end(buffer)?;
                zip.start_file(name.to_str().unwrap_or(""), *options)?;
                zip.write_all(buffer)?;
            }
        }
        Ok(())
    }
}

impl Drop for EpubEditorCore {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}
