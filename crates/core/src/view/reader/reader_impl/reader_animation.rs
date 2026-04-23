//! Page Transition Animation Rendering
//!
//! Functions for rendering page transition animations (slide, peel, fade, flip).

use super::reader_core::{AnimState, PageAnimKind, PageAnimation, RenderChunk, Resource};
use crate::framebuffer::{Framebuffer, Pixmap};
use crate::geom::{LinearDir, Point, Rectangle};

/// Contrast parameters for rendering
#[derive(Clone, Copy)]
struct ContrastParams {
    exponent: f32,
    gray: f32,
}

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
        let contrast = ContrastParams {
            exponent: contrast_exponent,
            gray: contrast_gray,
        };
        for chunk in previous_chunks {
            render_chunk_animation(cache, fb, rect, chunk, anim, contrast);
        }
    }
}

fn render_chunk_animation(
    cache: &std::collections::BTreeMap<usize, Resource>,
    fb: &mut dyn Framebuffer,
    rect: Rectangle,
    chunk: &RenderChunk,
    anim: &PageAnimation,
    contrast: ContrastParams,
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
                contrast,
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
    contrast: ContrastParams,
) {
    match anim {
        PageAnimation::None => {}
        PageAnimation::Slide(kind) => render_slide_animation(
            fb,
            pixmap,
            chunk_frame,
            chunk_position,
            kind,
            rect,
            contrast,
        ),
        PageAnimation::Peel(state) => render_peel_animation(
            fb,
            pixmap,
            chunk_frame,
            chunk_position,
            state,
            rect,
            contrast,
        ),
    }
}

fn render_slide_animation(
    fb: &mut dyn Framebuffer,
    pixmap: &Pixmap,
    chunk_frame: &Rectangle,
    chunk_position: Point,
    kind: &AnimState,
    rect: Rectangle,
    contrast: ContrastParams,
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
        contrast.exponent,
        contrast.gray,
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
    contrast: ContrastParams,
) {
    match state.kind {
        PageAnimKind::Fade => {
            render_fade_animation(fb, pixmap, chunk_frame, chunk_position, state, contrast)
        }
        PageAnimKind::Flip => render_flip_animation(
            fb,
            pixmap,
            chunk_frame,
            chunk_position,
            state,
            rect,
            contrast,
        ),
        _ => {}
    }
}

fn render_fade_animation(
    fb: &mut dyn Framebuffer,
    pixmap: &Pixmap,
    chunk_frame: &Rectangle,
    chunk_position: Point,
    state: &AnimState,
    contrast: ContrastParams,
) {
    let alpha = ((1.0 - state.progress) * 255.0) as u8;
    fb.draw_framed_pixmap_contrast_alpha(
        pixmap,
        chunk_frame,
        chunk_position,
        contrast.exponent,
        contrast.gray,
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
    contrast: ContrastParams,
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
        contrast.exponent,
        contrast.gray,
        alpha,
    );
}
