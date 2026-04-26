//! Event handling for the main application loop.

use crate::constants::{BATTERY_REFRESH_INTERVAL, KOBO_UPDATE_BUNDLE, PREPARE_SUSPEND_WAIT_DELAY};
use crate::helpers::{goto_view, ExitStatus, HistoryItem};
use crate::task::{resume, schedule_task, Task, TaskId};

use plato_core::context::DeviceFlags;
use plato_core::device::{Orientation, CURRENT_DEVICE};
use plato_core::framebuffer::UpdateMode;
use plato_core::helpers::load_toml;
use plato_core::input::{ButtonCode, ButtonStatus, DeviceEvent, InputEvent, PowerSource};
use plato_core::log_error;
use plato_core::settings::{IntermKind, RotationLock, Settings, SETTINGS_PATH};
use plato_core::theme;
use plato_core::view::calculator::Calculator;
use plato_core::view::common::locate;
use plato_core::view::cover_editor::CoverEditorView;
use plato_core::view::dialog::Dialog;
use plato_core::view::dictionary::Dictionary;
use plato_core::view::epub_editor::EpubEditor;
use plato_core::view::intermission::Intermission;
use plato_core::view::notification::Notification;
use plato_core::view::opds::OpdsView;
use plato_core::view::pdf_manipulator::PdfManipulatorView;
use plato_core::view::rotation_values::RotationValues;
use plato_core::view::sketch::Sketch;
use plato_core::view::statistics::StatisticsView;
use plato_core::view::touch_events::TouchEvents;
use plato_core::view::{
    handle_event, AppCmd, EntryId, Event, RenderData, RenderQueue, UpdateData, View, ViewId,
};
use std::collections::VecDeque;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::time::Instant;

/// Shared context for event handlers.
pub struct EventContext {
    pub tasks: Vec<Task>,
    pub inactive_since: Instant,
    pub exit_status: ExitStatus,
    pub raw_sender: mpsc::Sender<InputEvent>,
    pub current_dir: PathBuf,
}

impl EventContext {
    pub fn new(
        tasks: Vec<Task>,
        inactive_since: Instant,
        exit_status: ExitStatus,
        raw_sender: mpsc::Sender<InputEvent>,
        current_dir: PathBuf,
    ) -> Self {
        Self {
            tasks,
            inactive_since,
            exit_status,
            raw_sender,
            current_dir,
        }
    }
}

/// Handle device events.
pub fn handle_device_event(
    de: DeviceEvent,
    view: &mut dyn View,
    tx: &mpsc::Sender<Event>,
    bus: &mut VecDeque<Event>,
    rq: &mut RenderQueue,
    context: &mut plato_core::context::Context,
    ctx: &mut EventContext,
    history: &mut Vec<HistoryItem>,
    _updating: &mut Vec<UpdateData>,
    evt: &Event,
) -> bool {
    match de {
        DeviceEvent::Button {
            code: ButtonCode::Power,
            status: ButtonStatus::Released,
            ..
        } => {
            if context.flags.contains(DeviceFlags::SHARED)
                || context.flags.contains(DeviceFlags::COVERED)
            {
                return true; // continue
            }

            if ctx
                .tasks
                .iter()
                .any(|task| task.id == TaskId::PrepareSuspend)
            {
                resume(
                    TaskId::PrepareSuspend,
                    &mut ctx.tasks,
                    view,
                    tx,
                    rq,
                    context,
                );
            } else if ctx.tasks.iter().any(|task| task.id == TaskId::Suspend) {
                resume(TaskId::Suspend, &mut ctx.tasks, view, tx, rq, context);
            } else {
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
                    PREPARE_SUSPEND_WAIT_DELAY,
                    tx,
                    &mut ctx.tasks,
                );
                view.children_mut().push(Box::new(interm) as Box<dyn View>);
            }
        }
        DeviceEvent::Button {
            code: ButtonCode::Light,
            status: ButtonStatus::Pressed,
            ..
        } => {
            tx.send(Event::ToggleFrontlight).ok();
        }
        DeviceEvent::Button {
            code: ButtonCode::Light,
            status: ButtonStatus::Released,
            ..
        } => {
            tx.send(Event::ToggleFrontlight).ok();
        }
        DeviceEvent::CoverOn => {
            if context.flags.contains(DeviceFlags::COVERED) {
                return true; // continue
            }

            context.flags.insert(DeviceFlags::COVERED);

            if !context.settings.sleep_cover
                || context.flags.contains(DeviceFlags::SHARED)
                || ctx
                    .tasks
                    .iter()
                    .any(|task| task.id == TaskId::PrepareSuspend || task.id == TaskId::Suspend)
            {
                return true; // continue
            }

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
                PREPARE_SUSPEND_WAIT_DELAY,
                tx,
                &mut ctx.tasks,
            );
            view.children_mut().push(Box::new(interm) as Box<dyn View>);
        }
        DeviceEvent::CoverOff => {
            if !context.flags.contains(DeviceFlags::COVERED) {
                return true; // continue
            }

            context.flags.remove(DeviceFlags::COVERED);

            if context.flags.contains(DeviceFlags::SHARED) || !context.settings.sleep_cover {
                return true; // continue
            }

            if ctx
                .tasks
                .iter()
                .any(|task| task.id == TaskId::PrepareSuspend)
            {
                resume(
                    TaskId::PrepareSuspend,
                    &mut ctx.tasks,
                    view,
                    tx,
                    rq,
                    context,
                );
            } else if ctx.tasks.iter().any(|task| task.id == TaskId::Suspend) {
                resume(TaskId::Suspend, &mut ctx.tasks, view, tx, rq, context);
            }
        }
        DeviceEvent::NetUp => {
            if ctx
                .tasks
                .iter()
                .any(|task| task.id == TaskId::PrepareSuspend || task.id == TaskId::Suspend)
            {
                return true; // continue
            }
            let ip = Command::new("scripts/ip.sh")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim_end().to_string())
                .unwrap_or_default();
            let essid = Command::new("scripts/essid.sh")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim_end().to_string())
                .unwrap_or_default();
            let notif = Notification::new(
                format!("Network is up ({}, {}).", ip, essid),
                tx,
                rq,
                context,
            );
            context.flags.insert(DeviceFlags::ONLINE);
            view.children_mut().push(Box::new(notif) as Box<dyn View>);
            if view.is::<plato_core::view::home::Home>() {
                view.handle_event(evt, tx, bus, rq, context);
            } else if let Some(entry) = history
                .get_mut(0)
                .filter(|entry| entry.view.is::<plato_core::view::home::Home>())
            {
                let (tx, _rx) = mpsc::channel();
                entry.view.handle_event(
                    evt,
                    &tx,
                    &mut VecDeque::new(),
                    &mut RenderQueue::new(),
                    context,
                );
            }
        }
        DeviceEvent::Plug(power_source) => {
            if context.flags.contains(DeviceFlags::PLUGGED) {
                return true; // continue
            }

            context.flags.insert(DeviceFlags::PLUGGED);

            ctx.tasks.retain(|task| task.id != TaskId::CheckBattery);

            if context.flags.contains(DeviceFlags::COVERED) {
                return true; // continue
            }

            match power_source {
                PowerSource::Wall => {
                    if ctx.tasks.iter().any(|task| task.id == TaskId::Suspend) {
                        return true; // continue
                    }
                }
                PowerSource::Host => {
                    if ctx
                        .tasks
                        .iter()
                        .any(|task| task.id == TaskId::PrepareSuspend)
                    {
                        resume(
                            TaskId::PrepareSuspend,
                            &mut ctx.tasks,
                            view,
                            tx,
                            rq,
                            context,
                        );
                    } else if ctx.tasks.iter().any(|task| task.id == TaskId::Suspend) {
                        resume(TaskId::Suspend, &mut ctx.tasks, view, tx, rq, context);
                    }

                    if context.settings.auto_share {
                        tx.send(Event::PrepareShare).ok();
                    } else {
                        let dialog = Dialog::new(
                            ViewId::ShareDialog,
                            Some(Event::PrepareShare),
                            "Share storage via USB?".to_string(),
                            context,
                        );
                        rq.add(RenderData::new(
                            dialog.id(),
                            *dialog.rect(),
                            UpdateMode::Gui,
                        ));
                        view.children_mut().push(Box::new(dialog) as Box<dyn View>);
                    }

                    ctx.inactive_since = Instant::now();
                }
            }

            tx.send(Event::BatteryTick).ok();
        }
        DeviceEvent::Unplug(..) => {
            if !context.flags.contains(DeviceFlags::PLUGGED) {
                return true; // continue
            }

            if context.flags.contains(DeviceFlags::SHARED) {
                context.flags.remove(DeviceFlags::SHARED);
                Command::new("scripts/usb-disable.sh").status().ok();
                env::set_current_dir(&ctx.current_dir)
                    .map_err(|e| {
                        log_error!(
                            "Can't set current directory to {}: {:#}.",
                            ctx.current_dir.display(),
                            e
                        )
                    })
                    .ok();
                let path = Path::new(SETTINGS_PATH);
                if let Ok(settings) = load_toml::<Settings, _>(path)
                    .map_err(|e| log_error!("Can't load settings: {:#}.", e))
                {
                    let dark_mode = settings.dark_mode;
                    context.settings = settings;
                    theme::set_theme_mode(context.settings.theme_settings.mode);
                    theme::set_auto_threshold(context.settings.theme_settings.auto_threshold);
                    theme::set_dark_mode(dark_mode);
                }
                if context.settings.wifi {
                    Command::new("scripts/wifi-enable.sh").status().ok();
                }
                if context.settings.frontlight {
                    let levels = context.settings.frontlight_levels;
                    context.frontlight.set_warmth(levels.warmth);
                    context.frontlight.set_intensity(levels.intensity);
                }
                if let Some(index) = locate::<Intermission>(view) {
                    let rect = *view.child(index).rect();
                    view.children_mut().remove(index);
                    rq.add(RenderData::expose(rect, UpdateMode::Full));
                }
                if Path::new(KOBO_UPDATE_BUNDLE).exists() {
                    tx.send(Event::Select(EntryId::Reboot)).ok();
                }
                context.library.reload();
                if context.settings.import.unshare_trigger {
                    context.batch_import();
                    // Trigger book import plugins
                    if let Err(e) = context
                        .plugin_system
                        .on_book_import(&Path::new("/mnt/onboard"))
                    {
                        log_error!("Failed to trigger book import plugins: {}", e);
                    }
                }
                view.handle_event(&Event::Reseed, tx, bus, rq, context);
            } else {
                context.flags.remove(DeviceFlags::PLUGGED);
                schedule_task(
                    TaskId::CheckBattery,
                    Event::CheckBattery,
                    BATTERY_REFRESH_INTERVAL,
                    tx,
                    &mut ctx.tasks,
                );
                if ctx.tasks.iter().any(|task| task.id == TaskId::Suspend) {
                    if !context.flags.contains(DeviceFlags::COVERED) {
                        resume(TaskId::Suspend, &mut ctx.tasks, view, tx, rq, context);
                    }
                } else {
                    tx.send(Event::BatteryTick).ok();
                }
            }
        }
        DeviceEvent::RotateScreen(n) => {
            if context.flags.contains(DeviceFlags::SHARED)
                || ctx
                    .tasks
                    .iter()
                    .any(|task| task.id == TaskId::PrepareSuspend || task.id == TaskId::Suspend)
            {
                return true; // continue
            }

            if view.is::<RotationValues>() {
                println!("Gyro rotation: {}", n);
            }

            if let Some(rotation_lock) = context.settings.rotation_lock {
                let orientation = CURRENT_DEVICE.orientation(n);
                if rotation_lock == RotationLock::Current
                    || (rotation_lock == RotationLock::Portrait
                        && orientation == Orientation::Landscape)
                    || (rotation_lock == RotationLock::Landscape
                        && orientation == Orientation::Portrait)
                {
                    return true; // continue
                }
            }

            tx.send(Event::Select(EntryId::Rotate(n))).ok();
        }
        DeviceEvent::UserActivity if context.settings.auto_suspend > 0.0 => {
            ctx.inactive_since = Instant::now();
        }
        _ => {
            handle_event(view, evt, tx, bus, rq, context);
        }
    }
    false
}

/// Handle launch events for different applications.
pub fn handle_launch(
    app_cmd: AppCmd,
    view: &mut Box<dyn View>,
    tx: &mpsc::Sender<Event>,
    rq: &mut RenderQueue,
    context: &mut plato_core::context::Context,
    _ctx: &mut EventContext,
    history: &mut Vec<HistoryItem>,
) -> bool {
    let rotation = context.display.rotation;
    let monochrome = context.fb.monochrome();
    let dithered = context.fb.dithered();

    let next_view: Option<Box<dyn View>> = match app_cmd {
        AppCmd::Sketch => {
            context.fb.set_monochrome(true);
            Sketch::new(context.fb.rect(), rq, context)
                .map(|v| Box::new(v) as Box<dyn View>)
                .ok()
        }
        AppCmd::Calculator => Calculator::new(context.fb.rect(), tx, rq, context)
            .map(|v| Box::new(v) as Box<dyn View>)
            .ok(),
        AppCmd::Dictionary {
            ref query,
            ref language,
        } => Dictionary::new(context.fb.rect(), query, language, tx, rq, context)
            .map(|v| Some(Box::new(v) as Box<dyn View>))
            .unwrap_or(None),
        AppCmd::EpubEditor { ref path, chapter } => {
            EpubEditor::new(context.fb.rect(), path.clone(), chapter, tx, rq, context)
                .map(|v| Box::new(v) as Box<dyn View>)
                .ok()
        }
        AppCmd::CoverEditor => Some(Box::new(CoverEditorView::new(
            context.fb.rect(),
            rq,
            context,
        ))),
        AppCmd::Statistics => Some(Box::new(StatisticsView::new(
            context.fb.rect(),
            rq,
            context,
        ))),
        AppCmd::PdfManipulator => match PdfManipulatorView::new(context.fb.rect(), rq, context) {
            Ok(view) => Some(Box::new(view) as Box<dyn View>),
            Err(e) => {
                log_error!("Failed to open PDF Tools: {}", e);
                None
            }
        },
        AppCmd::OpenPdfManipulator(ref path) => {
            match PdfManipulatorView::for_file(context.fb.rect(), path.clone(), rq, context) {
                Ok(view) => Some(Box::new(view) as Box<dyn View>),
                Err(e) => {
                    log_error!("Failed to open PDF Tools for file: {}", e);
                    None
                }
            }
        }
        AppCmd::TouchEvents => Some(Box::new(TouchEvents::new(context.fb.rect(), rq, context))),
        AppCmd::RotationValues => Some(Box::new(RotationValues::new(
            context.fb.rect(),
            rq,
            context,
        ))),
        AppCmd::OpenCoverEditor(ref path) => {
            match CoverEditorView::for_book(context.fb.rect(), path.clone(), rq, context) {
                Ok(view) => Some(Box::new(view) as Box<dyn View>),
                Err(e) => {
                    log_error!("Failed to open Cover Editor: {}", e);
                    None
                }
            }
        }
        AppCmd::Opds { ref url } => {
            let url = url
                .as_ref()
                .cloned()
                .unwrap_or_else(|| context.settings.opds.catalogs[0].url.clone());
            Some(Box::new(OpdsView::new(context.fb.rect(), url, context)) as Box<dyn View>)
        }
    };

    if let Some(next_view) = next_view {
        goto_view(
            next_view, view, history, rotation, monochrome, dithered, rq, context,
        );
        true
    } else {
        false
    }
}
