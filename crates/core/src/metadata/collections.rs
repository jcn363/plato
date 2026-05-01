//! Collection management for organizing books
//!
//! Provides functionality for creating, managing, and querying collections.

use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::helpers::load_json;
use crate::helpers::save_json;

/// Collection metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Collection {
    /// Unique identifier for the collection
    pub id: String,
    /// Display name
    pub name: String,
    /// Parent collection ID (for nested collections)
    pub parent_id: Option<String>,
    /// Color for collection icon (hex color code)
    pub color: Option<String>,
    /// Icon identifier
    pub icon: Option<String>,
    /// Smart collection rules (if this is a smart collection)
    pub rules: Option<SmartCollectionRules>,
    /// Timestamp when created
    pub created_at: String,
    /// Timestamp when last modified
    pub modified_at: String,
}

/// Smart collection rules for auto-populating collections
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct SmartCollectionRules {
    /// Filter by reading status
    pub reading_status: Option<String>,
    /// Filter by author (regex pattern)
    pub author: Option<String>,
    /// Filter by series (regex pattern)
    pub series: Option<String>,
    /// Filter by tags (must contain all specified tags)
    pub tags: Vec<String>,
    /// Filter by categories (must contain all specified categories)
    pub categories: Vec<String>,
    /// Minimum file size in bytes
    pub min_size: Option<u64>,
    /// Maximum file size in bytes
    pub max_size: Option<u64>,
    /// Added after date
    pub added_after: Option<String>,
}

/// Collection manager
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Collections {
    /// Map of collection ID to collection
    pub collections: BTreeMap<String, Collection>,
}

impl Collections {
    /// Create a new empty collections manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a collection
    pub fn add(&mut self, collection: Collection) -> Result<()> {
        if collection.id.is_empty() {
            return Err(Error::msg("Collection ID cannot be empty"));
        }
        if collection.name.is_empty() {
            return Err(Error::msg("Collection name cannot be empty"));
        }
        self.collections.insert(collection.id.clone(), collection);
        Ok(())
    }

    /// Remove a collection by ID
    pub fn remove(&mut self, id: &str) -> Result<()> {
        self.collections
            .remove(id)
            .ok_or_else(|| Error::msg(format!("Collection with ID '{}' not found", id)))?;
        Ok(())
    }

    /// Get a collection by ID
    pub fn get(&self, id: &str) -> Option<&Collection> {
        self.collections.get(id)
    }

    /// Get a mutable reference to a collection by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Collection> {
        self.collections.get_mut(id)
    }

    /// Get all collections
    pub fn all(&self) -> Vec<&Collection> {
        self.collections.values().collect()
    }

    /// Get top-level collections (no parent)
    pub fn top_level(&self) -> Vec<&Collection> {
        self.collections
            .values()
            .filter(|c| c.parent_id.is_none())
            .collect()
    }

    /// Get child collections of a parent
    pub fn children(&self, parent_id: &str) -> Vec<&Collection> {
        self.collections
            .values()
            .filter(|c| c.parent_id.as_deref() == Some(parent_id))
            .collect()
    }

    /// Get the number of collections
    pub fn len(&self) -> usize {
        self.collections.len()
    }

    /// Check if there are any collections
    pub fn is_empty(&self) -> bool {
        self.collections.is_empty()
    }

    /// Clear all collections
    pub fn clear(&mut self) {
        self.collections.clear();
    }

    /// Load collections from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        load_json(path.as_ref()).with_context(|| {
            format!(
                "Failed to load collections from {}",
                path.as_ref().display()
            )
        })
    }

    /// Save collections to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        save_json(self, path.as_ref())
            .with_context(|| format!("Failed to save collections to {}", path.as_ref().display()))
    }

    /// Generate a unique ID for a new collection
    pub fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("collection_{}", timestamp)
    }

    /// Check if a book matches smart collection rules
    pub fn matches_smart_collection(
        &self,
        collection_id: &str,
        info: &crate::metadata::Info,
    ) -> bool {
        let Some(collection) = self.get(collection_id) else {
            return false;
        };

        let Some(rules) = &collection.rules else {
            return false;
        };

        // Check reading status
        if let Some(status) = &rules.reading_status {
            let status_matches = match status.as_str() {
                "reading" => info.simple_status() == crate::metadata::SimpleStatus::Reading,
                "new" => info.simple_status() == crate::metadata::SimpleStatus::New,
                "finished" => info.simple_status() == crate::metadata::SimpleStatus::Finished,
                _ => false,
            };
            if !status_matches {
                return false;
            }
        }

        // Check author
        if let Some(author_pattern) = &rules.author {
            if let Ok(re) = regex::Regex::new(author_pattern) {
                if !re.is_match(&info.author) {
                    return false;
                }
            }
        }

        // Check series
        if let Some(series_pattern) = &rules.series {
            if let Ok(re) = regex::Regex::new(series_pattern) {
                if !re.is_match(&info.series) {
                    return false;
                }
            }
        }

        // Check tags
        if !rules.tags.is_empty() {
            for tag in &rules.tags {
                if !info.tags.contains(tag) {
                    return false;
                }
            }
        }

        // Check categories
        if !rules.categories.is_empty() {
            for category in &rules.categories {
                if !info.categories.contains(category) {
                    return false;
                }
            }
        }

        // Check file size
        if let Some(min_size) = rules.min_size {
            if info.file.size < min_size {
                return false;
            }
        }
        if let Some(max_size) = rules.max_size {
            if info.file.size > max_size {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod collections_tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_collections_new() {
        let collections = Collections::new();
        assert!(collections.is_empty());
        assert_eq!(collections.len(), 0);
    }

    #[test]
    fn test_collections_add() {
        let mut collections = Collections::new();
        let collection = Collection {
            id: "test_id".to_string(),
            name: "Test Collection".to_string(),
            parent_id: None,
            color: Some("#FF0000".to_string()),
            icon: Some("folder".to_string()),
            rules: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            modified_at: "2024-01-01T00:00:00Z".to_string(),
        };

        collections.add(collection.clone()).expect("Test assertion failed");
        assert_eq!(collections.len(), 1);
        assert_eq!(collections.get("test_id"), Some(&collection));
    }

    #[test]
    fn test_collections_add_empty_id() {
        let mut collections = Collections::new();
        let collection = Collection {
            id: String::new(),
            name: "Test Collection".to_string(),
            parent_id: None,
            color: None,
            icon: None,
            rules: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            modified_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let result = collections.add(collection);
        assert!(result.is_err());
    }

    #[test]
    fn test_collections_add_empty_name() {
        let mut collections = Collections::new();
        let collection = Collection {
            id: "test_id".to_string(),
            name: String::new(),
            parent_id: None,
            color: None,
            icon: None,
            rules: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            modified_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let result = collections.add(collection);
        assert!(result.is_err());
    }

    #[test]
    fn test_collections_remove() {
        let mut collections = Collections::new();
        let collection = Collection {
            id: "test_id".to_string(),
            name: "Test Collection".to_string(),
            parent_id: None,
            color: None,
            icon: None,
            rules: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            modified_at: "2024-01-01T00:00:00Z".to_string(),
        };

        collections.add(collection).expect("Test assertion failed");
        collections.remove("test_id").expect("Test assertion failed");
        assert!(collections.is_empty());
    }

    #[test]
    fn test_collections_remove_not_found() {
        let mut collections = Collections::new();
        let result = collections.remove("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_collections_top_level() {
        let mut collections = Collections::new();

        collections
            .add(Collection {
                id: "parent".to_string(),
                name: "Parent".to_string(),
                parent_id: None,
                color: None,
                icon: None,
                rules: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                modified_at: "2024-01-01T00:00:00Z".to_string(),
            })
            .expect("Test assertion failed");

        collections
            .add(Collection {
                id: "child".to_string(),
                name: "Child".to_string(),
                parent_id: Some("parent".to_string()),
                color: None,
                icon: None,
                rules: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                modified_at: "2024-01-01T00:00:00Z".to_string(),
            })
            .expect("Test assertion failed");

        let top_level = collections.top_level();
        assert_eq!(top_level.len(), 1);
        assert_eq!(top_level[0].id, "parent");
    }

    #[test]
    fn test_collections_children() {
        let mut collections = Collections::new();

        collections
            .add(Collection {
                id: "parent".to_string(),
                name: "Parent".to_string(),
                parent_id: None,
                color: None,
                icon: None,
                rules: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                modified_at: "2024-01-01T00:00:00Z".to_string(),
            })
            .expect("Test assertion failed");

        collections
            .add(Collection {
                id: "child1".to_string(),
                name: "Child 1".to_string(),
                parent_id: Some("parent".to_string()),
                color: None,
                icon: None,
                rules: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                modified_at: "2024-01-01T00:00:00Z".to_string(),
            })
            .expect("Test assertion failed");

        collections
            .add(Collection {
                id: "child2".to_string(),
                name: "Child 2".to_string(),
                parent_id: Some("parent".to_string()),
                color: None,
                icon: None,
                rules: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                modified_at: "2024-01-01T00:00:00Z".to_string(),
            })
            .expect("Test assertion failed");

        let children = collections.children("parent");
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_collections_clear() {
        let mut collections = Collections::new();

        collections
            .add(Collection {
                id: "test_id".to_string(),
                name: "Test Collection".to_string(),
                parent_id: None,
                color: None,
                icon: None,
                rules: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                modified_at: "2024-01-01T00:00:00Z".to_string(),
            })
            .expect("Test assertion failed");

        collections.clear();
        assert!(collections.is_empty());
    }

    #[test]
    fn test_collections_save_load() {
        let mut collections = Collections::new();

        collections
            .add(Collection {
                id: "test_id".to_string(),
                name: "Test Collection".to_string(),
                parent_id: None,
                color: Some("#FF0000".to_string()),
                icon: Some("folder".to_string()),
                rules: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                modified_at: "2024-01-01T00:00:00Z".to_string(),
            })
            .expect("Test assertion failed");

        let temp_file = NamedTempFile::new().expect("Test assertion failed");
        let path = temp_file.path();

        collections.save(path).expect("Test assertion failed");
        let loaded = Collections::load(path).expect("Test assertion failed");

        assert_eq!(loaded.len(), 1);
        assert!(loaded.get("test_id").is_some());
    }

    #[test]
    fn test_collection_default() {
        let collection = Collection::default();
        assert!(collection.id.is_empty());
        assert!(collection.name.is_empty());
        assert!(collection.parent_id.is_none());
        assert!(collection.color.is_none());
        assert!(collection.icon.is_none());
        assert!(collection.rules.is_none());
        assert!(collection.created_at.is_empty());
        assert!(collection.modified_at.is_empty());
    }

    #[test]
    fn test_smart_collection_rules_default() {
        let rules = SmartCollectionRules::default();
        assert!(rules.reading_status.is_none());
        assert!(rules.author.is_none());
        assert!(rules.series.is_none());
        assert!(rules.tags.is_empty());
        assert!(rules.categories.is_empty());
        assert!(rules.min_size.is_none());
        assert!(rules.max_size.is_none());
        assert!(rules.added_after.is_none());
    }

    #[test]
    fn test_matches_smart_collection_reading_status() {
        let mut collections = Collections::new();

        collections
            .add(Collection {
                id: "reading".to_string(),
                name: "Reading".to_string(),
                parent_id: None,
                color: None,
                icon: None,
                rules: Some(SmartCollectionRules {
                    reading_status: Some("reading".to_string()),
                    ..Default::default()
                }),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                modified_at: "2024-01-01T00:00:00Z".to_string(),
            })
            .expect("Test assertion failed");

        let mut info = crate::metadata::Info::default();
        info.reader = Some(crate::metadata::ReaderInfo::default());

        assert!(collections.matches_smart_collection("reading", &info));
    }
}
