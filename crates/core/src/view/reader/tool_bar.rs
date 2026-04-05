mod layout;
mod resize;

use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::Framebuffer;
use crate::geom::Rectangle;
use crate::gesture::GestureEvent;
use crate::input::DeviceEvent;
use crate::metadata::{ReaderInfo, TextAlign};
use crate::settings::ReaderSettings;
use crate::view::{Bus, Event, Hub, Id, RenderQueue, View, ID_FEEDER};

use self::layout::{
    build_common_children, build_fixed_children, build_reflowable_children, calc_side,
};
use self::resize::{
    resize_common_children, resize_fixed_children, resize_reflowable_children,
    update_contrast_exponent_slider, update_contrast_gray_slider, update_font_family,
    update_font_size_slider, update_line_height, update_margin_width, update_text_align_icon,
};

#[derive(Debug)]
pub struct ToolBar {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    reflowable: bool,
}

impl ToolBar {
    pub fn new(
        rect: Rectangle,
        reflowable: bool,
        reader_info: Option<&ReaderInfo>,
        reader_settings: &ReaderSettings,
    ) -> ToolBar {
        let id = ID_FEEDER.next();
        let side = calc_side(rect);

        let mut children = if reflowable {
            build_reflowable_children(rect, reader_info, reader_settings, side)
        } else {
            build_fixed_children(rect, reader_info, side)
        };

        children.extend(build_common_children(rect, side));

        ToolBar {
            id,
            rect,
            children,
            reflowable,
        }
    }

    #[allow(dead_code)]
    pub fn update_margin_width(&mut self, margin_width: i32, rq: &mut RenderQueue) {
        update_margin_width(&mut self.children, margin_width, rq, self.reflowable);
    }

    #[allow(dead_code)]
    pub fn update_font_family(&mut self, font_family: String, rq: &mut RenderQueue) {
        update_font_family(&mut self.children, font_family, rq);
    }

    #[allow(dead_code)]
    pub fn update_line_height(&mut self, line_height: f32, rq: &mut RenderQueue) {
        update_line_height(&mut self.children, line_height, rq);
    }

    #[allow(dead_code)]
    pub fn update_text_align_icon(&mut self, text_align: TextAlign, rq: &mut RenderQueue) {
        update_text_align_icon(&mut self.children, text_align, rq);
    }

    #[allow(dead_code)]
    pub fn update_font_size_slider(&mut self, font_size: f32, rq: &mut RenderQueue) {
        update_font_size_slider(&mut self.children, font_size, rq);
    }

    #[allow(dead_code)]
    pub fn update_contrast_exponent_slider(&mut self, exponent: f32, rq: &mut RenderQueue) {
        update_contrast_exponent_slider(&mut self.children, exponent, rq);
    }

    #[allow(dead_code)]
    pub fn update_contrast_gray_slider(&mut self, gray: f32, rq: &mut RenderQueue) {
        update_contrast_gray_slider(&mut self.children, gray, rq);
    }
}

impl View for ToolBar {
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
        let side = calc_side(rect);

        if self.reflowable {
            resize_reflowable_children(&mut self.children, rect, side, hub, rq, context);
        } else {
            resize_fixed_children(&mut self.children, rect, side, hub, rq, context);
        }

        resize_common_children(&mut self.children, rect, side, hub, rq, context);

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
