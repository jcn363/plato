use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::document::{Document, Location};
use crate::framebuffer::UpdateMode;
use crate::geom::{halves, CycleDir, Point};
use crate::log_error;
use crate::unit::scale_by_dpi;
use crate::view::dictionary::Dictionary;
use crate::view::image::Image;
use crate::view::search_bar::SearchBar;
use crate::view::{RenderData, RenderQueue, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};

use super::bottom_bar::BottomBar;

pub fn query_to_content(
    query: &str,
    language: &String,
    fuzzy: bool,
    target: Option<&String>,
    context: &mut Context,
) -> String {
    let mut content = String::new();

    for (name, dict) in context.dictionaries.iter_mut() {
        if target.is_some() && target != Some(name) {
            continue;
        }

        if target.is_none()
            && !language.is_empty()
            && context.settings.dictionary.languages.contains_key(name)
            && !context.settings.dictionary.languages[name].contains(language)
        {
            continue;
        }

        if let Some(results) = dict
            .lookup(query, fuzzy)
            .map_err(|e| log_error!("Can't search dictionary: {:#}.", e))
            .ok()
            .filter(|r| !r.is_empty())
        {
            if target.is_none() {
                content.push_str(&format!(
                    "<h1 class=\"dictname\">{}</h1>\n",
                    name.replace('<', "&lt;").replace('>', "&gt;")
                ));
            }
            for [head, body] in results {
                if !body.trim_start().starts_with("<h2") {
                    content.push_str(&format!(
                        "<h2 class=\"headword\">{}</h2>\n",
                        head.replace('<', "&lt;").replace('>', "&gt;")
                    ));
                }
                if body.trim_start().starts_with('<') {
                    content.push_str(&body);
                } else {
                    content.push_str(&format!(
                        "<pre>{}</pre>",
                        body.replace('<', "&lt;").replace('>', "&gt;")
                    ));
                }
            }
        }
    }

    if content.is_empty() {
        if context.dictionaries.is_empty() {
            content.push_str("<p class=\"info\">No dictionaries present.</p>");
        } else {
            content.push_str("<p class=\"info\">No definitions found.</p>");
        }
    }

    content
}

impl Dictionary {
    pub fn define(&mut self, text: Option<&str>, rq: &mut RenderQueue, context: &mut Context) {
        if let Some(query) = text {
            self.query = query.to_string();
            if let Some(search_bar) = self.children[2].downcast_mut::<SearchBar>() {
                search_bar.set_text(query, rq, context);
            }
        }
        let content = query_to_content(
            &self.query,
            &self.language,
            self.fuzzy,
            self.target.as_ref(),
            context,
        );
        self.doc.update(&content);
        if let Some(image) = self.children[4].downcast_mut::<Image>() {
            if let Some((pixmap, loc)) =
                self.doc
                    .pixmap(Location::Exact(0), 1.0, CURRENT_DEVICE.color_samples())
            {
                image.update(pixmap, rq);
                self.location = loc;
            }
        }
        if let Some(bottom_bar) = self.children[6].downcast_mut::<BottomBar>() {
            bottom_bar.update_icons(
                false,
                self.doc
                    .resolve_location(Location::Next(self.location))
                    .is_some(),
                rq,
            );
        }
    }

    pub fn go_to_neighbor(&mut self, dir: CycleDir, rq: &mut RenderQueue) {
        let location = match dir {
            CycleDir::Previous => Location::Previous(self.location),
            CycleDir::Next => Location::Next(self.location),
        };
        if let Some(image) = self.children[4].downcast_mut::<Image>() {
            if let Some((pixmap, loc)) =
                self.doc
                    .pixmap(location, 1.0, CURRENT_DEVICE.color_samples())
            {
                image.update(pixmap, rq);
                self.location = loc;
            }
        }
        if let Some(bottom_bar) = self.children[6].downcast_mut::<BottomBar>() {
            bottom_bar.update_icons(
                self.doc
                    .resolve_location(Location::Previous(self.location))
                    .is_some(),
                self.doc
                    .resolve_location(Location::Next(self.location))
                    .is_some(),
                rq,
            );
        }
    }

    pub fn underlying_word(&mut self, pt: Point) -> Option<String> {
        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (_, big_thickness) = halves(thickness);
        let offset = pt!(
            self.rect.min.x,
            self.rect.min.y + 2 * small_height + big_thickness
        );

        if let Some((words, _)) = self.doc.words(Location::Exact(self.location)) {
            for word in words {
                let rect = word.rect.to_rect() + offset;
                if rect.includes(pt) {
                    return Some(word.text);
                }
            }
        }

        None
    }

    pub fn follow_link(&mut self, pt: Point, rq: &mut RenderQueue, context: &mut Context) {
        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (_, big_thickness) = halves(thickness);
        let offset = pt!(
            self.rect.min.x,
            self.rect.min.y + 2 * small_height + big_thickness
        );

        if let Some((links, _)) = self.doc.links(Location::Exact(self.location)) {
            for link in links {
                let rect = link.rect.to_rect() + offset;
                if rect.includes(pt) && link.text.starts_with('?') {
                    self.define(Some(&link.text[1..]), rq, context);
                    return;
                }
            }
        }

        let half_width = self.rect.width() as i32 / 2;
        if pt.x - offset.x < half_width {
            self.go_to_neighbor(CycleDir::Previous, rq);
        } else {
            self.go_to_neighbor(CycleDir::Next, rq);
        }
    }

    pub fn reseed(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        use crate::view::top_bar::TopBar;
        use crate::view::View;

        if let Some(top_bar) = self.child_mut(0).downcast_mut::<TopBar>() {
            top_bar.reseed(rq, context);
        }

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }
}
