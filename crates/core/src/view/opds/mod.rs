use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::Framebuffer;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::opds::OPDSCatalog;
use crate::view::common::locate_by_id;
use crate::view::menu::{Menu, MenuKind};
use crate::view::top_bar::TopBar;
use crate::view::{
    Bus, EntryId, EntryKind, Event, Hub, Id, RenderData, RenderQueue, View, ViewId, ID_FEEDER,
};
use std::fs;

pub struct OpdsView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    catalog: Option<OPDSCatalog>,
    stack: Vec<String>,
    current_url: String,
}

impl OpdsView {
    pub fn new(rect: Rectangle, url: String, context: &mut Context) -> OpdsView {
        let id = ID_FEEDER.next();
        let mut children = Vec::new();

        let top_bar_rect = rect![
            rect.min,
            pt!(rect.max.x, rect.min.y + context.display.dims.1 as i32 / 15)
        ];
        let top_bar = TopBar::new(
            top_bar_rect,
            Event::Back,
            "OPDS Catalog".to_string(),
            context,
        );
        children.push(Box::new(top_bar) as Box<dyn View>);

        let mut view = OpdsView {
            id,
            rect,
            children,
            catalog: None,
            stack: Vec::new(),
            current_url: url.clone(),
        };

        view.load_catalog(&url, context);
        view
    }

    fn load_catalog(&mut self, url: &str, context: &mut Context) {
        match OPDSCatalog::new(url) {
            Ok(catalog) => {
                self.current_url = url.to_string();
                self.catalog = Some(catalog);
                self.update_content(context);
            }
            Err(e) => {
                crate::log_error!("Failed to load OPDS catalog: {}", e);
            }
        }
    }

    fn update_content(&mut self, context: &mut Context) {
        if let Some(index) = locate_by_id(self, ViewId::DirectoryMenu) {
            self.children.remove(index);
        }

        let Some(catalog) = &self.catalog else { return };

        let mut entries = Vec::new();
        for entry in catalog.entries() {
            let title = entry.title.clone();
            if let Some(url) = entry.catalog_url() {
                entries.push(EntryKind::Command(title, EntryId::OpenOpds(url)));
            } else if let Some(url) = entry.download_url() {
                entries.push(EntryKind::Command(
                    format!("Download: {}", title),
                    EntryId::DownloadOpds(url),
                ));
            }
        }

        let menu_rect = rect![
            self.rect.min.x,
            self.children[0].rect().max.y,
            self.rect.max.x,
            self.rect.max.y
        ];
        let menu = Menu::new(
            menu_rect,
            ViewId::DirectoryMenu,
            MenuKind::Contextual,
            entries,
            context,
        );
        self.children.push(Box::new(menu) as Box<dyn View>);
    }

    fn download_book(&self, url: &str, hub: &Hub, context: &Context) {
        let url = url.to_string();
        let library_path = context.settings.libraries[context.settings.selected_library]
            .path
            .clone();
        let downloads_path = library_path.join("Downloads");
        let hub = hub.clone();

        if !downloads_path.exists() {
            let _ = fs::create_dir_all(&downloads_path);
        }

        std::thread::spawn(move || match reqwest::blocking::get(&url) {
            Ok(response) => {
                let filename = url.split('/').next_back().unwrap_or("book.epub");
                let mut dest_path = downloads_path.join(filename);
                if !dest_path.to_string_lossy().contains('.') {
                    dest_path.set_extension("epub");
                }

                match fs::File::create(&dest_path) {
                    Ok(mut file) => {
                        if let Ok(bytes) = response.bytes() {
                            if std::io::copy(&mut bytes.as_ref(), &mut file).is_ok() {
                                hub.send(Event::Notify(format!("Downloaded {}", filename)))
                                    .ok();
                                hub.send(Event::Select(EntryId::Import)).ok();
                            }
                        }
                    }
                    Err(e) => {
                        crate::log_error!("Failed to create file {:?}: {}", dest_path, e);
                    }
                }
            }
            Err(e) => {
                crate::log_error!("Failed to download book: {}", e);
            }
        });
    }
}

impl View for OpdsView {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        _bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Back => {
                if let Some(url) = self.stack.pop() {
                    self.load_catalog(&url, context);
                    rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                    true
                } else {
                    false
                }
            }
            Event::Select(EntryId::OpenOpds(ref url)) => {
                let old_url = self.current_url.clone();
                self.stack.push(old_url);
                self.load_catalog(url, context);
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::Select(EntryId::DownloadOpds(ref url)) => {
                self.download_book(url, hub, context);
                true
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        for child in &self.children {
            if let Some(intersection) = child.rect().intersection(&rect) {
                child.render(fb, intersection, fonts);
            }
        }
    }

    fn resize(&mut self, rect: Rectangle, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let top_bar_height = rect.height() as i32 / 15;
        let top_bar_rect = rect![rect.min, pt!(rect.max.x, rect.min.y + top_bar_height)];
        self.children[0].resize(top_bar_rect, hub, rq, context);
        if self.children.len() > 1 {
            let menu_rect = rect![rect.min.x, top_bar_rect.max.y, rect.max.x, rect.max.y];
            self.children[1].resize(menu_rect, hub, rq, context);
        }
        self.rect = rect;
    }

    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        &mut self.rect
    }

    fn children(&self) -> &Vec<Box<dyn View>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> {
        &mut self.children
    }

    fn id(&self) -> Id {
        self.id
    }
}
