//! Home Navigation Operations
//!
//! Handles directory/page navigation:
//! - select_directory()
//! - go_to_page()
//! - go_to_neighbor()
//! - go_to_status_change()

use std::path::Path;

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::CycleDir;
use crate::view::home::home_utils;
use crate::view::home::AddressBar;
use crate::view::home::NavigationBar;
use crate::view::home::Shelf;
use crate::view::{Hub, RenderData, RenderQueue, View};

use super::Home;

impl Home {
    pub(crate) fn select_directory(
        &mut self,
        path: &Path,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if self.current_directory == path {
            return;
        }

        let old_path = std::mem::replace(&mut self.current_directory, path.to_path_buf());
        self.terminate_fetchers(&old_path, true, hub, context);

        let selected_library = context.settings.selected_library;
        for hook in &context.settings.libraries[selected_library].hooks {
            if context.library.home.join(&hook.path) == path {
                self.insert_fetcher(hook, hub, context);
            }
        }

        let (files, dirs) =
            context
                .library
                .list(&self.current_directory, self.query.as_ref(), false);
        self.visible_books = files;
        self.current_page = 0;

        let mut index = 2;

        if context.settings.home.address_bar {
            let Some(addr_bar) = self.children[index].as_mut().downcast_mut::<AddressBar>() else {
                return;
            };
            addr_bar.set_text(self.current_directory.to_string_lossy(), rq, context);
            index += 2;
        }

        if context.settings.home.navigation_bar {
            let Some(nav_bar) = self.children[index]
                .as_mut()
                .downcast_mut::<NavigationBar>()
            else {
                return;
            };
            nav_bar.set_path(&self.current_directory, &dirs, rq, context);
            home_utils::adjust_shelf_top_edge(&mut self.children, self.shelf_index - 2);
            rq.add(RenderData::new(
                self.child(index + 1).id(),
                *self.child(index + 1).rect(),
                UpdateMode::Partial,
            ));
            rq.add(RenderData::new(
                self.child(index).id(),
                *self.child(index).rect(),
                UpdateMode::Partial,
            ));
        }

        home_utils::update_shelf_and_bottom_bar(true, self, hub, rq, context);
    }

    pub(crate) fn go_to_page(
        &mut self,
        index: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if index >= self.pages_count {
            return;
        }
        self.current_page = index;
        home_utils::update_shelf_and_bottom_bar(false, self, hub, rq, context);
    }

    pub(crate) fn go_to_neighbor(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        match dir {
            CycleDir::Next if self.current_page < self.pages_count.saturating_sub(1) => {
                self.current_page += 1;
            }
            CycleDir::Previous if self.current_page > 0 => {
                self.current_page -= 1;
            }
            _ => return,
        }

        home_utils::update_shelf_and_bottom_bar(false, self, hub, rq, context);
    }

    pub(crate) fn go_to_status_change(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if self.pages_count < 2 {
            return;
        }

        let max_lines = self.children[self.shelf_index]
            .as_ref()
            .downcast_ref::<Shelf>()
            .map_or(1, |shelf| shelf.max_lines);
        let index_lower = self.current_page * max_lines;
        let index_upper = (index_lower + max_lines).min(self.visible_books.len());
        let book_index = match dir {
            CycleDir::Next => index_upper.saturating_sub(1),
            CycleDir::Previous => index_lower,
        };
        let status = self.visible_books[book_index].simple_status();

        let page = match dir {
            CycleDir::Next => self.visible_books[book_index + 1..]
                .iter()
                .position(|info| info.simple_status() != status)
                .map(|delta| self.current_page + 1 + delta / max_lines),
            CycleDir::Previous => self.visible_books[..book_index]
                .iter()
                .rev()
                .position(|info| info.simple_status() != status)
                .map(|delta| self.current_page - 1 - delta / max_lines),
        };

        if let Some(page) = page {
            self.current_page = page;
            home_utils::update_shelf_and_bottom_bar(false, self, hub, rq, context);
        }
    }
}
