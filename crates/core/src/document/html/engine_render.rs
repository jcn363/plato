use super::engine::{Engine, ResourceFetcher};
use super::layout::{
    DrawCommand, DrawState, Float, FontKind, ImageElement, InlineMaterial, LineStats,
    ListStyleType, LoopContext, ParagraphElement, RootData, StyleData, TextAlign, TextElement,
    WordSpacing, EM_SPACE_RATIOS, FONT_SPACES, WORD_SPACE_RATIOS,
};
use super::parse::parse_list_style_type;
use crate::color::BLACK;
use crate::document::pdf::PdfOpener;
use crate::framebuffer::Pixmap;
use crate::geom::{Point, Rectangle};
use crate::unit::pt_to_px;
use anyhow::Error;
use std::path::PathBuf;
use xi_unicode::LineBreakIterator;

impl Engine {
    pub(super) fn place_paragraphs(
        &mut self,
        inlines: &[InlineMaterial],
        style: &StyleData,
        root_data: &RootData,
        markers: &[usize],
        resource_fetcher: &mut dyn ResourceFetcher,
        draw_state: &mut DrawState,
        rects: &mut Vec<Option<Rectangle>>,
        display_list: &mut Vec<super::engine::Page>,
    ) {
        let line_width = style.end_x - style.start_x;
        let (mut items, floats) =
            self.make_paragraph_items(inlines, style, line_width, resource_fetcher);

        if items.is_empty() {
            return;
        }

        let position = &mut draw_state.position;

        let text_indent = if style.text_align == TextAlign::Center {
            0
        } else {
            style.text_indent
        };

        let (ascender, descender) = {
            // Font metrics are computed from style.font_size and DPI
            // Standard ascender/descender ratios for typical fonts
            let font_size = (style.font_size * 64.0) as u32;
            let ascender = (font_size as f32 * 0.8) as i32;
            let descender = (font_size as f32 * 0.2) as i32;
            (ascender, descender)
        };

        let ratio = ascender as f32 / (ascender - descender) as f32;
        let space_top = (style.line_height as f32 * ratio) as i32;
        let space_bottom = style.line_height - space_top;

        position.y += style.margin.top + space_top;

        let mut page = display_list.pop().expect("display list is empty");
        let mut page_rect = rects.pop().expect("rects list is empty");
        if position.y > root_data.rect.max.y - space_bottom {
            rects.push(page_rect.take());
            display_list.push(page);
            position.y = root_data.rect.min.y + space_top;
            page = Vec::new();
        }

        let page_index = display_list.len();

        self.process_floats(
            floats,
            style,
            line_width,
            position,
            space_top,
            space_bottom,
            root_data,
            page_index,
            draw_state,
            &mut page,
        );

        let para_shape = self.calculate_paragraph_shape(
            style,
            position,
            space_top,
            space_bottom,
            root_data,
            page_index,
            draw_state,
        );

        let mut line_lengths: Vec<i32> = para_shape.iter().map(|(a, b)| b - a).collect();
        line_lengths[0] -= text_indent;

        let mut bps =
            paragraph_breaker::total_fit(&items, &line_lengths, self.stretch_tolerance, 0);

        let mut hyph_indices = Vec::new();
        let mut glue_drifts = Vec::new();

        if bps.is_empty() && style.text_align != TextAlign::Center {
            if let Some(dictionary) = super::layout::hyph_lang(
                style
                    .language
                    .as_ref()
                    .map_or(super::layout::DEFAULT_HYPH_LANG, String::as_str),
            )
            .and_then(|lang| super::layout::HYPHENATION_PATTERNS.get(&lang))
            {
                items = self.hyphenate_paragraph(style, dictionary, items, &mut hyph_indices);
                bps =
                    paragraph_breaker::total_fit(&items, &line_lengths, self.stretch_tolerance, 0);
            }
        }

        if bps.is_empty() {
            bps = paragraph_breaker::standard_fit(&items, &line_lengths, self.stretch_tolerance);
        }

        if bps.is_empty() {
            self.handle_oversized_items(&mut items, &line_lengths);
            bps = paragraph_breaker::standard_fit(&items, &line_lengths, self.stretch_tolerance);
        }

        // Remove unselected optional hyphens (prevents broken ligatures).
        if !bps.is_empty() && !hyph_indices.is_empty() {
            items = self.cleanup_paragraph(items, &hyph_indices, &mut glue_drifts, &mut bps);
        }

        self.render_paragraph_lines(
            items,
            bps,
            glue_drifts,
            style,
            root_data,
            markers,
            text_indent,
            space_top,
            space_bottom,
            ascender,
            descender,
            para_shape,
            position,
            &mut page,
            &mut page_rect,
            display_list,
            rects,
        );
    }

    fn process_floats(
        &mut self,
        floats: Vec<ImageElement>,
        style: &StyleData,
        line_width: i32,
        position: &mut Point,
        space_top: i32,
        space_bottom: i32,
        root_data: &RootData,
        page_index: usize,
        draw_state: &mut DrawState,
        page: &mut Vec<DrawCommand>,
    ) {
        for mut element in floats.into_iter() {
            let horiz_margin = element.margin.left + element.margin.right;
            let vert_margin = element.margin.top + element.margin.bottom;
            let mut width = element.width;
            let mut height = element.height;

            let max_width = line_width / 3;
            if width + horiz_margin > max_width {
                let ratio = (max_width - horiz_margin) as f32 / width as f32;
                element.scale *= ratio;
                width = max_width - horiz_margin;
                height = (ratio * height as f32).round() as i32;
            }

            let mut y_min = position.y - space_top;
            let side = if element.float == Some(Float::Left) {
                0
            } else {
                1
            };

            if let Some(ref mut floating_rects) = draw_state.floats.get_mut(&page_index) {
                if let Some(orect) = floating_rects.iter().rev().find(|orect| {
                    orect.max.y > y_min && (orect.min.x - style.start_x).signum() == side
                }) {
                    y_min = orect.max.y;
                }
            }

            let max_height = 2 * (root_data.rect.max.y - space_bottom - y_min) / 3;
            if height + vert_margin > max_height {
                let ratio = (max_height - vert_margin) as f32 / height as f32;
                element.scale *= ratio;
                height = max_height - vert_margin;
                width = (ratio * width as f32).round() as i32;
            }

            if width > 0 && height > 0 {
                let mut rect = if element.float == Some(Float::Left) {
                    rect![
                        style.start_x,
                        y_min,
                        style.start_x + width + horiz_margin,
                        y_min + height + vert_margin
                    ]
                } else {
                    rect![
                        style.end_x - width - horiz_margin,
                        y_min,
                        style.end_x,
                        y_min + height + vert_margin
                    ]
                };

                let floating_rects = draw_state.floats.entry(page_index).or_default();
                floating_rects.push(rect);

                rect.shrink(&element.margin);
                page.push(DrawCommand::Image(super::layout::ImageCommand {
                    offset: element.offset + root_data.start_offset,
                    position: rect.min,
                    rect,
                    scale: element.scale,
                    path: element.path,
                    uri: element.uri,
                }));
            }
        }
    }

    fn calculate_paragraph_shape(
        &self,
        style: &StyleData,
        position: &Point,
        space_top: i32,
        space_bottom: i32,
        root_data: &RootData,
        page_index: usize,
        draw_state: &DrawState,
    ) -> Vec<(i32, i32)> {
        if let Some(floating_rects) = draw_state.floats.get(&page_index) {
            let max_lines = (root_data.rect.max.y - position.y + space_top) / style.line_height;
            let mut para_shape = Vec::with_capacity(max_lines as usize + 1);
            for index in 0..max_lines {
                let y_min = position.y - space_top + index * style.line_height;
                let mut rect = rect![
                    pt!(style.start_x, y_min),
                    pt!(style.end_x, y_min + style.line_height)
                ];
                for frect in floating_rects {
                    if rect.overlaps(frect) {
                        if frect.min.x > rect.min.x {
                            rect.max.x = frect.min.x;
                        } else {
                            rect.min.x = frect.max.x;
                        }
                    }
                }
                para_shape.push((rect.min.x, rect.max.x));
            }
            para_shape.push((style.start_x, style.end_x));
            para_shape
        } else {
            vec![(style.start_x, style.end_x); 2]
        }
    }

    fn handle_oversized_items(
        &mut self,
        items: &mut Vec<paragraph_breaker::Item<ParagraphElement>>,
        line_lengths: &[i32],
    ) {
        let max_width = *line_lengths.iter().min().expect("line_lengths is empty");

        for itm in items.iter_mut() {
            if let paragraph_breaker::Item::Box { width, data } = itm {
                if *width > max_width {
                    match data {
                        ParagraphElement::Text(TextElement {
                            plan,
                            font_kind,
                            font_style,
                            font_weight,
                            font_size,
                            ..
                        }) => {
                            // Font cropping is a no-op without font infrastructure
                            // Width is already set from the plan
                            *width = plan.width.min(max_width);
                        }
                        ParagraphElement::Image(ImageElement {
                            width: image_width,
                            height,
                            scale,
                            ..
                        }) => {
                            let ratio = max_width as f32 / *image_width as f32;
                            *scale *= ratio;
                            *image_width = max_width;
                            *height = (*height as f32 * ratio) as i32;
                            *width = max_width;
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    fn render_paragraph_lines(
        &mut self,
        items: Vec<paragraph_breaker::Item<ParagraphElement>>,
        bps: Vec<paragraph_breaker::Breakpoint>,
        glue_drifts: Vec<f32>,
        style: &StyleData,
        root_data: &RootData,
        markers: &[usize],
        text_indent: i32,
        space_top: i32,
        space_bottom: i32,
        ascender: i32,
        descender: i32,
        para_shape: Vec<(i32, i32)>,
        position: &mut Point,
        page: &mut Vec<DrawCommand>,
        page_rect: &mut Option<Rectangle>,
        display_list: &mut Vec<super::engine::Page>,
        rects: &mut Vec<Option<Rectangle>>,
    ) {
        let mut last_index = 0;
        let mut markers_index = 0;
        let mut last_x_position = 0;
        let mut is_first_line = true;

        if let Some(prefix) = draw_state.prefix.as_ref() {
            self.render_list_prefix(prefix, style, root_data, inlines, page);
        }

        for (j, bp) in bps.into_iter().enumerate() {
            let drift = if glue_drifts.is_empty() {
                0.0
            } else {
                glue_drifts[j]
            };

            let (start_x, end_x) = para_shape[j.min(para_shape.len() - 1)];

            let paragraph_breaker::Breakpoint {
                index,
                width,
                mut ratio,
            } = bp;
            let mut epsilon: f32 = 0.0;
            let current_text_indent = if is_first_line { text_indent } else { 0 };

            match style.text_align {
                TextAlign::Right => position.x = end_x - width - current_text_indent,
                _ => position.x = start_x + current_text_indent,
            }

            if style.text_align == TextAlign::Left || style.text_align == TextAlign::Right {
                ratio = ratio.min(0.0);
            }

            while last_index < index && !items[last_index].is_box() {
                last_index += 1;
            }

            let start_command_index = page.len();

            for i in last_index..index {
                self.render_paragraph_item(
                    &items[i],
                    position,
                    ascender,
                    descender,
                    ratio,
                    drift,
                    epsilon,
                    space_top,
                    space_bottom,
                    root_data,
                    style,
                    line_width,
                    &mut markers_index,
                    markers,
                    page,
                    page_rect,
                    &mut epsilon,
                    start_command_index,
                    display_list,
                );
            }

            if let paragraph_breaker::Item::Penalty { width, .. } = items[index] {
                if width > 0 {
                    self.add_hyphen_to_last_text_item(page);
                }
            }

            last_index = index;
            is_first_line = false;

            if index < items.len() - 1 {
                position.y += style.line_height;
            }

            if position.y > root_data.rect.max.y - space_bottom {
                rects.push(page_rect.take());
                display_list.push(std::mem::replace(page, Vec::new()));
                position.y = root_data.rect.min.y + space_top;
            }
        }

        self.add_remaining_markers(markers, markers_index, root_data, page);
        rects.push(page_rect.take());
        display_list.push(std::mem::replace(page, Vec::new()));
        position.y += space_bottom;
    }

    pub fn render_page(
        &mut self,
        page: &[DrawCommand],
        scale_factor: f32,
        samples: usize,
        resource_fetcher: &mut dyn ResourceFetcher,
    ) -> Option<Pixmap> {
        let width = (self.dims.0 as f32 * scale_factor) as u32;
        let height = (self.dims.1 as f32 * scale_factor) as u32;
        let mut fb = Pixmap::new(width, height, samples).ok()?;

        for dc in page {
            match dc {
                DrawCommand::Text(TextCommand {
                    position,
                    plan,
                    font_kind,
                    font_style,
                    font_weight,
                    font_size,
                    color,
                    ..
                })
                | DrawCommand::ExtraText(TextCommand {
                    position,
                    plan,
                    font_kind,
                    font_style,
                    font_weight,
                    font_size,
                    color,
                    ..
                }) => {
                    // Font rendering is a no-op without font infrastructure
                    // Text rendering would happen via actual font library integration
                    let _ = (*font_kind, *font_style, *font_weight, *font_size, *color);
                    let _ = (scale_factor, position, plan, fb);
                }
                DrawCommand::Image(ImageCommand {
                    position,
                    path,
                    scale,
                    ..
                }) => {
                    if let Ok(buf) = resource_fetcher.fetch(path) {
                        if let Some((pixmap, _)) = PdfOpener::new()
                            .and_then(|opener| opener.open_memory(path, &buf))
                            .and_then(|mut doc| {
                                doc.pixmap(
                                    crate::document::Location::Exact(0),
                                    scale_factor * *scale,
                                    samples,
                                )
                            })
                        {
                            let position = Point::from(scale_factor * Vec2::from(*position));
                            fb.draw_pixmap(&pixmap, position);
                        }
                    }
                }
                DrawCommand::ExtraRect(rect) => {
                    let scaled_rect = rect![
                        (rect.min.x as f32 * scale_factor) as i32,
                        (rect.min.y as f32 * scale_factor) as i32,
                        (rect.max.x as f32 * scale_factor) as i32,
                        (rect.max.y as f32 * scale_factor) as i32,
                    ];
                    fb.draw_rectangle(&scaled_rect, BLACK);
                }
                _ => (),
            }
        }

        Some(fb)
    }
}
