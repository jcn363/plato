use crate::constants::BATTERY_REFRESH_INTERVAL;
use crate::event::EventContext;
use crate::helpers::{power_off, set_wifi, HistoryItem};
use crate::task::{schedule_task, TaskId};

use plato_core::chrono::Local;
use plato_core::context::DeviceFlags;
use plato_core::device::CURRENT_DEVICE;
use plato_core::framebuffer::UpdateMode;
use plato_core::geom::{DiagDir, Region};
use plato_core::gesture::GestureEvent;
use plato_core::input::{display_rotate_event, ButtonCode};
use plato_core::log_error;
use plato_core::settings::{IntermKind, ThemeMode};
use plato_core::theme;
use plato_core::view::home::Home;
use plato_core::view::intermission::Intermission;
use plato_core::view::notification::Notification;
use plato_core::view::reader::Reader;
use plato_core::view::{handle_event, wait_for_all};
use plato_core::view::{EntryId, Event, RenderData, RenderQueue, View};
use std::collections::VecDeque;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

pub(crate) fn handle_battery_event(
    context: &mut plato_core::context::Context,
    event_ctx: &mut EventContext,
    tx: &mpsc::Sender<Event>,
    view: &mut Box<dyn View>,
    history: &mut Vec<HistoryItem>,
    updating: &mut Vec<plato_core::view::UpdateData>,
    rq: &mut RenderQueue,
) {
    schedule_task(
        TaskId::CheckBattery,
        Event::CheckBattery,
        BATTERY_REFRESH_INTERVAL,
        tx,
        &mut event_ctx.tasks,
    );
    if event_ctx
        .tasks
        .iter()
        .any(|task| task.id == TaskId::PrepareSuspend || task.id == TaskId::Suspend)
    {
        return;
    }
    if let Ok(v) = context.battery.capacity().map(|v| v[0]) {
        if v < context.settings.battery.power_off {
            power_off(view.as_mut(), history, updating, context);
            event_ctx.exit_status = crate::helpers::ExitStatus::PowerOff;
        } else if v < context.settings.battery.warn {
            let notif = Notification::new(
                "The battery capacity is getting low.".to_string(),
                tx,
                rq,
                context,
            );
            view.children_mut().push(Box::new(notif) as Box<dyn View>);
        }
    }

    if context.settings.theme_settings.mode == ThemeMode::Auto && CURRENT_DEVICE.has_lightsensor() {
        if let Ok(level) = context.lightsensor.level() {
            theme::update_from_light_sensor(level);
            if theme::is_dark_mode() != context.settings.dark_mode {
                context.settings.dark_mode = theme::is_dark_mode();
            }
        }
    }

    if context.settings.theme_settings.mode == ThemeMode::Scheduled
        && context.settings.theme_settings.schedule.enabled
    {
        let now = Local::now();
        theme::update_from_schedule(&context.settings.theme_settings.schedule, &now);
        if theme::is_dark_mode() != context.settings.dark_mode {
            context.settings.dark_mode = theme::is_dark_mode();
        }
    }
}

pub(crate) fn handle_suspend_event(
    context: &mut plato_core::context::Context,
    event_ctx: &mut EventContext,
    tx: &mpsc::Sender<Event>,
    _view: &mut Box<dyn View>,
    updating: &mut Vec<plato_core::view::UpdateData>,
    _rq: &mut RenderQueue,
) {
    event_ctx
        .tasks
        .retain(|task| task.id != TaskId::PrepareSuspend);
    wait_for_all(updating, context);
    let path = Path::new(plato_core::settings::SETTINGS_PATH);
    plato_core::helpers::save_toml(&context.settings, path)
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
    schedule_task(
        TaskId::Suspend,
        Event::Suspend,
        crate::constants::SUSPEND_WAIT_DELAY,
        tx,
        &mut event_ctx.tasks,
    );
}

pub(crate) fn handle_suspend_execute_event(
    context: &mut plato_core::context::Context,
    event_ctx: &mut EventContext,
    tx: &mpsc::Sender<Event>,
    view: &mut Box<dyn View>,
    history: &mut Vec<HistoryItem>,
    updating: &mut Vec<plato_core::view::UpdateData>,
) {
    if context.settings.auto_power_off > 0.0 {
        context.rtc.iter().for_each(|rtc| {
            rtc.set_alarm(context.settings.auto_power_off)
                .map_err(|e| log_error!("Can't set alarm: {:#}.", e))
                .ok();
        });
    }
    let before = Local::now();
    println!(
        "{}",
        before.format("Went to sleep on %B %-d, %Y at %H:%M:%S.")
    );
    Command::new("scripts/suspend.sh").status().ok();
    let after = Local::now();
    println!("{}", after.format("Woke up on %B %-d, %Y at %H:%M:%S."));
    Command::new("scripts/resume.sh").status().ok();
    event_ctx.inactive_since = std::time::Instant::now();
    schedule_task(
        TaskId::Suspend,
        Event::Suspend,
        crate::constants::SUSPEND_WAIT_DELAY,
        tx,
        &mut event_ctx.tasks,
    );
    if context.settings.auto_power_off > 0.0 {
        let dur = plato_core::chrono::Duration::seconds(
            (86_400.0 * context.settings.auto_power_off) as i64,
        );
        if let Some(fired) = context.rtc.as_ref().and_then(|rtc| {
            rtc.alarm()
                .map_err(|e| log_error!("Can't get alarm: {:#}", e))
                .map(|rwa| {
                    !rwa.enabled()
                        || (rwa.year() <= 1970 && ((after - before) - dur).num_seconds().abs() < 3)
                })
                .ok()
        }) {
            if fired {
                power_off(view.as_mut(), history, updating, context);
                event_ctx.exit_status = crate::helpers::ExitStatus::PowerOff;
            } else {
                context.rtc.iter().for_each(|rtc| {
                    rtc.disable_alarm()
                        .map_err(|e| log_error!("Can't disable alarm: {:#}.", e))
                        .ok();
                });
            }
        }
    }
}

pub(crate) fn handle_gesture_event(
    ge: GestureEvent,
    context: &mut plato_core::context::Context,
    view: &mut Box<dyn View>,
    tx: &mpsc::Sender<Event>,
    _bus: &mut VecDeque<Event>,
    rq: &mut RenderQueue,
    event_ctx: &mut EventContext,
    history: &mut Vec<HistoryItem>,
    updating: &mut Vec<plato_core::view::UpdateData>,
) -> bool {
    match ge {
        GestureEvent::HoldButtonLong(ButtonCode::Power) => {
            power_off(view.as_mut(), history, updating, context);
            event_ctx.exit_status = crate::helpers::ExitStatus::PowerOff;
            true
        }
        GestureEvent::MultiSwipe {
            dir: _,
            starts,
            ends: _,
        } => {
            if context.settings.theme_settings.mode == ThemeMode::Auto
                || context.settings.theme_settings.mode == ThemeMode::Scheduled
            {
                let width = context.fb.dims().0 as i32;
                if starts[0].x < width / 4 {
                    context.settings.theme_settings.mode = ThemeMode::Dark;
                    theme::set_theme_mode(ThemeMode::Dark);
                    theme::set_dark_mode(true);
                    context.settings.dark_mode = true;
                } else if starts[0].x > (width * 3) / 4 {
                    context.settings.theme_settings.mode = ThemeMode::Sepia;
                    theme::set_theme_mode(ThemeMode::Sepia);
                    theme::set_dark_mode(false);
                    context.settings.dark_mode = false;
                }
            }
            false
        }
        GestureEvent::MultiTap(mut points) => {
            if points[0].x > points[1].x {
                points.swap(0, 1);
            }
            let rect = context.fb.rect();
            let r1 = Region::from_point(
                points[0],
                rect,
                context.settings.reader.strip_width,
                context.settings.reader.corner_width,
            );
            let r2 = Region::from_point(
                points[1],
                rect,
                context.settings.reader.strip_width,
                context.settings.reader.corner_width,
            );
            match (r1, r2) {
                (Region::Corner(DiagDir::SouthWest), Region::Corner(DiagDir::NorthEast)) => {
                    rq.add(RenderData::new(
                        view.id(),
                        context.fb.rect(),
                        UpdateMode::Full,
                    ));
                }
                (Region::Corner(DiagDir::NorthWest), Region::Corner(DiagDir::SouthEast)) => {
                    tx.send(Event::Select(EntryId::TakeScreenshot)).ok();
                }
                _ => (),
            }
            false
        }
        _ => false,
    }
}

pub(crate) fn handle_open_event(
    info: Box<plato_core::metadata::Info>,
    context: &mut plato_core::context::Context,
    view: &mut Box<dyn View>,
    tx: &mpsc::Sender<Event>,
    bus: &mut VecDeque<Event>,
    rq: &mut RenderQueue,
    event_ctx: &mut EventContext,
    history: &mut Vec<HistoryItem>,
) {
    let rotation = context.display.rotation;
    let dithered = context.fb.dithered();
    if let Some(reader_info) = info.reader.as_ref() {
        if let Some(n) = reader_info
            .rotation
            .map(|n| CURRENT_DEVICE.canonical_to_device(n))
        {
            if CURRENT_DEVICE.orientation(n) != CURRENT_DEVICE.orientation(rotation) {
                wait_for_all(&mut vec![], context);
                if let Ok(dims) = context.fb.set_rotation(n) {
                    event_ctx.raw_sender.send(display_rotate_event(n)).ok();
                    context.display.rotation = n;
                    context.display.dims = dims;
                }
            }
        }
        context.fb.set_dithered(reader_info.dithered);
    } else {
        context.fb.set_dithered(
            context
                .settings
                .reader
                .dithered_kinds
                .contains(&info.file.kind),
        );
    }
    let path = info.file.path.clone();
    if let Err(e) = context.plugin_system.on_book_open(&path) {
        log_error!("Failed to trigger book open plugins: {}", e);
    }

    if let Some(r) = Reader::new(context.fb.rect(), *info, tx, context) {
        crate::helpers::goto_view(
            Box::new(r),
            view,
            history,
            rotation,
            context.fb.monochrome(),
            dithered,
            rq,
            context,
        );
    } else {
        if context.display.rotation != rotation {
            if let Ok(dims) = context.fb.set_rotation(rotation) {
                event_ctx
                    .raw_sender
                    .send(display_rotate_event(rotation))
                    .ok();
                context.display.rotation = rotation;
                context.display.dims = dims;
            }
        }
        context.fb.set_dithered(dithered);
        handle_event(view.as_mut(), &Event::Invalid(path), tx, bus, rq, context);
    }
}

pub(crate) fn handle_back_event(
    view: &mut Box<dyn View>,
    history: &mut Vec<HistoryItem>,
    context: &mut plato_core::context::Context,
    event_ctx: &mut EventContext,
    tx: &mpsc::Sender<Event>,
    bus: &mut VecDeque<Event>,
    rq: &mut RenderQueue,
) -> bool {
    if let Some(item) = history.pop() {
        *view = item.view;
        if item.monochrome != context.fb.monochrome() {
            context.fb.set_monochrome(item.monochrome);
        }
        if item.dithered != context.fb.dithered() {
            context.fb.set_dithered(item.dithered);
        }
        if CURRENT_DEVICE.orientation(item.rotation)
            != CURRENT_DEVICE.orientation(context.display.rotation)
        {
            wait_for_all(&mut vec![], context);
            if let Ok(dims) = context.fb.set_rotation(item.rotation) {
                event_ctx
                    .raw_sender
                    .send(display_rotate_event(item.rotation))
                    .ok();
                context.display.rotation = item.rotation;
                context.display.dims = dims;
            }
        }
        view.handle_event(&Event::Reseed, tx, bus, rq, context);
        false
    } else if !view.is::<Home>() {
        true
    } else {
        false
    }
}

pub(crate) fn handle_wifi_event(enable: bool, context: &mut plato_core::context::Context) {
    set_wifi(enable, context);
}

pub(crate) fn handle_screenshot_event(
    context: &mut plato_core::context::Context,
    view: &mut Box<dyn View>,
    tx: &mpsc::Sender<Event>,
    rq: &mut RenderQueue,
) {
    let name = Local::now().format("screenshot-%Y%m%d_%H%M%S.png");
    let msg = match context.fb.save(&name.to_string()) {
        Err(e) => format!("{}", e),
        Ok(_) => format!("Saved {}.", name),
    };
    let notif = Notification::new(msg, tx, rq, context);
    view.children_mut().push(Box::new(notif) as Box<dyn View>);
}

pub(crate) fn handle_suspend_check_event(
    context: &mut plato_core::context::Context,
    event_ctx: &mut EventContext,
    view: &mut Box<dyn View>,
    tx: &mpsc::Sender<Event>,
    bus: &mut VecDeque<Event>,
    rq: &mut RenderQueue,
) -> bool {
    if context.flags.contains(DeviceFlags::SHARED)
        || event_ctx
            .tasks
            .iter()
            .any(|task| task.id == TaskId::PrepareSuspend || task.id == TaskId::Suspend)
    {
        event_ctx.inactive_since = std::time::Instant::now();
        return false;
    }
    let seconds = 60.0 * context.settings.auto_suspend;
    if event_ctx.inactive_since.elapsed() > Duration::from_secs_f32(seconds) {
        view.handle_event(&Event::Suspend, tx, bus, rq, context);
        let interm = Intermission::new(
            context.fb.rect(),
            IntermKind::Suspend,
            context.settings.sleep_cover_fill,
            context,
        );
        rq.add(RenderData::new(
            interm.id(),
            *interm.rect(),
            UpdateMode::Full,
        ));
        schedule_task(
            TaskId::PrepareSuspend,
            Event::PrepareSuspend,
            crate::constants::PREPARE_SUSPEND_WAIT_DELAY,
            tx,
            &mut event_ctx.tasks,
        );
        view.children_mut().push(Box::new(interm) as Box<dyn View>);
    }
    false
}
