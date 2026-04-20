//! Battery Module - Battery Status Abstraction
//!
//! This module provides hardware abstraction for battery monitoring via the
//! [`Battery`] trait. This allows the application to work with different battery
//! hardware implementations through a common interface.
//!
//! ## Architecture
//!
//! The module uses a trait-based abstraction:
//!
//! - **[`Battery`] trait**: Core abstraction for battery status
//! - **Hardware implementations**: `KoboBattery` for Kobo devices
//! - **Mock implementations**: [`MockBattery`](crate::test_mocks::MockBattery) for testing
//!
//! ## Trait-Based Design
//!
//! The [`Battery`] trait enables:
//! - **Hardware independence**: UI code doesn't depend on specific battery hardware
//! - **Testability**: Mock implementations allow testing without real hardware
//! - **Consistent interface**: All battery types provide capacity and status
//!
//! ## Usage
//!
//! ```rust,ignore
//! use plato_core::battery::{Battery, KoboBattery};
//!
//! let mut battery = KoboBattery::new();
//! let capacities = battery.capacity()?;
//! let statuses = battery.status()?;
//! ```

mod fake;
mod kobo;

use thiserror::Error;

pub use self::fake::FakeBattery;
pub use self::kobo::KoboBattery;

/// Battery error types
#[derive(Debug, Error)]
pub enum BatteryError {
    #[error("Failed to read battery capacity: {0}")]
    CapacityReadError(String),
    #[error("Failed to read battery status: {0}")]
    StatusReadError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Status {
    Discharging,
    Charging,
    Charged,
    Unknown, // Full,
}

impl Status {
    pub fn is_wired(self) -> bool {
        matches!(self, Status::Charging | Status::Charged)
    }
}

pub trait Battery {
    fn capacity(&mut self) -> Result<Vec<f32>, BatteryError>;
    fn status(&mut self) -> Result<Vec<Status>, BatteryError>;
}
