//! View System Module
//!
//! This module provides the UI view system for Plato, organizing views as a tree structure
//! with event propagation and rendering capabilities.
//!
//! ## Architecture
//!
//! Views are organized as a tree with the following properties:
//! - The z-level of the n-th child is <= the z-level of its (n+1)-th sibling
//! - Events travel from root to leaves, with only leaf views handling root events
//! - Events bubble up from children to parents if not handled
//! - Unhandled events reaching the root are broadcast to all leaves
//!
//! ## Module Organization
//!
//! Views are grouped by functional domain:
//!
//! ### Core UI Components
//! - **button.rs, rounded_button.rs**: Clickable buttons
//! - **label.rs**: Text display
//! - **input_field.rs, named_input.rs**: Text input
//! - **icon.rs**: Icon display
//! - **menu.rs**: Context menus
//! - **dialog.rs**: Modal dialogs
//! - **slider.rs**: Value sliders
//!
//! ### Reader View (`reader/`)
//! - **reader_impl/**: Reader implementation split into focused modules
//!   - Core rendering, navigation, annotations, search, settings
//! - **tool_bar/**: Reader toolbar with layout sub-module
//! - **margin_cropper.rs**: Interactive margin cropping
//! - **chapter_label.rs, results_label.rs, bottom_bar.rs**: Reader UI elements
//!
//! ### Home View (`home/`)
//! - **mod.rs**: Home view main implementation
//! - **shelf.rs**: Book shelf display
//! - **book.rs**: Book item rendering
//! - **directories_bar.rs**: Directory navigation
//! - **navigation_bar.rs, address_bar.rs**: Navigation controls
//! - **ui_toggles/**: Settings toggle components
//!
//! ### Specialized Views
//! - **dictionary/**: Dictionary lookup UI
//! - **sketch/**: Sketch/drawing view
//! - **calculator/**: Calculator widget
//! - **keyboard.rs**: On-screen keyboard
//! - **epub_editor/**: EPUB editing interface
//! - **cover_editor.rs**: Book cover editing
//! - **pdf_manipulator.rs**: PDF tools UI
//!
//! ### System Views
//! - **intermission.rs**: Sleep/power-off screens
//! - **notification.rs**: Notification display
//! - **battery.rs, clock.rs**: Status indicators
//! - **frontlight.rs**: Frontlight controls
//! - **search_bar.rs, search_replace.rs**: Search interface
//!
//! ## Module Hierarchy
//!
//! ```text
//! view/
//! ├── mod.rs              (view system core)
//! ├── common.rs           (shared view utilities)
//! ├── identifiers.rs      (ViewId definitions)
//! ├── rendering.rs        (render queue and constants)
//! ├── menu_helpers.rs     (menu utility functions)
//! │
//! ├── reader/             (reader view subtree)
//! │   ├── mod.rs
//! │   ├── reader_impl/    (implementation modules)
//! │   ├── tool_bar/
//! │   └── ... (UI elements)
//! │
//! ├── home/               (home view subtree)
//! │   ├── mod.rs
//! │   ├── shelf.rs
//! │   ├── book.rs
//! │   └── ... (sub-components)
//! │
//! ├── dictionary/         (dictionary subtree)
//! ├── sketch/             (sketch subtree)
//! ├── calculator/         (calculator subtree)
//! └── epub_editor/        (EPUB editor subtree)
//! ```
//!
//! ## Event Flow
//!
//! 1. Events enter at root, travel down to leaves
//! 2. Leaf views handle events they recognize
//! 3. Unhandled events bubble up to parent
//! 4. Events reaching root are queued for broadcast
//!
//! ## Dependencies
//!
//! Views depend on:
//! - `geom` - Geometry and positioning
//! - `framebuffer` - Rendering output
//! - `font` - Text rendering
//! - `input` - Event definitions

pub mod battery;
pub mod button;
pub mod calculator;
pub mod clock;
pub mod common;
pub mod cover_editor;
pub mod dialog;
pub mod dictionary;
pub mod epub_editor;
pub mod filler;
pub mod frontlight;
pub mod home;
pub mod icon;
pub mod image;
pub mod input_field;
pub mod intermission;
pub mod key;
pub mod keyboard;
pub mod label;
pub mod labeled_icon;
pub mod menu;
pub mod menu_entry;
pub mod menu_helpers;
pub mod named_input;
pub mod notification;
pub mod page_label;
pub mod pdf_manipulator;
pub mod preset;
pub mod presets_list;
pub mod reader;
pub mod rotation_values;
pub mod rounded_button;
pub mod search_bar;
pub mod search_replace;
pub mod settings;
pub mod sketch;
pub mod slider;
pub mod statistics;
pub mod top_bar;
pub mod touch_events;

mod entries;
mod event_dispatch;
mod events;
mod identifiers;
mod rendering;
mod view_trait;

pub use self::entries::{Align, EntryId, EntryKind, TextKind};
pub use self::event_dispatch::{handle_event, process_render_queue, render, wait_for_all};
pub use self::events::{Bus, Event, Hub, KeyboardEvent};
pub use self::identifiers::{AppCmd, Id, IdFeeder, PluginTriggerKind, SliderId, ViewId, ID_FEEDER};
pub use self::rendering::{
    RenderData, RenderQueue, UpdateData, BIG_BAR_HEIGHT, BORDER_RADIUS_LARGE, BORDER_RADIUS_MEDIUM,
    BORDER_RADIUS_SMALL, CLOSE_IGNITION_DELAY, MAX_UPDATE_DELAY, SMALL_BAR_HEIGHT, THICKNESS_LARGE,
    THICKNESS_MEDIUM, THICKNESS_SMALL,
};
pub use self::view_trait::View;
pub use crate::impl_view_boilerplate;
