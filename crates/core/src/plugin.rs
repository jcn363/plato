use crate::settings::{Plugin, PluginSettings, PluginTrigger};
use crate::{log_error, log_warn};
use anyhow::{bail, format_err, Error};
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct PluginSystem {
    settings: PluginSettings,
    plugins: FxHashMap<String, Plugin>,
}

impl PluginSystem {
    pub fn new(settings: &PluginSettings) -> PluginSystem {
        let mut system = PluginSystem {
            settings: settings.clone(),
            plugins: FxHashMap::with_hasher(FxBuildHasher),
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

        let canonical_plugins_dir = Self::canonicalize_plugins_dir(plugins_dir)?;

        for entry in fs::read_dir(plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(plugin) = self.load_plugin(&path, &canonical_plugins_dir)? {
                let name = plugin.name.clone();
                self.plugins.insert(name, plugin);
            }
        }

        Ok(())
    }

    fn canonicalize_plugins_dir(plugins_dir: &Path) -> Result<PathBuf, Error> {
        plugins_dir
            .canonicalize()
            .map_err(|e| format_err!("Failed to canonicalize plugins directory: {}", e))
    }

    fn load_plugin(
        &self,
        path: &Path,
        canonical_plugins_dir: &Path,
    ) -> Result<Option<Plugin>, Error> {
        let canonical_path = Self::validate_plugin_path(path, canonical_plugins_dir)?;
        let Some(filename) = path.file_name() else {
            return Ok(None);
        };

        let name = filename.to_string_lossy().to_string();

        if name.starts_with('.') || name.starts_with('_') {
            return Ok(None);
        }

        let triggers = self.detect_triggers(path)?;

        Ok(Some(Plugin {
            name: name.clone(),
            path: canonical_path,
            triggers,
            enabled: true,
        }))
    }

    fn validate_plugin_path(path: &Path, canonical_plugins_dir: &Path) -> Result<PathBuf, Error> {
        let canonical_path = path
            .canonicalize()
            .map_err(|e| format_err!("Failed to canonicalize plugin path: {}", e))?;
        if !canonical_path.starts_with(canonical_plugins_dir) {
            log_warn!(
                "Skipping plugin outside plugins directory: {}",
                path.display()
            );
            bail!("Plugin path is outside plugins directory");
        }
        Ok(canonical_path)
    }

    fn detect_triggers(&self, path: &Path) -> Result<Vec<PluginTrigger>, Error> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let triggers = match extension {
            "sh" | "bash" => self.parse_triggers(path, "#"),
            "py" => self.parse_triggers(path, "#"),
            "js" => vec![PluginTrigger::OnStartup],
            _ => vec![],
        };

        Ok(triggers)
    }

    fn parse_triggers(&self, path: &Path, comment_prefix: &str) -> Vec<PluginTrigger> {
        let mut triggers = Vec::new();

        if let Ok(content) = fs::read_to_string(path) {
            let prefix = format!("{} plato:on_", comment_prefix);
            if content.contains(&format!("{}book_import", prefix)) {
                triggers.push(PluginTrigger::OnBookImport);
            }
            if content.contains(&format!("{}book_open", prefix)) {
                triggers.push(PluginTrigger::OnBookOpen);
            }
            if content.contains(&format!("{}book_close", prefix)) {
                triggers.push(PluginTrigger::OnBookClose);
            }
            if content.contains(&format!("{}sync_complete", prefix)) {
                triggers.push(PluginTrigger::OnSyncComplete);
            }
            if content.contains(&format!("{}startup", prefix)) {
                triggers.push(PluginTrigger::OnStartup);
            }
            if content.contains(&format!("{}shutdown", prefix)) {
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

    fn run_plugin_command(interpreter: &str, path: &Path, args: &[&str]) -> Result<(), Error> {
        let mut cmd = Command::new(interpreter);
        cmd.arg(path);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.output()
            .map_err(|e| format_err!("Failed to execute plugin with {}: {}", interpreter, e))?;
        Ok(())
    }

    fn execute_plugin(&self, plugin: &Plugin, args: &[&str]) -> Result<(), Error> {
        let path = &plugin.path;
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let uses_network = self.plugin_uses_network(path)?;

        if uses_network && !self.settings.allow_network {
            return Err(format_err!(
                "Plugin {} requires network access but allow_network is disabled",
                plugin.name
            ));
        }

        match extension {
            "sh" | "bash" => Self::run_plugin_command("bash", path, args)
                .map_err(|e| format_err!("Failed to execute {}: {}", plugin.name, e))?,
            "py" => Self::run_plugin_command("python3", path, args)
                .map_err(|e| format_err!("Failed to execute {}: {}", plugin.name, e))?,
            "js" => Self::run_plugin_command("node", path, args)
                .map_err(|e| format_err!("Failed to execute {}: {}", plugin.name, e))?,
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
        if name.is_empty() {
            bail!("Plugin name cannot be empty");
        }
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.enabled = true;
            Ok(())
        } else {
            Err(format_err!("Plugin not found: {}", name))
        }
    }

    pub fn disable_plugin(&mut self, name: &str) -> Result<(), Error> {
        if name.is_empty() {
            bail!("Plugin name cannot be empty");
        }
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
