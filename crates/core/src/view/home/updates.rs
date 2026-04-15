//! Home State Updates
//!
//! Handles UI state refresh/updates:
//! - refresh_visibles()
//! - update_first_column()
//! - update_second_column()
//! - update_thumbnail_previews()
//! - update_shelf()
//! - update_top_bar()
//! - update_bottom_bar()

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::unit::scale_by_dpi;
use crate::view::home::home_utils;
use crate::view::home::BottomBar;
use crate::view::home::Shelf;
use crate::view::top_bar::TopBar;
use crate::view::{Hub, RenderQueue, View};

use super::Home;

impl Home {
    pub(crate) fn refresh_visibles(
        &mut self,
        update: bool,
        reset_page: bool,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let (files, _) = context
            .library
            .list(&self.current_directory, self.query.as_ref(), false);
        self.visible_books = files;

        let max_lines = {
            let shelf = self.child(self.shelf_index).downcast_ref::<Shelf>();
            shelf.map_or(1, |s| s.max_lines)
        };

        self.pages_count = (self.visible_books.len() as f32 / max_lines as f32).ceil() as usize;

        if reset_page {
            self.current_page = 0;
        } else if self.current_page >= self.pages_count {
            self.current_page = self.pages_count.saturating_sub(1);
        }

        if update {
            home_utils::update_shelf_and_bottom_bar(false, self, hub, rq, context);
        }
    }

    pub(crate) fn update_first_column(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let selected_library = context.settings.selected_library;
        if let Some(shelf) = self.children[self.shelf_index]
            .as_mut()
            .downcast_mut::<Shelf>()
        {
            shelf.set_first_column(context.settings.libraries[selected_library].first_column);
        }
        home_utils::update_shelf_and_bottom_bar(false, self, hub, rq, context);
    }

    pub(crate) fn update_second_column(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let selected_library = context.settings.selected_library;
        if let Some(shelf) = self.children[self.shelf_index]
            .as_mut()
            .downcast_mut::<Shelf>()
        {
            shelf.set_second_column(context.settings.libraries[selected_library].second_column);
        }
        home_utils::update_shelf_and_bottom_bar(false, self, hub, rq, context);
    }

    pub(crate) fn update_thumbnail_previews(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let selected_library = context.settings.selected_library;
        if let Some(shelf) = self.children[self.shelf_index]
            .as_mut()
            .downcast_mut::<Shelf>()
        {
            shelf.set_thumbnail_previews(
                context.settings.libraries[selected_library].thumbnail_previews,
            );
        }
        home_utils::update_shelf_and_bottom_bar(false, self, hub, rq, context);
    }

    pub(crate) fn update_shelf(
        &mut self,
        was_resized: bool,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let dpi = CURRENT_DEVICE.dpi;
        let big_height = scale_by_dpi(crate::view::BIG_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(crate::view::THICKNESS_MEDIUM, dpi) as i32;
        let shelf = match self.children[self.shelf_index]
            .as_mut()
            .downcast_mut::<Shelf>()
        {
            Some(s) => s,
            None => return,
        };
        let max_lines = ((shelf.rect.height() as i32 + thickness) / big_height) as usize;

        if was_resized {
            let page_position = if self.visible_books.is_empty() {
                0.0
            } else {
                self.current_page as f32
                    * (shelf.max_lines as f32 / self.visible_books.len() as f32)
            };

            let mut page_guess = page_position * self.visible_books.len() as f32 / max_lines as f32;
            let page_ceil = page_guess.ceil();

            if (page_ceil - page_guess).abs() < f32::EPSILON {
                page_guess = page_ceil;
            }

            self.pages_count = (self.visible_books.len() as f32 / max_lines as f32).ceil() as usize;
            self.current_page = (page_guess as usize).min(self.pages_count.saturating_sub(1));
        }

        let index_lower = self.current_page * max_lines;
        let index_upper = (index_lower + max_lines).min(self.visible_books.len());

        shelf.update(
            &self.visible_books[index_lower..index_upper],
            hub,
            rq,
            context,
        );
    }

    pub(crate) fn update_top_bar(&mut self, search_visible: bool, rq: &mut RenderQueue) {
        if let Some(index) = home_utils::find_child_index_by_type::<TopBar>(&self.children) {
            if let Some(top_bar) = self.children[index].as_mut().downcast_mut::<TopBar>() {
                let name = if search_visible { "back" } else { "search" };
                top_bar.update_root_icon(name, rq);
                top_bar.update_title_label(&self.sort_method.title(), rq);
            }
        }
    }

    pub(crate) fn update_bottom_bar(&mut self, rq: &mut RenderQueue, context: &Context) {
        if let Some(index) = home_utils::find_child_index_by_type::<BottomBar>(&self.children) {
            let Some(bottom_bar) = self.children[index].as_mut().downcast_mut::<BottomBar>() else {
                return;
            };
            let filter = self.query.is_some() || self.current_directory != context.library.home;
            let selected_library = context.settings.selected_library;
            let library_settings = &context.settings.libraries[selected_library];
            bottom_bar.update_library_label(
                &library_settings.name,
                self.visible_books.len(),
                filter,
                rq,
            );
            bottom_bar.update_page_label(self.current_page, self.pages_count, rq);
            bottom_bar.update_icons(self.current_page, self.pages_count, rq);
        }
    }
}
