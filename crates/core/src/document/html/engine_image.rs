use super::engine::{Engine, ResourceFetcher};
use super::layout::{
    Float, ImageElement, InlineMaterial, ParagraphElement, ParagraphItem, StyleData,
};
use super::parse::{
    parse_display, parse_float, parse_font_kind, parse_font_size, parse_font_style,
    parse_font_weight, parse_height, parse_width,
};
use super::style::specified_values;
use crate::document::pdf::PdfOpener;
use crate::unit::pt_to_px;
use anyhow::Error;
use percent_encoding::percent_decode_str;
use std::path::PathBuf;

impl Engine {
    pub(super) fn process_image_element(
        &self,
        node: super::dom::NodeRef,
        stylesheet: &super::style::StyleSheet,
        parent_style: &StyleData,
        spine_dir: &PathBuf,
        inlines: &mut Vec<InlineMaterial>,
        markers: &mut Vec<usize>,
    ) {
        if let super::dom::NodeData::Element(ref element_data) = node.data() {
            let offset = element_data.offset;
            let name = &element_data.name;
            let attributes = &element_data.attributes;
            let props = specified_values(node, stylesheet);

            match name.as_ref() {
                "img" | "image" => {
                    let attr = if name == "img" { "src" } else { "xlink:href" };
                    let path = attributes
                        .get(attr)
                        .and_then(|src| {
                            spine_dir
                                .join(src)
                                .canonicalize()
                                .unwrap_or_else(|_| spine_dir.join(src))
                                .to_str()
                                .map(|uri| {
                                    percent_decode_str(&crate::helpers::decode_entities(uri))
                                        .decode_utf8_lossy()
                                        .into_owned()
                                })
                        })
                        .unwrap_or_default();

                    let mut style = StyleData::default();
                    style.font_style = parent_style.font_style;
                    style.line_height = parent_style.line_height;
                    style.text_indent = parent_style.text_indent;
                    style.retain_whitespace = parent_style.retain_whitespace;
                    style.language = parent_style.language.clone();
                    style.uri = parent_style.uri.clone();

                    style.display = props
                        .get("display")
                        .and_then(|value| parse_display(value))
                        .unwrap_or(super::layout::Display::Inline);

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
                        .and_then(|value| super::parse::parse_color(value))
                        .unwrap_or(parent_style.color);

                    style.float = props.get("float").and_then(|value| parse_float(value));

                    let is_block = style.display == super::layout::Display::Block;
                    if is_block || style.float.is_some() {
                        style.margin = super::parse::parse_edge(
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

                    if node.id().is_some() {
                        markers.push(node.offset());
                    }

                    if is_block {
                        inlines.push(InlineMaterial::LineBreak);
                    }
                    inlines.push(InlineMaterial::Image(super::layout::ImageMaterial {
                        offset: *offset,
                        path,
                        style,
                    }));
                    if is_block {
                        inlines.push(InlineMaterial::LineBreak);
                    }
                }
                "a" => {
                    if let Some(uri) = attributes.get("href") {
                        // This would be handled in the main gather_inline_material function
                        // as it needs to modify the style for subsequent elements
                    }
                }
                _ => {}
            }
        }
    }

    pub(super) fn create_image_element(
        &mut self,
        offset: usize,
        path: &str,
        style: &StyleData,
        resource_fetcher: &mut dyn ResourceFetcher,
    ) -> Option<ImageElement> {
        let (mut width, mut height) = (style.width, style.height);
        let mut scale = 1.0;
        let dpi = self.dpi;

        if let Ok(buf) = resource_fetcher.fetch(path) {
            if let Some(doc) = PdfOpener::new().and_then(|opener| opener.open_memory(path, &buf)) {
                if let Some((w, h)) = doc.dims(0) {
                    if width == 0 && height == 0 {
                        width = pt_to_px(w, dpi).round() as i32;
                        height = pt_to_px(h, dpi).round() as i32;
                    } else if width != 0 {
                        height = (width as f32 * h / w).round() as i32;
                    } else if height != 0 {
                        width = (height as f32 * w / h).round() as i32;
                    }
                    scale = width as f32 / w;
                }
            }

            if width * height > 0 {
                Some(ImageElement {
                    offset,
                    width,
                    height,
                    scale,
                    vertical_align: style.vertical_align,
                    display: style.display,
                    margin: style.margin,
                    float: style.float,
                    path: path.to_string(),
                    uri: style.uri.clone(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(super) fn add_image_to_paragraph_items(
        &mut self,
        element: ImageElement,
        items: &mut Vec<ParagraphItem<ParagraphElement>>,
        floats: &mut Vec<ImageElement>,
    ) {
        if element.float.is_none() {
            items.push(ParagraphItem::Box {
                width: element.width,
                data: ParagraphElement::Image(element),
            });
        } else {
            floats.push(element);
        }
    }
}
