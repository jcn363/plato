//! Buffer reuse pools for temporary work
//!
//! This module provides device-aware thread-local buffer pools to reduce allocations
//! for thumbnail generation and document parsing. Buffer sizes are optimized based
//! on device capabilities (standard Kobo, Elipsa 1GB, Android 12GB).

use crate::consts::buffer_pool as buffer_consts;
use crate::device::{is_android, is_elipsa, is_linuxmint};
use std::sync::LazyLock;

/// Get the appropriate thumbnail buffer size for the current device
fn get_thumbnail_buffer_size() -> usize {
    if is_linuxmint() {
        buffer_consts::LINUXMINT_THUMBNAIL_BUFFER_SIZE
    } else if is_elipsa() {
        buffer_consts::ELIPSA_THUMBNAIL_BUFFER_SIZE
    } else if is_android() {
        buffer_consts::ANDROID_THUMBNAIL_BUFFER_SIZE
    } else {
        buffer_consts::THUMBNAIL_BUFFER_SIZE
    }
}

/// Get the appropriate document buffer size for the current device
fn get_document_buffer_size() -> usize {
    if is_linuxmint() {
        buffer_consts::LINUXMINT_DOCUMENT_BUFFER_SIZE
    } else if is_elipsa() {
        buffer_consts::ELIPSA_DOCUMENT_BUFFER_SIZE
    } else if is_android() {
        buffer_consts::ANDROID_DOCUMENT_BUFFER_SIZE
    } else {
        buffer_consts::DOCUMENT_BUFFER_SIZE
    }
}

/// Thread-local buffer pool for thumbnail generation (device-aware sizing)
pub static THUMBNAIL_BUFFER: LazyLock<std::sync::Mutex<Vec<u8>>> =
    LazyLock::new(|| std::sync::Mutex::new(Vec::with_capacity(get_thumbnail_buffer_size())));

/// Thread-local buffer pool for document parsing (device-aware sizing)
pub static DOCUMENT_BUFFER: LazyLock<std::sync::Mutex<Vec<u8>>> =
    LazyLock::new(|| std::sync::Mutex::new(Vec::with_capacity(get_document_buffer_size())));

/// Get a thumbnail buffer from the pool
pub fn with_thumbnail_buffer<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<u8>) -> R,
{
    let mut buffer = THUMBNAIL_BUFFER
        .lock()
        .expect("THUMBNAIL_BUFFER lock poisoned");
    buffer.clear();
    f(&mut buffer)
}

/// Get a document buffer from the pool
pub fn with_document_buffer<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<u8>) -> R,
{
    let mut buffer = DOCUMENT_BUFFER
        .lock()
        .expect("DOCUMENT_BUFFER lock poisoned");
    buffer.clear();
    f(&mut buffer)
}
