use crate::validation::validate_finite_f32;
use anyhow::Error;
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

#[allow(clippy::derivable_impls)]
impl Default for BatterySettings {
    fn default() -> Self {
        BatterySettings {
            warn: 10.0,
            power_off: 3.0,
        }
    }
}

impl BatterySettings {
    /// Validates battery settings are within acceptable ranges
    ///
    /// # Validation Rules
    /// - warn: 1.0% to 50.0% (warn threshold must be reasonable)
    /// - power_off: 0.5% to 20.0% (power off threshold must be reasonable)
    /// - warn must be greater than power_off
    pub fn validate(&self) -> Result<(), Error> {
        // Warn threshold must be reasonable
        validate_finite_f32(self.warn, "battery.warn", 1.0, 50.0)?;

        // Power off threshold must be reasonable
        validate_finite_f32(self.power_off, "battery.power_off", 0.5, 20.0)?;

        // Warn threshold must be greater than power_off threshold
        if self.warn <= self.power_off {
            return Err(anyhow::format_err!(
                "battery.warn ({}) must be greater than battery.power_off ({})",
                self.warn,
                self.power_off
            ));
        }

        Ok(())
    }
}

#[allow(clippy::derivable_impls)]
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

impl NightLightSchedule {
    /// Validates night light schedule settings are within acceptable ranges
    ///
    /// # Validation Rules
    /// - start_hour: 0 to 23
    /// - start_minute: 0 to 59
    /// - end_hour: 0 to 23
    /// - end_minute: 0 to 59
    /// - warmth_start: 0.0 to 1.0
    /// - warmth_end: 0.0 to 1.0
    pub fn validate(&self) -> Result<(), Error> {
        // Validate time ranges
        if self.start_hour > 23 {
            return Err(anyhow::format_err!(
                "night_light_schedule.start_hour ({}) must be 0-23",
                self.start_hour
            ));
        }
        if self.start_minute > 59 {
            return Err(anyhow::format_err!(
                "night_light_schedule.start_minute ({}) must be 0-59",
                self.start_minute
            ));
        }
        if self.end_hour > 23 {
            return Err(anyhow::format_err!(
                "night_light_schedule.end_hour ({}) must be 0-23",
                self.end_hour
            ));
        }
        if self.end_minute > 59 {
            return Err(anyhow::format_err!(
                "night_light_schedule.end_minute ({}) must be 0-59",
                self.end_minute
            ));
        }

        // Validate warmth values
        validate_finite_f32(
            self.warmth_start,
            "night_light_schedule.warmth_start",
            0.0,
            1.0,
        )?;
        validate_finite_f32(self.warmth_end, "night_light_schedule.warmth_end", 0.0, 1.0)?;

        Ok(())
    }
}
