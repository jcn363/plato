//! AI Chat View for Plato Reader
//!
//! Provides an AI chat sidebar in the reader for asking questions about the current document.

use crate::context::Context;
use crate::geom::Rectangle;
use crate::view::{Bus, EntryId, Event, Hub, Id, RenderQueue, View, ViewId};

pub struct AiChatView {
    id: Id,
    rect: Rectangle,
}

impl AiChatView {
    pub fn new(rect: Rectangle) -> AiChatView {
        let id = crate::view::ID_FEEDER.next();
        AiChatView { id, rect }
    }
}

impl View for AiChatView {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }

    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        &mut self.rect
    }

    fn children(&self) -> &Vec<Box<dyn View>> {
        &Vec::new()
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> {
        &mut Vec::new()
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

    fn is_empty(&self) -> bool {
        true
    }
}
