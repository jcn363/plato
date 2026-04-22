//! Buffer reuse pools for temporary work
//! 
//! This module provides thread-local buffer pools to reduce allocations
//! for thumbnail generation and document parsing.

use std::sync::LazyLock;

/// Thread-local buffer pool for thumbnail generation
pub static THUMBNAIL_BUFFER: LazyLock<std::sync::Mutex<Vec<u8>>> =
    LazyLock::new(|| std::sync::Mutex::new(Vec::with_capacity(1024 * 1024)));

/// Thread-local buffer pool for document parsing
pub static DOCUMENT_BUFFER: LazyLock<std::sync::Mutex<Vec<u8>>> =
    LazyLock::new(|| std::sync::Mutex::new(Vec::with_capacity(4 * 1024 * 1024)));

/// Get a thumbnail buffer from the pool
pub fn with_thumbnail_buffer<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<u8>) -> R,
{
    let mut buffer = THUMBNAIL_BUFFER.lock().expect("THUMBNAIL_BUFFER lock poisoned");
    buffer.clear();
    let result = f(&mut buffer);
    result
}

/// Get a document buffer from the pool
pub fn with_document_buffer<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<u8>) -> R,
{
    let mut buffer = DOCUMENT_BUFFER.lock().expect("DOCUMENT_BUFFER lock poisoned");
    buffer.clear();
    let result = f(&mut buffer);
    result
}
