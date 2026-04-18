// Plato - Document reader for Kobo devices
// Copyright (C) 2025 Pedro Depasco
// Licensed under the GPL-3.0 License.

//! Cloud sharing dialog for document upload to cloud providers
//! Supports Dropbox, Google Drive, and other cloud storage services

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
use std::path::PathBuf;

const LABEL_CANCEL: &str = "Cancel";
const LABEL_DROPBOX: &str = "Dropbox";
const LABEL_GOOGLE_DRIVE: &str = "Google Drive";
const LABEL_CONFIGURE: &str = "Configure";

/// Cloud provider types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CloudProvider {
    Dropbox,
    GoogleDrive,
}

/// Cloud sharing dialog for document upload
pub struct CloudDialog {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    view_id: ViewId,
    will_close: bool,
    document_path: Option<PathBuf>,
    selected_provider: Option<CloudProvider>,
}

impl CloudDialog {
    pub fn new(context: &mut Context, document_path: Option<PathBuf>) -> CloudDialog {
        let id = ID_FEEDER.next();
        let view_id = ViewId::CloudDialog;
        let mut children: Vec<Box<dyn View>> = Vec::new();
        let dpi = crate::unit::get_device_dpi();
        let (width, height) = context.display.dims;

        let font = font_from_style(&mut context.fonts, &NORMAL_STYLE, dpi);
        let x_height = font.x_heights.0 as i32;
        let padding = font.em() as i32;

        let button_height = 4 * x_height;
        let max_button_width = width as i32 / 4;

        // Calculate dialog dimensions
        let dialog_width = width as i32 / 2;
        let dialog_height = button_height * 4 + 6 * padding;

        let dx = (width as i32 - dialog_width) / 2;
        let dy = (height as i32 - dialog_height) / 2;
        let rect = rect![dx, dy, dx + dialog_width, dy + dialog_height];

        // Add title label
        let title = if document_path.is_some() {
            "Upload to Cloud".to_string()
        } else {
            "Cloud Providers".to_string()
        };
        let title_width = font.plan(&title, None, None).width as i32;
        let rect_title = rect![
            (rect.min.x + rect.max.x - title_width) / 2,
            rect.min.y + padding,
            (rect.min.x + rect.max.x + title_width) / 2,
            rect.min.y + padding + button_height
        ];
        let label_title = Label::new(rect_title, title, Align::Center);
        children.push(Box::new(label_title));

        // Add cloud provider buttons
        let providers = [
            (LABEL_DROPBOX, CloudProvider::Dropbox),
            (LABEL_GOOGLE_DRIVE, CloudProvider::GoogleDrive),
        ];

        for (i, (label, _provider)) in providers.iter().enumerate() {
            let i = i as i32;
            let _button_width = dialog_width - 2 * padding;
            let rect_button = rect![
                rect.min.x + padding,
                rect.min.y + 2 * padding + button_height + i * (button_height + padding),
                rect.max.x - padding,
                rect.min.y
                    + 2 * padding
                    + button_height
                    + i * (button_height + padding)
                    + button_height
            ];
            let button = Button::new(rect_button, Event::Validate, label.to_string());
            children.push(Box::new(button));
        }

        // Add Configure button
        let config_width = font
            .plan(LABEL_CONFIGURE, Some(max_button_width), None)
            .width as i32
            + padding;
        let rect_config = rect![
            (rect.min.x + rect.max.x - config_width) / 2,
            rect.max.y - 2 * button_height - 2 * padding,
            (rect.min.x + rect.max.x + config_width) / 2,
            rect.max.y - button_height - 2 * padding
        ];
        let config_button = Button::new(
            rect_config,
            Event::Toggle(ViewId::SystemInfo),
            LABEL_CONFIGURE.to_string(),
        );
        children.push(Box::new(config_button));

        // Add Cancel button
        let cancel_width =
            font.plan(LABEL_CANCEL, Some(max_button_width), None).width as i32 + padding;
        let rect_cancel = rect![
            (rect.min.x + rect.max.x - cancel_width) / 2,
            rect.max.y - button_height - padding,
            (rect.min.x + rect.max.x + cancel_width) / 2,
            rect.max.y - padding
        ];
        let cancel_button = Button::new(rect_cancel, Event::Cancel, LABEL_CANCEL.to_string());
        children.push(Box::new(cancel_button));

        CloudDialog {
            id,
            rect,
            children,
            view_id,
            will_close: false,
            document_path,
            selected_provider: None,
        }
    }

    /// Get the selected cloud provider
    pub fn selected_provider(&self) -> Option<CloudProvider> {
        self.selected_provider
    }

    /// Get the document path
    pub fn document_path(&self) -> Option<&PathBuf> {
        self.document_path.as_ref()
    }
}

impl View for CloudDialog {
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
                // Cloud provider selected - show configuration guidance
                if self.will_close {
                    return true;
                }
                self.will_close = true;

                let msg = if let Some(ref path) = self.document_path {
                    format!(
                        "Cloud upload ready for:\n{}\n\nConfigure OAuth in Settings > Cloud Providers",
                        path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default()
                    )
                } else {
                    "Select a document first, then configure cloud providers in Settings"
                        .to_string()
                };
                bus.push_back(Event::Notify(msg));
                bus.push_back(Event::Close(ViewId::CloudDialog));
                true
            }
            Event::Cancel => {
                if self.will_close {
                    return true;
                }
                self.will_close = true;
                bus.push_back(Event::Close(ViewId::CloudDialog));
                true
            }
            Event::Gesture(GestureEvent::Tap(center)) if !self.rect.includes(center) => {
                if self.will_close {
                    return true;
                }
                self.will_close = true;
                bus.push_back(Event::Close(ViewId::CloudDialog));
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
