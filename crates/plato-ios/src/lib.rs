#![cfg_attr(not(target_os = "ios"), allow(dead_code, unused_imports))]
#![warn(missing_docs)]
#![deny(warnings)]
#![cfg_attr(feature = "ios", allow(static_mut_refs, dead_code))]

//! Plato iOS library
//!
//! This library provides the iOS-specific implementation for Plato,
//! a document reader for e-readers. It handles the iOS app lifecycle
//! and event loop through a Swift bridge.

/// iOS framebuffer implementation using Metal
pub mod framebuffer;

/// iOS input event translation from UITouch
pub mod input;

/// iOS path resolution for library and settings
pub mod storage;

#[cfg(feature = "ios")]
use anyhow::{Context as AnyhowContext, Result};
#[cfg(feature = "ios")]
use plato_core::battery::FakeBattery;
#[cfg(feature = "ios")]
use plato_core::context::Context;
#[cfg(feature = "ios")]
use plato_core::framebuffer::Framebuffer;
#[cfg(feature = "ios")]
use plato_core::font::Fonts;
#[cfg(feature = "ios")]
use plato_core::frontlight::LightLevels;
#[cfg(feature = "ios")]
use plato_core::helpers::load_toml;
#[cfg(feature = "ios")]
use plato_core::input::DeviceEvent;
#[cfg(feature = "ios")]
use plato_core::library::Library;
#[cfg(feature = "ios")]
use plato_core::mobile_optimizations::{AnimationConfig, MemoryConfig, TouchConfig};
#[cfg(feature = "ios")]
use plato_core::mobile_theme::{set_mobile_theme_mode, MobileThemeMode};
#[cfg(feature = "ios")]
use plato_core::plugin::PluginSystem;
#[cfg(feature = "ios")]
use plato_core::settings::{LibraryMode, LibrarySettings};
#[cfg(feature = "ios")]
use plato_core::metadata::SortMethod;
#[cfg(feature = "ios")]
use plato_core::settings::{FirstColumn, SecondColumn};
#[cfg(feature = "ios")]
use plato_core::settings::Settings;
#[cfg(feature = "ios")]
use plato_core::sync::BackgroundSync;
#[cfg(feature = "ios")]
use plato_core::view::home::Home;
#[cfg(feature = "ios")]
use plato_core::view::{RenderQueue, View, Hub, Bus};
#[cfg(feature = "ios")]
use plato_core::rustc_hash::FxHashMap;
#[cfg(feature = "ios")]
use plato_core::geom::Point;
#[cfg(feature = "ios")]
use plato_core::gesture::GestureEvent;
#[cfg(feature = "ios")]
use plato_core::input::{ButtonCode, FingerStatus};
#[cfg(feature = "ios")]
use std::path::{Path, PathBuf};
#[cfg(feature = "ios")]
use std::collections::VecDeque;
#[cfg(feature = "ios")]
use std::sync::mpsc;
#[cfg(feature = "ios")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "ios")]
use std::default::Default;

/// Global context for iOS app
#[cfg(feature = "ios")]
static mut CONTEXT: Option<Context> = None;

/// Global framebuffer for iOS app (stored as concrete type for render access)
/// Uses Arc<Mutex> to share ownership with Context safely
#[cfg(feature = "ios")]
static mut FRAMEBUFFER: Option<std::sync::Arc<std::sync::Mutex<framebuffer::IOSFramebuffer>>> = None;

/// Global event sender for iOS app
#[cfg(feature = "ios")]
static mut EVENT_TX: Option<std::sync::mpsc::Sender<plato_core::input::DeviceEvent>> = None;

/// Global event receiver for iOS app
#[cfg(feature = "ios")]
static mut EVENT_RX: Option<std::sync::mpsc::Receiver<plato_core::input::DeviceEvent>> = None;

/// Global view for iOS app
#[cfg(feature = "ios")]
static mut VIEW: Option<Box<dyn View>> = None;

/// Global event hub for iOS app
#[cfg(feature = "ios")]
static mut HUB: Option<Hub> = None;

/// Global event hub receiver for iOS app
#[cfg(feature = "ios")]
static mut HUB_RX: Option<std::sync::mpsc::Receiver<plato_core::view::Event>> = None;

/// Global render queue for iOS app
#[cfg(feature = "ios")]
static mut RENDER_QUEUE: Option<RenderQueue> = None;

/// Global gesture state: contacts (finger touch tracking)
#[cfg(feature = "ios")]
static mut CONTACTS: Option<Arc<Mutex<FxHashMap<i32, (Point, f64)>>>> = None;

/// Global gesture state: segments (touch path segments)
#[cfg(feature = "ios")]
static mut SEGMENTS: Option<Arc<Mutex<Vec<Vec<Point>>>>> = None;

/// Get mutable reference to global context (internal use only)
#[cfg(feature = "ios")]
#[no_mangle]
pub unsafe extern "C" fn plato_get_context() -> *mut Context {
    CONTEXT.as_mut().map(|c| c as *mut Context).unwrap_or(std::ptr::null_mut())
}

/// Initialize the Plato core
/// Called from Swift when the app launches
#[cfg(feature = "ios")]
#[no_mangle]
pub unsafe extern "C" fn plato_init(width: u32, height: u32) -> bool {
    log::info!("Plato iOS initializing...");

    // Initialize mobile optimization configs
    let touch_config = TouchConfig::platform_optimal();
    let _animation_config = AnimationConfig::default();
    let _memory_config = MemoryConfig::default();

    // Set mobile theme mode for OLED-optimized color palette
    set_mobile_theme_mode(MobileThemeMode::System);

    log::info!("Touch config: tap_jitter={}mm, hold_delay={}ms",
        touch_config.tap_jitter_mm, touch_config.hold_delay_ms);

    // Use iOS-specific path resolution
    let library_path = storage::ios_library_path();
    let settings_path = storage::ios_settings_path();

    log::info!("Library path: {:?}", library_path);
    log::info!("Settings path: {:?}", settings_path);

    // Create event channel for touch events
    let (event_tx, event_rx) = mpsc::channel();
    EVENT_TX = Some(event_tx);
    EVENT_RX = Some(event_rx);

    // Create event hub for view event dispatch
    let (hub, hub_rx) = mpsc::channel();
    HUB = Some(hub);
    HUB_RX = Some(hub_rx);

    // Create render queue
    let rq = RenderQueue::new();
    RENDER_QUEUE = Some(rq);

    // Create framebuffer wrapped in Arc<Mutex> for shared ownership
    let fb = match framebuffer::IOSFramebuffer::new(width, height) {
        Ok(fb) => fb,
        Err(e) => {
            log::error!("Failed to create framebuffer: {}", e);
            return false;
        }
    };

    let fb_arc = Arc::new(Mutex::new(fb));

    // Store framebuffer for render access (shared ownership)
    FRAMEBUFFER = Some(fb_arc.clone());

    // Box the framebuffer for context (same Arc instance)
    let fb_boxed = Box::new(framebuffer::ArcFramebuffer(fb_arc)) as Box<dyn plato_core::framebuffer::Framebuffer>;

    // Load settings
    let settings_path = Path::new(&settings_path).join("Settings.toml");
    let settings = if settings_path.exists() {
        match load_toml::<Settings, _>(&settings_path) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to load settings: {}", e);
                Settings::default()
            }
        }
    } else {
        Settings::default()
    };

    // Ensure settings has at least one library
    let settings = if settings.libraries.is_empty() {
        let mut default_settings = settings;
        default_settings.libraries.push(LibrarySettings {
            name: "iOS Library".to_string(),
            path: PathBuf::from(library_path.clone()),
            mode: LibraryMode::Database,
            sort_method: SortMethod::Title,
            first_column: FirstColumn::TitleAndAuthor,
            second_column: SecondColumn::Progress,
            thumbnail_previews: true,
            hooks: Vec::new(),
        });
        default_settings.selected_library = 0;
        default_settings
    } else {
        settings
    };

    // Load library
    let library_settings = &settings.libraries[settings.selected_library];
    let library = match Library::new(&library_settings.path, library_settings.mode) {
        Ok(lib) => lib,
        Err(e) => {
            log::error!("Failed to load library: {}", e);
            return false;
        }
    };

    // Load fonts
    let fonts = match Fonts::load() {
        Ok(f) => f,
        Err(e) => {
            log::error!("Failed to load fonts: {}", e);
            return false;
        }
    };

    // Initialize stubs for hardware not present on iOS
    let battery = Box::new(FakeBattery::new()) as Box<dyn plato_core::battery::Battery>;
    let frontlight = Box::new(LightLevels::default()) as Box<dyn plato_core::frontlight::Frontlight>;
    let lightsensor = Box::new(0u16) as Box<dyn plato_core::lightsensor::LightSensor>;

    // Initialize plugin system and background sync
    let plugin_system = PluginSystem::new(&settings.plugin_settings);
    let background_sync = BackgroundSync::new(&settings.background_sync);

    // Create context
    let context = Context::new(
        fb_boxed,
        None, // No RTC on iOS
        library,
        settings,
        fonts,
        battery,
        frontlight,
        lightsensor,
        plugin_system,
        background_sync,
    );

    CONTEXT = Some(context);

    // Initialize gesture state (simplified for iOS)
    let contacts: Arc<Mutex<FxHashMap<i32, (Point, f64)>>> =
        Arc::new(Mutex::new(FxHashMap::default()));
    let segments: Arc<Mutex<Vec<Vec<Point>>>> = Arc::new(Mutex::new(Vec::new()));
    CONTACTS = Some(contacts);
    SEGMENTS = Some(segments);

    // Create Home view
    let fb_width = FRAMEBUFFER.as_ref().unwrap().width();
    let fb_height = FRAMEBUFFER.as_ref().unwrap().height();
    let fb_rect = plato_core::geom::Rectangle::new(
        plato_core::geom::Point::new(0, 0),
        plato_core::geom::Point::new(fb_width as i32, fb_height as i32),
    );
    let hub_ref = HUB.as_ref().unwrap();
    let rq_ref = RENDER_QUEUE.as_mut().unwrap();
    let context_ref = CONTEXT.as_mut().unwrap();
    match Home::new(fb_rect, hub_ref, rq_ref, context_ref) {
        Ok(home) => {
            VIEW = Some(Box::new(home) as Box<dyn View>);
            log::info!("Home view initialized successfully");
        }
        Err(e) => {
            log::error!("Failed to create Home view: {}", e);
            return false;
        }
    }

    true
}

/// Handle touch down event
/// Called from Swift when touch down occurs
#[cfg(feature = "ios")]
#[no_mangle]
pub unsafe extern "C" fn plato_touch_down(id: i32, x: i32, y: i32) {
    if let Some(ref tx) = EVENT_TX {
        crate::input::translate_touch_event(id, x as f32, y as f32, 0, tx);
        log::debug!("Touch down at ({}, {}) with id {}", x, y, id);
    }
}

/// Handle touch move event
/// Called from Swift when touch move occurs
#[cfg(feature = "ios")]
#[no_mangle]
pub unsafe extern "C" fn plato_touch_move(id: i32, x: i32, y: i32) {
    if let Some(ref tx) = EVENT_TX {
        crate::input::translate_touch_event(id, x as f32, y as f32, 1, tx);
        log::debug!("Touch move at ({}, {}) with id {}", x, y, id);
    }
}

/// Handle touch up event
/// Called from Swift when touch up occurs
#[cfg(feature = "ios")]
#[no_mangle]
pub unsafe extern "C" fn plato_touch_up(id: i32, x: i32, y: i32) {
    if let Some(ref tx) = EVENT_TX {
        crate::input::translate_touch_event(id, x as f32, y as f32, 2, tx);
        log::debug!("Touch up at ({}, {}) with id {}", x, y, id);
    }
}

/// Render the current view to a caller-provided buffer
/// Called from Swift on each frame or when needed
#[cfg(feature = "ios")]
#[no_mangle]
pub unsafe extern "C" fn plato_render(buffer_ptr: *mut u8, len: usize) -> bool {
    // Validate pointer and length
    if buffer_ptr.is_null() || len == 0 {
        log::error!("Invalid buffer parameters");
        return false;
    }

    // Create a mutable slice from the buffer
    let buffer = std::slice::from_raw_parts_mut(buffer_ptr, len);

    // Collect DeviceEvents and detect simple taps
    let mut device_events = VecDeque::new();
    if let Some(ref mut rx) = EVENT_RX {
        while let Ok(device_event) = rx.try_recv() {
            device_events.push_back(device_event);
        }
    }

    // Simple gesture detection: detect taps from DeviceEvents
    // For MVP, convert Finger events directly to GestureEvent::Tap
    let mut bus = Bus::new();
    if let (Some(ref mut view), Some(ref mut rq), Some(ref mut context), Some(ref hub)) = 
        (VIEW.as_mut(), RENDER_QUEUE.as_mut(), CONTEXT.as_mut(), HUB.as_ref()) {
        for device_event in device_events {
            if let plato_core::input::DeviceEvent::Finger { status: FingerStatus::Up, position, .. } = device_event {
                // Convert finger up to tap gesture (simplified)
                let gesture_event = plato_core::view::Event::Gesture(GestureEvent::Tap(position));
                view.handle_event(&gesture_event, hub, &mut bus, rq, context);
            }
        }
    }

    // Render the view to framebuffer (simplified)
    if let (Some(ref mut view), Some(ref mut context)) = (VIEW.as_mut(), CONTEXT.as_mut()) {
        let fb_rect = plato_core::geom::Rectangle::new(
            plato_core::geom::Point::new(0, 0),
            plato_core::geom::Point::new(
                FRAMEBUFFER.as_ref().unwrap().width() as i32,
                FRAMEBUFFER.as_ref().unwrap().height() as i32,
            ),
        );
        view.render(context.fb.as_mut(), fb_rect, &mut context.fonts);
    }

    // Get framebuffer and fill the Swift-allocated buffer in-place
    if let Some(ref fb_arc) = FRAMEBUFFER {
        fb_arc.fill_rgba_buffer(buffer);
        true
    } else {
        log::error!("No framebuffer available");
        false
    }
}

/// Cleanup resources
/// Called from Swift when the app is terminating
#[cfg(feature = "ios")]
#[no_mangle]
pub unsafe extern "C" fn plato_deinit() {
    log::info!("Plato iOS cleanup");
    CONTEXT = None;
    VIEW = None;
    HUB = None;
    HUB_RX = None;
    RENDER_QUEUE = None;
    CONTACTS = None;
    SEGMENTS = None;
    EVENT_TX = None;
    EVENT_RX = None;
    FRAMEBUFFER = None;
}

#[cfg(not(target_os = "ios"))]
fn main() {
    println!("This is an iOS-only library");
}
