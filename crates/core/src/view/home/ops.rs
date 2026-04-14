//! Home Document Operations
//!
//! Handles CRUD operations on documents in the library.

use std::fs;
use std::path::Path;

use anyhow::Error;

use crate::context::Context;
use crate::library::Library;
use crate::log_error;
use crate::metadata::{sort, Info, SimpleStatus, SortMethod};
use crate::settings::LibraryMode;
use crate::view::common::locate_by_id;
use crate::view::menu_entry::MenuEntry;
use crate::view::{Hub, RenderQueue, View, ViewId};

use super::Home;

pub const TRASH_DIRNAME: &str = ".trash";

impl Home {
    pub fn add_document(
        &mut self,
        info: Info,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        context.library.add_document(info);
        self.sort(false, hub, rq, context);
        self.refresh_visibles(true, false, hub, rq, context);
    }

    pub fn set_status(
        &mut self,
        path: &Path,
        status: SimpleStatus,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        context.library.set_status(path, status);
        if self.sort_method.is_status_related() {
            self.sort(false, hub, rq, context);
        }
        self.refresh_visibles(true, false, hub, rq, context);
    }

    pub fn empty_trash(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let trash_path = context.library.home.join(TRASH_DIRNAME);
        let trash = Library::new(trash_path, LibraryMode::Database)
            .map_err(|e| log_error!("Can't load trash: {:#}.", e));
        let mut trash = if let Ok(trash) = trash {
            trash
        } else {
            return;
        };

        let (files, _) = trash.list(&trash.home, None, false);
        if files.is_empty() {
            return;
        }

        let mut count = 0;
        for info in files {
            match trash.remove(&info.file.path) {
                Err(e) => log_error!("Can't erase {}: {:#}.", info.file.path.display(), e),
                Ok(()) => count += 1,
            }
        }
        trash.flush();
        let message = format!(
            "Removed {} book{}.",
            count,
            if count != 1 { "s" } else { "" }
        );
        let notif = crate::view::notification::Notification::new(message, hub, rq, context);
        self.children.push(Box::new(notif) as Box<dyn View>);
    }

    pub fn rename(
        &mut self,
        path: &Path,
        file_name: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<(), Error> {
        context.library.rename(path, file_name)?;
        self.refresh_visibles(true, false, hub, rq, context);
        Ok(())
    }

    pub fn remove(
        &mut self,
        path: &Path,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<(), Error> {
        let full_path = context.library.home.join(path);
        if full_path.exists() {
            let trash_path = context.library.home.join(TRASH_DIRNAME);
            if !trash_path.is_dir() {
                fs::create_dir(&trash_path)?;
            }
            let mut trash = Library::new(trash_path, LibraryMode::Database)?;
            context.library.move_to(path, &mut trash)?;
            let (mut files, _) = trash.list(&trash.home, None, false);
            let mut size = files.iter().map(|info| info.file.size).sum::<u64>();
            if size > context.settings.home.max_trash_size {
                sort(&mut files, SortMethod::Added, true);
                while size > context.settings.home.max_trash_size {
                    if let Some(info) = files.pop() {
                        if let Err(e) = trash.remove(&info.file.path) {
                            log_error!("Can't erase {}: {:#}", info.file.path.display(), e);
                            break;
                        }
                        size -= info.file.size;
                    } else {
                        break;
                    }
                }
            }
            trash.flush();
        } else {
            context.library.remove(path)?;
        }
        self.refresh_visibles(true, false, hub, rq, context);
        Ok(())
    }

    pub fn copy_to(
        &mut self,
        path: &Path,
        index: usize,
        context: &mut Context,
    ) -> Result<(), Error> {
        let library_settings = &context.settings.libraries[index];
        let mut library = Library::new(&library_settings.path, library_settings.mode)?;
        context.library.copy_to(path, &mut library)?;
        library.flush();
        Ok(())
    }

    pub fn move_to(
        &mut self,
        path: &Path,
        index: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<(), Error> {
        let library_settings = &context.settings.libraries[index];
        let mut library = Library::new(&library_settings.path, library_settings.mode)?;
        context.library.move_to(path, &mut library)?;
        library.flush();
        self.refresh_visibles(true, false, hub, rq, context);
        Ok(())
    }

    pub fn set_reverse_order(
        &mut self,
        value: bool,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        self.reverse_order = value;
        self.current_page = 0;
        self.sort(true, hub, rq, context);
    }

    pub fn set_sort_method(
        &mut self,
        sort_method: SortMethod,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        self.sort_method = sort_method;
        self.reverse_order = sort_method.reverse_order();

        if let Some(index) = locate_by_id(self, ViewId::SortMenu) {
            if let Some(entry) = self
                .child_mut(index)
                .children_mut()
                .last_mut()
                .and_then(|c| c.downcast_mut::<MenuEntry>())
            {
                entry.update(sort_method.reverse_order(), rq);
            }
        }

        self.current_page = 0;
        self.sort(true, hub, rq, context);
    }

    pub fn sort(&mut self, update: bool, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        context.library.sort(self.sort_method, self.reverse_order);
        sort(
            &mut self.visible_books,
            self.sort_method,
            self.reverse_order,
        );

        if update {
            self.update_shelf(false, hub, rq, context);
            let search_visible =
                crate::view::common::rlocate::<crate::view::search_bar::SearchBar>(self).is_some();
            self.update_top_bar(search_visible, rq);
            self.update_bottom_bar(rq, context);
        }
    }
}
