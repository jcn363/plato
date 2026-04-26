//! iOS storage and path resolution
//!
//! This module provides iOS-specific path resolution for library and settings,
//! taking into account iOS sandboxing and file system structure.

#![cfg(feature = "ios")]
#![deny(warnings)]

use std::path::PathBuf;
use std::sync::Mutex;

/// Global library path set from Swift
static LIBRARY_PATH: Mutex<Option<String>> = Mutex::new(None);

/// Global settings path set from Swift
static SETTINGS_PATH: Mutex<Option<String>> = Mutex::new(None);

/// Global cache path set from Swift
static CACHE_PATH: Mutex<Option<String>> = Mutex::new(None);

/// Global temp path set from Swift
static TEMP_PATH: Mutex<Option<String>> = Mutex::new(None);

/// Set the library path from Swift
/// This should be called during initialization before any path resolution
pub fn set_library_path(path: String) {
    *LIBRARY_PATH.lock().expect("LIBRARY_PATH lock poisoned") = Some(path);
}

/// Set the settings path from Swift
/// This should be called during initialization before any path resolution
pub fn set_settings_path(path: String) {
    *SETTINGS_PATH.lock().expect("SETTINGS_PATH lock poisoned") = Some(path);
}

/// Set the cache path from Swift
/// This should be called during initialization before any path resolution
pub fn set_cache_path(path: String) {
    *CACHE_PATH.lock().expect("CACHE_PATH lock poisoned") = Some(path);
}

/// Set the temp path from Swift
/// This should be called during initialization before any path resolution
pub fn set_temp_path(path: String) {
    *TEMP_PATH.lock().expect("TEMP_PATH lock poisoned") = Some(path);
}

/// Get the default library path for iOS
/// This would typically be in the app's Documents directory
pub fn ios_library_path() -> String {
    // Return the path set from Swift, or derive from home directory as fallback
    if let Some(path) = LIBRARY_PATH
        .lock()
        .expect("LIBRARY_PATH lock poisoned")
        .as_ref()
    {
        return path.clone();
    }

    // Fallback: derive from process home directory
    if let Ok(home) = std::env::var("HOME") {
        format!("{home}/Documents")
    } else {
        // Last resort fallback (should not happen on real iOS)
        "/var/mobile/Containers/Data/Application/Library".to_string()
    }
}

/// Get the default settings path for iOS
/// This would typically be in the app's Library directory
pub fn ios_settings_path() -> String {
    // Return the path set from Swift, or derive from home directory as fallback
    if let Some(path) = SETTINGS_PATH
        .lock()
        .expect("SETTINGS_PATH lock poisoned")
        .as_ref()
    {
        return path.clone();
    }

    // Fallback: derive from process home directory
    if let Ok(home) = std::env::var("HOME") {
        format!("{home}/Library")
    } else {
        // Last resort fallback (should not happen on real iOS)
        "/var/mobile/Containers/Data/Application/Library".to_string()
    }
}

/// Get the cache directory for iOS
/// This would typically be in the app's Caches directory
#[must_use]
pub fn ios_cache_path() -> String {
    // Return the path set from Swift, or derive from home directory as fallback
    if let Some(path) = CACHE_PATH
        .lock()
        .expect("CACHE_PATH lock poisoned")
        .as_ref()
    {
        return path.clone();
    }

    // Fallback: derive from process home directory
    if let Ok(home) = std::env::var("HOME") {
        format!("{home}/Library/Caches")
    } else {
        // Last resort fallback (should not happen on real iOS)
        "/var/mobile/Containers/Data/Application/Library/Caches".to_string()
    }
}

/// Get the temporary directory for iOS
/// This would typically be `NSTemporaryDirectory()`
#[must_use]
pub fn ios_temp_path() -> String {
    // Return the path set from Swift, or derive from system temp as fallback
    if let Some(path) = TEMP_PATH.lock().expect("TEMP_PATH lock poisoned").as_ref() {
        return path.clone();
    }

    // Fallback: use system temp directory
    std::env::temp_dir().to_string_lossy().to_string()
}

/// Resolve a path relative to the library directory
#[must_use]
pub fn resolve_library_path(relative: &str) -> PathBuf {
    let base = ios_library_path();
    PathBuf::from(base).join(relative)
}

/// Resolve a path relative to the settings directory
#[must_use]
pub fn resolve_settings_path(relative: &str) -> PathBuf {
    let base = ios_settings_path();
    PathBuf::from(base).join(relative)
}

/// Resolve a path relative to the cache directory
#[must_use]
pub fn resolve_cache_path(relative: &str) -> PathBuf {
    let base = ios_cache_path();
    PathBuf::from(base).join(relative)
}
