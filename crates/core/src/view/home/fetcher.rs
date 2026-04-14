//! Home Fetcher Operations
//!
//! Handles background fetcher/process management:
//! - terminate_fetchers()
//! - insert_fetcher()
//! - spawn_child()
//! - reseed()

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;

use anyhow::{format_err, Error};

use crate::context::Context;
use crate::context::DeviceFlags;
use crate::framebuffer::UpdateMode;
use crate::settings::Hook;
use crate::view::home::Fetcher;
use crate::view::top_bar::TopBar;
use crate::view::{EntryId, Event, Hub, RenderData, RenderQueue, View};

use super::Home;

impl Home {
    pub(crate) fn terminate_fetchers(
        &mut self,
        path: &Path,
        update: bool,
        hub: &Hub,
        context: &mut Context,
    ) {
        self.background_fetchers.retain(|id, fetcher| {
            if fetcher.full_path == path {
                unsafe { libc::kill(*id as libc::pid_t, libc::SIGTERM) };
                fetcher.process.wait().ok();
                if update {
                    if let Some(sort_method) = fetcher.sort_method {
                        hub.send(Event::Select(EntryId::Sort(sort_method))).ok();
                    }
                    if let Some(first_column) = fetcher.first_column {
                        hub.send(Event::Select(EntryId::FirstColumn(first_column)))
                            .ok();
                    }
                    if let Some(second_column) = fetcher.second_column {
                        hub.send(Event::Select(EntryId::SecondColumn(second_column)))
                            .ok();
                    }
                } else {
                    let selected_library = context.settings.selected_library;
                    if let Some(sort_method) = fetcher.sort_method {
                        context.settings.libraries[selected_library].sort_method = sort_method;
                    }
                    if let Some(first_column) = fetcher.first_column {
                        context.settings.libraries[selected_library].first_column = first_column;
                    }
                    if let Some(second_column) = fetcher.second_column {
                        context.settings.libraries[selected_library].second_column = second_column;
                    }
                }
                false
            } else {
                true
            }
        });
    }

    pub(crate) fn insert_fetcher(&mut self, hook: &Hook, hub: &Hub, context: &Context) {
        let library_path = &context.library.home;
        let save_path = context.library.home.join(&hook.path);
        match self.spawn_child(
            library_path,
            &save_path,
            &hook.program,
            context.settings.wifi,
            context.flags.contains(DeviceFlags::ONLINE),
            hub,
        ) {
            Ok(process) => {
                let mut sort_method = hook.sort_method;
                let mut first_column = hook.first_column;
                let mut second_column = hook.second_column;
                if let Some(sort_method) = sort_method.replace(self.sort_method) {
                    hub.send(Event::Select(EntryId::Sort(sort_method))).ok();
                }
                let selected_library = context.settings.selected_library;
                if let Some(first_column) =
                    first_column.replace(context.settings.libraries[selected_library].first_column)
                {
                    hub.send(Event::Select(EntryId::FirstColumn(first_column)))
                        .ok();
                }
                if let Some(second_column) = second_column
                    .replace(context.settings.libraries[selected_library].second_column)
                {
                    hub.send(Event::Select(EntryId::SecondColumn(second_column)))
                        .ok();
                }
                self.background_fetchers.insert(
                    process.id(),
                    Fetcher {
                        path: hook.path.clone(),
                        full_path: save_path,
                        process,
                        sort_method,
                        first_column,
                        second_column,
                    },
                );
            }
            Err(e) => crate::log_error!("Can't spawn child: {:#}.", e),
        }
    }

    pub(crate) fn spawn_child(
        &mut self,
        library_path: &Path,
        save_path: &Path,
        program: &Path,
        wifi: bool,
        online: bool,
        hub: &Hub,
    ) -> Result<Child, Error> {
        let path = program.canonicalize()?;
        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        let mut process = Command::new(&path)
            .current_dir(parent)
            .arg(library_path)
            .arg(save_path)
            .arg(wifi.to_string())
            .arg(online.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| format_err!("can't take stdout"))?;
        let id = process.id();
        let hub2 = hub.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line_res in reader.lines() {
                if let Ok(line) = line_res {
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) {
                        match event.get("type").and_then(serde_json::Value::as_str) {
                            Some("notify") => {
                                if let Some(msg) =
                                    event.get("message").and_then(serde_json::Value::as_str)
                                {
                                    hub2.send(Event::Notify(msg.to_string())).ok();
                                }
                            }
                            Some("setWifi") => {
                                if let Some(enable) =
                                    event.get("enable").and_then(serde_json::Value::as_bool)
                                {
                                    hub2.send(Event::SetWifi(enable)).ok();
                                }
                            }
                            Some("addDocument") => {
                                if let Some(info) = event
                                    .get("info")
                                    .map(ToString::to_string)
                                    .and_then(|v| serde_json::from_str(&v).ok())
                                {
                                    hub2.send(Event::FetcherAddDocument(id, Box::new(info)))
                                        .ok();
                                }
                            }
                            Some("removeDocument") => {
                                if let Some(path) =
                                    event.get("path").and_then(serde_json::Value::as_str)
                                {
                                    hub2.send(Event::FetcherRemoveDocument(
                                        id,
                                        std::path::PathBuf::from(path),
                                    ))
                                    .ok();
                                }
                            }
                            Some("search") => {
                                let path = event
                                    .get("path")
                                    .and_then(serde_json::Value::as_str)
                                    .map(std::path::PathBuf::from);
                                let query = event
                                    .get("query")
                                    .and_then(serde_json::Value::as_str)
                                    .map(String::from);
                                let sort_by = event
                                    .get("sortBy")
                                    .map(ToString::to_string)
                                    .and_then(|v| serde_json::from_str(&v).ok());
                                hub2.send(Event::FetcherSearch {
                                    id,
                                    path,
                                    query,
                                    sort_by,
                                })
                                .ok();
                            }
                            _ => (),
                        }
                    }
                } else {
                    break;
                }
            }
            hub2.send(Event::CheckFetcher(id)).ok();
        });
        Ok(process)
    }

    pub(crate) fn reseed(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        context.library.sort(self.sort_method, self.reverse_order);
        self.refresh_visibles(true, false, hub, &mut RenderQueue::new(), context);

        if let Some(top_bar) = self.child_mut(0).downcast_mut::<TopBar>() {
            top_bar.reseed(&mut RenderQueue::new(), context);
        }

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }
}
