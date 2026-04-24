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
    /// Returns an error if creating the regex pattern for the field tag fails.
    fn update_opf_field(&self, content: &str, tag: &str, value: &str) -> Result<String> {
        let regex_str = format!(r#"<{}[^>]*>[^<]*</{}>"#, tag, tag);
        let regex = regex::Regex::new(&regex_str)
            .map_err(|e| anyhow::format_err!("Invalid regex: {}", e))?;
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
