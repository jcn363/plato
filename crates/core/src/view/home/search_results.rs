//! Semantic search results UI view

use crate::color::{background, foreground, text_normal};
use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use crate::input::DeviceEvent;
use crate::view::label::Label;
use crate::view::{Align, Bus, Event, Hub, Id, RenderData, RenderQueue, View, ID_FEEDER};
use plato_search::search::SearchIndexer;

#[derive(Debug)]
pub struct SearchResults {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
}

impl SearchResults {
    pub fn new(rect: Rectangle, _context: &Context, query: &str, indexer: &SearchIndexer) -> Self {
        let id = ID_FEEDER.next();
        let dpi = crate::unit::get_device_dpi();
        let small_height = crate::unit::scale_by_dpi(40.0, dpi) as i32;
        let padding = crate::unit::scale_by_dpi(20.0, dpi) as i32;
        let mut children = Vec::new();
        let mut y = rect.min.y + padding;

        let title = Label::new(
            rect![rect.min.x + padding, y, rect.max.x - padding, y + small_height],
            format!("Results for '{}':", query),
            Align::Left(0),
        );
        children.push(Box::new(title) as Box<dyn View>);
        y += small_height + padding;

        if let Ok(results) = indexer.search(query, 5) {
            for (doc_id, score) in results {
                let res_label = Label::new(
                    rect![rect.min.x + padding, y, rect.max.x - padding, y + small_height],
                    format!("{} ({:.2})", doc_id, score),
                    Align::Left(0),
                );
                children.push(Box::new(res_label) as Box<dyn View>);
                y += small_height;
            }
        }

        SearchResults { id, rect, children }
    }
}

impl View for SearchResults {
    fn handle_event(&mut self, evt: &Event, _hub: &Hub, _bus: &mut Bus, _rq: &mut RenderQueue, _context: &mut Context) -> bool {
        match *evt {
            Event::Device(DeviceEvent::Finger { position, .. }) if self.rect.includes(position) => true,
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        fb.draw_rectangle(&rect, background(crate::theme::is_dark_mode()));
        for child in self.children.iter() {
            let child_rect = child.rect();
            if let Some(intersection) = rect.intersection(child_rect) {
                child.render(fb, intersection, fonts);
            }
        }
    }

    fn rect(&self) -> &Rectangle { &self.rect }
    fn rect_mut(&mut self) -> &mut Rectangle { &mut self.rect }
    fn children(&self) -> &Vec<Box<dyn View>> { &self.children }
    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> { &mut self.children }
    fn id(&self) -> Id { self.id }
}
