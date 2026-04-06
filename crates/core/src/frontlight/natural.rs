use super::{Frontlight, LightLevels};
use crate::device::{Model, CURRENT_DEVICE};
use anyhow::Error;
use rustc_hash::FxHashMap;
use lazy_static::lazy_static;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

const FRONTLIGHT_INTERFACE: &str = "/sys/class/backlight";

// Aura ONE
const FRONTLIGHT_WHITE_A: &str = "lm3630a_led1b";
const FRONTLIGHT_RED_A: &str = "lm3630a_led1a";
const FRONTLIGHT_GREEN_A: &str = "lm3630a_ledb";

// Aura H₂O Edition 2
const FRONTLIGHT_WHITE_B: &str = "lm3630a_ledb";
const FRONTLIGHT_ORANGE_B: &str = "lm3630a_leda";

const FRONTLIGHT_VALUE: &str = "brightness";
const FRONTLIGHT_MAX_VALUE: &str = "max_brightness";
const FRONTLIGHT_POWER: &str = "bl_power";

const FRONTLIGHT_POWER_ON: i16 = 31;
const FRONTLIGHT_POWER_OFF: i16 = 0;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum LightColor {
    White,
    Red,
    Green,
    Orange,
}

lazy_static! {
    pub static ref FRONTLIGHT_DIRS: FxHashMap<LightColor, &'static str> = match CURRENT_DEVICE.model
    {
        Model::AuraONE | Model::AuraONELimEd => {
            [
                (LightColor::White, FRONTLIGHT_WHITE_A),
                (LightColor::Red, FRONTLIGHT_RED_A),
                (LightColor::Green, FRONTLIGHT_GREEN_A),
            ]
            .iter()
            .cloned()
            .collect()
        }
        _ => {
            [
                (LightColor::White, FRONTLIGHT_WHITE_B),
                (LightColor::Orange, FRONTLIGHT_ORANGE_B),
            ]
            .iter()
            .cloned()
            .collect()
        }
    };
}

pub struct NaturalFrontlight {
    intensity: f32,
    warmth: f32,
    values: FxHashMap<LightColor, File>,
    powers: FxHashMap<LightColor, File>,
    maxima: FxHashMap<LightColor, i16>,
    /// Track power state to avoid unnecessary writes
    power_states: FxHashMap<LightColor, i16>,
}

impl NaturalFrontlight {
    pub fn new(intensity: f32, warmth: f32) -> Result<NaturalFrontlight, Error> {
        let mut maxima = FxHashMap::default();
        let mut values = FxHashMap::default();
        let mut powers = FxHashMap::default();
        let mut power_states = FxHashMap::default();
        let base = PathBuf::from(FRONTLIGHT_INTERFACE);
        for (light, name) in FRONTLIGHT_DIRS.iter() {
            let dir = base.join(name);
            let mut buf = String::with_capacity(16);
            let mut file = File::open(dir.join(FRONTLIGHT_MAX_VALUE))?;
            file.read_to_string(&mut buf)?;
            maxima.insert(*light, buf.trim_end().parse()?);
            let file = OpenOptions::new()
                .write(true)
                .open(dir.join(FRONTLIGHT_VALUE))?;
            values.insert(*light, file);
            let file = OpenOptions::new()
                .write(true)
                .open(dir.join(FRONTLIGHT_POWER))?;
            powers.insert(*light, file);
            power_states.insert(*light, FRONTLIGHT_POWER_OFF);
        }
        Ok(NaturalFrontlight {
            intensity,
            warmth,
            maxima,
            values,
            powers,
            power_states,
        })
    }

    fn set(&mut self, c: LightColor, percent: f32) {
        let max_value = self.maxima[&c] as f32;
        let value = (percent.clamp(0.0, 100.0) / 100.0 * max_value) as i16;
        let mut file = &self.values[&c];
        write!(file, "{}", value).expect("write to sysfs failed");

        let new_power = if value > 0 {
            FRONTLIGHT_POWER_ON
        } else {
            FRONTLIGHT_POWER_OFF
        };

        // Only write power state if it changed (optimization)
        let old_power = self
            .power_states
            .get(&c)
            .copied()
            .unwrap_or(FRONTLIGHT_POWER_OFF);
        if new_power != old_power {
            let mut file = &self.powers[&c];
            write!(file, "{}", new_power).expect("write to sysfs failed");
            self.power_states.insert(c, new_power);
        }
    }

    fn update(&mut self, intensity: f32, warmth: f32) {
        let i = intensity / 100.0;
        let w = warmth / 100.0;
        let white = 80.0 * i * (1.0 - w).sqrt();
        self.set(LightColor::White, white);

        if self.values.len() == 3 {
            let green = 64.0 * (w * i).sqrt();
            let red = if green == 0.0 {
                0.0
            } else {
                green + 20.0 + 7.0 * (1.0 - green / 64.0) + w * 4.0
            };
            self.set(LightColor::Red, red);
            self.set(LightColor::Green, green);
        } else {
            let orange = 95.0 * (w * i).sqrt();
            self.set(LightColor::Orange, orange);
        }

        self.intensity = intensity;
        self.warmth = warmth;
    }
}

impl Frontlight for NaturalFrontlight {
    fn set_intensity(&mut self, value: f32) {
        let warmth = self.warmth;
        self.update(value, warmth);
    }

    fn set_warmth(&mut self, value: f32) {
        let intensity = self.intensity;
        self.update(intensity, value);
    }

    fn levels(&self) -> LightLevels {
        LightLevels {
            intensity: self.intensity,
            warmth: self.warmth,
        }
    }
}
