#![cfg_attr(not(target_os = "android"), allow(dead_code, unused_imports))]
#![warn(missing_docs)]
#![deny(warnings)]

//! Plato Android library
//!
//! This library provides the Android-specific implementation for Plato,
//! a document reader for e-readers. It handles the Android activity lifecycle
//! and event loop.

/// Android framebuffer implementation using ANativeWindow
pub mod framebuffer;

/// Android input event translation
pub mod input;

/// Android path resolution for library and settings
pub mod storage;

use anyhow::{Context as AnyhowContext, Result};

#[cfg(feature = "android")]
use std::collections::VecDeque;
#[cfg(feature = "android")]
use std::default::Default;
#[cfg(feature = "android")]
use std::path::Path;
#[cfg(feature = "android")]
use std::sync::mpsc;
#[cfg(feature = "android")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "android")]
use android_activity::{AndroidApp, MainEvent, PollEvent};

#[cfg(feature = "android")]
use plato_core::battery::FakeBattery;
#[cfg(feature = "android")]
use plato_core::context::Context;
#[cfg(feature = "android")]
use plato_core::font::Fonts;
#[cfg(feature = "android")]
use plato_core::frontlight::LightLevels;
#[cfg(feature = "android")]
use plato_core::geom::Point;
#[cfg(feature = "android")]
use plato_core::helpers::load_toml;
#[cfg(feature = "android")]
use plato_core::input::DeviceEvent;
#[cfg(feature = "android")]
use plato_core::library::Library;
#[cfg(feature = "android")]
use plato_core::metadata::SortMethod;
#[cfg(feature = "android")]
use plato_core::mobile_optimizations::{AnimationConfig, MemoryConfig, TouchConfig};
#[cfg(feature = "android")]
use plato_core::mobile_theme::{set_mobile_theme_mode, MobileThemeMode};
#[cfg(feature = "android")]
use plato_core::plugin::PluginSystem;
#[cfg(feature = "android")]
use plato_core::rustc_hash::FxHashMap;
#[cfg(feature = "android")]
use plato_core::settings::Settings;
#[cfg(feature = "android")]
use plato_core::settings::{FirstColumn, SecondColumn};
#[cfg(feature = "android")]
use plato_core::settings::{LibraryMode, LibrarySettings};
#[cfg(feature = "android")]
use plato_core::sync::BackgroundSync;
#[cfg(feature = "android")]
use plato_core::view::home::Home;
#[cfg(feature = "android")]
use plato_core::view::{RenderQueue, View};

#[cfg(all(target_os = "android", feature = "android"))]
#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("plato-android"),
    );

    log::info!("Plato Android starting...");

    if let Err(e) = run_android_app(app) {
        log::error!("Android app failed: {}", e);
    }
}

#[cfg(all(target_os = "android", feature = "android"))]
fn run_android_app(app: AndroidApp) -> Result<()> {
    // Initialize mobile optimization configs
    let touch_config = TouchConfig::platform_optimal();
    let _animation_config = AnimationConfig::default();
    let _memory_config = MemoryConfig::default();

    // Set mobile theme mode for OLED-optimized color palette
    set_mobile_theme_mode(MobileThemeMode::System);

    log::info!("Touch config: tap_jitter={}mm, hold_delay={}ms",
        touch_config.tap_jitter_mm, touch_config.hold_delay_ms);

    // Wait for native window to be created
    let mut native_window = None;
    let mut window_created = false;

    while !window_created {
        app.poll_events(None, |event| {
            if let PollEvent::Main(MainEvent::InitWindow { .. }) = event {
                // Get the native window from the app
                if let Some(window) = app.native_window() {
                    native_window = Some(window);
                    window_created = true;
                }
            }
        });
    }

    let window = native_window.context("Native window not created")?;
    let fb = Box::new(framebuffer::AndroidFramebuffer::new(window)?);

    // Initialize paths
    let library_path = storage::android_library_path();
    let settings_path = storage::android_settings_path();

    log::info!("Library path: {:?}", library_path);
    log::info!("Settings path: {:?}", settings_path);

    // Load settings
    let settings_path = Path::new(&settings_path).join("Settings.toml");
    let settings = if settings_path.exists() {
        load_toml::<Settings, _>(&settings_path)
            .with_context(|| "Failed to load settings")?
    } else {
        Settings::default()
    };

    // Ensure settings has at least one library
    let settings = if settings.libraries.is_empty() {
        let mut default_settings = settings;
        default_settings.libraries.push(LibrarySettings {
            name: "Android Library".to_string(),
            path: library_path.clone(),
            mode: LibraryMode::Database,
            // Use Title as default sort method
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
    let library = Library::new(&library_settings.path, library_settings.mode)
        .with_context(|| "Failed to load library")?;

    // Load fonts
    let fonts = Fonts::load().with_context(|| "Failed to load fonts")?;

    // Initialize stubs for hardware not present on Android devices.
    // Android devices don't have Kobo-specific hardware like battery sensors,
    // frontlight controllers, or ambient light sensors, so we use fake implementations.
    let battery = Box::new(FakeBattery::new()) as Box<dyn plato_core::battery::Battery>;
    let frontlight = Box::new(LightLevels::default()) as Box<dyn plato_core::frontlight::Frontlight>;
    let lightsensor = Box::new(0u16) as Box<dyn plato_core::lightsensor::LightSensor>;

    // Initialize plugin system and background sync
    let plugin_system = PluginSystem::new(&settings.plugin_settings);
    let background_sync = BackgroundSync::new(&settings.background_sync);

    // Create context
    let mut context = Context::new(
        fb,
        None, // No RTC on Android
        library,
        settings,
        fonts,
        battery,
        frontlight,
        lightsensor,
        plugin_system,
        background_sync,
    );

    // Set up gesture processing
    let (device_tx, device_rx) = mpsc::channel();
    let gesture_events = plato_core::gesture::gesture_events(device_rx);

    // Spawn gesture processor thread
    let gesture_tx_clone = gesture_tx.clone();
    std::thread::spawn(move || {
        while let Ok(event) = gesture_events.recv() {
            let _ = gesture_tx_clone.send(plato_core::view::Event::Gesture(event));
        }
    });

    // Create Home view
    let mut rq = RenderQueue::new();
    let mut view: Box<dyn View> = Box::new(Home::new(
        context.fb.rect(),
        &hub,
        &mut rq,
        &mut context,
    )?);

    log::info!("App loop starting...");

    // Main event loop
    let mut running = true;
    while running {
        app.poll_events(None, |event| {
            match event {
                PollEvent::Main(MainEvent::InputAvailable) => {
                    app.poll_events(None, |event| {
                        if let PollEvent::Main(MainEvent::InputAvailable) = event {
                             // This is where we receive motion events
                        }
                    });
                    // For MVP, capture and translate events directly
                    if let Some(event) = app.native_window().and_then(|_| app.poll_input()) {
                         if let Some(motion) = event.motion() {
                             crate::input::translate_motion_event(&motion, &device_tx);
                         }
                    }
                }
                PollEvent::Main(MainEvent::Pause) => {
                    log::info!("App paused");
                }
                PollEvent::Main(MainEvent::Resume { .. }) => {
                    log::info!("App resumed");
                }
                PollEvent::Main(MainEvent::Destroy) => {
                    log::info!("App destroyed, exiting");
                    running = false;
                }
                _ => {}
            }
        });

        // Process gesture events - gesture handlers send Event::Gesture internally
        while let Ok(event) = gesture_rx.try_recv() {
            view.handle_event(
                &event,
                &hub,
                &mut VecDeque::new(),
                &mut rq,
                &mut context,
            );
        }

        // Render if needed
        // For MVP, we'll do a simple render on each frame
        // In production, this should be driven by the render queue
    }

    log::info!("App loop ended");
    Ok(())
}

#[cfg(not(target_os = "android"))]
fn main() {
    println!("This is an Android-only library");
}
