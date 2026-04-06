use std::ops::{Deref, DerefMut};
use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;

use super::identifiers::Id;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;

pub struct RenderData {
    pub id: Option<Id>,
    pub rect: Rectangle,
    pub mode: UpdateMode,
    pub wait: bool,
}

impl RenderData {
    pub fn new(id: Id, rect: Rectangle, mode: UpdateMode) -> RenderData {
        RenderData {
            id: Some(id),
            rect,
            mode,
            wait: true,
        }
    }

    pub fn no_wait(id: Id, rect: Rectangle, mode: UpdateMode) -> RenderData {
        RenderData {
            id: Some(id),
            rect,
            mode,
            wait: false,
        }
    }

    pub fn expose(rect: Rectangle, mode: UpdateMode) -> RenderData {
        RenderData {
            id: None,
            rect,
            mode,
            wait: true,
        }
    }
}

pub struct UpdateData {
    pub token: u32,
    pub time: Instant,
    pub rect: Rectangle,
}

pub const MAX_UPDATE_DELAY: Duration = Duration::from_millis(600);

impl UpdateData {
    pub fn has_completed(&self) -> bool {
        self.time.elapsed() >= MAX_UPDATE_DELAY
    }
}

type RQ = FxHashMap<(UpdateMode, bool), Vec<(Option<Id>, Rectangle)>>;

pub struct RenderQueue(RQ);

impl RenderQueue {
    pub fn new() -> RenderQueue {
        RenderQueue(FxHashMap::default())
    }

    /// Add render data to queue, deduplicating same (id, rect) pairs.
    /// Only the first entry is kept; subsequent duplicates are ignored.
    pub fn add(&mut self, data: RenderData) {
        let key = (data.mode, data.wait);
        let entry = self.entry(key).or_insert_with(|| Vec::with_capacity(8));

        // Skip if this (id, rect) pair already exists
        let new_pair = (data.id, data.rect);
        if !entry.iter().any(|existing| existing == &new_pair) {
            entry.push(new_pair);
        }
    }
}

impl Default for RenderQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for RenderQueue {
    type Target = RQ;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Border thicknesses in pixels, at 300 DPI.
pub const THICKNESS_SMALL: f32 = 1.5;
pub const THICKNESS_MEDIUM: f32 = 2.0;
pub const THICKNESS_LARGE: f32 = 3.0;

// Border radii in pixels, at 300 DPI.
pub const BORDER_RADIUS_SMALL: f32 = 6.0;
pub const BORDER_RADIUS_MEDIUM: f32 = 9.0;
pub const BORDER_RADIUS_LARGE: f32 = 12.0;

// Big and small bar heights in pixels, at 300 DPI.
// On the *Aura ONE*, the height is exactly `2 * sb + 10 * bb`.
pub const SMALL_BAR_HEIGHT: f32 = 121.0;
pub const BIG_BAR_HEIGHT: f32 = 163.0;

pub const CLOSE_IGNITION_DELAY: Duration = Duration::from_millis(150);
