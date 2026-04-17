//! Frontlight Module - Frontlight Control Abstraction
//!
//! This module provides hardware abstraction for frontlight control via the
//! [`Frontlight`] trait. This allows the application to work with different
//! frontlight hardware implementations through a common interface.
//!
//! ## Architecture
//!
//! The module uses a trait-based abstraction:
//!
//! - **[`Frontlight`] trait**: Core abstraction for frontlight operations
//! - **Hardware implementations**: `NaturalFrontlight`, `StandardFrontlight`, `PremixedFrontlight`
//! - **Mock implementations**: [`MockFrontlight`](crate::test_mocks::MockFrontlight) for testing
//!
//! ## Trait-Based Design
//!
//! The [`Frontlight`] trait enables:
//! - **Hardware independence**: UI code doesn't depend on specific frontlight hardware
//! - **Testability**: Mock implementations allow testing without real hardware
//! - **Feature variants**: Different frontlight types (natural, standard, premixed)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use plato_core::frontlight::{Frontlight, NaturalFrontlight};
//!
//! let mut light = NaturalFrontlight::new();
//! light.set_intensity(50.0);
//! light.set_warmth(30.0);
//! ```

mod natural;
mod premixed;
mod standard;

pub use self::natural::NaturalFrontlight;
pub use self::premixed::PremixedFrontlight;
pub use self::standard::StandardFrontlight;
use crate::geom::lerp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LightLevels {
    pub intensity: f32,
    pub warmth: f32,
}

impl Default for LightLevels {
    fn default() -> Self {
        LightLevels {
            intensity: 0.0,
            warmth: 0.0,
        }
    }
}

impl LightLevels {
    pub fn interpolate(self, other: Self, t: f32) -> Self {
        LightLevels {
            intensity: lerp(self.intensity, other.intensity, t),
            warmth: lerp(self.warmth, other.warmth, t),
        }
    }
}

/// Frontlight control trait for Kobo e-readers.
///
/// Not all devices support all features. Warmth control is only available
/// on devices with natural frontlight (ComfortLight Pro).
pub trait Frontlight {
    /// Sets the frontlight intensity as a percentage (0-100).
    fn set_intensity(&mut self, value: f32);

    /// Sets the frontlight color warmth (0-100).
    /// Only supported on devices with natural/comfort light.
    /// On devices without warmth support, this is a no-op.
    fn set_warmth(&mut self, value: f32);

    /// Returns current light levels.
    fn levels(&self) -> LightLevels;
}

impl Frontlight for LightLevels {
    fn set_intensity(&mut self, value: f32) {
        self.intensity = value;
    }

    fn set_warmth(&mut self, value: f32) {
        self.warmth = value;
    }

    fn levels(&self) -> LightLevels {
        *self
    }
}
