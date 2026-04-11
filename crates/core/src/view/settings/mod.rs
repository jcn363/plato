mod defaults;
mod display;
mod interface;
mod reading;
mod reading_advanced;
mod security;

use super::button::Button;
use super::icon::Icon;
use super::label::Label;
use super::{Align, Bus, EntryId, Event, Hub, Id, RenderQueue, View, ViewId, ID_FEEDER};
use super::{BORDER_RADIUS_MEDIUM, SMALL_BAR_HEIGHT, THICKNESS_LARGE};
use crate::color::{background, foreground};
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::font::{font_from_style, Fonts, NORMAL_STYLE};
use crate::framebuffer::Framebuffer;
use crate::geom::{BorderSpec, CornerSpec, Rectangle};
use crate::log_error;
use crate::theme;
use crate::unit::scale_by_dpi;

pub use defaults::LABEL_SAVE;
pub use security::save_settings;

pub struct SettingsEditor {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    dirty: bool,
}

impl SettingsEditor {
    pub fn new(context: &mut Context) -> SettingsEditor {
        let id = ID_FEEDER.next();
        let fonts = &mut context.fonts;
        let settings = &context.settings;
        let mut children = Vec::new();
        let dpi = CURRENT_DEVICE.dpi;
        let (width, height) = context.display.dims;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as i32;
        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;

        let (x_height, padding) = {
            let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
            (font.x_heights.0 as i32, font.em() as i32)
        };

        let window_width = width as i32 - 2 * padding;
        let window_height = small_height * 2 + 2 * padding + x_height;

        let max_label_width = {
            let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
            [
                "Frontlight",
                "WiFi",
                "Inverted",
                "Sleep Cover",
                "Auto Suspend (min)",
                "Auto Power Off (h)",
                "Auto Dual Page",
                "Finished Action",
                "Language",
                "UI Font",
            ]
            .iter()
            .map(|t| font.plan(t, None, None).width)
            .max()
            .unwrap_or(100) as i32
        };

        let dx = (width as i32 - window_width) / 2;
        let dy = (height as i32 - window_height) / 3;

        let rect = rect![dx, dy, dx + window_width, dy + window_height];

        let corners = CornerSpec::Detailed {
            north_west: 0,
            north_east: border_radius - thickness,
            south_east: 0,
            south_west: 0,
        };

        let close_icon = Icon::new(
            "close",
            rect![
                rect.max.x - small_height,
                rect.min.y + thickness,
                rect.max.x - thickness,
                rect.min.y + small_height
            ],
            Event::Close(ViewId::SettingsEditor),
        )
        .corners(Some(corners));

        children.push(Box::new(close_icon) as Box<dyn View>);

        let label = Label::new(
            rect![
                rect.min.x + small_height,
                rect.min.y + thickness,
                rect.max.x - small_height,
                rect.min.y + small_height
            ],
            "Settings".to_string(),
            Align::Center,
        );

        children.push(Box::new(label) as Box<dyn View>);

        let mut y_pos = rect.min.y + 2 * small_height;

        let (mut display_children, y) = display::build_rows(
            &rect,
            y_pos,
            small_height,
            padding,
            max_label_width,
            settings,
            context,
        );
        children.append(&mut display_children);
        y_pos = y;

        let (mut interface_children, y) = interface::build_rows(
            &rect,
            y_pos,
            small_height,
            padding,
            max_label_width,
            settings,
        );
        children.append(&mut interface_children);
        y_pos = y;

        let (mut reading_children, y) = reading::build_rows(
            &rect,
            y_pos,
            small_height,
            padding,
            max_label_width,
            settings,
        );
        children.append(&mut reading_children);
        y_pos = y;

        let (mut reading_adv_children, y) = reading_advanced::build_rows(
            &rect,
            y_pos,
            small_height,
            padding,
            max_label_width,
            settings,
        );
        children.append(&mut reading_adv_children);
        y_pos = y;

        let button_height = x_height * 3;
        let button_width = window_width - 2 * padding;
        let save_rect = rect![
            rect.min.x + padding,
            y_pos,
            rect.min.x + button_width + padding,
            y_pos + button_height
        ];
        let save_button = Button::new(
            save_rect,
            Event::Select(EntryId::SaveSettings),
            LABEL_SAVE.to_string(),
        );
        children.push(Box::new(save_button) as Box<dyn View>);

        let total_height = y_pos + button_height + padding;
        let final_rect = rect![dx, dy, dx + window_width, dy + total_height];

        SettingsEditor {
            id,
            rect: final_rect,
            children,
            dirty: false,
        }
    }
}

impl View for SettingsEditor {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        let display_offset = 2;
        let interface_offset = display_offset + display::CHILD_COUNT;
        let reading_offset = interface_offset + interface::CHILD_COUNT;
        let reading_adv_offset = reading_offset + reading_advanced::CHILD_COUNT;

        if display::handle_event(evt, &mut self.children, display_offset, bus, rq, context) {
            self.dirty = true;
            return true;
        }

        if interface::handle_event(evt, &mut self.children, interface_offset, bus, rq, context) {
            self.dirty = true;
            return true;
        }

        if reading::handle_event(evt, &mut self.children, reading_offset, bus, rq, context) {
            self.dirty = true;
            return true;
        }

        if reading_advanced::handle_event(
            evt,
            &mut self.children,
            reading_adv_offset,
            bus,
            rq,
            context,
        ) {
            self.dirty = true;
            return true;
        }

        if interface::handle_event(evt, &mut self.children, interface_offset, bus, rq, context) {
            self.dirty = true;
            return true;
        }

        if reading::handle_event(evt, &mut self.children, reading_offset, bus, rq, context) {
            self.dirty = true;
            return true;
        }

        match *evt {
            Event::Select(EntryId::SaveSettings) => {
                if self.dirty {
                    if let Err(e) = save_settings(&context.settings) {
                        log_error!("Failed to save settings: {}", e);
                    }
                }
                bus.push_back(Event::Close(ViewId::SettingsEditor));
                true
            }
            Event::Close(ViewId::SettingsEditor) => {
                bus.push_back(Event::Close(ViewId::SettingsEditor));
                true
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
        let dpi = CURRENT_DEVICE.dpi;
        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;
        let border_thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as u16;

        let corners = CornerSpec::Detailed {
            north_west: 0,
            north_east: border_radius - border_thickness as i32,
            south_east: 0,
            south_west: 0,
        };

        fb.draw_rounded_rectangle_with_border(
            &self.rect,
            &corners,
            &BorderSpec {
                thickness: border_thickness,
                color: foreground(theme::is_dark_mode()),
            },
            &background(theme::is_dark_mode()),
        );
    }

    fn is_background(&self) -> bool {
        true
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
        Some(ViewId::SettingsEditor)
    }
}
