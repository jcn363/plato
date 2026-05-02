//! Plato UI Module
//!
//! This crate provides UI component and view functionality for Plato.

pub use plato_core::view::{
    handle_event, impl_view_boilerplate, process_render_queue, render, wait_for_all, Align, AppCmd,
    Bus, EntryId, EntryKind, Event, Hub, Id, IdFeeder, KeyboardEvent, PluginTriggerKind, SliderId,
    TextKind, View, ViewId, ID_FEEDER,
};
