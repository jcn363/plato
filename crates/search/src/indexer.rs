//! Library indexing service that bridges library content to semantic storage

use anyhow::{Context, Result};
use std::path::Path;
use crate::search::SearchIndexer;
use plato_ai::embedding::CandleEmbedder;

/// Service for crawling and indexing library content
pub struct LibraryIndexer {
    indexer: SearchIndexer,
}

impl LibraryIndexer {
    /// Create a new indexer
    pub fn new(db_path: &str, embedder: CandleEmbedder) -> Result<Self> {
        let indexer = SearchIndexer::new(db_path, embedder)?;
        Ok(Self { indexer })
    }

    /// Index an entire EPUB document
    pub fn index_document<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let doc_id = path.as_ref().to_string_lossy().to_string();
        
        let dummy_text = "This is a placeholder for actual extracted text from the EPUB document.";
        self.indexer.index_chunk(&doc_id, dummy_text)
            .with_context(|| format!("Failed to index document: {}", doc_id))?;
            
        Ok(())
    }
}
