use crate::log_info;
use crate::settings::BackgroundSyncSettings;
use anyhow::{format_err, Context, Error};
use rustc_hash::FxBuildHasher;
use std::collections::HashSet;
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
        #[cfg(target_os = "ios")]
        {
            // iOS manages WiFi automatically, assume connected for sync
            return true;
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
        #[cfg(target_os = "ios")]
        {
            // iOS manages WiFi automatically
        }
        Ok(())
    }

    pub #[cfg(target_os = "ios")]
        {
            // iOS manages WiFi automatically
        }
        fn disable_wifi() -> Result<(), Error> {
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

    pub fn wifi_only(&self) -> bool {
        self.settings.wifi_only
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

/// Builds and executes a curl command safely using direct argument passing,
/// avoiding shell interpretation that could lead to command injection.
fn run_curl_command(
    url: &str,
    username: Option<&str>,
    password: Option<&str>,
    extra_args: &[&str],
) -> Result<std::process::Output, Error> {
    let mut cmd = Command::new("curl");

    for arg in extra_args {
        // Skip shell redirections that don't apply to direct execution
        if !arg.starts_with("2>") {
            cmd.arg(arg);
        }
    }

    if let (Some(user), Some(pass)) = (username, password) {
        cmd.arg("-u").arg(format!("{}:{}", user, pass));
    }

    cmd.arg(url);
    cmd.output()
        .map_err(|e| format_err!("Failed to execute curl: {}", e))
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

        let output = run_curl_command(
            &full_url,
            username,
            password,
            &["-s", "-X", "PROPFIND", "-H", "Depth: 1"],
        )
        .map_err(|e| format_err!("WebDAV sync failed: {}", e))?;

        let response = String::from_utf8_lossy(&output.stdout);

        if response.contains("<d:response>") {
            log_info!("WebDAV: Connected to server, sync available");
        }
    }
    #[cfg(target_os = "ios")]
    {
        // WebDAV sync not yet implemented for iOS
        log_info!("WebDAV sync not available on iOS");
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

        let output = run_curl_command(
            &full_url,
            username,
            password,
            &["-s", "-X", "PROPFIND", "-H", "Depth: 1"],
        )
        .map_err(|e| format_err!("WebDAV list failed: {}", e))?;

        let response = String::from_utf8_lossy(&output.stdout);
        // Pre-allocate files vector with estimated capacity to reduce reallocations
        let mut files = Vec::with_capacity(64);

        let re = regex::Regex::new(r"<d:href>([^<]+)</d:href>").expect("invalid WebDAV href regex");
        for cap in re.captures_iter(&response) {
            if let Some(m) = cap.get(1) {
                let href = m.as_str();
                if !href.ends_with("/") && !href.contains(".metadata.json") {
                    let filename = std::path::Path::new(href)
                        .file_name()
                        .and_then(|f| f.to_str())
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
    #[cfg(target_os = "ios")]
    {
        // WebDAV not yet implemented for iOS
        Ok(vec![])
    }
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
        let local_path_str = local_path.display().to_string();

        run_curl_command(&full_url, username, password, &["-o", &local_path_str])
            .map_err(|e| format_err!("Download failed: {}", e))?;
    }
    #[cfg(target_os = "ios")]
    {
        // WebDAV download not yet implemented for iOS
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
        let local_path_str = local_path.display().to_string();

        run_curl_command(&full_url, username, password, &["-T", &local_path_str])
            .map_err(|e| format_err!("Upload failed: {}", e))?;
    }
    #[cfg(target_os = "ios")]
    {
        // WebDAV upload not yet implemented for iOS
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
        std::fs::create_dir_all(&annotations_dir).with_context(|| {
            format!(
                "Failed to create annotations directory: {}",
                annotations_dir.display()
            )
        })?;

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
    #[cfg(target_os = "ios")]
    {
        // WebDAV annotations sync not yet implemented for iOS
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

        let output = run_curl_command(&full_url, username, password, &["-s"])
            .map_err(|e| format_err!("Fetch failed: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(target_os = "ios")]
    {
        Ok(String::new())
    }
    #[cfg(not(any(target_os = "linux", target_os = "ios")))]
    Ok(String::new())
}

fn merge_json(local: &str, remote: &str) -> String {
    let local_val: serde_json::Value =
        serde_json::from_str(local).unwrap_or(serde_json::Value::Array(Vec::new()));
    let remote_val: serde_json::Value =
        serde_json::from_str(remote).unwrap_or(serde_json::Value::Array(Vec::new()));

    // Pre-allocate merged vector with estimated capacity to reduce reallocations
    let mut merged = Vec::with_capacity(32);
    let mut seen = HashSet::with_hasher(FxBuildHasher);

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
    #[cfg(target_os = "ios")]
    {
        // Reading progress sync not yet implemented for iOS
    }
    Ok(())
}

fn fetch_kobocloud_sync_status(device_id: &str) -> Result<serde_json::Value, Error> {
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
            .post(format!("{}/syncStatus", api_url))
            .json(&device_info)
            .send()
            .map_err(|e| format_err!("KoboCloud sync failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format_err!("KoboCloud API error: {}", response.status()));
        }

        let sync_data: serde_json::Value = response
            .json()
            .map_err(|e| format_err!("Failed to parse sync response: {}", e))?;

        Ok(sync_data)
    }
    #[cfg(target_os = "ios")]
    {
        Err(format_err!("KoboCloud sync not available on iOS"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "ios")))]
    Err(format_err!("KoboCloud sync only available on Linux"))
}

fn process_kobocloud_books(
    sync_data: &serde_json::Value,
    reading_states_dir: &std::path::Path,
) -> Result<(), Error> {
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
                        std::fs::write(reading_state_file, serde_json::to_string_pretty(&state)?)
                            .with_context(|| {
                            format!("Failed to write reading state file: {}", reading_state_file)
                        })?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn prepare_upload_data(
    device_id: &str,
    reading_states_dir: &std::path::Path,
) -> Result<serde_json::Value, Error> {
    let mut upload_data = serde_json::json!({
        "DeviceId": device_id,
        "Books": [],
    });

    if reading_states_dir.exists() {
        // Pre-allocate books vector with estimated capacity to reduce reallocations
        let mut books_to_upload = Vec::with_capacity(16);
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

    Ok(upload_data)
}

fn upload_to_kobocloud(_device_id: &str, upload_data: &serde_json::Value) -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        let api_url = "https://api.kobobooks.com/v1";
        let client = reqwest::blocking::Client::new();

        client
            .post(format!("{}/sync", api_url))
            .json(upload_data)
            .send()
            .map_err(|e| format_err!("KoboCloud upload failed: {}", e))?;
    }
    #[cfg(target_os = "ios")]
    {
        return Err(format_err!("KoboCloud sync not available on iOS"));
    }
    #[cfg(not(any(target_os = "linux", target_os = "ios")))]
    {
        return Err(format_err!("KoboCloud sync only available on Linux"));
    }

    Ok(())
}

pub fn sync_with_kobocloud(
    device_id: &str,
    _local_library_path: &std::path::Path,
    reading_states_dir: &std::path::Path,
) -> Result<(), Error> {
    let sync_data = fetch_kobocloud_sync_status(device_id)?;
    process_kobocloud_books(&sync_data, reading_states_dir)?;

    let upload_data = prepare_upload_data(device_id, reading_states_dir)?;
    upload_to_kobocloud(device_id, &upload_data)?;

    Ok(())
}
