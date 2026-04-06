use crate::document::html::css::CssParser;
use crate::document::html::engine::Page;
use crate::document::html::layout::{DrawCommand, DrawState, RootData};
use crate::document::html::layout::{LoopContext, StyleData};
use crate::document::html::style::StyleSheet;
use crate::document::html::xml::XmlParser;
use crate::helpers::Normalize;
use crate::unit::pt_to_px;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use super::opener::EpubDocument;

const VIEWER_STYLESHEET: &str = "css/epub.css";
const USER_STYLESHEET: &str = "css/epub-user.css";

impl EpubDocument {
    #[inline]
    pub fn page_index(
        &mut self,
        offset: usize,
        index: usize,
        start_offset: usize,
    ) -> Option<usize> {
        if !self.cache.contains_key(&index) {
            let display_list = self.build_display_list(index, start_offset);
            self.cache.insert(index, display_list);
        }
        self.cache.get(&index).map(|display_list| {
            if display_list.len() < 2
                || display_list[1].first().map(|dc| offset < dc.offset()) == Some(true)
            {
                return 0;
            } else if display_list[display_list.len() - 1]
                .first()
                .map(|dc| offset >= dc.offset())
                == Some(true)
            {
                return display_list.len() - 1;
            } else {
                for i in 1..display_list.len() - 1 {
                    if display_list[i].first().map(|dc| offset >= dc.offset()) == Some(true)
                        && display_list[i + 1].first().map(|dc| offset < dc.offset()) == Some(true)
                    {
                        return i;
                    }
                }
            }
            0
        })
    }

    pub fn build_display_list(&mut self, index: usize, start_offset: usize) -> Vec<Page> {
        let mut spine_dir = PathBuf::default();
        let text = {
            let path = &self.spine[index].path;
            if let Some(parent) = Path::new(path).parent() {
                spine_dir = parent.to_path_buf();
            }
            let mut zf = self.archive.by_name(path).ok();
            let size = zf.as_ref().map(|f| f.size() as usize).unwrap_or(0);
            let mut s = String::with_capacity(size);
            if let Some(ref mut f) = zf {
                f.read_to_string(&mut s).ok();
            }
            s
        };

        let mut root = XmlParser::new(&text).parse();
        root.wrap_lost_inlines();

        let mut stylesheet = StyleSheet::new();

        if let Ok(text) = fs::read_to_string(VIEWER_STYLESHEET) {
            let mut css = CssParser::new(&text).parse();
            stylesheet.append(&mut css, true);
        }

        if let Ok(text) = fs::read_to_string(USER_STYLESHEET) {
            let mut css = CssParser::new(&text).parse();
            stylesheet.append(&mut css, true);
        }

        if !self.ignore_document_css {
            let mut inner_css = StyleSheet::new();
            if let Some(head) = root.root().find("head") {
                for child in head.children() {
                    if child.tag_name() == Some("link")
                        && child.attribute("rel") == Some("stylesheet")
                    {
                        if let Some(href) = child.attribute("href") {
                            if let Some(name) = spine_dir.join(href).normalize().to_str() {
                                if let Ok(mut zf) = self.archive.by_name(name) {
                                    let size = zf.size() as usize;
                                    let mut text = String::with_capacity(size);
                                    zf.read_to_string(&mut text).ok();
                                    let mut css = CssParser::new(&text).parse();
                                    inner_css.append(&mut css, false);
                                }
                            }
                        }
                    } else if child.tag_name() == Some("style")
                        && child.attribute("type") == Some("text/css")
                    {
                        let mut css = CssParser::new(&child.text()).parse();
                        inner_css.append(&mut css, false);
                    }
                }
            }

            stylesheet.append(&mut inner_css, true);
        }

        let mut display_list = Vec::new();

        if let Some(body) = root.root().find("body") {
            let mut rect = self.engine.rect();
            rect.shrink(&self.engine.margin);

            let language = self.language().or_else(|| {
                root.root()
                    .find("html")
                    .and_then(|html| html.attribute("xml:lang"))
                    .map(String::from)
            });

            let style = StyleData {
                language,
                font_size: self.engine.font_size,
                line_height: pt_to_px(
                    self.engine.line_height * self.engine.font_size,
                    self.engine.dpi,
                )
                .round() as i32,
                text_align: self.engine.text_align,
                start_x: rect.min.x,
                end_x: rect.max.x,
                width: rect.max.x - rect.min.x,
                ..Default::default()
            };

            let loop_context = LoopContext::default();
            let mut draw_state = DrawState {
                position: rect.min,
                ..Default::default()
            };

            let root_data = RootData {
                start_offset,
                spine_dir,
                rect,
            };

            display_list.push(Vec::new());

            self.engine.build_display_list(
                body,
                &style,
                &loop_context,
                &stylesheet,
                &root_data,
                &mut self.archive,
                &mut draw_state,
                &mut display_list,
            );

            display_list.retain(|page| !page.is_empty());

            if display_list.is_empty() {
                display_list.push(vec![DrawCommand::Marker(start_offset + body.offset())]);
            }
        } else {
            display_list.push(vec![DrawCommand::Marker(start_offset)]);
        }

        display_list
    }
}
