use super::book::Book;
use crate::color::{background as bg, separator as sep};
use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::divide;
use crate::geom::{halves, CycleDir, Dir, Rectangle};
use crate::gesture::GestureEvent;
use crate::metadata::Info;
use crate::settings::{FirstColumn, SecondColumn};
use crate::theme;
use crate::thumbnail::ThumbnailManager;
use crate::unit::scale_by_dpi;
use crate::view::filler::Filler;
use crate::view::{Bus, Event, Hub, Id, RenderData, RenderQueue, View, ID_FEEDER};
use crate::view::{BIG_BAR_HEIGHT, THICKNESS_MEDIUM};
use std::path::{Path, PathBuf};

pub struct Shelf {
    id: Id,
    pub rect: Rectangle,
    children: Vec<Box<dyn View>>,
    pub max_lines: usize,
    first_column: FirstColumn,
    second_column: SecondColumn,
    thumbnail_previews: bool,
}

impl Shelf {
    pub fn new(
        rect: Rectangle,
        first_column: FirstColumn,
        second_column: SecondColumn,
        thumbnail_previews: bool,
    ) -> Shelf {
        let dpi = crate::unit::get_device_dpi();
        let big_height = scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let max_lines = ((rect.height() as i32 + thickness) / big_height) as usize;
        Shelf {
            id: ID_FEEDER.next(),
            rect,
            children: Vec::new(),
            max_lines,
            first_column,
            second_column,
            thumbnail_previews,
        }
    }

    pub fn set_first_column(&mut self, first_column: FirstColumn) {
        self.first_column = first_column;
    }

    pub fn set_second_column(&mut self, second_column: SecondColumn) {
        self.second_column = second_column;
    }

    pub fn set_thumbnail_previews(&mut self, thumbnail_previews: bool) {
        self.thumbnail_previews = thumbnail_previews;
    }

    pub fn update(
        &mut self,
        metadata: &[Info],
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &Context,
    ) {
        self.children.clear();
        let (max_lines, book_heights, thickness, big_thickness) =
            Self::calculate_layout_metrics(&self.rect);
        let mut y_pos = self.rect.min.y;

        for (index, info) in metadata.iter().enumerate() {
            let (y_min, y_max) = Self::calculate_book_rect(
                y_pos,
                index,
                &book_heights,
                max_lines,
                thickness,
                big_thickness,
            );
            let preview_path = Self::get_preview_path(info, context);
            Self::add_book(
                &mut self.children,
                &self.rect,
                y_min,
                y_max,
                info,
                index,
                preview_path,
            );
            Self::add_separator_if_needed(
                &mut self.children,
                &self.rect,
                index,
                max_lines,
                y_max,
                thickness,
            );
            y_pos += book_heights[index];
        }

        Self::add_filler_if_needed(
            &mut self.children,
            &self.rect,
            metadata.len(),
            max_lines,
            y_pos,
            thickness,
        );
        self.max_lines = max_lines;
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    fn calculate_layout_metrics(rect: &Rectangle) -> (usize, Vec<i32>, i32, i32) {
        let dpi = crate::unit::get_device_dpi();
        let big_height = scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (_small_thickness, big_thickness) = halves(thickness);
        let max_lines = ((rect.height() as i32 + thickness) / big_height) as usize;
        let book_heights = divide(rect.height() as i32, max_lines as i32);
        (max_lines, book_heights, thickness, big_thickness)
    }

    fn calculate_book_rect(
        y_pos: i32,
        index: usize,
        book_heights: &[i32],
        max_lines: usize,
        thickness: i32,
        big_thickness: i32,
    ) -> (i32, i32) {
        let y_min = y_pos + if index > 0 { big_thickness } else { 0 };
        let y_max = y_pos + book_heights[index]
            - if index < max_lines - 1 {
                thickness / 2
            } else {
                0
            };
        (y_min, y_max)
    }

    fn get_preview_path(info: &Info, context: &Context) -> Option<PathBuf> {
        if !context
            .settings
            .libraries
            .get(context.settings.selected_library)
            .map(|l| l.thumbnail_previews)
            .unwrap_or(false)
        {
            return None;
        }
        if let Some(thumbnail_manager) = context.thumbnail_manager() {
            Self::request_thumbnail_from_manager(thumbnail_manager, &info.file.path)
        } else {
            Self::get_fallback_thumbnail(context, &info.file.path)
        }
    }

    fn request_thumbnail_from_manager(
        thumbnail_manager: &ThumbnailManager,
        path: &Path,
    ) -> Option<PathBuf> {
        match thumbnail_manager.request_thumbnail(path) {
            Ok(Some(path)) => Some(path),
            Ok(None) => Some(PathBuf::default()),
            Err(e) => {
                eprintln!("Thumbnail request failed for {}: {:?}", path.display(), e);
                Some(PathBuf::default())
            }
        }
    }

    fn get_fallback_thumbnail(context: &Context, path: &Path) -> Option<PathBuf> {
        let thumb_path = context.library.thumbnail_preview(path);
        if thumb_path.exists() {
            Some(thumb_path)
        } else {
            Some(PathBuf::default())
        }
    }

    fn add_book(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        y_min: i32,
        y_max: i32,
        info: &Info,
        index: usize,
        preview_path: Option<PathBuf>,
    ) {
        let book = Book::new(
            rect![rect.min.x, y_min, rect.max.x, y_max],
            info.clone(),
            index,
            FirstColumn::default(),
            SecondColumn::default(),
            preview_path,
        );
        children.push(Box::new(book) as Box<dyn View>);
    }

    fn add_separator_if_needed(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        index: usize,
        max_lines: usize,
        y_max: i32,
        thickness: i32,
    ) {
        if index < max_lines - 1 {
            let separator = Filler::new(
                rect![rect.min.x, y_max, rect.max.x, y_max + thickness],
                sep(theme::is_dark_mode()),
            );
            children.push(Box::new(separator) as Box<dyn View>);
        }
    }

    fn add_filler_if_needed(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        metadata_len: usize,
        max_lines: usize,
        y_pos: i32,
        thickness: i32,
    ) {
        if metadata_len < max_lines {
            let y_start = y_pos + if metadata_len == 0 { 0 } else { thickness };
            let filler = Filler::new(
                rect![rect.min.x, y_start, rect.max.x, rect.max.y],
                bg(theme::is_dark_mode()),
            );
            children.push(Box::new(filler) as Box<dyn View>);
        }
    }
}

impl View for Shelf {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Gesture(GestureEvent::Swipe { dir, start, .. }) if self.rect.includes(start) => {
                match dir {
                    Dir::West => {
                        bus.push_back(Event::Page(CycleDir::Next));
                        true
                    }
                    Dir::East => {
                        bus.push_back(Event::Page(CycleDir::Previous));
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        for child in self.children().iter() {
            let child_rect = child.rect();
            if let Some(intersection) = rect.intersection(child_rect) {
                child.render(fb, intersection, fonts);
            }
        }
    }

    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        &mut self.rect
    }

    fn children(&self) -> &Vec<Box<dyn View>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> {
        &mut self.children
    }

    fn id(&self) -> Id {
        self.id
    }
}
