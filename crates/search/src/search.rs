//! Semantic search indexing using embeddings and SQLite
use anyhow::Result;
use plato_ai::embedding::CandleEmbedder;
use plato_ai::traits::VectorEmbedder;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Handles indexing and searching of document embeddings
#[derive(Clone)]
pub struct SearchIndexer {
    conn: Arc<Mutex<Connection>>,
    embedder: CandleEmbedder,
}

impl std::fmt::Debug for SearchIndexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchIndexer").finish()
    }
}

impl SearchIndexer {
    /// Initialize indexer with a SQLite database
    pub fn new(db_path: &str, embedder: CandleEmbedder) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS embeddings (
                id INTEGER PRIMARY KEY,
                doc_id TEXT NOT NULL,
                content TEXT NOT NULL,
                vector BLOB NOT NULL
            )",
            [],
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            embedder,
        })
    }

    /// Index a document chunk
    pub fn index_chunk(&self, doc_id: &str, content: &str) -> Result<()> {
        let vector = self.embedder.embed(content)?;
        let blob: Vec<u8> = vector.iter().flat_map(|f| f.to_le_bytes()).collect();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO embeddings (doc_id, content, vector) VALUES (?1, ?2, ?3)",
            (doc_id, content, blob),
        )?;
        Ok(())
    }

    /// Search for semantically similar documents
    pub fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> plato_error::PlatoResult<Vec<(String, f32, String)>> {
        let query_vec = self.embedder.embed(query)?;

        let conn = self
            .conn
            .lock()
            .map_err(|e| plato_error::PlatoError::Database(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT doc_id, content, vector FROM embeddings")
            .map_err(|e| plato_error::PlatoError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let doc_id: String = row.get(0)?;
                let content: String = row.get(1)?;
                let blob: Vec<u8> = row.get(2)?;
                let vector: Vec<f32> = blob
                    .chunks(4)
                    .map(|chunk| {
                        let bytes: [u8; 4] = chunk.try_into().unwrap_or([0; 4]);
                        f32::from_le_bytes(bytes)
                    })
                    .collect();
                Ok((doc_id, content, vector))
            })
            .map_err(|e| plato_error::PlatoError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let (doc_id, content, vector) =
                row.map_err(|e| plato_error::PlatoError::Database(e.to_string()))?;
            let sim = <CandleEmbedder as VectorEmbedder>::similarity(&query_vec, &vector);
            results.push((doc_id, sim, content));
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results.into_iter().take(limit).collect())
    }
}
