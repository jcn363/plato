//! Home Library Operations
//!
//! Handles library state management:
//! - load_library()
//! - import()
//! - clean_up()
//! - flush()

use std::mem;

use crate::context::Context;
use crate::library::Library;
use crate::log_error;
use crate::view::common::rlocate;
use crate::view::home::Shelf;
use crate::view::search_bar::SearchBar;
use crate::view::{Event, Hub, RenderQueue};

use super::Home;

impl Home {
    pub(crate) fn load_library(
        &mut self,
        index: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if index == context.settings.selected_library {
            return;
        }

        let library_settings = context.settings.libraries[index].clone();
        let library = Library::new(&library_settings.path, library_settings.mode)
            .map_err(|e| log_error!("Can't load library: {:#}.", e));

        let library = if let Ok(library) = library {
            library
        } else {
            return;
        };

        let old_path = mem::take(&mut self.current_directory);
        self.terminate_fetchers(&old_path, false, hub, context);

        let mut update_top_bar = false;

        if self.query.is_some() {
            self.toggle_search_bar(Some(false), false, hub, rq, context);
            update_top_bar = true;
        }

        context.library.flush();

        context.library = library;
        context.settings.selected_library = index;

        if self.sort_method != library_settings.sort_method {
            self.sort_method = library_settings.sort_method;
            self.reverse_order = library_settings.sort_method.reverse_order();
            update_top_bar = true;
        }

        context.library.sort(self.sort_method, self.reverse_order);

        if update_top_bar {
            let search_visible = rlocate::<SearchBar>(self).is_some();
            self.update_top_bar(search_visible, rq);
        }

        if let Some(shelf) = self.children[self.shelf_index]
            .as_mut()
            .downcast_mut::<Shelf>()
        {
            shelf.set_first_column(library_settings.first_column);
            shelf.set_second_column(library_settings.second_column);
            shelf.set_thumbnail_previews(library_settings.thumbnail_previews);
        }

        let home = context.library.home.clone();
        self.select_directory(&home, hub, rq, context);
    }

    pub(crate) fn import(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        context.library.import(&context.settings.import);

        if context.settings.external_storage.enabled
            && context.settings.external_storage.auto_import
        {
            match context
                .library
                .import_from_external(&context.settings.external_storage)
            {
                Ok(count) if count > 0 => {
                    let msg = format!("Imported {} file(s) from SD card", count);
                    hub.send(Event::Render(msg)).ok();
                }
                Err(e) => {
                    log_error!("External storage import error: {}", e);
                }
                _ => {}
            }
        }

        context.library.sort(self.sort_method, self.reverse_order);
        self.refresh_visibles(true, false, hub, rq, context);
    }

    pub(crate) fn clean_up(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        context.library.clean_up();
        self.refresh_visibles(true, false, hub, rq, context);
    }

    pub(crate) fn flush(&mut self, context: &mut Context) {
        context.library.flush();
    }
}
