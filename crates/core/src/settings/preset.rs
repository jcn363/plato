use crate::frontlight::LightLevels;
use crate::geom::circular_distances;
use chrono::{Local, Timelike};
use serde::{Deserialize, Serialize};

const MINUTES_PER_DAY: u16 = 24 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LightPreset {
    pub timestamp: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightsensor_level: Option<u16>,
    pub frontlight_levels: LightLevels,
}

#[allow(clippy::derivable_impls)]
impl Default for LightPreset {
    fn default() -> Self {
        let now = Local::now();
        LightPreset {
            timestamp: (60 * now.hour() + now.minute()) as u16,
            frontlight_levels: LightLevels::default(),
            lightsensor_level: None,
        }
    }
}

impl LightPreset {
    pub fn name(&self) -> String {
        let hours = self.timestamp / 60;
        let minutes = self.timestamp - hours * 60;
        format!("{:02}:{:02}", hours, minutes)
    }
}

pub fn guess_frontlight(
    lightsensor_level: Option<u16>,
    light_presets: &[LightPreset],
) -> Option<LightLevels> {
    if light_presets.len() < 2 {
        return None;
    }
    let cur = LightPreset {
        lightsensor_level,
        ..Default::default()
    };

    let (dmin, index) = if light_presets[0].lightsensor_level.is_some() {
        find_nearest_by_lightsensor(&cur, light_presets)
    } else {
        find_nearest_by_timestamp(&cur, light_presets)
    };

    interpolate_frontlight_levels(dmin, index, light_presets)
}

fn find_nearest_by_lightsensor(
    cur: &LightPreset,
    light_presets: &[LightPreset],
) -> ([u16; 2], [usize; 2]) {
    let s = cur.lightsensor_level.unwrap_or_default();
    let mut dmin = [u16::MAX; 2];
    let mut index = [usize::MAX; 2];

    for (i, lp) in light_presets.iter().enumerate() {
        let p = lp.lightsensor_level.unwrap_or_default();
        let d = s.abs_diff(p);

        if p >= s && d < dmin[0] {
            dmin[0] = d;
            index[0] = i;
        }

        if p <= s && d < dmin[1] {
            dmin[1] = d;
            index[1] = i;
        }
    }

    (dmin, index)
}

fn find_nearest_by_timestamp(
    cur: &LightPreset,
    light_presets: &[LightPreset],
) -> ([u16; 2], [usize; 2]) {
    let mut dmin = [u16::MAX; 2];
    let mut index = [usize::MAX; 2];

    for (i, lp) in light_presets.iter().enumerate() {
        let (d0, d1) = circular_distances(cur.timestamp, lp.timestamp, MINUTES_PER_DAY);

        if d0 < dmin[0] {
            dmin[0] = d0;
            index[0] = i;
        }

        if d1 < dmin[1] {
            dmin[1] = d1;
            index[1] = i;
        }
    }

    (dmin, index)
}

fn interpolate_frontlight_levels(
    dmin: [u16; 2],
    index: [usize; 2],
    light_presets: &[LightPreset],
) -> Option<LightLevels> {
    if dmin[0] == 0 || dmin[1] == u16::MAX {
        return Some(light_presets[index[0]].frontlight_levels);
    }

    if dmin[1] == 0 || dmin[0] == u16::MAX {
        return Some(light_presets[index[1]].frontlight_levels);
    }

    let fl0 = light_presets[index[0]].frontlight_levels;
    let fl1 = light_presets[index[1]].frontlight_levels;
    let t = dmin[0] as f32 / (dmin[0] + dmin[1]) as f32;

    Some(fl0.interpolate(fl1, t))
}
