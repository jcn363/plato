//! Platform-Specific Gesture Configuration
//!
//! Functions for getting platform-optimized gesture parameters.

use crate::consts::gesture::HOLD_JITTER_MM;
use crate::device::CURRENT_DEVICE;
use crate::unit::mm_to_px;

/// Get platform-optimized tap jitter tolerance in millimeters
#[inline]
pub fn platform_tap_jitter_mm() -> f32 {
    if crate::mobile_optimizations::is_mobile_platform() {
        crate::consts::input::MOBILE_TAP_JITTER_MM
    } else {
        crate::consts::input::EINK_TAP_JITTER_MM
    }
}

/// Get platform-optimized hold delay in milliseconds
#[inline]
pub fn platform_hold_delay_ms() -> u64 {
    if crate::mobile_optimizations::is_mobile_platform() {
        crate::consts::input::MOBILE_HOLD_DELAY_MS
    } else {
        crate::consts::input::EINK_HOLD_DELAY_MS
    }
}

/// Get platform-optimized tap jitter in pixels
#[allow(dead_code)]
pub fn platform_tap_jitter_px() -> f32 {
    mm_to_px(platform_tap_jitter_mm(), CURRENT_DEVICE.dpi)
}

/// Get platform-optimized hold jitter in pixels
#[allow(dead_code)]
pub fn platform_hold_jitter_px() -> f32 {
    mm_to_px(HOLD_JITTER_MM, CURRENT_DEVICE.dpi)
}
