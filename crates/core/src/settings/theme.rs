use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeMode {
    Light,
    Dark,
    Sepia,
    Auto,
    Scheduled,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Light
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct TimeOfDay {
    pub hour: u8,
    pub minute: u8,
}

impl TimeOfDay {
    pub fn new(hour: u8, minute: u8) -> Self {
        Self { hour, minute }
    }

    pub fn as_minutes(&self) -> u16 {
        (self.hour as u16) * 60 + (self.minute as u16)
    }
}

impl Default for TimeOfDay {
    fn default() -> Self {
        Self { hour: 6, minute: 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ThemeSchedule {
    pub dark_start: TimeOfDay,
    pub dark_end: TimeOfDay,
    pub enabled: bool,
}

impl Default for ThemeSchedule {
    fn default() -> Self {
        Self {
            dark_start: TimeOfDay::new(20, 0),
            dark_end: TimeOfDay::new(6, 0),
            enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ThemeSettings {
    pub mode: ThemeMode,
    pub auto_threshold: u16,
    pub schedule: ThemeSchedule,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Light,
            auto_threshold: 100,
            schedule: ThemeSchedule::default(),
        }
    }
}
