//! AI Chat View for Plato Reader
//!
//! Provides an AI chat sidebar in the reader for asking questions about the current document.

use super::{Bus, Event, Hub, Id, RenderQueue, View, ViewId, ID_FEEDER};
use crate::color::text_inverted_hard;
use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::Framebuffer;
use crate::geom::Rectangle;
use crate::theme;

pub struct AiChatView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
}

impl AiChatView {
    pub fn new(rect: Rectangle) -> AiChatView {
        let id = ID_FEEDER.next();
        AiChatView {
            id,
            rect,
            children: Vec::new(),
        }
    }

    pub fn show(rect: Rectangle, _context: &mut Context) -> Box<dyn View> {
        Box::new(AiChatView::new(rect))
    }
}

impl View for AiChatView {
    fn id(&self) -> Id {
        self.id
    }

    fn view_id(&self) -> Option<ViewId> {
        Some(ViewId::AiChat)
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

    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match evt {
            Event::Close(ViewId::AiChat) => {
                bus.push_back(Event::Close(ViewId::AiChat));
                true
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
        let scheme = text_inverted_hard(theme::is_dark_mode());
        fb.draw_rectangle(&self.rect, scheme[0]);
    }
}
