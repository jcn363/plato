use anyhow::{format_err, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

lazy_static! {
    static ref TITLE_RE: Regex = Regex::new(r#"<dc:title[^>]*>([^<]+)</dc:title>"#).unwrap();
    static ref AUTHOR_RE: Regex = Regex::new(r#"<dc:creator[^>]*>([^<]+)</dc:creator>"#).unwrap();
    static ref LANGUAGE_RE: Regex =
        Regex::new(r#"<dc:language[^>]*>([^<]+)</dc:language>"#).unwrap();
    static ref IDENTIFIER_RE: Regex =
        Regex::new(r#"<dc:identifier[^>]*>([^<]+)</dc:identifier>"#).unwrap();
    static ref PUBLISHER_RE: Regex =
        Regex::new(r#"<dc:publisher[^>]*>([^<]+)</dc:publisher>"#).unwrap();
    static ref DATE_RE: Regex = Regex::new(r#"<dc:date[^>]*>([^<]+)</dc:date>"#).unwrap();
    static ref DESCRIPTION_RE: Regex =
        Regex::new(r#"<dc:description[^>]*>([^<]+)</dc:description>"#).unwrap();
    static ref ROOTFILE_RE: Regex = Regex::new(r#"rootfile[^"]*"?([^"]+)"?"#).unwrap();
    static ref ITEM_RE: Regex =
        Regex::new(r#"<item[^>]+href="([^"]+)"[^>]+id="([^"]+)"[^>]*>"#).unwrap();
    static ref SPINE_RE: Regex = Regex::new(r#"<itemref[^>]+idref="([^"]+)"[^>]*>"#).unwrap();
}

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
}

pub struct EpubEditorCore {
    pub epub_path: PathBuf,
    pub metadata: EpubMetadata,
    pub chapters: Vec<EpubChapter>,
    pub temp_dir: PathBuf,
    pub undo_stack: Vec<UndoAction>,
    pub redo_stack: Vec<UndoAction>,
}

impl EpubEditorCore {
    pub fn new(epub_path: &str) -> Result<Self> {
        let temp_dir = Self::create_temp_dir()?;
        let mut editor = Self {
            epub_path: PathBuf::from(epub_path),
            metadata: EpubMetadata::default(),
            chapters: Vec::new(),
            temp_dir,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        editor.extract()?;
        editor.parse_metadata()?;
        editor.parse_content()?;

        Ok(editor)
    }

    fn create_temp_dir() -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir().join(format!("epub_editor_{}", uuid::Uuid::new_v4()));
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;
        Ok(temp_dir)
    }

    fn extract(&self) -> Result<()> {
        let file = File::open(&self.epub_path).context("Failed to open EPUB file")?;
        let mut archive = ZipArchive::new(file).context("Failed to read ZIP archive")?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = self.temp_dir.join(file.name());

            if file.name().ends_with('/') {
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

    fn parse_metadata(&mut self) -> Result<()> {
        let container_path = self.temp_dir.join("META-INF/container.xml");
        if !container_path.exists() {
            return Err(format_err!("META-INF/container.xml not found"));
        }

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

    pub fn to_plato_metadata(&self) -> EpubMetadata {
        self.metadata.clone()
    }

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
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

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
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn search_in_chapter(&self, index: usize, query: &str) -> Vec<(usize, usize)> {
        if index >= self.chapters.len() || query.is_empty() {
            return Vec::new();
        }
        let content = &self.chapters[index].content;
        let mut matches = Vec::new();
        let mut start = 0;
        while let Some(pos) = content[start..].find(query) {
            let abs_pos = start + pos;
            matches.push((abs_pos, abs_pos + query.len()));
            start = abs_pos + 1;
        }
        matches
    }

    pub fn replace_in_chapter(
        &mut self,
        index: usize,
        search: &str,
        replace: &str,
    ) -> Result<usize> {
        if index >= self.chapters.len() || search.is_empty() {
            return Ok(0);
        }
        let old_content = self.chapters[index].content.clone();
        let count = old_content.matches(search).count();
        if count == 0 {
            return Ok(0);
        }
        let new_content = old_content.replace(search, replace);
        self.undo_stack
            .push(UndoAction::Chapter(index, old_content));
        self.redo_stack.clear();
        self.chapters[index].content = new_content.clone();
        let chapter = &self.chapters[index];
        let file_path = self.temp_dir.join(&chapter.href);
        fs::write(&file_path, &new_content)?;
        Ok(count)
    }

    pub fn replace_all_in_document(&mut self, search: &str, replace: &str) -> Result<usize> {
        if search.is_empty() {
            return Ok(0);
        }
        let mut total = 0;
        for i in 0..self.chapters.len() {
            let count = self.replace_in_chapter(i, search, replace)?;
            total += count;
        }
        Ok(total)
    }

    pub fn search_all_chapters(&self, query: &str) -> Vec<(usize, Vec<(usize, usize)>)> {
        if query.is_empty() {
            return Vec::new();
        }
        self.chapters
            .iter()
            .enumerate()
            .filter_map(|(i, _)| {
                let matches = self.search_in_chapter(i, query);
                if matches.is_empty() {
                    None
                } else {
                    Some((i, matches))
                }
            })
            .collect()
    }

    pub fn save(&self) -> Result<()> {
        self.update_opf_metadata()?;
        let file =
            File::create(&self.epub_path).context("Failed to create EPUB file for saving")?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
        let mut buffer = Vec::new();
        self.walk_dir(&self.temp_dir, &mut zip, &options, &mut buffer)?;
        zip.finish()?;
        Ok(())
    }

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

    fn update_opf_field(&self, content: &str, tag: &str, value: &str) -> Result<String> {
        let regex = Regex::new(&format!(r#"<{}[^>]*>[^<]*</{}>"#, tag, tag))
            .map_err(|e| format_err!("Invalid regex: {}", e))?;
        Ok(regex
            .replace(content, format!("<{}>{}</{}>", tag, value, tag))
            .to_string())
    }

    fn walk_dir<W: Write + io::Seek>(
        &self,
        dir: &Path,
        zip: &mut ZipWriter<W>,
        options: &FileOptions,
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
