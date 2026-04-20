use super::common::locate_by_id;
use super::filler::Filler;
use super::menu_entry::MenuEntry;
use super::{Bus, Event, Hub, RenderData, RenderQueue, View};
use super::{EntryKind, Id, ViewId, CLOSE_IGNITION_DELAY, ID_FEEDER};
use super::{BORDER_RADIUS_MEDIUM, SMALL_BAR_HEIGHT, THICKNESS_LARGE, THICKNESS_MEDIUM};
use crate::color::{separator, separator_strong, Color};
use crate::context::Context;
use crate::font::{font_from_style, Fonts, NORMAL_STYLE};
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{big_half, small_half, BorderSpec, CornerSpec, Point, Rectangle};
use crate::gesture::GestureEvent;
use crate::theme::{self, background, foreground};
use crate::unit::scale_by_dpi;
use std::thread;

pub struct Menu {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    view_id: ViewId,
    kind: MenuKind,
    center: Point,
    root: bool,
    sub_id: u8,
    dir: i32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MenuKind {
    DropDown,
    SubMenu,
    Contextual,
}

// TOP MENU       C
//    ───         B
//  ↓  A       ↑  A
//     B         ───
//     C     BOTTOM MENU

impl Menu {
    pub fn new(
        target: Rectangle,
        view_id: ViewId,
        kind: MenuKind,
        mut entries: Vec<EntryKind>,
        context: &mut Context,
    ) -> Menu {
        let id = ID_FEEDER.next();
        let dpi = crate::unit::get_device_dpi();
        let (width, height) = context.display.dims;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;

        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let border_thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as i32;
        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM - THICKNESS_LARGE, dpi) as i32;

        let dark = crate::theme::is_dark_mode();
        let sep_color = if context.fb.monochrome() {
            separator_strong(dark)
        } else {
            separator(dark)
        };
        let font = font_from_style(&mut context.fonts, &NORMAL_STYLE, dpi);
        let entry_height = font.x_heights.0 as i32 * 6;
        let padding = 4 * font.em() as i32;

        let center = target.center();

        let (dir, y_start, border_space) = Self::calculate_layout_params(
            kind,
            target,
            height,
            small_height,
            thickness,
            border_thickness,
        );

        let max_entries = Self::calculate_max_entries(
            dir,
            y_start,
            height,
            small_height,
            thickness,
            border_space,
            entry_height,
        );

        entries = Self::handle_entry_overflow(entries, max_entries);

        let y_pos = y_start + dir * (border_space - border_thickness);

        let entry_width = Self::calculate_entry_width(
            kind,
            target,
            width,
            padding,
            border_thickness,
            &entries,
            font,
        );

        let (x_min, x_max) = Self::calculate_x_position(kind, target, width, entry_width, center);

        let children = Self::build_menu_children(
            &entries,
            x_min,
            x_max,
            y_pos,
            dir,
            entry_height,
            thickness,
            border_thickness,
            border_radius,
            kind,
            sep_color,
        );

        let triangle_space = if kind == MenuKind::Contextual {
            font.x_heights.1 as i32
        } else {
            0
        };

        let total_entries = entries.iter().filter(|e| !e.is_separator()).count();
        let menu_height = total_entries as i32 * entry_height + border_space;

        let (y_min, y_max) = if dir.is_positive() {
            (y_start - triangle_space, y_start + menu_height)
        } else {
            (y_start - menu_height, y_start + triangle_space)
        };

        let rect = rect![x_min, y_min, x_max, y_max];

        Menu {
            id,
            rect,
            children,
            view_id,
            kind,
            center,
            root: true,
            sub_id: 0,
            dir,
        }
    }

    pub fn root(mut self, root: bool) -> Menu {
        self.root = root;
        self
    }

    fn calculate_layout_params(
        kind: MenuKind,
        target: Rectangle,
        height: u32,
        _small_height: i32,
        _thickness: i32,
        border_thickness: i32,
    ) -> (i32, i32, i32) {
        let north_space = target.min.y;
        let south_space = height as i32 - target.max.y;

        let (dir, y_start): (i32, i32) = if kind == MenuKind::SubMenu {
            if north_space < south_space {
                (1, target.min.y - border_thickness)
            } else {
                (-1, target.max.y + border_thickness)
            }
        } else {
            if north_space < south_space {
                (1, target.max.y)
            } else {
                (-1, target.min.y)
            }
        };

        let border_space = if kind == MenuKind::DropDown {
            border_thickness
        } else {
            2 * border_thickness
        };

        (dir, y_start, border_space)
    }

    fn calculate_max_entries(
        dir: i32,
        y_start: i32,
        height: u32,
        small_height: i32,
        thickness: i32,
        border_space: i32,
        entry_height: i32,
    ) -> usize {
        let top_min = small_height + big_half(thickness);
        let bottom_max = height as i32 - small_height - small_half(thickness);

        let usable_space = if dir.is_positive() {
            bottom_max - y_start
        } else {
            y_start - top_min
        };

        ((usable_space - border_space) / entry_height) as usize
    }

    fn handle_entry_overflow(mut entries: Vec<EntryKind>, max_entries: usize) -> Vec<EntryKind> {
        let total_entries = entries.iter().filter(|e| !e.is_separator()).count();

        if total_entries > max_entries {
            let mut kind_counts = [0, 0];
            for e in &entries {
                kind_counts[e.is_separator() as usize] += 1;
                if kind_counts[0] >= max_entries {
                    break;
                }
            }
            let index = kind_counts[0] + kind_counts[1] - 1;
            let more = entries.drain(index..).collect::<Vec<EntryKind>>();
            entries.push(EntryKind::More(more));
        }

        entries
    }

    fn calculate_entry_width(
        _kind: MenuKind,
        _target: Rectangle,
        width: u32,
        padding: i32,
        border_thickness: i32,
        entries: &[EntryKind],
        font: &mut crate::font::Font,
    ) -> i32 {
        let max_width = 2 * width as i32 / 3;
        let free_width = padding
            + 2 * border_thickness
            + entries
                .iter()
                .map(|e| font.plan(e.text(), None, None).width)
                .max()
                .unwrap_or(0);

        free_width.min(max_width)
    }

    fn calculate_x_position(
        kind: MenuKind,
        target: Rectangle,
        width: u32,
        entry_width: i32,
        center: Point,
    ) -> (i32, i32) {
        let (mut x_min, mut x_max) = if kind == MenuKind::SubMenu {
            let west_space = target.min.x;
            let east_space = width as i32 - target.max.x;
            if west_space > east_space {
                (target.min.x - entry_width, target.min.x)
            } else {
                (target.max.x, target.max.x + entry_width)
            }
        } else {
            (
                center.x - small_half(entry_width),
                center.x + big_half(entry_width),
            )
        };

        if x_min < 0 {
            x_max -= x_min;
            x_min = 0;
        }

        if x_max > width as i32 {
            x_min += width as i32 - x_max;
            x_max = width as i32;
        }

        (x_min, x_max)
    }

    #[allow(clippy::too_many_arguments)]
    fn build_menu_children(
        entries: &[EntryKind],
        x_min: i32,
        x_max: i32,
        mut y_pos: i32,
        dir: i32,
        entry_height: i32,
        thickness: i32,
        border_thickness: i32,
        border_radius: i32,
        kind: MenuKind,
        sep_color: Color,
    ) -> Vec<Box<dyn View>> {
        let mut children = Vec::new();
        let entries_count = entries.len();

        for i in 0..entries_count {
            if entries[i].is_separator() {
                let rect = rect![
                    x_min + border_thickness,
                    y_pos - small_half(thickness),
                    x_max - border_thickness,
                    y_pos + big_half(thickness)
                ];
                let separator = Filler::new(rect, sep_color);
                children.push(Box::new(separator) as Box<dyn View>);
            } else {
                let (y_min, y_max) = if dir.is_positive() {
                    (y_pos, y_pos + entry_height)
                } else {
                    (y_pos - entry_height, y_pos)
                };

                let mut rect = rect![
                    x_min + border_thickness,
                    y_min,
                    x_max - border_thickness,
                    y_max
                ];

                let anchor = rect;

                if i > 0 && entries[i - 1].is_separator() {
                    if dir.is_positive() {
                        rect.min.y += big_half(thickness);
                    } else {
                        rect.max.y -= small_half(thickness);
                    }
                }

                if i < entries_count - 1 && entries[i + 1].is_separator() {
                    if dir.is_positive() {
                        rect.max.y -= small_half(thickness);
                    } else {
                        rect.min.y += big_half(thickness);
                    }
                }

                let corner_spec =
                    Self::calculate_corner_spec(kind, i, entries_count, dir, border_radius);

                let menu_entry = MenuEntry::new(rect, entries[i].clone(), anchor, corner_spec);
                children.push(Box::new(menu_entry) as Box<dyn View>);

                y_pos += dir * entry_height;
            }
        }

        children
    }

    fn calculate_corner_spec(
        kind: MenuKind,
        index: usize,
        entries_count: usize,
        dir: i32,
        border_radius: i32,
    ) -> Option<CornerSpec> {
        if kind != MenuKind::DropDown && entries_count == 1 {
            Some(CornerSpec::Uniform(border_radius))
        } else if index == entries_count - 1 {
            if dir.is_positive() {
                Some(CornerSpec::South(border_radius))
            } else {
                Some(CornerSpec::North(border_radius))
            }
        } else if kind != MenuKind::DropDown && index == 0 {
            if dir.is_positive() {
                Some(CornerSpec::North(border_radius))
            } else {
                Some(CornerSpec::South(border_radius))
            }
        } else {
            None
        }
    }

    fn handle_select_event(
        &mut self,
        entry_id: crate::view::entries::EntryId,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        self.handle_event(&Event::PropagateSelect(entry_id), hub, bus, rq, context);
    }

    fn handle_propagate_select(
        &mut self,
        evt: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        for c in &mut self.children {
            if c.handle_event(evt, hub, bus, rq, context) {
                break;
            }
        }
    }

    fn handle_validate_event(&mut self, hub: &Hub) {
        let hub2 = hub.clone();
        let view_id = self.view_id;
        thread::spawn(move || {
            thread::sleep(CLOSE_IGNITION_DELAY);
            hub2.send(Event::Close(view_id)).ok();
        });
    }

    fn handle_tap_outside(&mut self, _center: Point, bus: &mut Bus) {
        if self.root {
            bus.push_back(Event::Close(self.view_id));
        } else {
            bus.push_back(Event::CloseSub(self.view_id));
        }
    }

    fn handle_submenu_event(
        &mut self,
        rect: Rectangle,
        entries: &[EntryKind],
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let menu = Menu::new(
            rect,
            ViewId::SubMenu(self.sub_id),
            MenuKind::SubMenu,
            entries.to_vec(),
            context,
        )
        .root(false);
        rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
        self.children.push(Box::new(menu) as Box<dyn View>);
        self.sub_id = self.sub_id.wrapping_add(1);
    }

    fn handle_close_sub_event(&mut self, id: ViewId, rq: &mut RenderQueue) {
        if let Some(index) = locate_by_id(self, id) {
            rq.add(RenderData::expose(
                *self.children[index].rect(),
                UpdateMode::Gui,
            ));
            self.children.remove(index);
        }
    }

    fn calculate_render_corners(&self, border_radius: i32) -> CornerSpec {
        if self.kind == MenuKind::DropDown {
            if self.dir.is_positive() {
                CornerSpec::South(border_radius)
            } else {
                CornerSpec::North(border_radius)
            }
        } else {
            CornerSpec::Uniform(border_radius)
        }
    }

    fn render_contextual_menu(
        &self,
        fb: &mut dyn Framebuffer,
        fonts: &mut Fonts,
        dpi: u16,
        _border_radius: i32,
        border_thickness: u16,
        corners: CornerSpec,
    ) {
        let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
        let triangle_space = font.x_heights.1 as i32;
        let mut rect = self.rect;

        if self.dir.is_positive() {
            rect.min.y += triangle_space
        } else {
            rect.max.y -= triangle_space
        }

        fb.draw_rounded_rectangle_with_border(
            &rect,
            &corners,
            &BorderSpec {
                thickness: border_thickness,
                color: foreground(theme::is_dark_mode()),
            },
            &background(theme::is_dark_mode()),
        );

        self.draw_triangle_indicator(fb, rect, triangle_space, border_thickness);
    }

    fn render_standard_menu(
        &self,
        fb: &mut dyn Framebuffer,
        _border_radius: i32,
        border_thickness: u16,
        corners: CornerSpec,
    ) {
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

    fn draw_triangle_indicator(
        &self,
        fb: &mut dyn Framebuffer,
        rect: Rectangle,
        triangle_space: i32,
        border_thickness: u16,
    ) {
        let y_b = if self.dir.is_positive() {
            self.rect.min.y
        } else {
            self.rect.max.y - 1
        };

        let side = triangle_space + border_thickness as i32;
        let x_b = self
            .center
            .x
            .max(rect.min.x + 2 * side)
            .min(rect.max.x - 2 * side);

        let mut b = pt!(x_b, y_b);
        let mut a = b + pt!(-side, self.dir * side);
        let mut c = a + pt!(2 * side, 0);

        fb.draw_triangle(&[a, b, c], foreground(theme::is_dark_mode()));
        let drift = (border_thickness as f32 * ::std::f32::consts::SQRT_2) as i32;

        b += pt!(0, self.dir * drift);
        a += pt!(drift, 0);
        c -= pt!(drift, 0);

        fb.draw_triangle(&[a, b, c], background(theme::is_dark_mode()));
    }
}

impl View for Menu {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Select(ref entry_id) if self.root => {
                self.handle_select_event(entry_id.clone(), hub, bus, rq, context);
                false
            }
            Event::PropagateSelect(..) => {
                self.handle_propagate_select(evt, hub, bus, rq, context);
                true
            }
            Event::Validate if self.root => {
                self.handle_validate_event(hub);
                true
            }
            Event::Gesture(GestureEvent::Tap(center)) if !self.rect.includes(center) => {
                self.handle_tap_outside(center, bus);
                self.root
            }
            Event::Gesture(GestureEvent::HoldFingerShort(center, ..))
                if !self.rect.includes(center) =>
            {
                self.root
            }
            Event::SubMenu(rect, ref entries) => {
                self.handle_submenu_event(rect, entries, rq, context);
                true
            }
            Event::CloseSub(id) => {
                self.handle_close_sub_event(id, rq);
                true
            }
            Event::Gesture(..) => true,
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, fonts: &mut Fonts) {
        let dpi = crate::unit::get_device_dpi();
        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;
        let border_thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as u16;

        let corners = self.calculate_render_corners(border_radius);

        if self.kind == MenuKind::Contextual {
            self.render_contextual_menu(fb, fonts, dpi, border_radius, border_thickness, corners);
        } else {
            self.render_standard_menu(fb, border_radius, border_thickness, corners);
        }
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
        Some(self.view_id)
    }
}
