//! Plato Document Module
//!
//! This crate provides document handling functionality for Plato.

pub use plato_core::document::{
    annotations_as_html, asciify, bookmarks_as_html, chapter_from_uri, file_kind, guess_kind, open,
    open_html, sys_info_as_html, toc_as_html, BoundedText, Location, Neighbors, SimpleTocEntry,
    TextLocation, TocEntry, TocLocation,
};
