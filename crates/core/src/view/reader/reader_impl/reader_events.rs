//! Reader Event Handling
//!
//! This module handles user input events for the Reader view.
//! It processes touch events, keyboard input, and menu interactions,
//! dispatching to appropriate handlers based on the current state.
//!
//! The main entry point is `handle_menu_event`, which processes
//! events when menus are active or when standard reader interactions
//! need to be handled.
use crate::context::Context;
use crate::geom::Rectangle;
use crate::view::{Hub, RenderQueue, ViewId, SliderId};
use crate::view::menu::{toggle_main_menu, toggle_battery_menu, toggle_clock_menu};
use crate::input::{FingerStatus};

use super::reader::Reader;

impl Reader {
    pub(crate) fn handle_menu_event(&mut self, evt: &crate::view::Event, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) -> bool {
        use crate::view::Event;
        match evt {
            Event::Update(mode) => {
                self.update(Some(*mode), hub, rq, context);
                true
            }
            Event::LoadPixmap(location) => {
                self.load_pixmap(*location, hub, rq, context);
                true
            }
            Event::Submit(ViewId::GoToPageInput, ref text) => {
                self.handle_go_to_page_submit(text.parse().unwrap_or(0), hub, rq, context);
                true
            }
            Event::Submit(ViewId::GoToResultsPageInput, ref text) => {
                if let Ok(index) = text.parse::<usize>() {
                    self.go_to_results_page(index.saturating_sub(1), hub, rq, context);
                }
                true
            }
            Event::Submit(ViewId::NamePageInput, ref text) => {
                if !text.is_empty() {
                    if let Some(ref mut r) = self.info.reader {
                        r.page_names.insert(self.current_page, text.to_string());
                    }
                }
                self.toggle_keyboard(false, None, hub, rq, context);
                true
            }
            Event::Submit(ViewId::EditNoteInput, ref note) => {
                self.handle_edit_note_submit(note, hub, rq, context);
                true
            }
            Event::Submit(ViewId::ReaderSearchInput, ref text) => {
                self.handle_search_submit(text, hub, rq, context);
                true
            }
            Event::Page(dir) => {
                self.go_to_neighbor(*dir, hub, rq, context);
                true
            }
            Event::GoTo(location) | Event::Select(crate::view::EntryId::GoTo(location)) => {
                self.go_to_page(*location, true, hub, rq, context);
                true
            }
            Event::GoToLocation(ref location) => {
                self.handle_go_to_location(location, hub, rq, context);
                true
            }
            Event::Chapter(dir) => {
                self.go_to_chapter(*dir, hub, rq, context);
                true
            }
            Event::ResultsPage(dir) => {
                self.go_to_results_neighbor(*dir, hub, rq, context);
                true
            }
            Event::CropMargins(ref margin) => {
                let current_page = self.current_page;
                self.crop_margins(current_page, margin.as_ref(), hub, rq, context);
                true
            }
            Event::Toggle(ViewId::TopBottomBars) => {
                self.toggle_bars(None, hub, rq, context);
                true
            }
            Event::Toggle(ViewId::GoToPage) => {
                self.toggle_go_to_page(None, ViewId::GoToPage, hub, rq, context);
                true
            }
            Event::Toggle(ViewId::GoToResultsPage) => {
                self.toggle_go_to_page(None, ViewId::GoToResultsPage, hub, rq, context);
                true
            }
            Event::Slider(SliderId::FontSize, font_size, FingerStatus::Up) => {
                self.set_font_size(*font_size, hub, rq, context);
                true
            }
            Event::Slider(SliderId::ContrastExponent, exponent, FingerStatus::Up) => {
                self.set_contrast_exponent(*exponent, hub, rq, context);
                true
            }
            Event::Slider(SliderId::ContrastGray, gray, FingerStatus::Up) => {
                self.set_contrast_gray(*gray, hub, rq, context);
                true
            }
            Event::ToggleNear(ViewId::TitleMenu, rect) => {
                self.toggle_title_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::MainMenu, rect) => {
                toggle_main_menu(self, *rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::BatteryMenu, rect) => {
                toggle_battery_menu(self, *rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::ClockMenu, rect) => {
                toggle_clock_menu(self, *rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::MarginCropperMenu, rect) => {
                self.toggle_margin_cropper_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::SearchMenu, rect) => {
                self.toggle_search_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::FontFamilyMenu, rect) => {
                self.toggle_font_family_menu(*rect, None, hub, rq, context);
                true
            }
            Event::ToggleNear(ViewId::FontSizeMenu, rect) => {
                self.toggle_font_size_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::TextAlignMenu, rect) => {
                self.toggle_text_align_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::MarginWidthMenu, rect) => {
                self.toggle_margin_width_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::LineHeightMenu, rect) => {
                self.toggle_line_height_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::ContrastExponentMenu, rect) => {
                self.toggle_contrast_exponent_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::ContrastGrayMenu, rect) => {
                self.toggle_contrast_gray_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::PageMenu, rect) => {
                self.toggle_page_menu(*rect, None, rq, context);
                true
            }
            Event::Close(ViewId::MainMenu) => {
                toggle_main_menu(self, Rectangle::default(), Some(false), rq, context);
                true
            }
            Event::Close(ViewId::SearchBar) => {
                self.handle_close_search_bar(hub, rq, context);
                true
            }
            Event::Close(ViewId::GoToPage) => {
                self.toggle_go_to_page(Some(false), ViewId::GoToPage, hub, rq, context);
                true
            }
            Event::Close(ViewId::GoToResultsPage) => {
                self.toggle_go_to_page(Some(false), ViewId::GoToResultsPage, hub, rq, context);
                true
            }
            _ => false,
        }
    }
}
