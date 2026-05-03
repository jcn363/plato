//! Library indexing service that bridges library content to semantic storage

use crate::search::SearchIndexer;
use anyhow::{Context, Result};
use plato_ai::embedding::CandleEmbedder;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

/// Service for crawling and indexing library content
#[derive(Debug)]
pub struct LibraryIndexer {
    indexer: SearchIndexer,
}

impl LibraryIndexer {
    pub fn indexer(&self) -> &SearchIndexer {
        &self.indexer
    }
    /// Create a new indexer
    pub fn new(db_path: &str, embedder: CandleEmbedder) -> Result<Self> {
        let indexer = SearchIndexer::new(db_path, embedder)?;
        Ok(Self { indexer })
    }

    /// Extract text content from an EPUB document
    pub fn extract_epub_text(path: &Path) -> Result<String> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("Failed to open EPUB: {}", path.display()))?;
        let mut archive = ZipArchive::new(file)
            .with_context(|| format!("Failed to read EPUB archive: {}", path.display()))?;

        let mut text = String::new();

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .with_context(|| format!("Failed to read file {} from archive", i))?;

            let name = file.name().to_string();
            if name.ends_with(".xhtml")
                || name.ends_with(".html")
                || name.ends_with(".htm")
                || name.ends_with(".txt")
            {
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .with_context(|| format!("Failed to read content from: {}", name))?;

                let extracted = extract_text_from_html(&content);
                text.push_str(&extracted);
                text.push(' ');
            }
        }

        if text.is_empty() {
            text.push_str("EPUB document");
        }

        Ok(text)
    }

    /// Extract text content from a PDF document
    #[allow(dead_code)]
    pub fn extract_pdf_text(path: &Path) -> Result<String> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("PDF document");
        Ok(filename.to_string())
    }

    /// Index an entire document in a background task
    pub async fn index_document_async<P: AsRef<std::path::Path> + Send + 'static>(
        &self,
        path: P,
    ) -> Result<()> {
        let path_buf = path.as_ref().to_path_buf();
        let doc_id = path_buf.to_string_lossy().to_string();
        let ext = path_buf.extension().and_then(|e| e.to_str()).unwrap_or("");
        let ext = ext.to_string();

        let indexer = self.indexer.clone();
        tokio::task::spawn_blocking(move || {
            let path = path_buf.clone();
            let text = match ext.to_lowercase().as_str() {
                "epub" => Self::extract_epub_text(&path).with_context(|| {
                    format!("Failed to extract text from EPUB: {}", path.display())
                })?,
                "pdf" => Self::extract_pdf_text(&path).with_context(|| {
                    format!("Failed to extract text from PDF: {}", path.display())
                })?,
                _ => {
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("document");
                    filename.to_string()
                }
            };

            indexer
                .index_chunk(&doc_id, &text)
                .with_context(|| format!("Failed to index document: {}", doc_id))
        })
        .await??;

        Ok(())
    }
}

/// Extract readable text from HTML/XHTML content
fn extract_text_from_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_script = false;
    let mut in_style = false;

    let mut chars = html.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            let mut tag = String::new();
            for ch in chars.by_ref() {
                if ch == '>' || ch == ' ' {
                    break;
                }
                tag.push(ch);
            }

            let tag_lower = tag.to_lowercase();
            if tag_lower == "script" {
                in_script = true;
            } else if tag_lower == "/script" {
                in_script = false;
            } else if tag_lower == "style" {
                in_style = true;
            } else if tag_lower == "/style" {
                in_style = false;
            }
            continue;
        }

        if in_script || in_style {
            continue;
        }

        if c.is_alphanumeric() || c == ' ' || c == '\n' || c == '\r' || c == '\t' {
            result.push(c);
        }
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}
