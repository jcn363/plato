use crate::constants::{
    APP_NAME, AUTO_SUSPEND_REFRESH_INTERVAL, BATTERY_REFRESH_INTERVAL, BUTTON_INPUTS,
    CLOCK_REFRESH_INTERVAL, FB_DEVICE, POWER_INPUTS, TOUCH_INPUTS,
};
use crate::event::{handle_device_event, handle_launch, EventContext};
use crate::event_handlers::{
    handle_back_event, handle_battery_event, handle_gesture_event, handle_open_event,
    handle_screenshot_event, handle_suspend_check_event, handle_suspend_event,
    handle_suspend_execute_event, handle_wifi_event,
};
use crate::helpers::{build_context, goto_view, set_wifi, ExitStatus, HistoryItem};
use crate::task::{schedule_task, Task, TaskId};

use plato_core::anyhow::{Context as ResultExt, Error};
use plato_core::context::DeviceFlags;
use plato_core::device::CURRENT_DEVICE;
use plato_core::document::sys_info_as_html;
use plato_core::framebuffer::{Framebuffer, KoboFramebuffer1, KoboFramebuffer2, UpdateMode};
use plato_core::gesture::gesture_events;
use plato_core::helpers::save_toml;
use plato_core::input::{device_events, display_rotate_event, raw_events, usb_events};
use plato_core::log_error;
use plato_core::settings::{IntermKind, SETTINGS_PATH};
use plato_core::view::common::locate_by_id;
use plato_core::view::dialog::Dialog;
use plato_core::view::frontlight::FrontlightWindow;
use plato_core::view::home::Home;
use plato_core::view::intermission::Intermission;
use plato_core::view::menu::{Menu, MenuKind};
use plato_core::view::notification::Notification;
use plato_core::view::reader::Reader;
use plato_core::view::{handle_event, process_render_queue, wait_for_all};
use plato_core::view::{EntryId, EntryKind, Event, RenderData, RenderQueue, View, ViewId};
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

pub fn run() -> Result<(), Error> {
    let inactive_since = Instant::now();
    let exit_status = ExitStatus::Quit;

    let mut fb: Box<dyn Framebuffer> = if CURRENT_DEVICE.mark() == 8 {
        Box::new(KoboFramebuffer2::new(FB_DEVICE).context("can't create framebuffer")?)
    } else {
        Box::new(KoboFramebuffer1::new(FB_DEVICE).context("can't create framebuffer")?)
    };

    let initial_rotation = CURRENT_DEVICE.transformed_rotation(fb.rotation());
    let startup_rotation = CURRENT_DEVICE.startup_rotation();
    if !CURRENT_DEVICE.has_gyroscope() && initial_rotation != startup_rotation {
        fb.set_rotation(startup_rotation).ok();
    }

    let mut context = build_context(fb).context("can't build context")?;

    context.flags.set(
        DeviceFlags::PLUGGED,
        context.battery.status().is_ok_and(|v| v[0].is_wired()),
    );

    if context.settings.import.startup_trigger {
        context.batch_import();
        // Trigger book import plugins
        if let Err(e) = context
            .plugin_system
            .on_book_import(&Path::new("/mnt/onboard"))
        {
            log_error!("Failed to trigger book import plugins: {}", e);
        }
    }
    context.load_dictionaries();
    context.load_keyboard_layouts();

    let mut paths = Vec::new();
    for ti in &TOUCH_INPUTS {
        if Path::new(ti).exists() {
            paths.push(ti.to_string());
            break;
        }
    }
    for bi in &BUTTON_INPUTS {
        if Path::new(bi).exists() {
            paths.push(bi.to_string());
            break;
        }
    }
    for pi in &POWER_INPUTS {
        if Path::new(pi).exists() {
            paths.push(pi.to_string());
            break;
        }
    }

    let (raw_sender, raw_receiver) = raw_events(paths);
    let touch_screen = gesture_events(device_events(
        raw_receiver,
        context.display,
        context.settings.button_scheme,
    ));
    let usb_port = usb_events();

    let (mut tx, rx) = mpsc::channel();
    let tx2 = tx.clone();

    thread::spawn(move || {
        while let Ok(evt) = touch_screen.recv() {
            tx2.send(evt).ok();
        }
    });

    let tx3 = tx.clone();
    thread::spawn(move || {
        while let Ok(evt) = usb_port.recv() {
            tx3.send(Event::Device(evt)).ok();
        }
    });

    let tx4 = tx.clone();
    thread::spawn(move || loop {
        thread::sleep(CLOCK_REFRESH_INTERVAL);
        tx4.send(Event::ClockTick).ok();
    });

    let tx5 = tx.clone();
    thread::spawn(move || loop {
        thread::sleep(BATTERY_REFRESH_INTERVAL);
        tx5.send(Event::BatteryTick).ok();
    });

    if context.settings.auto_suspend > 0.0 {
        let tx6 = tx.clone();
        thread::spawn(move || loop {
            thread::sleep(AUTO_SUSPEND_REFRESH_INTERVAL);
            tx6.send(Event::MightSuspend).ok();
        });
    }

    context.fb.set_inverted(context.settings.inverted);

    if context.settings.wifi {
        Command::new("scripts/wifi-enable.sh").status().ok();
    } else {
        Command::new("scripts/wifi-disable.sh").status().ok();
    }

    if context.settings.frontlight {
        let levels = context.settings.frontlight_levels;
        context.frontlight.set_warmth(levels.warmth);
        context.frontlight.set_intensity(levels.intensity);
    } else {
        context.frontlight.set_intensity(0.0);
        context.frontlight.set_warmth(0.0);
    }

    // Pre-allocate vectors with estimated capacities to reduce reallocations
    let mut tasks: Vec<Task> = Vec::with_capacity(8);
    let mut history: Vec<HistoryItem> = Vec::with_capacity(16);
    let mut rq = RenderQueue::new();
    let mut view: Box<dyn View> =
        Box::new(Home::new(context.fb.rect(), &tx, &mut rq, &mut context)?);

    let mut updating = Vec::with_capacity(8);
    let current_dir = env::current_dir()?;

    println!(
        "{} is running on a Kobo {}.",
        APP_NAME, CURRENT_DEVICE.model
    );
    println!(
        "The framebuffer resolution is {} by {}.",
        context.fb.rect().width(),
        context.fb.rect().height()
    );

    let mut bus = VecDeque::with_capacity(4);

    schedule_task(
        TaskId::CheckBattery,
        Event::CheckBattery,
        BATTERY_REFRESH_INTERVAL,
        &tx,
        &mut tasks,
    );
    tx.send(Event::WakeUp).ok();

    // Trigger startup plugins
    if let Err(e) = context.plugin_system.on_startup() {
        log_error!("Failed to trigger startup plugins: {}", e);
    }

    let mut event_ctx =
        EventContext::new(tasks, inactive_since, exit_status, raw_sender, current_dir);

    while let Ok(evt) = rx.recv() {
        match evt {
            Event::Device(de) => {
                if handle_device_event(
                    de,
                    view.as_mut(),
                    &tx,
                    &mut bus,
                    &mut rq,
                    &mut context,
                    &mut event_ctx,
                    &mut history,
                    &mut updating,
                    &evt,
                ) {
                    continue;
                }
            }
            Event::BatteryTick => {
                // Check if background sync is needed
                if context.background_sync.sync_needed() {
                    if context.background_sync.should_auto_enable_wifi()
                        && !context.flags.contains(DeviceFlags::ONLINE)
                    {
                        set_wifi(true, &mut context);
                    }

                    if !context.background_sync.wifi_only()
                        || context.flags.contains(DeviceFlags::ONLINE)
                    {
                        context.background_sync.trigger_sync();

                        // Perform actual sync in a background thread
                        let tx_sync = tx.clone();
                        let settings = context.settings.clone();
                        let library_home = context.library.home.clone();
                        thread::spawn(move || {
                            if let Err(e) = plato_core::sync::check_network_and_sync(
                                &settings.cloud_sync,
                                &settings.background_sync,
                                &library_home,
                            ) {
                                log_error!("Background sync failed: {}", e);
                            } else {
                                tx_sync
                                    .send(Event::Notify("Background sync completed".to_string()))
                                    .ok();
                            }
                        });

                        // Trigger sync complete plugins
                        if let Err(e) = context.plugin_system.on_sync_complete() {
                            log_error!("Failed to trigger sync complete plugins: {}", e);
                        }
                    }
                }
            }
            Event::CheckBattery => {
                handle_battery_event(
                    &mut context,
                    &mut event_ctx,
                    &tx,
                    &mut view,
                    &mut history,
                    &mut updating,
                    &mut rq,
                );
                if event_ctx.exit_status == ExitStatus::PowerOff {
                    break;
                }
            }
            Event::PrepareSuspend => {
                handle_suspend_event(
                    &mut context,
                    &mut event_ctx,
                    &tx,
                    &mut view,
                    &mut updating,
                    &mut rq,
                );
            }
            Event::Suspend => {
                handle_suspend_execute_event(
                    &mut context,
                    &mut event_ctx,
                    &tx,
                    &mut view,
                    &mut history,
                    &mut updating,
                );
                if event_ctx.exit_status == ExitStatus::PowerOff {
                    break;
                }
            }
            Event::PrepareShare => {
                if context.flags.contains(DeviceFlags::SHARED) {
                    continue;
                }

                event_ctx.tasks.clear();
                view.handle_event(&Event::Back, &tx, &mut bus, &mut rq, &mut context);
                while let Some(mut item) = history.pop() {
                    item.view
                        .handle_event(&Event::Back, &tx, &mut bus, &mut rq, &mut context);
                    if item.rotation != context.display.rotation {
                        wait_for_all(&mut updating, &mut context);
                        if let Ok(dims) = context.fb.set_rotation(item.rotation) {
                            event_ctx
                                .raw_sender
                                .send(display_rotate_event(item.rotation))
                                .ok();
                            context.display.rotation = item.rotation;
                            context.display.dims = dims;
                        }
                    }
                    view = item.view;
                }
                let path = Path::new(SETTINGS_PATH);
                save_toml(&context.settings, path)
                    .map_err(|e| log_error!("Can't save settings: {:#}.", e))
                    .ok();
                context.library.flush();

                if context.settings.frontlight {
                    context.settings.frontlight_levels = context.frontlight.levels();
                    context.frontlight.set_intensity(0.0);
                    context.frontlight.set_warmth(0.0);
                }
                if context.settings.wifi {
                    Command::new("scripts/wifi-disable.sh").status().ok();
                    context.flags.remove(DeviceFlags::ONLINE);
                }

                let interm = Intermission::new(
                    context.fb.rect(),
                    IntermKind::Share,
                    context.settings.sleep_cover_fill,
                    &context,
                );
                rq.add(RenderData::new(
                    interm.id(),
                    *interm.rect(),
                    UpdateMode::Full,
                ));
                view.children_mut().push(Box::new(interm) as Box<dyn View>);
                tx.send(Event::Share).ok();
            }
            Event::Share => {
                if context.flags.contains(DeviceFlags::SHARED) {
                    continue;
                }

                context.flags.insert(DeviceFlags::SHARED);
                Command::new("scripts/usb-enable.sh").status().ok();
            }
            Event::Gesture(ge) => {
                if handle_gesture_event(
                    ge,
                    &mut context,
                    &mut view,
                    &tx,
                    &mut bus,
                    &mut rq,
                    &mut event_ctx,
                    &mut history,
                    &mut updating,
                ) {
                    break;
                }
                handle_event(view.as_mut(), &evt, &tx, &mut bus, &mut rq, &mut context);
            }
            Event::Open(info) => {
                handle_open_event(
                    info,
                    &mut context,
                    &mut view,
                    &tx,
                    &mut bus,
                    &mut rq,
                    &mut event_ctx,
                    &mut history,
                );
            }
            Event::Select(EntryId::About) => {
                let dialog = Dialog::new(
                    ViewId::AboutDialog,
                    None,
                    format!("Plato {}", env!("CARGO_PKG_VERSION")),
                    &mut context,
                );
                rq.add(RenderData::new(
                    dialog.id(),
                    *dialog.rect(),
                    UpdateMode::Gui,
                ));
                view.children_mut().push(Box::new(dialog) as Box<dyn View>);
            }
            Event::Select(EntryId::SystemInfo) => {
                let html = sys_info_as_html();
                if let Ok(r) = Reader::from_html(context.fb.rect(), &html, None, &tx, &mut context)
                {
                    goto_view(
                        Box::new(r),
                        &mut view,
                        &mut history,
                        context.display.rotation,
                        context.fb.monochrome(),
                        context.fb.dithered(),
                        &mut rq,
                        &mut context,
                    );
                }
            }
            Event::OpenHtml(ref html, ref link_uri) => {
                if let Ok(r) = Reader::from_html(
                    context.fb.rect(),
                    html,
                    link_uri.as_deref(),
                    &tx,
                    &mut context,
                ) {
                    goto_view(
                        Box::new(r),
                        &mut view,
                        &mut history,
                        context.display.rotation,
                        context.fb.monochrome(),
                        context.fb.dithered(),
                        &mut rq,
                        &mut context,
                    );
                }
            }
            Event::Select(EntryId::Launch(app_cmd)) => {
                handle_launch(
                    app_cmd,
                    &mut view,
                    &mut tx,
                    &mut rq,
                    &mut context,
                    &mut event_ctx,
                    &mut history,
                );
            }
            #[cfg(any(target_os = "android", target_os = "ios", target_os = "linux"))]
            Event::Select(EntryId::FillForms(ref path)) => {
                match plato_core::view::forms::FormsView::new(
                    context.fb.rect(),
                    path,
                    &mut rq,
                    &mut context,
                ) {
                    Ok(forms_view) => {
                        goto_view(
                            Box::new(forms_view),
                            &mut view,
                            &mut history,
                            context.display.rotation,
                            context.fb.monochrome(),
                            context.fb.dithered(),
                            &mut rq,
                            &mut context,
                        );
                    }
                    Err(e) => {
                        log_error!("Failed to open PDF Forms: {}", e);
                    }
                }
            }
            #[cfg(target_os = "linux")]
            Event::Select(EntryId::SignDocument(ref path)) => {
                match plato_core::view::signatures::SignaturesView::new(
                    context.fb.rect(),
                    path,
                    &mut rq,
                    &mut context,
                ) {
                    Ok(signatures_view) => {
                        goto_view(
                            Box::new(signatures_view),
                            &mut view,
                            &mut history,
                            context.display.rotation,
                            context.fb.monochrome(),
                            context.fb.dithered(),
                            &mut rq,
                            &mut context,
                        );
                    }
                    Err(e) => {
                        log_error!("Failed to open Digital Signatures: {}", e);
                    }
                }
            }
            #[cfg(target_os = "linux")]
            Event::Select(EntryId::ValidatePdfA(ref path, ref level)) => {
                match plato_core::view::validation::ValidationView::new(
                    context.fb.rect(),
                    path,
                    &mut rq,
                    &mut context,
                ) {
                    Ok(mut validation_view) => {
                        if let Err(e) = validation_view.validate_pdfa(level.clone()) {
                            log_error!("Failed to validate PDF/A: {}", e);
                        }
                        goto_view(
                            Box::new(validation_view),
                            &mut view,
                            &mut history,
                            context.display.rotation,
                            context.fb.monochrome(),
                            context.fb.dithered(),
                            &mut rq,
                            &mut context,
                        );
                    }
                    Err(e) => {
                        log_error!("Failed to open PDF Validation: {}", e);
                    }
                }
            }
            #[cfg(target_os = "linux")]
            Event::Select(EntryId::ValidatePdfX(ref path, ref level)) => {
                match plato_core::view::validation::ValidationView::new(
                    context.fb.rect(),
                    path,
                    &mut rq,
                    &mut context,
                ) {
                    Ok(mut validation_view) => {
                        if let Err(e) = validation_view.validate_pdfx(level.clone()) {
                            log_error!("Failed to validate PDF/X: {}", e);
                        }
                        goto_view(
                            Box::new(validation_view),
                            &mut view,
                            &mut history,
                            context.display.rotation,
                            context.fb.monochrome(),
                            context.fb.dithered(),
                            &mut rq,
                            &mut context,
                        );
                    }
                    Err(e) => {
                        log_error!("Failed to open PDF Validation: {}", e);
                    }
                }
            }
            Event::Back => {
                if handle_back_event(
                    &mut view,
                    &mut history,
                    &mut context,
                    &mut event_ctx,
                    &tx,
                    &mut bus,
                    &mut rq,
                ) {
                    break;
                }
            }
            Event::TogglePresetMenu(rect, index) => {
                if let Some(idx) = locate_by_id(view.as_ref(), ViewId::PresetMenu) {
                    let rect = *view.child(idx).rect();
                    view.children_mut().remove(idx);
                    rq.add(RenderData::expose(rect, UpdateMode::Gui));
                } else {
                    let preset_menu = Menu::new(
                        rect,
                        ViewId::PresetMenu,
                        MenuKind::Contextual,
                        vec![EntryKind::Command(
                            "Remove".to_string(),
                            EntryId::RemovePreset(index),
                        )],
                        &mut context,
                    );
                    rq.add(RenderData::new(
                        preset_menu.id(),
                        *preset_menu.rect(),
                        UpdateMode::Gui,
                    ));
                    view.children_mut()
                        .push(Box::new(preset_menu) as Box<dyn View>);
                }
            }
            Event::Show(ViewId::Frontlight) => {
                if !context.settings.frontlight {
                    context.set_frontlight(true);
                    view.handle_event(
                        &Event::ToggleFrontlight,
                        &tx,
                        &mut bus,
                        &mut rq,
                        &mut context,
                    );
                }
                let flw = FrontlightWindow::new(&mut context);
                rq.add(RenderData::new(flw.id(), *flw.rect(), UpdateMode::Gui));
                view.children_mut().push(Box::new(flw) as Box<dyn View>);
            }
            Event::SetWifi(enable) => {
                handle_wifi_event(enable, &mut context);
            }
            Event::Select(EntryId::ToggleWifi) => {
                handle_wifi_event(!context.settings.wifi, &mut context);
            }
            Event::Select(EntryId::TakeScreenshot) => {
                handle_screenshot_event(&mut context, &mut view, &tx, &mut rq);
            }
            Event::CheckFetcher(..)
            | Event::FetcherAddDocument(..)
            | Event::FetcherRemoveDocument(..)
            | Event::FetcherSearch { .. }
                if !view.is::<Home>() =>
            {
                if let Some(entry) = history.get_mut(0).filter(|entry| entry.view.is::<Home>()) {
                    let (tx, _rx) = mpsc::channel();
                    entry.view.handle_event(
                        &evt,
                        &tx,
                        &mut VecDeque::new(),
                        &mut RenderQueue::new(),
                        &mut context,
                    );
                }
            }
            Event::Notify(msg) => {
                let notif = Notification::new(msg, &tx, &mut rq, &mut context);
                view.children_mut().push(Box::new(notif) as Box<dyn View>);
            }
            Event::Select(EntryId::Reboot) => {
                event_ctx.exit_status = ExitStatus::Reboot;
                break;
            }
            Event::Select(EntryId::Quit) => {
                // Trigger shutdown plugins
                if let Err(e) = context.plugin_system.on_shutdown() {
                    log_error!("Failed to trigger shutdown plugins: {}", e);
                }
                break;
            }
            Event::MightSuspend if context.settings.auto_suspend > 0.0 => {
                if handle_suspend_check_event(
                    &mut context,
                    &mut event_ctx,
                    &mut view,
                    &tx,
                    &mut bus,
                    &mut rq,
                ) {
                    continue;
                }
            }
            _ => {
                handle_event(view.as_mut(), &evt, &tx, &mut bus, &mut rq, &mut context);
            }
        }

        process_render_queue(view.as_ref(), &mut rq, &mut context, &mut updating);

        while let Some(ce) = bus.pop_front() {
            tx.send(ce).ok();
        }
    }

    if event_ctx.exit_status == ExitStatus::Quit
        && !CURRENT_DEVICE.has_gyroscope()
        && context.display.rotation != initial_rotation
    {
        context.fb.set_rotation(initial_rotation).ok();
    }

    if event_ctx
        .tasks
        .iter()
        .all(|task| task.id != TaskId::Suspend)
    {
        if context.settings.frontlight {
            context.settings.frontlight_levels = context.frontlight.levels();
        }
    }

    context.library.flush();

    let path = Path::new(SETTINGS_PATH);
    save_toml(&context.settings, path).context("can't save settings")?;

    match exit_status {
        ExitStatus::Reboot => {
            File::create("./tmp/reboot").ok();
        }
        ExitStatus::PowerOff => {
            File::create("./tmp/power_off").ok();
        }
        _ => (),
    }

    Ok(())
}
