use crate::metadata::TextAlign;

pub const PLATO_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SETTINGS_PATH: &str = "Settings.toml";
pub const DEFAULT_FONT_PATH: &str = "/mnt/onboard/fonts";
pub const INTERNAL_CARD_ROOT: &str = "/mnt/onboard";
pub const EXTERNAL_CARD_ROOT: &str = "/mnt/sd";
pub const LOGO_SPECIAL_PATH: &str = "logo:";
pub const COVER_SPECIAL_PATH: &str = "cover:";
pub const DEFAULT_FONT_SIZE: f32 = 11.0;
pub const DEFAULT_DICTIONARY_FONT_SIZE: f32 = 11.0;
pub const DEFAULT_MARGIN_WIDTH: i32 = 8;
pub const DEFAULT_LINE_HEIGHT: f32 = 1.2;
pub const DEFAULT_FONT_FAMILY: &str = "Libertinus Serif";
pub const DEFAULT_TEXT_ALIGN: TextAlign = TextAlign::Left;
pub const DEFAULT_DITHERED_KINDS: &[&str] = &["cbz", "jpg", "png", "jpeg"];
pub const HYPHEN_PENALTY: i32 = 50;
pub const STRETCH_TOLERANCE: f32 = 1.26;
