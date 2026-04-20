//! Reader Rendering Implementation
//!
//! This module provides rendering and layout methods for the Reader view.
//! It handles view resizing, layout calculations for UI components (toolbars,
//! menus, search bars), and rendering helper functions.
//!
//! The main methods include:
//! - `resize`: Handles view resizing and child view repositioning
//! - `render_rect`: Calculates visible rectangles for rendering operations
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::document::Location;
use crate::framebuffer::UpdateMode;
use crate::geom::halves;
use crate::geom::Rectangle;
use crate::metadata::ZoomMode;
use crate::unit::scale_by_dpi;
use crate::view::common::locate;
use crate::view::filler::Filler;
use crate::view::keyboard::Keyboard;
use crate::view::menu::Menu;
use crate::view::reader::bottom_bar::BottomBar;
use crate::view::reader::tool_bar::ToolBar;
use crate::view::search_bar::SearchBar;
use crate::view::top_bar::TopBar;
use crate::view::{Hub, RenderQueue, BIG_BAR_HEIGHT, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};

use super::reader::Reader;

impl Reader {
    pub fn render_rect(&self, rect: &Rectangle) -> Rectangle {
        rect.intersection(&self.rect).unwrap_or(self.rect)
    }

    pub fn resize(
        &mut self,
        rect: Rectangle,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if !self.children.is_empty() {
            let dpi = crate::unit::get_device_dpi();
            let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
            let (small_thickness, big_thickness) = halves(thickness);
            let (small_height, big_height) = (
                scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
                scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32,
            );
            let mut floating_layer_start = 0;

            self.children.retain(|child| !child.is::<Menu>());

            if self.children[0].is::<TopBar>() {
                let top_bar_rect = rect![
                    rect.min.x,
                    rect.min.y,
                    rect.max.x,
                    small_height - small_thickness
                ];
                self.children[0].resize(top_bar_rect, hub, rq, context);
                let separator_rect = rect![
                    rect.min.x,
                    small_height - small_thickness,
                    rect.max.x,
                    small_height + big_thickness
                ];
                self.children[1].resize(separator_rect, hub, rq, context);
            } else if self.children[0].is::<Filler>() {
                let mut index = 1;
                if self.children[index].is::<SearchBar>() {
                    let sb_rect = rect![
                        rect.min.x,
                        rect.max.y - (3 * big_height + 2 * small_height) + big_thickness,
                        rect.max.x,
                        rect.max.y - (3 * big_height + small_height) - small_thickness
                    ];
                    self.children[index].resize(sb_rect, hub, rq, context);
                    self.children[index - 1].resize(
                        rect![
                            rect.min.x,
                            sb_rect.min.y - thickness,
                            rect.max.x,
                            sb_rect.min.y
                        ],
                        hub,
                        rq,
                        context,
                    );
                    index += 2;
                }
                if self.children[index].is::<Keyboard>() {
                    let kb_rect = rect![
                        rect.min.x,
                        rect.max.y - (small_height + 3 * big_height) + big_thickness,
                        rect.max.x,
                        rect.max.y - small_height - small_thickness
                    ];
                    self.children[index].resize(kb_rect, hub, rq, context);
                    self.children[index + 1].resize(
                        rect![
                            rect.min.x,
                            kb_rect.max.y,
                            rect.max.x,
                            kb_rect.max.y + thickness
                        ],
                        hub,
                        rq,
                        context,
                    );
                    let kb_rect = *self.children[index].rect();
                    self.children[index - 1].resize(
                        rect![
                            rect.min.x,
                            kb_rect.min.y - thickness,
                            rect.max.x,
                            kb_rect.min.y
                        ],
                        hub,
                        rq,
                        context,
                    );
                    index += 2;
                }
                floating_layer_start = index;
            }

            if let Some(mut index) = locate::<BottomBar>(self) {
                floating_layer_start = index + 1;
                let separator_rect = rect![
                    rect.min.x,
                    rect.max.y - small_height - small_thickness,
                    rect.max.x,
                    rect.max.y - small_height + big_thickness
                ];
                self.children[index - 1].resize(separator_rect, hub, rq, context);
                let bottom_bar_rect = rect![
                    rect.min.x,
                    rect.max.y - small_height + big_thickness,
                    rect.max.x,
                    rect.max.y
                ];
                self.children[index].resize(bottom_bar_rect, hub, rq, context);

                index -= 2;

                while index > 2 {
                    let bar_height = if self.children[index].is::<ToolBar>() {
                        2 * big_height
                    } else if self.children[index].is::<Keyboard>() {
                        3 * big_height
                    } else {
                        small_height
                    };

                    let y_max = self.children[index + 1].rect().min.y;
                    let bar_rect = rect![
                        rect.min.x,
                        y_max - bar_height + thickness,
                        rect.max.x,
                        y_max
                    ];
                    self.children[index].resize(bar_rect, hub, rq, context);
                    let y_max = self.children[index].rect().min.y;
                    let sp_rect = rect![rect.min.x, y_max - thickness, rect.max.x, y_max];
                    self.children[index - 1].resize(sp_rect, hub, rq, context);

                    index -= 2;
                }
            }

            for i in floating_layer_start..self.children.len() {
                self.children[i].resize(rect, hub, rq, context);
            }
        }

        match self.view_port.zoom_mode {
            ZoomMode::FitToWidth => {
                let ratio = (rect.width() as i32 - 2 * self.view_port.margin_width) as f32
                    / (self.rect.width() as i32 - 2 * self.view_port.margin_width) as f32;
                self.view_port.page_offset.y = (self.view_port.page_offset.y as f32 * ratio) as i32;
            }
            ZoomMode::Custom(_) => {
                self.view_port.page_offset += pt!(
                    self.rect.width() as i32 - rect.width() as i32,
                    self.rect.height() as i32 - rect.height() as i32
                ) / 2;
            }
            _ => (),
        }

        self.rect = rect;

        if self.reflowable {
            let font_size = self
                .info
                .reader
                .as_ref()
                .and_then(|r| r.font_size)
                .unwrap_or(context.settings.reader.font_size);
            let current_page = self.current_page;
            let location = {
                let mut doc = self._doc.lock().expect("Document lock poisoned");
                doc.layout(rect.width(), rect.height(), font_size, CURRENT_DEVICE.dpi);
                let current_page = current_page.min(doc.pages_count() - 1);
                doc.resolve_location(Location::Exact(current_page))
            };
            if let Some(location) = location {
                self.current_page = location;
            }
            self.text.clear();
        }

        self.cache.clear();
        self.update(Some(UpdateMode::Full), hub, rq, context);
    }

    pub fn might_rotate(&self) -> bool {
        self.search.is_none()
    }

    pub fn is_background(&self) -> bool {
        true
    }

    pub fn rect(&self) -> &Rectangle {
        &self.rect
    }
}
