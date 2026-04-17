mod bottom_bar;
mod display;
mod events;
mod lookup;

use crate::anyhow::Error;
use crate::color::BLACK;
use crate::context::Context;
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
        let dpi = crate::unit::get_device_dpi();
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);

        Self::add_top_bar(&mut children, rect, small_height, small_thickness, context);
        Self::add_separator(
            &mut children,
            rect,
            small_height,
            small_thickness,
            big_thickness,
            0,
        );
        Self::add_search_bar(
            &mut children,
            rect,
            small_height,
            big_thickness,
            small_thickness,
            query,
            context,
        );
        Self::add_separator(
            &mut children,
            rect,
            small_height,
            small_thickness,
            big_thickness,
            2,
        );

        let target = Self::find_target_dictionary(context, language);
        let image_rect =
            Self::calculate_image_rect(rect, small_height, big_thickness, small_thickness);

        let _ = Self::add_image(&mut children, image_rect);
        let doc = Self::create_document(image_rect, context, dpi);
        Self::add_separator(
            &mut children,
            rect,
            small_height,
            small_thickness,
            big_thickness,
            -1,
        );
        Self::add_bottom_bar(&mut children, rect, small_height, big_thickness, &target);

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

    fn add_top_bar(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        small_height: i32,
        small_thickness: i32,
        context: &mut Context,
    ) {
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
        children.push(Box::new(top_bar));
    }

    fn add_separator(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        small_height: i32,
        small_thickness: i32,
        big_thickness: i32,
        offset: i32,
    ) {
        let y_offset = if offset >= 0 {
            rect.min.y + offset * small_height - small_thickness
        } else {
            rect.max.y + offset * small_height - small_thickness
        };
        let separator = Filler::new(
            rect![
                rect.min.x,
                y_offset,
                rect.max.x,
                y_offset + small_thickness + big_thickness
            ],
            BLACK,
        );
        children.push(Box::new(separator));
    }

    fn add_search_bar(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        small_height: i32,
        big_thickness: i32,
        small_thickness: i32,
        query: &str,
        context: &mut Context,
    ) {
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
        children.push(Box::new(search_bar));
    }

    fn find_target_dictionary(context: &Context, language: &str) -> Option<String> {
        let langs = &context.settings.dictionary.languages;
        let matches = context
            .dictionaries
            .keys()
            .filter(|&k| langs.contains_key(k) && langs[k].contains(&language.to_string()))
            .collect::<Vec<&String>>();
        if matches.len() == 1 {
            Some(matches[0].clone())
        } else if context.dictionaries.len() == 1 {
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
    }

    fn calculate_image_rect(
        rect: Rectangle,
        small_height: i32,
        big_thickness: i32,
        small_thickness: i32,
    ) -> Rectangle {
        rect![
            rect.min.x,
            rect.min.y + 2 * small_height + big_thickness,
            rect.max.x,
            rect.max.y - small_height - small_thickness
        ]
    }

    fn add_image(children: &mut Vec<Box<dyn View>>, image_rect: Rectangle) -> Result<(), Error> {
        let image = Image::new(image_rect, Pixmap::new(1, 1, 1)?);
        children.push(Box::new(image));
        Ok(())
    }

    fn create_document(image_rect: Rectangle, context: &Context, dpi: u16) -> HtmlDocument {
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
        doc
    }

    fn add_bottom_bar(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        small_height: i32,
        big_thickness: i32,
        target: &Option<String>,
    ) {
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
        children.push(Box::new(bottom_bar));
    }
}
