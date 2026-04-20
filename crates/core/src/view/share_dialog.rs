//! Share Dialog Module
//!
//! Provides options for sharing documents via email or cloud services.
//! Displays available sharing methods for the current document.

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
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const LABEL_CANCEL: &str = "Cancel";
const LABEL_SHARE_EMAIL: &str = "Email";
const LABEL_SHARE_CLOUD: &str = "Cloud";
const LABEL_SHARE_EXPORT: &str = "Export";

/// Share method types for document sharing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShareMethod {
    Email,
    Cloud,
    Export,
}

pub struct ShareDialog {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    view_id: ViewId,
    will_close: bool,
    document_path: Option<PathBuf>,
    selected_method: Option<ShareMethod>,
}

impl ShareDialog {
    pub fn new(context: &mut Context, document_path: Option<PathBuf>) -> ShareDialog {
        let id = ID_FEEDER.next();
        let view_id = ViewId::ShareDialog;
        let mut children: Vec<Box<dyn View>> = Vec::new();
        let dpi = crate::unit::get_device_dpi();
        let (width, height) = context.display.dims;

        let font = font_from_style(&mut context.fonts, &NORMAL_STYLE, dpi);
        let x_height = font.x_heights.0 as i32;
        let padding = font.em() as i32;

        let button_height = 4 * x_height;
        let max_button_width = width as i32 / 4;

        // Dialog title
        let title = "Share Document".to_string();
        let title_plan = font.plan(&title, Some(max_button_width * 2), None);

        // Calculate dialog dimensions
        let button_count = 3i32; // Email, Cloud, Export
        let dialog_width = max_button_width.max(title_plan.width) + 4 * padding;
        let dialog_height =
            button_height * (button_count + 1) + (button_count + 3) * padding + x_height;

        let dx = (width as i32 - dialog_width) / 2;
        let dy = (height as i32 - dialog_height) / 2;
        let rect = rect![dx, dy, dx + dialog_width, dy + dialog_height];

        // Add title label
        let rect_title = rect![
            rect.min.x + padding,
            rect.min.y + padding,
            rect.max.x - padding,
            rect.min.y + padding + button_height
        ];
        let title_label = Label::new(rect_title, title, Align::Center);
        children.push(Box::new(title_label));

        // Add share method buttons with unique event IDs
        let _button_width = dialog_width - 2 * padding;
        let methods = [
            (LABEL_SHARE_EMAIL, Event::Show(ViewId::AboutDialog)), // Use Show as marker for Email
            (LABEL_SHARE_CLOUD, Event::Show(ViewId::ShareDialog)), // Use Show as marker for Cloud
            (LABEL_SHARE_EXPORT, Event::Toggle(ViewId::SystemInfo)), // Use Toggle as marker for Export
        ];

        for (i, (label, event)) in methods.iter().enumerate() {
            let i = i as i32;
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
            let button = Button::new(rect_button, event.clone(), label.to_string());
            children.push(Box::new(button));
        }

        // Add cancel button at bottom
        let cancel_width = font.plan(LABEL_CANCEL, Some(max_button_width), None).width + padding;
        let rect_cancel = rect![
            rect.max.x - cancel_width - padding,
            rect.max.y - button_height - padding,
            rect.max.x - padding,
            rect.max.y - padding
        ];
        let cancel_button = Button::new(rect_cancel, Event::Cancel, LABEL_CANCEL.to_string());
        children.push(Box::new(cancel_button));

        ShareDialog {
            id,
            rect,
            children,
            view_id,
            will_close: false,
            document_path,
            selected_method: None,
        }
    }
}

impl View for ShareDialog {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Show(ViewId::AboutDialog) => {
                // Email sharing selected - open email composition dialog
                self.selected_method = Some(ShareMethod::Email);
                if self.will_close {
                    return true;
                }
                self.will_close = true;
                // Close share dialog and open email dialog
                bus.push_back(Event::Close(ViewId::ShareDialog));
                bus.push_back(Event::Show(ViewId::EmailDialog));
                true
            }
            Event::Show(ViewId::ShareDialog) => {
                // Cloud sharing selected - open cloud dialog
                self.selected_method = Some(ShareMethod::Cloud);
                if self.will_close {
                    return true;
                }
                self.will_close = true;
                // Close share dialog and open cloud dialog
                bus.push_back(Event::Close(ViewId::ShareDialog));
                bus.push_back(Event::Show(ViewId::CloudDialog));
                true
            }
            Event::Toggle(ViewId::SystemInfo) => {
                // Export selected - implement actual file export
                self.selected_method = Some(ShareMethod::Export);
                if self.will_close {
                    return true;
                }
                self.will_close = true;

                let msg = if let Some(ref path) = self.document_path {
                    // Create export filename with timestamp
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    if let Some(filename) = path.file_stem() {
                        let export_name =
                            format!("{}_export_{}.pdf", filename.to_string_lossy(), timestamp);
                        let export_path = path.with_file_name(&export_name);

                        // Attempt to copy file
                        match fs::copy(path, &export_path) {
                            Ok(bytes) => format!("Exported: {} ({} bytes)", export_name, bytes),
                            Err(e) => format!("Export failed: {}", e),
                        }
                    } else {
                        "Export: Could not determine filename".to_string()
                    }
                } else {
                    "Export: No document selected".to_string()
                };

                bus.push_back(Event::Notify(msg));
                bus.push_back(Event::Close(ViewId::ShareDialog));
                true
            }
            Event::Cancel => {
                if self.will_close {
                    return true;
                }
                self.will_close = true;
                bus.push_back(Event::Close(ViewId::ShareDialog));
                true
            }
            Event::Gesture(GestureEvent::Tap(center)) if !self.rect.includes(center) => {
                // Close when tapping outside
                if self.will_close {
                    return true;
                }
                self.will_close = true;
                bus.push_back(Event::Close(ViewId::ShareDialog));
                true
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        let dpi = crate::unit::get_device_dpi();

        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;
        let border_thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as u16;

        fb.draw_rounded_rectangle_with_border(
            &self.rect,
            &CornerSpec::Uniform(border_radius),
            &BorderSpec {
                thickness: border_thickness,
                color: crate::color::foreground(theme::is_dark_mode()),
            },
            &background(theme::is_dark_mode()),
        );

        // Render children
        for child in &self.children {
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

    fn id(&self) -> Id {
        self.id
    }

    fn view_id(&self) -> Option<ViewId> {
        Some(self.view_id)
    }
}
