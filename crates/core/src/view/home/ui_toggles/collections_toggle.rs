//! Collections Toggle Module
//!
//! This module handles collection management UI for the Home view.

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::metadata::{Collection, Collections};
use crate::view::menu::{Menu, MenuKind};
use crate::view::{EntryId, EntryKind, Event, Hub, RenderData, RenderQueue, View, ViewId};

pub trait HomeCollectionsExt {
    fn toggle_collections_menu(
        &mut self,
        enable: Option<bool>,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    );
    fn handle_collections_menu_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool;
    fn create_collection(
        &mut self,
        name: String,
        parent_id: Option<String>,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    );
    fn delete_collection(
        &mut self,
        id: String,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    );
    fn get_current_collection(&self) -> Option<Collection>;
    fn filter_by_collection(&self, fingerprints: Vec<crate::helpers::Fp>) -> Vec<crate::helpers::Fp>;
    fn hide_collections_menu(&mut self, rq: &mut RenderQueue, context: &mut Context);
    fn show_collections_menu(&mut self, rq: &mut RenderQueue, context: &mut Context);
    fn calculate_collections_menu_rect(&self, context: &Context) -> Rectangle;
    fn create_collections_menu(&self, rect: Rectangle, context: &mut Context) -> Menu;
    fn handle_collections_selection(
        &mut self,
        entry_id: &EntryId,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    );
}

use super::super::Home;

/// Collections toggle configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CollectionsToggleConfig {
    pub show_collections: bool,
    pub allow_nested: bool,
    pub max_depth: u8,
}

impl Default for CollectionsToggleConfig {
    fn default() -> Self {
        Self {
            show_collections: true,
            allow_nested: true,
            max_depth: 3,
        }
    }
}

/// Collections toggle state
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct CollectionsToggleState {
    pub visible: bool,
    pub active: bool,
    pub config: CollectionsToggleConfig,
    pub selected_collection: Option<String>,
}

impl HomeCollectionsExt for Home {
    /// Toggle collections menu visibility
    fn toggle_collections_menu(
        &mut self,
        enable: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(self.collections_menu.is_none());

        if should_enable {
            self.show_collections_menu(rq, context);
        } else {
            self.hide_collections_menu(rq, context);
        }
    }

    /// Show collections menu
    fn show_collections_menu(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.collections_menu.is_some() {
            return;
        }

        let rect = self.calculate_collections_menu_rect(context);
        let menu = self.create_collections_menu(rect, context);

        self.collections_menu = Some(Box::new(menu) as Box<dyn View>);
        self.focus = Some(ViewId::CollectionsMenu);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide collections menu
    fn hide_collections_menu(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
        if self.collections_menu.is_none() {
            return;
        }

        self.collections_menu = None;
        self.focus = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate collections menu rectangle
    fn calculate_collections_menu_rect(&self, context: &Context) -> Rectangle {
        let screen_width = context.display.dims.0 as i32;
        let screen_height = context.display.dims.1 as i32;

        let width = (screen_width as f32 * 0.5) as i32;
        let height = (screen_height as f32 * 0.7) as i32;
        let x = (screen_width - width) / 2;
        let y = (screen_height - height) / 2;

        rect![x, y, width, height]
    }

    /// Create collections menu
    fn create_collections_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let mut entries = vec![];

        // Add "All Books" option
        entries.push(EntryKind::Command(
            "All Books".to_string(),
            EntryId::AllBooks,
        ));

        entries.push(EntryKind::Separator);

        // Add top-level collections
        if let Some(library) = &self.library {
            let top_level = library.collections.top_level();
            for collection in top_level {
                entries.push(EntryKind::Command(
                    collection.name.clone(),
                    EntryId::Collection(collection.id.clone()),
                ));
            }
        }

        // Add "Create New Collection" option
        entries.push(EntryKind::Separator);
        entries.push(EntryKind::Command(
            "Create New Collection".to_string(),
            EntryId::CreateCollection,
        ));

        Menu::new(
            rect,
            ViewId::CollectionsMenu,
            MenuKind::DropDown,
            entries,
            context,
        )
    }

    /// Handle collections menu events
    fn handle_collections_menu_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Close(ViewId::CollectionsMenu) => {
                self.hide_collections_menu(rq, context);
                true
            }
            Event::Select(ref entry_id) => {
                self.handle_collections_selection(entry_id, hub, rq, context);
                true
            }
            _ => false,
        }
    }

    /// Handle collections menu selection
    fn handle_collections_selection(
        &mut self,
        entry_id: &EntryId,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        match entry_id {
            EntryId::AllBooks => {
                self.current_collection_id = None;
                self.hide_collections_menu(rq, context);
                hub.send(Event::Back).ok();
            }
            EntryId::Collection(id) => {
                self.current_collection_id = Some(id.clone());
                self.hide_collections_menu(rq, context);
                hub.send(Event::Back).ok();
            }
            EntryId::CreateCollection => {
                self.hide_collections_menu(rq, context);
                hub.send(Event::Show(ViewId::CreateCollectionDialog)).ok();
            }
            _ => {
                self.hide_collections_menu(rq, context);
            }
        }
    }

    /// Create new collection
    fn create_collection(
        &mut self,
        name: String,
        parent_id: Option<String>,
        hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        if let Some(library) = &mut self.library {
            let id = Collections::generate_id();
            let now = chrono::Local::now().naive_local().to_string();

            let collection = Collection {
                id: id.clone(),
                name,
                parent_id,
                color: None,
                icon: Some("folder".to_string()),
                rules: None,
                created_at: now.clone(),
                modified_at: now,
            };

            if library.collections.add(collection).is_ok() {
                hub.send(Event::Notify("Collection created".to_string())).ok();
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
            }
        }
    }

    /// Delete collection
    fn delete_collection(
        &mut self,
        id: String,
        hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        if let Some(library) = &mut self.library {
            if library.collections.remove(&id).is_ok() {
                // Reset current collection if it was deleted
                if self.current_collection_id.as_ref() == Some(&id) {
                    self.current_collection_id = None;
                }
                hub.send(Event::Notify("Collection deleted".to_string())).ok();
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
            }
        }
    }

    /// Get current collection
    fn get_current_collection(&self) -> Option<Collection> {
        if let Some(id) = &self.current_collection_id {
            if let Some(library) = &self.library {
                library.collections.get(id).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Filter books by current collection
    fn filter_by_collection(&self, fingerprints: Vec<crate::helpers::Fp>) -> Vec<crate::helpers::Fp> {
        if let Some(collection) = self.get_current_collection() {
            if let Some(_rules) = &collection.rules {
                // Smart collection - filter by rules
                if let Some(library) = &self.library {
                    fingerprints
                        .into_iter()
                        .filter(|fp| {
                            if let Some(info) = library.db.get(fp) {
                                library.collections.matches_smart_collection(&collection.id, info)
                            } else {
                                false
                            }
                        })
                        .collect()
                } else {
                    fingerprints
                }
            } else {
                // Regular collection - filter by collection_id in metadata
                if let Some(library) = &self.library {
                    fingerprints
                        .into_iter()
                        .filter(|fp| {
                            if let Some(info) = library.db.get(fp) {
                                info.collection.as_ref() == Some(&collection.id)
                            } else {
                                false
                            }
                        })
                        .collect()
                } else {
                    fingerprints
                }
            }
        } else {
            // No collection selected - show all
            fingerprints
        }
    }
}
