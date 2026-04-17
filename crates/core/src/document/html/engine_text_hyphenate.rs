use super::engine::{Engine, ResourceFetcher};
use super::layout::{LineStats, ParagraphElement, StyleData, TextAlign, TextElement};
use kl_hyphenate::{Hyphenator, Standard};
use paragraph_breaker::{Breakpoint, Item as ParagraphItem, INFINITE_PENALTY};

impl Engine {
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
                    self.hyphenate_text_element(
                        element,
                        style,
                        dictionary,
                        hyph_indices,
                        &mut hyph_items,
                    );
                }
                _ => hyph_items.push(itm),
            }
        }

        hyph_items
    }

    fn hyphenate_text_element(
        &mut self,
        element: &TextElement,
        style: &StyleData,
        dictionary: &Standard,
        hyph_indices: &mut Vec<[usize; 2]>,
        hyph_items: &mut Vec<ParagraphItem<ParagraphElement>>,
    ) {
        let text = &element.text;
        let (hyphen_width, stretch) = self.calculate_hyphen_metrics(element);

        let mut index_before = text.find(char::is_alphabetic).unwrap_or_else(|| text.len());
        if index_before > 0 {
            let subelem = self.box_from_chunk(&text[0..index_before], 0, element);
            hyph_items.push(subelem);
        }

        let mut index_after = text[index_before..]
            .find(|c: char| !c.is_alphabetic())
            .map(|i| index_before + i)
            .unwrap_or_else(|| text.len());

        while index_before < index_after {
            let len_before = hyph_items.len();
            self.hyphenate_chunk(
                &text[index_before..index_after],
                index_before,
                element,
                style,
                hyphen_width,
                stretch,
                hyph_items,
            );

            let len_after = hyph_items.len();
            if len_after > 1 + len_before {
                hyph_indices.push([len_before, len_after]);
            }

            index_before = text[index_after..]
                .find(char::is_alphabetic)
                .map(|i| index_after + i)
                .unwrap_or_else(|| text.len());

            if index_before > index_after {
                let subelem =
                    self.box_from_chunk(&text[index_after..index_before], index_after, element);
                hyph_items.push(subelem);
            }

            index_after = text[index_before..]
                .find(|c: char| !c.is_alphabetic())
                .map(|i| index_before + i)
                .unwrap_or_else(|| text.len());
        }
    }

    fn calculate_hyphen_metrics(&mut self, element: &TextElement) -> (i32, i32) {
        let font = self.fonts.as_mut().expect("fonts not initialized").get_mut(
            element.font_kind,
            element.font_style,
            element.font_weight,
        );
        font.set_size(element.font_size, self.dpi);
        let plan = font.plan(" -", None, element.font_features.as_deref());
        (plan.glyph_advance(1), 3 * plan.glyph_advance(0))
    }

    fn hyphenate_chunk(
        &mut self,
        chunk: &str,
        index_before: usize,
        element: &TextElement,
        style: &StyleData,
        hyphen_width: i32,
        stretch: i32,
        hyph_items: &mut Vec<ParagraphItem<ParagraphElement>>,
    ) {
        let mut index = 0;
        for segment in dictionary.hyphenate(chunk) {
            let subelem = self.box_from_chunk(segment.as_str(), index_before + index, element);
            hyph_items.push(subelem);
            index += segment.len();
            if index < chunk.len() {
                self.add_hyphen_penalty_items(style, hyphen_width, stretch, hyph_items);
            }
        }
    }

    fn add_hyphen_penalty_items(
        &self,
        style: &StyleData,
        hyphen_width: i32,
        stretch: i32,
        hyph_items: &mut Vec<ParagraphItem<ParagraphElement>>,
    ) {
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
                self.process_breakpoint(
                    &mut merged_element,
                    &mut line_stats,
                    &mut merged_items,
                    glue_drifts,
                    &mut index_drift,
                    &mut k,
                    bps,
                    &mut bp,
                    i,
                    &mut start_index,
                );
                merged_items.push(itm);
                if i >= start_index && i < end_index {
                    start_index = i + 1;
                }
            } else if i >= start_index && i < end_index {
                self.process_hyphenated_segment(
                    itm,
                    &mut merged_element,
                    &mut line_stats,
                    &mut index_drift,
                    i,
                    &mut start_index,
                    &mut end_index,
                    &mut j,
                    hyph_indices,
                    &mut merged_items,
                );
            } else {
                self.process_regular_item(itm, &mut line_stats, &mut merged_items);
            }
        }

        merged_items
    }

    fn process_breakpoint(
        &mut self,
        merged_element: &mut ParagraphElement,
        line_stats: &mut LineStats,
        merged_items: &mut Vec<ParagraphItem<ParagraphElement>>,
        glue_drifts: &mut Vec<f32>,
        index_drift: &mut usize,
        k: &mut usize,
        bps: &mut Vec<Breakpoint>,
        bp: &mut Breakpoint,
        i: usize,
        start_index: &mut usize,
    ) {
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
                data: std::mem::replace(merged_element, ParagraphElement::Nothing),
            });
        }

        line_stats.merged_width += merged_width;
        let delta_width = line_stats.merged_width - line_stats.width;
        glue_drifts.push(-delta_width as f32 / line_stats.glues_count as f32);

        bps[*k].index = bps[*k].index.saturating_sub(*index_drift);
        bps[*k].width += delta_width;
        *k += 1;

        if *k < bps.len() {
            *bp = bps[*k];
        }

        *line_stats = LineStats::default();
        *start_index = i + 1;
    }

    fn process_hyphenated_segment(
        &mut self,
        itm: ParagraphItem<ParagraphElement>,
        merged_element: &mut ParagraphElement,
        line_stats: &mut LineStats,
        index_drift: &mut usize,
        i: usize,
        start_index: &mut usize,
        end_index: &mut usize,
        j: &mut usize,
        hyph_indices: &[[usize; 2]],
        merged_items: &mut Vec<ParagraphItem<ParagraphElement>>,
    ) {
        if i > *start_index {
            *index_drift += 1;
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
                ParagraphElement::Nothing => *merged_element = data,
                _ => (),
            }
            line_stats.width += width;
            if !line_stats.started {
                line_stats.started = true;
            }
        }
        if i == *end_index - 1 {
            *j += 1;
            if let Some(&[s, e]) = hyph_indices.get(*j) {
                *start_index = s;
                *end_index = e;
            } else {
                *start_index = usize::MAX;
                *end_index = 0;
            }
            self.finalize_merged_segment(merged_element, line_stats, merged_items);
        }
    }

    fn finalize_merged_segment(
        &mut self,
        merged_element: &mut ParagraphElement,
        line_stats: &mut LineStats,
        merged_items: &mut Vec<ParagraphItem<ParagraphElement>>,
    ) {
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
            data: std::mem::replace(merged_element, ParagraphElement::Nothing),
        });
        line_stats.merged_width += merged_width;
    }

    fn process_regular_item(
        &self,
        itm: ParagraphItem<ParagraphElement>,
        line_stats: &mut LineStats,
        merged_items: &mut Vec<ParagraphItem<ParagraphElement>>,
    ) {
        match itm {
            ParagraphItem::Glue { .. } if line_stats.started => line_stats.glues_count += 1,
            ParagraphItem::Box { .. } if !line_stats.started => line_stats.started = true,
            _ => (),
        }
        merged_items.push(itm);
    }
}
