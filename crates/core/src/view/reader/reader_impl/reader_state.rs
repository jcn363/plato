//! Reader State Management
//!
//! This module contains the core state management functionality for the Reader view,
//! including state initialization, updates, and persistence.

use crate::context::Context;
use crate::geom::Rectangle;
use crate::metadata::Info;
use crate::view::reader::reader_impl::reader_core::State;
use crate::view::{Hub, RenderQueue};

/// Reader state management utilities
#[allow(dead_code)] // Reserved for future state management features
pub struct ReaderStateManager {
    pub state: State,
    pub info: Info,
    pub current_page: usize,
    pub pages_count: usize,
}

#[allow(dead_code)] // Reserved for future state management features
impl ReaderStateManager {
    /// Create a new reader state manager
    pub fn new(info: Info, initial_page: usize, pages_count: usize) -> Self {
        Self {
            state: State::default(),
            info,
            current_page: initial_page,
            pages_count,
        }
    }

    /// Update the reader state
    pub fn update_state(
        &mut self,
        new_page: Option<usize>,
        new_pages_count: Option<usize>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        if let Some(page) = new_page {
            self.current_page = page;
        }
        if let Some(count) = new_pages_count {
            self.pages_count = count;
        }

        // Trigger state update
        rq.add(crate::view::RenderData::new(
            crate::view::ID_FEEDER.next(),
            Rectangle::default(),
            crate::framebuffer::UpdateMode::Partial,
        ));
    }

    /// Get current page
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// Get pages count
    pub fn pages_count(&self) -> usize {
        self.pages_count
    }

    /// Check if at first page
    pub fn is_first_page(&self) -> bool {
        self.current_page == 0
    }

    /// Check if at last page
    pub fn is_last_page(&self) -> bool {
        self.current_page >= self.pages_count.saturating_sub(1)
    }
}
