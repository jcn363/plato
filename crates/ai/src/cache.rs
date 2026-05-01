//! SQLite-based caching for AI responses

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Error};
use rusqlite::{params, Connection};

use crate::AiContext;
use crate::AiResponse;

/// Cache for AI responses to avoid re-computation
pub struct AiCache {
    conn: Connection,
}

impl AiCache {
    /// Open or create the cache database
    pub fn open(path: &str) -> Result<Self, Error> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open AI cache database at {path}"))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_cache (
                id INTEGER PRIMARY KEY,
                prompt_hash TEXT NOT NULL,
                document_path TEXT NOT NULL,
                page INTEGER NOT NULL,
                response TEXT NOT NULL,
                model TEXT NOT NULL,
                provider TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )
        .context("Failed to create ai_cache table")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_prompt_doc ON ai_cache(prompt_hash, document_path, page)",
            [],
        )
        .context("Failed to create index on ai_cache")?;

        Ok(Self { conn })
    }

    /// Get cached response (if exists and not stale)
    pub fn get(
        &self,
        prompt: &str,
        context: &AiContext,
        max_age_seconds: i64,
    ) -> Result<Option<AiResponse>, Error> {
        let prompt_hash = Self::hash_prompt(prompt);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let min_timestamp = now - max_age_seconds;

        // Debug: print what we're looking for
        println!(
            "Looking for: hash={}, path={}, page={}, min_timestamp={}",
            prompt_hash, context.document_path, context.current_page, min_timestamp
        );

        let mut stmt = self.conn.prepare(
            "SELECT response, model, provider, timestamp
             FROM ai_cache
             WHERE prompt_hash = ?1 AND document_path = ?2 AND page = ?3 AND created_at > ?4
             ORDER BY created_at DESC
             LIMIT 1",
        )?;

        let result: Result<(String, String, String, i64), _> = stmt.query_row(
            params![
                prompt_hash,
                context.document_path,
                context.current_page as i64,
                min_timestamp
            ],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        );

        match result {
            Ok((response_text, model, provider, timestamp)) => Ok(Some(AiResponse {
                content: response_text,
                model,
                provider,
                timestamp,
                cached: true,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Store response in cache
    pub fn put(
        &self,
        prompt: &str,
        context: &AiContext,
        response: &AiResponse,
    ) -> Result<(), Error> {
        let prompt_hash = Self::hash_prompt(prompt);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT INTO ai_cache (prompt_hash, document_path, page, response, model, provider, timestamp, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                prompt_hash,
                context.document_path,
                context.current_page as i64,
                response.content,
                response.model,
                response.provider,
                response.timestamp,
                now,
            ],
        )?;

        Ok(())
    }

    /// Clear expired entries
    pub fn cleanup(&self, max_age_seconds: i64) -> Result<usize, Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let min_timestamp = now - max_age_seconds;

        let count = self.conn.execute(
            "DELETE FROM ai_cache WHERE timestamp < ?1",
            params![min_timestamp],
        )?;

        Ok(count)
    }

    fn hash_prompt(prompt: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        prompt.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AiContext;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cache_put_and_get() {
        let temp_file = NamedTempFile::new().expect("Test assertion failed");
        let cache = AiCache::open(temp_file.path().to_str().expect("Test assertion failed"))
            .expect("Test assertion failed");

        let context = AiContext::new("/test.epub".into(), 5, 20);
        let response = AiResponse {
            content: "Test summary".into(),
            model: "phi3:mini".into(),
            provider: "ollama".into(),
            timestamp: 12345,
            cached: false,
        };

        cache
            .put("Summarize chapter 1", &context, &response)
            .expect("Test assertion failed");

        // Debug: check what's in the cache
        let check: Result<(String, String), _> = cache.conn.query_row(
            "SELECT prompt_hash, document_path FROM ai_cache",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );
        println!("Cache contents: {:?}", check);

        let cached = cache
            .get("Summarize chapter 1", &context, 3600)
            .expect("Test assertion failed");
        assert!(cached.is_some(), "Expected cache to return Some, got None");
        let cached = cached.expect("Test assertion failed");
        assert_eq!(cached.content, "Test summary", "Cached content mismatch");
    }
}
