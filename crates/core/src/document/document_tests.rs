//! Tests for document module validation
//!
//! This file contains tests for the input validation added to document public APIs.

use super::*;
use std::path::Path;

#[test]
fn test_guess_kind_empty_path() {
    let result = guess_kind("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_guess_kind_null_bytes() {
    let result = guess_kind("/path/to/file\0.txt");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("null bytes"));
}

#[test]
fn test_guess_kind_too_long() {
    // Create a path that exceeds MAX_PATH_LENGTH (4096)
    let long_path = "a".repeat(5000);
    let result = guess_kind(&long_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
}

#[test]
fn test_guess_kind_no_real_component() {
    let result = guess_kind("..");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must contain at least one file"));
}

#[test]
fn test_open_empty_path() {
    let result = open("");
    assert!(result.is_none());
}

#[test]
fn test_open_null_bytes() {
    let result = open("/path/to/file\0.txt");
    assert!(result.is_none());
}

#[test]
fn test_open_too_long() {
    let long_path = "a".repeat(5000);
    let result = open(&long_path);
    assert!(result.is_none());
}

#[test]
fn test_open_no_real_component() {
    let result = open("..");
    assert!(result.is_none());
}

#[test]
fn test_open_html_empty_content() {
    let result = open_html("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_open_html_whitespace_only() {
    let result = open_html("   \n\t  ");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_open_logs_validation_error() {
    // Test that validation errors are logged when open fails
    let result = open(""); // Empty path should fail validation
    assert!(result.is_none());
    // The error should be logged (we can't test log output directly, but we verify the behavior)
}
