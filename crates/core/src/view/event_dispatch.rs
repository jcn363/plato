use std::collections::VecDeque;
use std::time::Instant;

use fxhash::FxHashMap;

use super::events::{Bus, Hub};
use super::identifiers::Id;
use super::rendering::{RenderQueue, UpdateData};
use super::view_trait::View;
use super::Event;
use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::Framebuffer;
use crate::geom::Rectangle;
use crate::log_error;

// We start delivering events from the highest z-level to prevent views from capturing
// gestures that occurred in higher views.
// The consistency must also be ensured by the views: popups, for example, need to
// capture any tap gesture with a touch point inside their rectangle.
// A child can send events to the main channel through the *hub* or communicate with its parent through the *bus*.
// A view that wants to render can write to the rendering queue.
pub fn handle_event(
    view: &mut dyn View,
    evt: &Event,
    hub: &Hub,
    parent_bus: &mut Bus,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if view.len() > 0 {
        let mut captured = false;

        if view.might_skip(evt) {
            return captured;
        }

        let mut child_bus: Bus = VecDeque::with_capacity(1);

        for i in (0..view.len()).rev() {
            if handle_event(view.child_mut(i), evt, hub, &mut child_bus, rq, context) {
                captured = true;
                break;
            }
        }

        let mut temp_bus: Bus = VecDeque::with_capacity(1);

        child_bus
            .retain(|child_evt| !view.handle_event(child_evt, hub, &mut temp_bus, rq, context));

        parent_bus.append(&mut child_bus);
        parent_bus.append(&mut temp_bus);

        captured || view.handle_event(evt, hub, parent_bus, rq, context)
    } else {
        view.handle_event(evt, hub, parent_bus, rq, context)
    }
}

// We render from bottom to top. For a view to render it has to either appear in `ids` or intersect
// one of the rectangles in `bgs`. When we're about to render a view, if `wait` is true, we'll wait
// for all the updates in `updating` that intersect with the view.
pub fn render(
    view: &dyn View,
    wait: bool,
    ids: &FxHashMap<Id, Vec<Rectangle>>,
    rects: &mut Vec<Rectangle>,
    bgs: &mut Vec<Rectangle>,
    fb: &mut dyn Framebuffer,
    fonts: &mut Fonts,
    updating: &mut Vec<UpdateData>,
) {
    let mut render_rects = Vec::new();

    if view.len() == 0 || view.is_background() {
        for rect in ids
            .get(&view.id())
            .cloned()
            .into_iter()
            .flatten()
            .chain(rects.iter().filter_map(|r| r.intersection(view.rect())))
            .chain(bgs.iter().filter_map(|r| r.intersection(view.rect())))
        {
            let render_rect = view.render_rect(&rect);

            if wait {
                updating.retain(|update| {
                    let overlaps = render_rect.overlaps(&update.rect);
                    if overlaps && !update.has_completed() {
                        fb.wait(update.token)
                            .map_err(|e| {
                                log_error!(
                                    "Can't wait for {}, {}: {:#}",
                                    update.token,
                                    update.rect,
                                    e
                                )
                            })
                            .ok();
                    }
                    !overlaps
                });
            }

            view.render(fb, rect, fonts);
            render_rects.push(render_rect);

            // Most views can't render a subrectangle of themselves.
            if *view.rect() == render_rect {
                break;
            }
        }
    } else {
        bgs.extend(ids.get(&view.id()).cloned().into_iter().flatten());
    }

    // Merge the contiguous zones to avoid having to schedule lots of small frambuffer updates.
    for rect in render_rects.into_iter() {
        if rects.is_empty() {
            rects.push(rect);
        } else {
            if let Some(last) = rects.last_mut() {
                if rect.extends(last) {
                    last.absorb(&rect);
                    let mut i = rects.len();
                    while i > 1 && rects[i - 1].extends(&rects[i - 2]) {
                        if let Some(rect) = rects.pop() {
                            if let Some(last) = rects.last_mut() {
                                last.absorb(&rect);
                            }
                        }
                        i -= 1;
                    }
                } else {
                    let mut i = rects.len();
                    while i > 0 && !rects[i - 1].contains(&rect) {
                        i -= 1;
                    }
                    if i == 0 {
                        rects.push(rect);
                    }
                }
            }
        }
    }

    for i in 0..view.len() {
        render(view.child(i), wait, ids, rects, bgs, fb, fonts, updating);
    }
}

#[inline]
pub fn process_render_queue(
    view: &dyn View,
    rq: &mut RenderQueue,
    context: &mut Context,
    updating: &mut Vec<UpdateData>,
) {
    for ((mode, wait), pairs) in rq.drain() {
        let mut ids = FxHashMap::default();
        let mut rects = Vec::new();
        let mut bgs = Vec::new();

        for (id, rect) in pairs.into_iter().rev() {
            if let Some(id) = id {
                ids.entry(id).or_insert_with(Vec::new).push(rect);
            } else {
                bgs.push(rect);
            }
        }

        render(
            view,
            wait,
            &ids,
            &mut rects,
            &mut bgs,
            context.fb.as_mut(),
            &mut context.fonts,
            updating,
        );

        for rect in rects {
            match context.fb.update(&rect, mode) {
                Ok(token) => {
                    updating.push(UpdateData {
                        token,
                        rect,
                        time: Instant::now(),
                    });
                }
                Err(err) => {
                    log_error!("Can't update {}: {:#}.", rect, err);
                }
            }
        }
    }
}

#[inline]
pub fn wait_for_all(updating: &mut Vec<UpdateData>, context: &mut Context) {
    for update in updating.drain(..) {
        if update.has_completed() {
            continue;
        }
        context
            .fb
            .wait(update.token)
            .map_err(|e| log_error!("Can't wait for {}, {}: {:#}", update.token, update.rect, e))
            .ok();
    }
}
