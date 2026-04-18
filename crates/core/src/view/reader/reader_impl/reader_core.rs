//! Reader Core Module - Shared Types
//!
//! This module defines shared types used across Reader functionality.
//! These types are the canonical definitions and are re-exported by the parent modules.
//!
//! ## Canonical Types
//!
//! The following types are defined here and used throughout the reader implementation:
//! - `State` - Reader state machine (Idle, Selection, AdjustSelection)
//! - `Selection` - Text selection with anchor point
//! - `Contrast` - Contrast adjustment parameters
//! - `PageAnimKind` - Page animation types (Slide, Fade, Flip)
//! - `AnimState` - Animation state during page transitions
//! - `PageAnimation` - Page animation states
//! - `ViewPort` - Viewport configuration (zoom, scroll, offset, margins)
//! - `RenderChunk` - A rendered chunk of a page
//! - `Resource` - Cached rendered resource (pixmap, frame, scale)
//! - `Search` - Search state and results
//!
//! ## Design Notes
//!
//! Types were consolidated here from reader.rs to provide a single canonical location.
//! Previously, duplicate definitions existed (e.g., ViewPort was private in reader.rs
//! and public in reader_core.rs). Now reader_core.rs is the single source of truth.
//!
//! The Reader struct remains in reader.rs due to high interdependency with its methods.

use std::sync::atomic;

use crate::document::Location;
use crate::framebuffer::Pixmap;
use crate::geom::{LinearDir, Point, Rectangle};
use crate::metadata::{ScrollMode, ZoomMode};
use rustc_hash::FxHashMap;

// ===========================================================================
// Shared Types - Used Across Modules
// ===========================================================================

/// Reader state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Idle,
    Selection(usize),
    AdjustSelection,
}

impl Default for State {
    fn default() -> Self {
        State::Idle
    }
}

/// Text selection with anchor point
#[derive(Debug, Clone)]
pub struct Selection {
    pub start: Point,
    pub end: Point,
    pub anchor: Point,
}

/// Contrast adjustment parameters
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contrast {
    pub gray: f32,
    pub exponent: f32,
}

impl Default for Contrast {
    fn default() -> Self {
        Contrast {
            gray: 224.0,
            exponent: 1.0,
        }
    }
}

/// Page animation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageAnimKind {
    Slide,
    Fade,
    Flip,
}

/// Animation state during page transitions
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimState {
    pub kind: PageAnimKind,
    pub direction: LinearDir,
    pub progress: f32,
}

/// Page animation states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageAnimation {
    None,
    Slide(AnimState),
    Peel(AnimState),
}

/// A rendered chunk of a page
#[derive(Debug, Clone)]
pub struct RenderChunk {
    pub page: usize,
    pub location: usize,
    pub rect: Rectangle,
    pub frame: Rectangle,
    pub position: Point,
    pub scale: f32,
}

/// Search state
#[derive(Debug)]
pub struct Search {
    pub _query: String,
    pub results: Vec<Location>,
    pub index: usize,
    pub running: atomic::AtomicBool,
    pub _results_count: usize,
    pub highlights: FxHashMap<usize, Vec<Rectangle>>,
    pub direction: crate::geom::LinearDir,
}

/// Annotation type for categorizing annotations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationType {
    Highlight,
    Note,
    Bookmark,
    Definition,
}

impl Default for AnnotationType {
    fn default() -> Self {
        AnnotationType::Highlight
    }
}

/// Color for annotation highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationColor {
    Yellow,
    Green,
    Blue,
    Pink,
    Orange,
}

impl Default for AnnotationColor {
    fn default() -> Self {
        AnnotationColor::Yellow
    }
}

/// Annotation representing a user highlight, note, or bookmark
#[derive(Debug, Clone)]
pub struct Annotation {
    pub id: usize,
    pub page: usize,
    pub rect: Rectangle,
    pub text: String,
    pub note: Option<String>,
    pub annotation_type: AnnotationType,
    pub color: AnnotationColor,
    pub timestamp: u64,
}

impl Annotation {
    /// Create a new annotation
    pub fn new(
        id: usize,
        page: usize,
        rect: Rectangle,
        text: String,
        annotation_type: AnnotationType,
    ) -> Self {
        Annotation {
            id,
            page,
            rect,
            text,
            note: None,
            annotation_type,
            color: AnnotationColor::default(),
            timestamp: 0, // Would be set to actual timestamp in production
        }
    }

    /// Add a note to the annotation
    pub fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }

    /// Set the annotation color
    pub fn with_color(mut self, color: AnnotationColor) -> Self {
        self.color = color;
        self
    }
}

/// Annotation list for managing document annotations
#[derive(Debug, Default)]
pub struct AnnotationList {
    pub annotations: Vec<Annotation>,
    pub next_id: usize,
}

impl AnnotationList {
    /// Create a new empty annotation list
    pub fn new() -> Self {
        AnnotationList {
            annotations: Vec::new(),
            next_id: 1,
        }
    }

    /// Add an annotation to the list
    pub fn add(&mut self, mut annotation: Annotation) -> usize {
        let id = self.next_id;
        annotation.id = id;
        self.next_id += 1;
        self.annotations.push(annotation);
        id
    }

    /// Remove an annotation by ID
    pub fn remove(&mut self, id: usize) -> bool {
        if let Some(pos) = self.annotations.iter().position(|a| a.id == id) {
            self.annotations.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get all annotations for a specific page
    pub fn get_for_page(&self, page: usize) -> Vec<&Annotation> {
        self.annotations.iter().filter(|a| a.page == page).collect()
    }

    /// Find next annotation from current page
    pub fn find_next(&self, current_page: usize) -> Option<&Annotation> {
        self.annotations
            .iter()
            .filter(|a| a.page > current_page)
            .min_by_key(|a| a.page)
    }

    /// Find previous annotation from current page
    pub fn find_previous(&self, current_page: usize) -> Option<&Annotation> {
        self.annotations
            .iter()
            .filter(|a| a.page < current_page)
            .max_by_key(|a| a.page)
    }

    /// Get annotation by ID
    pub fn get(&self, id: usize) -> Option<&Annotation> {
        self.annotations.iter().find(|a| a.id == id)
    }

    /// Check if there are any annotations
    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }

    /// Get total annotation count
    pub fn len(&self) -> usize {
        self.annotations.len()
    }
}

/// Cached rendered resource
#[derive(Debug)]
pub struct Resource {
    pub pixmap: Pixmap,
    pub frame: Rectangle,
    pub scale: f32,
}

/// Viewport configuration
#[derive(Debug)]
pub struct ViewPort {
    pub zoom_mode: ZoomMode,
    pub scroll_mode: ScrollMode,
    pub page_offset: Point,
    pub margin_width: i32,
}

impl Default for ViewPort {
    fn default() -> Self {
        ViewPort {
            zoom_mode: ZoomMode::FitToWidth,
            scroll_mode: ScrollMode::Screen,
            page_offset: pt!(0, 0),
            margin_width: 0,
        }
    }
}
