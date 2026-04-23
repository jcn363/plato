//! Constants for the Plato emulator.

use std::time::Duration;

/// Application name.
pub const APP_NAME: &str = "Plato";

/// Default rotation for the emulator.
pub const DEFAULT_ROTATION: i8 = 1;

/// Clock refresh interval.
pub const CLOCK_REFRESH_INTERVAL: Duration = Duration::from_secs(60);
