//! Instapaper API Integration Module
//!
#![allow(dead_code)]

//! Provides full integration with the Instapaper API (instapaper.com) for syncing
//! saved articles, managing read/unread status, archiving, favoriting, and organizing
//! into folders.
//!
//! ## Features
//!
//! - **Authentication**: Simple username/password auth (OAuth not required)
//! - **Article Sync**: Fetch all saved articles with metadata
//! - **Folder Support**: Organize articles into folders
//! - **Bookmark Management**: Add, delete, move bookmarks
//! - **Highlight Sync**: Retrieve user highlights
//! - **Offline Support**: Download article content for offline reading
//! - **Incremental Sync**: Efficient updates using timestamps
//!
//! ## API Documentation
//!
//! Based on the official Instapaper API documentation:
//! - https://www.instapaper.com/api/simple
//! - https://www.instapaper.com/api/full
//!
//! ## Usage
//!
//! ```rust,ignore
//! use plato_core::instapaper::{InstapaperClient, InstapaperAuth};
//! use plato_core::settings::InstapaperSettings;
//!
//! // Initialize client
//! let client = InstapaperClient::new(&settings)?;
//!
//! // Fetch unread articles
//! let articles = client.fetch_unread_bookmarks()?;
//!
//! // Archive an article
//! client.archive_bookmark(12345)?;
//! ```

use crate::article::{
    Article, ArticleSource, ArticleStatus, ArticleCollection,
};
use crate::log_info;
use crate::settings::InstapaperSettings;
use anyhow::{bail, format_err, Context, Error};
use chrono::DateTime;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

/// Instapaper API base URL
const INSTAPAPER_API_BASE: &str = "https://www.instapaper.com/api/1";

/// Request timeout for API calls
const API_TIMEOUT_SECONDS: u64 = 30;

/// Instapaper API error response
#[derive(Debug, Deserialize)]
pub struct InstapaperError {
    /// Error code
    pub error_code: i32,
    /// Error message
    pub message: String,
}

/// Instapaper bookmark (article)
#[derive(Debug, Clone, Deserialize)]
struct InstapaperBookmark {
    #[serde(rename = "bookmark_id")]
    bookmark_id: i64,
    #[serde(rename = "hash")]
    hash: String,
    #[serde(rename = "description")]
    description: Option<String>,
    #[serde(rename = "url")]
    url: String,
    #[serde(rename = "title")]
    title: String,
    #[serde(rename = "time")]
    time: i64,
    #[serde(rename = "starred")]
    starred: String, // "0" or "1"
    #[serde(rename = "private_source")]
    private_source: Option<String>,
    #[serde(default)]
    #[serde(rename = "tags")]
    tags: String, // comma-separated
    #[serde(rename = "folder")]
    folder: Option<String>,
    #[serde(rename = "progress")]
    progress: Option<f32>, // 0.0 to 1.0
    #[serde(rename = "progress_timestamp")]
    progress_timestamp: Option<i64>,
}

/// Instapaper folder
#[derive(Debug, Clone, Deserialize)]
pub struct InstapaperFolder {
    /// Folder ID
    #[serde(rename = "folder_id")]
    pub folder_id: i64,
    /// Folder title
    #[serde(rename = "title")]
    pub title: String,
    /// Whether to sync to mobile
    #[serde(rename = "sync_to_mobile")]
    pub sync_to_mobile: Option<i32>,
    /// Folder position in list
    #[serde(rename = "folder_position")]
    pub folder_position: Option<i32>,
}

/// Instapaper highlight
#[derive(Debug, Clone, Deserialize)]
pub struct InstapaperHighlight {
    /// Highlight ID
    #[serde(rename = "highlight_id")]
    pub highlight_id: i64,
    /// Bookmark ID the highlight belongs to
    #[serde(rename = "bookmark_id")]
    pub bookmark_id: i64,
    /// Highlighted text
    #[serde(rename = "text")]
    pub text: String,
    /// Optional note on the highlight
    #[serde(rename = "note")]
    pub note: Option<String>,
    /// Timestamp when highlight was created
    #[serde(rename = "time")]
    pub time: i64,
    /// Position in the document
    #[serde(rename = "position")]
    pub position: Option<i32>,
}

/// Authentication credentials for Instapaper
pub struct InstapaperAuth {
    username: String,
    password: String,
}

impl InstapaperAuth {
    /// Create new auth credentials
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    /// Validate credentials are not empty
    pub fn validate(&self) -> Result<(), Error> {
        if self.username.is_empty() {
            bail!("Instapaper username cannot be empty");
        }
        if self.password.is_empty() {
            bail!("Instapaper password cannot be empty");
        }
        Ok(())
    }
}

/// Instapaper API client
pub struct InstapaperClient {
    auth: InstapaperAuth,
    client: Client,
}

impl InstapaperClient {
    /// Create new Instapaper client from settings
    pub fn new(settings: &InstapaperSettings) -> Result<Self, Error> {
        let username = settings
            .username
            .as_ref()
            .ok_or_else(|| format_err!("Instapaper username not configured"))?
            .clone();

        let password = settings
            .password
            .as_ref()
            .ok_or_else(|| format_err!("Instapaper password not configured"))?
            .clone();

        let auth = InstapaperAuth::new(username, password);
        auth.validate()?;

        let client = Client::builder()
            .timeout(Duration::from_secs(API_TIMEOUT_SECONDS))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { auth, client })
    }

    /// Create client with explicit credentials
    pub fn with_credentials(username: String, password: String) -> Result<Self, Error> {
        let auth = InstapaperAuth::new(username, password);
        auth.validate()?;

        let client = Client::builder()
            .timeout(Duration::from_secs(API_TIMEOUT_SECONDS))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { auth, client })
    }

    /// Check if client has valid credentials
    pub fn is_configured(&self) -> bool {
        !self.auth.username.is_empty() && !self.auth.password.is_empty()
    }

    /// Make authenticated API request
    fn make_request(
        &self,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/{}", INSTAPAPER_API_BASE, endpoint);

        let mut request = self
            .client
            .post(&url)
            .basic_auth(&self.auth.username, Some(&self.auth.password));

        if let Some(p) = params {
            // Build URL-encoded form data manually
            let form_data: String = p
                .iter()
                .map(|(k, v)| format!("{}={}",
                    percent_encoding::percent_encode(k.as_bytes(), percent_encoding::NON_ALPHANUMERIC),
                    percent_encoding::percent_encode(v.as_bytes(), percent_encoding::NON_ALPHANUMERIC)))
                .collect::<Vec<_>>()
                .join("&");
            request = request
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(form_data);
        }

        let response = request
            .send()
            .with_context(|| format!("Failed to make Instapaper API request to {}", endpoint))?;

        let status = response.status();
        let text = response
            .text()
            .unwrap_or_else(|_| "Failed to read response".to_string());

        if !status.is_success() {
            // Try to parse error
            if let Ok(error) = serde_json::from_str::<InstapaperError>(&text) {
                bail!(
                    "Instapaper API error ({}): {}",
                    error.error_code,
                    error.message
                );
            }
            bail!("Instapaper API error ({}): {}", status, text);
        }

        // Instapaper returns JSON arrays for successful responses
        let json: serde_json::Value =
            serde_json::from_str(&text).context("Failed to parse Instapaper API response")?;

        Ok(json)
    }

    /// Authenticate and verify credentials
    pub fn authenticate(&self) -> Result<(String, String), Error> {
        let response = self.make_request("authenticate", None)?;

        // Response should be an array with user info
        if let Some(array) = response.as_array() {
            if let Some(user) = array.first() {
                let username = user
                    .get("username")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let user_id = user
                    .get("user_id")
                    .and_then(|v| v.as_i64())
                    .map(|id| id.to_string())
                    .unwrap_or_default();
                return Ok((username, user_id));
            }
        }

        bail!("Unexpected response format from Instapaper authenticate")
    }

    /// Verify credentials are valid
    pub fn verify_credentials(&self) -> Result<bool, Error> {
        match self.authenticate() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// List all folders
    pub fn list_folders(&self) -> Result<Vec<InstapaperFolder>, Error> {
        let response = self.make_request("folders/list", None)?;

        let folders: Vec<InstapaperFolder> =
            serde_json::from_value(response).context("Failed to parse folders response")?;

        Ok(folders)
    }

    /// Add a new folder
    pub fn add_folder(&self, title: &str) -> Result<InstapaperFolder, Error> {
        let mut params = HashMap::new();
        params.insert("title".to_string(), title.to_string());

        let response = self.make_request("folders/add", Some(params))?;

        let folder: InstapaperFolder =
            serde_json::from_value(response).context("Failed to parse folder add response")?;

        Ok(folder)
    }

    /// Delete a folder
    pub fn delete_folder(&self, folder_id: i64) -> Result<(), Error> {
        let mut params = HashMap::new();
        params.insert("folder_id".to_string(), folder_id.to_string());

        self.make_request("folders/delete", Some(params))?;
        Ok(())
    }

    /// Set folder order
    pub fn set_folder_order(&self, folder_ids: &[i64]) -> Result<(), Error> {
        let ids_string = folder_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let mut params = HashMap::new();
        params.insert("order".to_string(), ids_string);

        self.make_request("folders/set_order", Some(params))?;
        Ok(())
    }

    /// Fetch bookmarks (articles) from a folder
    pub fn fetch_bookmarks(
        &self,
        folder_id: Option<i64>,
        limit: Option<i32>,
        have: Option<Vec<i64>>,
    ) -> Result<Vec<Article>, Error> {
        let mut params = HashMap::new();

        if let Some(id) = folder_id {
            params.insert("folder_id".to_string(), id.to_string());
        }

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        if let Some(h) = have {
            let have_string = h
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.insert("have".to_string(), have_string);
        }

        let response = self.make_request("bookmarks/list", Some(params))?;

        let bookmarks: Vec<InstapaperBookmark> =
            serde_json::from_value(response).context("Failed to parse bookmarks response")?;

        let mut articles = Vec::with_capacity(bookmarks.len());
        for bookmark in bookmarks {
            let article = self.convert_bookmark_to_article(bookmark)?;
            articles.push(article);
        }

        Ok(articles)
    }

    /// Fetch unread bookmarks
    pub fn fetch_unread_bookmarks(&self) -> Result<Vec<Article>, Error> {
        // Unread bookmarks are in the "unread" folder (folder_id not specified or 0)
        self.fetch_bookmarks(None, None, None)
    }

    /// Fetch archived bookmarks
    pub fn fetch_archived_bookmarks(&self) -> Result<Vec<Article>, Error> {
        // Archived bookmarks are in the "archive" folder
        self.fetch_bookmarks(Some(-1), None, None)
    }

    /// Fetch starred bookmarks
    pub fn fetch_starred_bookmarks(&self) -> Result<Vec<Article>, Error> {
        let all = self.fetch_all_bookmarks()?;
        let starred: Vec<Article> = all
            .into_iter()
            .filter(|a| a.status == ArticleStatus::Favorited)
            .collect();
        Ok(starred)
    }

    /// Fetch all bookmarks across all folders
    pub fn fetch_all_bookmarks(&self) -> Result<Vec<Article>, Error> {
        let mut all_articles = Vec::new();

        // Fetch unread
        let unread = self.fetch_unread_bookmarks()?;
        all_articles.extend(unread);

        // Fetch archived
        let archived = self.fetch_archived_bookmarks()?;
        all_articles.extend(archived);

        Ok(all_articles)
    }

    /// Convert Instapaper bookmark to Article struct
    fn convert_bookmark_to_article(
        &self,
        bookmark: InstapaperBookmark,
    ) -> Result<Article, Error> {
        let id = bookmark.bookmark_id.to_string();
        let url = bookmark.url.clone();
        let title = if bookmark.title.is_empty() {
            "Untitled".to_string()
        } else {
            bookmark.title.clone()
        };

        let mut article = Article::new(id, ArticleSource::Instapaper, url, title);

        // Set excerpt from description
        article.excerpt = bookmark.description;

        // Parse tags (comma-separated)
        if !bookmark.tags.is_empty() {
            for tag in bookmark.tags.split(',') {
                let trimmed = tag.trim();
                if !trimmed.is_empty() {
                    article.add_tag(trimmed.to_string());
                }
            }
        }

        // Parse status based on folder and starred
        article.status = if bookmark.starred == "1" {
            ArticleStatus::Favorited
        } else {
            ArticleStatus::Unread
        };

        // Parse timestamp
        if let Some(dt) = DateTime::from_timestamp(bookmark.time, 0) {
            article.added_at = dt;
            article.updated_at = article.added_at;
        }

        // Set reading progress if available
        if let Some(progress) = bookmark.progress {
            article.progress.update_percentage(progress);
        }

        // Store original metadata
        article.source_metadata.insert(
            "hash".to_string(),
            serde_json::Value::String(bookmark.hash),
        );
        if let Some(folder) = bookmark.folder {
            article.source_metadata.insert(
                "folder".to_string(),
                serde_json::Value::String(folder),
            );
        }

        // Extract domain
        if let Ok(parsed_url) = url::Url::parse(&bookmark.url) {
            article.domain = parsed_url.host_str().map(|h| h.to_string());
        }

        Ok(article)
    }

    /// Add a new bookmark
    pub fn add_bookmark(
        &self,
        url: &str,
        title: Option<&str>,
        folder_id: Option<i64>,
    ) -> Result<Article, Error> {
        let mut params = HashMap::new();
        params.insert("url".to_string(), url.to_string());

        if let Some(t) = title {
            params.insert("title".to_string(), t.to_string());
        }

        if let Some(id) = folder_id {
            params.insert("folder_id".to_string(), id.to_string());
        }

        let response = self.make_request("bookmarks/add", Some(params))?;

        let bookmark: InstapaperBookmark =
            serde_json::from_value(response).context("Failed to parse bookmark add response")?;

        self.convert_bookmark_to_article(bookmark)
    }

    /// Delete a bookmark
    pub fn delete_bookmark(&self, bookmark_id: i64) -> Result<(), Error> {
        let mut params = HashMap::new();
        params.insert("bookmark_id".to_string(), bookmark_id.to_string());

        self.make_request("bookmarks/delete", Some(params))?;
        Ok(())
    }

    /// Archive a bookmark (move to archive folder)
    pub fn archive_bookmark(&self, bookmark_id: i64) -> Result<(), Error> {
        let mut params = HashMap::new();
        params.insert("bookmark_id".to_string(), bookmark_id.to_string());

        self.make_request("bookmarks/archive", Some(params))?;
        Ok(())
    }

    /// Un-archive a bookmark (move back to unread)
    pub fn unarchive_bookmark(&self, bookmark_id: i64) -> Result<(), Error> {
        let mut params = HashMap::new();
        params.insert("bookmark_id".to_string(), bookmark_id.to_string());

        self.make_request("bookmarks/unarchive", Some(params))?;
        Ok(())
    }

    /// Star (favorite) a bookmark
    pub fn star_bookmark(&self, bookmark_id: i64) -> Result<(), Error> {
        let mut params = HashMap::new();
        params.insert("bookmark_id".to_string(), bookmark_id.to_string());

        self.make_request("bookmarks/star", Some(params))?;
        Ok(())
    }

    /// Unstar a bookmark
    pub fn unstar_bookmark(&self, bookmark_id: i64) -> Result<(), Error> {
        let mut params = HashMap::new();
        params.insert("bookmark_id".to_string(), bookmark_id.to_string());

        self.make_request("bookmarks/unstar", Some(params))?;
        Ok(())
    }

    /// Move bookmark to folder
    pub fn move_bookmark(&self, bookmark_id: i64, folder_id: i64) -> Result<(), Error> {
        let mut params = HashMap::new();
        params.insert("bookmark_id".to_string(), bookmark_id.to_string());
        params.insert("folder_id".to_string(), folder_id.to_string());

        self.make_request("bookmarks/move", Some(params))?;
        Ok(())
    }

    /// Update reading progress
    pub fn update_progress(
        &self,
        bookmark_id: i64,
        progress: f32,
        progress_timestamp: Option<i64>,
    ) -> Result<(), Error> {
        let mut params = HashMap::new();
        params.insert("bookmark_id".to_string(), bookmark_id.to_string());
        params.insert("progress".to_string(), progress.to_string());

        if let Some(ts) = progress_timestamp {
            params.insert("progress_timestamp".to_string(), ts.to_string());
        }

        self.make_request("bookmarks/update_read_progress", Some(params))?;
        Ok(())
    }

    /// Get highlights for a bookmark
    pub fn get_highlights(&self, bookmark_id: i64) -> Result<Vec<InstapaperHighlight>, Error> {
        let mut params = HashMap::new();
        params.insert("bookmark_id".to_string(), bookmark_id.to_string());

        let response = self.make_request("bookmarks/get_highlights", Some(params))?;

        let highlights: Vec<InstapaperHighlight> =
            serde_json::from_value(response).context("Failed to parse highlights response")?;

        Ok(highlights)
    }

    /// Add a highlight
    pub fn add_highlight(
        &self,
        bookmark_id: i64,
        text: &str,
        note: Option<&str>,
    ) -> Result<InstapaperHighlight, Error> {
        let mut params = HashMap::new();
        params.insert("bookmark_id".to_string(), bookmark_id.to_string());
        params.insert("text".to_string(), text.to_string());

        if let Some(n) = note {
            params.insert("note".to_string(), n.to_string());
        }

        let response = self.make_request("highlights/add", Some(params))?;

        let highlight: InstapaperHighlight =
            serde_json::from_value(response).context("Failed to parse highlight add response")?;

        Ok(highlight)
    }

    /// Delete a highlight
    pub fn delete_highlight(&self, highlight_id: i64) -> Result<(), Error> {
        let mut params = HashMap::new();
        params.insert("highlight_id".to_string(), highlight_id.to_string());

        self.make_request("highlights/delete", Some(params))?;
        Ok(())
    }

    /// Get text for a bookmark (full article content)
    pub fn get_text(&self, bookmark_id: i64) -> Result<String, Error> {
        let mut params = HashMap::new();
        params.insert("bookmark_id".to_string(), bookmark_id.to_string());

        let response = self.make_request("bookmarks/get_text", Some(params))?;

        // Response is typically HTML content
        if let Some(text) = response.as_str() {
            Ok(text.to_string())
        } else {
            bail!("Unexpected response format from get_text")
        }
    }

    /// Get statistics about Instapaper account
    pub fn get_stats(&self) -> Result<InstapaperStats, Error> {
        let all_bookmarks = self.fetch_all_bookmarks()?;
        let folders = self.list_folders()?;

        let unread_count = all_bookmarks
            .iter()
            .filter(|a| a.status == ArticleStatus::Unread)
            .count();

        let archived_count = all_bookmarks
            .iter()
            .filter(|a| a.status == ArticleStatus::Archived)
            .count();

        let starred_count = all_bookmarks
            .iter()
            .filter(|a| a.status == ArticleStatus::Favorited)
            .count();

        let total_words: u64 = all_bookmarks
            .iter()
            .filter_map(|a| a.word_count)
            .map(|w| w as u64)
            .sum();

        let unique_tags: std::collections::HashSet<_> = all_bookmarks
            .iter()
            .flat_map(|a| &a.tags)
            .cloned()
            .collect();

        Ok(InstapaperStats {
            total_bookmarks: all_bookmarks.len(),
            unread_count,
            archived_count,
            starred_count,
            total_words,
            folder_count: folders.len(),
            tag_count: unique_tags.len(),
        })
    }
}

/// Statistics for Instapaper account
#[derive(Debug, Clone)]
pub struct InstapaperStats {
    pub total_bookmarks: usize,
    pub unread_count: usize,
    pub archived_count: usize,
    pub starred_count: usize,
    pub total_words: u64,
    pub folder_count: usize,
    pub tag_count: usize,
}

/// Sync result for Instapaper
#[derive(Debug, Clone)]
pub struct InstapaperSyncResult {
    pub articles: Vec<Article>,
    pub new_articles: Vec<Article>,
    pub updated_articles: Vec<Article>,
    pub archived_articles: Vec<String>,
    pub deleted_articles: Vec<String>,
}

/// High-level Instapaper sync manager
pub struct InstapaperSyncManager {
    client: InstapaperClient,
}

impl InstapaperSyncManager {
    /// Create new sync manager
    pub fn new(client: InstapaperClient) -> Self {
        Self { client }
    }

    /// Create from settings
    pub fn from_settings(settings: &InstapaperSettings) -> Result<Self, Error> {
        let client = InstapaperClient::new(settings)?;
        Ok(Self::new(client))
    }

    /// Perform full sync
    pub fn sync(&self, collection: &mut ArticleCollection) -> Result<InstapaperSyncResult, Error> {
        let articles = self.client.fetch_all_bookmarks()?;

        let mut new_articles = Vec::new();
        let mut updated_articles = Vec::new();
        let mut archived_ids = Vec::new();

        for article in &articles {
            match article.status {
                ArticleStatus::Archived => {
                    archived_ids.push(article.id.clone());
                    if let Some(existing) = collection.get_mut(&article.id) {
                        existing.archive();
                    } else {
                        new_articles.push(article.clone());
                        collection.add(article.clone());
                    }
                }
                _ => {
                    if collection.get(&article.id).is_some() {
                        updated_articles.push(article.clone());
                        collection.update(article.clone());
                    } else {
                        new_articles.push(article.clone());
                        collection.add(article.clone());
                    }
                }
            }
        }

        Ok(InstapaperSyncResult {
            articles,
            new_articles,
            updated_articles,
            archived_articles: archived_ids,
            deleted_articles: Vec::new(),
        })
    }

    /// Archive and sync
    pub fn archive_and_sync(&self, collection: &mut ArticleCollection, bookmark_id: i64) -> Result<(), Error> {
        self.client.archive_bookmark(bookmark_id)?;

        if let Some(article) = collection.get_mut(&bookmark_id.to_string()) {
            article.archive();
        }

        Ok(())
    }

    /// Star and sync
    pub fn star_and_sync(&self, collection: &mut ArticleCollection, bookmark_id: i64) -> Result<(), Error> {
        self.client.star_bookmark(bookmark_id)?;

        if let Some(article) = collection.get_mut(&bookmark_id.to_string()) {
            article.favorite();
        }

        Ok(())
    }
}

/// Legacy compatibility function - sync Instapaper articles if auto_sync is enabled
pub fn sync_instapaper(settings: &InstapaperSettings) -> Result<(), Error> {
    if !settings.auto_sync {
        return Ok(());
    }

    let client = InstapaperClient::new(settings)?;
    let unread = client.fetch_unread_bookmarks()?;

    log_info!("Instapaper sync: fetched {} unread articles", unread.len());

    Ok(())
}

/// Validate Instapaper settings
pub fn validate_instapaper_settings(settings: &InstapaperSettings) -> Result<(), Error> {
    if let Some(username) = &settings.username {
        if username.is_empty() {
            bail!("Instapaper username cannot be empty");
        }
    }

    if let Some(password) = &settings.password {
        if password.is_empty() {
            bail!("Instapaper password cannot be empty");
        }
    }

    // If auto_sync is enabled, both credentials are required
    if settings.auto_sync {
        if settings.username.is_none() {
            bail!("Instapaper username is required when auto-sync is enabled");
        }
        if settings.password.is_none() {
            bail!("Instapaper password is required when auto-sync is enabled");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instapaper_auth() {
        let auth = InstapaperAuth::new("test@example.com".to_string(), "password123".to_string());
        assert!(auth.validate().is_ok());
    }

    #[test]
    fn test_instapaper_auth_empty() {
        let auth = InstapaperAuth::new("".to_string(), "password".to_string());
        assert!(auth.validate().is_err());

        let auth = InstapaperAuth::new("user".to_string(), "".to_string());
        assert!(auth.validate().is_err());
    }

    #[test]
    fn test_validate_settings() {
        let mut settings = InstapaperSettings::default();
        settings.username = Some("test@example.com".to_string());
        settings.password = Some("password123".to_string());
        settings.auto_sync = true;

        assert!(validate_instapaper_settings(&settings).is_ok());

        // Test empty username
        settings.username = Some("".to_string());
        assert!(validate_instapaper_settings(&settings).is_err());
    }

    #[test]
    fn test_instapaper_stats() {
        let stats = InstapaperStats {
            total_bookmarks: 100,
            unread_count: 50,
            archived_count: 45,
            starred_count: 5,
            total_words: 100000,
            folder_count: 3,
            tag_count: 20,
        };

        assert_eq!(stats.total_bookmarks, 100);
        assert_eq!(stats.unread_count, 50);
        assert_eq!(stats.folder_count, 3);
    }
}
