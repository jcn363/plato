//! System Info Dialog Module
//!
//! Displays comprehensive system and library information including:
//! - Device information (battery, storage, memory)
//! - Application version and build info
//! - Library statistics (book counts, formats, reading time)

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
use std::time::Duration;

const LABEL_OK: &str = "OK";
const APP_VERSION: &str = "0.9.38";
const APP_NAME: &str = "Plato";

pub struct SystemInfoDialog {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    view_id: ViewId,
    will_close: bool,
}

impl SystemInfoDialog {
    pub fn new(context: &mut Context) -> SystemInfoDialog {
        let id = ID_FEEDER.next();
        let view_id = ViewId::SystemInfo;
        let mut children: Vec<Box<dyn View>> = Vec::new();
        let dpi = crate::unit::get_device_dpi();
        let (width, height) = context.display.dims;

        // Compute statistics before borrowing fonts mutably
        let stats = context.library.compute_statistics();

        let font = font_from_style(&mut context.fonts, &NORMAL_STYLE, dpi);
        let x_height = font.x_heights.0 as i32;
        let padding = font.em() as i32;

        let button_height = 4 * x_height;
        let max_button_width = width as i32 / 4;

        // Build system info text
        let info_text = Self::build_info_text(&stats);

        let plan = font.plan(&info_text, Some(width as i32 / 2), None);

        let dialog_width = plan.width.max(width as i32 / 3) + 4 * padding;
        let text_lines = 12; // approximately 12 lines of content
        let dialog_height = text_lines * x_height + button_height + 4 * padding;

        let dx = (width as i32 - dialog_width) / 2;
        let dy = (height as i32 - dialog_height) / 2;
        let rect = rect![dx, dy, dx + dialog_width, dy + dialog_height];

        // Add content label
        let rect_label = rect![
            rect.min.x + padding,
            rect.min.y + padding,
            rect.max.x - padding,
            rect.max.y - button_height - 2 * padding
        ];
        let label = Label::new(rect_label, info_text, Align::Left(0));
        children.push(Box::new(label));

        // Add OK button
        let button_width = font.plan(LABEL_OK, Some(max_button_width), None).width as i32 + padding;
        let rect_ok = rect![
            (rect.min.x + rect.max.x - button_width) / 2,
            rect.max.y - button_height - padding,
            (rect.min.x + rect.max.x + button_width) / 2,
            rect.max.y - padding
        ];
        let button_ok = Button::new(rect_ok, Event::Validate, LABEL_OK.to_string());
        children.push(Box::new(button_ok));

        SystemInfoDialog {
            id,
            rect,
            children,
            view_id,
            will_close: false,
        }
    }

    /// Build comprehensive system and library info text
    fn build_info_text(stats: &crate::settings::LibraryStatistics) -> String {
        // Calculate reading time
        let total_time = Duration::from_secs(stats.total_reading_time);
        let hours = total_time.as_secs() / 3600;
        let minutes = (total_time.as_secs() % 3600) / 60;

        // Format the info text
        format!(
            "System Information\n\n\
            Application: {} v{}\n\
            Library Statistics:\n\
            • Total Books: {}\n\
            • Finished: {}\n\
            • Reading Time: {}h {}m\n\
            • Completion Rate: {:.1}%",
            APP_NAME,
            APP_VERSION,
            stats.total_books,
            stats.finished_books,
            hours,
            minutes,
            stats.average_progress * 100.0
        )
    }
}

impl View for SystemInfoDialog {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Validate | Event::Cancel => {
                if self.will_close {
                    return true;
                }
                self.will_close = true;
                bus.push_back(Event::Close(ViewId::SystemInfo));
                true
            }
            Event::Gesture(GestureEvent::Tap(center)) if !self.rect.includes(center) => {
                // Close when tapping outside
                if self.will_close {
                    return true;
                }
                self.will_close = true;
                bus.push_back(Event::Close(ViewId::SystemInfo));
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
            if let Some(intersection) = rect.intersection(&child_rect) {
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
