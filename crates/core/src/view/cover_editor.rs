use crate::color::WHITE;
use crate::context::Context;
use crate::cover_editor::{self, CoverEditor as CoverEditorLib};
use crate::device::CURRENT_DEVICE;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, Pixmap, UpdateMode};
use crate::geom::Rectangle;
use crate::settings::CoverEditorSettings;
use crate::unit::scale_by_dpi;
use crate::view::top_bar::TopBar;
use crate::view::{Bus, Event, Hub, Id, RenderData, RenderQueue, View, ID_FEEDER};
use crate::view::{BORDER_RADIUS_SMALL, SMALL_BAR_HEIGHT};
use anyhow::Error;
use image::DynamicImage;
use std::path::PathBuf;

enum EditorMode {
    SelectBook,
    EditCover,
}

#[allow(dead_code)]
pub struct CoverEditorView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    cover_editor: CoverEditorLib,
    settings: CoverEditorSettings,
    mode: EditorMode,
    current_image: Option<DynamicImage>,
    _current_book_path: Option<PathBuf>,
    _temp_path: Option<PathBuf>,
}

#[allow(dead_code)]
impl CoverEditorView {
    pub fn new(rect: Rectangle, rq: &mut RenderQueue, context: &mut Context) -> CoverEditorView {
        let id = ID_FEEDER.next();
        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let _border_radius = scale_by_dpi(BORDER_RADIUS_SMALL, dpi) as i32;

        let mut children = Vec::new();

        let top_bar_height = small_height;
        let top_bar_rect = rect![
            rect.min.x,
            rect.min.y,
            rect.max.x,
            rect.min.y + top_bar_height
        ];
        let top_bar = TopBar::new(
            top_bar_rect,
            Event::Back,
            "Cover Editor".to_string(),
            context,
        );
        children.push(Box::new(top_bar) as Box<dyn View>);

        let _content_rect = rect![
            rect.min.x,
            rect.min.y + top_bar_height,
            rect.max.x,
            rect.max.y
        ];

        rq.add(RenderData::new(id, rect, UpdateMode::Full));

        CoverEditorView {
            id,
            rect,
            children,
            cover_editor: CoverEditorLib::new(&context.settings.cover_editor),
            settings: context.settings.cover_editor.clone(),
            mode: EditorMode::SelectBook,
            current_image: None,
            _current_book_path: None,
            _temp_path: None,
        }
    }

    pub fn for_book(
        rect: Rectangle,
        path: PathBuf,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<CoverEditorView, Error> {
        let mut view = CoverEditorView::new(rect, rq, context);
        view.select_book(path)?;
        Ok(view)
    }

    pub fn select_book(&mut self, path: PathBuf) -> Result<(), Error> {
        let cover = cover_editor::extract_cover_from_epub(&path)?;
        self.current_image = Some(cover);
        self._current_book_path = Some(path);
        self.mode = EditorMode::EditCover;
        Ok(())
    }

    fn apply_crop(&mut self, rq: &mut RenderQueue, x: u32, y: u32, width: u32, height: u32) {
        if let Some(ref mut img) = self.current_image {
            *img = self.cover_editor.crop(img, x, y, width, height);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    fn apply_rotate(&mut self, rq: &mut RenderQueue, degrees: u32) {
        if let Some(ref mut img) = self.current_image {
            let rotated = match degrees {
                90 => self.cover_editor.rotate_90(img),
                180 => self.cover_editor.rotate_180(img),
                270 => self.cover_editor.rotate_270(img),
                _ => img.clone(),
            };
            *img = rotated;
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    fn apply_brightness(&mut self, rq: &mut RenderQueue, value: i32) {
        if let Some(ref mut img) = self.current_image {
            *img = self.cover_editor.adjust_brightness(img, value);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    fn apply_contrast(&mut self, rq: &mut RenderQueue, value: f32) {
        if let Some(ref mut img) = self.current_image {
            *img = self.cover_editor.adjust_contrast(img, value);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    fn apply_grayscale(&mut self, rq: &mut RenderQueue) {
        if let Some(ref mut img) = self.current_image {
            *img = self.cover_editor.grayscale(img);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    fn save_cover(&mut self, rq: &mut RenderQueue) -> Result<(), Error> {
        if let (Some(ref img), Some(ref book_path)) =
            (&self.current_image, &self._current_book_path)
        {
            let temp_cover_path = std::env::temp_dir().join("temp_cover.jpg");
            self.cover_editor.save_as_cover(img, &temp_cover_path)?;
            cover_editor::set_cover_in_epub(book_path, &temp_cover_path)?;
            std::fs::remove_file(&temp_cover_path).ok();
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
        Ok(())
    }
}

impl View for CoverEditorView {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match evt {
            Event::Back => {
                if let EditorMode::EditCover = self.mode {
                    self.mode = EditorMode::SelectBook;
                    self.current_image = None;
                    self._current_book_path = None;
                    rq.add(RenderData::new(self.id, self.rect, UpdateMode::Full));
                    return true;
                }
                return false;
            }
            _ => {}
        }
        for child in self.children_mut().iter_mut() {
            if child.handle_event(evt, hub, bus, rq, context) {
                return true;
            }
        }
        false
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        if let Some(r) = self.rect().intersection(&rect) {
            fb.draw_rectangle(&r, WHITE);
        }

        for child in self.children().iter() {
            child.render(fb, rect, fonts);
        }

        if let EditorMode::EditCover = self.mode {
            if let Some(ref img) = self.current_image {
                let (target_w, target_h) = self.cover_editor.get_cover_dimensions();
                let scaled = img.resize_to_fill(
                    target_w as u32,
                    target_h as u32,
                    image::imageops::FilterType::Lanczos3,
                );
                if let Some(pixmap) = Pixmap::from_dynamic_image(&scaled).ok() {
                    let x0 = self.rect.min.x + (self.rect.width() as i32 - pixmap.width as i32) / 2;
                    let y0 = self.rect.min.y + 100;

                    if let Some(r) = rect![
                        pt!(x0, y0),
                        pt!(x0 + pixmap.width as i32, y0 + pixmap.height as i32)
                    ]
                    .intersection(&rect)
                    {
                        let frame = r - pt!(x0, y0);
                        fb.draw_framed_pixmap(&pixmap, &frame, r.min);
                    }
                }
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

impl Drop for CoverEditorView {
    fn drop(&mut self) {
        if let Some(ref path) = self._temp_path {
            std::fs::remove_dir_all(path).ok();
        }
    }
}
