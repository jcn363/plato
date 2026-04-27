// Plato - Document reader for Kobo devices
// Copyright (C) 2025 Pedro Depasco
// Licensed under the GPL-3.0 License.

//! Create collection dialog for collection management

use super::button::Button;
use super::label::Label;
use super::{Align, Bus, EntryId, Event, Hub, Id, RenderQueue, View, ViewId, ID_FEEDER};
use super::BORDER_RADIUS_MEDIUM;
use crate::color;
use crate::context::Context;
use crate::font::{font_from_style, Fonts, NORMAL_STYLE};
use crate::framebuffer::Framebuffer;
use crate::geom::{CornerSpec, Rectangle};
use crate::gesture::GestureEvent;
use crate::unit::scale_by_dpi;
use crate::view::input_field::InputField;

const LABEL_CANCEL: &str = "Cancel";
const LABEL_CREATE: &str = "Create";
const LABEL_NAME: &str = "Name:";

/// Create collection dialog
pub struct CreateCollectionDialog {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    _view_id: ViewId,
    will_close: bool,
    collection_name: String,
}

impl CreateCollectionDialog {
    pub fn new(context: &mut Context) -> CreateCollectionDialog {
        let id = ID_FEEDER.next();
        let view_id = ViewId::CreateCollectionDialog;
        let mut children: Vec<Box<dyn View>> = Vec::new();
        let dpi = crate::unit::get_device_dpi();
        let (width, height) = context.display.dims;

        let font = font_from_style(&mut context.fonts, &NORMAL_STYLE, dpi);
        let x_height = font.x_heights.0 as i32;
        let padding = font.em() as i32;

        let input_height = 4 * x_height;
        let button_height = 4 * x_height;
        let max_button_width = width as i32 / 4;

        // Calculate dialog dimensions
        let dialog_width = width as i32 / 2;
        let dialog_height = input_height * 2 + button_height + 6 * padding;

        let dx = (width as i32 - dialog_width) / 2;
        let dy = (height as i32 - dialog_height) / 2;
        let rect = rect![dx, dy, dx + dialog_width, dy + dialog_height];

        // Add Name: label and input
        let y_start = rect.min.y + padding;
        let rect_label_name = rect![
            rect.min.x + padding,
            y_start,
            rect.min.x + padding + font.plan(LABEL_NAME, None, None).width,
            y_start + input_height
        ];
        let label_name = Label::new(rect_label_name, LABEL_NAME.to_string(), Align::Left(0));
        children.push(Box::new(label_name));

        let rect_input_name = rect![
            rect.min.x + 2 * padding + font.plan(LABEL_NAME, None, None).width,
            y_start,
            rect.max.x - padding,
            y_start + input_height
        ];
        let input_name = InputField::new(rect_input_name, ViewId::CreateCollectionDialog)
            .border(true)
            .placeholder("Collection name");
        children.push(Box::new(input_name));

        // Add Cancel button
        let button_width = font.plan(LABEL_CANCEL, Some(max_button_width), None).width + padding;
        let rect_cancel = rect![
            rect.min.x + padding,
            rect.max.y - button_height - padding,
            rect.min.x + padding + button_width,
            rect.max.y - padding
        ];
        let cancel_button = Button::new(rect_cancel, Event::Cancel, LABEL_CANCEL.to_string());
        children.push(Box::new(cancel_button));

        // Add Create button
        let rect_create = rect![
            rect.max.x - padding - button_width,
            rect.max.y - button_height - padding,
            rect.max.x - padding,
            rect.max.y - padding
        ];
        let create_button = Button::new(rect_create, Event::Validate, LABEL_CREATE.to_string());
        children.push(Box::new(create_button));

        CreateCollectionDialog {
            id,
            rect,
            children,
            _view_id: view_id,
            will_close: false,
            collection_name: String::new(),
        }
    }

    pub fn get_collection_name(&self) -> &str {
        &self.collection_name
    }
}

impl View for CreateCollectionDialog {
    fn id(&self) -> Id {
        self.id
    }

    fn handle_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Validate => {
                // Get the collection name from the input field
                if let Some(input_field) = self.children[1].downcast_ref::<InputField>() {
                    self.collection_name = input_field.get_text().to_string();
                    if !self.collection_name.trim().is_empty() {
                        hub.send(Event::Select(EntryId::CreateCollection)).ok();
                        self.will_close = true;
                    }
                }
                true
            }
            Event::Cancel => {
                self.will_close = true;
                bus.push_back(Event::Close(ViewId::CreateCollectionDialog));
                true
            }
            Event::Submit(ViewId::CreateCollectionDialog, ref text) => {
                self.collection_name = text.clone();
                true
            }
            Event::Focus(Some(ViewId::CreateCollectionDialog)) => {
                if let Some(input_field) = self.children[1].downcast_mut::<InputField>() {
                    input_field.handle_event(event, hub, bus, rq, context);
                }
                true
            }
            Event::Gesture(GestureEvent::Tap(center)) if self.rect.includes(*center) => {
                if let Some(index) = self.children.iter().position(|c| c.rect().includes(*center)) {
                    self.children[index].handle_event(event, hub, bus, rq, context);
                }
                true
            }
            Event::Gesture(GestureEvent::Tap(center)) if !self.rect.includes(*center) => {
                self.will_close = true;
                bus.push_back(Event::Close(ViewId::CreateCollectionDialog));
                true
            }
            _ => {
                for child in self.children.iter_mut() {
                    if child.handle_event(event, hub, bus, rq, context) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        let dpi = crate::unit::get_device_dpi();
        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;
        let corners = CornerSpec::Uniform(border_radius);
        fb.draw_rounded_rectangle(
            &rect,
            &corners,
            color::WHITE,
        );
        for child in self.children.iter() {
            let child_rect = child.rect();
            if let Some(intersection) = rect.intersection(child_rect) {
                child.render(fb, intersection, fonts);
            }
        }
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
}
