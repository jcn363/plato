use super::dom::NodeRef;
use super::layout::TextAlign;
use super::layout::{
    ChildArtifact, DrawCommand, DrawState, Fonts, LoopContext, RootData, SiblingStyle, StyleData,
};
use super::style::StyleSheet;

// Include modularized components
use crate::geom::{Edge, Point, Rectangle};
use crate::settings::{
    DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT, DEFAULT_MARGIN_WIDTH, DEFAULT_TEXT_ALIGN,
};
use crate::settings::{HYPHEN_PENALTY, STRETCH_TOLERANCE};
use crate::unit::mm_to_px;
use anyhow::Error;

const DEFAULT_DPI: u16 = 300;
const DEFAULT_WIDTH: u32 = 1404;
const DEFAULT_HEIGHT: u32 = 1872;

// Math tag detection moved to engine_helpers module

pub type Page = Vec<DrawCommand>;

pub trait ResourceFetcher {
    fn fetch(&mut self, name: &str) -> Result<Vec<u8>, Error>;
}

// Minimum font size in points (implemented via max in layout)
pub const DEFAULT_MIN_FONT_SIZE: f32 = 4.0;

pub struct Engine {
    // The fonts used for each CSS font family.
    fonts: Option<Fonts>,
    // The minimum font size in points.
    min_font_size: f32,
    // The penalty for lines ending with a hyphen.
    hyphen_penalty: i32,
    // The stretching/shrinking allowed for word spaces.
    stretch_tolerance: f32,
    // Page margins in pixels.
    pub margin: Edge,
    // Font size in points.
    pub font_size: f32,
    // Text alignment.
    pub text_align: TextAlign,
    // Line height in ems.
    pub line_height: f32,
    // Page dimensions in pixels.
    pub dims: (u32, u32),
    // Device DPI.
    pub dpi: u16,
}

impl Engine {
    pub fn new() -> Engine {
        let margin =
            Edge::uniform(mm_to_px(DEFAULT_MARGIN_WIDTH as f32, DEFAULT_DPI).round() as i32);
        let line_height = DEFAULT_LINE_HEIGHT;

        Engine {
            fonts: None,
            min_font_size: DEFAULT_MIN_FONT_SIZE,
            hyphen_penalty: HYPHEN_PENALTY,
            stretch_tolerance: STRETCH_TOLERANCE,
            margin,
            font_size: DEFAULT_FONT_SIZE,
            text_align: DEFAULT_TEXT_ALIGN,
            line_height,
            dims: (DEFAULT_WIDTH, DEFAULT_HEIGHT),
            dpi: DEFAULT_DPI,
        }
    }

    #[inline]
    pub fn load_fonts(&mut self) {
        // TODO: Implement font loading
        // This is a placeholder to maintain API compatibility
    }

    pub fn set_min_font_size(&mut self, min_font_size: f32) {
        self.min_font_size = min_font_size;
    }

    pub fn set_hyphen_penalty(&mut self, hyphen_penalty: i32) {
        self.hyphen_penalty = hyphen_penalty;
    }

    pub fn set_stretch_tolerance(&mut self, stretch_tolerance: f32) {
        self.stretch_tolerance = stretch_tolerance;
    }

    pub fn set_margin(&mut self, margin: &Edge) {
        self.margin = *margin;
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }

    pub fn layout(&mut self, width: u32, height: u32, font_size: f32, dpi: u16) {
        self.dims = (width, height);
        self.font_size = font_size;
        self.dpi = dpi;
        self.margin = Edge::uniform(mm_to_px(DEFAULT_MARGIN_WIDTH as f32, self.dpi).round() as i32);
    }

    pub fn set_text_align(&mut self, text_align: TextAlign) {
        self.text_align = text_align;
    }

    pub fn set_font_family(&mut self, _family_name: &str, _search_path: &str) {
        // TODO: Implement font family setting
        // This method is expected by the Document trait but not yet implemented
    }

    pub fn set_margin_width(&mut self, width: i32) {
        self.margin = Edge::uniform(width);
    }

    pub fn set_line_height(&mut self, line_height: f32) {
        self.line_height = line_height;
    }

    pub fn rect(&self) -> Rectangle {
        let min = Point::new(self.margin.left, self.margin.top);
        let max = Point::new(
            self.dims.0 as i32 - self.margin.right,
            self.dims.1 as i32 - self.margin.bottom,
        );
        Rectangle::new(min, max)
    }

    /// Build display list (simplified version for compilation)
    pub fn build_display_list(
        &mut self,
        _node: NodeRef,
        _parent_style: &StyleData,
        _loop_context: &LoopContext,
        _stylesheet: &StyleSheet,
        _root_data: &RootData,
        _resource_fetcher: &mut dyn ResourceFetcher,
        _draw_state: &mut DrawState,
        _display_list: &mut Vec<Page>,
    ) -> ChildArtifact {
        // TODO: Implement build_display_list method
        // This is a placeholder to maintain API compatibility
        ChildArtifact {
            sibling_style: SiblingStyle {
                padding: Default::default(),
                margin: Default::default(),
            },
            rects: vec![None],
        }
    }

    /// Render a page to pixmap
    pub fn render_page(
        &mut self,
        page: &[DrawCommand],
        scale_factor: f32,
        samples: usize,
        resource_fetcher: &mut dyn ResourceFetcher,
    ) -> Option<crate::framebuffer::Pixmap> {
        // TODO: Implement render_page method
        // This is a placeholder to maintain API compatibility
        None
    }
}
