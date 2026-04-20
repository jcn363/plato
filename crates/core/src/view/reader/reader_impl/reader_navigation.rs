//! Reader Navigation Module
//!
//! Handles document navigation (page turning, chapter switching, history,
//! bookmarks, annotations, and scrolling).

use super::reader::Reader;
use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::{CycleDir, Point};
use crate::view::{Hub, RenderData, RenderQueue};

impl Reader {
    /// Helper: Queue a partial update for the reader view
    #[inline]
    pub(crate) fn queue_partial_update(&self, rq: &mut RenderQueue) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    /// Navigate to a specific page
    pub fn go_to_page(
        &mut self,
        index: usize,
        save_state: bool,
        _hub: &Hub,
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

    /// Navigate to next or previous page
    pub fn go_to_neighbor(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let index = match dir {
            CycleDir::Next => self.current_page.saturating_add(1),
            CycleDir::Previous => self.current_page.saturating_sub(1),
        };
        self.go_to_page(index, true, hub, rq, context);
    }

    /// Navigate to next or previous chapter using TOC
    pub fn go_to_chapter(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(ref simple_toc) = self.info.simple_toc {
            if simple_toc.is_empty() {
                self.queue_partial_update(rq);
                return;
            }

            self.toc_manager.build_toc(&self.info);

            let current_chapter = self.toc_manager.get_chapter_for_page(self.current_page);

            let target_chapter = match dir {
                CycleDir::Next => current_chapter.and_then(|c| {
                    if c + 1 < self.toc_manager.toc_entries.len() {
                        Some(c + 1)
                    } else {
                        None
                    }
                }),
                CycleDir::Previous => {
                    current_chapter.and_then(|c| if c > 0 { Some(c - 1) } else { None })
                }
            };

            if let Some(chapter_idx) = target_chapter {
                if let Some(page) = self
                    .toc_manager
                    .navigate_to_chapter(chapter_idx, self.current_page)
                {
                    self.go_to_page(page, true, hub, rq, context);
                    return;
                }
            }
        }

        self.queue_partial_update(rq);
    }

    /// Navigate to next or previous bookmark
    pub fn go_to_bookmark(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let bookmarks = self
            .info
            .reader
            .as_ref()
            .map(|r| &r.bookmarks)
            .filter(|b| !b.is_empty());

        if let Some(bookmarks) = bookmarks {
            let target_page = match dir {
                CycleDir::Next => bookmarks.iter().find(|&&b| b > self.current_page).copied(),
                CycleDir::Previous => bookmarks
                    .iter()
                    .rev()
                    .find(|&&b| b < self.current_page)
                    .copied(),
            };

            if let Some(page) = target_page {
                self.go_to_page(page, true, hub, rq, context);
                return;
            }
        }

        self.queue_partial_update(rq);
    }

    /// Navigate to next or previous annotation
    pub fn go_to_annotation(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let annotations = self
            .info
            .reader
            .as_ref()
            .map(|r| &r.annotations)
            .filter(|a| !a.is_empty());

        if let Some(annotations) = annotations {
            let target_annotation = match dir {
                CycleDir::Next => annotations.iter().find(|a| {
                    let page = a.selection[0].location();
                    page > self.current_page
                }),
                CycleDir::Previous => annotations.iter().rev().find(|a| {
                    let page = a.selection[0].location();
                    page < self.current_page
                }),
            };

            if let Some(annotation) = target_annotation {
                let page = annotation.selection[0].location();
                self.go_to_page(page, true, hub, rq, context);
                return;
            }
        }

        self.queue_partial_update(rq);
    }

    /// Navigate to the last page of document
    pub fn go_to_last_page(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let last_page = self.pages_count.saturating_sub(1);
        if last_page != self.current_page {
            self.go_to_page(last_page, true, hub, rq, context);
        } else {
            self.queue_partial_update(rq);
        }
    }

    /// Directional scroll
    pub fn directional_scroll(
        &mut self,
        delta: Point,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.view_port.page_offset.y = (self.view_port.page_offset.y + delta.y).max(0);
        self.view_port.page_offset.x = (self.view_port.page_offset.x + delta.x).max(0);
        self.queue_partial_update(rq);
    }

    /// Vertical scroll
    pub fn vertical_scroll(
        &mut self,
        distance: i32,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.view_port.page_offset.y = (self.view_port.page_offset.y + distance).max(0);
        self.queue_partial_update(rq);
    }

    /// Handle back navigation
    pub fn handle_back(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if let Some(prev_page) = self.history.pop_back() {
            self.go_to_page(prev_page, true, hub, rq, context);
        } else {
            self.queue_partial_update(rq);
        }
    }

    /// Handle go to page submission
    pub fn handle_go_to_page_submit(
        &mut self,
        page: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let target_page = page.min(self.pages_count.saturating_sub(1));

        if target_page != self.current_page {
            self.go_to_page(target_page, true, hub, rq, context);
        } else {
            self.queue_partial_update(rq);
        }
    }

    /// Handle go to location
    pub fn handle_go_to_location(
        &mut self,
        location: &crate::document::Location,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let crate::document::Location::Exact(page) = *location {
            if page != self.current_page && page < self.pages_count {
                self.go_to_page(page, true, hub, rq, context);
            } else {
                self.queue_partial_update(rq);
            }
        } else {
            self.queue_partial_update(rq);
        }
    }
}
