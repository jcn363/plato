//! Helper functions for application lifecycle and view management.

use std::collections::VecDeque;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;

use plato_core::anyhow::{format_err, Context as ResultExt, Error};
use plato_core::battery::{Battery, KoboBattery};
use plato_core::context::Context;
use plato_core::device::{FrontlightKind, CURRENT_DEVICE};
use plato_core::font::Fonts;
use plato_core::framebuffer::Framebuffer;
use plato_core::frontlight::{
    Frontlight, NaturalFrontlight, PremixedFrontlight, StandardFrontlight,
};
use plato_core::helpers::load_toml;
use plato_core::lightsensor::{KoboLightSensor, LightSensor};
use plato_core::plugin::PluginSystem;
use plato_core::rtc::Rtc;
use plato_core::settings::{IntermKind, Settings, SETTINGS_PATH};
use plato_core::sync::BackgroundSync;
use plato_core::theme;
use plato_core::view::common::transfer_notifications;
use plato_core::view::intermission::Intermission;
use plato_core::view::menu::Menu;
use plato_core::view::reader::Reader;
use plato_core::view::wait_for_all;
use plato_core::view::{Event, RenderQueue, UpdateData, View};
use plato_core::{log_error, log_warn};

use crate::constants::RTC_DEVICE;

/// Application exit status.
#[derive(Clone, Copy, PartialEq)]
pub enum ExitStatus {
    /// Normal quit.
    Quit,
    /// Reboot the device.
    Reboot,
    /// Power off the device.
    PowerOff,
}

/// History item for view navigation.
pub struct HistoryItem {
    /// The view that was active.
    pub view: Box<dyn View>,
    /// Screen rotation when the view was active.
    pub rotation: i8,
    /// Whether the display was monochrome.
    pub monochrome: bool,
    /// Whether the display was dithered.
    pub dithered: bool,
}

/// Build the application context with all required components.
pub fn build_context(fb: Box<dyn Framebuffer>) -> Result<Context, Error> {
    let rtc = Rtc::new(RTC_DEVICE)
        .map_err(|e| log_error!("Can't open RTC device: {:#}.", e))
        .ok();
    let path = Path::new(SETTINGS_PATH);
    let mut settings = if path.exists() {
        load_toml::<Settings, _>(path).context("can't load settings")?
    } else {
        Default::default()
    };

    if settings.libraries.is_empty() {
        return Err(format_err!("no libraries found"));
    }

    if settings.selected_library >= settings.libraries.len() {
        settings.selected_library = 0;
    }

    if let Some(lang) = plato_core::i18n::Language::from_code(&settings.language) {
        plato_core::i18n::set_language(lang);
    }

    theme::set_theme_mode(settings.theme_settings.mode);
    theme::set_auto_threshold(settings.theme_settings.auto_threshold);
    theme::set_dark_mode(settings.dark_mode);

    let library_settings = &settings.libraries[settings.selected_library];
    let library = plato_core::library::Library::new(&library_settings.path, library_settings.mode)?;

    let fonts = Fonts::load().context("can't load fonts")?;

    let battery = Box::new(KoboBattery::new().context("can't create battery")?) as Box<dyn Battery>;

    let lightsensor = if CURRENT_DEVICE.has_lightsensor() {
        Box::new(KoboLightSensor::new().context("can't create light sensor")?)
            as Box<dyn LightSensor>
    } else {
        Box::new(0u16) as Box<dyn LightSensor>
    };

    let levels = settings.frontlight_levels;
    let frontlight: Box<dyn Frontlight> = match CURRENT_DEVICE.frontlight_kind() {
        FrontlightKind::Standard => StandardFrontlight::new(levels.intensity)
            .map(|fl| Box::new(fl) as Box<dyn Frontlight>)
            .unwrap_or_else(|_| {
                log_warn!("Warning: Standard frontlight unavailable, using no-op fallback");
                Box::new(levels) as Box<dyn Frontlight>
            }),
        FrontlightKind::Natural => NaturalFrontlight::new(levels.intensity, levels.warmth)
            .map(|fl| Box::new(fl) as Box<dyn Frontlight>)
            .unwrap_or_else(|_| {
                log_warn!("Warning: Natural frontlight unavailable, using no-op fallback");
                Box::new(levels) as Box<dyn Frontlight>
            }),
        FrontlightKind::Premixed => PremixedFrontlight::new(levels.intensity, levels.warmth)
            .map(|fl| Box::new(fl) as Box<dyn Frontlight>)
            .unwrap_or_else(|_| {
                log_warn!("Warning: Premixed frontlight unavailable, using no-op fallback");
                Box::new(levels) as Box<dyn Frontlight>
            }),
        _ => {
            log_warn!("Warning: Unknown frontlight kind, using no-op fallback");
            Box::new(levels) as Box<dyn Frontlight>
        }
    };

    let plugin_system = PluginSystem::new(&settings.plugin_settings);
    let background_sync = BackgroundSync::new(&settings.background_sync);

    Ok(Context::new(
        fb,
        rtc,
        library,
        settings,
        fonts,
        battery,
        frontlight,
        lightsensor,
        plugin_system,
        background_sync,
    ))
}

/// Power off the device, cleaning up all views and showing intermission screen.
pub fn power_off(
    view: &mut dyn View,
    history: &mut Vec<HistoryItem>,
    updating: &mut Vec<UpdateData>,
    context: &mut Context,
) {
    let (tx, _rx) = mpsc::channel();
    view.handle_event(
        &Event::Back,
        &tx,
        &mut VecDeque::new(),
        &mut RenderQueue::new(),
        context,
    );
    while let Some(mut item) = history.pop() {
        item.view.handle_event(
            &Event::Back,
            &tx,
            &mut VecDeque::new(),
            &mut RenderQueue::new(),
            context,
        );
    }
    let interm = Intermission::new(
        context.fb.rect(),
        IntermKind::PowerOff,
        context.settings.sleep_cover_fill,
        context,
    );
    wait_for_all(updating, context);
    interm.render(context.fb.as_mut(), *interm.rect(), &mut context.fonts);
    context
        .fb
        .update(interm.rect(), plato_core::framebuffer::UpdateMode::Full)
        .ok();
}

/// Enable or disable WiFi.
pub fn set_wifi(enable: bool, context: &mut Context) {
    if context.settings.wifi == enable {
        return;
    }
    context.settings.wifi = enable;
    if context.settings.wifi {
        Command::new("scripts/wifi-enable.sh").status().ok();
    } else {
        Command::new("scripts/wifi-disable.sh").status().ok();
        context
            .flags
            .remove(plato_core::context::DeviceFlags::ONLINE);
    }
}

/// Navigate to a new view, saving the current view to history.
pub fn goto_view(
    next_view: Box<dyn View>,
    view: &mut Box<dyn View>,
    history: &mut Vec<HistoryItem>,
    rotation: i8,
    monochrome: bool,
    dithered: bool,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    // Trigger book close plugins if current view is a Reader
    if view.is::<Reader>() {
        if let Err(e) = context.plugin_system.on_book_close(&Path::new("")) {
            log_error!("Failed to trigger book close plugins: {}", e);
        }
    }

    let mut next_view = next_view;
    view.children_mut().retain(|child| !child.is::<Menu>());
    transfer_notifications(view.as_mut(), next_view.as_mut(), rq, context);
    let item = HistoryItem {
        view: std::mem::replace(view, next_view),
        rotation,
        monochrome,
        dithered,
    };
    history.push(item);
}
