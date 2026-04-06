use std::fmt::{self, Debug};

use downcast_rs::{impl_downcast, Downcast};

use super::events::{Bus, Hub};
use super::rendering::{RenderData, RenderQueue};
use super::Event;
use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;

use super::identifiers::{Id, ViewId};

#[macro_export]
macro_rules! impl_view_boilerplate {
    () => {
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
    };
}

pub trait View: Downcast {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool;
    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts);
    fn rect(&self) -> &Rectangle;
    fn rect_mut(&mut self) -> &mut Rectangle;
    fn children(&self) -> &Vec<Box<dyn View>>;
    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>>;
    fn id(&self) -> Id;

    fn render_rect(&self, _rect: &Rectangle) -> Rectangle {
        *self.rect()
    }

    fn resize(
        &mut self,
        rect: Rectangle,
        _hub: &Hub,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        *self.rect_mut() = rect;
    }

    fn child(&self, index: usize) -> &dyn View {
        self.children()[index].as_ref()
    }

    fn child_mut(&mut self, index: usize) -> &mut dyn View {
        self.children_mut()[index].as_mut()
    }

    fn len(&self) -> usize {
        self.children().len()
    }

    fn might_skip(&self, _evt: &Event) -> bool {
        false
    }

    fn might_rotate(&self) -> bool {
        true
    }

    fn is_background(&self) -> bool {
        false
    }

    fn view_id(&self) -> Option<ViewId> {
        None
    }

    /// Queue this view for rendering with the specified update mode.
    ///
    /// This convenience method eliminates boilerplate by automatically creating
    /// a RenderData entry with this view's ID and rectangle.
    ///
    /// # Arguments
    ///
    /// * `rq` - The render queue to add this render operation to
    /// * `mode` - The update mode (e.g., Gui, Partial, Full)
    fn queue_render(&self, rq: &mut RenderQueue, mode: UpdateMode) {
        rq.add(RenderData::new(self.id(), *self.rect(), mode));
    }

    /// Queue a child view at the specified index for rendering.
    ///
    /// This convenience method eliminates boilerplate by automatically looking up
    /// the child and creating a RenderData entry with its ID and rectangle.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the child view to render
    /// * `rq` - The render queue to add this render operation to
    /// * `mode` - The update mode (e.g., Gui, Partial, Full)
    fn queue_child_render(&self, index: usize, rq: &mut RenderQueue, mode: UpdateMode) {
        if let Some(child) = self.children().get(index) {
            rq.add(RenderData::new(child.id(), *child.rect(), mode));
        }
    }
}

impl_downcast!(View);

impl Debug for Box<dyn View> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Box<dyn View>")
    }
}
