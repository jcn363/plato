use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct BatterySettings {
    pub warn: f32,
    pub power_off: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct NightLightSchedule {
    pub enabled: bool,
    pub start_hour: u8,
    pub start_minute: u8,
    pub end_hour: u8,
    pub end_minute: u8,
    pub warmth_start: f32,
    pub warmth_end: f32,
}

impl Default for BatterySettings {
    fn default() -> Self {
        BatterySettings {
            warn: 10.0,
            power_off: 3.0,
        }
    }
}

impl Default for NightLightSchedule {
    fn default() -> Self {
        NightLightSchedule {
            enabled: false,
            start_hour: 20,
            start_minute: 0,
            end_hour: 6,
            end_minute: 0,
            warmth_start: 0.5,
            warmth_end: 0.0,
        }
    }
}
