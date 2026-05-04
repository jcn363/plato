//! Article Management Module for Pocket/Instapaper Integration
//!
//! Provides data structures and utilities for managing web articles from
//! read-later services like Pocket and Instapaper. Supports article metadata,
//! tags, reading progress, and local caching.
//!
//! ## Features
//!
//! - **Article Data Structures**: Unified representation for articles from any source
//! - **Tag Management**: Hierarchical tagging system for article organization
//! - **Reading Progress**: Track reading position and completion status
//! - **Local Caching**: Store articles for offline reading
//! - **Export Support**: Export highlights to Readwise, Obsidian, and other services
//!
//! ## Architecture
//!
//! ```text
//! article/
//! ├── Article          - Core article data structure
//! ├── ArticleSource    - Enum for Pocket, Instapaper, etc.
//! ├── ArticleStatus    - Unread, archived, deleted, favorited
//! ├── TagManager       - Tag organization and filtering
//! └── ReadProgress     - Reading position tracking
//! ```

use anyhow::{Context, Error};
use chrono::{DateTime, Utc};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Unique identifier for articles across all sources
pub type ArticleId = String;

/// Source of the article (Pocket, Instapaper, or local)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleSource {
    /// Article from Pocket (getpocket.com)
    Pocket,
    /// Article from Instapaper (instapaper.com)
    Instapaper,
    /// Locally saved article (user imported)
    Local,
    /// Article from Wallabag instance
    Wallabag,
}

impl Default for ArticleSource {
    fn default() -> Self {
        ArticleSource::Local
    }
}

impl std::fmt::Display for ArticleSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArticleSource::Pocket => write!(f, "Pocket"),
            ArticleSource::Instapaper => write!(f, "Instapaper"),
            ArticleSource::Local => write!(f, "Local"),
            ArticleSource::Wallabag => write!(f, "Wallabag"),
        }
    }
}

/// Article reading status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleStatus {
    /// Article is unread
    Unread,
    /// Article has been read and archived
    Archived,
    /// Article is favorited/starred
    Favorited,
    /// Article is marked for deletion
    Deleted,
    /// Article is currently being read
    Reading,
}

impl Default for ArticleStatus {
    fn default() -> Self {
        ArticleStatus::Unread
    }
}

/// Article content format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleFormat {
    /// HTML content
    Html,
    /// Markdown content
    Markdown,
    /// Plain text content
    Text,
    /// EPUB format
    Epub,
}

impl Default for ArticleFormat {
    fn default() -> Self {
        ArticleFormat::Html
    }
}

/// Reading progress for an article
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadProgress {
    /// Percentage read (0.0 to 1.0)
    pub percentage: f32,
    /// Character position in the text
    pub char_position: usize,
    /// Last paragraph or section read
    pub section_index: Option<usize>,
    /// Estimated time spent reading (seconds)
    pub time_spent_seconds: u64,
    /// Last read timestamp
    pub last_read_at: Option<DateTime<Utc>>,
    /// Reading started timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Reading finished timestamp
    pub finished_at: Option<DateTime<Utc>>,
}

impl ReadProgress {
    /// Create new empty progress
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark reading as started
    pub fn mark_started(&mut self) {
        if self.started_at.is_none() {
            self.started_at = Some(Utc::now());
        }
        self.last_read_at = Some(Utc::now());
    }

    /// Mark reading as finished
    pub fn mark_finished(&mut self) {
        self.percentage = 1.0;
        self.finished_at = Some(Utc::now());
        self.last_read_at = Some(Utc::now());
    }

    /// Update progress with new percentage
    pub fn update_percentage(&mut self, percentage: f32) {
        self.percentage = percentage.clamp(0.0, 1.0);
        self.last_read_at = Some(Utc::now());
    }

    /// Check if article is finished
    pub fn is_finished(&self) -> bool {
        self.percentage >= 0.95 || self.finished_at.is_some()
    }

    /// Get estimated remaining time based on reading speed
    pub fn estimated_remaining_minutes(&self, words_per_minute: u32) -> Option<u32> {
        if self.is_finished() {
            return Some(0);
        }
        let remaining_percentage = 1.0 - self.percentage;
        let total_words_estimate = 2000; // Default estimate
        let words_remaining = (total_words_estimate as f32 * remaining_percentage) as u32;
        Some(words_remaining / words_per_minute.max(1))
    }
}

/// A highlight within an article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleHighlight {
    /// Unique highlight ID
    pub id: String,
    /// Highlighted text
    pub text: String,
    /// Start position in the document
    pub start_position: usize,
    /// End position in the document
    pub end_position: usize,
    /// Optional user note
    pub note: Option<String>,
    /// Timestamp when highlight was created
    pub created_at: DateTime<Utc>,
    /// Color/category of the highlight
    pub color: Option<String>,
}

/// Article image/asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleImage {
    /// Original URL of the image
    pub url: String,
    /// Local path if cached
    pub local_path: Option<PathBuf>,
    /// Image caption
    pub caption: Option<String>,
    /// Width in pixels
    pub width: Option<u32>,
    /// Height in pixels
    pub height: Option<u32>,
}

/// Core article data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    /// Unique article ID (source-specific)
    pub id: ArticleId,
    /// Article source (Pocket, Instapaper, etc.)
    pub source: ArticleSource,
    /// Original URL of the article
    pub url: String,
    /// Article title
    pub title: String,
    /// Article author(s)
    pub authors: Vec<String>,
    /// Article summary/excerpt
    pub excerpt: Option<String>,
    /// Article content (HTML, Markdown, or plain text)
    pub content: Option<String>,
    /// Content format
    pub content_format: ArticleFormat,
    /// Article language (ISO 639-1 code)
    pub language: Option<String>,
    /// Reading time estimate in minutes
    pub reading_time_minutes: Option<u32>,
    /// Word count
    pub word_count: Option<u32>,
    /// Tags associated with the article
    pub tags: Vec<String>,
    /// Article status
    pub status: ArticleStatus,
    /// Reading progress
    pub progress: ReadProgress,
    /// User highlights
    pub highlights: Vec<ArticleHighlight>,
    /// Article images
    pub images: Vec<ArticleImage>,
    /// When the article was added to the service
    pub added_at: DateTime<Utc>,
    /// When the article was last updated
    pub updated_at: DateTime<Utc>,
    /// When the article was favorited (if applicable)
    pub favorited_at: Option<DateTime<Utc>>,
    /// Source-specific metadata
    #[serde(flatten)]
    pub source_metadata: FxHashMap<String, serde_json::Value>,
    /// Local file path if article is cached
    pub local_path: Option<PathBuf>,
    /// Whether the article is available offline
    pub is_offline_available: bool,
    /// Whether the article content has been extracted
    pub is_content_extracted: bool,
    /// Original domain of the article
    pub domain: Option<String>,
    /// Article thumbnail/image URL
    pub thumbnail_url: Option<String>,
}

impl Article {
    /// Create a new article with minimal required fields
    pub fn new(id: ArticleId, source: ArticleSource, url: String, title: String) -> Self {
        let now = Utc::now();
        let domain = Self::extract_domain(&url);

        Self {
            id,
            source,
            url,
            title,
            authors: Vec::new(),
            excerpt: None,
            content: None,
            content_format: ArticleFormat::Html,
            language: None,
            reading_time_minutes: None,
            word_count: None,
            tags: Vec::new(),
            status: ArticleStatus::Unread,
            progress: ReadProgress::new(),
            highlights: Vec::new(),
            images: Vec::new(),
            added_at: now,
            updated_at: now,
            favorited_at: None,
            source_metadata: FxHashMap::default(),
            local_path: None,
            is_offline_available: false,
            is_content_extracted: false,
            domain,
            thumbnail_url: None,
        }
    }

    /// Extract domain from URL
    fn extract_domain(url: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
    }

    /// Add a tag to the article
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a tag from the article
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.updated_at = Utc::now();
    }

    /// Add a highlight
    pub fn add_highlight(&mut self, highlight: ArticleHighlight) {
        self.highlights.push(highlight);
        self.updated_at = Utc::now();
    }

    /// Mark article as favorited
    pub fn favorite(&mut self) {
        self.status = ArticleStatus::Favorited;
        self.favorited_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark article as archived (read)
    pub fn archive(&mut self) {
        self.status = ArticleStatus::Archived;
        self.progress.mark_finished();
        self.updated_at = Utc::now();
    }

    /// Mark article as deleted
    pub fn delete(&mut self) {
        self.status = ArticleStatus::Deleted;
        self.updated_at = Utc::now();
    }

    /// Re-add a deleted article
    pub fn readd(&mut self) {
        if self.status == ArticleStatus::Deleted {
            self.status = ArticleStatus::Unread;
            self.updated_at = Utc::now();
        }
    }

    /// Update content and recalculate word count
    pub fn set_content(&mut self, content: String, format: ArticleFormat) {
        self.content = Some(content.clone());
        self.content_format = format;
        self.word_count = Some(Self::estimate_word_count(&content));
        self.reading_time_minutes = self.word_count.map(|w| (w / 200).max(1)); // 200 WPM average
        self.is_content_extracted = true;
        self.updated_at = Utc::now();
    }

    /// Estimate word count from text
    fn estimate_word_count(text: &str) -> u32 {
        text.split_whitespace().count() as u32
    }

    /// Check if article is available for offline reading
    pub fn is_offline_ready(&self) -> bool {
        self.is_offline_available && self.local_path.is_some()
    }

    /// Get authors as a formatted string
    pub fn authors_string(&self) -> String {
        if self.authors.is_empty() {
            self.domain.clone().unwrap_or_else(|| "Unknown".to_string())
        } else {
            self.authors.join(", ")
        }
    }

    /// Export highlights to Readwise format
    pub fn export_highlights_readwise(&self) -> Vec<ReadwiseHighlight> {
        self.highlights
            .iter()
            .map(|h| ReadwiseHighlight {
                text: h.text.clone(),
                title: self.title.clone(),
                author: self.authors_string(),
                source_url: self.url.clone(),
                category: "articles".to_string(),
                note: h.note.clone(),
                highlighted_at: h.created_at,
            })
            .collect()
    }

    /// Export highlights to Obsidian format
    pub fn export_highlights_obsidian(&self) -> String {
        let mut output = format!("# {}\n\n", self.title);
        output.push_str(&format!("**Source:** [{}]({})\n\n", self.domain.as_deref().unwrap_or(""), self.url));
        output.push_str(&format!("**Author:** {}\n\n", self.authors_string()));

        if !self.highlights.is_empty() {
            output.push_str("## Highlights\n\n");
            for highlight in &self.highlights {
                output.push_str("> ");
                output.push_str(&highlight.text);
                output.push('\n');
                if let Some(note) = &highlight.note {
                    output.push_str(&format!("\n**Note:** {}\n", note));
                }
                output.push('\n');
            }
        }

        output
    }
}

/// Readwise highlight format for export
#[derive(Debug, Clone, Serialize)]
pub struct ReadwiseHighlight {
    pub text: String,
    pub title: String,
    pub author: String,
    pub source_url: String,
    pub category: String,
    pub note: Option<String>,
    pub highlighted_at: DateTime<Utc>,
}

/// Filter criteria for article queries
#[derive(Debug, Clone, Default)]
pub struct ArticleFilter {
    /// Filter by status
    pub status: Option<ArticleStatus>,
    /// Filter by source
    pub source: Option<ArticleSource>,
    /// Filter by tags (any of these)
    pub tags: Option<Vec<String>>,
    /// Filter by search text
    pub search_text: Option<String>,
    /// Filter by date range (added after)
    pub added_after: Option<DateTime<Utc>>,
    /// Filter by date range (added before)
    pub added_before: Option<DateTime<Utc>>,
    /// Only offline available articles
    pub offline_only: bool,
    /// Only favorited articles
    pub favorited_only: bool,
}

/// Sort options for articles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArticleSort {
    /// Sort by date added (newest first)
    DateAddedDesc,
    /// Sort by date added (oldest first)
    DateAddedAsc,
    /// Sort by updated date (newest first)
    DateUpdatedDesc,
    /// Sort by updated date (oldest first)
    DateUpdatedAsc,
    /// Sort by title (A-Z)
    TitleAsc,
    /// Sort by title (Z-A)
    TitleDesc,
    /// Sort by reading time (shortest first)
    ReadingTimeAsc,
    /// Sort by reading time (longest first)
    ReadingTimeDesc,
}

impl Default for ArticleSort {
    fn default() -> Self {
        ArticleSort::DateAddedDesc
    }
}

/// Collection of articles with filtering and sorting
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArticleCollection {
    articles: FxHashMap<ArticleId, Article>,
    /// Tag to article ID mapping
    tag_index: FxHashMap<String, HashSet<ArticleId>>,
    /// Source to article ID mapping
    source_index: FxHashMap<ArticleSource, HashSet<ArticleId>>,
    /// Status to article ID mapping
    status_index: FxHashMap<ArticleStatus, HashSet<ArticleId>>,
}

impl ArticleCollection {
    /// Create new empty collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an article to the collection
    pub fn add(&mut self, article: Article) {
        let id = article.id.clone();
        let source = article.source;
        let status = article.status;
        let tags = article.tags.clone();

        self.articles.insert(id.clone(), article);

        // Update indexes
        self.source_index
            .entry(source)
            .or_default()
            .insert(id.clone());
        self.status_index
            .entry(status)
            .or_default()
            .insert(id.clone());

        for tag in tags {
            self.tag_index.entry(tag).or_default().insert(id.clone());
        }
    }

    /// Remove an article from the collection
    pub fn remove(&mut self, id: &ArticleId) -> Option<Article> {
        let article = self.articles.remove(id)?;

        // Update indexes
        self.source_index
            .get_mut(&article.source)
            .map(|set| set.remove(id));
        self.status_index
            .get_mut(&article.status)
            .map(|set| set.remove(id));

        for tag in &article.tags {
            self.tag_index.get_mut(tag).map(|set| set.remove(id));
        }

        Some(article)
    }

    /// Get an article by ID
    pub fn get(&self, id: &ArticleId) -> Option<&Article> {
        self.articles.get(id)
    }

    /// Get mutable reference to an article
    pub fn get_mut(&mut self, id: &ArticleId) -> Option<&mut Article> {
        self.articles.get_mut(id)
    }

    /// Get all articles
    pub fn all(&self) -> Vec<&Article> {
        self.articles.values().collect()
    }

    /// Get filtered and sorted articles
    pub fn filter_and_sort(
        &self,
        filter: &ArticleFilter,
        sort: ArticleSort,
    ) -> Vec<&Article> {
        let mut results: Vec<&Article> = self
            .articles
            .values()
            .filter(|article| self.matches_filter(article, filter))
            .collect();

        // Sort
        results.sort_by(|a, b| self.compare_articles(a, b, sort));

        results
    }

    /// Check if article matches filter criteria
    fn matches_filter(&self, article: &Article, filter: &ArticleFilter) -> bool {
        if let Some(status) = filter.status {
            if article.status != status {
                return false;
            }
        }

        if let Some(source) = filter.source {
            if article.source != source {
                return false;
            }
        }

        if let Some(tags) = &filter.tags {
            if !tags.iter().any(|t| article.tags.contains(t)) {
                return false;
            }
        }

        if let Some(search) = &filter.search_text {
            let search_lower = search.to_lowercase();
            if !article.title.to_lowercase().contains(&search_lower)
                && !article.url.to_lowercase().contains(&search_lower)
                && !article.authors.iter().any(|a| a.to_lowercase().contains(&search_lower))
                && !article.tags.iter().any(|t| t.to_lowercase().contains(&search_lower))
            {
                return false;
            }
        }

        if let Some(after) = filter.added_after {
            if article.added_at < after {
                return false;
            }
        }

        if let Some(before) = filter.added_before {
            if article.added_at > before {
                return false;
            }
        }

        if filter.offline_only && !article.is_offline_ready() {
            return false;
        }

        if filter.favorited_only && article.status != ArticleStatus::Favorited {
            return false;
        }

        true
    }

    /// Compare two articles for sorting
    fn compare_articles(&self, a: &Article, b: &Article, sort: ArticleSort) -> std::cmp::Ordering {
        match sort {
            ArticleSort::DateAddedDesc => b.added_at.cmp(&a.added_at),
            ArticleSort::DateAddedAsc => a.added_at.cmp(&b.added_at),
            ArticleSort::DateUpdatedDesc => b.updated_at.cmp(&a.updated_at),
            ArticleSort::DateUpdatedAsc => a.updated_at.cmp(&b.updated_at),
            ArticleSort::TitleAsc => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            ArticleSort::TitleDesc => b.title.to_lowercase().cmp(&a.title.to_lowercase()),
            ArticleSort::ReadingTimeAsc => {
                a.reading_time_minutes.unwrap_or(0).cmp(&b.reading_time_minutes.unwrap_or(0))
            }
            ArticleSort::ReadingTimeDesc => {
                b.reading_time_minutes.unwrap_or(0).cmp(&a.reading_time_minutes.unwrap_or(0))
            }
        }
    }

    /// Get all unique tags in the collection
    pub fn all_tags(&self) -> Vec<String> {
        self.tag_index.keys().cloned().collect()
    }

    /// Get articles by tag
    pub fn by_tag(&self, tag: &str) -> Vec<&Article> {
        self.tag_index
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.articles.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get article count
    pub fn len(&self) -> usize {
        self.articles.len()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.articles.is_empty()
    }

    /// Get count by status
    pub fn count_by_status(&self, status: ArticleStatus) -> usize {
        self.status_index
            .get(&status)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    /// Get count by source
    pub fn count_by_source(&self, source: ArticleSource) -> usize {
        self.source_index
            .get(&source)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    /// Get count by tag
    pub fn count_by_tag(&self, tag: &str) -> usize {
        self.tag_index
            .get(tag)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    /// Update article (reindex if needed)
    pub fn update(&mut self, article: Article) {
        let old = self.remove(&article.id);

        // If tags changed, we need to handle reindexing
        if let Some(old_article) = old {
            let mut new_article = article;
            // Preserve certain fields if needed
            if new_article.progress.time_spent_seconds == 0 {
                new_article.progress.time_spent_seconds = old_article.progress.time_spent_seconds;
            }
            self.add(new_article);
        } else {
            self.add(article);
        }
    }

    /// Export all highlights for all articles
    pub fn export_all_highlights(&self) -> Vec<ReadwiseHighlight> {
        self.articles
            .values()
            .flat_map(|a| a.export_highlights_readwise())
            .collect()
    }

    /// Get unread count
    pub fn unread_count(&self) -> usize {
        self.count_by_status(ArticleStatus::Unread)
    }

    /// Get archived count
    pub fn archived_count(&self) -> usize {
        self.count_by_status(ArticleStatus::Archived)
    }

    /// Get favorited count
    pub fn favorited_count(&self) -> usize {
        self.count_by_status(ArticleStatus::Favorited)
    }

    /// Mark all articles in a tag as archived
    pub fn archive_by_tag(&mut self, tag: &str) -> usize {
        let ids: Vec<String> = self
            .tag_index
            .get(tag)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default();

        let mut count = 0;
        for id in ids {
            if let Some(article) = self.get_mut(&id) {
                if article.status == ArticleStatus::Unread
                    || article.status == ArticleStatus::Reading
                {
                    article.archive();
                    count += 1;
                }
            }
        }

        // Rebuild status index
        self.rebuild_status_index();

        count
    }

    /// Rebuild the status index
    fn rebuild_status_index(&mut self) {
        self.status_index.clear();
        for (id, article) in &self.articles {
            self.status_index
                .entry(article.status)
                .or_default()
                .insert(id.clone());
        }
    }

    /// Clear all articles
    pub fn clear(&mut self) {
        self.articles.clear();
        self.tag_index.clear();
        self.source_index.clear();
        self.status_index.clear();
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(self)
            .context("Failed to serialize article collection")
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json)
            .context("Failed to deserialize article collection")
    }
}

/// Statistics for article collection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArticleStats {
    pub total_articles: usize,
    pub unread_count: usize,
    pub archived_count: usize,
    pub favorited_count: usize,
    pub reading_count: usize,
    pub total_word_count: u64,
    pub total_reading_time_minutes: u64,
    pub source_breakdown: FxHashMap<ArticleSource, usize>,
    pub tag_breakdown: FxHashMap<String, usize>,
}

impl ArticleStats {
    /// Calculate stats from a collection
    pub fn from_collection(collection: &ArticleCollection) -> Self {
        let total_articles = collection.len();
        let unread_count = collection.count_by_status(ArticleStatus::Unread);
        let archived_count = collection.count_by_status(ArticleStatus::Archived);
        let favorited_count = collection.count_by_status(ArticleStatus::Favorited);
        let reading_count = collection.count_by_status(ArticleStatus::Reading);

        let mut total_word_count = 0u64;
        let mut total_reading_time = 0u64;
        let mut source_breakdown = FxHashMap::default();
        let mut tag_breakdown = FxHashMap::default();

        for article in collection.all() {
            total_word_count += article.word_count.unwrap_or(0) as u64;
            total_reading_time += article.reading_time_minutes.unwrap_or(0) as u64;

            *source_breakdown.entry(article.source).or_insert(0) += 1;

            for tag in &article.tags {
                *tag_breakdown.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        Self {
            total_articles,
            unread_count,
            archived_count,
            favorited_count,
            reading_count,
            total_word_count,
            total_reading_time_minutes: total_reading_time,
            source_breakdown,
            tag_breakdown,
        }
    }

    /// Estimated days to read all unread articles at given WPM
    pub fn estimated_days_to_clear(&self, words_per_day: u32) -> f64 {
        if words_per_day == 0 {
            return 0.0;
        }
        let unread_words = self.unread_count as u64 * (self.total_word_count / self.total_articles.max(1) as u64);
        (unread_words as f64) / (words_per_day as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_article(id: &str, title: &str) -> Article {
        Article::new(
            id.to_string(),
            ArticleSource::Pocket,
            format!("https://example.com/article/{}", id),
            title.to_string(),
        )
    }

    #[test]
    fn test_article_creation() {
        let article = create_test_article("1", "Test Article");
        assert_eq!(article.id, "1");
        assert_eq!(article.title, "Test Article");
        assert_eq!(article.source, ArticleSource::Pocket);
        assert_eq!(article.status, ArticleStatus::Unread);
    }

    #[test]
    fn test_article_tags() {
        let mut article = create_test_article("1", "Test");
        article.add_tag("rust".to_string());
        article.add_tag("programming".to_string());
        article.add_tag("rust".to_string()); // Duplicate

        assert_eq!(article.tags.len(), 2);
        assert!(article.tags.contains(&"rust".to_string()));
        assert!(article.tags.contains(&"programming".to_string()));

        article.remove_tag("rust");
        assert_eq!(article.tags.len(), 1);
    }

    #[test]
    fn test_read_progress() {
        let mut progress = ReadProgress::new();
        assert!(!progress.is_finished());

        progress.mark_started();
        assert!(progress.started_at.is_some());

        progress.mark_finished();
        assert!(progress.is_finished());
        assert_eq!(progress.percentage, 1.0);
    }

    #[test]
    fn test_article_collection() {
        let mut collection = ArticleCollection::new();

        let article1 = create_test_article("1", "Article 1");
        let article2 = create_test_article("2", "Article 2");

        collection.add(article1);
        collection.add(article2);

        assert_eq!(collection.len(), 2);
        assert!(collection.get(&"1".to_string()).is_some());
        assert!(collection.get(&"2".to_string()).is_some());
    }

    #[test]
    fn test_article_filter() {
        let mut collection = ArticleCollection::new();

        let mut article1 = create_test_article("1", "Rust Programming");
        article1.add_tag("rust".to_string());
        article1.status = ArticleStatus::Unread;

        let mut article2 = create_test_article("2", "Python Guide");
        article2.add_tag("python".to_string());
        article2.status = ArticleStatus::Archived;

        collection.add(article1);
        collection.add(article2);

        let filter = ArticleFilter {
            status: Some(ArticleStatus::Unread),
            ..Default::default()
        };
        let results = collection.filter_and_sort(&filter, ArticleSort::default());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming");

        let filter = ArticleFilter {
            search_text: Some("python".to_string()),
            ..Default::default()
        };
        let results = collection.filter_and_sort(&filter, ArticleSort::default());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Python Guide");
    }

    #[test]
    fn test_highlights_export() {
        let mut article = create_test_article("1", "Test Article");
        article.authors.push("John Doe".to_string());

        let highlight = ArticleHighlight {
            id: "h1".to_string(),
            text: "Important quote".to_string(),
            start_position: 0,
            end_position: 15,
            note: Some("My note".to_string()),
            created_at: Utc::now(),
            color: Some("yellow".to_string()),
        };

        article.add_highlight(highlight);

        let readwise = article.export_highlights_readwise();
        assert_eq!(readwise.len(), 1);
        assert_eq!(readwise[0].text, "Important quote");

        let obsidian = article.export_highlights_obsidian();
        assert!(obsidian.contains("# Test Article"));
        assert!(obsidian.contains("> Important quote"));
    }
}
