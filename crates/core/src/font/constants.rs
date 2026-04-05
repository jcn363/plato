// Font sizes in 1/64th of a point
pub const FONT_SIZES: [u32; 3] = [349, 524, 629];

pub const KEYBOARD_FONT_SIZES: [u32; 2] = [337, 843];

pub const DISPLAY_FONT_SIZE: u32 = 2516;

pub const NORMAL_STYLE: Style = Style {
    family: Family::SansSerif,
    variant: Variant::REGULAR,
    size: FONT_SIZES[1],
};

pub const SPECIAL_STYLE: Style = Style {
    family: Family::SansSerif,
    variant: Variant::ITALIC,
    size: FONT_SIZES[1],
};

pub const KBD_CHAR: Style = Style {
    family: Family::Keyboard,
    variant: Variant::REGULAR,
    size: KEYBOARD_FONT_SIZES[1],
};

pub const KBD_LABEL: Style = Style {
    family: Family::Keyboard,
    variant: Variant::REGULAR,
    size: FONT_SIZES[0],
};

pub const DISPLAY_STYLE: Style = Style {
    family: Family::Display,
    variant: Variant::REGULAR,
    size: DISPLAY_FONT_SIZE,
};

pub const MD_AUTHOR: Style = Style {
    family: Family::Serif,
    variant: Variant::REGULAR,
    size: FONT_SIZES[1],
};

pub const MD_YEAR: Style = Style {
    family: Family::SansSerif,
    variant: Variant::REGULAR,
    size: FONT_SIZES[0],
};

pub const MD_KIND: Style = Style {
    family: Family::SansSerif,
    variant: Variant::BOLD,
    size: FONT_SIZES[0],
};

pub const MD_SIZE: Style = Style {
    family: Family::SansSerif,
    variant: Variant::REGULAR,
    size: FONT_SIZES[0],
};

pub const SLIDER_VALUE: Style = MD_SIZE;
