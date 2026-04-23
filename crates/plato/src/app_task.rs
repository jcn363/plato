//! Task scheduling and management for background operations.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use plato_core::view::{Event, RenderQueue, View};
use plato_core::view::intermission::Intermission;
use plato_core::view::intermission::IntermKind;
use plato_core::view::wait_for_all;
use plato_core::context::Context;
use plato_core::framebuffer::UpdateMode;
use plato_core::view::common::locate;
use plato_core::view::RenderData;
use plato_core::log_warn;

/// A background task with a completion channel.
pub struct Task {
    /// The task identifier.
    pub id: TaskId,
    /// Channel receiver for task completion.
    pub _chan: Receiver<()>,
}

/// Task identifiers for scheduled background operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TaskId {
    /// Check battery level task.
    CheckBattery,
    /// Prepare for suspend task.
    PrepareSuspend,
    /// Suspend device task.
    Suspend,
}

/// Schedule a task to run after a delay.
pub fn schedule_task(
    id: TaskId,
    event: Event,
    delay: Duration,
    hub: &Sender<Event>,
    tasks: &mut Vec<Task>,
) {
    let (ty, ry) = mpsc::channel();
    let hub2 = hub.clone();
    tasks.retain(|task| task.id != id);
    tasks.push(Task { id, _chan: ry });
    thread::spawn(move || {
        thread::sleep(delay);
        if ty.send(()).is_ok() {
            hub2.send(event).ok();
        }
    });
}

/// Resume from suspend state.
pub fn resume(
    id: TaskId,
    tasks: &mut Vec<Task>,
    view: &mut dyn View,
    hub: &Sender<Event>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if id == TaskId::Suspend {
        tasks.retain(|task| task.id != TaskId::Suspend);
        if context.settings.frontlight {
            let levels = context.settings.frontlight_levels;
            context.frontlight.set_warmth(levels.warmth);
            context.frontlight.set_intensity(levels.intensity);
        }
        if context.settings.wifi {
            std::process::Command::new("scripts/wifi-enable.sh").status().ok();
        }
    }
    if id == TaskId::Suspend || id == TaskId::PrepareSuspend {
        tasks.retain(|task| task.id != TaskId::PrepareSuspend);
        if let Some(index) = locate::<Intermission>(view) {
            let rect = *view.child(index).rect();
            view.children_mut().remove(index);
            rq.add(RenderData::expose(rect, UpdateMode::Full));
        }
        hub.send(Event::ClockTick).ok();
        hub.send(Event::BatteryTick).ok();
    }
}
