use crate::framebuffer::{Framebuffer, Pixmap};
use crate::geom::Point;
use crate::geom::{LinearDir, Rectangle};
use crate::unit::scale_by_dpi;

#[derive(Debug, Clone, Copy)]
pub enum PageAnimKind {
    Slide,
    Fade,
    Flip,
}

#[derive(Debug, Clone)]
pub struct PageAnimation {
    pub kind: PageAnimKind,
    pub direction: LinearDir,
    pub progress: f32,
}

impl PageAnimation {
    pub fn new(kind: PageAnimKind, direction: LinearDir) -> Self {
        PageAnimation {
            kind,
            direction,
            progress: 0.0,
        }
    }

    pub fn advance(&mut self, delta: f32) -> bool {
        self.progress += delta;
        if self.progress >= 1.0 {
            self.progress = 1.0;
            false
        } else {
            true
        }
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    pub fn render_previous_page(
        &self,
        fb: &mut dyn Framebuffer,
        pixmap: &Pixmap,
        frame: &Rectangle,
        position: Point,
        screen_rect: Rectangle,
    ) {
        let chunk_rect = frame - frame.min + position;

        if let Some(region_rect) = screen_rect.intersection(&chunk_rect) {
            let chunk_frame = region_rect - position + frame.min;
            let chunk_position = region_rect.min;

            match self.kind {
                PageAnimKind::Slide => {
                    self.render_slide_animation(fb, pixmap, &chunk_frame, position, screen_rect)
                }
                PageAnimKind::Fade => {
                    self.render_fade_animation(fb, pixmap, &chunk_frame, chunk_position)
                }
                PageAnimKind::Flip => {
                    self.render_flip_animation(fb, pixmap, &chunk_frame, position, screen_rect)
                }
            }
        }
    }

    fn render_slide_animation(
        &self,
        fb: &mut dyn Framebuffer,
        pixmap: &Pixmap,
        chunk_frame: &Rectangle,
        position: Point,
        screen_rect: Rectangle,
    ) {
        let adjusted_position = self.calculate_adjusted_position(position, screen_rect.width());
        let alpha = (1.0 - self.progress) as u8;
        fb.draw_framed_pixmap_contrast_transparent(
            pixmap,
            chunk_frame,
            adjusted_position,
            1.0,
            0.5,
            alpha,
        );
    }

    fn render_fade_animation(
        &self,
        fb: &mut dyn Framebuffer,
        pixmap: &Pixmap,
        chunk_frame: &Rectangle,
        chunk_position: Point,
    ) {
        let alpha = ((1.0 - self.progress) * 255.0) as u8;
        fb.draw_framed_pixmap_contrast_transparent(
            pixmap,
            chunk_frame,
            chunk_position,
            1.0,
            0.5,
            alpha,
        );
    }

    fn render_flip_animation(
        &self,
        fb: &mut dyn Framebuffer,
        pixmap: &Pixmap,
        chunk_frame: &Rectangle,
        position: Point,
        screen_rect: Rectangle,
    ) {
        let adjusted_position = self.calculate_adjusted_position(position, screen_rect.width());
        let alpha = ((1.0 - self.progress * 0.5) * 255.0) as u8;
        fb.draw_framed_pixmap_contrast_transparent(
            pixmap,
            chunk_frame,
            adjusted_position,
            1.0,
            0.5,
            alpha,
        );
    }

    fn calculate_adjusted_position(&self, position: Point, width: u32) -> Point {
        let offset = (self.progress * width as f32) as i32;
        if matches!(self.direction, LinearDir::Forward) {
            Point::new(position.x - offset, position.y)
        } else {
            Point::new(position.x + offset, position.y)
        }
    }
}
