//! Light Sensor Module - Ambient Light Sensing Abstraction
//!
//! This module provides hardware abstraction for ambient light sensing via the
//! [`LightSensor`] trait. This allows the application to work with different
//! light sensor hardware implementations through a common interface.
//!
//! ## Architecture
//!
//! The module uses a trait-based abstraction:
//!
//! - **[`LightSensor`] trait**: Core abstraction for light sensing
//! - **Hardware implementations**: `KoboLightSensor` for Kobo devices
//! - **Mock implementations**: [`MockLightSensor`](crate::test_mocks::MockLightSensor) for testing
//!
//! ## Trait-Based Design
//!
//! The [`LightSensor`] trait enables:
//! - **Hardware independence**: Auto-brightness code doesn't depend on specific sensor
//! - **Testability**: Mock implementations allow testing without real hardware
//! - **Consistent interface**: All sensors provide light level readings
//!
//! ## Usage
//!
//! ```rust,ignore
//! use plato_core::lightsensor::{LightSensor, KoboLightSensor};
//!
//! let mut sensor = KoboLightSensor::new();
//! let level = sensor.level()?;
//! ```

mod kobo;

use anyhow::Error;

pub use self::kobo::KoboLightSensor;

pub trait LightSensor {
    fn level(&mut self) -> Result<u16, Error>;
}

impl LightSensor for u16 {
    fn level(&mut self) -> Result<u16, Error> {
        Ok(*self)
    }
}
