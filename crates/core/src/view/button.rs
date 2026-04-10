use super::{Bus, Event, Hub, Id, RenderData, RenderQueue, View, ID_FEEDER};
use super::{BORDER_RADIUS_LARGE, THICKNESS_MEDIUM};
use crate::color::text_normal;
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::font::{font_from_style, Fonts, NORMAL_STYLE};
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{BorderSpec, CornerSpec, Rectangle};
use crate::gesture::GestureEvent;
use crate::impl_view_boilerplate;
use crate::input::{DeviceEvent, FingerStatus};
use crate::unit::scale_by_dpi;

pub struct Button {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    event: Event,
    text: String,
    active: bool,
    pub disabled: bool,
}

impl Button {
    pub fn new(rect: Rectangle, event: Event, text: String) -> Button {
        Button {
            id: ID_FEEDER.next(),
            rect,
            children: Vec::new(),
            event,
            text,
            active: false,
            disabled: false,
        }
    }

    pub fn disabled(mut self, value: bool) -> Button {
        self.disabled = value;
        self
    }

    pub fn update(&mut self, text: String, rq: &mut RenderQueue) {
        if self.text != text {
            self.text = text;
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }
}

impl View for Button {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Device(DeviceEvent::Finger {
                status, position, ..
            }) if !self.disabled => match status {
                FingerStatus::Down if self.rect.includes(position) => {
                    self.active = true;
                    rq.add(RenderData::new(self.id, self.rect, UpdateMode::FastMono));
                    true
                }
                FingerStatus::Up if self.active => {
                    self.active = false;
                    rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                    true
                }
                _ => false,
            },
            Event::Gesture(GestureEvent::Tap(center)) if self.rect.includes(center) => {
                if !self.disabled {
                    bus.push_back(self.event.clone());
                }
                true
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, fonts: &mut Fonts) {
        let dpi = CURRENT_DEVICE.dpi;
        let dark = crate::theme::is_dark_mode();

        let scheme = if self.active {
            if dark {
                crate::color::DARK_TEXT_INVERTED_HARD
            } else {
                crate::color::TEXT_INVERTED_HARD
            }
        } else {
            text_normal(dark)
        };
        let foreground = if self.disabled { scheme[2] } else { scheme[1] };

        let border_radius = scale_by_dpi(BORDER_RADIUS_LARGE, dpi) as i32;
        let border_thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as u16;

        fb.draw_rounded_rectangle_with_border(
            &self.rect,
            &CornerSpec::Uniform(border_radius),
            &BorderSpec {
                thickness: border_thickness,
                color: foreground,
            },
            &scheme[0],
        );

        let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
        let x_height = font.x_heights.0 as i32;
        let padding = font.em() as i32;
        let max_width = self.rect.width() as i32 - padding;

        let plan = font.plan(&self.text, Some(max_width), None);

        let dx = (self.rect.width() as i32 - plan.width) / 2;
        let dy = (self.rect.height() as i32 - x_height) / 2;
        let pt = pt!(self.rect.min.x + dx, self.rect.max.y - dy);

        font.render(fb, foreground, &plan, pt);
    }

    impl_view_boilerplate!();
}
