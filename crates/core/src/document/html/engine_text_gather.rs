use super::engine::{Engine, ResourceFetcher};
use super::layout::{InlineMaterial, StyleData};
use super::parse::{
    parse_color, parse_display, parse_edge, parse_float, parse_font_features, parse_font_kind,
    parse_font_size, parse_font_style, parse_font_variant, parse_font_weight, parse_height,
    parse_inline_material, parse_letter_spacing, parse_line_height, parse_max_height,
    parse_max_width, parse_min_height, parse_min_width, parse_text_decoration,
    parse_text_indent, parse_text_transform, parse_vertical_align, parse_width,
    parse_word_spacing,
};
use super::style::specified_values;
use crate::helpers::decode_entities;
use percent_encoding::percent_decode_str;
use std::path::PathBuf;

impl Engine {
    pub(super) fn gather_inline_material(
        &self,
        node: super::dom::NodeRef,
        stylesheet: &super::style::StyleSheet,
        parent_style: &StyleData,
        spine_dir: &PathBuf,
        markers: &mut Vec<usize>,
        inlines: &mut Vec<InlineMaterial>,
    ) {
        match node.data() {
            super::dom::NodeData::Element(super::dom::ElementData {
                offset,
                name,
                attributes,
                ..
            }) => {
                let mut style = StyleData::default();
                let props = specified_values(node, stylesheet);

                self.inherit_parent_styles(&mut style, parent_style);
                
                style.display = props
                    .get("display")
                    .and_then(|value| parse_display(value))
                    .unwrap_or(super::layout::Display::Inline);

                if style.display == super::layout::Display::None {
                    return;
                }

                self.parse_element_styles(&mut style, &props, parent_style);
                self.handle_element_specifics(
                    &name,
                    &attributes,
                    &props,
                    &mut style,
                    parent_style,
                    spine_dir,
                    *offset,
                    markers,
                    inlines,
                );

                if node.id().is_some() {
                    markers.push(node.offset());
                }

                self.process_before_insert(&props, &style, inlines);
                
                for child in node.children() {
                    self.gather_inline_material(
                        child, stylesheet, &style, spine_dir, markers, inlines,
                    );
                }

                self.process_after_insert(&props, &style, inlines);
            }
            super::dom::NodeData::Text(super::dom::TextData { offset, text }) => {
                inlines.push(InlineMaterial::Text(super::layout::TextMaterial {
                    offset: *offset,
                    text: decode_entities(text).into_owned(),
                    style: parent_style.clone(),
                }));
            }
            super::dom::NodeData::Whitespace(super::dom::TextData { offset, text }) => {
                inlines.push(InlineMaterial::Text(super::layout::TextMaterial {
                    offset: *offset,
                    text: text.to_string(),
                    style: parent_style.clone(),
                }));
            }
            _ => (),
        }
    }

    fn inherit_parent_styles(&self, style: &mut StyleData, parent_style: &StyleData) {
        style.font_style = parent_style.font_style;
        style.line_height = parent_style.line_height;
        style.text_indent = parent_style.text_indent;
        style.retain_whitespace = parent_style.retain_whitespace;
        style.language = parent_style.language.clone();
        style.uri = parent_style.uri.clone();
    }

    fn parse_element_styles(
        &self,
        style: &mut StyleData,
        props: &std::collections::HashMap<String, String>,
        parent_style: &StyleData,
    ) {
        style.font_size = props
            .get("font-size")
            .and_then(|value| {
                parse_font_size(value, parent_style.font_size, self.font_size)
            })
            .unwrap_or(parent_style.font_size);

        style.width = props
            .get("width")
            .and_then(|value| {
                parse_width(
                    value,
                    style.font_size,
                    self.font_size,
                    parent_style.width,
                    self.dpi,
                )
            })
            .unwrap_or(0);

        style.height = props
            .get("height")
            .and_then(|value| {
                parse_height(
                    value,
                    style.font_size,
                    self.font_size,
                    parent_style.width,
                    self.dpi,
                )
            })
            .unwrap_or(0);

        style.font_kind = props
            .get("font-family")
            .and_then(|value| parse_font_kind(value))
            .unwrap_or(parent_style.font_kind);

        style.color = props
            .get("color")
            .and_then(|value| parse_color(value))
            .unwrap_or(parent_style.color);

        style.letter_spacing = props
            .get("letter-spacing")
            .and_then(|value| {
                parse_letter_spacing(value, style.font_size, self.font_size, self.dpi)
            })
            .unwrap_or(parent_style.letter_spacing);

        style.word_spacing = props
            .get("word-spacing")
            .and_then(|value| {
                parse_word_spacing(value, style.font_size, self.font_size, self.dpi)
            })
            .unwrap_or(parent_style.word_spacing);

        style.vertical_align = props
            .get("vertical-align")
            .and_then(|value| {
                parse_vertical_align(
                    value,
                    style.font_size,
                    self.font_size,
                    style.line_height,
                    self.dpi,
                )
            })
            .unwrap_or(parent_style.vertical_align);

        style.font_style = props
            .get("font-style")
            .and_then(|value| parse_font_style(value))
            .unwrap_or(parent_style.font_style);

        style.font_weight = props
            .get("font-weight")
            .and_then(|value| parse_font_weight(value))
            .unwrap_or(parent_style.font_weight);

        style.font_features = props
            .get("font-feature-settings")
            .map(|value| parse_font_features(value))
            .or_else(|| parent_style.font_features.clone());

        if let Some(value) = props.get("font-variant") {
            let mut features = parse_font_variant(value);
            if let Some(v) = style.font_features.as_mut() {
                v.append(&mut features);
            }
        }
    }

    fn handle_element_specifics(
        &self,
        name: &str,
        attributes: &std::collections::HashMap<String, String>,
        props: &std::collections::HashMap<String, String>,
        style: &mut StyleData,
        parent_style: &StyleData,
        spine_dir: &PathBuf,
        offset: usize,
        markers: &mut Vec<usize>,
        inlines: &mut Vec<InlineMaterial>,
    ) {
        match name {
            "img" | "image" => {
                self.handle_image_element(
                    name,
                    attributes,
                    props,
                    style,
                    parent_style,
                    spine_dir,
                    offset,
                    inlines,
                );
            }
            "a" => {
                self.handle_link_element(attributes, style);
            }
            "br" => {
                inlines.push(InlineMaterial::LineBreak);
            }
            _ => {}
        }
    }

    fn handle_image_element(
        &self,
        name: &str,
        attributes: &std::collections::HashMap<String, String>,
        props: &std::collections::HashMap<String, String>,
        style: &mut StyleData,
        parent_style: &StyleData,
        spine_dir: &PathBuf,
        offset: usize,
        inlines: &mut Vec<InlineMaterial>,
    ) {
        let attr = if name == "img" { "src" } else { "xlink:href" };

        let path = attributes
            .get(attr)
            .and_then(|src| {
                spine_dir.join(src).canonicalize().unwrap_or_else(|_| spine_dir.join(src)).to_str().map(|uri| {
                    percent_decode_str(&decode_entities(uri))
                        .decode_utf8_lossy()
                        .into_owned()
                })
            })
            .unwrap_or_default();

        style.float = props.get("float").and_then(|value| parse_float(value));

        let is_block = style.display == super::layout::Display::Block;
        if is_block || style.float.is_some() {
            style.margin = parse_edge(
                props.get("margin-top").map(String::as_str),
                props.get("margin-right").map(String::as_str),
                props.get("margin-bottom").map(String::as_str),
                props.get("margin-left").map(String::as_str),
                style.font_size,
                self.font_size,
                parent_style.width,
                self.dpi,
            );
        }
        if is_block {
            inlines.push(InlineMaterial::LineBreak);
        }
        inlines.push(InlineMaterial::Image(super::layout::ImageMaterial {
            offset,
            path,
            style: style.clone(),
        }));
        if is_block {
            inlines.push(InlineMaterial::LineBreak);
        }
    }

    fn handle_link_element(
        &self,
        attributes: &std::collections::HashMap<String, String>,
        style: &mut StyleData,
    ) {
        if let Some(uri) = attributes.get("href") {
            style.uri = Some(
                percent_decode_str(&decode_entities(uri))
                    .decode_utf8_lossy()
                    .into_owned(),
            );
        }
    }

    fn process_before_insert(
        &self,
        props: &std::collections::HashMap<String, String>,
        style: &StyleData,
        inlines: &mut Vec<InlineMaterial>,
    ) {
        if let Some(mut v) = props.get("-plato-insert-before").map(|value| {
            parse_inline_material(value, style.font_size, self.font_size, self.dpi)
        }) {
            inlines.append(&mut v);
        }
    }

    fn process_after_insert(
        &self,
        props: &std::collections::HashMap<String, String>,
        style: &StyleData,
        inlines: &mut Vec<InlineMaterial>,
    ) {
        if let Some(mut v) = props.get("-plato-insert-after").map(|value| {
            parse_inline_material(value, style.font_size, self.font_size, self.dpi)
        }) {
            inlines.append(&mut v);
        }
    }
}
