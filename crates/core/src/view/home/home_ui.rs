//! Home UI Module - Layout and Rendering Helpers
//!
//! This module contains UI layout helper methods for the Home view.
//! It handles child view creation, resizing, and dimension calculations.
//!
//! ## Module Structure
//!
//! - `add_*` methods - Create and add child views during initialization
//! - `resize_*` methods - Resize child views during layout updates
//! - `calculate_*` methods - Compute dimensions and helper values
//!
//! ## Design Notes
//!
//! This module was extracted from `mod.rs` as part of the Phase 5 refactoring
//! to separate UI layout concerns from data model and event handling per
//! AGENTS.md modular design rules. All methods are static or take explicit
//! parameters to maintain clear dependencies.

use std::path::Path;

use crate::context::Context;
use crate::geom::{halves, Rectangle};
use crate::metadata::SortMethod;
use crate::settings::FirstColumn;
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::filler::Filler;
use crate::view::top_bar::TopBar;
use crate::view::{Event, Hub, RenderQueue, View, ViewId};
use crate::view::{BIG_BAR_HEIGHT, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};

use super::home_core::Home;

impl Home {
    /// Calculate initial layout dimensions
    pub fn calculate_dimensions() -> (u16, i32, i32, i32, i32, i32) {
        let dpi = crate::unit::get_device_dpi();
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let big_height = scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32;

        (
            dpi,
            thickness,
            small_thickness,
            big_thickness,
            small_height,
            big_height,
        )
    }

    /// Get library settings from context
    pub fn get_library_settings(
        context: &Context,
    ) -> (
        usize,
        crate::metadata::SortMethod,
        bool,
    ) {
        let selected_library = context.settings.selected_library;
        let library = &context.library;
        let sort_method = library.sort_method;
        let reverse_order = library.reverse_order;

        (
            selected_library,
            sort_method,
            reverse_order,
        )
    }

    /// Add top bar to children vector
    pub fn add_top_bar(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        small_height: i32,
        small_thickness: i32,
        big_thickness: i32,
        sort_method: SortMethod,
        context: &mut Context,
    ) {
        let top_bar = TopBar::new(
            rect![
                rect.min.x,
                rect.min.y,
                rect.max.x,
                rect.min.y + small_height - small_thickness
            ],
            Event::Toggle(ViewId::SearchBar),
            sort_method.title(),
            context,
        );
        children.push(Box::new(top_bar) as Box<dyn View>);

        let separator = Filler::new(
            rect![
                rect.min.x,
                rect.min.y + small_height - small_thickness,
                rect.max.x,
                rect.min.y + small_height + big_thickness
            ],
            crate::color::foreground(theme::is_dark_mode()),
        );
        children.push(Box::new(separator) as Box<dyn View>);
    }

    /// Add address bar if enabled in settings
    pub fn add_address_bar_if_enabled(
        children: &mut Vec<Box<dyn View>>,
        context: &mut Context,
        rect: Rectangle,
        y_start: i32,
        thickness: i32,
        small_height: i32,
        _small_thickness: i32,
        current_directory: &Path,
        _shelf_index: usize,
    ) -> i32 {
        let mut y_start = y_start;
        if context.settings.home.address_bar {
            let addr_bar = super::address_bar::AddressBar::new(
                rect![
                    rect.min.x,
                    y_start,
                    rect.max.x,
                    y_start + small_height - thickness
                ],
                current_directory.to_string_lossy(),
                context,
            );
            children.push(Box::new(addr_bar) as Box<dyn View>);
            y_start += small_height - thickness;

            let separator = Filler::new(
                rect![rect.min.x, y_start, rect.max.x, y_start + thickness],
                crate::color::foreground(theme::is_dark_mode()),
            );
            children.push(Box::new(separator) as Box<dyn View>);
            y_start += thickness;
        }
        y_start
    }

    /// Add navigation bar if enabled in settings
    pub fn add_navigation_bar_if_enabled(
        children: &mut Vec<Box<dyn View>>,
        context: &mut Context,
        rect: Rectangle,
        y_start: i32,
        thickness: i32,
        small_height: i32,
        small_thickness: i32,
        _current_directory: &std::path::Path,
    ) -> i32 {
        let mut y_start = y_start;
        if context.settings.home.navigation_bar {
            let separator = Filler::new(
                rect![
                    rect.min.x,
                    y_start,
                    rect.max.x,
                    y_start + small_thickness
                ],
                crate::color::foreground(theme::is_dark_mode()),
            );
            children.push(Box::new(separator) as Box<dyn View>);
            y_start += small_thickness;

            let nav_bar = super::navigation_bar::NavigationBar::new(
                rect![
                    rect.min.x,
                    y_start,
                    rect.max.x,
                    y_start + small_height - thickness
                ],
                5, // vertical_limit (max levels to show)
                3, // max_levels
            );
            children.push(Box::new(nav_bar) as Box<dyn View>);
            y_start += small_height - thickness;

            let separator = Filler::new(
                rect![rect.min.x, y_start, rect.max.x, y_start + thickness],
                crate::color::foreground(theme::is_dark_mode()),
            );
            children.push(Box::new(separator) as Box<dyn View>);
            y_start += thickness;
        }
        y_start
    }

    /// Add shelf and bottom bar to children vector
    #[allow(clippy::too_many_arguments)]
    pub fn add_shelf_and_bottom_bar(
        children: &mut Vec<Box<dyn View>>,
        hub: &Hub,
        context: &mut Context,
        rect: Rectangle,
        y_min: i32,
        y_max: i32,
        current_page: usize,
        pages_count: usize,
        visible_books: &crate::metadata::Metadata,
        sort_method: SortMethod,
        reverse_order: bool,
    ) -> usize {
        // Shelf
        let shelf = super::shelf::Shelf::new(
            rect![rect.min.x, y_min, rect.max.x, y_max],
            visible_books.clone(),
            current_page,
            context,
        );
        children.push(Box::new(shelf) as Box<dyn View>);

        // Bottom bar
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, crate::unit::get_device_dpi()) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, crate::unit::get_device_dpi()) as i32;
        let small_thickness = (thickness + thickness % 2) / 2;

        let bottom_bar = super::bottom_bar::BottomBar::new(
            rect![
                rect.min.x,
                rect.max.y - small_height - thickness,
                rect.max.x,
                rect.max.y
            ],
            current_page,
            pages_count,
            hub,
            context,
        );
        children.push(Box::new(bottom_bar) as Box<dyn View>);

        let separator = Filler::new(
            rect![
                rect.min.x,
                rect.max.y - small_height - thickness,
                rect.max.x,
                rect.max.y - small_height + small_thickness
            ],
            crate::color::foreground(theme::is_dark_mode()),
        );
        children.push(Box::new(separator) as Box<dyn View>);

        // Return shelf index (second to last before bottom bar separator and bottom bar)
        children.len() - 3
    }

    /// Calculate resize dimensions
    pub fn calculate_resize_dimensions() -> (i32, i32, i32, i32, i32) {
        let dpi = crate::unit::get_device_dpi();
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let big_height = scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32;

        (
            thickness,
            small_thickness,
            big_thickness,
            small_height,
            big_height,
        )
    }

    /// Resize top bar
    pub fn resize_top_bar(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        small_height: i32,
        small_thickness: i32,
        big_thickness: i32,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let top_bar_rect = rect![
            rect.min.x,
            rect.min.y,
            rect.max.x,
            rect.min.y + small_height - small_thickness
        ];
        children[0].resize(top_bar_rect, hub, rq, context);

        let separator_rect = rect![
            rect.min.x,
            rect.min.y + small_height - small_thickness,
            rect.max.x,
            rect.min.y + small_height + big_thickness
        ];
        children[1].resize(separator_rect, hub, rq, context);
    }

    /// Resize address bar if enabled
    pub fn resize_address_bar_if_enabled(
        children: &mut Vec<Box<dyn View>>,
        context: &mut Context,
        rect: Rectangle,
        y_min: i32,
        thickness: i32,
        small_height: i32,
        _index: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
    ) -> i32 {
        let mut y_min = y_min;
        if context.settings.home.address_bar {
            let address_bar_rect = rect![rect.min.x, y_min, rect.max.x, y_min + small_height];
            children[2].resize(address_bar_rect, hub, rq, context);
            y_min += small_height;

            let separator_rect = rect![rect.min.x, y_min, rect.max.x, y_min + thickness];
            children[3].resize(separator_rect, hub, rq, context);
            y_min += thickness;
        }
        y_min
    }

    /// Get address bar end Y position
    pub fn get_address_bar_end_y(
        context: &mut Context,
        _rect: Rectangle,
        shelf_min_y: i32,
        small_height: i32,
        thickness: i32,
    ) -> i32 {
        if context.settings.home.address_bar {
            small_height + thickness
        } else {
            shelf_min_y
        }
    }

    /// Resize navigation bar if enabled
    pub fn resize_navigation_bar_if_enabled(
        children: &mut Vec<Box<dyn View>>,
        context: &mut Context,
        rect: Rectangle,
        y_min: i32,
        thickness: i32,
        small_height: i32,
        small_thickness: i32,
        hub: &Hub,
        rq: &mut RenderQueue,
    ) -> (usize, i32) {
        let mut index = 2;
        let mut y_min = y_min;

        if context.settings.home.address_bar {
            index += 2;
            y_min += small_height + thickness;
        }

        if context.settings.home.navigation_bar {
            let separator_rect = rect![rect.min.x, y_min, rect.max.x, y_min + small_thickness];
            children[index].resize(separator_rect, hub, rq, context);
            y_min += small_thickness;
            index += 1;

            let nav_bar_rect =
                rect![rect.min.x, y_min, rect.max.x, y_min + small_height - thickness];
            children[index].resize(nav_bar_rect, hub, rq, context);
            y_min += small_height - thickness;
            index += 1;

            let separator_rect = rect![rect.min.x, y_min, rect.max.x, y_min + thickness];
            children[index].resize(separator_rect, hub, rq, context);
            y_min += thickness;
            index += 1;
        }

        (index, y_min)
    }

    /// Resize bottom bar
    pub fn resize_bottom_bar(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        small_height: i32,
        thickness: i32,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> usize {
        let bottom_bar_index = children.len() - 2;
        let bottom_bar_rect = rect![
            rect.min.x,
            rect.max.y - small_height - thickness,
            rect.max.x,
            rect.max.y
        ];
        children[bottom_bar_index].resize(bottom_bar_rect, hub, rq, context);
        bottom_bar_index
    }

    /// Resize keyboard and search bar
    pub fn resize_keyboard_and_search_bar(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        bottom_bar_index: usize,
        small_height: i32,
        big_height: i32,
        thickness: i32,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> i32 {
        let mut shelf_max_y = rect.max.y - small_height - thickness;

        // Resize keyboard if present
        if bottom_bar_index > 2 {
            let has_keyboard = children.len() > bottom_bar_index + 2;
            if has_keyboard {
                let keyboard_index = bottom_bar_index - 1;
                let keyboard_rect = rect![
                    rect.min.x,
                    rect.max.y - small_height - big_height - thickness,
                    rect.max.x,
                    rect.max.y - small_height - thickness
                ];
                children[keyboard_index].resize(keyboard_rect, hub, rq, context);
                shelf_max_y -= big_height;
            }
        }

        shelf_max_y
    }

    /// Resize shelf
    pub fn resize_shelf(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        shelf_min_y: i32,
        shelf_max_y: i32,
        shelf_index: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let shelf_rect = rect![rect.min.x, shelf_min_y, rect.max.x, shelf_max_y];
        children[shelf_index].resize(shelf_rect, hub, rq, context);
    }

    /// Resize floating windows
    pub fn resize_floating_windows(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        bottom_bar_index: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        for i in bottom_bar_index + 2..children.len() {
            let child_rect = *children[i].rect();
            let new_rect = rect![
                child_rect.min.x,
                child_rect.min.y,
                child_rect.max.x.min(rect.max.x),
                child_rect.max.y.min(rect.max.y)
            ];
            children[i].resize(new_rect, hub, rq, context);
        }
    }
}
