//! Reader Settings Module
//!
//! Handles all font, contrast, zoom settings menus and configuration.
//!
//! ## Submodules
//! - `menu_toggles` - Font family, size, alignment, line height, contrast, margin menus
//! - `context_menus` - Annotation, selection, and title context menus
//! - `helpers` - Helper functions for updating settings

mod menu_toggles;
mod context_menus;
mod helpers;

pub(crate) use menu_toggles::*;
pub(crate) use context_menus::*;
pub(crate) use helpers::*;
