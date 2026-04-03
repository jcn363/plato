use crate::settings::{Plugin, PluginSettings, PluginTrigger};
use crate::{log_error, log_warn};
use anyhow::{format_err, Error};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

pub struct PluginSystem {
    settings: PluginSettings,
    plugins: HashMap<String, Plugin>,
}

impl PluginSystem {
    pub fn new(settings: &PluginSettings) -> PluginSystem {
        let mut system = PluginSystem {
            settings: settings.clone(),
            plugins: HashMap::new(),
        };

        if settings.enabled {
            if let Err(e) = system.load_plugins() {
                log_error!("Failed to load plugins: {}", e);
            }
        }

        system
    }

    pub fn load_plugins(&mut self) -> Result<(), Error> {
        self.plugins.clear();

        let plugins_dir = &self.settings.plugins_dir;
        if !plugins_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let Some(filename) = path.file_name() else {
                continue;
            };

            let name = filename.to_string_lossy().to_string();

            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }

            let triggers = self.detect_triggers(&path)?;

            let plugin = Plugin {
                name: name.clone(),
                path: path.clone(),
                triggers,
                enabled: true,
            };

            self.plugins.insert(name, plugin);
        }

        Ok(())
    }

    fn detect_triggers(&self, path: &Path) -> Result<Vec<PluginTrigger>, Error> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let triggers = match extension {
            "sh" | "bash" => self.parse_shell_triggers(path),
            "py" => self.parse_python_triggers(path),
            "js" => vec![PluginTrigger::OnStartup],
            _ => vec![],
        };

        Ok(triggers)
    }

    fn parse_shell_triggers(&self, path: &Path) -> Vec<PluginTrigger> {
        let mut triggers = Vec::new();

        if let Ok(content) = fs::read_to_string(path) {
            if content.contains("# plato:on_book_import") {
                triggers.push(PluginTrigger::OnBookImport);
            }
            if content.contains("# plato:on_book_open") {
                triggers.push(PluginTrigger::OnBookOpen);
            }
            if content.contains("# plato:on_book_close") {
                triggers.push(PluginTrigger::OnBookClose);
            }
            if content.contains("# plato:on_sync_complete") {
                triggers.push(PluginTrigger::OnSyncComplete);
            }
            if content.contains("# plato:on_startup") {
                triggers.push(PluginTrigger::OnStartup);
            }
            if content.contains("# plato:on_shutdown") {
                triggers.push(PluginTrigger::OnShutdown);
            }
        }

        if triggers.is_empty() {
            triggers.push(PluginTrigger::OnStartup);
        }

        triggers
    }

    fn parse_python_triggers(&self, path: &Path) -> Vec<PluginTrigger> {
        let mut triggers = Vec::new();

        if let Ok(content) = fs::read_to_string(path) {
            if content.contains("plato:on_book_import") {
                triggers.push(PluginTrigger::OnBookImport);
            }
            if content.contains("plato:on_book_open") {
                triggers.push(PluginTrigger::OnBookOpen);
            }
            if content.contains("plato:on_book_close") {
                triggers.push(PluginTrigger::OnBookClose);
            }
            if content.contains("plato:on_sync_complete") {
                triggers.push(PluginTrigger::OnSyncComplete);
            }
            if content.contains("plato:on_startup") {
                triggers.push(PluginTrigger::OnStartup);
            }
            if content.contains("plato:on_shutdown") {
                triggers.push(PluginTrigger::OnShutdown);
            }
        }

        if triggers.is_empty() {
            triggers.push(PluginTrigger::OnStartup);
        }

        triggers
    }

    pub fn trigger(&self, trigger: &PluginTrigger, args: &[&str]) -> Result<(), Error> {
        if !self.settings.enabled {
            return Ok(());
        }

        for plugin in self.plugins.values() {
            if !plugin.enabled {
                continue;
            }

            if !plugin.triggers.contains(trigger) {
                continue;
            }

            self.execute_plugin(plugin, args)?;
        }

        Ok(())
    }

    fn execute_plugin(&self, plugin: &Plugin, args: &[&str]) -> Result<(), Error> {
        let path = &plugin.path;
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let _timeout = Duration::from_secs(self.settings.timeout_seconds as u64);

        let uses_network = self.plugin_uses_network(path)?;

        if uses_network && !self.settings.allow_network {
            return Err(format_err!(
                "Plugin {} requires network access but allow_network is disabled",
                plugin.name
            ));
        }

        match extension {
            "sh" | "bash" => {
                let mut cmd = Command::new("bash");
                cmd.arg(path);
                for arg in args {
                    cmd.arg(arg);
                }
                cmd.output()
                    .map_err(|e| format_err!("Failed to execute {}: {}", plugin.name, e))?;
            }
            "py" => {
                let mut cmd = Command::new("python3");
                cmd.arg(path);
                for arg in args {
                    cmd.arg(arg);
                }
                cmd.output()
                    .map_err(|e| format_err!("Failed to execute {}: {}", plugin.name, e))?;
            }
            "js" => {
                let mut cmd = Command::new("node");
                cmd.arg(path);
                for arg in args {
                    cmd.arg(arg);
                }
                cmd.output()
                    .map_err(|e| format_err!("Failed to execute {}: {}", plugin.name, e))?;
            }
            _ => {
                log_warn!("Unknown plugin type: {}", plugin.name);
            }
        }

        Ok(())
    }

    fn plugin_uses_network(&self, path: &Path) -> Result<bool, Error> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let content = fs::read_to_string(path).unwrap_or_default();

        let network_indicators = match extension {
            "sh" | "bash" => vec![
                "curl ",
                "wget ",
                "http",
                "https",
                "ftp",
                "wpa_cli",
                "connmanctl",
                "netstat",
                "ping ",
                "ssh ",
                "scp ",
                "rsync",
                "fetch",
            ],
            "py" => vec![
                "requests", "urllib", "http", "https", "socket", "ftplib", "smtplib", "poplib",
                "imaplib",
            ],
            "js" => vec![
                "http", "https", "fetch", "axios", "socket", "net", "tls", "crypto",
            ],
            _ => vec![],
        };

        let content_lower = content.to_lowercase();
        for indicator in network_indicators {
            if content_lower.contains(indicator) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn on_book_import(&self, book_path: &Path) -> Result<(), Error> {
        self.trigger(
            &PluginTrigger::OnBookImport,
            &[&book_path.to_string_lossy()],
        )
    }

    pub fn on_book_open(&self, book_path: &Path) -> Result<(), Error> {
        self.trigger(&PluginTrigger::OnBookOpen, &[&book_path.to_string_lossy()])
    }

    pub fn on_book_close(&self, book_path: &Path) -> Result<(), Error> {
        self.trigger(&PluginTrigger::OnBookClose, &[&book_path.to_string_lossy()])
    }

    pub fn on_sync_complete(&self) -> Result<(), Error> {
        self.trigger(&PluginTrigger::OnSyncComplete, &[])
    }

    pub fn on_startup(&self) -> Result<(), Error> {
        self.trigger(&PluginTrigger::OnStartup, &[])
    }

    pub fn on_shutdown(&self) -> Result<(), Error> {
        self.trigger(&PluginTrigger::OnShutdown, &[])
    }

    pub fn list_plugins(&self) -> Vec<&Plugin> {
        self.plugins.values().collect()
    }

    pub fn enable_plugin(&mut self, name: &str) -> Result<(), Error> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.enabled = true;
            Ok(())
        } else {
            Err(format_err!("Plugin not found: {}", name))
        }
    }

    pub fn disable_plugin(&mut self, name: &str) -> Result<(), Error> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.enabled = false;
            Ok(())
        } else {
            Err(format_err!("Plugin not found: {}", name))
        }
    }

    pub fn reload(&mut self) -> Result<(), Error> {
        if self.settings.enabled {
            self.load_plugins()
        } else {
            self.plugins.clear();
            Ok(())
        }
    }
}

impl Default for PluginSystem {
    fn default() -> Self {
        PluginSystem::new(&PluginSettings::default())
    }
}

pub fn create_sample_plugin(name: &str, trigger: PluginTrigger) -> String {
    let shebang = match name.rsplit('.').next() {
        Some("sh") | Some("bash") => "#!/bin/bash\n# plato:on_book_import\n",
        Some("py") => "#!/usr/bin/env python3\n# plato:on_book_import\n",
        Some("js") => "#!/usr/bin/env node\n// plato:on_startup\n",
        _ => "#!/bin/bash\n# plato:on_startup\n",
    };

    let body = match trigger {
        PluginTrigger::OnBookImport => {
            r#"
echo "Book imported: $1"
# Process the book file
exit 0
"#
        }
        PluginTrigger::OnBookOpen => {
            r#"
echo "Opening book: $1"
exit 0
"#
        }
        PluginTrigger::OnBookClose => {
            r#"
echo "Closing book: $1"
exit 0
"#
        }
        PluginTrigger::OnSyncComplete => {
            r#"
echo "Sync completed"
exit 0
"#
        }
        PluginTrigger::OnStartup => {
            r#"
echo "Plato started"
exit 0
"#
        }
        PluginTrigger::OnShutdown => {
            r#"
echo "Plato shutting down"
exit 0
"#
        }
    };

    format!("{}{}", shebang, body)
}
