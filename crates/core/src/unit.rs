pub const MILLIMETERS_PER_INCH: f32 = 25.4;
pub const CENTIMETERS_PER_INCH: f32 = 2.54;
pub const POINTS_PER_INCH: f32 = 72.0;
pub const PICAS_PER_INCH: f32 = 6.0;

/// Base/reference DPI (300 DPI) for scaling calculations.
/// This is the canonical source for the base DPI value per Single Source of Truth rule.
pub const BASE_DPI: f32 = 300.0;

/// Default DPI for document rendering and UI scaling.
/// Re-exported from BASE_DPI for semantic clarity in different contexts.
pub const DEFAULT_DPI: u16 = BASE_DPI as u16;

#[inline]
pub fn pt_to_px(pt: f32, dpi: u16) -> f32 {
    if dpi == 0 {
        return pt; // Return unchanged if DPI is invalid
    }
    pt * (dpi as f32 / POINTS_PER_INCH)
}

#[inline]
pub fn pc_to_px(pc: f32, dpi: u16) -> f32 {
    if dpi == 0 {
        return pc; // Return unchanged if DPI is invalid
    }
    pc * (dpi as f32 / PICAS_PER_INCH)
}

#[inline]
pub fn in_to_px(inc: f32, dpi: u16) -> f32 {
    if dpi == 0 {
        return inc; // Return unchanged if DPI is invalid
    }
    inc * (dpi as f32)
}

#[inline]
pub fn mm_to_px(mm: f32, dpi: u16) -> f32 {
    if dpi == 0 {
        return mm; // Return unchanged if DPI is invalid
    }
    mm * (dpi as f32 / MILLIMETERS_PER_INCH)
}

#[inline]
pub fn scale_by_dpi_raw(x: f32, dpi: u16) -> f32 {
    if dpi == 0 {
        return x; // Return unchanged if DPI is invalid
    }
    x * (dpi as f32) / BASE_DPI
}

#[inline]
pub fn scale_by_dpi(x: f32, dpi: u16) -> f32 {
    scale_by_dpi_raw(x, dpi).round().max(1.0)
}

/// Get current device DPI.
/// DRY helper to avoid repeated `let dpi = CURRENT_DEVICE.dpi` pattern across view modules.
#[inline]
pub fn get_device_dpi() -> u16 {
    crate::device::CURRENT_DEVICE.dpi
}
