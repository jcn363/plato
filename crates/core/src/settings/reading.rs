use crate::metadata::TextAlign;
use fxhash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::defaults::{
    DEFAULT_DITHERED_KINDS, DEFAULT_FONT_FAMILY, DEFAULT_FONT_PATH, DEFAULT_FONT_SIZE,
    DEFAULT_LINE_HEIGHT, DEFAULT_MARGIN_WIDTH, DEFAULT_TEXT_ALIGN, HYPHEN_PENALTY,
    STRETCH_TOLERANCE,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ReaderSettings {
    pub finished: FinishedAction,
    pub south_east_corner: SouthEastCornerAction,
    pub bottom_right_gesture: BottomRightGestureAction,
    pub south_strip: SouthStripAction,
    pub west_strip: WestStripAction,
    pub east_strip: EastStripAction,
    pub strip_width: f32,
    pub corner_width: f32,
    pub font_path: String,
    pub font_family: String,
    pub available_fonts: Vec<String>,
    pub font_size: f32,
    pub min_font_size: f32,
    pub max_font_size: f32,
    pub text_align: TextAlign,
    pub margin_width: i32,
    pub min_margin_width: i32,
    pub max_margin_width: i32,
    pub line_height: f32,
    pub continuous_fit_to_width: bool,
    pub ignore_document_css: bool,
    pub dithered_kinds: FxHashSet<String>,
    pub paragraph_breaker: ParagraphBreakerSettings,
    pub refresh_rate: RefreshRateSettings,
    pub auto_dual_page: bool,
    pub css_overrides: CssOverrides,
    pub show_time: bool,
    pub show_battery: bool,
    pub manga_mode: bool,
    pub duplicates_detection: bool,
    pub external_storage_path: Option<String>,
    pub use_mupdf_search: bool,
    pub page_turn_animation: PageTurnAnimation,
    pub fast_page_turn: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PageTurnAnimation {
    None,
    Slide,
    Fade,
    Flip,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FinishedAction {
    Notify,
    Close,
    GoToNext,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SouthEastCornerAction {
    NextPage,
    GoToPage,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BottomRightGestureAction {
    ToggleDithered,
    ToggleInverted,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SouthStripAction {
    ToggleBars,
    NextPage,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EastStripAction {
    PreviousPage,
    NextPage,
    None,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WestStripAction {
    PreviousPage,
    NextPage,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ParagraphBreakerSettings {
    pub hyphen_penalty: i32,
    pub stretch_tolerance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct RefreshRateSettings {
    #[serde(flatten)]
    pub global: RefreshRatePair,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub by_kind: HashMap<String, RefreshRatePair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RefreshRatePair {
    pub regular: u8,
    pub inverted: u8,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct CssOverrides {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub margin_top: Option<i32>,
    pub margin_bottom: Option<i32>,
    pub margin_left: Option<i32>,
    pub margin_right: Option<i32>,
    pub text_align: Option<TextAlign>,
    pub force_hyphenation: bool,
    pub disable_indent: bool,
    pub footnotes_as_popups: bool,
    pub image_brightness: Option<f32>,
    pub image_contrast: Option<f32>,
}

impl Default for RefreshRateSettings {
    fn default() -> Self {
        RefreshRateSettings {
            global: RefreshRatePair {
                regular: 8,
                inverted: 2,
            },
            by_kind: HashMap::new(),
        }
    }
}

impl Default for ParagraphBreakerSettings {
    fn default() -> Self {
        ParagraphBreakerSettings {
            hyphen_penalty: HYPHEN_PENALTY,
            stretch_tolerance: STRETCH_TOLERANCE,
        }
    }
}

impl Default for ReaderSettings {
    fn default() -> Self {
        ReaderSettings {
            finished: FinishedAction::Close,
            south_east_corner: SouthEastCornerAction::GoToPage,
            bottom_right_gesture: BottomRightGestureAction::ToggleDithered,
            south_strip: SouthStripAction::ToggleBars,
            west_strip: WestStripAction::PreviousPage,
            east_strip: EastStripAction::NextPage,
            strip_width: 0.6,
            corner_width: 0.4,
            font_path: DEFAULT_FONT_PATH.to_string(),
            font_family: DEFAULT_FONT_FAMILY.to_string(),
            available_fonts: Vec::new(),
            font_size: DEFAULT_FONT_SIZE,
            min_font_size: DEFAULT_FONT_SIZE / 2.0,
            max_font_size: DEFAULT_FONT_SIZE * 1.5,
            text_align: DEFAULT_TEXT_ALIGN,
            margin_width: DEFAULT_MARGIN_WIDTH,
            min_margin_width: 0,
            max_margin_width: 10,
            line_height: DEFAULT_LINE_HEIGHT,
            continuous_fit_to_width: true,
            ignore_document_css: false,
            dithered_kinds: DEFAULT_DITHERED_KINDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            manga_mode: false,
            use_mupdf_search: false,
            auto_dual_page: false,
            css_overrides: CssOverrides::default(),
            show_time: true,
            show_battery: true,
            duplicates_detection: false,
            external_storage_path: None,
            paragraph_breaker: ParagraphBreakerSettings::default(),
            refresh_rate: RefreshRateSettings::default(),
            page_turn_animation: PageTurnAnimation::None,
            fast_page_turn: false,
        }
    }
}
