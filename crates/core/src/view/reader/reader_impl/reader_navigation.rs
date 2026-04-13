//! Reader Navigation Module
//!
//! Handles document navigation (page turning, chapter switching, history).

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::CycleDir;
use crate::view::{RenderData, RenderQueue};
use super::reader::Reader;

impl Reader {
    pub fn go_to_page(
        &mut self,
        index: usize,
        save_state: bool,
        _hub: &crate::view::Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        let pages_count = self.pages_count;
        if index < pages_count {
            if save_state {
                self.history.push_back(self.current_page);
                if self.history.len() > 100 {
                    self.history.pop_front();
                }
            }
            self.current_page = index;
            self.cache.clear();
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Full));
        }
    }

    pub fn go_to_neighbor(
        &mut self,
        dir: CycleDir,
        hub: &crate::view::Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let index = match dir {
            CycleDir::Next => self.current_page.saturating_add(1),
            CycleDir::Previous => self.current_page.saturating_sub(1),
        };
        self.go_to_page(index, true, hub, rq, context);
    }
}
