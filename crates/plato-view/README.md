# Plato‑View

The `plato‑view` crate provides the view‑tree infrastructure used by the main Plato application. It defines the core `View` trait, common UI components (buttons, menus, lists, dialogs), and the event‑bubbling system that drives the e‑ink user interface.

## Public API (selected)

| Symbol | Description |
|--------|-------------|
| `View` | Trait that every UI element implements. |
| `ViewAdapter` | Helper to wrap a `View` with additional behaviour. |
| `Event` | Enum that covers all input events (touch, key, gesture). |
| `RenderQueue` | Batch of rendering commands sent to the framebuffer. |
| `common` | Collection of ready‑made widgets: `Button`, `Label`, `Menu`, … |

The crate is intentionally kept free of hardware‑specific code so that it can be compiled for any target.

## Example

```rust
use plato_view::{View, Event, RenderQueue};

struct MyView;

impl View for MyView {
    fn handle_event(&mut self, evt: &Event, rq: &mut RenderQueue) {
        // handle taps, key presses, etc.
    }
    // … other required methods
}
```
