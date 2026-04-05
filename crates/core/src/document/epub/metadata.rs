use crate::document::{Document, Location, TocEntry};
use crate::helpers::decode_entities;
use std::collections::BTreeSet;

use super::opener::EpubDocument;

impl EpubDocument {
    pub fn categories(&self) -> BTreeSet<String> {
        let mut result = BTreeSet::new();

        if let Some(md) = self.info.root().find("metadata") {
            for child in md.children() {
                if child.tag_qualified_name() == Some("dc:subject") {
                    let text = child.text();
                    let subject = decode_entities(&text);
                    if subject.contains(" / ") {
                        for categ in subject.split('|') {
                            let start_index = if let Some(index) = categ.find(" - ") {
                                index + 3
                            } else {
                                0
                            };
                            result.insert(categ[start_index..].trim().replace(" / ", "."));
                        }
                    } else {
                        result.insert(subject.into_owned());
                    }
                }
            }
        }

        result
    }

    #[allow(clippy::too_many_arguments)]
    pub fn chapter_aux<'a>(
        &mut self,
        toc: &'a [TocEntry],
        offset: usize,
        next_offset: usize,
        path: &str,
        end_offset: &mut usize,
        chap_before: &mut Option<&'a TocEntry>,
        offset_before: &mut usize,
        chap_after: &mut Option<&'a TocEntry>,
        offset_after: &mut usize,
    ) {
        for entry in toc {
            if let Location::Uri(ref uri) = entry.location {
                if uri.starts_with(path) {
                    if let Some(entry_offset) = self.resolve_location(entry.location.clone()) {
                        if entry_offset < offset
                            && (chap_before.is_none() || entry_offset > *offset_before)
                        {
                            *chap_before = Some(entry);
                            *offset_before = entry_offset;
                        }
                        if entry_offset >= offset
                            && entry_offset < next_offset
                            && (chap_after.is_none() || entry_offset < *offset_after)
                        {
                            *chap_after = Some(entry);
                            *offset_after = entry_offset;
                        }
                        if entry_offset >= next_offset && entry_offset < *end_offset {
                            *end_offset = entry_offset;
                        }
                    }
                }
            }
            self.chapter_aux(
                &entry.children,
                offset,
                next_offset,
                path,
                end_offset,
                chap_before,
                offset_before,
                chap_after,
                offset_after,
            );
        }
    }

    pub fn previous_chapter<'a>(
        &mut self,
        chap: Option<&TocEntry>,
        start_offset: usize,
        end_offset: usize,
        toc: &'a [TocEntry],
    ) -> Option<&'a TocEntry> {
        for entry in toc.iter().rev() {
            let result = self.previous_chapter(chap, start_offset, end_offset, &entry.children);
            if result.is_some() {
                return result;
            }

            if let Some(chap) = chap {
                if entry.index < chap.index {
                    let entry_offset = self.resolve_location(entry.location.clone())?;
                    if entry_offset < start_offset || entry_offset >= end_offset {
                        return Some(entry);
                    }
                }
            } else {
                let entry_offset = self.resolve_location(entry.location.clone())?;
                if entry_offset < start_offset {
                    return Some(entry);
                }
            }
        }
        None
    }

    pub fn next_chapter<'a>(
        &mut self,
        chap: Option<&TocEntry>,
        start_offset: usize,
        end_offset: usize,
        toc: &'a [TocEntry],
    ) -> Option<&'a TocEntry> {
        for entry in toc {
            if let Some(chap) = chap {
                if entry.index > chap.index {
                    let entry_offset = self.resolve_location(entry.location.clone())?;
                    if entry_offset < start_offset || entry_offset >= end_offset {
                        return Some(entry);
                    }
                }
            } else {
                let entry_offset = self.resolve_location(entry.location.clone())?;
                if entry_offset >= end_offset {
                    return Some(entry);
                }
            }

            let result = self.next_chapter(chap, start_offset, end_offset, &entry.children);
            if result.is_some() {
                return result;
            }
        }
        None
    }

    pub fn series(&self) -> Option<(String, String)> {
        self.info.root().find("metadata").and_then(|md| {
            let mut title = None;
            let mut index = None;

            for child in md.children() {
                if child.tag_name() == Some("meta") {
                    if child.attribute("name") == Some("calibre:series") {
                        title = child
                            .attribute("content")
                            .map(|s| decode_entities(s).into_owned());
                    } else if child.attribute("name") == Some("calibre:series_index") {
                        index = child
                            .attribute("content")
                            .map(|s| decode_entities(s).into_owned());
                    } else if child.attribute("property") == Some("belongs-to-collection") {
                        title = Some(decode_entities(&child.text()).into_owned());
                    } else if child.attribute("property") == Some("group-position") {
                        index = Some(decode_entities(&child.text()).into_owned());
                    }
                }

                if title.is_some() && index.is_some() {
                    break;
                }
            }

            title.into_iter().zip(index).next()
        })
    }

    pub fn cover_image(&self) -> Option<&str> {
        self.info
            .root()
            .find("metadata")
            .and_then(|md| {
                md.children().find(|child| {
                    child.tag_name() == Some("meta") && child.attribute("name") == Some("cover")
                })
            })
            .and_then(|entry| entry.attribute("content"))
            .and_then(|cover_id| {
                self.info
                    .root()
                    .find("manifest")
                    .and_then(|entry| entry.find_by_id(cover_id))
                    .and_then(|entry| entry.attribute("href"))
            })
            .or_else(|| {
                self.info
                    .root()
                    .find("manifest")
                    .and_then(|mf| {
                        mf.children().find(|child| {
                            (child
                                .attribute("href")
                                .is_some_and(|hr| hr.contains("cover") || hr.contains("Cover"))
                                || child.id().is_some_and(|id| id.contains("cover")))
                                && child
                                    .attribute("media-type")
                                    .is_some_and(|mt| mt.starts_with("image/"))
                        })
                    })
                    .and_then(|entry| entry.attribute("href"))
            })
    }

    pub fn description(&self) -> Option<String> {
        self.metadata("dc:description")
    }

    pub fn publisher(&self) -> Option<String> {
        self.metadata("dc:publisher")
    }

    pub fn language(&self) -> Option<String> {
        self.metadata("dc:language")
    }

    pub fn year(&self) -> Option<String> {
        self.metadata("dc:date")
            .map(|s| s.chars().take(4).collect())
    }

    pub fn metadata(&self, key: &str) -> Option<String> {
        self.info
            .root()
            .find("metadata")
            .and_then(|md| {
                md.children()
                    .find(|child| child.tag_qualified_name() == Some(key))
            })
            .map(|child| decode_entities(&child.text()).into_owned())
    }
}
