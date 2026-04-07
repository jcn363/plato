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

use std::collections::VecDeque;
use std::sync::atomic;

use crate::document::Location;
use crate::framebuffer::Pixmap;
use crate::geom::{LinearDir, Point, Rectangle};
use crate::input::ButtonCode;
use crate::metadata::{ScrollMode, ZoomMode};
#[allow(unused_imports)]
use crate::view::ViewId;
use rustc_hash::{FxHashMap, FxHashSet};

// ===========================================================================
// Nested Structs for Reader Field Consolidation (Phase 4)
// ===========================================================================

/// Page navigation state
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageState {
    pub current_page: usize,
    pub pages_count: usize,
    pub synthetic: bool,
}

impl Default for PageState {
    fn default() -> Self {
        PageState {
            current_page: 0,
            pages_count: 0,
            synthetic: false,
        }
    }
}

/// Reader display settings
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplaySettings {
    pub contrast: Contrast,
    pub reflowable: bool,
    pub ephemeral: bool,
    pub finished: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        DisplaySettings {
            contrast: Contrast::default(),
            reflowable: true,
            ephemeral: false,
            finished: false,
        }
    }
}

/// Interaction state
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InteractionState {
    pub focus: Option<crate::view::ViewId>,
    pub selection: Option<Selection>,
    pub held_buttons: FxHashSet<ButtonCode>,
    pub history: VecDeque<usize>,
    pub state: State,
}

impl Default for InteractionState {
    fn default() -> Self {
        InteractionState {
            focus: None,
            selection: None,
            held_buttons: FxHashSet::default(),
            history: VecDeque::new(),
            state: State::Idle,
        }
    }
}

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
#[allow(dead_code)]
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
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageAnimation {
    None,
    Slide(AnimState),
    Peel(AnimState),
}

/// A rendered chunk of a page
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct Search {
    pub query: String,
    pub results: Vec<Location>,
    pub index: usize,
    pub running: atomic::AtomicBool,
    pub results_count: usize,
    pub highlights: FxHashMap<usize, Vec<Rectangle>>,
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
#[allow(dead_code)]
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
