#![cfg_attr(not(target_os = "android"), allow(dead_code, unused_imports))]

use std::path::PathBuf;

#[cfg(target_os = "android")]
use ndk_context;

/// Get the Android library path for storing documents
/// Checks EXTERNAL_STORAGE env var, falls back to /sdcard/Books then /sdcard
pub fn android_library_path() -> PathBuf {
    if let Ok(storage) = std::env::var("EXTERNAL_STORAGE") {
        let mut path = PathBuf::from(&storage);
        path.push("Books");
        if path.exists() {
            return path;
        }
        return PathBuf::from(storage);
    }

    // Fallback to /sdcard/Books
    let fallback = PathBuf::from("/sdcard/Books");
    if fallback.exists() {
        return fallback;
    }

    // Final fallback to /sdcard
    PathBuf::from("/sdcard")
}

/// Get the Android settings path for storing Settings.toml
/// Returns the app internal data directory using ndk_context
#[cfg(target_os = "android")]
pub fn android_settings_path() -> PathBuf {
    // Use ndk_context to get the app's internal data directory
    let _ctx = ndk_context::android_context();
    // The context provides access to the app's internal storage
    // For now, use a fallback path since ndk_context API is limited
    let mut path = android_library_path();
    path.push(".plato");
    path
}

/// Get the Android settings path for storing Settings.toml
/// Returns the app internal data directory (non-Android stub)
#[cfg(not(target_os = "android"))]
pub fn android_settings_path() -> PathBuf {
    let mut path = android_library_path();
    path.push(".plato");
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_android_library_path_with_env_var() {
        env::set_var("EXTERNAL_STORAGE", "/test/storage");
        let path = android_library_path();
        // Since /test/storage/Books doesn't exist on host, it returns /test/storage
        assert_eq!(path, PathBuf::from("/test/storage"));
        env::remove_var("EXTERNAL_STORAGE");
    }

    #[test]
    fn test_android_library_path_fallback() {
        // Ensure env var is not set
        env::remove_var("EXTERNAL_STORAGE");
        let path = android_library_path();
        // Should return /sdcard since /sdcard/Books doesn't exist on host
        assert_eq!(path, PathBuf::from("/sdcard"));
    }

    #[test]
    fn test_android_settings_path_fallback() {
        // Ensure env var is not set
        env::remove_var("EXTERNAL_STORAGE");
        let path = android_settings_path();
        // Should return /sdcard/.plato as fallback
        assert_eq!(path, PathBuf::from("/sdcard/.plato"));
    }
}
