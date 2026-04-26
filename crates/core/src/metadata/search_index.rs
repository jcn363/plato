//! Full-text search index for document content
//!
//! Provides efficient full-text search across PDF and EPUB documents
//! using an inverted index structure (word -> document IDs).

use anyhow::{Error, Result};
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::{Arc, Mutex};

use crate::document::{Document, Location};
use crate::metadata::Info;

/// Document ID type for indexing
pub type DocId = String;

/// Inverted index mapping words to document IDs
type InvertedIndex = FxHashMap<String, FxHashSet<DocId>>;

/// Full-text search index
#[derive(Debug, Clone)]
pub struct SearchIndex {
    /// Inverted index: word -> set of document IDs containing that word
    index: Arc<Mutex<InvertedIndex>>,
    /// Document ID -> document path mapping
    doc_paths: Arc<Mutex<FxHashMap<DocId, String>>>,
}

impl SearchIndex {
    /// Create a new empty search index
    pub fn new() -> Self {
        Self {
            index: Arc::new(Mutex::new(FxHashMap::default())),
            doc_paths: Arc::new(Mutex::new(FxHashMap::default())),
        }
    }

    /// Index a document by extracting its text content
    ///
    /// # Arguments
    /// * `doc_id` - Unique identifier for the document (typically file path)
    /// * `info` - Document metadata
    /// * `document` - The document to extract text from
    pub fn index_document<T: Document>(
        &self,
        doc_id: DocId,
        info: &Info,
        document: &mut T,
    ) -> Result<()> {
        // Validate inputs
        if doc_id.is_empty() {
            return Err(Error::msg("Document ID cannot be empty"));
        }

        // Extract text from document
        let text = self.extract_text(document, info)?;

        if text.is_empty() {
            return Ok(());
        }

        // Tokenize and index
        let words = self.tokenize(&text);
        let mut index = self.index.lock().expect("Index lock poisoned");
        let mut doc_paths = self.doc_paths.lock().expect("Doc paths lock poisoned");

        // Add document path mapping
        doc_paths.insert(
            doc_id.clone(),
            info.file.path.to_string_lossy().into_owned(),
        );

        // Add each word to the inverted index
        for word in words {
            if word.len() < 2 {
                continue; // Skip single-character words
            }
            index
                .entry(word.to_lowercase())
                .or_insert_with(FxHashSet::default)
                .insert(doc_id.clone());
        }

        Ok(())
    }

    /// Remove a document from the index
    pub fn remove_document(&self, doc_id: &DocId) -> Result<()> {
        let mut index = self.index.lock().expect("Index lock poisoned");
        let mut doc_paths = self.doc_paths.lock().expect("Doc paths lock poisoned");

        // Remove from path mapping
        doc_paths.remove(doc_id);

        // Remove from inverted index
        for word_set in index.values_mut() {
            word_set.remove(doc_id);
        }

        // Clean up empty word entries
        index.retain(|_, set| !set.is_empty());

        Ok(())
    }

    /// Search for documents containing the given query
    ///
    /// # Arguments
    /// * `query` - Search query string
    ///
    /// # Returns
    /// Set of document IDs matching the query
    pub fn search(&self, query: &str) -> Result<FxHashSet<DocId>> {
        if query.is_empty() {
            return Ok(FxHashSet::default());
        }

        let words = self.tokenize(query);
        let index = self.index.lock().expect("Index lock poisoned");

        if words.is_empty() {
            return Ok(FxHashSet::default());
        }

        // Find documents containing all query words (AND semantics)
        let mut result: Option<FxHashSet<DocId>> = None;

        for word in words {
            let word_lower = word.to_lowercase();
            if let Some(doc_ids) = index.get(&word_lower) {
                if let Some(ref mut current) = result {
                    // Intersect with current results
                    current.retain(|id| doc_ids.contains(id));
                } else {
                    // First word - initialize result
                    result = Some(doc_ids.clone());
                }
            } else {
                // Word not found in any document - return empty
                return Ok(FxHashSet::default());
            }
        }

        Ok(result.unwrap_or_default())
    }

    /// Get the number of indexed documents
    pub fn document_count(&self) -> usize {
        let doc_paths = self.doc_paths.lock().expect("Doc paths lock poisoned");
        doc_paths.len()
    }

    /// Get the number of unique words in the index
    pub fn word_count(&self) -> usize {
        let index = self.index.lock().expect("Index lock poisoned");
        index.len()
    }

    /// Clear the entire index
    pub fn clear(&self) {
        let mut index = self.index.lock().expect("Index lock poisoned");
        let mut doc_paths = self.doc_paths.lock().expect("Doc paths lock poisoned");
        index.clear();
        doc_paths.clear();
    }

    /// Extract text content from a document
    fn extract_text<T: Document>(&self, document: &mut T, info: &Info) -> Result<String> {
        let mut text = String::with_capacity(10_000);

        // Extract text based on document type
        match info.file.kind.as_str() {
            "pdf" => {
                // Extract text from PDF using PDFPurr
                if let Some((bounded_texts, _)) = document.words(Location::Exact(0)) {
                    for bt in bounded_texts {
                        text.push_str(&bt.text);
                        text.push(' ');
                    }
                }
            }
            "epub" => {
                // Extract text from EPUB
                if let Some((bounded_texts, _)) = document.words(Location::Exact(0)) {
                    for bt in bounded_texts {
                        text.push_str(&bt.text);
                        text.push(' ');
                    }
                }
            }
            _ => {
                // For other formats, use metadata as fallback
                text.push_str(&info.title);
                text.push(' ');
                text.push_str(&info.author);
                text.push(' ');
                text.push_str(&info.subtitle);
            }
        }

        Ok(text)
    }

    /// Tokenize text into words
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|word| {
                // Remove punctuation and convert to lowercase
                word.chars()
                    .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod search_index_tests {
    use super::*;

    #[test]
    fn test_search_index_new() {
        let index = SearchIndex::new();
        assert_eq!(index.document_count(), 0);
        assert_eq!(index.word_count(), 0);
    }

    #[test]
    fn test_search_index_clear() {
        let index = SearchIndex::new();
        let mut doc_paths = index.doc_paths.lock().expect("Lock poisoned");
        doc_paths.insert("doc1".to_string(), "/path/to/doc1".to_string());
        drop(doc_paths);

        index.clear();
        assert_eq!(index.document_count(), 0);
        assert_eq!(index.word_count(), 0);
    }

    #[test]
    fn test_tokenize() {
        let index = SearchIndex::new();
        let text = "Hello, World! This is a test.";
        let words = index.tokenize(text);
        assert_eq!(words.len(), 6);
        assert!(words.contains(&"hello".to_string()));
        assert!(words.contains(&"world".to_string()));
        assert!(words.contains(&"this".to_string()));
        assert!(words.contains(&"is".to_string()));
        assert!(words.contains(&"a".to_string()));
        assert!(words.contains(&"test".to_string()));
    }

    #[test]
    fn test_tokenize_empty() {
        let index = SearchIndex::new();
        let words = index.tokenize("");
        assert!(words.is_empty());
    }

    #[test]
    fn test_search_empty_query() {
        let index = SearchIndex::new();
        let result = index.search("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_remove_document() {
        let index = SearchIndex::new();
        let doc_id = "doc1".to_string();

        let mut doc_paths = index.doc_paths.lock().expect("Lock poisoned");
        doc_paths.insert(doc_id.clone(), "/path/to/doc1".to_string());
        drop(doc_paths);

        index.remove_document(&doc_id).unwrap();
        assert_eq!(index.document_count(), 0);
    }
}
