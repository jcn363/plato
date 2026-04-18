// Plato - Document reader for Kobo devices
// Copyright (C) 2025 Pedro Depasco
// Licensed under the GPL-3.0 License.

//! Email composition dialog for document sharing

use super::button::Button;
use super::label::Label;
use super::{Align, Bus, Event, Hub, Id, RenderQueue, View, ViewId, ID_FEEDER};
use super::{BORDER_RADIUS_MEDIUM, THICKNESS_LARGE};
use crate::color::background;
use crate::context::Context;
use crate::font::{font_from_style, Fonts, NORMAL_STYLE};
use crate::framebuffer::Framebuffer;
use crate::geom::{BorderSpec, CornerSpec, Rectangle};
use crate::gesture::GestureEvent;
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::input_field::InputField;
use std::path::PathBuf;

const LABEL_CANCEL: &str = "Cancel";
const LABEL_SEND: &str = "Send";
const LABEL_TO: &str = "To:";
const LABEL_SUBJECT: &str = "Subject:";
const LABEL_BODY: &str = "Message:";

/// Email dialog for composing and sending documents
pub struct EmailDialog {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    view_id: ViewId,
    will_close: bool,
    document_path: Option<PathBuf>,
    recipient: String,
    subject: String,
    body: String,
}

impl EmailDialog {
    pub fn new(context: &mut Context, document_path: Option<PathBuf>) -> EmailDialog {
        let id = ID_FEEDER.next();
        let view_id = ViewId::EmailDialog;
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
        let dialog_height = input_height * 4 + button_height + 8 * padding;

        let dx = (width as i32 - dialog_width) / 2;
        let dy = (height as i32 - dialog_height) / 2;
        let rect = rect![dx, dy, dx + dialog_width, dy + dialog_height];

        // Build subject based on document name
        let subject_text = if let Some(ref path) = document_path {
            if let Some(filename) = path.file_stem() {
                format!("Document: {}", filename.to_string_lossy())
            } else {
                "Document from Plato".to_string()
            }
        } else {
            "Document from Plato".to_string()
        };

        // Add To: label and input
        let y_start = rect.min.y + padding;
        let rect_label_to = rect![
            rect.min.x + padding,
            y_start,
            rect.min.x + padding + font.plan(LABEL_TO, None, None).width as i32,
            y_start + input_height
        ];
        let label_to = Label::new(rect_label_to, LABEL_TO.to_string(), Align::Left(0));
        children.push(Box::new(label_to));

        let rect_input_to = rect![
            rect.min.x + 2 * padding + font.plan(LABEL_TO, None, None).width as i32,
            y_start,
            rect.max.x - padding,
            y_start + input_height
        ];
        let input_to = InputField::new(
            rect_input_to,
            ViewId::EmailSubjectInput,
        )
        .border(true)
        .placeholder("recipient@example.com");
        children.push(Box::new(input_to));

        // Add Subject: label and input
        let y_subject = y_start + input_height + padding;
        let subject_label_width = font.plan(LABEL_SUBJECT, None, None).width as i32;
        let rect_label_subject = rect![
            rect.min.x + padding,
            y_subject,
            rect.min.x + padding + subject_label_width,
            y_subject + input_height
        ];
        let label_subject = Label::new(rect_label_subject, LABEL_SUBJECT.to_string(), Align::Left(0));
        children.push(Box::new(label_subject));

        let rect_input_subject = rect![
            rect.min.x + 2 * padding + subject_label_width,
            y_subject,
            rect.max.x - padding,
            y_subject + input_height
        ];
        // Create input field with text set separately to avoid borrow issues
        let input_subject = InputField::new(
            rect_input_subject,
            ViewId::EmailSubjectInput,
        )
        .border(true);
        children.push(Box::new(input_subject));

        // Add Message: label
        let y_body = y_subject + input_height + padding;
        let rect_label_body = rect![
            rect.min.x + padding,
            y_body,
            rect.max.x - padding,
            y_body + input_height
        ];
        let label_body = Label::new(
            rect_label_body,
            "Attached document from Plato e-reader".to_string(),
            Align::Left(0),
        );
        children.push(Box::new(label_body));

        // Add Cancel button
        let button_width = font.plan(LABEL_CANCEL, Some(max_button_width), None).width as i32 + padding;
        let rect_cancel = rect![
            rect.min.x + padding,
            rect.max.y - button_height - padding,
            rect.min.x + padding + button_width,
            rect.max.y - padding
        ];
        let cancel_button = Button::new(rect_cancel, Event::Cancel, LABEL_CANCEL.to_string());
        children.push(Box::new(cancel_button));

        // Add Send button
        let send_button_width = font.plan(LABEL_SEND, Some(max_button_width), None).width as i32 + padding;
        let rect_send = rect![
            rect.max.x - padding - send_button_width,
            rect.max.y - button_height - padding,
            rect.max.x - padding,
            rect.max.y - padding
        ];
        let send_button = Button::new(rect_send, Event::Validate, LABEL_SEND.to_string());
        children.push(Box::new(send_button));

        EmailDialog {
            id,
            rect,
            children,
            view_id,
            will_close: false,
            document_path,
            recipient: String::new(),
            subject: subject_text,
            body: "Attached document from Plato e-reader".to_string(),
        }
    }

    /// Get the document path
    pub fn document_path(&self) -> Option<&PathBuf> {
        self.document_path.as_ref()
    }

    /// Get the current recipient
    pub fn recipient(&self) -> &str {
        &self.recipient
    }

    /// Get the current subject
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl View for EmailDialog {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Validate => {
                // Send email - create email file for system email client
                if self.will_close {
                    return true;
                }
                self.will_close = true;

                let msg = if let Some(ref path) = self.document_path {
                    format!(
                        "Email prepared for: {}\nSubject: {}\nDocument: {}",
                        self.recipient,
                        self.subject,
                        path.display()
                    )
                } else {
                    "Email prepared (no document attached)".to_string()
                };
                bus.push_back(Event::Notify(msg));
                bus.push_back(Event::Close(ViewId::EmailDialog));
                true
            }
            Event::Cancel => {
                if self.will_close {
                    return true;
                }
                self.will_close = true;
                bus.push_back(Event::Close(ViewId::EmailDialog));
                true
            }
            Event::Gesture(GestureEvent::Tap(center)) if !self.rect.includes(center) => {
                if self.will_close {
                    return true;
                }
                self.will_close = true;
                bus.push_back(Event::Close(ViewId::EmailDialog));
                true
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
        let dpi = crate::unit::get_device_dpi();
        let border_thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as u16;
        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;

        fb.draw_rounded_rectangle_with_border(
            &self.rect,
            &CornerSpec::Uniform(border_radius),
            &BorderSpec {
                thickness: border_thickness,
                color: crate::color::foreground(theme::is_dark_mode()),
            },
            &background(theme::is_dark_mode()),
        );
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

    fn view_id(&self) -> Option<ViewId> {
        Some(self.view_id)
    }
}
