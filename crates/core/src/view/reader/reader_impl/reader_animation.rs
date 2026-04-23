//! Page Transition Animation Rendering
//!
//! Functions for rendering page transition animations (slide, peel, fade, flip).

use crate::framebuffer::{Framebuffer, Pixmap};
use crate::geom::{LinearDir, Point, Rectangle};
use super::reader_core::{AnimState, PageAnimKind, PageAnimation, RenderChunk, Resource};

pub(crate) fn render_animation(
    cache: &std::collections::BTreeMap<usize, Resource>,
    previous_chunks: &[RenderChunk],
    animation: &Option<PageAnimation>,
    fb: &mut dyn Framebuffer,
    rect: Rectangle,
    contrast_exponent: f32,
    contrast_gray: f32,
) {
    if let Some(ref anim) = animation {
        for chunk in previous_chunks {
            render_chunk_animation(cache, fb, rect, chunk, anim, contrast_exponent, contrast_gray);
        }
    }
}

fn render_chunk_animation(
    cache: &std::collections::BTreeMap<usize, Resource>,
    fb: &mut dyn Framebuffer,
    rect: Rectangle,
    chunk: &RenderChunk,
    anim: &PageAnimation,
    contrast_exponent: f32,
    contrast_gray: f32,
) {
    if let Some(resource) = cache.get(&chunk.location) {
        let chunk_rect = chunk.frame - chunk.frame.min + chunk.position;

        if let Some(region_rect) = rect.intersection(&chunk_rect) {
            let chunk_frame = region_rect - chunk.position + chunk.frame.min;
            let chunk_position = region_rect.min;
            let pixmap = &resource.pixmap;

            render_animation_kind(
                fb,
                pixmap,
                &chunk_frame,
                chunk_position,
                anim,
                rect,
                contrast_exponent,
                contrast_gray,
            );
        }
    }
}

fn render_animation_kind(
    fb: &mut dyn Framebuffer,
    pixmap: &Pixmap,
    chunk_frame: &Rectangle,
    chunk_position: Point,
    anim: &PageAnimation,
    rect: Rectangle,
    contrast_exponent: f32,
    contrast_gray: f32,
) {
    match anim {
        PageAnimation::None => {}
        PageAnimation::Slide(kind) => {
            render_slide_animation(
                fb,
                pixmap,
                chunk_frame,
                chunk_position,
                kind,
                rect,
                contrast_exponent,
                contrast_gray,
            )
        }
        PageAnimation::Peel(state) => {
            render_peel_animation(
                fb,
                pixmap,
                chunk_frame,
                chunk_position,
                state,
                rect,
                contrast_exponent,
                contrast_gray,
            )
        }
    }
}

fn render_slide_animation(
    fb: &mut dyn Framebuffer,
    pixmap: &Pixmap,
    chunk_frame: &Rectangle,
    chunk_position: Point,
    kind: &AnimState,
    rect: Rectangle,
    contrast_exponent: f32,
    contrast_gray: f32,
) {
    let offset = (kind.progress * rect.width() as f32) as i32;
    let adjusted_position = if matches!(kind.direction, LinearDir::Forward) {
        pt!(chunk_position.x - offset, chunk_position.y)
    } else {
        pt!(chunk_position.x + offset, chunk_position.y)
    };
    let alpha = (1.0 - kind.progress) as u8;
    fb.draw_framed_pixmap_contrast_alpha(
        pixmap,
        chunk_frame,
        adjusted_position,
        contrast_exponent,
        contrast_gray,
        alpha,
    );
}

fn render_peel_animation(
    fb: &mut dyn Framebuffer,
    pixmap: &Pixmap,
    chunk_frame: &Rectangle,
    chunk_position: Point,
    state: &AnimState,
    rect: Rectangle,
    contrast_exponent: f32,
    contrast_gray: f32,
) {
    match state.kind {
        PageAnimKind::Fade => {
            render_fade_animation(
                fb,
                pixmap,
                chunk_frame,
                chunk_position,
                state,
                contrast_exponent,
                contrast_gray,
            )
        }
        PageAnimKind::Flip => {
            render_flip_animation(
                fb,
                pixmap,
                chunk_frame,
                chunk_position,
                state,
                rect,
                contrast_exponent,
                contrast_gray,
            )
        }
        _ => {}
    }
}

fn render_fade_animation(
    fb: &mut dyn Framebuffer,
    pixmap: &Pixmap,
    chunk_frame: &Rectangle,
    chunk_position: Point,
    state: &AnimState,
    contrast_exponent: f32,
    contrast_gray: f32,
) {
    let alpha = ((1.0 - state.progress) * 255.0) as u8;
    fb.draw_framed_pixmap_contrast_alpha(
        pixmap,
        chunk_frame,
        chunk_position,
        contrast_exponent,
        contrast_gray,
        alpha,
    );
}

fn render_flip_animation(
    fb: &mut dyn Framebuffer,
    pixmap: &Pixmap,
    chunk_frame: &Rectangle,
    chunk_position: Point,
    state: &AnimState,
    rect: Rectangle,
    contrast_exponent: f32,
    contrast_gray: f32,
) {
    let offset = (state.progress * rect.width() as f32) as i32;
    let adjusted_position = if matches!(state.direction, LinearDir::Forward) {
        pt!(chunk_position.x - offset, chunk_position.y)
    } else {
        pt!(chunk_position.x + offset, chunk_position.y)
    };
    let alpha = ((1.0 - state.progress * 0.5) * 255.0) as u8;
    fb.draw_framed_pixmap_contrast_alpha(
        pixmap,
        chunk_frame,
        adjusted_position,
        contrast_exponent,
        contrast_gray,
        alpha,
    );
}
