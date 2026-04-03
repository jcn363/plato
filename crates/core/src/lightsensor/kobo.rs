use super::LightSensor;
use anyhow::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::time::{Duration, Instant};

// The Aura ONE uses a Silicon Graphics light sensor,
// the model code is si114x (where x is 5, 6, or 7).
const VISIBLE_PHOTODIODE: &str = "/sys/devices/virtual/input/input3/als_vis_data";

/// Light sensor with value caching to reduce file I/O.
/// Caches reading for a short interval to avoid excessive reads.
const CACHE_DURATION: Duration = Duration::from_millis(500);

pub struct KoboLightSensor {
    file: File,
    cached_level: Option<(u16, Instant)>,
}

impl KoboLightSensor {
    pub fn new() -> Result<Self, Error> {
        let file = File::open(VISIBLE_PHOTODIODE)?;
        Ok(KoboLightSensor {
            file,
            cached_level: None,
        })
    }
}

impl LightSensor for KoboLightSensor {
    fn level(&mut self) -> Result<u16, Error> {
        // Return cached value if still valid
        if let Some((level, timestamp)) = self.cached_level {
            if timestamp.elapsed() < CACHE_DURATION {
                return Ok(level);
            }
        }

        let mut buf = String::new();
        self.file.seek(SeekFrom::Start(0))?;
        self.file.read_to_string(&mut buf)?;
        let value = buf.trim_end().parse()?;

        // Cache the new value
        self.cached_level = Some((value, Instant::now()));
        Ok(value)
    }
}
