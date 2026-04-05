use crate::device::CURRENT_DEVICE;
use crate::font::{Family, Style, Variant};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MD_TITLE: Style = {
        // Compute the ratio between the physical width of the
        // current device and that of the Aura ONE.
        let ratio = (CURRENT_DEVICE.dims.0 as f32 * 300.0) /
                    (CURRENT_DEVICE.dpi as f32 * 1404.0);
        let size = ((super::FONT_SIZES[2] as f32 * ratio) as u32).clamp(super::FONT_SIZES[1],
                                                                          super::FONT_SIZES[2]);
        Style {
            family: Family::Serif,
            variant: Variant::ITALIC,
            size,
        }
    };
}
