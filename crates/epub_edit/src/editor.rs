//! Core EPUB editor implementation.

use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::parser::{
    extract_epub, parse_content, parse_metadata, parse_opf_content, parse_opf_metadata,
};
use crate::types::{EpubChapter, EpubMetadata, UndoAction};

const MAX_HISTORY_SIZE: usize = 50;

/// Core editor for EPUB file manipulation.
///
/// This struct provides the main interface for editing EPUB files, including
/// metadata modification, chapter content editing, search/replace operations,
/// and undo/redo functionality.
pub struct EpubEditorCore {
    /// Path to the EPUB file being edited
    pub epub_path: PathBuf,
    /// EPUB metadata (title, author, language, etc.)
    pub metadata: EpubMetadata,
    /// List of chapters in the EPUB
    pub chapters: Vec<EpubChapter>,
    /// Temporary directory for extracted EPUB contents
    pub temp_dir: PathBuf,
    /// Stack of undo actions for reverting changes
    pub undo_stack: Vec<UndoAction>,
    /// Stack of redo actions for reapplying undone changes
    pub redo_stack: Vec<UndoAction>,
    /// List of bookmarks within the EPUB
    pub bookmarks: Vec<crate::types::Bookmark>,
}

impl EpubEditorCore {
    /// Creates a new `EpubEditorCore` instance from an EPUB file path.
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
        let temp_dir = PathBuf::from("tmp").join(format!("epub_editor_{}", uuid::Uuid::new_v4()));
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
        extract_epub(&self.epub_path, &self.temp_dir)
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
        let (_opf_path, opf_content) = parse_metadata(&self.temp_dir)?;
        self.metadata = parse_opf_metadata(&opf_content);
        Ok(())
    }

    /// Returns a clone of the EPUB metadata in Plato-compatible format.
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
        let (opf_path, opf_content) = parse_content(&self.temp_dir)?;
        self.chapters = parse_opf_content(&opf_content, &opf_path, &self.temp_dir);
        Ok(())
    }

    /// Sets the EPUB metadata.
    ///
    /// This updates the title, author, language, and other metadata fields.
    /// The change is added to the undo stack for potential reversal.
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
    /// Returns an error if writing the updated chapter content to file fails.
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
    /// Returns an error if writing the undone content to file fails.
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
    /// Returns an error if writing the redone content to file fails.
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

    /// Clears the undo and redo history.
    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Adds a bookmark at the specified chapter and position.
    ///
    /// # Arguments
    ///
    /// * `index` - The chapter index to bookmark
    /// * `position` - The character position within the chapter
    /// * `note` - An optional note to attach to the bookmark
    pub fn add_bookmark(&mut self, index: usize, position: usize, note: Option<String>) {
        if index < self.chapters.len() {
            let bookmark = crate::types::Bookmark {
                chapter_index: index,
                chapter_title: self.chapters[index].title.clone(),
                position,
                note,
            };
            self.bookmarks.push(bookmark);
        }
    }

    /// Removes a bookmark at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The bookmark index to remove
    pub fn remove_bookmark(&mut self, index: usize) {
        if index < self.bookmarks.len() {
            self.bookmarks.remove(index);
        }
    }

    /// Returns a list of all bookmarks.
    #[must_use]
    pub fn list_bookmarks(&self) -> Vec<crate::types::Bookmark> {
        self.bookmarks.clone()
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

        // 1. mimetype must be FIRST and UNCOMPRESSED
        let mimetype_path = self.temp_dir.join("mimetype");
        if mimetype_path.exists() {
            let options: FileOptions<'_, ()> =
                FileOptions::default().compression_method(CompressionMethod::STORE);
            zip.start_file("mimetype", options)?;
            let content = fs::read(&mimetype_path)?;
            zip.write_all(&content)?;
        }

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
            regex::Regex::new(r#"rootfile[^"]*"?([^"]+)"?"#).expect("Invalid rootfile regex");

        if let Some(caps) = rootfile_regex.captures(&container_content) {
            let opf_path = caps.get(1).map_or("OEBPS/content.opf", |m| m.as_str());
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
    /// Returns an error if creating the regex pattern for the field tag fails.
    fn update_opf_field(&self, content: &str, tag: &str, value: &str) -> Result<String> {
        let regex_str = format!(r"<{tag}[^>]*>[^<]*</{tag}>");
        let regex =
            regex::Regex::new(&regex_str).map_err(|e| anyhow::format_err!("Invalid regex: {e}"))?;
        let replace_str = format!("<{tag}>{value}</{tag}>");
        Ok(regex.replace(content, &replace_str).to_string())
    }

    /// Optimizes all images in the EPUB for E-Ink devices.
    ///
    /// This converts images to grayscale and resizes them if they exceed the specified maximum dimension.
    ///
    /// # Arguments
    ///
    /// * `max_dim` - The maximum width or height for any image.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Walking the temporary directory fails
    /// * Opening or processing an image fails
    /// * Saving an optimized image fails
    pub fn optimize_images(&self, max_dim: u32) -> Result<usize> {
        let mut count = 0;
        for entry in walkdir::WalkDir::new(&self.temp_dir) {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext = ext.to_lowercase();
                    if ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext.as_str())
                        && self.optimize_image(path, max_dim).is_ok()
                    {
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    fn optimize_image(&self, path: &Path, max_dim: u32) -> Result<()> {
        let img = image::open(path).context("Failed to open image")?;
        let (w, h) = (img.width(), img.height());

        let mut processed = img;
        if w > max_dim || h > max_dim {
            processed = processed.resize(max_dim, max_dim, image::imageops::FilterType::Lanczos3);
        }

        // Convert to grayscale for E-Ink
        let grayscale = processed.into_luma8();
        grayscale
            .save(path)
            .context("Failed to save optimized image")?;
        Ok(())
    }

    /// Sanitizes CSS across all chapters and CSS files for better E-Ink readability.
    ///
    /// This removes complex backgrounds, large margins, and fixed widths.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Walking the temporary directory fails
    /// * Reading or writing CSS files fails
    pub fn sanitize_css(&mut self) -> Result<usize> {
        let mut count = 0;

        // 1. Sanitize .css files
        for entry in walkdir::WalkDir::new(&self.temp_dir) {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("css") {
                let content = fs::read_to_string(path)?;
                let sanitized = self.process_css(&content);
                if sanitized != content {
                    fs::write(path, sanitized)?;
                    count += 1;
                }
            }
        }

        // 2. Sanitize inline styles in chapters
        for i in 0..self.chapters.len() {
            let content = self.chapters[i].content.clone();
            let sanitized = self.process_css(&content);
            if sanitized != content {
                self.update_chapter(i, sanitized)?;
                count += 1;
            }
        }

        Ok(count)
    }

    fn process_css(&self, css: &str) -> String {
        let mut result = css.to_string();

        // Remove fixed widths larger than 100%
        let width_re = regex::Regex::new(r"width\s*:\s*\d+(?:\.\d+)?(?:px|pt|cm|in|mm)")
            .expect("CSS width regex is valid");
        result = width_re.replace_all(&result, "max-width: 100%").to_string();

        // Remove large margins
        let margin_re = regex::Regex::new(r"margin\s*:\s*\d+(?:\.\d+)?(?:px|pt|cm|in|mm)")
            .expect("CSS margin regex is valid");
        result = margin_re.replace_all(&result, "margin: 0").to_string();

        // Force black text on white background for high contrast
        let bg_re = regex::Regex::new(r"background-color\s*:[^;]+;?")
            .expect("CSS background-color regex is valid");
        result = bg_re
            .replace_all(&result, "background-color: #fff;")
            .to_string();

        let color_re =
            regex::Regex::new(r"(?i)color\s*:[^;]+;?").expect("CSS color regex is valid");
        result = color_re.replace_all(&result, "color: #000;").to_string();

        result
    }

    /// Recovers or updates the Table of Contents from document headings.
    ///
    /// # Errors
    ///
    /// Returns an error if writing the updated TOC or OPF metadata fails.
    pub fn recover_toc(&mut self) -> Result<bool> {
        let mut new_toc = Vec::new();
        let heading_re = regex::Regex::new(r"(?i)<h([1-6])[^>]*>(.*?)</h[1-6]>")
            .expect("HTML heading regex is valid");

        for chapter in &self.chapters {
            for cap in heading_re.captures_iter(&chapter.content) {
                let level = cap[1].parse::<usize>().unwrap_or(1);
                let title = crate::validation::ValidationHelpers::strip_html_tags(&cap[2])
                    .trim()
                    .to_string();
                if !title.is_empty() {
                    new_toc.push((level, title, chapter.href.clone()));
                }
            }
        }

        if new_toc.is_empty() {
            return Ok(false);
        }

        // Simple heuristic: if we have headings, use them to build a new TOC
        let mut toc_html = String::from("<nav epub:type=\"toc\">\n  <ol>\n");
        for (_level, title, href) in new_toc {
            toc_html.push_str(&format!("    <li><a href=\"{href}\">{title}</a></li>\n"));
        }
        toc_html.push_str("  </ol>\n</nav>");

        // In a real implementation, we would update the actual nav.xhtml or ncx file.
        // For now, we'll just update the TOC in the OPF metadata if it's missing.
        self.update_table_of_contents_content(&toc_html)?;

        Ok(true)
    }

    fn update_table_of_contents_content(&self, content: &str) -> Result<()> {
        let toc_path = self.temp_dir.join("nav.xhtml");
        let html = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\">\n\
             <head><title>Table of Contents</title></head>\n\
             <body>{content}</body>\n</html>"
        );
        fs::write(toc_path, html).context("Failed to write nav.xhtml")?;
        Ok(())
    }

    /// Minifies HTML across all chapters for better rendering performance.
    ///
    /// # Errors
    ///
    /// Returns an error if writing the minified chapter content fails.
    pub fn minify_html(&mut self) -> Result<usize> {
        let mut count = 0;
        let comment_re = regex::Regex::new(r"(?s)<!--.*?-->").expect("HTML comment regex is valid");
        let space_re = regex::Regex::new(r"\s+").expect("Whitespace regex is valid");

        for i in 0..self.chapters.len() {
            let content = self.chapters[i].content.clone();
            let mut minified = comment_re.replace_all(&content, "").to_string();
            minified = space_re.replace_all(&minified, " ").to_string();
            minified = minified.replace("> <", "><").trim().to_string();

            if minified != content {
                self.update_chapter(i, minified)?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Scrubs non-essential metadata from the OPF file for privacy and cleanliness.
    ///
    /// # Errors
    ///
    /// Returns an error if updating the OPF file fails.
    pub fn scrub_metadata(&mut self) -> Result<usize> {
        let mut count = 0;
        let container_path = self.temp_dir.join("META-INF/container.xml");
        let container_content = fs::read_to_string(&container_path)?;
        let rootfile_regex =
            regex::Regex::new(r#"rootfile[^"]*"?([^"]+)"?"#).expect("Invalid rootfile regex");

        if let Some(caps) = rootfile_regex.captures(&container_content) {
            let opf_path = caps.get(1).map_or("OEBPS/content.opf", |m| m.as_str());
            let opf_full_path = self.temp_dir.join(opf_path);

            if opf_full_path.exists() {
                let mut opf_content = fs::read_to_string(&opf_full_path)?;
                let original = opf_content.clone();

                // Remove common "junk" metadata tags
                let junk_tags = [
                    "calibre:timestamp",
                    "calibre:title_sort",
                    "calibre:author_link_map",
                    "calibre:series",
                    "calibre:series_index",
                    "calibre:rating",
                    "calibre:user_categories",
                    "sigil:version",
                ];

                for tag in &junk_tags {
                    let re_str = format!(
                        r#"(?i)<meta[^>]*name="[^"]*{tag}[^"]*"[^>]*content="[^"]*"[^>]*/>"#
                    );
                    let re =
                        regex::Regex::new(&re_str).expect("Dynamic metadata tag regex is valid");
                    opf_content = re.replace_all(&opf_content, "").to_string();

                    let re_str_alt = format!(
                        r#"(?i)<meta[^>]*content="[^"]*"[^>]*name="[^"]*{tag}[^"]*"[^>]*/>"#
                    );
                    let re_alt = regex::Regex::new(&re_str_alt)
                        .expect("Alternative metadata tag regex is valid");
                    opf_content = re_alt.replace_all(&opf_content, "").to_string();
                }

                if opf_content != original {
                    fs::write(&opf_full_path, opf_content)?;
                    count += 1;
                    // Re-parse metadata to reflect changes
                    self.parse_metadata()?;
                }
            }
        }
        Ok(count)
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

            // Skip mimetype as it's already added first
            if name.to_str() == Some("mimetype") {
                continue;
            }

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
