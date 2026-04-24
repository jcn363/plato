//! Helper functions for the Plato emulator.

use plato_core::anyhow::Error;
use plato_core::battery::{Battery, FakeBattery};
use plato_core::context::Context;
use plato_core::font::Fonts;
use plato_core::framebuffer::Framebuffer;
use plato_core::frontlight::{Frontlight, LightLevels};
use plato_core::helpers::load_toml;
use plato_core::helpers::xdg::{self, ensure_xdg_dirs};
use plato_core::input::{DeviceEvent, FingerStatus};
use plato_core::library::Library;
use plato_core::lightsensor::LightSensor;
use plato_core::plugin::PluginSystem;
use plato_core::pt;
use plato_core::settings::Settings;
use plato_core::sync::BackgroundSync;
use sdl2::event::Event as SdlEvent;

/// Build the application context for the emulator.
pub fn build_context(fb: Box<dyn Framebuffer>) -> Result<Context, Error> {
    // Ensure XDG directories exist for installed desktop version
    ensure_xdg_dirs().ok();

    let settings_path = xdg::settings_path();
    let settings = if settings_path.exists() {
        load_toml::<Settings, _>(&settings_path)?
    } else {
        // Create default settings
        let default_settings = Settings::default();
        // Save to XDG config path
        plato_core::helpers::save_toml(&default_settings, &settings_path)?;
        default_settings
    };

    let library_settings = &settings.libraries[settings.selected_library];
    let library_path = if std::path::Path::new(&library_settings.path).is_absolute() {
        library_settings.path.clone()
    } else {
        xdg::library_path().join(&library_settings.path)
    };
    let library = Library::new(&library_path, library_settings.mode)?;

    let battery = Box::new(FakeBattery::new()) as Box<dyn Battery>;
    let frontlight = Box::new(LightLevels::default()) as Box<dyn Frontlight>;
    let lightsensor = Box::new(0u16) as Box<dyn LightSensor>;
    let fonts = Fonts::load()?;
    let plugin_settings = settings.plugin_settings.clone();
    let background_sync = settings.background_sync.clone();

    Ok(Context::new(
        fb,
        None,
        library,
        settings,
        fonts,
        battery,
        frontlight,
        lightsensor,
        PluginSystem::new(&plugin_settings),
        BackgroundSync::new(&background_sync),
    ))
}

/// Convert SDL timestamp to seconds.
#[inline]
pub fn seconds(timestamp: u32) -> f64 {
    timestamp as f64 / 1000.0
}

/// Convert SDL event to device event.
pub fn device_event(event: SdlEvent) -> Option<DeviceEvent> {
    match event {
        SdlEvent::MouseButtonDown {
            timestamp, x, y, ..
        } => Some(DeviceEvent::Finger {
            id: 0,
            status: FingerStatus::Down,
            position: pt!(x, y),
            time: seconds(timestamp),
        }),
        SdlEvent::MouseButtonUp {
            timestamp, x, y, ..
        } => Some(DeviceEvent::Finger {
            id: 0,
            status: FingerStatus::Up,
            position: pt!(x, y),
            time: seconds(timestamp),
        }),
        SdlEvent::MouseMotion {
            timestamp, x, y, ..
        } => Some(DeviceEvent::Finger {
            id: 0,
            status: FingerStatus::Motion,
            position: pt!(x, y),
            time: seconds(timestamp),
        }),
        _ => None,
    }
}
