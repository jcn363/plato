use crate::view::{Align, Bus, EntryId, Event, Hub, Id, RenderData, RenderQueue, View, ID_FEEDER};
use crate::color::background;
use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use crate::gesture::GestureEvent;
use crate::input::DeviceEvent;
use crate::unit::scale_by_dpi;
use crate::view::button::Button;
use crate::view::label::Label;

#[derive(Debug)]
pub struct SearchMenu {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
}

impl SearchMenu {
    pub fn new(rect: Rectangle, _context: &mut Context) -> SearchMenu {
        let id = ID_FEEDER.next();
        let dpi = crate::unit::get_device_dpi();
        let small_height = scale_by_dpi(40.0, dpi) as i32;
        let padding = scale_by_dpi(20.0, dpi) as i32;
        let mut children = Vec::new();
        let mut y = rect.min.y + padding;

        // Title
        let title = Label::new(
            rect![rect.min.x + padding, y, rect.max.x - padding, y + small_height],
            "Advanced Search".to_string(),
            Align::Left(padding),
        );
        children.push(Box::new(title) as Box<dyn View>);
        y += small_height;

        // Search fields
        let fields = [
            ("Title", EntryId::SearchTitle),
            ("Series", EntryId::SearchSeries),
            ("Publisher", EntryId::SearchPublisher),
            ("Year", EntryId::SearchYear),
        ];

        for (label, entry_id) in fields {
            let label_rect = rect![rect.min.x + padding, y, rect.min.x + 150, y + small_height];
            let label = Label::new(label_rect, label.to_string(), Align::Left(0));
            children.push(Box::new(label) as Box<dyn View>);

            let button_rect = rect![rect.min.x + 160, y, rect.max.x - padding, y + small_height];
            let button = Button::new(
                button_rect,
                Event::Select(entry_id),
                "Add".to_string(),
            );
            children.push(Box::new(button) as Box<dyn View>);
            y += small_height;
        }

        // Status filters
        y += padding;
        let status_fields = [
            ("Reading", EntryId::ToggleSearchReading),
            ("New", EntryId::ToggleSearchNew),
            ("Finished", EntryId::ToggleSearchFinished),
        ];

        for (label, entry_id) in status_fields {
            let label_rect = rect![rect.min.x + padding, y, rect.min.x + 150, y + small_height];
            let label = Label::new(label_rect, label.to_string(), Align::Left(0));
            children.push(Box::new(label) as Box<dyn View>);

            let button_rect = rect![rect.min.x + 160, y, rect.max.x - padding, y + small_height];
            let button = Button::new(
                button_rect,
                Event::Select(entry_id),
                "Off".to_string(),
            );
            children.push(Box::new(button) as Box<dyn View>);
            y += small_height;
        }

        // Clear button
        y += padding;
        let clear_rect = rect![rect.min.x + padding, y, rect.max.x - padding, y + small_height];
        let clear = Button::new(
            clear_rect,
            Event::Select(EntryId::ClearSearchFilters),
            "Clear All Filters".to_string(),
        );
        children.push(Box::new(clear) as Box<dyn View>);

        SearchMenu {
            id,
            rect,
            children,
        }
    }
}

impl View for SearchMenu {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        _bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Gesture(GestureEvent::Tap(center))
            | Event::Gesture(GestureEvent::HoldFingerShort(center, ..))
                if self.rect.includes(center) =>
            {
                true
            }
            Event::Gesture(GestureEvent::Swipe { start, .. }) if self.rect.includes(start) => true,
            Event::Device(DeviceEvent::Finger { position, .. }) if self.rect.includes(position) => {
                true
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        fb.draw_rectangle(
            &rect,
            background(crate::theme::is_dark_mode()),
        );
        for child in self.children.iter() {
            let child_rect = child.rect();
            if let Some(intersection) = rect.intersection(child_rect) {
                child.render(fb, intersection, fonts);
            }
        }
    }

    fn resize(&mut self, rect: Rectangle, _hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        self.rect = rect;
        // Rebuild children with new rect
        let new_menu = Self::new(rect, context);
        self.children = new_menu.children;
        rq.add(RenderData::new(self.id, rect, UpdateMode::Gui));
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
