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

/// Set the library path from Swift
/// This should be called during initialization before any path resolution
pub fn set_library_path(path: String) {
    *LIBRARY_PATH.lock().unwrap() = Some(path);
}

/// Set the settings path from Swift
/// This should be called during initialization before any path resolution
pub fn set_settings_path(path: String) {
    *SETTINGS_PATH.lock().unwrap() = Some(path);
}

/// Get the default library path for iOS
/// This would typically be in the app's Documents directory
pub fn ios_library_path() -> String {
    // Return the path set from Swift, or derive from home directory as fallback
    if let Some(path) = LIBRARY_PATH.lock().unwrap().as_ref() {
        return path.clone();
    }

    // Fallback: derive from process home directory
    if let Ok(home) = std::env::var("HOME") {
        format!("{}/Documents", home)
    } else {
        // Last resort fallback (should not happen on real iOS)
        "/var/mobile/Containers/Data/Application/Library".to_string()
    }
}

/// Get the default settings path for iOS
/// This would typically be in the app's Library directory
pub fn ios_settings_path() -> String {
    // Return the path set from Swift, or derive from home directory as fallback
    if let Some(path) = SETTINGS_PATH.lock().unwrap().as_ref() {
        return path.clone();
    }

    // Fallback: derive from process home directory
    if let Ok(home) = std::env::var("HOME") {
        format!("{}/Library", home)
    } else {
        // Last resort fallback (should not happen on real iOS)
        "/var/mobile/Containers/Data/Application/Library".to_string()
    }
}

/// Get the cache directory for iOS
/// This would typically be in the app's Caches directory
pub fn ios_cache_path() -> String {
    // For MVP, use a placeholder
    // In production, this would be obtained from Swift via:
    // NSSearchPathForDirectoriesInDomains(.cachesDirectory, .userDomainMask, true)
    "/var/mobile/Containers/Data/Application/Library/Caches".to_string()
}

/// Get the temporary directory for iOS
/// This would typically be NSTemporaryDirectory()
pub fn ios_temp_path() -> String {
    // For MVP, use a placeholder
    // In production, this would be obtained from Swift via NSTemporaryDirectory()
    "/tmp".to_string()
}

/// Resolve a path relative to the library directory
pub fn resolve_library_path(relative: &str) -> PathBuf {
    let base = ios_library_path();
    PathBuf::from(base).join(relative)
}

/// Resolve a path relative to the settings directory
pub fn resolve_settings_path(relative: &str) -> PathBuf {
    let base = ios_settings_path();
    PathBuf::from(base).join(relative)
}

/// Resolve a path relative to the cache directory
pub fn resolve_cache_path(relative: &str) -> PathBuf {
    let base = ios_cache_path();
    PathBuf::from(base).join(relative)
}
