use super::engine::{Engine, ResourceFetcher};
use super::layout::{
    FontKind, GlueMaterial, ImageElement, InlineMaterial, ParagraphElement, ParagraphItem,
    PenaltyMaterial, StyleData, TextAlign, TextElement, WordSpacing, EM_SPACE_RATIOS, FONT_SPACES,
    WORD_SPACE_RATIOS,
};
use super::xml;
use super::xml::XmlExt;
use crate::document::pdf::PdfOpener;
use crate::unit::pt_to_px;
use paragraph_breaker::{Breakpoint, Item as ParagraphItem, INFINITE_PENALTY};
use xi_unicode::LineBreakIterator;

impl Engine {
    pub(super) fn make_paragraph_items(
        &mut self,
        inlines: &[InlineMaterial],
        parent_style: &StyleData,
        line_width: i32,
        resource_fetcher: &mut dyn ResourceFetcher,
    ) -> (Vec<ParagraphItem<ParagraphElement>>, Vec<ImageElement>) {
        let mut items = Vec::with_capacity(inlines.len());
        let mut floats = Vec::new();
        let big_stretch = self.calculate_big_stretch(parent_style);

        if parent_style.text_align == TextAlign::Center {
            self.add_center_alignment_glue(&mut items, big_stretch);
        }

        for (index, mater) in inlines.iter().enumerate() {
            match mater {
                InlineMaterial::Image(image_material) => {
                    self.process_image_material(
                        image_material,
                        parent_style,
                        resource_fetcher,
                        &mut items,
                        &mut floats,
                    );
                }
                InlineMaterial::Text(text_material) => {
                    self.process_text_material(
                        text_material,
                        parent_style,
                        index,
                        inlines,
                        line_width,
                        big_stretch,
                        &mut items,
                    );
                }
                InlineMaterial::LineBreak => {
                    self.add_line_break_items(&mut items, parent_style, line_width, big_stretch);
                }
                InlineMaterial::Glue(glue_material) => {
                    items.push(ParagraphItem::Glue {
                        width: glue_material.width,
                        stretch: glue_material.stretch,
                        shrink: glue_material.shrink,
                    });
                }
                InlineMaterial::Penalty(penalty_material) => {
                    items.push(ParagraphItem::Penalty {
                        width: penalty_material.width,
                        penalty: penalty_material.penalty,
                        flagged: penalty_material.flagged,
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

        self.add_paragraph_trailing_items(&mut items, parent_style, line_width, big_stretch);

        (items, floats)
    }

    fn calculate_big_stretch(&mut self, parent_style: &StyleData) -> i32 {
        // Estimate space width based on font size using approximate metrics.
        let font_size = parent_style.font_size as f32;
        let space_width = (font_size * 0.25) as i32;
        3 * space_width
    }

    fn add_center_alignment_glue(
        &self,
        items: &mut Vec<ParagraphItem<ParagraphElement>>,
        big_stretch: i32,
    ) {
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

    fn process_image_material(
        &mut self,
        image_material: &super::layout::ImageMaterial,
        parent_style: &StyleData,
        resource_fetcher: &mut dyn ResourceFetcher,
        items: &mut Vec<ParagraphItem<ParagraphElement>>,
        floats: &mut Vec<ImageElement>,
    ) {
        if let Some(element) = self.create_image_element(
            image_material.offset,
            &image_material.path,
            &image_material.style,
            resource_fetcher,
        ) {
            self.add_image_to_paragraph_items(element, items, floats);
        }
    }

    fn process_text_material(
        &mut self,
        text_material: &super::layout::TextMaterial,
        parent_style: &StyleData,
        index: usize,
        inlines: &[InlineMaterial],
        line_width: i32,
        big_stretch: i32,
        items: &mut Vec<ParagraphItem<ParagraphElement>>,
    ) {
        let font_size = (text_material.style.font_size * 64.0) as u32;
        // Estimate space width based on font size using approximate metrics.
        let space_width = (font_size as f32 * 0.25) as i32;

        let mut start_index = 0;
        for (end_index, _is_hardbreak) in LineBreakIterator::new(&text_material.text) {
            for chunk in
                text_material.text[start_index..end_index].split_inclusive(char::is_whitespace)
            {
                if let Some((i, c)) = chunk.char_indices().next_back() {
                    let j = i + if c.is_whitespace() { 0 } else { c.len_utf8() };
                    if j > 0 {
                        let buf = &text_material.text[start_index..start_index + j];
                        let local_offset = text_material.offset + start_index;
                        // Estimate text width based on character count and font size.
                        // Uses approximate metrics when full text shaping is not available.
                        let avg_char_width = (font_size as f32 * 0.6) as i32;
                        let width = buf.len() as i32 * avg_char_width;
                        let plan = super::layout::TextPlan {
                            width,
                            ascent: (font_size as f32 * 0.8) as i32,
                            descent: (font_size as f32 * 0.2) as i32,
                            ..Default::default()
                        };

                        items.push(ParagraphItem::Box {
                            width: plan.width,
                            data: ParagraphElement::Text(TextElement {
                                offset: local_offset,
                                language: text_material.style.language.clone(),
                                text: buf.to_string(),
                                plan,
                                font_features: text_material.style.font_features.clone(),
                                font_kind: text_material.style.font_kind,
                                font_style: text_material.style.font_style,
                                font_weight: text_material.style.font_weight,
                                vertical_align: text_material.style.vertical_align,
                                letter_spacing: text_material.style.letter_spacing,
                                font_size,
                                color: text_material.style.color,
                                uri: text_material.style.uri.clone(),
                            }),
                        });
                    }

                    if c.is_whitespace() {
                        self.process_whitespace_character(
                            c,
                            &text_material.text,
                            start_index + i,
                            index,
                            inlines,
                            parent_style,
                            line_width,
                            big_stretch,
                            space_plan,
                            &text_material.style,
                            &mut start_index,
                            chunk.len(),
                            items,
                        );
                    } else if end_index < text_material.text.len() {
                        self.process_break_opportunity(c, parent_style, space_plan, items);
                    }
                }
                start_index += chunk.len();
            }
        }
    }

    fn process_whitespace_character(
        &self,
        c: char,
        text: &str,
        char_index: usize,
        text_index: usize,
        inlines: &[InlineMaterial],
        parent_style: &StyleData,
        line_width: i32,
        big_stretch: i32,
        space_plan: &super::layout::FontPlan,
        style: &StyleData,
        start_index: &mut usize,
        chunk_len: usize,
        items: &mut Vec<ParagraphItem<ParagraphElement>>,
    ) {
        if c == '\n' && parent_style.retain_whitespace {
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
            *start_index += chunk_len;
            return;
        }

        let last_c = text[..char_index].chars().next_back().or_else(|| {
            if text_index > 0 {
                inlines[text_index - 1]
                    .text()
                    .and_then(|text| text.chars().next_back())
            } else {
                None
            }
        });

        let has_more = text[char_index..].chars().any(|c| !c.is_xml_whitespace())
            || inlines[text_index + 1..].iter().any(|m| {
                m.text()
                    .map(|text| text.chars().any(|c| !c.is_xml_whitespace())).unwrap_or(false)
            });

        if !parent_style.retain_whitespace
            && c.is_xml_whitespace()
            && (last_c.map(|c| c.is_xml_whitespace()) != Some(false) || !has_more)
        {
            *start_index += chunk_len;
            return;
        }

        let mut width = if !parent_style.retain_whitespace {
            space_plan.glyph_advance(0)
        } else if let Some(index) = FONT_SPACES.chars().position(|x| x == c) {
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

        let is_unbreakable = c == '\u{00A0}' || c == '\u{202F}' || c == '\u{2007}';

        if (is_unbreakable || (parent_style.retain_whitespace && c.is_xml_whitespace()))
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

        self.add_glue_for_text_align(
            parent_style.text_align,
            style,
            is_unbreakable,
            width,
            big_stretch,
            space_plan,
            items,
        );
    }

    fn add_glue_for_text_align(
        &self,
        text_align: TextAlign,
        style: &StyleData,
        is_unbreakable: bool,
        width: i32,
        big_stretch: i32,
        space_plan: &super::layout::FontPlan,
        items: &mut Vec<ParagraphItem<ParagraphElement>>,
    ) {
        match text_align {
            TextAlign::Justify => {
                items.push(ParagraphItem::Glue {
                    width,
                    stretch: width / 2,
                    shrink: width / 3,
                });
            }
            TextAlign::Center => {
                if style.font_kind == FontKind::Monospace || is_unbreakable {
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
                if style.font_kind == FontKind::Monospace || is_unbreakable {
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
    }

    fn process_break_opportunity(
        &self,
        c: char,
        parent_style: &StyleData,
        space_plan: &super::layout::FontPlan,
        items: &mut Vec<ParagraphItem<ParagraphElement>>,
    ) {
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

    fn add_line_break_items(
        &self,
        items: &mut Vec<ParagraphItem<ParagraphElement>>,
        parent_style: &StyleData,
        line_width: i32,
        big_stretch: i32,
    ) {
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

    fn add_paragraph_trailing_items(
        &self,
        items: &mut Vec<ParagraphItem<ParagraphElement>>,
        parent_style: &StyleData,
        line_width: i32,
        big_stretch: i32,
    ) {
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
    }
}
