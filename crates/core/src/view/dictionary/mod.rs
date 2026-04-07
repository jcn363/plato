mod bottom_bar;
mod display;
mod events;
mod lookup;

use crate::anyhow::Error;
use crate::color::BLACK;
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::document::html::HtmlDocument;
use crate::document::Document;
use crate::framebuffer::{Pixmap, UpdateMode};
use crate::geom::{halves, Rectangle};
use crate::unit::scale_by_dpi;
use crate::view::filler::Filler;
use crate::view::image::Image;
use crate::view::search_bar::SearchBar;
use crate::view::top_bar::TopBar;
use crate::view::{Event, Hub, RenderData, RenderQueue, View};
use crate::view::{Id, ViewId, ID_FEEDER};
use crate::view::{SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};

pub use lookup::query_to_content;

const VIEWER_STYLESHEET: &str = "css/dictionary.css";
const USER_STYLESHEET: &str = "css/dictionary-user.css";

pub struct Dictionary {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    doc: HtmlDocument,
    location: usize,
    fuzzy: bool,
    query: String,
    language: String,
    target: Option<String>,
    focus: Option<ViewId>,
}

impl Dictionary {
    pub fn new(
        rect: Rectangle,
        query: &str,
        language: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<Dictionary, Error> {
        let id = ID_FEEDER.next();
        let mut children = Vec::new();
        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);

        let top_bar = TopBar::new(
            rect![
                rect.min.x,
                rect.min.y,
                rect.max.x,
                rect.min.y + small_height - small_thickness
            ],
            Event::Back,
            "Dictionary".to_string(),
            context,
        );
        children.push(Box::new(top_bar) as Box<dyn View>);

        let separator = Filler::new(
            rect![
                rect.min.x,
                rect.min.y + small_height - small_thickness,
                rect.max.x,
                rect.min.y + small_height + big_thickness
            ],
            BLACK,
        );
        children.push(Box::new(separator) as Box<dyn View>);

        let search_bar = SearchBar::new(
            rect![
                rect.min.x,
                rect.min.y + small_height + big_thickness,
                rect.max.x,
                rect.min.y + 2 * small_height - small_thickness
            ],
            ViewId::DictionarySearchInput,
            "",
            query,
            context,
        );
        children.push(Box::new(search_bar) as Box<dyn View>);

        let separator = Filler::new(
            rect![
                rect.min.x,
                rect.min.y + 2 * small_height - small_thickness,
                rect.max.x,
                rect.min.y + 2 * small_height + big_thickness
            ],
            BLACK,
        );
        children.push(Box::new(separator) as Box<dyn View>);

        let langs = &context.settings.dictionary.languages;
        let matches = context
            .dictionaries
            .keys()
            .filter(|&k| langs.contains_key(k) && langs[k].contains(&language.to_string()))
            .collect::<Vec<&String>>();
        let target = if matches.len() == 1 {
            Some(matches[0].clone())
        } else {
            if context.dictionaries.len() == 1 {
                Some(
                    context
                        .dictionaries
                        .keys()
                        .next()
                        .cloned()
                        .unwrap_or_default(),
                )
            } else {
                None
            }
        };

        let image_rect = rect![
            rect.min.x,
            rect.min.y + 2 * small_height + big_thickness,
            rect.max.x,
            rect.max.y - small_height - small_thickness
        ];

        let image = Image::new(image_rect, Pixmap::new(1, 1, 1)?);
        children.push(Box::new(image) as Box<dyn View>);

        let mut doc = HtmlDocument::new_from_memory("");
        doc.layout(
            image_rect.width(),
            image_rect.height(),
            context.settings.dictionary.font_size,
            dpi,
        );
        doc.set_margin_width(context.settings.dictionary.margin_width);
        doc.set_viewer_stylesheet(VIEWER_STYLESHEET);
        doc.set_user_stylesheet(USER_STYLESHEET);

        let separator = Filler::new(
            rect![
                rect.min.x,
                rect.max.y - small_height - small_thickness,
                rect.max.x,
                rect.max.y - small_height + big_thickness
            ],
            BLACK,
        );
        children.push(Box::new(separator) as Box<dyn View>);

        let bottom_bar = bottom_bar::BottomBar::new(
            rect![
                rect.min.x,
                rect.max.y - small_height + big_thickness,
                rect.max.x,
                rect.max.y
            ],
            target.as_deref().unwrap_or("All"),
            false,
            false,
        );
        children.push(Box::new(bottom_bar) as Box<dyn View>);

        rq.add(RenderData::new(id, rect, UpdateMode::Gui));

        if query.is_empty() {
            hub.send(Event::Focus(Some(ViewId::DictionarySearchInput)))
                .ok();
        } else {
            hub.send(Event::Define(query.to_string())).ok();
        }

        Ok(Dictionary {
            id,
            rect,
            children,
            doc,
            location: 0,
            fuzzy: false,
            query: query.to_string(),
            language: language.to_string(),
            target,
            focus: None,
        })
    }
}
