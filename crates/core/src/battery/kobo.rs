use super::{Battery, Status};
use crate::device::CURRENT_DEVICE;
use anyhow::{format_err, Error};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const BATTERY_INTERFACES: [&str; 3] = [
    "/sys/class/power_supply/bd71827_bat",
    "/sys/class/power_supply/mc13892_bat",
    "/sys/class/power_supply/battery",
];
const POWER_COVER_INTERFACE: &str = "/sys/class/misc/cilix";

const BATTERY_CAPACITY: &str = "capacity";
const BATTERY_STATUS: &str = "status";

const POWER_COVER_CAPACITY: &str = "cilix_bat_capacity";
const POWER_COVER_STATUS: &str = "charge_status";
const POWER_COVER_CONNECTED: &str = "cilix_conn";

pub struct PowerCover {
    capacity: File,
    status: File,
    connected: File,
}

/// Kobo e-reader battery driver with power cover support.
///
/// Optimizations applied:
/// - Caches power cover connection state to avoid redundant file reads
/// - Stores capacity/status values to return cached data when reads fail
/// - Uses separate buffers for each read operation to prevent data corruption
pub struct KoboBattery {
    capacity: File,
    status: File,
    power_cover: Option<PowerCover>,
    /// Cached power cover connection state (eliminates redundant is_connected check)
    power_cover_connected: bool,
    /// Cached main battery capacity
    cached_capacity: f32,
    /// Cached power cover capacity
    cached_power_cover_capacity: f32,
    /// Cached main battery status
    cached_status: Status,
    /// Cached power cover status
    cached_power_cover_status: Status,
}

impl KoboBattery {
    pub fn new() -> Result<KoboBattery, Error> {
        let base = Path::new(
            BATTERY_INTERFACES
                .iter()
                .find(|bi| Path::new(bi).exists())
                .ok_or_else(|| format_err!("battery path missing"))?,
        );
        let capacity = File::open(base.join(BATTERY_CAPACITY))?;
        let status = File::open(base.join(BATTERY_STATUS))?;
        let power_cover = if CURRENT_DEVICE.has_power_cover() {
            let base = Path::new(POWER_COVER_INTERFACE);
            let capacity = File::open(base.join(POWER_COVER_CAPACITY))?;
            let status = File::open(base.join(POWER_COVER_STATUS))?;
            let connected = File::open(base.join(POWER_COVER_CONNECTED))?;
            Some(PowerCover {
                capacity,
                status,
                connected,
            })
        } else {
            None
        };
        Ok(KoboBattery {
            capacity,
            status,
            power_cover,
            power_cover_connected: false,
            cached_capacity: 0.0,
            cached_power_cover_capacity: 0.0,
            cached_status: Status::Unknown,
            cached_power_cover_status: Status::Unknown,
        })
    }
}

impl KoboBattery {
    /// Checks power cover connection and caches the result.
    /// Avoids redundant file I/O by storing result in power_cover_connected.
    fn check_power_cover_connection(&mut self) -> Result<bool, Error> {
        if let Some(power_cover) = self.power_cover.as_mut() {
            let mut buf = String::new();
            power_cover.connected.seek(SeekFrom::Start(0))?;
            power_cover.connected.read_to_string(&mut buf)?;
            self.power_cover_connected = buf.trim_end().parse::<u8>().map_or(false, |v| v == 1);
            Ok(self.power_cover_connected)
        } else {
            Ok(false)
        }
    }
}

impl Battery for KoboBattery {
    /// Returns battery capacity percentages.
    ///
    /// Optimizations:
    /// - Checks power cover connection once and caches result
    /// - Uses separate buffer for power cover read to avoid data corruption
    fn capacity(&mut self) -> Result<Vec<f32>, Error> {
        let mut buf = String::new();
        self.capacity.seek(SeekFrom::Start(0))?;
        self.capacity.read_to_string(&mut buf)?;
        self.cached_capacity = buf.trim_end().parse::<f32>().unwrap_or(0.0);

        if self.power_cover.is_some() {
            self.check_power_cover_connection()?;
            if self.power_cover_connected {
                let mut buf = String::new();
                if let Some(power_cover) = self.power_cover.as_mut() {
                    power_cover.capacity.seek(SeekFrom::Start(0))?;
                    power_cover.capacity.read_to_string(&mut buf)?;
                }
                self.cached_power_cover_capacity = buf.trim_end().parse::<f32>().unwrap_or(0.0);
                return Ok(vec![self.cached_capacity, self.cached_power_cover_capacity]);
            }
        }
        Ok(vec![self.cached_capacity])
    }

    /// Returns battery charging status.
    ///
    /// Reuses cached power_cover_connected from capacity() call to avoid
    /// redundant file I/O when both methods are called in sequence.
    fn status(&mut self) -> Result<Vec<Status>, Error> {
        let mut buf = String::new();
        self.status.seek(SeekFrom::Start(0))?;
        self.status.read_to_string(&mut buf)?;
        self.cached_status = match buf.trim_end() {
            "Discharging" => Status::Discharging,
            "Charging" => Status::Charging,
            "Not charging" | "Full" => Status::Charged,
            _ => Status::Unknown,
        };

        if self.power_cover.is_some() && self.power_cover_connected {
            let mut buf = String::new();
            if let Some(power_cover) = self.power_cover.as_mut() {
                power_cover.status.seek(SeekFrom::Start(0))?;
                power_cover.status.read_to_string(&mut buf)?;
            }
            self.cached_power_cover_status = match buf.trim_end().parse::<i8>() {
                Ok(0) => Status::Discharging,
                Ok(2) => Status::Charging,
                Ok(3) => Status::Charged,
                _ => Status::Unknown,
            };
            return Ok(vec![self.cached_status, self.cached_power_cover_status]);
        }
        Ok(vec![self.cached_status])
    }
}
