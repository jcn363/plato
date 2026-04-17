//! Font Rendering Subsystem
//!
//! This module provides font handling for Plato, using pure Rust libraries:
//! - skrifa for font loading and parsing
//! - rustybuzz for text shaping and layout
//! - ab_glyph for glyph rasterization (when needed)
//!
//! ## Architecture
//!
//! - **skrifa_wrapper**: Safe wrapper around skrifa for font loading and metrics
//! - **rustybuzz_wrapper**: Safe wrapper around rustybuzz for text shaping
//!
//! The subsystem handles:
//! - Font discovery and loading from filesystem
//! - Embedded font resources
//! - Glyph rasterization via ab_glyph (optional)
//! - Text shaping (glyph positioning) via rustybuzz
//! - Variable font support
//! - Complex script handling

pub mod face;
pub mod library;
mod rustybuzz_wrapper;
pub mod shaper;
mod skrifa_wrapper;
mod types;

// Public re-exports
pub use self::face::Font;
pub use self::library::FontOpener;
pub use self::types::{GlyphPlan, RenderPlan};

// ===========================================================================
// Imports and Re-exports
// ===========================================================================

use crate::device::CURRENT_DEVICE;
use crate::helpers::IsHidden;
use crate::{log_error, log_warn};
use anyhow::{format_err, Error};
use bitflags::bitflags;
use globset::Glob;
use rustc_hash::FxHashMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::str;
use std::sync::LazyLock;
use walkdir::WalkDir;

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

pub static MD_TITLE: LazyLock<Style> = LazyLock::new(|| {
    // Compute the ratio between the physical width of the
    // current device and that of the Aura ONE.
    let ratio = (CURRENT_DEVICE.dims.0 as f32 * 300.0) / (CURRENT_DEVICE.dpi as f32 * 1404.0);
    let size = ((FONT_SIZES[2] as f32 * ratio) as u32).clamp(FONT_SIZES[1], FONT_SIZES[2]);
    Style {
        family: Family::Serif,
        variant: Variant::ITALIC,
        size,
    }
});

// ===========================================================================
// Font Size Constants and Style Definitions
// ===========================================================================

pub const MD_AUTHOR: Style = Style {
    family: Family::Serif,
    variant: Variant::REGULAR,
    size: FONT_SIZES[1],
};

pub const MD_YEAR: Style = NORMAL_STYLE;

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

// ===========================================================================
// Embedded Font Data Declarations (platform-specific)
// ===========================================================================

#[cfg(any(not(target_os = "linux"), target_arch = "arm"))]
#[link(name = "mupdf")]
extern "C" {
    // Based on the outputs of:
    // arm-linux-gnueabihf-readelf -Ws ./libs/libmupdf.so | grep '\b_binary_' | \
    // grep -v '_size$' | awk '{print $8, strtonum($3)-1}' | sort -u
    pub static _binary_DroidSansFallback_ttf: [libc::c_uchar; 3556308];
    pub static _binary_NotoEmoji_Regular_ttf: [libc::c_uchar; 418804];
    pub static _binary_NotoMusic_Regular_otf: [libc::c_uchar; 60812];
    pub static _binary_NotoNaskhArabic_Regular_otf: [libc::c_uchar; 119664];
    pub static _binary_NotoNastaliqUrdu_Regular_otf: [libc::c_uchar; 373220];
    pub static _binary_NotoSans_Regular_otf: [libc::c_uchar; 290336];
    pub static _binary_NotoSansAdlam_Regular_otf: [libc::c_uchar; 33448];
    pub static _binary_NotoSansAnatolianHieroglyphs_Regular_otf: [libc::c_uchar; 134420];
    pub static _binary_NotoSansAvestan_Regular_otf: [libc::c_uchar; 9300];
    pub static _binary_NotoSansBamum_Regular_otf: [libc::c_uchar; 103668];
    pub static _binary_NotoSansBassaVah_Regular_otf: [libc::c_uchar; 6300];
    pub static _binary_NotoSansBatak_Regular_otf: [libc::c_uchar; 11108];
    pub static _binary_NotoSansBhaiksuki_Regular_otf: [libc::c_uchar; 121620];
    pub static _binary_NotoSansBrahmi_Regular_otf: [libc::c_uchar; 29544];
    pub static _binary_NotoSansBuginese_Regular_otf: [libc::c_uchar; 6256];
    pub static _binary_NotoSansBuhid_Regular_otf: [libc::c_uchar; 5076];
    pub static _binary_NotoSansCanadianAboriginal_Regular_otf: [libc::c_uchar; 38068];
    pub static _binary_NotoSansCarian_Regular_otf: [libc::c_uchar; 5592];
    pub static _binary_NotoSansCaucasianAlbanian_Regular_otf: [libc::c_uchar; 17388];
    pub static _binary_NotoSansChakma_Regular_otf: [libc::c_uchar; 29488];
    pub static _binary_NotoSansCham_Regular_otf: [libc::c_uchar; 21224];
    pub static _binary_NotoSansCherokee_Regular_otf: [libc::c_uchar; 57308];
    pub static _binary_NotoSansChorasmian_Regular_otf: [libc::c_uchar; 12460];
    pub static _binary_NotoSansCoptic_Regular_otf: [libc::c_uchar; 21380];
    pub static _binary_NotoSansCuneiform_Regular_otf: [libc::c_uchar; 416308];
    pub static _binary_NotoSansCypriot_Regular_otf: [libc::c_uchar; 7024];
    pub static _binary_NotoSansCyproMinoan_Regular_otf: [libc::c_uchar; 8568];
    pub static _binary_NotoSansDeseret_Regular_otf: [libc::c_uchar; 9016];
    pub static _binary_NotoSansDuployan_Regular_otf: [libc::c_uchar; 10276];
    pub static _binary_NotoSansEgyptianHieroglyphs_Regular_otf: [libc::c_uchar; 362960];
    pub static _binary_NotoSansElbasan_Regular_otf: [libc::c_uchar; 8684];
    pub static _binary_NotoSansElymaic_Regular_otf: [libc::c_uchar; 7620];
    pub static _binary_NotoSansGlagolitic_Regular_otf: [libc::c_uchar; 17176];
    pub static _binary_NotoSansGothic_Regular_otf: [libc::c_uchar; 5416];
    pub static _binary_NotoSansGunjalaGondi_Regular_otf: [libc::c_uchar; 32372];
    pub static _binary_NotoSansHanifiRohingya_Regular_otf: [libc::c_uchar; 16576];
    pub static _binary_NotoSansHanunoo_Regular_otf: [libc::c_uchar; 6596];
    pub static _binary_NotoSansHatran_Regular_otf: [libc::c_uchar; 4324];
    pub static _binary_NotoSansImperialAramaic_Regular_otf: [libc::c_uchar; 5436];
    pub static _binary_NotoSansInscriptionalPahlavi_Regular_otf: [libc::c_uchar; 5464];
    pub static _binary_NotoSansInscriptionalParthian_Regular_otf: [libc::c_uchar; 6788];
    pub static _binary_NotoSansJavanese_Regular_otf: [libc::c_uchar; 86944];
    pub static _binary_NotoSansKaithi_Regular_otf: [libc::c_uchar; 39756];
    pub static _binary_NotoSansKawi_Regular_otf: [libc::c_uchar; 30940];
    pub static _binary_NotoSansKayahLi_Regular_otf: [libc::c_uchar; 7100];
    pub static _binary_NotoSansKharoshthi_Regular_otf: [libc::c_uchar; 27708];
    pub static _binary_NotoSansKhudawadi_Regular_otf: [libc::c_uchar; 14764];
    pub static _binary_NotoSansLepcha_Regular_otf: [libc::c_uchar; 18832];
    pub static _binary_NotoSansLimbu_Regular_otf: [libc::c_uchar; 10040];
    pub static _binary_NotoSansLinearA_Regular_otf: [libc::c_uchar; 33640];
    pub static _binary_NotoSansLinearB_Regular_otf: [libc::c_uchar; 36892];
    pub static _binary_NotoSansLisu_Regular_otf: [libc::c_uchar; 5688];
    pub static _binary_NotoSansLycian_Regular_otf: [libc::c_uchar; 4108];
    pub static _binary_NotoSansLydian_Regular_otf: [libc::c_uchar; 4088];
    pub static _binary_NotoSansMahajani_Regular_otf: [libc::c_uchar; 10136];
    pub static _binary_NotoSansMandaic_Regular_otf: [libc::c_uchar; 13160];
    pub static _binary_NotoSansManichaean_Regular_otf: [libc::c_uchar; 16496];
    pub static _binary_NotoSansMarchen_Regular_otf: [libc::c_uchar; 69240];
    pub static _binary_NotoSansMasaramGondi_Regular_otf: [libc::c_uchar; 23052];
    pub static _binary_NotoSansMath_Regular_otf: [libc::c_uchar; 258796];
    pub static _binary_NotoSansMedefaidrin_Regular_otf: [libc::c_uchar; 27060];
    pub static _binary_NotoSansMeeteiMayek_Regular_otf: [libc::c_uchar; 13056];
    pub static _binary_NotoSansMendeKikakui_Regular_otf: [libc::c_uchar; 19664];
    pub static _binary_NotoSansMeroitic_Regular_otf: [libc::c_uchar; 19980];
    pub static _binary_NotoSansMiao_Regular_otf: [libc::c_uchar; 26460];
    pub static _binary_NotoSansModi_Regular_otf: [libc::c_uchar; 29412];
    pub static _binary_NotoSansMongolian_Regular_otf: [libc::c_uchar; 111040];
    pub static _binary_NotoSansMro_Regular_otf: [libc::c_uchar; 5608];
    pub static _binary_NotoSansMultani_Regular_otf: [libc::c_uchar; 7852];
    pub static _binary_NotoSansNabataean_Regular_otf: [libc::c_uchar; 6448];
    pub static _binary_NotoSansNagMundari_Regular_otf: [libc::c_uchar; 8612];
    pub static _binary_NotoSansNandinagari_Regular_otf: [libc::c_uchar; 86940];
    pub static _binary_NotoSansNewa_Regular_otf: [libc::c_uchar; 99568];
    pub static _binary_NotoSansNewTaiLue_Regular_otf: [libc::c_uchar; 10884];
    pub static _binary_NotoSansNKo_Regular_otf: [libc::c_uchar; 15164];
    pub static _binary_NotoSansNushu_Regular_otf: [libc::c_uchar; 72472];
    pub static _binary_NotoSansOgham_Regular_otf: [libc::c_uchar; 3720];
    pub static _binary_NotoSansOlChiki_Regular_otf: [libc::c_uchar; 7024];
    pub static _binary_NotoSansOldHungarian_Regular_otf: [libc::c_uchar; 44628];
    pub static _binary_NotoSansOldItalic_Regular_otf: [libc::c_uchar; 6360];
    pub static _binary_NotoSansOldNorthArabian_Regular_otf: [libc::c_uchar; 6132];
    pub static _binary_NotoSansOldPermic_Regular_otf: [libc::c_uchar; 8512];
    pub static _binary_NotoSansOldPersian_Regular_otf: [libc::c_uchar; 9856];
    pub static _binary_NotoSansOldSogdian_Regular_otf: [libc::c_uchar; 12260];
    pub static _binary_NotoSansOldSouthArabian_Regular_otf: [libc::c_uchar; 4624];
    pub static _binary_NotoSansOldTurkic_Regular_otf: [libc::c_uchar; 6884];
    pub static _binary_NotoSansOsage_Regular_otf: [libc::c_uchar; 9292];
    pub static _binary_NotoSansOsmanya_Regular_otf: [libc::c_uchar; 6784];
    pub static _binary_NotoSansPahawhHmong_Regular_otf: [libc::c_uchar; 13024];
    pub static _binary_NotoSansPalmyrene_Regular_otf: [libc::c_uchar; 8480];
    pub static _binary_NotoSansPauCinHau_Regular_otf: [libc::c_uchar; 8124];
    pub static _binary_NotoSansPhagsPa_Regular_otf: [libc::c_uchar; 24036];
    pub static _binary_NotoSansPhoenician_Regular_otf: [libc::c_uchar; 5288];
    pub static _binary_NotoSansPsalterPahlavi_Regular_otf: [libc::c_uchar; 12748];
    pub static _binary_NotoSansRejang_Regular_otf: [libc::c_uchar; 6440];
    pub static _binary_NotoSansRunic_Regular_otf: [libc::c_uchar; 7200];
    pub static _binary_NotoSansSamaritan_Regular_otf: [libc::c_uchar; 9024];
    pub static _binary_NotoSansSaurashtra_Regular_otf: [libc::c_uchar; 16020];
    pub static _binary_NotoSansSharada_Regular_otf: [libc::c_uchar; 32824];
    pub static _binary_NotoSansShavian_Regular_otf: [libc::c_uchar; 5468];
    pub static _binary_NotoSansSiddham_Regular_otf: [libc::c_uchar; 91992];
    pub static _binary_NotoSansSignWriting_Regular_otf: [libc::c_uchar; 2780224];
    pub static _binary_NotoSansSogdian_Regular_otf: [libc::c_uchar; 48356];
    pub static _binary_NotoSansSoraSompeng_Regular_otf: [libc::c_uchar; 6332];
    pub static _binary_NotoSansSoyombo_Regular_otf: [libc::c_uchar; 52036];
    pub static _binary_NotoSansSundanese_Regular_otf: [libc::c_uchar; 9420];
    pub static _binary_NotoSansSylotiNagri_Regular_otf: [libc::c_uchar; 12852];
    pub static _binary_NotoSansSymbols_Regular_otf: [libc::c_uchar; 109696];
    pub static _binary_NotoSansSymbols2_Regular_otf: [libc::c_uchar; 375388];
    pub static _binary_NotoSansSyriac_Regular_otf: [libc::c_uchar; 124756];
    pub static _binary_NotoSansTagalog_Regular_otf: [libc::c_uchar; 5500];
    pub static _binary_NotoSansTagbanwa_Regular_otf: [libc::c_uchar; 5356];
    pub static _binary_NotoSansTaiLe_Regular_otf: [libc::c_uchar; 8616];
    pub static _binary_NotoSansTaiTham_Regular_otf: [libc::c_uchar; 76880];
    pub static _binary_NotoSansTaiViet_Regular_otf: [libc::c_uchar; 12280];
    pub static _binary_NotoSansTakri_Regular_otf: [libc::c_uchar; 17864];
    pub static _binary_NotoSansTangsa_Regular_otf: [libc::c_uchar; 16908];
    pub static _binary_NotoSansThaana_Regular_otf: [libc::c_uchar; 12392];
    pub static _binary_NotoSansTifinagh_Regular_otf: [libc::c_uchar; 24776];
    pub static _binary_NotoSansTirhuta_Regular_otf: [libc::c_uchar; 52432];
    pub static _binary_NotoSansUgaritic_Regular_otf: [libc::c_uchar; 5048];
    pub static _binary_NotoSansVai_Regular_otf: [libc::c_uchar; 24088];
    pub static _binary_NotoSansWancho_Regular_otf: [libc::c_uchar; 15140];
    pub static _binary_NotoSansWarangCiti_Regular_otf: [libc::c_uchar; 23484];
    pub static _binary_NotoSansYi_Regular_otf: [libc::c_uchar; 92164];
    pub static _binary_NotoSansZanabazarSquare_Regular_otf: [libc::c_uchar; 13804];
    pub static _binary_NotoSerif_Regular_otf: [libc::c_uchar; 289412];
    pub static _binary_NotoSerifAhom_Regular_otf: [libc::c_uchar; 14516];
    pub static _binary_NotoSerifArmenian_Regular_otf: [libc::c_uchar; 14160];
    pub static _binary_NotoSerifBalinese_Regular_otf: [libc::c_uchar; 32348];
    pub static _binary_NotoSerifBengali_Regular_otf: [libc::c_uchar; 101332];
    pub static _binary_NotoSerifDevanagari_Regular_otf: [libc::c_uchar; 169744];
    pub static _binary_NotoSerifDivesAkuru_Regular_otf: [libc::c_uchar; 27972];
    pub static _binary_NotoSerifDogra_Regular_otf: [libc::c_uchar; 19944];
    pub static _binary_NotoSerifEthiopic_Regular_otf: [libc::c_uchar; 113328];
    pub static _binary_NotoSerifGeorgian_Regular_otf: [libc::c_uchar; 31988];
    pub static _binary_NotoSerifGrantha_Regular_otf: [libc::c_uchar; 368396];
    pub static _binary_NotoSerifGujarati_Regular_otf: [libc::c_uchar; 64848];
    pub static _binary_NotoSerifGurmukhi_Regular_otf: [libc::c_uchar; 26992];
    pub static _binary_NotoSerifHebrew_Regular_otf: [libc::c_uchar; 15320];
    pub static _binary_NotoSerifKannada_Regular_otf: [libc::c_uchar; 89032];
    pub static _binary_NotoSerifKhitanSmallScript_Regular_otf: [libc::c_uchar; 508920];
    pub static _binary_NotoSerifKhmer_Regular_otf: [libc::c_uchar; 40436];
    pub static _binary_NotoSerifKhojki_Regular_otf: [libc::c_uchar; 60112];
    pub static _binary_NotoSerifLao_Regular_otf: [libc::c_uchar; 16196];
    pub static _binary_NotoSerifMakasar_Regular_otf: [libc::c_uchar; 5864];
    pub static _binary_NotoSerifMalayalam_Regular_otf: [libc::c_uchar; 45668];
    pub static _binary_NotoSerifMyanmar_Regular_otf: [libc::c_uchar; 127564];
    pub static _binary_NotoSerifNyiakengPuachueHmong_Regular_otf: [libc::c_uchar; 12208];
    pub static _binary_NotoSerifOldUyghur_Regular_otf: [libc::c_uchar; 15620];
    pub static _binary_NotoSerifOriya_Regular_otf: [libc::c_uchar; 105824];
    pub static _binary_NotoSerifSinhala_Regular_otf: [libc::c_uchar; 74924];
    pub static _binary_NotoSerifTamil_Regular_otf: [libc::c_uchar; 33752];
    pub static _binary_NotoSerifTelugu_Regular_otf: [libc::c_uchar; 82032];
    pub static _binary_NotoSerifThai_Regular_otf: [libc::c_uchar; 17556];
    pub static _binary_NotoSerifTibetan_Regular_otf: [libc::c_uchar; 334156];
    pub static _binary_NotoSerifToto_Regular_otf: [libc::c_uchar; 5732];
    pub static _binary_NotoSerifVithkuqi_Regular_otf: [libc::c_uchar; 42508];
    pub static _binary_NotoSerifYezidi_Regular_otf: [libc::c_uchar; 8664];
}

#[cfg(all(target_os = "linux", not(target_arch = "arm")))]
#[link(name = "mupdf")]
extern "C" {
    pub static _binary_resources_fonts_droid_DroidSansFallback_ttf_start: [libc::c_uchar; 3556308];
    pub static _binary_resources_fonts_noto_NotoEmoji_Regular_ttf_start: [libc::c_uchar; 418804];
    pub static _binary_resources_fonts_noto_NotoMusic_Regular_otf_start: [libc::c_uchar; 60812];
    pub static _binary_resources_fonts_noto_NotoNaskhArabic_Regular_otf_start:
        [libc::c_uchar; 119664];
    pub static _binary_resources_fonts_noto_NotoNastaliqUrdu_Regular_otf_start:
        [libc::c_uchar; 373220];
    pub static _binary_resources_fonts_noto_NotoSans_Regular_otf_start: [libc::c_uchar; 290336];
    pub static _binary_resources_fonts_noto_NotoSansAdlam_Regular_otf_start: [libc::c_uchar; 33448];
    pub static _binary_resources_fonts_noto_NotoSansAnatolianHieroglyphs_Regular_otf_start:
        [libc::c_uchar; 134420];
    pub static _binary_resources_fonts_noto_NotoSansAvestan_Regular_otf_start:
        [libc::c_uchar; 9300];
    pub static _binary_resources_fonts_noto_NotoSansBamum_Regular_otf_start:
        [libc::c_uchar; 103668];
    pub static _binary_resources_fonts_noto_NotoSansBassaVah_Regular_otf_start:
        [libc::c_uchar; 6300];
    pub static _binary_resources_fonts_noto_NotoSansBatak_Regular_otf_start: [libc::c_uchar; 11108];
    pub static _binary_resources_fonts_noto_NotoSansBhaiksuki_Regular_otf_start:
        [libc::c_uchar; 121620];
    pub static _binary_resources_fonts_noto_NotoSansBrahmi_Regular_otf_start:
        [libc::c_uchar; 29544];
    pub static _binary_resources_fonts_noto_NotoSansBuginese_Regular_otf_start:
        [libc::c_uchar; 6256];
    pub static _binary_resources_fonts_noto_NotoSansBuhid_Regular_otf_start: [libc::c_uchar; 5076];
    pub static _binary_resources_fonts_noto_NotoSansCanadianAboriginal_Regular_otf_start:
        [libc::c_uchar; 38068];
    pub static _binary_resources_fonts_noto_NotoSansCarian_Regular_otf_start: [libc::c_uchar; 5592];
    pub static _binary_resources_fonts_noto_NotoSansCaucasianAlbanian_Regular_otf_start:
        [libc::c_uchar; 17388];
    pub static _binary_resources_fonts_noto_NotoSansChakma_Regular_otf_start:
        [libc::c_uchar; 29488];
    pub static _binary_resources_fonts_noto_NotoSansCham_Regular_otf_start: [libc::c_uchar; 21224];
    pub static _binary_resources_fonts_noto_NotoSansCherokee_Regular_otf_start:
        [libc::c_uchar; 57308];
    pub static _binary_resources_fonts_noto_NotoSansChorasmian_Regular_otf_start:
        [libc::c_uchar; 12460];
    pub static _binary_resources_fonts_noto_NotoSansCoptic_Regular_otf_start:
        [libc::c_uchar; 21380];
    pub static _binary_resources_fonts_noto_NotoSansCuneiform_Regular_otf_start:
        [libc::c_uchar; 416308];
    pub static _binary_resources_fonts_noto_NotoSansCypriot_Regular_otf_start:
        [libc::c_uchar; 7024];
    pub static _binary_resources_fonts_noto_NotoSansCyproMinoan_Regular_otf_start:
        [libc::c_uchar; 8568];
    pub static _binary_resources_fonts_noto_NotoSansDeseret_Regular_otf_start:
        [libc::c_uchar; 9016];
    pub static _binary_resources_fonts_noto_NotoSansDuployan_Regular_otf_start:
        [libc::c_uchar; 10276];
    pub static _binary_resources_fonts_noto_NotoSansEgyptianHieroglyphs_Regular_otf_start:
        [libc::c_uchar; 362960];
    pub static _binary_resources_fonts_noto_NotoSansElbasan_Regular_otf_start:
        [libc::c_uchar; 8684];
    pub static _binary_resources_fonts_noto_NotoSansElymaic_Regular_otf_start:
        [libc::c_uchar; 7620];
    pub static _binary_resources_fonts_noto_NotoSansGlagolitic_Regular_otf_start:
        [libc::c_uchar; 17176];
    pub static _binary_resources_fonts_noto_NotoSansGothic_Regular_otf_start: [libc::c_uchar; 5416];
    pub static _binary_resources_fonts_noto_NotoSansGunjalaGondi_Regular_otf_start:
        [libc::c_uchar; 32372];
    pub static _binary_resources_fonts_noto_NotoSansHanifiRohingya_Regular_otf_start:
        [libc::c_uchar; 16576];
    pub static _binary_resources_fonts_noto_NotoSansHanunoo_Regular_otf_start:
        [libc::c_uchar; 6596];
    pub static _binary_resources_fonts_noto_NotoSansHatran_Regular_otf_start: [libc::c_uchar; 4324];
    pub static _binary_resources_fonts_noto_NotoSansImperialAramaic_Regular_otf_start:
        [libc::c_uchar; 5436];
    pub static _binary_resources_fonts_noto_NotoSansInscriptionalPahlavi_Regular_otf_start:
        [libc::c_uchar; 5464];
    pub static _binary_resources_fonts_noto_NotoSansInscriptionalParthian_Regular_otf_start:
        [libc::c_uchar; 6788];
    pub static _binary_resources_fonts_noto_NotoSansJavanese_Regular_otf_start:
        [libc::c_uchar; 86944];
    pub static _binary_resources_fonts_noto_NotoSansKaithi_Regular_otf_start:
        [libc::c_uchar; 39756];
    pub static _binary_resources_fonts_noto_NotoSansKawi_Regular_otf_start: [libc::c_uchar; 30940];
    pub static _binary_resources_fonts_noto_NotoSansKayahLi_Regular_otf_start:
        [libc::c_uchar; 7100];
    pub static _binary_resources_fonts_noto_NotoSansKharoshthi_Regular_otf_start:
        [libc::c_uchar; 27708];
    pub static _binary_resources_fonts_noto_NotoSansKhudawadi_Regular_otf_start:
        [libc::c_uchar; 14764];
    pub static _binary_resources_fonts_noto_NotoSansLepcha_Regular_otf_start:
        [libc::c_uchar; 18832];
    pub static _binary_resources_fonts_noto_NotoSansLimbu_Regular_otf_start: [libc::c_uchar; 10040];
    pub static _binary_resources_fonts_noto_NotoSansLinearA_Regular_otf_start:
        [libc::c_uchar; 33640];
    pub static _binary_resources_fonts_noto_NotoSansLinearB_Regular_otf_start:
        [libc::c_uchar; 36892];
    pub static _binary_resources_fonts_noto_NotoSansLisu_Regular_otf_start: [libc::c_uchar; 5688];
    pub static _binary_resources_fonts_noto_NotoSansLycian_Regular_otf_start: [libc::c_uchar; 4108];
    pub static _binary_resources_fonts_noto_NotoSansLydian_Regular_otf_start: [libc::c_uchar; 4088];
    pub static _binary_resources_fonts_noto_NotoSansMahajani_Regular_otf_start:
        [libc::c_uchar; 10136];
    pub static _binary_resources_fonts_noto_NotoSansMandaic_Regular_otf_start:
        [libc::c_uchar; 13160];
    pub static _binary_resources_fonts_noto_NotoSansManichaean_Regular_otf_start:
        [libc::c_uchar; 16496];
    pub static _binary_resources_fonts_noto_NotoSansMarchen_Regular_otf_start:
        [libc::c_uchar; 69240];
    pub static _binary_resources_fonts_noto_NotoSansMasaramGondi_Regular_otf_start:
        [libc::c_uchar; 23052];
    pub static _binary_resources_fonts_noto_NotoSansMath_Regular_otf_start: [libc::c_uchar; 258796];
    pub static _binary_resources_fonts_noto_NotoSansMedefaidrin_Regular_otf_start:
        [libc::c_uchar; 27060];
    pub static _binary_resources_fonts_noto_NotoSansMeeteiMayek_Regular_otf_start:
        [libc::c_uchar; 13056];
    pub static _binary_resources_fonts_noto_NotoSansMendeKikakui_Regular_otf_start:
        [libc::c_uchar; 19664];
    pub static _binary_resources_fonts_noto_NotoSansMeroitic_Regular_otf_start:
        [libc::c_uchar; 19980];
    pub static _binary_resources_fonts_noto_NotoSansMiao_Regular_otf_start: [libc::c_uchar; 26460];
    pub static _binary_resources_fonts_noto_NotoSansModi_Regular_otf_start: [libc::c_uchar; 29412];
    pub static _binary_resources_fonts_noto_NotoSansMongolian_Regular_otf_start:
        [libc::c_uchar; 111040];
    pub static _binary_resources_fonts_noto_NotoSansMro_Regular_otf_start: [libc::c_uchar; 5608];
    pub static _binary_resources_fonts_noto_NotoSansMultani_Regular_otf_start:
        [libc::c_uchar; 7852];
    pub static _binary_resources_fonts_noto_NotoSansNabataean_Regular_otf_start:
        [libc::c_uchar; 6448];
    pub static _binary_resources_fonts_noto_NotoSansNagMundari_Regular_otf_start:
        [libc::c_uchar; 8612];
    pub static _binary_resources_fonts_noto_NotoSansNandinagari_Regular_otf_start:
        [libc::c_uchar; 86940];
    pub static _binary_resources_fonts_noto_NotoSansNewa_Regular_otf_start: [libc::c_uchar; 99568];
    pub static _binary_resources_fonts_noto_NotoSansNewTaiLue_Regular_otf_start:
        [libc::c_uchar; 10884];
    pub static _binary_resources_fonts_noto_NotoSansNKo_Regular_otf_start: [libc::c_uchar; 15164];
    pub static _binary_resources_fonts_noto_NotoSansNushu_Regular_otf_start: [libc::c_uchar; 72472];
    pub static _binary_resources_fonts_noto_NotoSansOgham_Regular_otf_start: [libc::c_uchar; 3720];
    pub static _binary_resources_fonts_noto_NotoSansOlChiki_Regular_otf_start:
        [libc::c_uchar; 7024];
    pub static _binary_resources_fonts_noto_NotoSansOldHungarian_Regular_otf_start:
        [libc::c_uchar; 44628];
    pub static _binary_resources_fonts_noto_NotoSansOldItalic_Regular_otf_start:
        [libc::c_uchar; 6360];
    pub static _binary_resources_fonts_noto_NotoSansOldNorthArabian_Regular_otf_start:
        [libc::c_uchar; 6132];
    pub static _binary_resources_fonts_noto_NotoSansOldPermic_Regular_otf_start:
        [libc::c_uchar; 8512];
    pub static _binary_resources_fonts_noto_NotoSansOldPersian_Regular_otf_start:
        [libc::c_uchar; 9856];
    pub static _binary_resources_fonts_noto_NotoSansOldSogdian_Regular_otf_start:
        [libc::c_uchar; 12260];
    pub static _binary_resources_fonts_noto_NotoSansOldSouthArabian_Regular_otf_start:
        [libc::c_uchar; 4624];
    pub static _binary_resources_fonts_noto_NotoSansOldTurkic_Regular_otf_start:
        [libc::c_uchar; 6884];
    pub static _binary_resources_fonts_noto_NotoSansOsage_Regular_otf_start: [libc::c_uchar; 9292];
    pub static _binary_resources_fonts_noto_NotoSansOsmanya_Regular_otf_start:
        [libc::c_uchar; 6784];
    pub static _binary_resources_fonts_noto_NotoSansPahawhHmong_Regular_otf_start:
        [libc::c_uchar; 13024];
    pub static _binary_resources_fonts_noto_NotoSansPalmyrene_Regular_otf_start:
        [libc::c_uchar; 8480];
    pub static _binary_resources_fonts_noto_NotoSansPauCinHau_Regular_otf_start:
        [libc::c_uchar; 8124];
    pub static _binary_resources_fonts_noto_NotoSansPhagsPa_Regular_otf_start:
        [libc::c_uchar; 24036];
    pub static _binary_resources_fonts_noto_NotoSansPhoenician_Regular_otf_start:
        [libc::c_uchar; 5288];
    pub static _binary_resources_fonts_noto_NotoSansPsalterPahlavi_Regular_otf_start:
        [libc::c_uchar; 12748];
    pub static _binary_resources_fonts_noto_NotoSansRejang_Regular_otf_start: [libc::c_uchar; 6440];
    pub static _binary_resources_fonts_noto_NotoSansRunic_Regular_otf_start: [libc::c_uchar; 7200];
    pub static _binary_resources_fonts_noto_NotoSansSamaritan_Regular_otf_start:
        [libc::c_uchar; 9024];
    pub static _binary_resources_fonts_noto_NotoSansSaurashtra_Regular_otf_start:
        [libc::c_uchar; 16020];
    pub static _binary_resources_fonts_noto_NotoSansSharada_Regular_otf_start:
        [libc::c_uchar; 32824];
    pub static _binary_resources_fonts_noto_NotoSansShavian_Regular_otf_start:
        [libc::c_uchar; 5468];
    pub static _binary_resources_fonts_noto_NotoSansSiddham_Regular_otf_start:
        [libc::c_uchar; 91992];
    pub static _binary_resources_fonts_noto_NotoSansSignWriting_Regular_otf_start:
        [libc::c_uchar; 2780224];
    pub static _binary_resources_fonts_noto_NotoSansSogdian_Regular_otf_start:
        [libc::c_uchar; 48356];
    pub static _binary_resources_fonts_noto_NotoSansSoraSompeng_Regular_otf_start:
        [libc::c_uchar; 6332];
    pub static _binary_resources_fonts_noto_NotoSansSoyombo_Regular_otf_start:
        [libc::c_uchar; 52036];
    pub static _binary_resources_fonts_noto_NotoSansSundanese_Regular_otf_start:
        [libc::c_uchar; 9420];
    pub static _binary_resources_fonts_noto_NotoSansSylotiNagri_Regular_otf_start:
        [libc::c_uchar; 12852];
    pub static _binary_resources_fonts_noto_NotoSansSymbols_Regular_otf_start:
        [libc::c_uchar; 109696];
    pub static _binary_resources_fonts_noto_NotoSansSymbols2_Regular_otf_start:
        [libc::c_uchar; 375388];
    pub static _binary_resources_fonts_noto_NotoSansSyriac_Regular_otf_start:
        [libc::c_uchar; 124756];
    pub static _binary_resources_fonts_noto_NotoSansTagalog_Regular_otf_start:
        [libc::c_uchar; 5500];
    pub static _binary_resources_fonts_noto_NotoSansTagbanwa_Regular_otf_start:
        [libc::c_uchar; 5356];
    pub static _binary_resources_fonts_noto_NotoSansTaiLe_Regular_otf_start: [libc::c_uchar; 8616];
    pub static _binary_resources_fonts_noto_NotoSansTaiTham_Regular_otf_start:
        [libc::c_uchar; 76880];
    pub static _binary_resources_fonts_noto_NotoSansTaiViet_Regular_otf_start:
        [libc::c_uchar; 12280];
    pub static _binary_resources_fonts_noto_NotoSansTakri_Regular_otf_start: [libc::c_uchar; 17864];
    pub static _binary_resources_fonts_noto_NotoSansTangsa_Regular_otf_start:
        [libc::c_uchar; 16908];
    pub static _binary_resources_fonts_noto_NotoSansThaana_Regular_otf_start:
        [libc::c_uchar; 12392];
    pub static _binary_resources_fonts_noto_NotoSansTifinagh_Regular_otf_start:
        [libc::c_uchar; 24776];
    pub static _binary_resources_fonts_noto_NotoSansTirhuta_Regular_otf_start:
        [libc::c_uchar; 52432];
    pub static _binary_resources_fonts_noto_NotoSansUgaritic_Regular_otf_start:
        [libc::c_uchar; 5048];
    pub static _binary_resources_fonts_noto_NotoSansVai_Regular_otf_start: [libc::c_uchar; 24088];
    pub static _binary_resources_fonts_noto_NotoSansWancho_Regular_otf_start:
        [libc::c_uchar; 15140];
    pub static _binary_resources_fonts_noto_NotoSansWarangCiti_Regular_otf_start:
        [libc::c_uchar; 23484];
    pub static _binary_resources_fonts_noto_NotoSansYi_Regular_otf_start: [libc::c_uchar; 92164];
    pub static _binary_resources_fonts_noto_NotoSansZanabazarSquare_Regular_otf_start:
        [libc::c_uchar; 13804];
    pub static _binary_resources_fonts_noto_NotoSerif_Regular_otf_start: [libc::c_uchar; 289412];
    pub static _binary_resources_fonts_noto_NotoSerifAhom_Regular_otf_start: [libc::c_uchar; 14516];
    pub static _binary_resources_fonts_noto_NotoSerifArmenian_Regular_otf_start:
        [libc::c_uchar; 14160];
    pub static _binary_resources_fonts_noto_NotoSerifBalinese_Regular_otf_start:
        [libc::c_uchar; 32348];
    pub static _binary_resources_fonts_noto_NotoSerifBengali_Regular_otf_start:
        [libc::c_uchar; 101332];
    pub static _binary_resources_fonts_noto_NotoSerifDevanagari_Regular_otf_start:
        [libc::c_uchar; 169744];
    pub static _binary_resources_fonts_noto_NotoSerifDivesAkuru_Regular_otf_start:
        [libc::c_uchar; 27972];
    pub static _binary_resources_fonts_noto_NotoSerifDogra_Regular_otf_start:
        [libc::c_uchar; 19944];
    pub static _binary_resources_fonts_noto_NotoSerifEthiopic_Regular_otf_start:
        [libc::c_uchar; 113328];
    pub static _binary_resources_fonts_noto_NotoSerifGeorgian_Regular_otf_start:
        [libc::c_uchar; 31988];
    pub static _binary_resources_fonts_noto_NotoSerifGrantha_Regular_otf_start:
        [libc::c_uchar; 368396];
    pub static _binary_resources_fonts_noto_NotoSerifGujarati_Regular_otf_start:
        [libc::c_uchar; 64848];
    pub static _binary_resources_fonts_noto_NotoSerifGurmukhi_Regular_otf_start:
        [libc::c_uchar; 26992];
    pub static _binary_resources_fonts_noto_NotoSerifHebrew_Regular_otf_start:
        [libc::c_uchar; 15320];
    pub static _binary_resources_fonts_noto_NotoSerifKannada_Regular_otf_start:
        [libc::c_uchar; 89032];
    pub static _binary_resources_fonts_noto_NotoSerifKhitanSmallScript_Regular_otf_start:
        [libc::c_uchar; 508920];
    pub static _binary_resources_fonts_noto_NotoSerifKhmer_Regular_otf_start:
        [libc::c_uchar; 40436];
    pub static _binary_resources_fonts_noto_NotoSerifKhojki_Regular_otf_start:
        [libc::c_uchar; 60112];
    pub static _binary_resources_fonts_noto_NotoSerifLao_Regular_otf_start: [libc::c_uchar; 16196];
    pub static _binary_resources_fonts_noto_NotoSerifMakasar_Regular_otf_start:
        [libc::c_uchar; 5864];
    pub static _binary_resources_fonts_noto_NotoSerifMalayalam_Regular_otf_start:
        [libc::c_uchar; 45668];
    pub static _binary_resources_fonts_noto_NotoSerifMyanmar_Regular_otf_start:
        [libc::c_uchar; 127564];
    pub static _binary_resources_fonts_noto_NotoSerifNyiakengPuachueHmong_Regular_otf_start:
        [libc::c_uchar; 12208];
    pub static _binary_resources_fonts_noto_NotoSerifOldUyghur_Regular_otf_start:
        [libc::c_uchar; 15620];
    pub static _binary_resources_fonts_noto_NotoSerifOriya_Regular_otf_start:
        [libc::c_uchar; 105824];
    pub static _binary_resources_fonts_noto_NotoSerifSinhala_Regular_otf_start:
        [libc::c_uchar; 74924];
    pub static _binary_resources_fonts_noto_NotoSerifTamil_Regular_otf_start:
        [libc::c_uchar; 33752];
    pub static _binary_resources_fonts_noto_NotoSerifTelugu_Regular_otf_start:
        [libc::c_uchar; 82032];
    pub static _binary_resources_fonts_noto_NotoSerifThai_Regular_otf_start: [libc::c_uchar; 17556];
    pub static _binary_resources_fonts_noto_NotoSerifTibetan_Regular_otf_start:
        [libc::c_uchar; 334156];
    pub static _binary_resources_fonts_noto_NotoSerifToto_Regular_otf_start: [libc::c_uchar; 5732];
    pub static _binary_resources_fonts_noto_NotoSerifVithkuqi_Regular_otf_start:
        [libc::c_uchar; 42508];
    pub static _binary_resources_fonts_noto_NotoSerifYezidi_Regular_otf_start:
        [libc::c_uchar; 8664];
}

// ===========================================================================
// Font Family and Discovery Utilities
// ===========================================================================

pub const SLIDER_VALUE: Style = MD_SIZE;

pub struct FontFamily {
    pub regular: Font,
    pub italic: Font,
    pub bold: Font,
    pub bold_italic: Font,
}

pub fn family_names<P: AsRef<Path>>(search_path: P) -> Result<BTreeSet<String>, Error> {
    if !search_path.as_ref().exists() {
        return Err(format_err!("the search path doesn't exist"));
    }

    let opener = FontOpener::new()?;
    let glob = Glob::new("**/*.[ot]tf")?.compile_matcher();

    let mut families = BTreeSet::new();

    for entry in WalkDir::new(search_path.as_ref())
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| !e.is_hidden())
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !glob.is_match(path) {
            continue;
        }
        if let Ok(font) = opener
            .open(path)
            .map_err(|e| log_error!("Can't open '{}': {:#}.", path.display(), e))
        {
            if let Some(family_name) = font.family_name() {
                families.insert(family_name.to_string());
            } else {
                log_warn!("Can't get the family name of '{}'.", path.display());
            }
        }
    }

    Ok(families)
}

impl FontFamily {
    pub fn from_name<P: AsRef<Path>>(
        family_name: &str,
        search_path: P,
    ) -> Result<FontFamily, Error> {
        let opener = FontOpener::new()?;
        let glob = Glob::new("**/*.[ot]tf")?.compile_matcher();
        let mut styles = FxHashMap::default();

        for entry in WalkDir::new(search_path.as_ref())
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| !e.is_hidden())
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !glob.is_match(path) {
                continue;
            }
            if let Ok(font) = opener
                .open(path)
                .map_err(|e| log_error!("Can't open '{}': {:#}.", path.display(), e))
            {
                if font.family_name().as_deref() == Some(&family_name) {
                    styles.insert(
                        font.style_name()
                            .map(String::from)
                            .unwrap_or_else(|| "Regular".to_string()),
                        path.to_path_buf(),
                    );
                }
            }
        }

        let regular_path = if styles.len() == 1 {
            styles
                .values()
                .next()
                .ok_or_else(|| format_err!("styles is empty"))?
        } else {
            styles
                .get("Regular")
                .or_else(|| styles.get("Roman"))
                .or_else(|| styles.get("Book"))
                .ok_or_else(|| format_err!("can't find regular style"))?
        };
        let italic_path = styles
            .get("Italic")
            .or_else(|| styles.get("Book Italic"))
            .or_else(|| styles.get("Regular Italic"))
            .unwrap_or(regular_path);
        let bold_path = styles
            .get("Bold")
            .or_else(|| styles.get("Semibold"))
            .or_else(|| styles.get("SemiBold"))
            .or_else(|| styles.get("Medium"))
            .unwrap_or(regular_path);
        let bold_italic_path = styles
            .get("Bold Italic")
            .or_else(|| styles.get("SemiBold Italic"))
            .or_else(|| styles.get("Medium Italic"))
            .unwrap_or(italic_path);
        Ok(FontFamily {
            regular: opener.open(regular_path)?,
            italic: opener.open(italic_path)?,
            bold: opener.open(bold_path)?,
            bold_italic: opener.open(bold_italic_path)?,
        })
    }
}

pub struct Fonts {
    pub sans_serif: FontFamily,
    pub serif: FontFamily,
    pub monospace: FontFamily,
    pub keyboard: Font,
    pub display: Font,
}

impl Fonts {
    pub fn load() -> Result<Fonts, Error> {
        let opener = FontOpener::new()?;
        let mut fonts = Fonts {
            sans_serif: FontFamily {
                regular: opener.open("fonts/NotoSans-Regular.ttf")?,
                italic: opener.open("fonts/NotoSans-Italic.ttf")?,
                bold: opener.open("fonts/NotoSans-Bold.ttf")?,
                bold_italic: opener.open("fonts/NotoSans-BoldItalic.ttf")?,
            },
            serif: FontFamily {
                regular: opener.open("fonts/NotoSerif-Regular.ttf")?,
                italic: opener.open("fonts/NotoSerif-Italic.ttf")?,
                bold: opener.open("fonts/NotoSerif-Bold.ttf")?,
                bold_italic: opener.open("fonts/NotoSerif-BoldItalic.ttf")?,
            },
            monospace: FontFamily {
                regular: opener.open("fonts/SourceCodeVariable-Roman.otf")?,
                italic: opener.open("fonts/SourceCodeVariable-Italic.otf")?,
                bold: opener.open("fonts/SourceCodeVariable-Roman.otf")?,
                bold_italic: opener.open("fonts/SourceCodeVariable-Italic.otf")?,
            },
            keyboard: opener.open("fonts/VarelaRound-Regular.ttf")?,
            display: opener.open("fonts/Cormorant-Regular.ttf")?,
        };
        fonts.monospace.bold.set_variations(&["wght=600"]);
        fonts.monospace.bold_italic.set_variations(&["wght=600"]);
        Ok(fonts)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Variant: u8 {
        const REGULAR = 0;
        const ITALIC = 1;
        const BOLD = 2;
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Family {
    SansSerif,
    Serif,
    Monospace,
    Keyboard,
    Display,
}

pub struct Style {
    family: Family,
    variant: Variant,
    pub size: u32,
}

pub fn font_from_variant(family: &mut FontFamily, variant: Variant) -> &mut Font {
    if variant.contains(Variant::ITALIC | Variant::BOLD) {
        &mut family.bold_italic
    } else if variant.contains(Variant::ITALIC) {
        &mut family.italic
    } else if variant.contains(Variant::BOLD) {
        &mut family.bold
    } else {
        &mut family.regular
    }
}

pub fn font_from_style<'a>(fonts: &'a mut Fonts, style: &Style, dpi: u16) -> &'a mut Font {
    let font = match style.family {
        Family::SansSerif => {
            let family = &mut fonts.sans_serif;
            font_from_variant(family, style.variant)
        }
        Family::Serif => {
            let family = &mut fonts.serif;
            font_from_variant(family, style.variant)
        }
        Family::Monospace => {
            let family = &mut fonts.monospace;
            font_from_variant(family, style.variant)
        }
        Family::Keyboard => &mut fonts.keyboard,
        Family::Display => &mut fonts.display,
    };
    font.set_size(style.size, dpi);
    font
}
