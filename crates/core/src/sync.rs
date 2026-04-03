use crate::settings::BackgroundSyncSettings;
use anyhow::{format_err, Error};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct BackgroundSync {
    settings: BackgroundSyncSettings,
    last_sync: Option<Instant>,
    running: Arc<AtomicBool>,
}

impl BackgroundSync {
    pub fn new(settings: &BackgroundSyncSettings) -> BackgroundSync {
        BackgroundSync {
            settings: settings.clone(),
            last_sync: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_wifi_connected() -> bool {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("wpa_cli").arg("status").output() {
                let status = String::from_utf8_lossy(&output.stdout);
                return status.contains("CONNECTED");
            }
        }
        false
    }

    pub fn enable_wifi() -> Result<(), Error> {
        #[cfg(target_os = "linux")]
        {
            Command::new("sh")
                .arg("-c")
                .arg("connmanctl enable wifi")
                .output()
                .map_err(|e| format_err!("Failed to enable WiFi: {}", e))?;
        }
        Ok(())
    }

    pub fn disable_wifi() -> Result<(), Error> {
        #[cfg(target_os = "linux")]
        {
            Command::new("sh")
                .arg("-c")
                .arg("connmanctl disable wifi")
                .output()
                .map_err(|e| format_err!("Failed to disable WiFi: {}", e))?;
        }
        Ok(())
    }

    pub fn sync_needed(&self) -> bool {
        if !self.settings.enabled {
            return false;
        }

        if self.settings.wifi_only && !Self::is_wifi_connected() {
            return false;
        }

        if let Some(last) = self.last_sync {
            let interval = Duration::from_secs(self.settings.sync_interval_minutes as u64 * 60);
            return last.elapsed() >= interval;
        }

        true
    }

    pub fn should_auto_enable_wifi(&self) -> bool {
        self.settings.enabled && self.settings.auto_wifi
    }

    pub fn should_keep_wifi_on(&self) -> bool {
        self.settings.enabled && self.settings.keep_wifi_on
    }

    pub fn on_book_opened(&mut self) {
        if self.settings.sync_on_open && self.sync_needed() {
            self.trigger_sync();
        }
    }

    pub fn on_book_closed(&mut self) {
        if self.settings.sync_on_close && self.sync_needed() {
            self.trigger_sync();
        }
    }

    pub fn trigger_sync(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        self.last_sync = Some(Instant::now());
    }

    pub fn sync_complete(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.last_sync = Some(Instant::now());
    }

    pub fn is_syncing(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn time_since_last_sync(&self) -> Option<Duration> {
        self.last_sync.map(|last| last.elapsed())
    }

    pub fn update_settings(&mut self, settings: BackgroundSyncSettings) {
        self.settings = settings;
    }
}

impl Default for BackgroundSync {
    fn default() -> Self {
        BackgroundSync::new(&BackgroundSyncSettings::default())
    }
}

pub fn check_network_and_sync(
    cloud_settings: &crate::settings::CloudSyncSettings,
    background_settings: &BackgroundSyncSettings,
) -> Result<(), Error> {
    if !background_settings.enabled {
        return Ok(());
    }

    if background_settings.wifi_only && !BackgroundSync::is_wifi_connected() {
        if background_settings.auto_wifi {
            BackgroundSync::enable_wifi()?;
        } else {
            return Ok(());
        }
    }

    if !cloud_settings.enabled {
        return Ok(());
    }

    let url = cloud_settings
        .url
        .as_ref()
        .ok_or_else(|| format_err!("No WebDAV URL configured"))?;

    sync_with_webdav(
        url,
        cloud_settings.username.as_deref(),
        cloud_settings.password.as_deref(),
        &cloud_settings.remote_path,
    )?;

    if !background_settings.keep_wifi_on && !background_settings.auto_wifi {
        BackgroundSync::disable_wifi()?;
    }

    Ok(())
}

fn sync_with_webdav(
    url: &str,
    username: Option<&str>,
    password: Option<&str>,
    remote_path: &str,
) -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        let base_url = url.trim_end_matches('/');
        let full_url = format!("{}{}", base_url, remote_path);

        let mut curl_cmd = String::from("curl -s -X PROPFIND -H \"Depth: 1\"");

        if let (Some(user), Some(pass)) = (username, password) {
            curl_cmd.push_str(&format!(" -u {}:{}", user, pass));
        }

        curl_cmd.push_str(&format!(" {}", full_url));
        curl_cmd.push_str(" 2>/dev/null");

        let output = Command::new("sh")
            .arg("-c")
            .arg(&curl_cmd)
            .output()
            .map_err(|e| format_err!("WebDAV sync failed: {}", e))?;

        let response = String::from_utf8_lossy(&output.stdout);

        if response.contains("<d:response>") {
            println!("WebDAV: Connected to server, sync available");
        }
    }

    Ok(())
}

pub fn list_webdav_files(
    url: &str,
    username: Option<&str>,
    password: Option<&str>,
    remote_path: &str,
) -> Result<Vec<String>, Error> {
    #[cfg(target_os = "linux")]
    {
        let base_url = url.trim_end_matches('/');
        let full_url = format!("{}{}", base_url, remote_path);

        let mut curl_cmd = String::from("curl -s -X PROPFIND -H \"Depth: 1\"");

        if let (Some(user), Some(pass)) = (username, password) {
            curl_cmd.push_str(&format!(" -u {}:{}", user, pass));
        }

        curl_cmd.push_str(&format!(" {}", full_url));

        let output = Command::new("sh")
            .arg("-c")
            .arg(&curl_cmd)
            .output()
            .map_err(|e| format_err!("WebDAV list failed: {}", e))?;

        let response = String::from_utf8_lossy(&output.stdout);
        let mut files = Vec::new();

        let re = regex::Regex::new(r"<d:href>([^<]+)</d:href>").unwrap();
        for cap in re.captures_iter(&response) {
            if let Some(m) = cap.get(1) {
                let href = m.as_str();
                if !href.ends_with("/") && !href.contains(".metadata.json") {
                    let filename = std::path::Path::new(href)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    if !filename.is_empty() {
                        files.push(filename);
                    }
                }
            }
        }

        Ok(files)
    }

    #[cfg(not(target_os = "linux"))]
    Ok(Vec::new())
}

pub fn download_from_webdav(
    url: &str,
    username: Option<&str>,
    password: Option<&str>,
    remote_path: &str,
    local_path: &std::path::Path,
) -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        let full_url = format!("{}/{}", url.trim_end_matches('/'), remote_path);
        let mut curl_cmd = String::from("curl");

        if let (Some(user), Some(pass)) = (username, password) {
            curl_cmd.push_str(&format!(" -u {}:{}", user, pass));
        }

        curl_cmd.push_str(&format!(" -o {}", local_path.display()));
        curl_cmd.push_str(&format!(" {}", full_url));

        Command::new("sh")
            .arg("-c")
            .arg(&curl_cmd)
            .output()
            .map_err(|e| format_err!("Download failed: {}", e))?;
    }

    Ok(())
}

pub fn upload_to_webdav(
    url: &str,
    username: Option<&str>,
    password: Option<&str>,
    local_path: &std::path::Path,
    remote_path: &str,
) -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        let full_url = format!("{}/{}", url.trim_end_matches('/'), remote_path);
        let mut curl_cmd = String::from("curl -T");

        if let (Some(user), Some(pass)) = (username, password) {
            curl_cmd.push_str(&format!(" -u {}:{}", user, pass));
        }

        curl_cmd.push_str(&format!(" {} {}", local_path.display(), full_url));

        Command::new("sh")
            .arg("-c")
            .arg(&curl_cmd)
            .output()
            .map_err(|e| format_err!("Upload failed: {}", e))?;
    }

    Ok(())
}

pub fn sync_annotations_with_webdav(
    url: &str,
    username: Option<&str>,
    password: Option<&str>,
    remote_base: &str,
    local_library_path: &std::path::Path,
    library_db: &serde_json::Value,
) -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        let annotations_dir = local_library_path.join(".annotations");
        std::fs::create_dir_all(&annotations_dir).ok();

        let remote_annotations_url = format!("{}/.annotations", remote_base.trim_end_matches('/'));

        for (fingerprint, info) in library_db
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid library DB"))?
            .iter()
        {
            if let Some(annotations) = info.get("annotations") {
                if !annotations.as_array().unwrap_or(&Vec::new()).is_empty() {
                    let local_file = annotations_dir.join(format!("{}.json", fingerprint));
                    let remote_file = format!("{}/{}.json", remote_annotations_url, fingerprint);

                    let local_content = if local_file.exists() {
                        std::fs::read_to_string(&local_file).unwrap_or_default()
                    } else {
                        String::new()
                    };

                    let remote_content = fetch_remote_file(url, username, password, &remote_file)
                        .unwrap_or_default();

                    let merged = merge_json(&local_content, &remote_content);
                    std::fs::write(&local_file, &merged)?;

                    upload_to_webdav(url, username, password, &local_file, &remote_file)?;
                }
            }
        }
    }
    Ok(())
}

fn fetch_remote_file(
    url: &str,
    username: Option<&str>,
    password: Option<&str>,
    remote_path: &str,
) -> Result<String, Error> {
    #[cfg(target_os = "linux")]
    {
        let full_url = format!("{}/{}", url.trim_end_matches('/'), remote_path);
        let mut curl_cmd = String::from("curl -s");

        if let (Some(user), Some(pass)) = (username, password) {
            curl_cmd.push_str(&format!(" -u {}:{}", user, pass));
        }

        curl_cmd.push_str(&format!(" {}", full_url));

        let output = Command::new("sh")
            .arg("-c")
            .arg(&curl_cmd)
            .output()
            .map_err(|e| format_err!("Fetch failed: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    #[cfg(not(target_os = "linux"))]
    Ok(String::new())
}

fn merge_json(local: &str, remote: &str) -> String {
    let local_val: serde_json::Value =
        serde_json::from_str(local).unwrap_or(serde_json::Value::Array(Vec::new()));
    let remote_val: serde_json::Value =
        serde_json::from_str(remote).unwrap_or(serde_json::Value::Array(Vec::new()));

    let mut merged = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let empty = Vec::new();
    let local_items = local_val.as_array().unwrap_or(&empty);
    let remote_items = remote_val.as_array().unwrap_or(&empty);

    for item in local_items {
        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
            if !seen.contains(text) {
                seen.insert(text);
                merged.push(item.clone());
            }
        }
    }

    for item in remote_items {
        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
            if !seen.contains(text) {
                seen.insert(text);
                merged.push(item.clone());
            }
        }
    }

    serde_json::to_string_pretty(&merged).unwrap_or_else(|_| "[]".to_string())
}

pub fn sync_reading_progress_with_webdav(
    url: &str,
    username: Option<&str>,
    password: Option<&str>,
    remote_base: &str,
    local_states_dir: &std::path::Path,
) -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        let remote_states_url = format!("{}/.reading-states", remote_base.trim_end_matches('/'));

        if local_states_dir.exists() {
            for entry in std::fs::read_dir(local_states_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    let remote_file = format!("{}/{}", remote_states_url, filename);

                    upload_to_webdav(url, username, password, &path, &remote_file)?;
                }
            }
        }
    }
    Ok(())
}

pub fn sync_with_kobocloud(
    device_id: &str,
    _local_library_path: &std::path::Path,
    reading_states_dir: &std::path::Path,
) -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        let api_url = "https://api.kobobooks.com/v1";

        let client = reqwest::blocking::Client::new();

        let device_info = serde_json::json!({
            "DeviceId": device_id,
            "DeviceType": "KoboPlato",
            "AppVersion": "3.19.0",
        });

        let response = client
            .post(&format!("{}/syncStatus", api_url))
            .json(&device_info)
            .send()
            .map_err(|e| format_err!("KoboCloud sync failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format_err!("KoboCloud API error: {}", response.status()));
        }

        let sync_data: serde_json::Value = response
            .json()
            .map_err(|e| format_err!("Failed to parse sync response: {}", e))?;

        if let Some(books) = sync_data.get("Books").and_then(|b| b.as_array()) {
            for book in books {
                if let Some(book_id) = book.get("BookId").and_then(|b| b.as_str()) {
                    if let Some(progress) = book.get("Progress").and_then(|p| p.as_f64()) {
                        if let Some(reading_state_file) = reading_states_dir
                            .join(format!("{}.json", book_id))
                            .to_str()
                        {
                            let state = serde_json::json!({
                                "progress": progress,
                                "timestamp": chrono::Utc::now().to_rfc3339(),
                            });
                            std::fs::write(
                                reading_state_file,
                                serde_json::to_string_pretty(&state)?,
                            )
                            .ok();
                        }
                    }
                }
            }
        }

        let mut upload_data = serde_json::json!({
            "DeviceId": device_id,
            "Books": [],
        });

        if reading_states_dir.exists() {
            let mut books_to_upload = Vec::new();
            for entry in std::fs::read_dir(reading_states_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(state) = serde_json::from_str::<serde_json::Value>(&content) {
                            let book_id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                            books_to_upload.push(serde_json::json!({
                                "BookId": book_id,
                                "Progress": state.get("progress").and_then(|p| p.as_f64()).unwrap_or(0.0),
                                "LastModified": state.get("timestamp").and_then(|t| t.as_str()).unwrap_or(""),
                            }));
                        }
                    }
                }
            }
            upload_data["Books"] = serde_json::json!(books_to_upload);
        }

        client
            .post(&format!("{}/sync", api_url))
            .json(&upload_data)
            .send()
            .map_err(|e| format_err!("KoboCloud upload failed: {}", e))?;
    }

    #[cfg(not(target_os = "linux"))]
    {
        return Err(format_err!("KoboCloud sync only available on Linux"));
    }

    Ok(())
}
