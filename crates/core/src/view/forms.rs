//! PDF Forms UI View
//!
//! Provides UI for filling PDF forms on mobile/desktop platforms.
//! Supports text fields, checkboxes, radio buttons, dropdowns, and lists.

#![cfg(any(target_os = "android", target_os = "ios", target_os = "linux"))]

use crate::context::Context;
use crate::document::forms::{FormField, FormFieldType, FormParser, FormValues, FormExporter};
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use std::path::{Path, PathBuf};

use crate::unit::scale_by_dpi;
use crate::view::button::Button;
use crate::view::input_field::InputField;
use crate::view::label::Label;
use crate::view::top_bar::TopBar;
use crate::view::{Align, Bus, Event, Hub, RenderData, RenderQueue, View};
use crate::view::{EntryId, Id, ViewId, ID_FEEDER};
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
    _current_field_index: usize,
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
            // Show form fields with interactive components
            let mut y = content_y;
            for (i, field) in form_fields.iter().enumerate() {
                let field_label = format!("{}: {}", 
                    field.label.as_ref().unwrap_or(&field.name),
                    if field.required { "*" } else { "" }
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

                // Add interactive component based on field type
                match field.field_type {
                    FormFieldType::Text => {
                        let view_id = ViewId::FormsField(i as u16);
                        let _current_value = field.value.as_ref().unwrap_or(&String::new()).clone();
                        let input_rect = rect![
                            rect.min.x + PADDING,
                            y,
                            rect.max.x - PADDING,
                            y + BUTTON_HEIGHT
                        ];
                        let input_field = InputField::new(input_rect, view_id);
                        children.push(Box::new(input_field) as Box<dyn View>);
                        y += BUTTON_HEIGHT + BUTTON_SPACING;
                    }
                    FormFieldType::Checkbox => {
                        let is_checked = field.value.as_ref().map(|v| v == "true").unwrap_or(false);
                        let checkbox_btn = Button::new(
                            rect![
                                rect.min.x + PADDING,
                                y,
                                rect.min.x + PADDING + 200,
                                y + BUTTON_HEIGHT
                            ],
                            Event::Select(EntryId::ToggleCheckbox(field.name.clone())),
                            if is_checked { "☑️ Checked".to_string() } else { "☐ Unchecked".to_string() },
                        );
                        children.push(Box::new(checkbox_btn) as Box<dyn View>);
                        y += BUTTON_HEIGHT + BUTTON_SPACING;
                    }
                    FormFieldType::Radio => {
                        for (opt_i, option) in field.options.iter().enumerate() {
                            let is_selected = field.value.as_ref() == Some(option);
                            let radio_btn = Button::new(
                                rect![
                                    rect.min.x + PADDING,
                                    y,
                                    rect.min.x + PADDING + 300,
                                    y + BUTTON_HEIGHT
                                ],
                                Event::Select(EntryId::SelectRadio(field.name.clone(), opt_i)),
                                format!("{} {}", if is_selected { "⦿" } else { "○" }, option),
                            );
                            children.push(Box::new(radio_btn) as Box<dyn View>);
                            y += BUTTON_HEIGHT + BUTTON_SPACING;
                        }
                    }
                    FormFieldType::Dropdown => {
                        let current_value = field.value.as_ref().unwrap_or(&String::new()).clone();
                        let dropdown_label = Label::new(
                            rect![
                                rect.min.x + PADDING,
                                y,
                                rect.max.x - PADDING,
                                y + BUTTON_HEIGHT
                            ],
                            format!("Dropdown: {}", current_value),
                            Align::Left(0),
                        );
                        children.push(Box::new(dropdown_label) as Box<dyn View>);
                        y += BUTTON_HEIGHT + BUTTON_SPACING;
                    }
                    FormFieldType::List => {
                        let current_value = field.value.as_ref().unwrap_or(&String::new()).clone();
                        let list_label = Label::new(
                            rect![
                                rect.min.x + PADDING,
                                y,
                                rect.max.x - PADDING,
                                y + BUTTON_HEIGHT
                            ],
                            format!("List: {}", current_value),
                            Align::Left(0),
                        );
                        children.push(Box::new(list_label) as Box<dyn View>);
                        y += BUTTON_HEIGHT + BUTTON_SPACING;
                    }
                    FormFieldType::Signature => {
                        let signature_label = Label::new(
                            rect![
                                rect.min.x + PADDING,
                                y,
                                rect.max.x - PADDING,
                                y + BUTTON_HEIGHT
                            ],
                            "Signature field (tap to sign)".to_string(),
                            Align::Left(0),
                        );
                        children.push(Box::new(signature_label) as Box<dyn View>);
                        y += BUTTON_HEIGHT + BUTTON_SPACING;
                    }
                    FormFieldType::Button => {
                        let button = Button::new(
                            rect![
                                rect.min.x + PADDING,
                                y,
                                rect.min.x + PADDING + 200,
                                y + BUTTON_HEIGHT
                            ],
                            Event::Select(EntryId::ClickButton(field.name.clone())),
                            field.name.clone(),
                        );
                        children.push(Box::new(button) as Box<dyn View>);
                        y += BUTTON_HEIGHT + BUTTON_SPACING;
                    }
                }
            }
        }

        // Add save button at the bottom
        let save_y = rect.max.y - BUTTON_HEIGHT - PADDING;
        let save_btn = Button::new(
            rect![
                rect.min.x + PADDING,
                save_y,
                rect.max.x - PADDING,
                save_y + BUTTON_HEIGHT
            ],
            Event::Select(EntryId::Save),
            "💾 Save Form".to_string(),
        );
        children.push(Box::new(save_btn) as Box<dyn View>);

        rq.add(RenderData::new(id, rect, UpdateMode::Full));

        Ok(FormsView {
            id,
            rect,
            children,
            form_fields,
            form_values,
            _current_field_index: 0,
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
        bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match event {
            Event::Select(EntryId::ToggleCheckbox(field_name)) => {
                let new_value = if let Some(field) = self.form_fields.iter().find(|f| &f.name == field_name) {
                    let current = field.value.as_ref().map(|v| v == "true").unwrap_or(false);
                    format!("{}", !current)
                } else {
                    "true".to_string()
                };
                self.set_field_value(field_name, new_value);
                return true;
            }
            Event::Select(EntryId::SelectRadio(field_name, option_index)) => {
                if let Some(field) = self.form_fields.iter().find(|f| &f.name == field_name) {
                    if let Some(option) = field.options.get(*option_index) {
                        self.set_field_value(field_name, option.clone());
                        return true;
                    }
                }
            }
            Event::Select(EntryId::ClickButton(field_name)) => {
                // Handle button click - could trigger form submission or other actions
                bus.push_back(Event::Notify(format!("Button clicked: {}", field_name)));
                return true;
            }
            Event::Select(EntryId::SetValue(field_name, value)) => {
                self.set_field_value(field_name, value.clone());
                return true;
            }
            Event::Select(EntryId::Save) => {
                // Export form values to PDF
                let output_path = self.pdf_path.with_extension("_filled.pdf");
                match FormExporter::export_to_pdf(&self.pdf_path, &output_path, &self.form_values) {
                    Ok(_) => {
                        bus.push_back(Event::Notify(format!("Form saved to: {}", output_path.display())));
                    }
                    Err(e) => {
                        bus.push_back(Event::Notify(format!("Failed to save form: {}", e)));
                    }
                }
                return true;
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
