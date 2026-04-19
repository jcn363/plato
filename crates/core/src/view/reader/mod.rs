mod chapter_label;
mod margin_cropper;
mod reader_impl;
pub mod results_label;
pub mod tool_bar;
pub mod bottom_bar;

pub use margin_cropper::MarginCropper;
pub use results_label::ResultsLabel;

pub use reader_impl::{
    Contrast, PageAnimKind, PageAnimation, Reader, RenderChunk, Resource, Selection, State,
    ViewPort,
};

pub use crate::view::{
    AppCmd, Bus, EntryId, EntryKind, Event, Hub, Id, RenderData, RenderQueue, SliderId, View,
    ViewId, BIG_BAR_HEIGHT, ID_FEEDER, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM,
};
