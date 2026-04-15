//! Reader Annotation Handling
//!
//! This module handles all annotation-related functionality for the Reader view,
//! including bookmarks, highlights, and notes management.

use crate::document::Location;
use crate::geom::Rectangle;
use crate::metadata::Annotation;
use crate::view::reader::reader_impl::reader_core::Selection;
use crate::view::{Hub, RenderQueue};
use crate::context::Context;

/// Annotation manager for the Reader view
pub struct ReaderAnnotationManager {
    pub annotations: std::collections::HashMap<usize, Vec<Annotation>>,
    pub current_selection: Option<Selection>,
    pub target_annotation: Option<[crate::document::TextLocation; 2]>,
}

impl ReaderAnnotationManager {
    /// Create a new annotation manager
    pub fn new() -> Self {
        Self {
            annotations: std::collections::HashMap::new(),
            current_selection: None,
            target_annotation: None,
        }
    }

    /// Add a new annotation
    pub fn add_annotation(&mut self, page: usize, annotation: Annotation) {
        self.annotations.entry(page).or_insert_with(Vec::new).push(annotation);
    }

    /// Remove an annotation
    pub fn remove_annotation(&mut self, page: usize, annotation: &Annotation) -> bool {
        if let Some(annotations) = self.annotations.get_mut(&page) {
            let pos = annotations.iter().position(|a| a == annotation);
            if let Some(pos) = pos {
                annotations.remove(pos);
                return true;
            }
        }
        false
    }

    /// Get all annotations for a page
    pub fn get_annotations(&self, page: usize) -> &[Annotation] {
        self.annotations.get(&page).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Set the current selection
    pub fn set_selection(&mut self, selection: Option<Selection>) {
        self.current_selection = selection;
    }

    /// Get the current selection
    pub fn get_selection(&self) -> Option<&Selection> {
        self.current_selection.as_ref()
    }

    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        self.current_selection = None;
    }

    /// Set the target annotation for editing
    pub fn set_target_annotation(&mut self, target: Option<[crate::document::TextLocation; 2]>) {
        self.target_annotation = target;
    }

    /// Get the target annotation
    pub fn get_target_annotation(&self) -> Option<&[crate::document::TextLocation; 2]> {
        self.target_annotation.as_ref()
    }

    /// Create a new bookmark
    pub fn create_bookmark(&mut self, page: usize, location: Location, title: String) -> Annotation {
        let bookmark = Annotation {
            note: title.clone(),
            text: title,
            selection: [crate::document::TextLocation::Dynamic(0), crate::document::TextLocation::Dynamic(1)],
            ..Default::default()
        };
        self.add_annotation(page, bookmark.clone());
        bookmark
    }

    /// Create a new highlight
    pub fn create_highlight(
        &mut self,
        page: usize,
        location: Location,
        text: String,
        _color: crate::color::Color,
    ) -> Annotation {
        let highlight = Annotation {
            note: String::new(),
            text,
            selection: [crate::document::TextLocation::Dynamic(0), crate::document::TextLocation::Dynamic(1)],
            ..Default::default()
        };
        self.add_annotation(page, highlight.clone());
        highlight
    }

    /// Create a new note
    pub fn create_note(
        &mut self,
        page: usize,
        location: Location,
        text: String,
        note: String,
    ) -> Annotation {
        let annotation = Annotation {
            note,
            text,
            selection: [crate::document::TextLocation::Dynamic(0), crate::document::TextLocation::Dynamic(1)],
            ..Default::default()
        };
        self.add_annotation(page, annotation.clone());
        annotation
    }

    /// Get all bookmarks (annotations with note but no text content)
    pub fn get_bookmarks(&self) -> Vec<&Annotation> {
        let mut bookmarks = Vec::new();
        for annotations in self.annotations.values() {
            for annotation in annotations {
                if !annotation.note.is_empty() && annotation.text.is_empty() {
                    bookmarks.push(annotation);
                }
            }
        }
        bookmarks.sort_by(|a, b| a.selection[0].cmp(&b.selection[0]));
        bookmarks
    }

    /// Get all highlights (annotations with text content)
    pub fn get_highlights(&self) -> Vec<&Annotation> {
        let mut highlights = Vec::new();
        for annotations in self.annotations.values() {
            for annotation in annotations {
                if !annotation.text.is_empty() {
                    highlights.push(annotation);
                }
            }
        }
        highlights.sort_by(|a, b| a.selection[0].cmp(&b.selection[0]));
        highlights
    }

    /// Get all notes (annotations with both note and text content)
    pub fn get_notes(&self) -> Vec<&Annotation> {
        let mut notes = Vec::new();
        for annotations in self.annotations.values() {
            for annotation in annotations {
                if !annotation.note.is_empty() && !annotation.text.is_empty() {
                    notes.push(annotation);
                }
            }
        }
        notes.sort_by(|a, b| a.selection[0].cmp(&b.selection[0]));
        notes
    }

    /// Check if page has any annotations
    pub fn has_annotations(&self, page: usize) -> bool {
        self.annotations.contains_key(&page) && !self.annotations[&page].is_empty()
    }

    /// Get annotation count for a page
    pub fn annotation_count(&self, page: usize) -> usize {
        self.annotations.get(&page).map(|v| v.len()).unwrap_or(0)
    }
}
