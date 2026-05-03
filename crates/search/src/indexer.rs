//! Library indexing service that bridges library content to semantic storage

use crate::search::SearchIndexer;
use anyhow::{Context, Result};
use plato_ai::embedding::CandleEmbedder;
use std::path::Path;

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

    /// Index an entire EPUB document in a background task
    pub async fn index_document_async<P: AsRef<Path> + Send + 'static>(
        &self,
        path: P,
    ) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        let doc_id = path.to_string_lossy().to_string();

        // Clone the indexer reference to move into the background task
        // We'll wrap SearchIndexer in an Arc for shared access if necessary,
        // but for now, we'll index a copy.

        // Offload the heavy embedding calculation to a blocking task
        let indexer = self.indexer.clone();
        tokio::task::spawn_blocking(move || {
            let dummy_text =
                "This is a placeholder for actual extracted text from the EPUB document.";
            indexer
                .index_chunk(&doc_id, dummy_text)
                .with_context(|| format!("Failed to index document: {}", doc_id))
        })
        .await??;

        Ok(())
    }
}
