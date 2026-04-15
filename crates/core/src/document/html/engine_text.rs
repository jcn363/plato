use super::engine::{Engine, ResourceFetcher};
use super::layout::{
    collapse_margins, hyph_lang, DEFAULT_HYPH_LANG, HYPHENATION_PATTERNS, ChildArtifact,
    DrawCommand, DrawState, FontKind, Float, GlueMaterial, ImageElement, InlineMaterial,
    LineStats, ListStyleType, LoopContext, ParagraphElement, PenaltyMaterial, RootData,
    StyleData, TextElement, TextAlign, WordSpacing, EM_SPACE_RATIOS, FONT_SPACES,
    WORD_SPACE_RATIOS,
};
use super::parse::{
    parse_color, parse_display, parse_edge, parse_float, parse_font_features, parse_font_kind,
    parse_font_size, parse_font_style, parse_font_variant, parse_font_weight, parse_height,
    parse_inline_material, parse_letter_spacing, parse_line_height, parse_max_height,
    parse_max_width, parse_min_height, parse_min_width, parse_text_align, parse_text_decoration,
    parse_text_indent, parse_text_transform, parse_vertical_align, parse_width,
    parse_word_spacing,
};
use super::style::specified_values;
use super::xml;
use super::xml::XmlExt;
use super::style::{specified_values, StyleSheet};
use crate::color::BLACK;
use crate::document::pdf::PdfOpener;
use crate::geom::{Point, Rectangle};
use crate::helpers::decode_entities;
use crate::settings::HYPHEN_PENALTY;
use crate::unit::pt_to_px;
use anyhow::Error;
use kl_hyphenate::{Hyphenator, Standard};
use paragraph_breaker::{standard_fit, total_fit};
use paragraph_breaker::{Breakpoint, Item as ParagraphItem, INFINITE_PENALTY};
use percent_encoding::percent_decode_str;
use septem::Roman;
use std::convert::TryFrom;
use std::path::PathBuf;
use xi_unicode::LineBreakIterator;

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

                if style.display == super::layout::Display::None {
                    return;
                }

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

                if node.id().is_some() {
                    markers.push(node.offset());
                }

                match name.as_ref() {
                    "img" | "image" => {
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
                            offset: *offset,
                            path,
                            style,
                        }));
                        if is_block {
                            inlines.push(InlineMaterial::LineBreak);
                        }
                        return;
                    }
                    "a" => {
                        style.uri = attributes.get("href").map(|uri| {
                            percent_decode_str(&decode_entities(uri))
                                .decode_utf8_lossy()
                                .into_owned()
                        });
                    }
                    "br" => {
                        inlines.push(InlineMaterial::LineBreak);
                        return;
                    }
                    _ => {}
                }

                if let Some(mut v) = props.get("-plato-insert-before").map(|value| {
                    parse_inline_material(value, style.font_size, self.font_size, self.dpi)
                }) {
                    inlines.append(&mut v);
                }

                for child in node.children() {
                    self.gather_inline_material(
                        child, stylesheet, &style, spine_dir, markers, inlines,
                    );
                }

                if let Some(mut v) = props.get("-plato-insert-after").map(|value| {
                    parse_inline_material(value, style.font_size, self.font_size, self.dpi)
                }) {
                    inlines.append(&mut v);
                }
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

    pub(super) fn make_paragraph_items(
        &mut self,
        inlines: &[InlineMaterial],
        parent_style: &StyleData,
        line_width: i32,
        resource_fetcher: &mut dyn ResourceFetcher,
    ) -> (Vec<ParagraphItem<ParagraphElement>>, Vec<ImageElement>) {
        let mut items = Vec::with_capacity(inlines.len());
        let mut floats = Vec::new();
        let big_stretch = 3 * {
            let font_size = (parent_style.font_size * 64.0) as u32;
            let font = self.fonts.as_mut().expect("fonts not initialized").get_mut(
                parent_style.font_kind,
                parent_style.font_style,
                parent_style.font_weight,
            );
            font.set_size(font_size, self.dpi);
            font.plan(" ", None, None).width
        };

        if parent_style.text_align == TextAlign::Center {
            items.push(ParagraphItem::Box {
                width: 0,
                data: ParagraphElement::Nothing,
            });
            items.push(ParagraphItem::Glue {
                width: 0,
                stretch: big_stretch,
                shrink: 0,
            });
        }

        for (index, mater) in inlines.iter().enumerate() {
            match mater {
                InlineMaterial::Image(super::layout::ImageMaterial {
                    offset,
                    path,
                    style,
                }) => {
                    let (mut width, mut height) = (style.width, style.height);
                    let mut scale = 1.0;
                    let dpi = self.dpi;

                    if let Ok(buf) = resource_fetcher.fetch(path) {
                        if let Some(doc) =
                            PdfOpener::new().and_then(|opener| opener.open_memory(path, &buf))
                        {
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
                            let element = ImageElement {
                                offset: *offset,
                                width,
                                height,
                                scale,
                                vertical_align: style.vertical_align,
                                display: style.display,
                                margin: style.margin,
                                float: style.float,
                                path: path.clone(),
                                uri: style.uri.clone(),
                            };
                            if style.float.is_none() {
                                items.push(ParagraphItem::Box {
                                    width,
                                    data: ParagraphElement::Image(element),
                                });
                            } else {
                                floats.push(element);
                            }
                        }
                    }
                }
                InlineMaterial::Text(super::layout::TextMaterial {
                    offset,
                    text,
                    style,
                }) => {
                    let font_size = (style.font_size * 64.0) as u32;
                    let space_plan = {
                        let font = self.fonts.as_mut().expect("fonts not initialized").get_mut(
                            parent_style.font_kind,
                            parent_style.font_style,
                            parent_style.font_weight,
                        );
                        font.set_size(font_size, self.dpi);
                        font.plan(" 0.", None, None)
                    };
                    let mut start_index = 0;
                    for (end_index, _is_hardbreak) in LineBreakIterator::new(text) {
                        for chunk in
                            text[start_index..end_index].split_inclusive(char::is_whitespace)
                        {
                            if let Some((i, c)) = chunk.char_indices().next_back() {
                                let j = i + if c.is_whitespace() { 0 } else { c.len_utf8() };
                                if j > 0 {
                                    let buf = &text[start_index..start_index + j];
                                    let local_offset = offset + start_index;
                                    let mut plan = {
                                        let font = self
                                            .fonts
                                            .as_mut()
                                            .expect("fonts not initialized")
                                            .get_mut(
                                                style.font_kind,
                                                style.font_style,
                                                style.font_weight,
                                            );
                                        font.set_size(font_size, self.dpi);
                                        font.plan(buf, None, style.font_features.as_deref())
                                    };
                                    plan.space_out(style.letter_spacing);

                                    items.push(ParagraphItem::Box {
                                        width: plan.width,
                                        data: ParagraphElement::Text(TextElement {
                                            offset: local_offset,
                                            language: style.language.clone(),
                                            text: buf.to_string(),
                                            plan,
                                            font_features: style.font_features.clone(),
                                            font_kind: style.font_kind,
                                            font_style: style.font_style,
                                            font_weight: style.font_weight,
                                            vertical_align: style.vertical_align,
                                            letter_spacing: style.letter_spacing,
                                            font_size,
                                            color: style.color,
                                            uri: style.uri.clone(),
                                        }),
                                    });
                                }
                                if c.is_whitespace() {
                                    if c == '\n' && parent_style.retain_whitespace {
                                        let stretch =
                                            if parent_style.text_align == TextAlign::Center {
                                                big_stretch
                                            } else {
                                                line_width
                                            };

                                        items.push(ParagraphItem::Penalty {
                                            penalty: INFINITE_PENALTY,
                                            width: 0,
                                            flagged: false,
                                        });
                                        items.push(ParagraphItem::Glue {
                                            width: 0,
                                            stretch,
                                            shrink: 0,
                                        });

                                        items.push(ParagraphItem::Penalty {
                                            width: 0,
                                            penalty: -INFINITE_PENALTY,
                                            flagged: false,
                                        });

                                        if parent_style.text_align == TextAlign::Center {
                                            items.push(ParagraphItem::Box {
                                                width: 0,
                                                data: ParagraphElement::Nothing,
                                            });
                                            items.push(ParagraphItem::Penalty {
                                                width: 0,
                                                penalty: INFINITE_PENALTY,
                                                flagged: false,
                                            });
                                            items.push(ParagraphItem::Glue {
                                                width: 0,
                                                stretch: big_stretch,
                                                shrink: 0,
                                            });
                                        }
                                        start_index += chunk.len();
                                        continue;
                                    }

                                    let last_c =
                                        text[..start_index + i].chars().next_back().or_else(|| {
                                            if index > 0 {
                                                inlines[index - 1]
                                                    .text()
                                                    .and_then(|text| text.chars().next_back())
                                            } else {
                                                None
                                            }
                                        });

                                    let has_more = text[start_index + i..]
                                        .chars()
                                        .any(|c| !c.is_xml_whitespace())
                                        || inlines[index + 1..].iter().any(|m| {
                                            m.text().map_or(false, |text| {
                                                text.chars().any(|c| !c.is_xml_whitespace())
                                            })
                                        });

                                    if !parent_style.retain_whitespace
                                        && c.is_xml_whitespace()
                                        && (last_c.map(|c| c.is_xml_whitespace()) != Some(false)
                                            || !has_more)
                                    {
                                        start_index += chunk.len();
                                        continue;
                                    }

                                    let mut width = if !parent_style.retain_whitespace {
                                        space_plan.glyph_advance(0)
                                    } else if let Some(index) =
                                        FONT_SPACES.chars().position(|x| x == c)
                                    {
                                        space_plan.glyph_advance(index)
                                    } else if let Some(ratio) = WORD_SPACE_RATIOS.get(&c) {
                                        (space_plan.glyph_advance(0) as f32 * ratio) as i32
                                    } else if let Some(ratio) = EM_SPACE_RATIOS.get(&c) {
                                        pt_to_px(style.font_size * ratio, self.dpi).round() as i32
                                    } else {
                                        space_plan.glyph_advance(0)
                                    };

                                    width += match style.word_spacing {
                                        WordSpacing::Normal => 0,
                                        WordSpacing::Length(l) => l,
                                        WordSpacing::Ratio(r) => (r * width as f32) as i32,
                                    } + style.letter_spacing;

                                    let is_unbreakable =
                                        c == '\u{00A0}' || c == '\u{202F}' || c == '\u{2007}';

                                    if (is_unbreakable
                                        || (parent_style.retain_whitespace
                                            && c.is_xml_whitespace()))
                                        && (last_c == Some('\n') || last_c.is_none())
                                    {
                                        items.push(ParagraphItem::Box {
                                            width: 0,
                                            data: ParagraphElement::Nothing,
                                        });
                                    }

                                    if is_unbreakable {
                                        items.push(ParagraphItem::Penalty {
                                            width: 0,
                                            penalty: INFINITE_PENALTY,
                                            flagged: false,
                                        });
                                    }

                                    match parent_style.text_align {
                                        TextAlign::Justify => {
                                            items.push(ParagraphItem::Glue {
                                                width,
                                                stretch: width / 2,
                                                shrink: width / 3,
                                            });
                                        }
                                        TextAlign::Center => {
                                            if style.font_kind == FontKind::Monospace
                                                || is_unbreakable
                                            {
                                                items.push(ParagraphItem::Glue {
                                                    width,
                                                    stretch: 0,
                                                    shrink: 0,
                                                });
                                            } else {
                                                let stretch = 3 * width;
                                                items.push(ParagraphItem::Glue {
                                                    width: 0,
                                                    stretch,
                                                    shrink: 0,
                                                });
                                                items.push(ParagraphItem::Penalty {
                                                    width: 0,
                                                    penalty: 0,
                                                    flagged: false,
                                                });
                                                items.push(ParagraphItem::Glue {
                                                    width,
                                                    stretch: -2 * stretch,
                                                    shrink: 0,
                                                });
                                                items.push(ParagraphItem::Box {
                                                    width: 0,
                                                    data: ParagraphElement::Nothing,
                                                });
                                                items.push(ParagraphItem::Penalty {
                                                    width: 0,
                                                    penalty: INFINITE_PENALTY,
                                                    flagged: false,
                                                });
                                                items.push(ParagraphItem::Glue {
                                                    width: 0,
                                                    stretch,
                                                    shrink: 0,
                                                });
                                            }
                                        }
                                        TextAlign::Left | TextAlign::Right => {
                                            if style.font_kind == FontKind::Monospace
                                                || is_unbreakable
                                            {
                                                items.push(ParagraphItem::Glue {
                                                    width,
                                                    stretch: 0,
                                                    shrink: 0,
                                                });
                                            } else {
                                                let stretch = 3 * width;
                                                items.push(ParagraphItem::Glue {
                                                    width: 0,
                                                    stretch,
                                                    shrink: 0,
                                                });
                                                items.push(ParagraphItem::Penalty {
                                                    width: 0,
                                                    penalty: 0,
                                                    flagged: false,
                                                });
                                                items.push(ParagraphItem::Glue {
                                                    width,
                                                    stretch: -stretch,
                                                    shrink: 0,
                                                });
                                            }
                                        }
                                    }
                                } else if end_index < text.len() {
                                    let penalty = if c == '-' { self.hyphen_penalty } else { 0 };
                                    let flagged = penalty > 0;
                                    if matches!(
                                        parent_style.text_align,
                                        TextAlign::Justify | TextAlign::Center
                                    ) {
                                        items.push(ParagraphItem::Penalty {
                                            width: 0,
                                            penalty,
                                            flagged,
                                        });
                                    } else {
                                        let stretch = 3 * space_plan.glyph_advance(0);
                                        items.push(ParagraphItem::Penalty {
                                            width: 0,
                                            penalty: INFINITE_PENALTY,
                                            flagged: false,
                                        });
                                        items.push(ParagraphItem::Glue {
                                            width: 0,
                                            stretch,
                                            shrink: 0,
                                        });
                                        items.push(ParagraphItem::Penalty {
                                            width: 0,
                                            penalty: 10 * penalty,
                                            flagged: true,
                                        });
                                        items.push(ParagraphItem::Glue {
                                            width: 0,
                                            stretch: -stretch,
                                            shrink: 0,
                                        });
                                    }
                                }
                            }
                            start_index += chunk.len();
                        }
                    }
                }
                InlineMaterial::LineBreak => {
                    let stretch = if parent_style.text_align == TextAlign::Center {
                        big_stretch
                    } else {
                        line_width
                    };

                    items.push(ParagraphItem::Penalty {
                        penalty: INFINITE_PENALTY,
                        width: 0,
                        flagged: false,
                    });
                    items.push(ParagraphItem::Glue {
                        width: 0,
                        stretch,
                        shrink: 0,
                    });

                    items.push(ParagraphItem::Penalty {
                        width: 0,
                        penalty: -INFINITE_PENALTY,
                        flagged: false,
                    });

                    if parent_style.text_align == TextAlign::Center {
                        items.push(ParagraphItem::Box {
                            width: 0,
                            data: ParagraphElement::Nothing,
                        });
                        items.push(ParagraphItem::Penalty {
                            width: 0,
                            penalty: INFINITE_PENALTY,
                            flagged: false,
                        });
                        items.push(ParagraphItem::Glue {
                            width: 0,
                            stretch: big_stretch,
                            shrink: 0,
                        });
                    }
                }
                InlineMaterial::Glue(GlueMaterial {
                    width,
                    stretch,
                    shrink,
                }) => {
                    items.push(ParagraphItem::Glue {
                        width: *width,
                        stretch: *stretch,
                        shrink: *shrink,
                    });
                }
                InlineMaterial::Penalty(PenaltyMaterial {
                    width,
                    penalty,
                    flagged,
                }) => {
                    items.push(ParagraphItem::Penalty {
                        width: *width,
                        penalty: *penalty,
                        flagged: *flagged,
                    });
                }
                InlineMaterial::Box(width) => {
                    items.push(ParagraphItem::Box {
                        width: *width,
                        data: ParagraphElement::Nothing,
                    });
                }
            }
        }

        if !items.is_empty() && items.last().map(ParagraphItem::penalty) != Some(-INFINITE_PENALTY)
        {
            items.push(ParagraphItem::Penalty {
                penalty: INFINITE_PENALTY,
                width: 0,
                flagged: false,
            });

            let stretch = if parent_style.text_align == TextAlign::Center {
                big_stretch
            } else {
                line_width
            };
            items.push(ParagraphItem::Glue {
                width: 0,
                stretch,
                shrink: 0,
            });

            items.push(ParagraphItem::Penalty {
                penalty: -INFINITE_PENALTY,
                width: 0,
                flagged: true,
            });
        }

        (items, floats)
    }

    #[inline]
    pub(super) fn box_from_chunk(
        &mut self,
        chunk: &str,
        index: usize,
        element: &TextElement,
    ) -> ParagraphItem<ParagraphElement> {
        let offset = element.offset + index;
        let mut plan = {
            let font = self.fonts.as_mut().expect("fonts not initialized").get_mut(
                element.font_kind,
                element.font_style,
                element.font_weight,
            );
            font.set_size(element.font_size, self.dpi);
            font.plan(chunk, None, element.font_features.as_deref())
        };
        plan.space_out(element.letter_spacing);
        ParagraphItem::Box {
            width: plan.width,
            data: ParagraphElement::Text(TextElement {
                offset,
                text: chunk.to_string(),
                plan,
                language: element.language.clone(),
                font_features: element.font_features.clone(),
                font_kind: element.font_kind,
                font_style: element.font_style,
                font_weight: element.font_weight,
                font_size: element.font_size,
                vertical_align: element.vertical_align,
                letter_spacing: element.letter_spacing,
                color: element.color,
                uri: element.uri.clone(),
            }),
        }
    }

    pub(super) fn hyphenate_paragraph(
        &mut self,
        style: &StyleData,
        dictionary: &Standard,
        items: Vec<ParagraphItem<ParagraphElement>>,
        hyph_indices: &mut Vec<[usize; 2]>,
    ) -> Vec<ParagraphItem<ParagraphElement>> {
        let mut hyph_items = Vec::with_capacity(items.len());

        for itm in items {
            match itm {
                ParagraphItem::Box {
                    data: ParagraphElement::Text(ref element),
                    ..
                } => {
                    let text = &element.text;
                    let (hyphen_width, stretch) = {
                        let font = self.fonts.as_mut().expect("fonts not initialized").get_mut(
                            element.font_kind,
                            element.font_style,
                            element.font_weight,
                        );
                        font.set_size(element.font_size, self.dpi);
                        let plan = font.plan(" -", None, element.font_features.as_deref());
                        (plan.glyph_advance(1), 3 * plan.glyph_advance(0))
                    };

                    let mut index_before =
                        text.find(char::is_alphabetic).unwrap_or_else(|| text.len());
                    if index_before > 0 {
                        let subelem = self.box_from_chunk(&text[0..index_before], 0, element);
                        hyph_items.push(subelem);
                    }

                    let mut index_after = text[index_before..]
                        .find(|c: char| !c.is_alphabetic())
                        .map(|i| index_before + i)
                        .unwrap_or_else(|| text.len());
                    while index_before < index_after {
                        let mut index = 0;
                        let chunk = &text[index_before..index_after];
                        let len_before = hyph_items.len();

                        for segment in dictionary.hyphenate(chunk) {
                            let subelem =
                                self.box_from_chunk(segment, index_before + index, element);
                            hyph_items.push(subelem);
                            index += segment.len();
                            if index < chunk.len() {
                                if style.text_align == TextAlign::Justify {
                                    hyph_items.push(ParagraphItem::Penalty {
                                        width: hyphen_width,
                                        penalty: self.hyphen_penalty,
                                        flagged: true,
                                    });
                                } else {
                                    hyph_items.push(ParagraphItem::Penalty {
                                        width: 0,
                                        penalty: INFINITE_PENALTY,
                                        flagged: false,
                                    });
                                    hyph_items.push(ParagraphItem::Glue {
                                        width: 0,
                                        stretch,
                                        shrink: 0,
                                    });
                                    hyph_items.push(ParagraphItem::Penalty {
                                        width: hyphen_width,
                                        penalty: 10 * self.hyphen_penalty,
                                        flagged: true,
                                    });
                                    hyph_items.push(ParagraphItem::Glue {
                                        width: 0,
                                        stretch: -stretch,
                                        shrink: 0,
                                    });
                                }
                            }
                        }

                        let len_after = hyph_items.len();
                        if len_after > 1 + len_before {
                            hyph_indices.push([len_before, len_after]);
                        }
                        index_before = text[index_after..]
                            .find(char::is_alphabetic)
                            .map(|i| index_after + i)
                            .unwrap_or_else(|| text.len());
                        if index_before > index_after {
                            let subelem = self.box_from_chunk(
                                &text[index_after..index_before],
                                index_after,
                                &element,
                            );
                            hyph_items.push(subelem);
                        }

                        index_after = text[index_before..]
                            .find(|c: char| !c.is_alphabetic())
                            .map(|i| index_before + i)
                            .unwrap_or_else(|| text.len());
                    }
                }
                _ => hyph_items.push(itm),
            }
        }

        hyph_items
    }

    pub(super) fn cleanup_paragraph(
        &mut self,
        items: Vec<ParagraphItem<ParagraphElement>>,
        hyph_indices: &[[usize; 2]],
        glue_drifts: &mut Vec<f32>,
        bps: &mut Vec<Breakpoint>,
    ) -> Vec<ParagraphItem<ParagraphElement>> {
        let mut merged_items = Vec::with_capacity(items.len());
        let mut j = 0;
        let mut k = 0;
        let mut index_drift = 0;
        let [mut start_index, mut end_index] = hyph_indices[j];
        let mut bp = bps[k];
        let mut line_stats = LineStats::default();
        let mut merged_element = ParagraphElement::Nothing;

        for (i, itm) in items.into_iter().enumerate() {
            if i == bp.index {
                let mut merged_width = 0;

                if let ParagraphElement::Text(TextElement {
                    ref text,
                    ref mut plan,
                    font_size,
                    font_kind,
                    font_style,
                    font_weight,
                    letter_spacing,
                    ref font_features,
                    ..
                }) = merged_element
                {
                    *plan = {
                        let font = self.fonts.as_mut().expect("fonts not initialized").get_mut(
                            font_kind,
                            font_style,
                            font_weight,
                        );
                        font.set_size(font_size, self.dpi);
                        font.plan(text, None, font_features.as_ref().map(Vec::as_slice))
                    };
                    plan.space_out(letter_spacing);
                    merged_width = plan.width;
                }

                if merged_width > 0 {
                    merged_items.push(ParagraphItem::Box {
                        width: merged_width,
                        data: merged_element,
                    });
                    merged_element = ParagraphElement::Nothing;
                }

                line_stats.merged_width += merged_width;
                let delta_width = line_stats.merged_width - line_stats.width;
                glue_drifts.push(-delta_width as f32 / line_stats.glues_count as f32);

                bps[k].index = bps[k].index.saturating_sub(index_drift);
                bps[k].width += delta_width;
                k += 1;

                if k < bps.len() {
                    bp = bps[k];
                }

                line_stats = LineStats::default();
                merged_items.push(itm);
                if i >= start_index && i < end_index {
                    start_index = i + 1;
                }
            } else if i >= start_index && i < end_index {
                if i > start_index {
                    index_drift += 1;
                }
                if let ParagraphItem::Box { width, data } = itm {
                    match merged_element {
                        ParagraphElement::Text(TextElement { ref mut text, .. }) => {
                            if let ParagraphElement::Text(TextElement {
                                text: other_text, ..
                            }) = data
                            {
                                text.push_str(&other_text);
                            }
                        }
                        ParagraphElement::Nothing => merged_element = data,
                        _ => (),
                    }
                    line_stats.width += width;
                    if !line_stats.started {
                        line_stats.started = true;
                    }
                }
                if i == end_index - 1 {
                    j += 1;
                    if let Some(&[s, e]) = hyph_indices.get(j) {
                        start_index = s;
                        end_index = e;
                    } else {
                        start_index = usize::MAX;
                        end_index = 0;
                    }
                    let mut merged_width = 0;
                    if let ParagraphElement::Text(TextElement {
                        ref text,
                        ref mut plan,
                        font_size,
                        font_kind,
                        font_style,
                        font_weight,
                        letter_spacing,
                        ref font_features,
                        ..
                    }) = merged_element
                    {
                        *plan = {
                            let font = self.fonts.as_mut().expect("fonts not initialized").get_mut(
                                font_kind,
                                font_style,
                                font_weight,
                            );
                            font.set_size(font_size, self.dpi);
                            font.plan(text, None, font_features.as_ref().map(Vec::as_slice))
                        };
                        plan.space_out(letter_spacing);
                        merged_width = plan.width;
                    }
                    merged_items.push(ParagraphItem::Box {
                        width: merged_width,
                        data: merged_element,
                    });
                    merged_element = ParagraphElement::Nothing;
                    line_stats.merged_width += merged_width;
                }
            } else {
                match itm {
                    ParagraphItem::Glue { .. } if line_stats.started => line_stats.glues_count += 1,
                    ParagraphItem::Box { .. } if !line_stats.started => line_stats.started = true,
                    _ => (),
                }
                merged_items.push(itm);
            }
        }

        merged_items
    }
}

fn format_list_prefix(kind: ListStyleType, index: usize) -> Option<String> {
    match kind {
        ListStyleType::None => None,
        ListStyleType::Disc => Some("· ".to_string()),
        ListStyleType::Circle => Some("o ".to_string()),
        ListStyleType::Square => Some("· ".to_string()),
        ListStyleType::Decimal => Some(format!("{}. ", index + 1)),
        ListStyleType::LowerRoman => Some(format!(
            "{}. ",
            Roman::from_unchecked(index as u32 + 1).to_lowercase()
        )),
        ListStyleType::UpperRoman => Some(format!(
            "{}. ",
            Roman::from_unchecked(index as u32 + 1).to_uppercase()
        )),
        ListStyleType::LowerAlpha | ListStyleType::UpperAlpha => {
            let i = index as u32 % 26;
            let start = if kind == ListStyleType::LowerAlpha {
                0x61
            } else {
                0x41
            };
            Some(format!("{}. ", char::try_from(start + i).unwrap_or('?')))
        }
        ListStyleType::LowerGreek | ListStyleType::UpperGreek => {
            let mut i = index as u32 % 24;
            // Skip .
            if i >= 17 {
                i += 1;
            }
            let start = if kind == ListStyleType::LowerGreek {
                0x03B1
            } else {
                0x0391
            };
            Some(format!("{}. ", char::try_from(start + i).unwrap_or('?')))
        }
    }
}
