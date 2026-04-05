//! Views are organized as a tree. A view might receive / send events and render itself.
//!
//! The z-level of the n-th child of a view is less or equal to the z-level of its n+1-th child.
//!
//! Events travel from the root to the leaves, only the leaf views will handle the root events, but
//! any view can send events to its parent. From the events it receives from its children, a view
//! resends the ones it doesn't handle to its own parent. Hence an event sent from a child might
//! bubble up to the root. If it reaches the root without being captured by any view, then it will
//! be written to the main event channel and will be sent to every leaf in one of the next loop
//! iterations.

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
