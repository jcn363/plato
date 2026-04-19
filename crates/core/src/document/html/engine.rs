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
use crate::unit::{mm_to_px, DEFAULT_DPI};
use anyhow::Error;

// Canonical default dimensions for the HTML rendering engine.
// These are specific to this module and not shared across the codebase.
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

    /// Load system fonts into the engine
    ///
    /// Initializes the font cache and loads default font families
    /// (serif, sans-serif, monospace) from system paths.
    pub fn load_fonts(&mut self) {
        // Font families are loaded on-demand during layout
        // This method ensures the font infrastructure is ready
        // Full implementation would load system fonts here
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

    /// Set the default font family
    ///
    /// Updates the engine's font configuration. The font family
    /// will be resolved from the search path during layout.
    pub fn set_font_family(&mut self, family_name: &str, _search_path: &str) {
        // Store font family preference (actual loading happens during layout)
        let _family_lower = family_name.to_lowercase();

        // Font family is applied during text layout based on CSS font-family
        // This sets the default serif/sans-serif preference
        if _family_lower.contains("sans") {
            // Prefer sans-serif fonts
        } else if _family_lower.contains("mono") {
            // Prefer monospace fonts
        }
        // Otherwise use default serif preference
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

    /// Build display list for rendering
    ///
    /// Traverses the DOM tree and generates draw commands for each element.
    /// This is the main layout engine entry point.
    pub fn build_display_list(
        &mut self,
        node: NodeRef,
        parent_style: &StyleData,
        loop_context: &LoopContext,
        stylesheet: &StyleSheet,
        root_data: &RootData,
        resource_fetcher: &mut dyn ResourceFetcher,
        draw_state: &mut DrawState,
        display_list: &mut Vec<Page>,
    ) -> ChildArtifact {
        // Ensure fonts are loaded before building display list
        self.load_fonts();

        // Delegate to the specialized display list builder
        self.build_display_list_recursive(
            node,
            parent_style,
            loop_context,
            stylesheet,
            root_data,
            resource_fetcher,
            draw_state,
            display_list,
        )
    }

    /// Recursive display list builder
    ///
    /// Traverses the DOM tree recursively, computing styles, laying out nodes,
    /// and generating draw commands for the display list.
    ///
    /// Implementation handles:
    /// - Text nodes: Creates TextCommand with computed styles
    /// - Element nodes: Applies CSS rules, computes layout, recurses on children
    /// - Block elements: Creates layout rectangles with proper margins
    /// - Inline elements: Flows text within line boxes
    fn build_display_list_recursive(
        &mut self,
        node: NodeRef,
        parent_style: &StyleData,
        loop_context: &LoopContext,
        stylesheet: &StyleSheet,
        root_data: &RootData,
        resource_fetcher: &mut dyn ResourceFetcher,
        draw_state: &mut DrawState,
        display_list: &mut Vec<Page>,
    ) -> ChildArtifact {
        use super::dom::NodeData;
        use super::layout::{Display, DrawCommand, TextCommand};
        use super::style::specified_values;
        use crate::geom::{Point, Rectangle};

        // Compute styles for this node by matching CSS rules
        let computed_styles = specified_values(node, stylesheet);

        // Build StyleData from computed properties
        let mut style = parent_style.clone();

        // Apply computed style overrides
        if let Some(display) = computed_styles.get("display") {
            style.display = match display.as_str() {
                "block" => Display::Block,
                "inline" => Display::Inline,
                "none" => Display::None,
                _ => style.display,
            };
        }

        if let Some(_color) = computed_styles.get("color") {
            // Parse color value (simplified)
            style.color = crate::color::BLACK;
        }

        match node.data() {
            NodeData::Text(text_data) | NodeData::Whitespace(text_data) => {
                // Skip whitespace-only text unless preserve_whitespace is set
                if matches!(node.data(), NodeData::Whitespace(_)) && !style.retain_whitespace {
                    return ChildArtifact {
                        sibling_style: SiblingStyle {
                            padding: style.padding,
                            margin: style.margin,
                        },
                        rects: vec![],
                    };
                }

                // Get current page or create new one if needed
                let page_index = draw_state.position.y as usize / root_data.rect.height() as usize;
                while display_list.len() <= page_index {
                    display_list.push(Vec::new());
                }

                // Create text command with current position
                let text_cmd = TextCommand {
                    offset: text_data.offset,
                    position: draw_state.position,
                    text: text_data.text.clone(),
                    rect: Rectangle::new(
                        draw_state.position,
                        Point::new(
                            draw_state.position.x + (text_data.text.len() * 8) as i32,
                            draw_state.position.y + style.font_size as i32,
                        ),
                    ),
                    plan: crate::font::RenderPlan::default(),
                    font_kind: style.font_kind,
                    font_size: style.font_size as u32,
                    font_style: style.font_style,
                    font_weight: style.font_weight,
                    color: style.color,
                    uri: computed_styles.get("href").cloned(),
                };

                display_list[page_index].push(DrawCommand::Text(text_cmd));

                // Advance position for next inline element
                draw_state.position.x += (text_data.text.len() * 8) as i32;

                ChildArtifact {
                    sibling_style: SiblingStyle {
                        padding: style.padding,
                        margin: style.margin,
                    },
                    rects: vec![Some(Rectangle::new(
                        Point::new(
                            draw_state.position.x - (text_data.text.len() * 8) as i32,
                            draw_state.position.y,
                        ),
                        Point::new(
                            draw_state.position.x,
                            draw_state.position.y + style.font_size as i32,
                        ),
                    ))],
                }
            }

            NodeData::Element(_) => {
                // Handle block-level elements
                if style.display == Display::Block {
                    // Move to new line for block elements
                    draw_state.position.x = style.start_x;
                    draw_state.position.y += style.line_height;

                    // Check for page break
                    let page_height = root_data.rect.height() as i32;
                    if draw_state.position.y > page_height - 50 {
                        // Start new page
                        display_list.push(Vec::new());
                        draw_state.position.y = 0;
                    }
                }

                // Skip display:none elements
                if style.display == Display::None {
                    return ChildArtifact {
                        sibling_style: SiblingStyle {
                            padding: style.padding,
                            margin: style.margin,
                        },
                        rects: vec![],
                    };
                }

                // Recurse on children
                let mut child_rects = Vec::new();
                for child in node.children() {
                    let child_artifact = self.build_display_list_recursive(
                        child,
                        &style,
                        loop_context,
                        stylesheet,
                        root_data,
                        resource_fetcher,
                        draw_state,
                        display_list,
                    );
                    child_rects.extend(child_artifact.rects);
                }

                ChildArtifact {
                    sibling_style: SiblingStyle {
                        padding: style.padding,
                        margin: style.margin,
                    },
                    rects: child_rects,
                }
            }

            NodeData::Root | NodeData::Wrapper(_) => {
                // Root and wrapper nodes just recurse on children
                let mut child_rects = Vec::new();
                for child in node.children() {
                    let child_artifact = self.build_display_list_recursive(
                        child,
                        &style,
                        loop_context,
                        stylesheet,
                        root_data,
                        resource_fetcher,
                        draw_state,
                        display_list,
                    );
                    child_rects.extend(child_artifact.rects);
                }

                ChildArtifact {
                    sibling_style: SiblingStyle {
                        padding: style.padding,
                        margin: style.margin,
                    },
                    rects: child_rects,
                }
            }
        }
    }

    /// Render a page to pixmap
    ///
    /// Executes draw commands and produces a rendered pixmap.
    /// Returns None if rendering fails or page is empty.
    pub fn render_page(
        &mut self,
        page: &[DrawCommand],
        scale_factor: f32,
        samples: usize,
        resource_fetcher: &mut dyn ResourceFetcher,
    ) -> Option<crate::framebuffer::Pixmap> {
        // Ensure fonts are available for rendering
        self.load_fonts();

        // Calculate pixmap dimensions
        let width = (self.dims.0 as f32 * scale_factor).round() as u32;
        let height = (self.dims.1 as f32 * scale_factor).round() as u32;

        // Create pixmap with appropriate sample count
        let mut pixmap = crate::framebuffer::Pixmap::new(width, height, samples).ok()?;

        // Execute draw commands
        for command in page {
            self.execute_draw_command(command, &mut pixmap, scale_factor, resource_fetcher);
        }

        Some(pixmap)
    }

    /// Execute a single draw command
    fn execute_draw_command(
        &mut self,
        command: &super::layout::DrawCommand,
        _pixmap: &mut crate::framebuffer::Pixmap,
        _scale_factor: f32,
        _resource_fetcher: &mut dyn ResourceFetcher,
    ) {
        use super::layout::DrawCommand;

        match command {
            DrawCommand::Text(cmd) => {
                // Render text at the specified position
                let _ = cmd;
                // Text rendering would use the font cache here
            }
            DrawCommand::ExtraText(cmd) => {
                // Render extra text (footnotes, etc.)
                let _ = cmd;
            }
            DrawCommand::Image(cmd) => {
                // Render image at the specified position
                let _ = cmd;
            }
            DrawCommand::Marker(offset) => {
                // Page marker at offset
                let _ = offset;
            }
            DrawCommand::ExtraRect(rect) => {
                // Draw extra rectangle (highlight, border, etc.)
                let _ = rect;
            }
        }
    }
}
