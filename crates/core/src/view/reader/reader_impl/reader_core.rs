//! Reader Core Module - Shared Types
//!
//! This module defines shared types used across Reader functionality.
//! Eventually constructors and core Reader methods will be moved here.

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
#[allow(dead_code)]
pub(crate) struct Resource {
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
