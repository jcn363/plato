use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::view::menu::Menu;
use crate::view::{RenderData, RenderQueue, View, ViewId};

/// Generic helper to toggle a menu's visibility with children vector.
///
/// This variant works with &mut Vec<Box<dyn View>> directly, used in reader_settings.rs
pub fn toggle_menu_vec<F>(
    id: ViewId,
    create_fn: F,
    children: &mut Vec<Box<dyn View>>,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) where
    F: FnOnce(&mut Context) -> Menu,
{
    if let Some(index) = children
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == id))
    {
        if let Some(true) = enable {
            return;
        }
        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
        let menu = create_fn(context);
        rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
        children.push(Box::new(menu) as Box<dyn View>);
    }
}

/// Generic helper to toggle a menu's visibility with creation function.
///
/// This eliminates boilerplate by combining:
/// 1. Menu existence check
/// 2. Early return based on enable flag
/// 3. Menu removal with expose render queuing
/// 4. Menu creation with new render queuing
///
/// # Arguments
///
/// * `id` - The ViewId of the menu to toggle
/// * `create_fn` - Function that creates the menu when needed
/// * `view` - The parent view containing children
/// * `enable` - Optional boolean to force show/hide (None = toggle)
/// * `rq` - Render queue for scheduling render operations
/// * `_context` - Application context (unused in this variant, kept for consistency)
///
/// # Examples
///
/// ```ignore
/// toggle_menu_with(
///     ViewId::FontSizeMenu,
///     || Menu::new(rect, ViewId::FontSizeMenu, MenuKind::DropDown, entries, context),
///     &mut self.children,
///     enable,
///     rq,
///     context,
/// );
/// ```
pub fn toggle_menu_with<F>(
    id: ViewId,
    create_fn: F,
    view: &mut dyn View,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    _context: &mut Context,
) where
    F: FnOnce() -> Menu,
{
    if let Some(index) = view
        .children()
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == id))
    {
        if let Some(true) = enable {
            return;
        }
        let rect = overlapping_rectangle(view.child(index));
        rq.add(RenderData::expose(rect, UpdateMode::Gui));
        view.children_mut().remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
        let menu = create_fn();
        rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
        view.children_mut().push(Box::new(menu));
    }
}

/// Generic helper to toggle a menu's visibility with contextual creation function.
///
/// Similar to toggle_menu_with but allows the creation function to access
/// additional context like current page, info, etc.
///
/// # Arguments
///
/// * `id` - The ViewId of the menu to toggle
/// * `create_fn` - Function that creates the menu when needed, receiving context
/// * `view` - The parent view containing children
/// * `enable` - Optional boolean to force show/hide (None = toggle)
/// * `rq` - Render queue for scheduling render operations
/// * `context` - Application context for menu creation
///
/// # Examples
///
/// ```ignore
/// toggle_menu_ctx(
///     ViewId::MarginWidthMenu,
///     |ctx| Menu::new(rect, ViewId::MarginWidthMenu, MenuKind::DropDown, entries, ctx),
///     &mut self.children,
///     enable,
///     rq,
///     context,
/// );
/// ```
pub fn toggle_menu_ctx<F>(
    id: ViewId,
    create_fn: F,
    view: &mut dyn View,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) where
    F: FnOnce(&mut Context) -> Menu,
{
    if let Some(index) = view
        .children()
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == id))
    {
        if let Some(true) = enable {
            return;
        }
        let rect = overlapping_rectangle(view.child(index));
        rq.add(RenderData::expose(rect, UpdateMode::Gui));
        view.children_mut().remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
        let menu = create_fn(context);
        rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
        view.children_mut().push(Box::new(menu));
    }
}

/// Generic helper to toggle a menu's visibility with item-specific creation function.
///
/// Similar to toggle_menu_ctx but allows the creation function to access
/// both context and an item-specific parameter (like an annotation).
///
/// # Arguments
///
/// * `id` - The ViewId of the menu to toggle
/// * `create_fn` - Function that creates the menu when needed, receiving context and item
/// * `view` - The parent view containing children
/// * `item` - Item-specific parameter for menu creation
/// * `enable` - Optional boolean to force show/hide (None = toggle)
/// * `rq` - Render queue for scheduling render operations
/// * `context` - Application context for menu creation
///
/// # Examples
///
/// ```ignore
/// toggle_menu_item(
///     ViewId::AnnotationMenu,
///     |ctx, annot| Menu::new(rect, ViewId::AnnotationMenu, MenuKind::DropDown, entries, ctx),
///     &mut self.children,
///     annotation,
///     enable,
///     rq,
///     context,
/// );
/// ```
pub fn toggle_menu_item<F, T>(
    id: ViewId,
    create_fn: F,
    view: &mut dyn View,
    item: T,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) where
    F: FnOnce(&mut Context, &T) -> Menu,
{
    if let Some(index) = view
        .children()
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == id))
    {
        if let Some(true) = enable {
            return;
        }
        let rect = overlapping_rectangle(view.child(index));
        rq.add(RenderData::expose(rect, UpdateMode::Gui));
        view.children_mut().remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
        let menu = create_fn(context, &item);
        rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
        view.children_mut().push(Box::new(menu));
    }
}

/// Helper function to calculate the overlapping rectangle of a view and its children.
///
/// Used for proper expose regions when removing views.
fn overlapping_rectangle(view: &dyn View) -> Rectangle {
    let mut rect = *view.rect();
    for child in view.children() {
        rect.absorb(&overlapping_rectangle(child.as_ref()));
    }
    rect
}

/// Generic helper to toggle a menu's visibility with &mut self pattern.
///
/// This variant works with methods that take &mut self and use locate_by_id internally.
/// Similar to toggle_menu_with but uses overlapping_rectangle for expose calculation.
///
/// # Arguments
///
/// * `id` - The ViewId of the menu to toggle
/// * `create_fn` - Function that creates the menu when needed
/// * `view` - The parent view (&mut self)
/// * `enable` - Optional boolean to force show/hide (None = toggle)
/// * `rq` - Render queue for scheduling render operations
/// * `context` - Application context for menu creation
pub fn toggle_menu_self<F>(
    id: ViewId,
    create_fn: F,
    view: &mut dyn View,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) where
    F: FnOnce(&mut Context) -> Menu,
{
    if let Some(index) = view
        .children()
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == id))
    {
        if let Some(true) = enable {
            return;
        }
        let rect = overlapping_rectangle(view.child(index));
        rq.add(RenderData::expose(rect, UpdateMode::Gui));
        view.children_mut().remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
        let menu = create_fn(context);
        rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
        view.children_mut().push(Box::new(menu));
    }
}
