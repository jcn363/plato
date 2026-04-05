use super::state::Calculator;
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::{halves, Rectangle};
use crate::unit::scale_by_dpi;
use crate::view::common::locate_by_id;
use crate::view::menu::{Menu, MenuKind};
use crate::view::top_bar::TopBar;
use crate::view::view_trait::View;
use crate::view::{
    EntryId, EntryKind, Hub, RenderData, RenderQueue, ViewId, BIG_BAR_HEIGHT, SMALL_BAR_HEIGHT,
    THICKNESS_MEDIUM,
};

impl Calculator {
    pub(super) fn update_size(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        let dpi = CURRENT_DEVICE.dpi;
        let font = &mut context.fonts.monospace.regular;
        font.set_size((64.0 * self.font_size) as u32, dpi);
        let char_width = font.plan(" ", None, None).width;
        let line_height = font.ascender() - font.descender();
        let margin_width_px = crate::unit::mm_to_px(self.margin_width as f32, dpi) as i32;
        if let Some(code_area) = self.children[2].downcast_mut::<super::code_area::CodeArea>() {
            let columns_count =
                (code_area.rect().width() as i32 - 2 * margin_width_px) / char_width;
            let lines_count =
                (code_area.rect().height() as i32 - 2 * margin_width_px) / line_height;
            self.size = (lines_count as usize, columns_count as usize);
            code_area.update(self.font_size, self.margin_width);
        }
        if let Some(bottom_bar) = self.children[8].downcast_mut::<super::bottom_bar::BottomBar>() {
            bottom_bar.update_font_size(self.font_size, rq);
            bottom_bar.update_margin_width(self.margin_width, rq);
        }
    }

    pub(super) fn set_font_size(
        &mut self,
        font_size: f32,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        self.font_size = font_size;
        self.update_size(rq, context);
        self.refresh(context);
    }

    pub(super) fn set_margin_width(
        &mut self,
        margin_width: i32,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        self.margin_width = margin_width;
        self.update_size(rq, context);
        self.refresh(context);
    }

    pub(super) fn toggle_margin_width_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(index) = locate_by_id(self, ViewId::MarginWidthMenu) {
            if let Some(true) = enable {
                return;
            }

            rq.add(RenderData::expose(
                *self.child(index).rect(),
                UpdateMode::Gui,
            ));
            self.children.remove(index);
        } else {
            if let Some(false) = enable {
                return;
            }

            let entries = (0..=10)
                .map(|mw| {
                    EntryKind::RadioButton(
                        format!("{}", mw),
                        EntryId::SetMarginWidth(mw),
                        mw == self.margin_width,
                    )
                })
                .collect();
            let margin_width_menu = Menu::new(
                rect,
                ViewId::MarginWidthMenu,
                MenuKind::DropDown,
                entries,
                context,
            );
            rq.add(RenderData::new(
                margin_width_menu.id(),
                *margin_width_menu.rect(),
                UpdateMode::Gui,
            ));
            self.children
                .push(Box::new(margin_width_menu) as Box<dyn View>);
        }
    }

    pub(super) fn toggle_font_size_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(index) = locate_by_id(self, ViewId::FontSizeMenu) {
            if let Some(true) = enable {
                return;
            }

            rq.add(RenderData::expose(
                *self.child(index).rect(),
                UpdateMode::Gui,
            ));
            self.children.remove(index);
        } else {
            if let Some(false) = enable {
                return;
            }

            let entries = (0..=20)
                .map(|v| {
                    let fs = 6.0 + v as f32 / 10.0;
                    EntryKind::RadioButton(
                        format!("{:.1}", fs),
                        EntryId::SetFontSize(v),
                        (fs - self.font_size).abs() < 0.05,
                    )
                })
                .collect();
            let font_size_menu = Menu::new(
                rect,
                ViewId::FontSizeMenu,
                MenuKind::DropDown,
                entries,
                context,
            );
            rq.add(RenderData::new(
                font_size_menu.id(),
                *font_size_menu.rect(),
                UpdateMode::Gui,
            ));
            self.children
                .push(Box::new(font_size_menu) as Box<dyn View>);
        }
    }

    pub(super) fn reseed(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if let Some(top_bar) = self.child_mut(0).downcast_mut::<TopBar>() {
            top_bar.reseed(rq, context);
        }

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    pub(super) fn quit(&mut self, context: &mut Context) {
        // SAFETY: Process ID is valid. Signal is a valid POSIX signal.
        unsafe { libc::kill(self.process.id() as libc::pid_t, libc::SIGTERM) };
        self.process
            .wait()
            .map_err(|e| crate::log_error!("Can't wait for child process: {:#}.", e))
            .ok();
        context.settings.calculator.font_size = self.font_size;
        context.settings.calculator.margin_width = self.margin_width;
    }

    pub(super) fn resize_view(
        &mut self,
        rect: Rectangle,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let dpi = CURRENT_DEVICE.dpi;
        let (small_height, big_height) = (
            scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
            scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32,
        );
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);
        let side = small_height;

        self.children.retain(|child| !child.is::<Menu>());

        // Top bar.
        let top_bar_rect = rect![
            rect.min.x,
            rect.min.y,
            rect.max.x,
            rect.min.y + side - small_thickness
        ];
        self.children[0].resize(top_bar_rect, hub, rq, context);

        let separator_rect = rect![
            rect.min.x,
            rect.min.y + side - small_thickness,
            rect.max.x,
            rect.min.y + side + big_thickness
        ];
        self.children[1].resize(separator_rect, hub, rq, context);

        let kb_rect = rect![
            rect.min.x,
            rect.max.y - (small_height + 3 * big_height) as i32 + big_thickness,
            rect.max.x,
            rect.max.y - small_height - small_thickness
        ];
        self.children[6].resize(kb_rect, hub, rq, context);
        let kb_rect = *self.children[6].rect();

        let sp_rect = rect![
            rect.min.x,
            kb_rect.min.y - thickness,
            rect.max.x,
            kb_rect.min.y
        ];

        let sp_rect2 = rect![
            rect.min.x,
            sp_rect.min.y - side,
            rect.max.x,
            sp_rect.min.y - side + thickness
        ];

        let input_bar_rect = rect![
            rect.min.x,
            sp_rect.min.y - side + thickness,
            rect.max.x,
            sp_rect.min.y
        ];

        let code_area_rect = rect![
            rect.min.x,
            rect.min.y + side + big_thickness,
            rect.max.x,
            sp_rect2.min.y
        ];

        self.children[2].resize(code_area_rect, hub, rq, context);
        self.children[3].resize(sp_rect2, hub, rq, context);
        self.children[4].resize(input_bar_rect, hub, rq, context);
        self.children[5].resize(sp_rect, hub, rq, context);

        let sp_rect = rect![
            rect.min.x,
            rect.max.y - side - small_thickness,
            rect.max.x,
            rect.max.y - side + big_thickness
        ];

        self.children[7].resize(sp_rect, hub, rq, context);

        let bottom_bar_rect = rect![
            rect.min.x,
            rect.max.y - side + big_thickness,
            rect.max.x,
            rect.max.y
        ];

        self.children[8].resize(bottom_bar_rect, hub, rq, context);

        for i in 9..self.children.len() {
            self.children[i].resize(rect, hub, rq, context);
        }

        self.update_size(&mut RenderQueue::new(), context);
        self.refresh(context);

        self.rect = rect;
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Full));
    }
}
