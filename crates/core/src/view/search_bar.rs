use super::filler::Filler;
use super::icon::Icon;
use super::input_field::InputField;
use super::{Bus, Event, Hub, Id, RenderQueue, View, ViewId, ID_FEEDER, THICKNESS_MEDIUM};
use crate::color::{separator, text_bump_small};
use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::Framebuffer;
use crate::geom::Rectangle;
use crate::gesture::GestureEvent;
use crate::input::DeviceEvent;
use crate::unit::scale_by_dpi;
use crate::view::reader::results_label::ResultsLabel;

#[derive(Debug)]
pub struct SearchBar {
    id: Id,
    pub rect: Rectangle,
    children: Vec<Box<dyn View>>,
    results_label_index: Option<usize>,
}

impl SearchBar {
    pub fn new(
        rect: Rectangle,
        input_id: ViewId,
        placeholder: &str,
        text: &str,
        context: &mut Context,
    ) -> SearchBar {
        let id = ID_FEEDER.next();
        let (thickness, side) = Self::calculate_metrics(&rect);
        let mut children = Vec::new();

        Self::add_search_icon(&mut children, &rect, side);
        Self::add_left_separator(&mut children, &rect, thickness, side);
        Self::add_input_field(
            &mut children,
            &rect,
            thickness,
            side,
            input_id,
            text,
            placeholder,
            context,
        );
        Self::add_right_separator(&mut children, &rect, thickness, side);
        Self::add_close_icon(&mut children, &rect, side);

        // Add ResultsLabel to display search results count
        let results_label = ResultsLabel::new(rect![rect.min, rect.min], 0, false);
        let results_label_index = children.len();
        children.push(Box::new(results_label) as Box<dyn View>);

        SearchBar {
            id,
            rect,
            children,
            results_label_index: Some(results_label_index),
        }
    }

    fn calculate_metrics(rect: &Rectangle) -> (i32, i32) {
        let dpi = crate::unit::get_device_dpi();
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let side = rect.height() as i32;
        (thickness, side)
    }

    fn add_search_icon(children: &mut Vec<Box<dyn View>>, rect: &Rectangle, side: i32) {
        let search_rect = rect![rect.min, rect.min + side];
        let search_icon = Icon::new(
            "search",
            search_rect,
            Event::ToggleNear(ViewId::SearchMenu, search_rect),
        )
        .background(text_bump_small(crate::theme::is_dark_mode())[0]);
        children.push(Box::new(search_icon) as Box<dyn View>);
    }

    fn add_left_separator(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        thickness: i32,
        side: i32,
    ) {
        let sep = Filler::new(
            rect![
                pt!(rect.min.x + side, rect.min.y),
                pt!(rect.min.x + side + thickness, rect.max.y)
            ],
            separator(crate::theme::is_dark_mode()),
        );
        children.push(Box::new(sep) as Box<dyn View>);
    }

    fn add_input_field(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        thickness: i32,
        side: i32,
        input_id: ViewId,
        text: &str,
        placeholder: &str,
        context: &mut Context,
    ) {
        let input_field = InputField::new(
            rect![
                pt!(rect.min.x + side + thickness, rect.min.y),
                pt!(rect.max.x - side - thickness, rect.max.y)
            ],
            input_id,
        )
        .border(false)
        .text(text, context)
        .placeholder(placeholder);
        children.push(Box::new(input_field) as Box<dyn View>);
    }

    fn add_right_separator(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        thickness: i32,
        side: i32,
    ) {
        let sep = Filler::new(
            rect![
                pt!(rect.max.x - side - thickness, rect.min.y),
                pt!(rect.max.x - side, rect.max.y)
            ],
            separator(crate::theme::is_dark_mode()),
        );
        children.push(Box::new(sep) as Box<dyn View>);
    }

    fn add_close_icon(children: &mut Vec<Box<dyn View>>, rect: &Rectangle, side: i32) {
        let close_icon = Icon::new(
            "close",
            rect![
                pt!(rect.max.x - side, rect.min.y),
                pt!(rect.max.x, rect.max.y)
            ],
            Event::Close(ViewId::SearchBar),
        )
        .background(text_bump_small(crate::theme::is_dark_mode())[0]);
        children.push(Box::new(close_icon) as Box<dyn View>);
    }

    pub fn set_text(&mut self, text: &str, rq: &mut RenderQueue, context: &mut Context) {
        if let Some(input_field) = self.children[2].downcast_mut::<InputField>() {
            input_field.set_text(text, true, rq, context);
        }
    }

    pub fn update_results(
        &mut self,
        count: usize,
        completed: bool,
        hub: &Hub,
        rq: &mut RenderQueue,
    ) {
        if let Some(index) = self.results_label_index {
            if let Some(results_label) = self.children[index].downcast_mut::<ResultsLabel>() {
                results_label.update(count, rq);
                if completed {
                    hub.send(Event::EndOfSearch).ok();
                }
            }
        }
    }
}

impl View for SearchBar {
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

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {}

    fn resize(&mut self, rect: Rectangle, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let dpi = crate::unit::get_device_dpi();
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let side = rect.height() as i32;
        self.children[0].resize(rect![rect.min, rect.min + side], hub, rq, context);
        self.children[1].resize(
            rect![
                pt!(rect.min.x + side, rect.min.y),
                pt!(rect.min.x + side + thickness, rect.max.y)
            ],
            hub,
            rq,
            context,
        );
        self.children[2].resize(
            rect![
                pt!(rect.min.x + side + thickness, rect.min.y),
                pt!(rect.max.x - side - thickness, rect.max.y)
            ],
            hub,
            rq,
            context,
        );
        self.children[3].resize(
            rect![
                pt!(rect.max.x - side - thickness, rect.min.y),
                pt!(rect.max.x - side, rect.max.y)
            ],
            hub,
            rq,
            context,
        );
        self.children[4].resize(
            rect![
                pt!(rect.max.x - side, rect.min.y),
                pt!(rect.max.x, rect.max.y)
            ],
            hub,
            rq,
            context,
        );
        // Resize ResultsLabel (hidden by default, positioned at origin)
        if let Some(index) = self.results_label_index {
            self.children[index].resize(rect![rect.min, rect.min], hub, rq, context);
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
