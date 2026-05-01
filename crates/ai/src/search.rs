//! Semantic search indexing using embeddings and SQLite

use anyhow::Result;
use rusqlite::Connection;
use crate::embedding::CandleEmbedder;
use crate::traits::VectorEmbedder;

/// Handles indexing and searching of document embeddings
pub struct SearchIndexer {
    conn: Connection,
    embedder: CandleEmbedder,
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
        Ok(Self { conn, embedder })
    }

    /// Index a document chunk
    pub fn index_chunk(&self, doc_id: &str, content: &str) -> Result<()> {
        let vector = self.embedder.embed(content)?;
        let blob: Vec<u8> = vector.iter().flat_map(|f| f.to_le_bytes()).collect();
        self.conn.execute(
            "INSERT INTO embeddings (doc_id, content, vector) VALUES (?1, ?2, ?3)",
            (doc_id, content, blob),
        )?;
        Ok(())
    }

    /// Search for semantically similar documents
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<(String, f32)>> {
        let query_vec = self.embedder.embed(query)?;
        
        let mut stmt = self.conn.prepare("SELECT doc_id, vector FROM embeddings")?;
        let rows = stmt.query_map([], |row| {
            let doc_id: String = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            let vector: Vec<f32> = blob.chunks(4)
                .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
                .collect();
            Ok((doc_id, vector))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (doc_id, vector) = row?;
            let sim = <CandleEmbedder as VectorEmbedder>::similarity(&query_vec, &vector);
            results.push((doc_id, sim));
        }
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(results.into_iter().take(limit).collect())
    }
}
