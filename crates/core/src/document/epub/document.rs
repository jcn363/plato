use crate::document::epub::opener::EpubDocument;
use crate::document::html::engine::ResourceFetcher;
use crate::document::html::layout::{DrawCommand, ImageCommand, TextCommand};
use crate::document::html::xml::XmlParser;
use crate::document::pdf::PdfOpener;
use crate::document::{chapter_from_uri, BoundedText, Document, Location, TocEntry};
use crate::framebuffer::Pixmap;
use crate::geom::{Boundary, CycleDir};
use crate::helpers::{decode_entities, Normalize};
use crate::metadata::TextAlign;
use rustc_hash::FxHashMap;
use std::io::Read;
use std::path::Path;

impl Document for EpubDocument {
    fn preview_pixmap(&mut self, width: f32, height: f32, samples: usize) -> Option<Pixmap> {
        let opener = PdfOpener::new()?;
        self.cover_image()
            .map(|path| self.parent.join(path).to_string_lossy().into_owned())
            .and_then(|path| {
                self.archive
                    .fetch(&path)
                    .ok()
                    .and_then(|buf| opener.open_memory(&path, &buf))
                    .and_then(|mut doc| {
                        doc.dims(0).and_then(|dims| {
                            let scale = (width / dims.0).min(height / dims.1);
                            doc.pixmap(Location::Exact(0), scale, samples)
                        })
                    })
            })
            .or_else(|| {
                self.dims(0).and_then(|dims| {
                    let scale = (width / dims.0).min(height / dims.1);
                    self.pixmap(Location::Exact(0), scale, samples)
                })
            })
            .map(|(pixmap, _)| pixmap)
    }

    #[inline]
    fn dims(&self, _index: usize) -> Option<(f32, f32)> {
        Some((self.engine.dims.0 as f32, self.engine.dims.1 as f32))
    }

    fn pages_count(&self) -> usize {
        self.spine.iter().map(|c| c.size).sum()
    }

    fn toc(&mut self) -> Option<Vec<TocEntry>> {
        let name = self
            .info
            .root()
            .find("spine")
            .and_then(|spine| spine.attribute("toc"))
            .and_then(|toc_id| {
                self.info
                    .root()
                    .find("manifest")
                    .and_then(|manifest| manifest.find_by_id(toc_id))
                    .and_then(|entry| entry.attribute("href"))
            })
            .or_else(|| {
                self.info
                    .root()
                    .find("manifest")
                    .and_then(|manifest| {
                        manifest.children().find(|child| {
                            child
                                .attribute("properties")
                                .iter()
                                .any(|props| props.split_whitespace().any(|prop| prop == "nav"))
                        })
                    })
                    .and_then(|entry| entry.attribute("href"))
            })
            .map(|href| {
                self.parent
                    .join(href)
                    .normalize()
                    .to_string_lossy()
                    .into_owned()
            })?;

        let toc_dir = Path::new(&name).parent().unwrap_or_else(|| Path::new(""));

        let text = if let Ok(mut zf) = self.archive.by_name(&name) {
            let size = zf.size() as usize;
            let mut text = String::with_capacity(size);
            zf.read_to_string(&mut text).ok();
            text
        } else {
            return None;
        };

        let root = XmlParser::new(&text).parse();

        if name.ends_with(".ncx") {
            root.root()
                .find("navMap")
                .map(|map| self.walk_toc_ncx(map, toc_dir, &mut 0, &mut FxHashMap::default()))
        } else {
            root.root()
                .descendants()
                .find(|desc| {
                    desc.tag_name() == Some("nav") && desc.attribute("epub:type") == Some("toc")
                })
                .and_then(|map| map.find("ol"))
                .map(|map| self.walk_toc_nav(map, toc_dir, &mut 0, &mut FxHashMap::default()))
        }
    }

    fn chapter<'a>(&mut self, offset: usize, toc: &'a [TocEntry]) -> Option<(&'a TocEntry, f32)> {
        let next_offset = self
            .resolve_location(Location::Next(offset))
            .unwrap_or(usize::MAX);
        let (index, start_offset) = self.vertebra_coordinates(offset)?;
        let path = self.spine[index].path.clone();
        let mut end_offset = start_offset + self.spine[index].size;
        let mut chap_before = None;
        let mut chap_after = None;
        let mut offset_before = 0;
        let mut offset_after = usize::MAX;

        self.chapter_aux(
            toc,
            offset,
            next_offset,
            &path,
            &mut end_offset,
            &mut chap_before,
            &mut offset_before,
            &mut chap_after,
            &mut offset_after,
        );

        if chap_after.is_none() && chap_before.is_none() {
            for i in (0..index).rev() {
                let chap = chapter_from_uri(&self.spine[i].path, toc);
                if chap.is_some() {
                    end_offset = if let Some(j) = (index + 1..self.spine.len())
                        .find(|&j| chapter_from_uri(&self.spine[j].path, toc).is_some())
                    {
                        self.offset(j)
                    } else {
                        self.size()
                    };
                    let chap_offset = self.offset(i);
                    let progress =
                        (offset - chap_offset) as f32 / (end_offset - chap_offset) as f32;
                    return chap.zip(Some(progress));
                }
            }
            None
        } else {
            match (chap_after, chap_before) {
                (Some(..), _) => chap_after.zip(Some(0.0)),
                (None, Some(..)) => chap_before.zip(Some(
                    (offset - offset_before) as f32 / (end_offset - offset_before) as f32,
                )),
                _ => None,
            }
        }
    }

    fn chapter_relative<'a>(
        &mut self,
        offset: usize,
        dir: CycleDir,
        toc: &'a [TocEntry],
    ) -> Option<&'a TocEntry> {
        let next_offset = self
            .resolve_location(Location::Next(offset))
            .unwrap_or(usize::MAX);
        let chap = self.chapter(offset, toc).map(|(c, _)| c);

        match dir {
            CycleDir::Previous => self.previous_chapter(chap, offset, next_offset, toc),
            CycleDir::Next => self.next_chapter(chap, offset, next_offset, toc),
        }
    }

    fn resolve_location(&mut self, loc: Location) -> Option<usize> {
        self.engine.load_fonts();

        match loc {
            Location::Exact(offset) => {
                let (index, start_offset) = self.vertebra_coordinates(offset)?;
                let page_index = self.page_index(offset, index, start_offset)?;
                self.cache
                    .get(&index)
                    .and_then(|display_list| display_list[page_index].first())
                    .map(DrawCommand::offset)
            }
            Location::Previous(offset) => {
                let (index, start_offset) = self.vertebra_coordinates(offset)?;
                let page_index = self.page_index(offset, index, start_offset)?;
                if page_index > 0 {
                    self.cache.get(&index).and_then(|display_list| {
                        display_list[page_index - 1]
                            .first()
                            .map(DrawCommand::offset)
                    })
                } else {
                    if index == 0 {
                        return None;
                    }
                    let (index, start_offset) =
                        (index - 1, start_offset - self.spine[index - 1].size);
                    if !self.cache.contains_key(&index) {
                        let display_list = self.build_display_list(index, start_offset);
                        self.cache.insert(index, display_list);
                    }
                    self.cache.get(&index).and_then(|display_list| {
                        display_list
                            .last()
                            .and_then(|page| page.first())
                            .map(DrawCommand::offset)
                    })
                }
            }
            Location::Next(offset) => {
                let (index, start_offset) = self.vertebra_coordinates(offset)?;
                let page_index = self.page_index(offset, index, start_offset)?;
                if page_index < self.cache.get(&index).map(Vec::len)? - 1 {
                    self.cache.get(&index).and_then(|display_list| {
                        display_list[page_index + 1]
                            .first()
                            .map(DrawCommand::offset)
                    })
                } else {
                    if index == self.spine.len() - 1 {
                        return None;
                    }
                    let (index, start_offset) = (index + 1, start_offset + self.spine[index].size);
                    if !self.cache.contains_key(&index) {
                        let display_list = self.build_display_list(index, start_offset);
                        self.cache.insert(index, display_list);
                    }
                    self.cache.get(&index).and_then(|display_list| {
                        display_list
                            .first()
                            .and_then(|page| page.first())
                            .map(|dc| dc.offset())
                    })
                }
            }
            Location::LocalUri(offset, ref uri) => {
                let mut cache = FxHashMap::default();
                let normalized_uri: String = {
                    let (index, _) = self.vertebra_coordinates(offset)?;
                    let path = &self.spine[index].path;
                    if uri.starts_with('#') {
                        format!("{}{}", path, uri)
                    } else {
                        let parent = Path::new(path).parent().unwrap_or_else(|| Path::new(""));
                        parent.join(uri).normalize().to_string_lossy().into_owned()
                    }
                };
                self.resolve_link(&normalized_uri, &mut cache)
            }
            Location::Uri(ref uri) => {
                let mut cache = FxHashMap::default();
                self.resolve_link(uri, &mut cache)
            }
        }
    }

    fn words(&mut self, loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        if self.spine.is_empty() {
            return None;
        }

        let offset = self.resolve_location(loc)?;
        let (index, start_offset) = self.vertebra_coordinates(offset)?;
        let page_index = self.page_index(offset, index, start_offset)?;

        self.cache.get(&index).map(|display_list| {
            (
                display_list[page_index]
                    .iter()
                    .filter_map(|dc| match dc {
                        DrawCommand::Text(TextCommand {
                            text,
                            rect,
                            offset: _,
                            ..
                        }) => {
                            let bounds: Boundary = (*rect).into();
                            Some(BoundedText {
                                text: text.clone(),
                                rect: bounds,
                                location: bounds.min.into(),
                            })
                        }
                        _ => None,
                    })
                    .collect(),
                offset,
            )
        })
    }

    fn lines(&mut self, _loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        None
    }

    fn links(&mut self, loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        if self.spine.is_empty() {
            return None;
        }

        let offset = self.resolve_location(loc)?;
        let (index, start_offset) = self.vertebra_coordinates(offset)?;
        let page_index = self.page_index(offset, index, start_offset)?;

        self.cache.get(&index).map(|display_list| {
            (
                display_list[page_index]
                    .iter()
                    .filter_map(|dc| match dc {
                        DrawCommand::Text(TextCommand {
                            uri, rect, offset, ..
                        })
                        | DrawCommand::Image(ImageCommand {
                            uri, rect, offset, ..
                        }) if uri.is_some() => {
                            let bounds: Boundary = (*rect).into();
                            Some(BoundedText {
                                text: uri.clone().expect("URI is missing"),
                                rect: bounds,
                                location: bounds.min.into(),
                            })
                        }
                        _ => None,
                    })
                    .collect(),
                offset,
            )
        })
    }

    fn images(&mut self, loc: Location) -> Option<(Vec<Boundary>, usize)> {
        if self.spine.is_empty() {
            return None;
        }

        let offset = self.resolve_location(loc)?;
        let (index, start_offset) = self.vertebra_coordinates(offset)?;
        let page_index = self.page_index(offset, index, start_offset)?;

        self.cache.get(&index).map(|display_list| {
            (
                display_list[page_index]
                    .iter()
                    .filter_map(|dc| match dc {
                        DrawCommand::Image(ImageCommand { rect, .. }) => Some((*rect).into()),
                        _ => None,
                    })
                    .collect(),
                offset,
            )
        })
    }

    fn pixmap(&mut self, loc: Location, scale: f32, samples: usize) -> Option<(Pixmap, usize)> {
        if self.spine.is_empty() {
            return None;
        }

        let offset = self.resolve_location(loc)?;
        let (index, start_offset) = self.vertebra_coordinates(offset)?;

        let page_index = self.page_index(offset, index, start_offset)?;
        let page = self.cache.get(&index)?.get(page_index)?.clone();

        let pixmap = self
            .engine
            .render_page(&page, scale, samples, &mut self.archive)?;

        Some((pixmap, offset))
    }

    fn layout(&mut self, width: u32, height: u32, font_size: f32, dpi: u16) {
        self.engine.layout(width, height, font_size, dpi);
        self.cache.clear();
    }

    fn set_text_align(&mut self, text_align: TextAlign) {
        self.engine.set_text_align(text_align);
        self.cache.clear();
    }

    fn set_font_family(&mut self, family_name: &str, search_path: &str) {
        self.engine.set_font_family(family_name, search_path);
        self.cache.clear();
    }

    fn set_margin_width(&mut self, width: i32) {
        self.engine.set_margin_width(width);
        self.cache.clear();
    }

    fn set_line_height(&mut self, line_height: f32) {
        self.engine.set_line_height(line_height);
        self.cache.clear();
    }

    fn set_hyphen_penalty(&mut self, hyphen_penalty: i32) {
        self.engine.set_hyphen_penalty(hyphen_penalty);
        self.cache.clear();
    }

    fn set_stretch_tolerance(&mut self, stretch_tolerance: f32) {
        self.engine.set_stretch_tolerance(stretch_tolerance);
        self.cache.clear();
    }

    fn set_ignore_document_css(&mut self, ignore: bool) {
        self.ignore_document_css = ignore;
        self.cache.clear();
    }

    fn title(&self) -> Option<String> {
        self.metadata("dc:title")
    }

    fn author(&self) -> Option<String> {
        if let Some(root) = self.info.root().find("metadata") {
            if let Some(md) = root.find("dc:creator") {
                if let Some(file_as) = md.attribute("file-as") {
                    return Some(file_as.to_string());
                }
            }
        }
        self.metadata("dc:creator")
    }

    fn metadata(&self, key: &str) -> Option<String> {
        self.info
            .root()
            .find("metadata")
            .and_then(|md| {
                md.children()
                    .find(|child| child.tag_qualified_name() == Some(key))
            })
            .map(|child| decode_entities(&child.text()).into_owned())
    }

    fn is_reflowable(&self) -> bool {
        true
    }

    fn has_synthetic_page_numbers(&self) -> bool {
        true
    }
}
