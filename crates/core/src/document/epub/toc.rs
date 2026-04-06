use crate::document::html::dom::NodeRef;
use crate::document::html::xml::XmlParser;
use crate::document::Location;
use crate::document::TocEntry;
use crate::helpers::{decode_entities, Normalize};
use percent_encoding::percent_decode_str;
use rustc_hash::FxHashMap;
use std::io::Read;
use std::path::Path;

use super::opener::EpubDocument;

type UriCache = FxHashMap<String, usize>;

impl EpubDocument {
    #[allow(clippy::only_used_in_recursion)]
    pub fn walk_toc_ncx(
        &mut self,
        node: NodeRef,
        toc_dir: &Path,
        index: &mut usize,
        cache: &mut UriCache,
    ) -> Vec<TocEntry> {
        let mut nav_points: Vec<(usize, NodeRef)> = node
            .children()
            .filter(|child| child.tag_name() == Some("navPoint"))
            .map(|child| {
                let play_order = child
                    .attribute("playOrder")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(*index + 1);
                (play_order, child)
            })
            .collect();

        nav_points.sort_by_key(|(order, _)| *order);

        let mut entries = Vec::with_capacity(nav_points.len());

        nav_points.sort_by_key(|(order, _)| *order);

        for (_, child) in nav_points {
            let title = child
                .find("navLabel")
                .and_then(|label| label.find("text"))
                .map(|text| decode_entities(&text.text()).into_owned())
                .unwrap_or_default();

            let rel_uri = child
                .find("content")
                .and_then(|content| {
                    content.attribute("src").map(|src| {
                        percent_decode_str(&decode_entities(src))
                            .decode_utf8_lossy()
                            .into_owned()
                    })
                })
                .unwrap_or_default();

            let loc = toc_dir
                .join(&rel_uri)
                .normalize()
                .to_str()
                .map(|uri| Location::Uri(uri.to_string()));

            let current_index = *index;
            *index += 1;

            let sub_entries = if child.children().count() > 2 {
                self.walk_toc_ncx(child, toc_dir, index, cache)
            } else {
                Vec::new()
            };

            if let Some(location) = loc {
                entries.push(TocEntry {
                    title,
                    location,
                    index: current_index,
                    children: sub_entries,
                });
            }
        }

        entries
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn walk_toc_nav(
        &mut self,
        node: NodeRef,
        toc_dir: &Path,
        index: &mut usize,
        cache: &mut UriCache,
    ) -> Vec<TocEntry> {
        let child_count = node
            .children()
            .filter(|c| c.tag_name() == Some("li"))
            .count();
        let mut entries = Vec::with_capacity(child_count);

        for child in node.children() {
            if child.tag_name() == Some("li") {
                let link = child.children().find(|child| child.tag_name() == Some("a"));
                let title = link
                    .map(|link| decode_entities(&link.text()).into_owned())
                    .unwrap_or_default();
                let rel_uri = link
                    .and_then(|link| {
                        link.attribute("href").map(|href| {
                            percent_decode_str(&decode_entities(href))
                                .decode_utf8_lossy()
                                .into_owned()
                        })
                    })
                    .unwrap_or_default();

                let loc = toc_dir
                    .join(&rel_uri)
                    .normalize()
                    .to_str()
                    .map(|uri| Location::Uri(uri.to_string()));

                let current_index = *index;
                *index += 1;

                let sub_entries = if let Some(sub_list) = child.find("ol") {
                    self.walk_toc_nav(sub_list, toc_dir, index, cache)
                } else {
                    Vec::new()
                };

                if let Some(location) = loc {
                    entries.push(TocEntry {
                        title,
                        location,
                        index: current_index,
                        children: sub_entries,
                    });
                }
            }
        }

        entries
    }

    pub fn resolve_link(&mut self, uri: &str, cache: &mut UriCache) -> Option<usize> {
        use crate::document::html::layout::DrawCommand;

        let frag_index_opt = uri.find('#');
        let name = &uri[..frag_index_opt.unwrap_or(uri.len())];

        let (index, start_offset) = self.vertebra_coordinates_from_name(name)?;

        if frag_index_opt.is_some() {
            let text = {
                let mut zf = self.archive.by_name(name).ok()?;
                let size = zf.size() as usize;
                let mut text = String::with_capacity(size);
                zf.read_to_string(&mut text).ok()?;
                text
            };
            let root = XmlParser::new(&text).parse();
            self.cache_uris(root.root(), name, start_offset, cache);
            cache.get(uri).cloned()
        } else {
            let page_index = self.page_index(start_offset, index, start_offset)?;
            let offset = self
                .cache
                .get(&index)
                .and_then(|display_list| display_list[page_index].first())
                .map(DrawCommand::offset)?;
            cache.insert(uri.to_string(), offset);
            Some(offset)
        }
    }

    fn cache_uris(&mut self, node: NodeRef, name: &str, start_offset: usize, cache: &mut UriCache) {
        if let Some(id) = node.attribute("id") {
            let location = start_offset + node.offset();
            cache.insert(format!("{}#{}", name, id), location);
        }
        for child in node.children() {
            self.cache_uris(child, name, start_offset, cache);
        }
    }
}
