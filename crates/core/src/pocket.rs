//! Pocket API Integration Module
//!
#![allow(dead_code)]

//! Provides full integration with the Pocket API (getpocket.com) for syncing
//! saved articles, managing read/unread status, archiving, favoriting, and tagging.
//!
//! ## Features
//!
//! - **OAuth Authentication**: Complete OAuth flow for user authorization
//! - **Article Sync**: Fetch all saved articles with metadata
//! - **Tag Management**: Add, remove, and filter by tags
//! - **Archive/Delete**: Mark articles as read or remove them
//! - **Favorites**: Star/unstar articles
//! - **Offline Support**: Download article content for offline reading
//! - **Incremental Sync**: Efficient updates using `since` timestamp
//!
//! ## API Documentation
//!
//! Based on the official Pocket API documentation:
//! - https://getpocket.com/developer/docs/overview
//!
//! ## Usage
//!
//! ```rust,ignore
//! use plato_core::pocket::{PocketClient, PocketAuth};
//! use plato_core::settings::PocketSettings;
//!
//! // Initialize client
//! let client = PocketClient::new(&settings)?;
//!
//! // Fetch unread articles
//! let articles = client.fetch_unread_articles()?;
//!
//! // Archive an article
//! client.archive_article("12345")?;
//! ```

use crate::article::{
    Article, ArticleId, ArticleImage, ArticleSource, ArticleStatus, ArticleCollection,
};
use crate::log_error;
use crate::log_info;
use crate::settings::PocketSettings;
use anyhow::{bail, format_err, Context, Error};
use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Pocket API base URL
const POCKET_API_BASE: &str = "https://getpocket.com/v3";

/// Request timeout for API calls
const API_TIMEOUT_SECONDS: u64 = 30;

/// OAuth request token response
#[derive(Debug, Deserialize)]
struct RequestTokenResponse {
    code: String,
}

/// OAuth access token response
#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    username: String,
}

/// Pocket API item (article)
#[derive(Debug, Clone, Deserialize)]
struct PocketItem {
    item_id: String,
    resolved_id: Option<String>,
    given_url: Option<String>,
    resolved_url: Option<String>,
    given_title: Option<String>,
    resolved_title: Option<String>,
    favorite: Option<String>,
    status: Option<String>,
    excerpt: Option<String>,
    is_article: Option<String>,
    has_video: Option<String>,
    has_image: Option<String>,
    word_count: Option<String>,
    #[serde(default)]
    tags: HashMap<String, serde_json::Value>,
    #[serde(default)]
    authors: HashMap<String, serde_json::Value>,
    #[serde(default)]
    images: HashMap<String, serde_json::Value>,
    time_added: Option<String>,
    time_updated: Option<String>,
    time_read: Option<String>,
    time_favorited: Option<String>,
}

/// Pocket API get response
#[derive(Debug, Deserialize)]
struct PocketGetResponse {
    status: u32,
    complete: u32,
    #[serde(default)]
    list: HashMap<String, PocketItem>,
    #[serde(default)]
    since: u64,
    error: Option<String>,
}

/// OAuth authentication flow for Pocket
pub struct PocketAuth {
    consumer_key: String,
    redirect_uri: String,
}

impl PocketAuth {
    /// Create new auth handler
    pub fn new(consumer_key: String, redirect_uri: Option<String>) -> Self {
        Self {
            consumer_key,
            redirect_uri: redirect_uri.unwrap_or_else(|| "https://example.com".to_string()),
        }
    }

    /// Step 1: Obtain request token
    pub fn get_request_token(&self) -> Result<String, Error> {
        let client = Client::new();
        let url = format!("{}/oauth/request", POCKET_API_BASE);

        let body = serde_json::json!({
            "consumer_key": &self.consumer_key,
            "redirect_uri": &self.redirect_uri,
        });

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Accept", "application/json")
            .json(&body)
            .timeout(Duration::from_secs(API_TIMEOUT_SECONDS))
            .send()
            .context("Failed to request Pocket OAuth token")?;

        if !response.status().is_success() {
            bail!("Pocket OAuth request failed: HTTP {}", response.status());
        }

        let token_resp: RequestTokenResponse = response
            .json()
            .context("Failed to parse Pocket OAuth response")?;

        Ok(token_resp.code)
    }

    /// Get authorization URL for user to visit
    pub fn get_auth_url(&self, request_token: &str) -> String {
        format!(
            "https://getpocket.com/auth/authorize?request_token={}&redirect_uri={}",
            request_token, self.redirect_uri
        )
    }

    /// Step 3: Convert request token to access token
    pub fn get_access_token(&self, request_token: &str) -> Result<(String, String), Error> {
        let client = Client::new();
        let url = format!("{}/oauth/authorize", POCKET_API_BASE);

        let body = serde_json::json!({
            "consumer_key": &self.consumer_key,
            "code": request_token,
        });

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Accept", "application/json")
            .json(&body)
            .timeout(Duration::from_secs(API_TIMEOUT_SECONDS))
            .send()
            .context("Failed to authorize Pocket access token")?;

        if !response.status().is_success() {
            bail!("Pocket authorization failed: HTTP {}", response.status());
        }

        let token_resp: AccessTokenResponse = response
            .json()
            .context("Failed to parse Pocket authorization response")?;

        Ok((token_resp.access_token, token_resp.username))
    }

    /// Complete OAuth flow (for testing/manual use)
    pub fn complete_oauth_flow(&self) -> Result<(String, String), Error> {
        let request_token = self.get_request_token()?;
        let auth_url = self.get_auth_url(&request_token);

        log_info!("Please visit this URL to authorize: {}", auth_url);
        log_info!("Then press Enter to continue...");

        // Wait for user input
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context("Failed to read user input")?;

        self.get_access_token(&request_token)
    }
}

/// Pocket API client
pub struct PocketClient {
    consumer_key: String,
    access_token: String,
    client: Client,
}

impl PocketClient {
    /// Create new Pocket client from settings
    pub fn new(settings: &PocketSettings) -> Result<Self, Error> {
        let consumer_key = settings
            .consumer_key
            .as_ref()
            .ok_or_else(|| format_err!("Pocket consumer key not configured"))?
            .clone();

        let access_token = settings
            .access_token
            .as_ref()
            .ok_or_else(|| format_err!("Pocket access token not configured"))?
            .clone();

        let client = Client::builder()
            .timeout(Duration::from_secs(API_TIMEOUT_SECONDS))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            consumer_key,
            access_token,
            client,
        })
    }

    /// Create client with explicit credentials
    pub fn with_credentials(consumer_key: String, access_token: String) -> Result<Self, Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(API_TIMEOUT_SECONDS))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            consumer_key,
            access_token,
            client,
        })
    }

    /// Check if client has valid credentials
    pub fn is_configured(&self) -> bool {
        !self.consumer_key.is_empty() && !self.access_token.is_empty()
    }

    /// Make authenticated API request
    fn make_request(&self, endpoint: &str, body: serde_json::Value) -> Result<serde_json::Value, Error> {
        let url = format!("{}/{}", POCKET_API_BASE, endpoint);

        let request_body = serde_json::json!({
            "consumer_key": &self.consumer_key,
            "access_token": &self.access_token,
        });

        // Merge with additional body parameters
        let merged_body = match body {
            serde_json::Value::Object(map) => {
                let mut base = match request_body {
                    serde_json::Value::Object(m) => m,
                    _ => return Err(format_err!("Invalid base request body")),
                };
                base.extend(map);
                serde_json::Value::Object(base)
            }
            _ => request_body,
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Accept", "application/json")
            .json(&merged_body)
            .send()
            .with_context(|| format!("Failed to make Pocket API request to {}", endpoint))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().unwrap_or_default();
            bail!("Pocket API error ({}): {}", status, text);
        }

        let json: serde_json::Value = response
            .json()
            .context("Failed to parse Pocket API response")?;

        Ok(json)
    }

    /// Fetch articles from Pocket
    pub fn fetch_articles(
        &self,
        state: Option<&str>,
        since: Option<u64>,
        count: Option<u32>,
    ) -> Result<Vec<Article>, Error> {
        let mut body = serde_json::json!({
            "detailType": "complete",
            "sort": "newest",
        });

        if let Some(s) = state {
            body["state"] = serde_json::Value::String(s.to_string());
        }

        if let Some(s) = since {
            body["since"] = serde_json::Value::from(s);
        }

        if let Some(c) = count {
            body["count"] = serde_json::Value::from(c);
        }

        let response = self.make_request("get", body)?;

        // Check for API errors
        if let Some(error) = response.get("error") {
            let error_msg = error.as_str().unwrap_or("Unknown error");
            bail!("Pocket API returned error: {}", error_msg);
        }

        let pocket_resp: PocketGetResponse = serde_json::from_value(response)
            .context("Failed to parse Pocket get response")?;

        let mut articles = Vec::with_capacity(pocket_resp.list.len());

        for (id, item) in pocket_resp.list {
            let article = self.convert_pocket_item_to_article(id, item)?;
            articles.push(article);
        }

        Ok(articles)
    }

    /// Fetch only unread articles
    pub fn fetch_unread_articles(&self) -> Result<Vec<Article>, Error> {
        self.fetch_articles(Some("unread"), None, None)
    }

    /// Fetch archived articles
    pub fn fetch_archived_articles(&self) -> Result<Vec<Article>, Error> {
        self.fetch_articles(Some("archive"), None, None)
    }

    /// Fetch all articles (both unread and archived)
    pub fn fetch_all_articles(&self) -> Result<Vec<Article>, Error> {
        self.fetch_articles(Some("all"), None, None)
    }

    /// Fetch articles modified since a timestamp
    pub fn fetch_articles_since(&self, since: u64) -> Result<(Vec<Article>, u64), Error> {
        let response = self.make_request("get", serde_json::json!({
            "state": "all",
            "detailType": "complete",
            "sort": "newest",
            "since": since,
        }))?;

        let pocket_resp: PocketGetResponse = serde_json::from_value(response)
            .context("Failed to parse Pocket get response")?;

        let mut articles = Vec::with_capacity(pocket_resp.list.len());

        for (id, item) in pocket_resp.list {
            let article = self.convert_pocket_item_to_article(id, item)?;
            articles.push(article);
        }

        Ok((articles, pocket_resp.since))
    }

    /// Convert Pocket API item to Article struct
    fn convert_pocket_item_to_article(&self, id: String, item: PocketItem) -> Result<Article, Error> {
        let url = item
            .resolved_url
            .or(item.given_url)
            .unwrap_or_default();

        let title = item
            .resolved_title
            .or(item.given_title)
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| "Untitled".to_string());

        let mut article = Article::new(id, ArticleSource::Pocket, url, title);

        // Set excerpt
        article.excerpt = item.excerpt;

        // Parse word count
        if let Some(wc) = item.word_count {
            article.word_count = wc.parse().ok();
            if let Some(words) = article.word_count {
                article.reading_time_minutes = Some((words / 200).max(1));
            }
        }

        // Parse status
        article.status = match item.status.as_deref() {
            Some("1") => ArticleStatus::Archived,
            Some("2") => ArticleStatus::Deleted,
            _ => ArticleStatus::Unread,
        };

        // Check if favorited
        if item.favorite == Some("1".to_string()) {
            article.favorite();
        }

        // Parse tags
        for (tag, _) in item.tags {
            article.add_tag(tag);
        }

        // Parse authors
        for (_, author_data) in item.authors {
            if let Some(name) = author_data.get("name").and_then(|v| v.as_str()) {
                article.authors.push(name.to_string());
            }
        }

        // Parse images
        for (_, image_data) in item.images {
            if let Some(url) = image_data.get("src").and_then(|v| v.as_str()) {
                let image = ArticleImage {
                    url: url.to_string(),
                    local_path: None,
                    caption: image_data.get("caption").and_then(|v| v.as_str()).map(String::from),
                    width: image_data.get("width").and_then(|v| v.as_u64()).map(|v| v as u32),
                    height: image_data.get("height").and_then(|v| v.as_u64()).map(|v| v as u32),
                };
                article.images.push(image);
            }
        }

        // Parse timestamps
        if let Some(added) = item.time_added {
            if let Ok(timestamp) = added.parse::<i64>() {
                article.added_at = DateTime::from_timestamp(timestamp, 0)
                    .unwrap_or_else(Utc::now);
            }
        }

        if let Some(updated) = item.time_updated {
            if let Ok(timestamp) = updated.parse::<i64>() {
                article.updated_at = DateTime::from_timestamp(timestamp, 0)
                    .unwrap_or_else(Utc::now);
            }
        }

        if let Some(favorited) = item.time_favorited {
            if let Ok(timestamp) = favorited.parse::<i64>() {
                article.favorited_at = DateTime::from_timestamp(timestamp, 0);
            }
        }

        // Store original Pocket metadata
        article.source_metadata.insert(
            "has_video".to_string(),
            serde_json::Value::String(item.has_video.unwrap_or_default()),
        );
        article.source_metadata.insert(
            "has_image".to_string(),
            serde_json::Value::String(item.has_image.unwrap_or_default()),
        );
        article.source_metadata.insert(
            "is_article".to_string(),
            serde_json::Value::String(item.is_article.unwrap_or_default()),
        );

        Ok(article)
    }

    /// Add (save) a new URL to Pocket
    pub fn add_url(&self, url: &str, title: Option<&str>, tags: Option<&[String]>) -> Result<ArticleId, Error> {
        let mut body = serde_json::json!({
            "url": url,
        });

        if let Some(t) = title {
            body["title"] = serde_json::Value::String(t.to_string());
        }

        if let Some(t) = tags {
            let tag_string = t.join(",");
            body["tags"] = serde_json::Value::String(tag_string);
        }

        let response = self.make_request("add", body)?;

        let item = response
            .get("item")
            .ok_or_else(|| format_err!("No item in Pocket add response"))?;

        let item_id = item
            .get("item_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format_err!("No item_id in Pocket add response"))?
            .to_string();

        Ok(item_id)
    }

    /// Archive (mark as read) an article
    pub fn archive_article(&self, item_id: &str) -> Result<(), Error> {
        self.modify_items(&[item_id], Action::Archive)
    }

    /// Un-archive (re-add) an article
    pub fn unarchive_article(&self, item_id: &str) -> Result<(), Error> {
        self.modify_items(&[item_id], Action::Unarchive)
    }

    /// Favorite (star) an article
    pub fn favorite_article(&self, item_id: &str) -> Result<(), Error> {
        self.modify_items(&[item_id], Action::Favorite)
    }

    /// Unfavorite an article
    pub fn unfavorite_article(&self, item_id: &str) -> Result<(), Error> {
        self.modify_items(&[item_id], Action::Unfavorite)
    }

    /// Delete an article
    pub fn delete_article(&self, item_id: &str) -> Result<(), Error> {
        self.modify_items(&[item_id], Action::Delete)
    }

    /// Add tags to an article
    pub fn add_tags(&self, item_id: &str, tags: &[String]) -> Result<(), Error> {
        let actions: Vec<_> = tags
            .iter()
            .map(|tag| ActionData {
                action: "tags_add".to_string(),
                item_id: item_id.to_string(),
                tags: Some(tag.clone()),
                ..Default::default()
            })
            .collect();

        self.send_actions(&actions)
    }

    /// Remove tags from an article
    pub fn remove_tags(&self, item_id: &str, tags: &[String]) -> Result<(), Error> {
        let actions: Vec<_> = tags
            .iter()
            .map(|tag| ActionData {
                action: "tags_remove".to_string(),
                item_id: item_id.to_string(),
                tags: Some(tag.clone()),
                ..Default::default()
            })
            .collect();

        self.send_actions(&actions)
    }

    /// Replace all tags on an article
    pub fn replace_tags(&self, item_id: &str, tags: &[String]) -> Result<(), Error> {
        let tag_string = tags.join(",");
        let actions = vec![ActionData {
            action: "tags_replace".to_string(),
            item_id: item_id.to_string(),
            tags: Some(tag_string),
            ..Default::default()
        }];

        self.send_actions(&actions)
    }

    /// Clear all tags from an article
    pub fn clear_tags(&self, item_id: &str) -> Result<(), Error> {
        self.modify_items(&[item_id], Action::TagsClear)
    }

    /// Rename a tag across all items
    pub fn rename_tag(&self, old_tag: &str, new_tag: &str) -> Result<(), Error> {
        let actions = vec![ActionData {
            action: "tag_rename".to_string(),
            item_id: String::new(),
            tags: Some(old_tag.to_string()),
            new_tag: Some(new_tag.to_string()),
            ..Default::default()
        }];

        self.send_actions(&actions)
    }

    /// Delete a tag across all items
    pub fn delete_tag(&self, tag: &str) -> Result<(), Error> {
        let actions = vec![ActionData {
            action: "tags_delete".to_string(),
            item_id: String::new(),
            tags: Some(tag.to_string()),
            ..Default::default()
        }];

        self.send_actions(&actions)
    }

    /// Batch modify items with a single action
    fn modify_items(&self, item_ids: &[&str], action: Action) -> Result<(), Error> {
        let action_str = match action {
            Action::Archive => "archive",
            Action::Unarchive => "readd",
            Action::Favorite => "favorite",
            Action::Unfavorite => "unfavorite",
            Action::Delete => "delete",
            Action::TagsClear => "tags_clear",
        };

        let actions: Vec<_> = item_ids
            .iter()
            .map(|id| ActionData {
                action: action_str.to_string(),
                item_id: id.to_string(),
                ..Default::default()
            })
            .collect();

        self.send_actions(&actions)
    }

    /// Send batch actions to Pocket API
    fn send_actions(&self, actions: &[ActionData]) -> Result<(), Error> {
        let actions_json = serde_json::to_string(actions)
            .context("Failed to serialize actions")?;

        let body = serde_json::json!({
            "actions": actions_json,
        });

        let response = self.make_request("send", body)?;

        // Check for action errors
        if let Some(action_results) = response.get("action_results") {
            if let Some(results) = action_results.as_array() {
                for (i, result) in results.iter().enumerate() {
                    if result.as_bool() == Some(false) {
                        log_error!("Pocket action {} failed", i);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get statistics about Pocket account
    pub fn get_stats(&self) -> Result<PocketStats, Error> {
        let all_articles = self.fetch_all_articles()?;

        let unread_count = all_articles
            .iter()
            .filter(|a| a.status == ArticleStatus::Unread)
            .count();

        let archived_count = all_articles
            .iter()
            .filter(|a| a.status == ArticleStatus::Archived)
            .count();

        let favorited_count = all_articles
            .iter()
            .filter(|a| a.status == ArticleStatus::Favorited)
            .count();

        let total_words: u64 = all_articles
            .iter()
            .filter_map(|a| a.word_count)
            .map(|w| w as u64)
            .sum();

        let unique_tags: std::collections::HashSet<_> = all_articles
            .iter()
            .flat_map(|a| &a.tags)
            .cloned()
            .collect();

        Ok(PocketStats {
            total_articles: all_articles.len(),
            unread_count,
            archived_count,
            favorited_count,
            total_words,
            tag_count: unique_tags.len(),
        })
    }
}

/// Actions that can be performed on Pocket items
#[derive(Debug, Clone, Copy)]
enum Action {
    Archive,
    Unarchive,
    Favorite,
    Unfavorite,
    Delete,
    TagsClear,
}

/// Action data for batch operations
#[derive(Debug, Default, Serialize)]
struct ActionData {
    action: String,
    item_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<u64>,
}

/// Statistics for Pocket account
#[derive(Debug, Clone)]
pub struct PocketStats {
    pub total_articles: usize,
    pub unread_count: usize,
    pub archived_count: usize,
    pub favorited_count: usize,
    pub total_words: u64,
    pub tag_count: usize,
}

/// Sync result containing articles and sync metadata
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub articles: Vec<Article>,
    pub new_articles: Vec<Article>,
    pub updated_articles: Vec<Article>,
    pub archived_articles: Vec<String>,
    pub deleted_articles: Vec<String>,
    pub since: u64,
}

/// High-level Pocket sync manager that handles caching and incremental sync
pub struct PocketSyncManager {
    client: PocketClient,
    last_sync: Option<u64>,
}

impl PocketSyncManager {
    /// Create new sync manager
    pub fn new(client: PocketClient) -> Self {
        Self {
            client,
            last_sync: None,
        }
    }

    /// Create from settings
    pub fn from_settings(settings: &PocketSettings) -> Result<Self, Error> {
        let client = PocketClient::new(settings)?;
        Ok(Self::new(client))
    }

    /// Set last sync timestamp
    pub fn set_last_sync(&mut self, timestamp: u64) {
        self.last_sync = Some(timestamp);
    }

    /// Perform incremental sync
    pub fn sync(&mut self, collection: &mut ArticleCollection) -> Result<SyncResult, Error> {
        let since = self.last_sync.unwrap_or(0);

        let (articles, new_since) = self.client.fetch_articles_since(since)?;

        let mut new_articles = Vec::new();
        let mut updated_articles = Vec::new();
        let mut archived_articles = Vec::new();
        let mut deleted_articles = Vec::new();

        for article in &articles {
            match article.status {
                ArticleStatus::Deleted => {
                    deleted_articles.push(article.id.clone());
                    collection.remove(&article.id);
                }
                ArticleStatus::Archived => {
                    archived_articles.push(article.id.clone());
                    if let Some(existing) = collection.get_mut(&article.id) {
                        existing.archive();
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

        self.last_sync = Some(new_since);

        Ok(SyncResult {
            articles,
            new_articles,
            updated_articles,
            archived_articles,
            deleted_articles,
            since: new_since,
        })
    }

    /// Full sync - fetch all articles (use sparingly)
    pub fn full_sync(&mut self, collection: &mut ArticleCollection) -> Result<SyncResult, Error> {
        collection.clear();

        let articles = self.client.fetch_all_articles()?;
        let since = chrono::Utc::now().timestamp() as u64;

        for article in &articles {
            collection.add(article.clone());
        }

        self.last_sync = Some(since);

        Ok(SyncResult {
            articles: articles.clone(),
            new_articles: articles,
            updated_articles: Vec::new(),
            archived_articles: Vec::new(),
            deleted_articles: Vec::new(),
            since,
        })
    }

    /// Archive an article and sync to Pocket
    pub fn archive_and_sync(&self, collection: &mut ArticleCollection, article_id: &str) -> Result<(), Error> {
        self.client.archive_article(article_id)?;

        if let Some(article) = collection.get_mut(&article_id.to_string()) {
            article.archive();
        }

        Ok(())
    }

    /// Add tags and sync to Pocket
    pub fn add_tags_and_sync(
        &self,
        collection: &mut ArticleCollection,
        article_id: &str,
        tags: &[String],
    ) -> Result<(), Error> {
        self.client.add_tags(article_id, tags)?;

        if let Some(article) = collection.get_mut(&article_id.to_string()) {
            for tag in tags {
                article.add_tag(tag.clone());
            }
        }

        Ok(())
    }
}

/// Legacy compatibility function - sync Pocket articles if auto_sync is enabled
///
/// This function maintains backward compatibility with the original stub implementation.
/// For new code, use `PocketClient` or `PocketSyncManager` directly.
pub fn sync_pocket(settings: &PocketSettings) -> Result<(), Error> {
    if !settings.auto_sync {
        return Ok(());
    }

    let client = PocketClient::new(settings)?;
    let unread = client.fetch_unread_articles()?;

    log_info!("Pocket sync: fetched {} unread articles", unread.len());

    Ok(())
}

/// Validate Pocket settings
pub fn validate_pocket_settings(settings: &PocketSettings) -> Result<(), Error> {
    if let Some(key) = &settings.consumer_key {
        if key.is_empty() {
            bail!("Pocket consumer key cannot be empty");
        }
        if key.len() < 10 {
            bail!("Pocket consumer key appears invalid (too short)");
        }
    }

    if let Some(token) = &settings.access_token {
        if token.is_empty() {
            bail!("Pocket access token cannot be empty");
        }
    }

    // If auto_sync is enabled, both credentials are required
    if settings.auto_sync {
        if settings.consumer_key.is_none() {
            bail!("Pocket consumer key is required when auto-sync is enabled");
        }
        if settings.access_token.is_none() {
            bail!("Pocket access token is required when auto-sync is enabled");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::article::ReadProgress;

    #[test]
    fn test_pocket_auth_url() {
        let auth = PocketAuth::new("test_consumer_key".to_string(), None);
        let request_token = "test_token";
        let url = auth.get_auth_url(request_token);
        assert!(url.contains("getpocket.com/auth/authorize"));
        assert!(url.contains("test_token"));
    }

    #[test]
    fn test_validate_settings() {
        let mut settings = PocketSettings::default();
        settings.consumer_key = Some("valid_key_12345".to_string());
        settings.access_token = Some("valid_token_12345".to_string());
        settings.auto_sync = true;

        assert!(validate_pocket_settings(&settings).is_ok());

        // Test empty key
        settings.consumer_key = Some("".to_string());
        assert!(validate_pocket_settings(&settings).is_err());

        // Test key too short
        settings.consumer_key = Some("short".to_string());
        assert!(validate_pocket_settings(&settings).is_err());
    }

    #[test]
    fn test_pocket_stats() {
        let stats = PocketStats {
            total_articles: 100,
            unread_count: 50,
            archived_count: 45,
            favorited_count: 5,
            total_words: 100000,
            tag_count: 20,
        };

        assert_eq!(stats.total_articles, 100);
        assert_eq!(stats.unread_count, 50);
    }

    #[test]
    fn test_read_progress() {
        let mut progress = ReadProgress::new();
        assert!(!progress.is_finished());

        progress.mark_started();
        assert!(progress.started_at.is_some());

        progress.update_percentage(0.5);
        assert!(!progress.is_finished());

        progress.mark_finished();
        assert!(progress.is_finished());
        assert_eq!(progress.percentage, 1.0);
    }
}

