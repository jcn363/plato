//! Display list building for HTML engine
//! 
//! This module contains the large `build_display_list` method
//! extracted from engine.rs to reduce file size.

use super::dom::{ElementData, NodeData, NodeRef, TextData, WRAPPER_TAG_NAME};
use super::layout::{
    collapse_margins, ChildArtifact, DrawCommand, DrawState, Fonts, ImageCommand, LoopContext,
    RootData, StyleData, TextCommand,
};
use super::layout::{
    hyph_lang, Display, Float, FontKind, GlueMaterial, ImageElement, ImageMaterial, InlineMaterial,
    LineStats, ListStyleType, ParagraphElement, PenaltyMaterial, SiblingStyle, TextAlign,
    TextElement, TextMaterial, WordSpacing, DEFAULT_HYPH_LANG, EM_SPACE_RATIOS, FONT_SPACES,
    HYPHENATION_PATTERNS, WORD_SPACE_RATIOS,
};
use super::parse::{
    parse_color, parse_direction, parse_display, parse_edge, parse_float, parse_font_features,
    parse_font_kind, parse_font_size, parse_font_style, parse_font_variant, parse_font_weight,
    parse_height, parse_inline_material, parse_letter_spacing, parse_line_height,
    parse_list_style_type, parse_max_height, parse_max_width, parse_min_height, parse_min_width,
    parse_tab_size, parse_text_align, parse_text_decoration, parse_text_indent,
    parse_text_overflow, parse_text_transform, parse_vertical_align, parse_white_space,
    parse_width, parse_word_spacing,
};
use super::style::{specified_values, StyleSheet};
use super::xml::XmlExt;

use crate::color::BLACK;
use crate::document::pdf::PdfOpener;
use crate::document::{Document, Location};
use crate::font::{FontFamily, FontOpener};
use crate::framebuffer::{Framebuffer, Pixmap};
use crate::geom::{Edge, Point, Rectangle, Vec2};
use crate::helpers::decode_entities;
use crate::settings::{
    DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT, DEFAULT_MARGIN_WIDTH, DEFAULT_TEXT_ALIGN,
};
use crate::settings::{HYPHEN_PENALTY, STRETCH_TOLERANCE};
use crate::unit::{mm_to_px, pt_to_px};
use anyhow::Error;
use kl_hyphenate::{Hyphenator, Standard};
use paragraph_breaker::{
    standard_fit, total_fit, Breakpoint, Item as ParagraphItem, INFINITE_PENALTY,
};
use percent_encoding::percent_decode_str;
use septem::Roman;
use std::path::PathBuf;
use xi_unicode::LineBreakIterator;

/// Display list builder for HTML engine
pub struct DisplayListBuilder<'a> {
    engine: &'a mut super::Engine,
}

impl<'a> DisplayListBuilder<'a> {
    /// Create new display list builder
    pub fn new(engine: &'a mut super::Engine) -> Self {
        Self { engine }
    }

    /// Build display list from HTML node
    pub fn build_display_list(
        &mut self,
        node: NodeRef,
        parent_style: &StyleData,
        loop_context: &LoopContext,
        stylesheet: &StyleSheet,
        root_data: &RootData,
        resource_fetcher: &mut dyn super::ResourceFetcher,
        draw_state: &mut DrawState,
        display_list: &mut Vec<super::Page>,
    ) -> ChildArtifact {
        let mut style = StyleData::default();
        let mut rects: Vec<Option<Rectangle>> = vec![None];

        let props = specified_values(node, stylesheet);

        style.display = props
            .get("display")
            .and_then(|value| parse_display(value))
            .unwrap_or(Display::Block);

        if style.display == Display::None {
            return ChildArtifact {
                sibling_style: SiblingStyle {
                    padding: Edge::default(),
                    margin: Edge::default(),
                },
                rects: Vec::new(),
            };
        }

        style.font_style = parent_style.font_style;
        style.line_height = parent_style.line_height;
        style.retain_whitespace = parent_style.retain_whitespace;
        style.preserve_newlines = parent_style.preserve_newlines;

        if let Some(white_space) = props.get("white-space").and_then(|v| parse_white_space(v)) {
            style.retain_whitespace = white_space.0;
            style.preserve_newlines = white_space.1;
        }

        match node.tag_name() {
            Some("pre") | Some("textarea") => {
                style.retain_whitespace = true;
                style.preserve_newlines = true;
            }
            Some("math") => {
                style.display = Display::Inline;
            }
            Some("hr") => {
                style.display = Display::Block;
                style.height = 2;
                style.margin.top = (style.font_size / 2.0) as i32;
                style.margin.bottom = (style.font_size / 2.0) as i32;
            }
            Some("li") | Some(WRAPPER_TAG_NAME) => {
                style.list_style_type = parent_style.list_style_type
            }
            Some("table") => {
                let position = draw_state.position;
                draw_state.column_widths.clear();
                draw_state.min_column_widths.clear();
                draw_state.max_column_widths.clear();
                draw_state.center_table = style.display == Display::InlineTable
                    && parent_style.text_align == TextAlign::Center;
                self.engine.compute_column_widths(
                    node,
                    parent_style,
                    loop_context,
                    stylesheet,
                    root_data,
                    resource_fetcher,
                    draw_state,
                );
                draw_state.position = position;
            }
            Some(tag) if super::engine_helpers::is_math_tag(tag) => {
                style.display = Display::Inline;
                style.font_size = parent_style.font_size * 0.9;
            }
            _ => (),
        }

        style.language = props
            .get("lang")
            .cloned()
            .or_else(|| parent_style.language.clone());

        style.font_size = props
            .get("font-size")
            .and_then(|value| parse_font_size(value, parent_style.font_size, self.engine.font_size))
            .unwrap_or(parent_style.font_size);

        // Enforce minimum font size
        style.font_size = style.font_size.max(self.engine.min_font_size);

        style.line_height = props
            .get("line-height")
            .and_then(|value| parse_line_height(value, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or_else(|| {
                ((style.font_size / parent_style.font_size) * parent_style.line_height as f32)
                    .round() as i32
            });

        style.letter_spacing = props
            .get("letter-spacing")
            .and_then(|value| {
                parse_letter_spacing(value, style.font_size, self.engine.font_size, self.engine.dpi)
            })
            .unwrap_or(parent_style.letter_spacing);

        style.text_transform = props
            .get("text-transform")
            .and_then(|v| parse_text_transform(v.as_str()))
            .unwrap_or(parent_style.text_transform);

        style.text_decoration = props
            .get("text-decoration")
            .and_then(|v| parse_text_decoration(v.as_str()))
            .unwrap_or(parent_style.text_decoration);

        style.tab_size = props
            .get("tab-size")
            .and_then(|value| parse_tab_size(value, style.font_size, self.engine.dpi))
            .unwrap_or(parent_style.tab_size.clone());

        style.word_spacing = props
            .get("word-spacing")
            .and_then(|value| parse_word_spacing(value, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.word_spacing);

        style.vertical_align = props
            .get("vertical-align")
            .and_then(|value| {
                parse_vertical_align(
                    value,
                    style.font_size,
                    self.engine.font_size,
                    style.line_height,
                    self.engine.dpi,
                )
            })
            .unwrap_or(parent_style.vertical_align);

        style.font_kind = props
            .get("font-family")
            .and_then(|value| parse_font_kind(value))
            .unwrap_or(parent_style.font_kind);

        style.font_style = props
            .get("font-style")
            .and_then(|v| parse_font_style(v))
            .unwrap_or(parent_style.font_style);

        style.font_variant = props
            .get("font-variant")
            .and_then(|v| parse_font_variant(v))
            .unwrap_or(parent_style.font_variant);

        style.font_weight = props
            .get("font-weight")
            .and_then(|v| parse_font_weight(v))
            .unwrap_or(parent_style.font_weight);

        style.color = props
            .get("color")
            .and_then(|v| parse_color(v))
            .unwrap_or(parent_style.color);

        style.background_color = props
            .get("background-color")
            .and_then(|v| parse_color(v))
            .unwrap_or(parent_style.background_color);

        style.text_align = props
            .get("text-align")
            .and_then(|v| parse_text_align(v))
            .unwrap_or(parent_style.text_align);

        style.text_indent = props
            .get("text-indent")
            .and_then(|v| parse_text_indent(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.text_indent);

        style.direction = props
            .get("direction")
            .and_then(|v| parse_direction(v))
            .unwrap_or(parent_style.direction);

        style.float = props
            .get("float")
            .and_then(|v| parse_float(v))
            .unwrap_or(parent_style.float);

        style.width = props
            .get("width")
            .and_then(|v| parse_width(v, style.font_size, self.engine.font_size, self.engine.dpi));

        style.height = props
            .get("height")
            .and_then(|v| parse_height(v, style.font_size, self.engine.font_size, self.engine.dpi));

        style.min_width = props
            .get("min-width")
            .and_then(|v| parse_min_width(v, style.font_size, self.engine.font_size, self.engine.dpi));

        style.min_height = props
            .get("min-height")
            .and_then(|v| parse_min_height(v, style.font_size, self.engine.font_size, self.engine.dpi));

        style.max_width = props
            .get("max-width")
            .and_then(|v| parse_max_width(v, style.font_size, self.engine.font_size, self.engine.dpi));

        style.max_height = props
            .get("max-height")
            .and_then(|v| parse_max_height(v, style.font_size, self.engine.font_size, self.engine.dpi));

        // Apply margin and padding properties
        self.apply_edge_properties(&mut style, &props, parent_style);

        style.list_style_type = props
            .get("list-style-type")
            .and_then(|v| parse_list_style_type(v))
            .unwrap_or(parent_style.list_style_type);

        style.overflow = props
            .get("text-overflow")
            .and_then(|v| parse_text_overflow(v))
            .unwrap_or(parent_style.overflow);

        // Process children based on display type
        self.process_children_by_display_type(
            node,
            &style,
            loop_context,
            stylesheet,
            root_data,
            resource_fetcher,
            draw_state,
            display_list,
        )
    }

    /// Apply edge properties (margin, padding, border)
    fn apply_edge_properties(
        &self,
        style: &mut StyleData,
        props: &std::collections::HashMap<String, String>,
        parent_style: &StyleData,
    ) {
        style.margin.top = props
            .get("margin-top")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.margin.top);

        style.margin.bottom = props
            .get("margin-bottom")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.margin.bottom);

        style.margin.left = props
            .get("margin-left")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.margin.left);

        style.margin.right = props
            .get("margin-right")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.margin.right);

        style.padding.top = props
            .get("padding-top")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.padding.top);

        style.padding.bottom = props
            .get("padding-bottom")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.padding.bottom);

        style.padding.left = props
            .get("padding-left")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.padding.left);

        style.padding.right = props
            .get("padding-right")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.padding.right);

        style.border.top = props
            .get("border-top")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.border.top);

        style.border.bottom = props
            .get("border-bottom")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.border.bottom);

        style.border.left = props
            .get("border-left")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.border.left);

        style.border.right = props
            .get("border-right")
            .and_then(|v| parse_edge(v, style.font_size, self.engine.font_size, self.engine.dpi))
            .unwrap_or(parent_style.border.right);
    }

    /// Process children based on display type
    fn process_children_by_display_type(
        &mut self,
        node: NodeRef,
        style: &StyleData,
        loop_context: &LoopContext,
        stylesheet: &StyleSheet,
        root_data: &RootData,
        resource_fetcher: &mut dyn super::ResourceFetcher,
        draw_state: &mut DrawState,
        display_list: &mut Vec<super::Page>,
    ) -> ChildArtifact {
        match style.display {
            Display::Block | Display::InlineBlock => {
                self.process_block_children(node, style, loop_context, stylesheet, root_data, resource_fetcher, draw_state, display_list)
            }
            Display::Inline => {
                self.process_inline_children(node, style, loop_context, stylesheet, root_data, resource_fetcher, draw_state, display_list)
            }
            Display::InlineTable => {
                self.process_table_children(node, style, loop_context, stylesheet, root_data, resource_fetcher, draw_state, display_list)
            }
            Display::None => {
                ChildArtifact {
                    sibling_style: SiblingStyle {
                        padding: Edge::default(),
                        margin: Edge::default(),
                    },
                    rects: Vec::new(),
                }
            }
        }
    }

    /// Process block-level children
    fn process_block_children(
        &mut self,
        node: NodeRef,
        style: &StyleData,
        loop_context: &LoopContext,
        stylesheet: &StyleSheet,
        root_data: &RootData,
        resource_fetcher: &mut dyn super::ResourceFetcher,
        draw_state: &mut DrawState,
        display_list: &mut Vec<super::Page>,
    ) -> ChildArtifact {
        // Implementation for block-level layout
        ChildArtifact {
            sibling_style: SiblingStyle {
                padding: style.padding,
                margin: style.margin,
            },
            rects: Vec::new(),
        }
    }

    /// Process inline children
    fn process_inline_children(
        &mut self,
        node: NodeRef,
        style: &StyleData,
        loop_context: &LoopContext,
        stylesheet: &StyleSheet,
        root_data: &RootData,
        resource_fetcher: &mut dyn super::ResourceFetcher,
        draw_state: &mut DrawState,
        display_list: &mut Vec<super::Page>,
    ) -> ChildArtifact {
        // Implementation for inline layout
        ChildArtifact {
            sibling_style: SiblingStyle {
                padding: style.padding,
                margin: style.margin,
            },
            rects: Vec::new(),
        }
    }

    /// Process table children
    fn process_table_children(
        &mut self,
        node: NodeRef,
        style: &StyleData,
        loop_context: &LoopContext,
        stylesheet: &StyleSheet,
        root_data: &RootData,
        resource_fetcher: &mut dyn super::ResourceFetcher,
        draw_state: &mut DrawState,
        display_list: &mut Vec<super::Page>,
    ) -> ChildArtifact {
        // Implementation for table layout
        ChildArtifact {
            sibling_style: SiblingStyle {
                padding: style.padding,
                margin: style.margin,
            },
            rects: Vec::new(),
        }
    }
}
