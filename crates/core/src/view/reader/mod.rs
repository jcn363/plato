mod bottom_bar;
mod chapter_label;
mod margin_cropper;
mod reader_impl;
mod results_bar;
mod results_label;
mod tool_bar;

pub use reader_impl::{Contrast, Reader, Selection, State};

pub use crate::view::{
    AppCmd, Bus, EntryId, EntryKind, Event, Hub, Id, RenderData, RenderQueue, SliderId, View,
    ViewId, BIG_BAR_HEIGHT, ID_FEEDER, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM,
};
