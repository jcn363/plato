//! PDF Forms UI View
//!
//! Provides UI for filling PDF forms on mobile/desktop platforms.
//! Supports text fields, checkboxes, radio buttons, dropdowns, and lists.

#![cfg(any(target_os = "android", target_os = "ios", target_os = "linux"))]

use crate::context::Context;
use crate::document::forms::{FormField, FormParser, FormValues};
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use std::path::{Path, PathBuf};

use crate::unit::scale_by_dpi;
use crate::view::label::Label;
use crate::view::top_bar::TopBar;
use crate::view::{Align, Bus, Event, Hub, RenderData, RenderQueue, View};
use crate::view::{EntryId, Id, ID_FEEDER};
use crate::view::{SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};
use anyhow::Error;

pub const BUTTON_HEIGHT: i32 = 48;
pub const BUTTON_SPACING: i32 = 12;
pub const PADDING: i32 = 16;

pub struct FormsView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    form_fields: Vec<FormField>,
    form_values: FormValues,
    current_field_index: usize,
    pdf_path: PathBuf,
}

impl FormsView {
    pub fn new(
        rect: Rectangle,
        pdf_path: &Path,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<FormsView, Error> {
        let id = ID_FEEDER.next();
        let dpi = crate::unit::get_device_dpi();
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;

        // Parse form fields from PDF
        let form_fields = FormParser::parse_document(pdf_path)
            .unwrap_or_default();

        let form_values = FormValues::new();

        let mut children = Vec::new();

        let top_bar = TopBar::new(
            rect![
                rect.min.x,
                rect.min.y,
                rect.max.x,
                rect.min.y + small_height
            ],
            Event::Back,
            "PDF Forms".to_string(),
            context,
        );
        children.push(Box::new(top_bar) as Box<dyn View>);

        let content_y = rect.min.y + small_height + thickness;

        if form_fields.is_empty() {
            let no_forms_label = Label::new(
                rect![
                    rect.min.x + PADDING,
                    content_y,
                    rect.max.x - PADDING,
                    content_y + BUTTON_HEIGHT
                ],
                "No form fields found in this PDF.".to_string(),
                Align::Left(0),
            );
            children.push(Box::new(no_forms_label) as Box<dyn View>);
        } else {
            // Show form fields list
            let mut y = content_y;
            for (i, field) in form_fields.iter().enumerate() {
                let field_label = format!("{}: {} ({:?})", 
                    i + 1, 
                    field.label.as_ref().unwrap_or(&field.name),
                    field.field_type
                );
                let label = Label::new(
                    rect![
                        rect.min.x + PADDING,
                        y,
                        rect.max.x - PADDING,
                        y + BUTTON_HEIGHT
                    ],
                    field_label,
                    Align::Left(0),
                );
                children.push(Box::new(label) as Box<dyn View>);
                y += BUTTON_HEIGHT + BUTTON_SPACING;
            }
        }

        rq.add(RenderData::new(id, rect, UpdateMode::Full));

        Ok(FormsView {
            id,
            rect,
            children,
            form_fields,
            form_values,
            current_field_index: 0,
            pdf_path: pdf_path.to_path_buf(),
        })
    }

    pub fn set_field_value(&mut self, field_name: &str, value: String) {
        self.form_values.set(field_name, value);
    }

    pub fn get_form_values(&self) -> &FormValues {
        &self.form_values
    }

    pub fn is_complete(&self) -> bool {
        self.form_values.is_complete(&self.form_fields)
    }
}

impl View for FormsView {
    fn handle_event(
        &mut self,
        event: &Event,
        _hub: &Hub,
        _bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match event {
            Event::Select(EntryId::NextField) => {
                if self.current_field_index + 1 < self.form_fields.len() {
                    self.current_field_index += 1;
                    return true;
                }
            }
            Event::Select(EntryId::PreviousField) => {
                if self.current_field_index > 0 {
                    self.current_field_index -= 1;
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
        // Rendering handled by children
    }

    fn resize(&mut self, rect: Rectangle, _hub: &Hub, _rq: &mut RenderQueue, _context: &mut Context) {
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
