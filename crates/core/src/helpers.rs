//! Helper utilities module
//!
//! This module provides general-purpose utility functions including:
//! - File system operations (load_json, save_json, load_toml, save_toml)
//! - Character entity decoding (decode_entities)
//! - File fingerprinting (Fingerprint trait, Fp struct)
//! - Path normalization (Normalize trait for Path)
//! - Number-to-words conversion for TTS/accessibility (number_to_words, text_to_words)
//! - BZIP2 compression/decompression (compress_bzip2, decompress_bzip2)
//! - URL encoding/decoding (url_encode, url_decode)
//! - Globset-powered file selection (select_files_by_pattern)
//! - Enhanced HTTP client with retry logic (HttpClient)
//!
//! ## Dependencies
//!
//! - `rustc-hash` - For fast hashing (FxHashMap, FxHashSet)
//! - `num2words` - For number-to-words conversion
//! - `bzip2` - For BZIP2 compression
//! - `percent-encoding` - For URL encoding
//! - `globset` - For glob-based file selection
//! - `reqwest` - For HTTP client with retry logic

use anyhow::{Context, Error};
use bzip2::read::{BzDecoder, BzEncoder};
use entities::ENTITIES;
use globset::{Glob, GlobSetBuilder};
use num2words::Num2Words;
use percent_encoding::{percent_decode, percent_encode, AsciiSet, CONTROLS};
use rustc_hash::FxHashMap;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::char;
use std::fmt;
use std::fs::{self, File, Metadata};
use std::io::{self, BufReader, BufWriter, Read};
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;
use std::sync::LazyLock;
use std::time::SystemTime;
use walkdir::{DirEntry, WalkDir};

/// Log an error message to stderr.
/// Use this instead of raw `eprintln!` for consistent error logging.
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("[ERROR] {}", format!($($arg)*))
    };
}

/// Log a warning message to stderr.
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        eprintln!("[WARN] {}", format!($($arg)*))
    };
}

/// Log an info message to stderr.
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        eprintln!("[INFO] {}", format!($($arg)*))
    };
}

pub static CHARACTER_ENTITIES: LazyLock<FxHashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        let mut m = FxHashMap::default();
        for e in ENTITIES.iter() {
            m.insert(e.entity, e.characters);
        }
        m
    });

/// Walk a directory, filtering hidden files and skipping errors.
/// This is the standard directory traversal pattern for the codebase.
pub fn walkdir_visible(path: &Path) -> impl Iterator<Item = DirEntry> {
    WalkDir::new(path)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| !e.is_hidden())
        .filter_map(|e| e.ok())
}

pub fn decode_entities(text: &str) -> Cow<'_, str> {
    if text.find('&').is_none() {
        return Cow::Borrowed(text);
    }

    let mut cursor = text;
    let mut buf = String::with_capacity(text.len());

    while let Some(start_index) = cursor.find('&') {
        buf.push_str(&cursor[..start_index]);
        cursor = &cursor[start_index..];
        if let Some(end_index) = cursor.find(';') {
            if let Some(repl) = CHARACTER_ENTITIES.get(&cursor[..=end_index]) {
                buf.push_str(repl);
            } else if cursor[1..].starts_with('#') {
                let radix = if cursor[2..].starts_with('x') { 16 } else { 10 };
                let drift_index = 2 + radix as usize / 16;
                if let Some(ch) = u32::from_str_radix(&cursor[drift_index..end_index], radix)
                    .ok()
                    .and_then(char::from_u32)
                {
                    buf.push(ch);
                } else {
                    buf.push_str(&cursor[..=end_index]);
                }
            } else {
                buf.push_str(&cursor[..=end_index]);
            }
            cursor = &cursor[end_index + 1..];
        } else {
            break;
        }
    }

    buf.push_str(cursor);
    Cow::Owned(buf)
}

pub fn load_json<T, P: AsRef<Path>>(path: P) -> Result<T, Error>
where
    for<'a> T: Deserialize<'a>,
{
    let file = File::open(path.as_ref())
        .with_context(|| format!("can't open file {}", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader)
        .with_context(|| format!("can't parse JSON from {}", path.as_ref().display()))
}

pub fn save_json<T, P: AsRef<Path>>(data: &T, path: P) -> Result<(), Error>
where
    T: Serialize,
{
    let file = File::create(path.as_ref())
        .with_context(|| format!("can't create file {}", path.as_ref().display()))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, data)
        .with_context(|| format!("can't serialize to JSON file {}", path.as_ref().display()))?;
    writer
        .into_inner()
        .map_err(|e| io::Error::other(e.to_string()))
        .with_context(|| format!("can't finalize JSON file {}", path.as_ref().display()))?
        .sync_all()
        .with_context(|| format!("can't sync JSON file to disk {}", path.as_ref().display()))
}

pub fn load_toml<T, P: AsRef<Path>>(path: P) -> Result<T, Error>
where
    for<'a> T: Deserialize<'a>,
{
    let s = fs::read_to_string(path.as_ref())
        .with_context(|| format!("can't read file {}", path.as_ref().display()))?;
    toml::from_str(&s)
        .with_context(|| format!("can't parse TOML content from {}", path.as_ref().display()))
}

pub fn save_toml<T, P: AsRef<Path>>(data: &T, path: P) -> Result<(), Error>
where
    T: Serialize,
{
    let s = toml::to_string(data).context("can't convert to TOML format")?;
    let path = path.as_ref();
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, &s)
        .with_context(|| format!("can't write to temp file {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path)
        .with_context(|| format!("can't rename temp file to {}", path.display()))
}

pub trait Fingerprint {
    fn fingerprint(&self, epoch: SystemTime) -> io::Result<Fp>;
}

impl Fingerprint for Metadata {
    fn fingerprint(&self, epoch: SystemTime) -> io::Result<Fp> {
        let m = self
            .modified()?
            .duration_since(epoch)
            .map_or_else(|e| e.duration().as_secs(), |v| v.as_secs());
        Ok(Fp(m.rotate_left(32) ^ self.len()))
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
pub struct Fp(pub u64);

impl Deref for Fp {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Fp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for Fp {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(s, 16).map(Fp)
    }
}

impl From<u64> for Fp {
    fn from(v: u64) -> Self {
        Fp(v)
    }
}

impl fmt::Display for Fp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016X}", self.0)
    }
}

impl Serialize for Fp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct FpVisitor;

impl<'de> Visitor<'de> for FpVisitor {
    type Value = Fp;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::Value::from_str(value)
            .map_err(|e| E::custom(format!("can't parse fingerprint: {}", e)))
    }
}

impl<'de> Deserialize<'de> for Fp {
    fn deserialize<D>(deserializer: D) -> Result<Fp, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FpVisitor)
    }
}

pub trait Normalize: ToOwned {
    fn normalize(&self) -> Self::Owned;
}

impl Normalize for Path {
    fn normalize(&self) -> PathBuf {
        let mut result = PathBuf::default();

        for c in self.components() {
            match c {
                Component::ParentDir => {
                    result.pop();
                }
                Component::CurDir => (),
                _ => result.push(c),
            }
        }

        result
    }
}

pub trait AsciiExtension {
    fn to_alphabetic_digit(self) -> Option<u32>;
}

impl AsciiExtension for char {
    fn to_alphabetic_digit(self) -> Option<u32> {
        if self.is_ascii_uppercase() {
            Some(self as u32 - 65)
        } else {
            None
        }
    }
}

pub mod datetime_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub trait IsHidden {
    fn is_hidden(&self) -> bool;
}

impl IsHidden for DirEntry {
    fn is_hidden(&self) -> bool {
        self.file_name()
            .to_str()
            .is_some_and(|s| s.starts_with('.'))
    }
}

/// Convert a number to English words using num2words
/// Returns the number as words (e.g., 42 -> "forty-two")
pub fn number_to_words(n: u64) -> String {
    Num2Words::new(n)
        .to_words()
        .unwrap_or_else(|_| n.to_string())
}

/// Convert text containing numbers to words using num2words
/// Finds numbers in the text and converts them to words
pub fn text_to_words(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    let mut last_end = 0;

    for (start, number) in find_numbers(text) {
        result.push_str(&text[last_end..start]);
        result.push_str(&number_to_words(number));
        last_end = start + number.to_string().len();
    }

    result.push_str(&text[last_end..]);
    result
}

/// Find all numbers in text, returning (position, value) pairs
fn find_numbers(text: &str) -> Vec<(usize, u64)> {
    let mut numbers = Vec::new();
    let mut current_num = String::new();
    let mut start_pos = 0;

    for (i, ch) in text.char_indices() {
        if ch.is_ascii_digit() {
            if current_num.is_empty() {
                start_pos = i;
            }
            current_num.push(ch);
        } else if !current_num.is_empty() {
            if let Ok(n) = current_num.parse::<u64>() {
                numbers.push((start_pos, n));
            }
            current_num.clear();
        }
    }

    // Handle number at end of string
    if !current_num.is_empty() {
        if let Ok(n) = current_num.parse::<u64>() {
            numbers.push((start_pos, n));
        }
    }

    numbers
}

/// Compress data using BZIP2 algorithm
/// Returns compressed bytes
pub fn compress_bzip2(data: &[u8]) -> Result<Vec<u8>, Error> {
    if data.is_empty() {
        return Err(Error::msg("Cannot compress empty data"));
    }
    use bzip2::Compression;
    let mut encoder = BzEncoder::new(data, Compression::best()); // Maximum compression level
    let mut compressed = Vec::new();
    encoder
        .read_to_end(&mut compressed)
        .context("Failed to compress data with BZIP2")?;
    Ok(compressed)
}

/// Decompress BZIP2 compressed data
/// Returns decompressed bytes
pub fn decompress_bzip2(data: &[u8]) -> Result<Vec<u8>, Error> {
    if data.is_empty() {
        return Err(Error::msg("Cannot decompress empty data"));
    }
    let mut decoder = BzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .context("Failed to decompress BZIP2 data")?;
    Ok(decompressed)
}

/// Compress a file using BZIP2 and write to output path
pub fn compress_file_bzip2<P: AsRef<Path>>(input: P, output: P) -> Result<(), Error> {
    let data = fs::read(input.as_ref())
        .with_context(|| format!("Failed to read file {}", input.as_ref().display()))?;
    let compressed = compress_bzip2(&data)?;
    fs::write(output.as_ref(), compressed).with_context(|| {
        format!(
            "Failed to write compressed file to {}",
            output.as_ref().display()
        )
    })
}

/// Decompress a BZIP2 compressed file and write to output path
pub fn decompress_file_bzip2<P: AsRef<Path>>(input: P, output: P) -> Result<(), Error> {
    let compressed = fs::read(input.as_ref()).with_context(|| {
        format!(
            "Failed to read compressed file {}",
            input.as_ref().display()
        )
    })?;
    let decompressed = decompress_bzip2(&compressed)?;
    fs::write(output.as_ref(), decompressed).with_context(|| {
        format!(
            "Failed to write decompressed file to {}",
            output.as_ref().display()
        )
    })
}

/// URL-safe encoding set (excluding reserved characters for URL structure)
const URL_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'|')
    .add(b'\\')
    .add(b'^')
    .add(b'[')
    .add(b']');

/// Encode a string for safe use in URLs
pub fn url_encode(s: &str) -> String {
    percent_encode(s.as_bytes(), URL_ENCODE_SET).to_string()
}

/// Decode a percent-encoded string
pub fn url_decode(s: &str) -> Result<String, Error> {
    if s.is_empty() {
        return Ok(String::new());
    }
    percent_decode(s.as_bytes())
        .decode_utf8()
        .map(|s| s.to_string())
        .context("Failed to decode URL-encoded string")
}

/// Encode a URL path component
pub fn url_path_encode(s: &str) -> String {
    percent_encode(s.as_bytes(), URL_ENCODE_SET).to_string()
}

/// Decode a URL path component
pub fn url_path_decode(s: &str) -> Result<String, Error> {
    if s.is_empty() {
        return Ok(String::new());
    }
    url_decode(s)
}

/// Format a number as words for UI display (e.g., "123" -> "one hundred twenty-three")
/// Uses number_to_words internally but adds proper capitalization for UI
pub fn format_number_for_ui(n: u64) -> String {
    let s = number_to_words(n);
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Select files matching a glob pattern in a directory
/// Returns matching file paths relative to the base directory
pub fn select_files_by_pattern<P: AsRef<Path>>(
    base_dir: P,
    pattern: &str,
) -> Result<Vec<PathBuf>, Error> {
    let glob = Glob::new(pattern).with_context(|| format!("Invalid glob pattern: {}", pattern))?;

    let mut builder = GlobSetBuilder::new();
    builder.add(glob);
    let glob_set = builder.build().context("Failed to build glob set")?;

    let mut matches = Vec::new();

    for entry in walkdir_visible(base_dir.as_ref()) {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(base_dir.as_ref())
            .with_context(|| format!("Failed to strip prefix from {}", path.display()))?;

        if glob_set.is_match(relative_path) {
            matches.push(relative_path.to_path_buf());
        }
    }

    Ok(matches)
}

/// Select files matching multiple glob patterns
/// Returns files that match any of the provided patterns
pub fn select_files_by_patterns<P: AsRef<Path>>(
    base_dir: P,
    patterns: &[&str],
) -> Result<Vec<PathBuf>, Error> {
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        let glob =
            Glob::new(pattern).with_context(|| format!("Invalid glob pattern: {}", pattern))?;
        builder.add(glob);
    }

    let glob_set = builder.build().context("Failed to build glob set")?;

    let mut matches = Vec::new();

    for entry in walkdir_visible(base_dir.as_ref()) {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(base_dir.as_ref())
            .with_context(|| format!("Failed to strip prefix from {}", path.display()))?;

        if glob_set.is_match(relative_path) {
            matches.push(relative_path.to_path_buf());
        }
    }

    Ok(matches)
}

/// Check if a file matches any of the provided glob patterns
pub fn file_matches_patterns<P: AsRef<Path>>(
    file_path: P,
    patterns: &[&str],
) -> Result<bool, Error> {
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        let glob =
            Glob::new(pattern).with_context(|| format!("Invalid glob pattern: {}", pattern))?;
        builder.add(glob);
    }

    let glob_set = builder.build().context("Failed to build glob set")?;

    let file_name = file_path
        .as_ref()
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name: {}", file_path.as_ref().display()))?;

    Ok(glob_set.is_match(file_name))
}

/// Enhanced HTTP client configuration using reqwest
/// Provides retry logic, timeouts, and proxy support
pub struct HttpClient {
    client: reqwest::blocking::Client,
    max_retries: u32,
}

impl HttpClient {
    /// Create a new HTTP client with default settings
    pub fn new() -> Result<Self, Error> {
        let timeout = std::time::Duration::from_secs(30);
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            max_retries: 3,
        })
    }

    /// Create a new HTTP client with custom settings
    pub fn with_settings(max_retries: u32, timeout_seconds: u64) -> Result<Self, Error> {
        let max_retries = max_retries.clamp(1, 10);
        let timeout_seconds = timeout_seconds.clamp(1, 300);
        let timeout = std::time::Duration::from_secs(timeout_seconds);
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            max_retries,
        })
    }

    /// Create a new HTTP client with proxy support
    pub fn with_proxy(proxy_url: &str) -> Result<Self, Error> {
        let proxy = reqwest::Proxy::all(proxy_url).context("Failed to parse proxy URL")?;

        let timeout = std::time::Duration::from_secs(30);
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .proxy(proxy)
            .build()
            .context("Failed to build HTTP client with proxy")?;

        Ok(Self {
            client,
            max_retries: 3,
        })
    }

    /// Fetch a URL with retry logic
    pub fn fetch_with_retry(&self, url: &str) -> Result<String, Error> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.client.get(url).send() {
                Ok(response) => {
                    if response.status().is_success() {
                        let text = response.text().context("Failed to read response body")?;
                        return Ok(text);
                    } else {
                        let status = response.status();
                        last_error = Some(anyhow::anyhow!("HTTP error: {}", status));
                    }
                }
                Err(e) => {
                    last_error = Some(Error::from(e));
                }
            }

            if attempt < self.max_retries {
                std::thread::sleep(std::time::Duration::from_millis(100 * (attempt + 1) as u64));
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Failed to fetch URL after retries")))
    }

    /// Fetch a URL as bytes with retry logic
    pub fn fetch_bytes_with_retry(&self, url: &str) -> Result<Vec<u8>, Error> {
        if url.is_empty() {
            return Err(Error::msg("URL cannot be empty"));
        }
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.client.get(url).send() {
                Ok(response) => {
                    if response.status().is_success() {
                        let bytes = response.bytes().context("Failed to read response body")?;
                        return Ok(bytes.to_vec());
                    } else {
                        let status = response.status();
                        last_error = Some(anyhow::anyhow!("HTTP error: {}", status));
                    }
                }
                Err(e) => {
                    last_error = Some(Error::from(e));
                }
            }

            if attempt < self.max_retries {
                std::thread::sleep(std::time::Duration::from_millis(100 * (attempt + 1) as u64));
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Failed to fetch URL after retries")))
    }

    /// Get the underlying reqwest client
    pub fn client(&self) -> &reqwest::blocking::Client {
        &self.client
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

/// XDG Base Directory path resolution for desktop Linux
///
/// Provides functions to resolve paths following the XDG Base Directory Specification
/// for proper integration with Linux desktop environments.
pub mod xdg {
    use std::env;
    use std::path::{Path, PathBuf};

    /// Get the XDG config directory for Plato
    pub fn config_dir() -> PathBuf {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| dirs_next::home_dir().unwrap_or_default().join(".config"))
            .join("plato")
    }

    /// Get the XDG data directory for Plato
    pub fn data_dir() -> PathBuf {
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs_next::home_dir()
                    .unwrap_or_default()
                    .join(".local/share")
            })
            .join("plato")
    }

    /// Get the system data directory for installed resources
    pub fn system_data_dir() -> PathBuf {
        PathBuf::from("/usr/share/plato")
    }

    /// Check if running from source (development mode)
    /// Returns true if Settings.toml exists in current directory
    pub fn is_development_mode() -> bool {
        Path::new("Settings.toml").exists() || Path::new("fonts").exists()
    }

    /// Resolve a resource path
    /// Checks in order: current dir (dev), XDG data dir, system data dir
    pub fn resolve_resource_path(relative_path: &str) -> PathBuf {
        if is_development_mode() {
            PathBuf::from(relative_path)
        } else {
            let xdg_path = data_dir().join(relative_path);
            if xdg_path.exists() {
                xdg_path
            } else {
                system_data_dir().join(relative_path)
            }
        }
    }

    /// Resolve the settings file path
    pub fn settings_path() -> PathBuf {
        if is_development_mode() {
            PathBuf::from("Settings.toml")
        } else {
            config_dir().join("Settings.toml")
        }
    }

    /// Resolve the library/data path
    pub fn library_path() -> PathBuf {
        if is_development_mode() {
            PathBuf::from(".")
        } else {
            data_dir()
        }
    }

    /// Ensure XDG directories exist
    pub fn ensure_xdg_dirs() -> std::io::Result<()> {
        std::fs::create_dir_all(config_dir())?;
        std::fs::create_dir_all(data_dir())?;
        Ok(())
    }
}
