use super::{Battery, BatteryError, Status};

/// Fake battery implementation for testing.
/// Values are cached in struct fields (no file I/O required).
pub struct FakeBattery {
    capacity: f32,
    status: Status,
}

#[allow(clippy::derivable_impls)]
impl Default for FakeBattery {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeBattery {
    pub fn new() -> FakeBattery {
        FakeBattery {
            capacity: 50.0,
            status: Status::Discharging,
        }
    }

    /// Update the fake battery capacity (for testing)
    pub fn set_capacity(&mut self, capacity: f32) {
        self.capacity = capacity.clamp(0.0, 100.0);
    }

    /// Update the fake battery status (for testing)
    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }
}

impl Battery for FakeBattery {
    fn capacity(&mut self) -> Result<Vec<f32>, BatteryError> {
        Ok(vec![self.capacity])
    }

    fn status(&mut self) -> Result<Vec<Status>, BatteryError> {
        Ok(vec![self.status])
    }
}
