use crate::color::WHITE;
use crate::device::CURRENT_DEVICE;
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
                    let offset = (self.progress * screen_rect.width() as f32) as i32;
                    let adjusted_position = if matches!(self.direction, LinearDir::Forward) {
                        Point::new(position.x - offset, position.y)
                    } else {
                        Point::new(position.x + offset, position.y)
                    };
                    let alpha = (1.0 - self.progress) as u8;
                    fb.draw_framed_pixmap_contrast_transparent(
                        pixmap,
                        &chunk_frame,
                        adjusted_position,
                        1.0,
                        0.5,
                        alpha,
                    );
                }
                PageAnimKind::Fade => {
                    let alpha = ((1.0 - self.progress) * 255.0) as u8;
                    fb.draw_framed_pixmap_contrast_transparent(
                        pixmap,
                        &chunk_frame,
                        chunk_position,
                        1.0,
                        0.5,
                        alpha,
                    );
                }
                PageAnimKind::Flip => {
                    let offset = (self.progress * screen_rect.width() as f32) as i32;
                    let adjusted_position = if matches!(self.direction, LinearDir::Forward) {
                        Point::new(position.x - offset, position.y)
                    } else {
                        Point::new(position.x + offset, position.y)
                    };
                    let alpha = ((1.0 - self.progress * 0.5) * 255.0) as u8;
                    fb.draw_framed_pixmap_contrast_transparent(
                        pixmap,
                        &chunk_frame,
                        adjusted_position,
                        1.0,
                        0.5,
                        alpha,
                    );
                }
            }
        }
    }
}
