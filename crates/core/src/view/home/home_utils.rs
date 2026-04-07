use crate::context::Context;
use crate::geom::Point;
use crate::view::{Hub, RenderQueue};
use crate::view::{View, ViewId};

/// Find the index of a child view with the specified ViewId.
pub fn find_child_index_by_view_id(children: &[Box<dyn View>], view_id: ViewId) -> Option<usize> {
    children
        .iter()
        .position(|child| child.view_id() == Some(view_id))
}

/// Find the index of a child view of the specified type.
pub fn find_child_index_by_type<T: 'static + View>(children: &[Box<dyn View>]) -> Option<usize> {
    children.iter().position(|child| child.is::<T>())
}

/// Adjust the shelf top edge based on the separator above it.
/// Assumes the shelf is at index+2 and separator is at index+1 relative to the bar at index.
pub fn adjust_shelf_top_edge(children: &mut [Box<dyn View>], bar_index: usize) {
    if bar_index + 2 >= children.len() {
        return;
    }

    let y_shift = children[bar_index].rect().max.y - children[bar_index + 1].rect().min.y;
    *children[bar_index + 1].rect_mut() += Point::new(0, y_shift);
    children[bar_index + 2].rect_mut().min.y = children[bar_index + 1].rect().max.y;
}

/// Update the shelf and bottom bar together.
/// This is a common pattern throughout the Home view implementation.
pub fn update_shelf_and_bottom_bar(
    was_resized: bool,
    home: &mut super::Home,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    home.update_shelf(was_resized, hub, rq, context);
    home.update_bottom_bar(rq, context);
}
