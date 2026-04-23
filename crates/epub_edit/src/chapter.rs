//! Chapter management operations.

use anyhow::{bail, Context};
use regex::Regex;
use std::fs;
use std::path::Path;

use crate::editor::EpubEditorCore;
use crate::types::{CSSInfo, ImageInfo};

impl EpubEditorCore {
    /// Renames a chapter at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The chapter index to rename
    /// * `new_title` - The new title for the chapter
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds or writing the file fails.
    pub fn rename_chapter(&mut self, index: usize, new_title: &str) -> anyhow::Result<()> {
        if index >= self.chapters.len() {
            bail!("Chapter index out of bounds");
        }
        let old_title = self.chapters[index].title.clone();
        self.chapters[index].title = new_title.to_string();
        self.undo_stack
            .push(crate::types::UndoAction::RenameChapter(index, old_title));
        self.redo_stack.clear();
        let chapter = &self.chapters[index];
        let file_path = self.temp_dir.join(&chapter.href);
        let content = chapter.content.clone();
        fs::write(&file_path, &content)?;
        Ok(())
    }

    /// Deletes a chapter at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The chapter index to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds or removing the file fails.
    pub fn delete_chapter(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.chapters.len() {
            bail!("Chapter index out of bounds");
        }
        let chapter = self.chapters.remove(index);
        self.undo_stack.push(crate::types::UndoAction::Chapter(
            index,
            chapter.content.clone(),
        ));
        self.redo_stack.clear();
        let file_path = self.temp_dir.join(&chapter.href);
        fs::remove_file(&file_path)?;
        Ok(())
    }

    /// Reorders chapters by moving a chapter from one index to another.
    ///
    /// # Arguments
    ///
    /// * `from_index` - The source chapter index
    /// * `to_index` - The destination chapter index
    ///
    /// # Errors
    ///
    /// Returns an error if either index is out of bounds.
    pub fn reorder_chapters(&mut self, from_index: usize, to_index: usize) -> anyhow::Result<()> {
        if from_index >= self.chapters.len() || to_index >= self.chapters.len() {
            bail!("Chapter index out of bounds");
        }
        if from_index == to_index {
            return Ok(());
        }
        let chapter = self.chapters.remove(from_index);
        self.chapters.insert(to_index, chapter);
        self.undo_stack
            .push(crate::types::UndoAction::ReorderChapters(
                from_index, to_index,
            ));
        self.redo_stack.clear();
        Ok(())
    }

    /// Exports a chapter to a text file with HTML tags stripped.
    ///
    /// # Arguments
    ///
    /// * `index` - The chapter index to export
    /// * `path` - The destination file path
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds or writing the file fails.
    pub fn export_chapter(&self, index: usize, path: &Path) -> anyhow::Result<()> {
        if index >= self.chapters.len() {
            bail!("Chapter index out of bounds");
        }
        let chapter = &self.chapters[index];
        let content = crate::validation::ValidationHelpers::strip_html_tags(&chapter.content);
        fs::write(path, &content)
            .with_context(|| format!("Failed to write chapter export to {}", path.display()))?;
        Ok(())
    }

    /// Imports chapter content from a text file.
    ///
    /// # Arguments
    ///
    /// * `index` - The chapter index to import into
    /// * `path` - The source file path
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds, reading the file fails,
    /// or writing the chapter content fails.
    pub fn import_chapter(&mut self, index: usize, path: &Path) -> anyhow::Result<()> {
        if index >= self.chapters.len() {
            bail!("Chapter index out of bounds");
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read chapter import from {}", path.display()))?;
        let old_content = self.chapters[index].content.clone();
        self.chapters[index].content = content;
        self.undo_stack
            .push(crate::types::UndoAction::Chapter(index, old_content));
        self.redo_stack.clear();
        let chapter = &self.chapters[index];
        let file_path = self.temp_dir.join(&chapter.href);
        fs::write(&file_path, &chapter.content)?;
        Ok(())
    }

    /// Updates the table of contents file in the EPUB.
    ///
    /// # Errors
    ///
    /// Returns an error if writing the table of contents file fails.
    pub fn update_table_of_contents(&mut self) -> anyhow::Result<()> {
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

    /// Generates a table of contents in EPUB navigation format.
    ///
    /// # Returns
    ///
    /// A string containing the table of contents in XHTML navigation format.
    #[must_use]
    pub fn generate_table_of_contents(&self) -> String {
        let mut toc = String::new();
        toc.push_str("<nav epub:type=\"toc\">\n");
        toc.push_str("  <ol>\n");

        for chapter in self.chapters.iter() {
            toc.push_str(&format!(
                "    <li><a href=\"{}\">{}</a></li>\n",
                chapter.href, chapter.title
            ));
        }

        toc.push_str("  </ol>\n");
        toc.push_str("</nav>\n");
        toc
    }

    /// Lists all images found in the EPUB chapters.
    ///
    /// # Returns
    ///
    /// A vector of image information including source, alt text, and chapter location.
    #[must_use]
    pub fn list_images(&self) -> Vec<ImageInfo> {
        let mut images = Vec::new();
        let img_re = Regex::new(r#"<img[^>]+src=["']([^"']+)["'][^>]*>"#).unwrap();
        let alt_re = Regex::new(r#"alt=["']([^"']*)["']"#).unwrap();

        for (index, chapter) in self.chapters.iter().enumerate() {
            for cap in img_re.captures_iter(&chapter.content) {
                let src = cap
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let alt = alt_re
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

    /// Lists all CSS files and inline styles in the EPUB chapters.
    ///
    /// # Returns
    ///
    /// A vector of CSS information including href, media type, and chapter location.
    #[must_use]
    pub fn list_css(&self) -> Vec<CSSInfo> {
        let mut css_files = Vec::new();
        let link_re =
            Regex::new(r#"<link[^>]+rel=["']stylesheet["'][^>]*href=["']([^"']+)["'][^>]*>"#)
                .unwrap();
        let media_type_re = Regex::new(r#"media=["']([^"']*)["']"#).unwrap();
        let style_re = Regex::new(r#"<style[^>]*>(.*?)</style>"#).unwrap();

        for (index, chapter) in self.chapters.iter().enumerate() {
            for cap in link_re.captures_iter(&chapter.content) {
                let href = cap
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let media_type = media_type_re
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
}
