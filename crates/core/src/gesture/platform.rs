//! Platform-Specific Gesture Configuration
//!
//! Functions for getting platform-optimized gesture parameters.

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
