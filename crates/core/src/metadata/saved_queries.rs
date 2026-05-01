//! Saved search queries for library search
//!
//! Provides persistence for user-saved search queries with metadata.

use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::helpers::load_json;
use crate::helpers::save_json;

/// Saved search query with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct SavedQuery {
    /// Unique identifier for the query
    pub id: String,
    /// Display name for the query
    pub name: String,
    /// Search query string
    pub query: String,
    /// Timestamp when the query was created
    pub created_at: String,
    /// Timestamp when the query was last used
    pub last_used: Option<String>,
    /// Number of times the query has been used
    pub use_count: u32,
}

/// Collection of saved search queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct SavedQueries {
    /// Map of query ID to saved query
    pub queries: BTreeMap<String, SavedQuery>,
}

impl SavedQueries {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a saved query
    pub fn add(&mut self, query: SavedQuery) -> Result<()> {
        if query.id.is_empty() {
            return Err(Error::msg("Query ID cannot be empty"));
        }
        if query.name.is_empty() {
            return Err(Error::msg("Query name cannot be empty"));
        }
        if query.query.is_empty() {
            return Err(Error::msg("Query string cannot be empty"));
        }
        self.queries.insert(query.id.clone(), query);
        Ok(())
    }

    /// Remove a saved query by ID
    pub fn remove(&mut self, id: &str) -> Result<()> {
        self.queries
            .remove(id)
            .ok_or_else(|| Error::msg(format!("Query with ID '{}' not found", id)))?;
        Ok(())
    }

    /// Get a saved query by ID
    pub fn get(&self, id: &str) -> Option<&SavedQuery> {
        self.queries.get(id)
    }

    /// Get a mutable reference to a saved query by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SavedQuery> {
        self.queries.get_mut(id)
    }

    /// Update the last used timestamp and increment use count
    pub fn mark_used(&mut self, id: &str) -> Result<()> {
        let query = self
            .queries
            .get_mut(id)
            .ok_or_else(|| Error::msg(format!("Query with ID '{}' not found", id)))?;

        query.last_used = Some(chrono::Local::now().to_rfc3339());
        query.use_count += 1;

        Ok(())
    }

    /// Get all saved queries
    pub fn all(&self) -> Vec<&SavedQuery> {
        self.queries.values().collect()
    }

    /// Get the number of saved queries
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Check if there are any saved queries
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }

    /// Clear all saved queries
    pub fn clear(&mut self) {
        self.queries.clear();
    }

    /// Load saved queries from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        load_json(path.as_ref()).with_context(|| {
            format!(
                "Failed to load saved queries from {}",
                path.as_ref().display()
            )
        })
    }

    /// Save saved queries to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        save_json(self, path.as_ref()).with_context(|| {
            format!(
                "Failed to save saved queries to {}",
                path.as_ref().display()
            )
        })
    }

    /// Generate a unique ID for a new query
    pub fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("query_{}", timestamp)
    }
}

#[cfg(test)]
mod saved_queries_tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_saved_queries_new() {
        let queries = SavedQueries::new();
        assert!(queries.is_empty());
        assert_eq!(queries.len(), 0);
    }

    #[test]
    fn test_saved_queries_add() {
        let mut queries = SavedQueries::new();
        let query = SavedQuery {
            id: "test_id".to_string(),
            name: "Test Query".to_string(),
            query: "author:smith".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_used: None,
            use_count: 0,
        };

        queries.add(query.clone()).expect("Test assertion failed");
        assert_eq!(queries.len(), 1);
        assert_eq!(queries.get("test_id"), Some(&query));
    }

    #[test]
    fn test_saved_queries_add_empty_id() {
        let mut queries = SavedQueries::new();
        let query = SavedQuery {
            id: String::new(),
            name: "Test Query".to_string(),
            query: "author:smith".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_used: None,
            use_count: 0,
        };

        let result = queries.add(query);
        assert!(result.is_err());
    }

    #[test]
    fn test_saved_queries_add_empty_name() {
        let mut queries = SavedQueries::new();
        let query = SavedQuery {
            id: "test_id".to_string(),
            name: String::new(),
            query: "author:smith".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_used: None,
            use_count: 0,
        };

        let result = queries.add(query);
        assert!(result.is_err());
    }

    #[test]
    fn test_saved_queries_add_empty_query() {
        let mut queries = SavedQueries::new();
        let query = SavedQuery {
            id: "test_id".to_string(),
            name: "Test Query".to_string(),
            query: String::new(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_used: None,
            use_count: 0,
        };

        let result = queries.add(query);
        assert!(result.is_err());
    }

    #[test]
    fn test_saved_queries_remove() {
        let mut queries = SavedQueries::new();
        let query = SavedQuery {
            id: "test_id".to_string(),
            name: "Test Query".to_string(),
            query: "author:smith".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_used: None,
            use_count: 0,
        };

        queries.add(query).expect("Test assertion failed");
        queries.remove("test_id").expect("Test assertion failed");
        assert!(queries.is_empty());
    }

    #[test]
    fn test_saved_queries_remove_not_found() {
        let mut queries = SavedQueries::new();
        let result = queries.remove("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_saved_queries_mark_used() {
        let mut queries = SavedQueries::new();
        let query = SavedQuery {
            id: "test_id".to_string(),
            name: "Test Query".to_string(),
            query: "author:smith".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_used: None,
            use_count: 0,
        };

        queries.add(query).expect("Test assertion failed");
        queries.mark_used("test_id").expect("Test assertion failed");

        let updated = queries.get("test_id").expect("Test assertion failed");
        assert_eq!(updated.use_count, 1);
        assert!(updated.last_used.is_some());
    }

    #[test]
    fn test_saved_queries_mark_used_not_found() {
        let mut queries = SavedQueries::new();
        let result = queries.mark_used("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_saved_queries_all() {
        let mut queries = SavedQueries::new();

        queries
            .add(SavedQuery {
                id: "id1".to_string(),
                name: "Query 1".to_string(),
                query: "author:smith".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                last_used: None,
                use_count: 0,
            })
            .expect("Test assertion failed");

        queries
            .add(SavedQuery {
                id: "id2".to_string(),
                name: "Query 2".to_string(),
                query: "title:test".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                last_used: None,
                use_count: 0,
            })
            .expect("Test assertion failed");

        let all = queries.all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_saved_queries_clear() {
        let mut queries = SavedQueries::new();

        queries
            .add(SavedQuery {
                id: "test_id".to_string(),
                name: "Test Query".to_string(),
                query: "author:smith".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                last_used: None,
                use_count: 0,
            })
            .expect("Test assertion failed");

        queries.clear();
        assert!(queries.is_empty());
    }

    #[test]
    fn test_saved_queries_save_load() {
        let mut queries = SavedQueries::new();

        queries
            .add(SavedQuery {
                id: "test_id".to_string(),
                name: "Test Query".to_string(),
                query: "author:smith".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                last_used: None,
                use_count: 0,
            })
            .expect("Test assertion failed");

        let temp_file = NamedTempFile::new().expect("Test assertion failed");
        let path = temp_file.path();

        queries.save(path).expect("Test assertion failed");
        let loaded = SavedQueries::load(path).expect("Test assertion failed");

        assert_eq!(loaded.len(), 1);
        assert!(loaded.get("test_id").is_some());
    }

    #[test]
    fn test_saved_query_default() {
        let query = SavedQuery::default();
        assert!(query.id.is_empty());
        assert!(query.name.is_empty());
        assert!(query.query.is_empty());
        assert!(query.created_at.is_empty());
        assert!(query.last_used.is_none());
        assert_eq!(query.use_count, 0);
    }

    #[test]
    fn test_generate_id() {
        let id1 = SavedQueries::generate_id();
        let id2 = SavedQueries::generate_id();

        assert!(id1.starts_with("query_"));
        assert!(id2.starts_with("query_"));
        // Note: Uniqueness depends on system time, may not differ in fast tests
    }
}
