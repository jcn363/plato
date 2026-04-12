use crate::color::WHITE;
use crate::context::Context;
use crate::cover_editor::{self, CoverEditor as CoverEditorLib};
use crate::device::CURRENT_DEVICE;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, Pixmap, UpdateMode};
use crate::geom::Rectangle;
use crate::input::{DeviceEvent, FingerStatus};
use crate::settings::CoverEditorSettings;
use crate::unit::scale_by_dpi;
use crate::view::entries::EntryId;
use crate::view::top_bar::TopBar;
use crate::view::SMALL_BAR_HEIGHT;
use crate::view::{Bus, Event, Hub, Id, RenderData, RenderQueue, View, ID_FEEDER};
use anyhow::Error;
use image::{DynamicImage, GenericImageView};
use std::path::PathBuf;

#[derive(Clone, PartialEq)]
enum EditorMode {
    SelectBook,
    EditCover,
    CropMode,
}

#[derive(Clone, PartialEq)]
enum CropState {
    None,
    Selecting { start: (i32, i32), end: (i32, i32) },
}

pub struct CoverEditorView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    cover_editor: CoverEditorLib,
    #[allow(dead_code)]
    _settings: CoverEditorSettings,
    mode: EditorMode,
    crop_state: CropState,
    current_image: Option<DynamicImage>,
    book_path: Option<PathBuf>,
}

impl CoverEditorView {
    pub fn new(rect: Rectangle, rq: &mut RenderQueue, context: &mut Context) -> CoverEditorView {
        let id = ID_FEEDER.next();
        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;

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

        rq.add(RenderData::new(id, rect, UpdateMode::Full));

        CoverEditorView {
            id,
            rect,
            children,
            cover_editor: CoverEditorLib::new(&context.settings.cover_editor),
            _settings: context.settings.cover_editor.clone(),
            mode: EditorMode::SelectBook,
            crop_state: CropState::None,
            current_image: None,
            book_path: None,
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
        self.book_path = Some(path);
        self.mode = EditorMode::EditCover;
        Ok(())
    }

    fn enter_crop_mode(&mut self, rq: &mut RenderQueue) {
        self.mode = EditorMode::CropMode;
        self.crop_state = CropState::None;
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    fn apply_crop_rect(&mut self, rq: &mut RenderQueue, rect: Rectangle) {
        if let Some(ref mut img) = self.current_image {
            let (img_w, img_h): (u32, u32) = img.dimensions();
            let _target_w = self.cover_editor.get_cover_dimensions().0;
            let _target_h = self.cover_editor.get_cover_dimensions().1;
            let scale_x = img_w as f32 / self.rect.width() as f32;
            let scale_y = img_h as f32 / self.rect.height() as f32;
            let x = ((rect.min.x as f32 * scale_x) as u32).min(img_w.saturating_sub(1));
            let y = ((rect.min.y as f32 * scale_y) as u32).min(img_h.saturating_sub(1));
            let w = ((rect.width() as f32 * scale_x) as u32)
                .max(1)
                .min(img_w.saturating_sub(x));
            let h = ((rect.height() as f32 * scale_y) as u32)
                .max(1)
                .min(img_h.saturating_sub(y));
            if w > 0 && h > 0 {
                *img = self.cover_editor.crop(img, x, y, w, h);
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
            }
        }
        self.mode = EditorMode::EditCover;
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

    #[allow(dead_code)]
    fn apply_brightness(&mut self, rq: &mut RenderQueue, value: i32) {
        if let Some(ref mut img) = self.current_image {
            *img = self.cover_editor.adjust_brightness(img, value);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    #[allow(dead_code)]
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
        if let (Some(ref img), Some(ref book_path)) = (&self.current_image, &self.book_path) {
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
                if self.mode == EditorMode::CropMode {
                    self.mode = EditorMode::EditCover;
                    self.crop_state = CropState::None;
                    rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                    return true;
                }
                if self.mode == EditorMode::EditCover {
                    self.mode = EditorMode::SelectBook;
                    self.current_image = None;
                    self.book_path = None;
                    rq.add(RenderData::new(self.id, self.rect, UpdateMode::Full));
                    return true;
                }
                return false;
            }
            Event::Select(EntryId::CoverRotate90) => {
                self.apply_rotate(rq, 90);
                return true;
            }
            Event::Select(EntryId::CoverRotate180) => {
                self.apply_rotate(rq, 180);
                return true;
            }
            Event::Select(EntryId::CoverRotate270) => {
                self.apply_rotate(rq, 270);
                return true;
            }
            Event::Select(EntryId::CoverGrayscale) => {
                self.apply_grayscale(rq);
                return true;
            }
            Event::Select(EntryId::CoverCrop) => {
                self.enter_crop_mode(rq);
                return true;
            }
            Event::Select(EntryId::CoverSave) => {
                if let Err(e) = self.save_cover(rq) {
                    bus.push_back(Event::Render(format!("Save error: {}", e)));
                }
                return true;
            }
            Event::Device(DeviceEvent::Finger {
                status, position, ..
            }) => {
                if self.mode == EditorMode::CropMode {
                    match status {
                        FingerStatus::Down => {
                            self.crop_state = CropState::Selecting {
                                start: (position.x, position.y),
                                end: (position.x, position.y),
                            };
                            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                            return true;
                        }
                        FingerStatus::Motion => {
                            if let CropState::Selecting { start, .. } = &self.crop_state {
                                self.crop_state = CropState::Selecting {
                                    start: *start,
                                    end: (position.x, position.y),
                                };
                                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                                return true;
                            }
                        }
                        FingerStatus::Up => {
                            if let CropState::Selecting { start, end } = &self.crop_state {
                                let x0 = start.0.min(end.0);
                                let y0 = start.1.min(end.1);
                                let x1 = start.0.max(end.0);
                                let y1 = start.1.max(end.1);
                                if x1 - x0 > 10 && y1 - y0 > 10 {
                                    let crop_rect = Rectangle::new(pt!(x0, y0), pt!(x1, y1));
                                    self.apply_crop_rect(rq, crop_rect);
                                }
                                self.crop_state = CropState::None;
                                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                                return true;
                            }
                        }
                    }
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

        if let EditorMode::EditCover | EditorMode::CropMode = self.mode {
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
    fn drop(&mut self) {}
}
