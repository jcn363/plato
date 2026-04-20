//! Input validation helpers for public APIs
//!
//! This module provides validation functions that follow AGENTS.md rules:
//! - Validate all inputs at public API boundaries
//! - Fail fast with clear, actionable error messages
//! - Never trust external data

use anyhow::{bail, format_err, Error};
use std::path::{Path, PathBuf};

/// Maximum allowed path length for file operations
pub const MAX_PATH_LENGTH: usize = 4096;

/// Maximum allowed file name length
pub const MAX_FILENAME_LENGTH: usize = 255;

/// Validates that a path is acceptable for file operations
///
/// Checks:
/// - Path is not empty
/// - Path length is within reasonable limits
/// - Path does not contain null bytes
/// - Path is not purely relative navigation (e.g., "..")
pub fn validate_path<P: AsRef<Path>>(path: P, context: &str) -> Result<(), Error> {
    let path_ref = path.as_ref();

    // Check for empty path
    if path_ref.as_os_str().is_empty() {
        bail!("{}: path cannot be empty", context);
    }

    // Check path length
    let path_str = path_ref.to_string_lossy();
    if path_str.len() > MAX_PATH_LENGTH {
        bail!(
            "{}: path length {} exceeds maximum {}",
            context,
            path_str.len(),
            MAX_PATH_LENGTH
        );
    }

    // Check for null bytes in path
    if path_str.contains('\0') {
        bail!("{}: path contains null bytes", context);
    }

    // Check that path has some meaningful component beyond just navigation
    let has_real_component = path_ref
        .components()
        .any(|c| matches!(c, std::path::Component::Normal(_)));

    if !has_real_component {
        bail!(
            "{}: path must contain at least one file or directory name",
            context
        );
    }

    Ok(())
}

/// Validates a file name for safety
///
/// Checks:
/// - Name is not empty
/// - Name length is within limits
/// - Name does not contain path separators
/// - Name is not ".." or "."
pub fn validate_filename(name: &str, context: &str) -> Result<(), Error> {
    // Check for empty name
    if name.is_empty() {
        bail!("{}: file name cannot be empty", context);
    }

    // Check name length
    if name.len() > MAX_FILENAME_LENGTH {
        bail!(
            "{}: file name length {} exceeds maximum {}",
            context,
            name.len(),
            MAX_FILENAME_LENGTH
        );
    }

    // Check for path separators in name
    if name.contains('/') || name.contains('\\') {
        bail!("{}: file name cannot contain path separators", context);
    }

    // Check for special names
    if name == "." || name == ".." {
        bail!("{}: file name cannot be '.' or '..'", context);
    }

    // Check for null bytes
    if name.contains('\0') {
        bail!("{}: file name cannot contain null bytes", context);
    }

    Ok(())
}

/// Validates that a numeric value is within an acceptable range
pub fn validate_range<T>(value: T, min: T, max: T, name: &str) -> Result<(), Error>
where
    T: std::fmt::Display + PartialOrd,
{
    if value < min || value > max {
        bail!(
            "{}: value {} is outside valid range [{}, {}]",
            name,
            value,
            min,
            max
        );
    }
    Ok(())
}

/// Validates that a floating-point value is finite and within range
pub fn validate_finite_f32(value: f32, name: &str, min: f32, max: f32) -> Result<(), Error> {
    if !value.is_finite() {
        bail!("{}: value must be finite (not inf or nan)", name);
    }
    validate_range(value, min, max, name)
}

/// Validates that a string is not empty and within length limits
pub fn validate_string_length(s: &str, name: &str, min: usize, max: usize) -> Result<(), Error> {
    let len = s.len();
    if len < min {
        bail!("{}: length {} is below minimum {}", name, len, min);
    }
    if len > max {
        bail!("{}: length {} exceeds maximum {}", name, len, max);
    }
    Ok(())
}

/// Validates that a path is within a base directory (prevents directory traversal)
pub fn validate_path_within_base<P: AsRef<Path>>(
    base: P,
    path: P,
    context: &str,
) -> Result<(), Error> {
    let base_ref = base.as_ref();
    let path_ref = path.as_ref();

    // Canonicalize paths for comparison
    let canonical_base = base_ref.canonicalize().map_err(|e| {
        format_err!(
            "{}: cannot canonicalize base path {}: {}",
            context,
            base_ref.display(),
            e
        )
    })?;

    let canonical_path = path_ref.canonicalize().map_err(|e| {
        format_err!(
            "{}: cannot canonicalize path {}: {}",
            context,
            path_ref.display(),
            e
        )
    })?;

    // Check that the path starts with the base
    if !canonical_path.starts_with(&canonical_base) {
        bail!(
            "{}: path {} is outside base directory {}",
            context,
            path_ref.display(),
            base_ref.display()
        );
    }

    Ok(())
}

/// Validates that a directory path is suitable for library operations
pub fn validate_library_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, Error> {
    let path_ref = path.as_ref();

    validate_path(path_ref, "library path")?;

    // If path exists, verify it's a directory
    if path_ref.exists() && !path_ref.is_dir() {
        bail!(
            "library path {} exists but is not a directory",
            path_ref.display()
        );
    }

    Ok(path_ref.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_empty() {
        let result = validate_path("", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_path_valid() {
        let result = validate_path("/home/user/file.txt", "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_with_null() {
        let result = validate_path("/home/user/file\0.txt", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_filename_empty() {
        let result = validate_filename("", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_filename_with_separator() {
        let result = validate_filename("file/name.txt", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_filename_special() {
        let result = validate_filename("..", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_range_in_range() {
        let result = validate_range(50, 0, 100, "value");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_range_below_min() {
        let result = validate_range(-1, 0, 100, "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_range_above_max() {
        let result = validate_range(101, 0, 100, "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_finite_f32_nan() {
        let result = validate_finite_f32(f32::NAN, "value", 0.0, 100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_finite_f32_infinite() {
        let result = validate_finite_f32(f32::INFINITY, "value", 0.0, 100.0);
        assert!(result.is_err());
    }
}
