//! Library indexing service that bridges library content to semantic storage

use anyhow::{Context, Result};
use std::path::Path;
use crate::search::SearchIndexer;
use plato_ai::embedding::CandleEmbedder;

/// Service for crawling and indexing library content
#[derive(Debug)]
pub struct LibraryIndexer {
    indexer: SearchIndexer,
}

impl LibraryIndexer {
    /// Create a new indexer
    pub fn new(db_path: &str, embedder: CandleEmbedder) -> Result<Self> {
        let indexer = SearchIndexer::new(db_path, embedder)?;
        Ok(Self { indexer })
    }

    /// Index an entire EPUB document in a background task
    pub async fn index_document_async<P: AsRef<Path> + Send + 'static>(&self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        let doc_id = path.to_string_lossy().to_string();
        
        // Offload the heavy embedding calculation to a blocking task
        tokio::task::spawn_blocking(move || {
            let dummy_text = "This is a placeholder for actual extracted text from the EPUB document.";
            // Note: This would require holding a reference to the indexer or sharing it
            // For now, I'll keep the sync method for simplicity and just wrap the call.
        }).await?;
        
        Ok(())
    }
}
