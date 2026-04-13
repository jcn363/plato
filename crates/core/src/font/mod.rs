//! Font Rendering Subsystem
//!
//! This module provides font handling for Plato, wrapping FreeType and HarfBuzz libraries.
//!
//! ## Architecture
//!
//! - **freetype_sys**: Low-level FreeType FFI bindings
//! - **harfbuzz_sys**: Low-level HarfBuzz FFI bindings
//! - **freetype.rs** (implied): Safe FreeType wrappers with RAII
//! - **harfbuzz.rs** (implied): Safe HarfBuzz wrappers with RAII
//!
//! The subsystem handles:
//! - Font discovery and loading from filesystem
//! - Embedded font resources
//! - Glyph rasterization via FreeType
//! - Text shaping (glyph positioning) via HarfBuzz
//! - Missing glyph handling

pub mod face;
pub mod freetype;
mod freetype_error;
mod freetype_sys;
pub mod harfbuzz;
mod harfbuzz_sys;
pub mod library;
pub mod rasterizer;
pub mod shaper;
mod types;

// Public re-exports - types now use safe wrappers
pub use self::face::Font;
pub use self::freetype_error::FreetypeError;
pub use self::freetype_sys::FtError;
pub use self::library::FontOpener;
pub use self::types::{GlyphPlan, RenderPlan};

// ===========================================================================
// Imports and Re-exports
// ===========================================================================

use self::freetype_sys::*;
use self::harfbuzz_sys::*;
use self::library::FontLibrary;

use crate::color::Color;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::Framebuffer;
use crate::geom::{Point, Vec2};
use crate::helpers::IsHidden;
use crate::{log_error, log_warn};
use anyhow::{format_err, Error};
use bitflags::bitflags;
use globset::Glob;
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use std::collections::BTreeSet;
use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::rc::Rc;
use std::slice;
use std::str;
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

lazy_static! {
    pub static ref MD_TITLE: Style = {
        // Compute the ratio between the physical width of the
        // current device and that of the Aura ONE.
        let ratio = (CURRENT_DEVICE.dims.0 as f32 * 300.0) /
                    (CURRENT_DEVICE.dpi as f32 * 1404.0);
        let size = ((FONT_SIZES[2] as f32 * ratio) as u32).clamp(FONT_SIZES[1],
                                                                 FONT_SIZES[2]);
        Style {
            family: Family::Serif,
            variant: Variant::ITALIC,
            size,
        }
    };
}

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

// ===========================================================================
// Script-to-Font Mapping and Unicode Script Detection
// ===========================================================================

#[inline]
unsafe fn font_data_from_script(script: HbScript) -> &'static [libc::c_uchar] {
    // Extracted from mupdf in source/fitz/noto.c
    #[cfg(any(not(target_os = "linux"), target_arch = "arm"))]
    match script {
        HB_SCRIPT_HANGUL | HB_SCRIPT_HIRAGANA | HB_SCRIPT_KATAKANA | HB_SCRIPT_BOPOMOFO
        | HB_SCRIPT_HAN => &_binary_DroidSansFallback_ttf,

        HB_SCRIPT_ARABIC => &_binary_NotoNaskhArabic_Regular_otf,
        HB_SCRIPT_SYRIAC => &_binary_NotoSansSyriac_Regular_otf,
        HB_SCRIPT_MEROITIC_CURSIVE | HB_SCRIPT_MEROITIC_HIEROGLYPHS => {
            &_binary_NotoSansMeroitic_Regular_otf
        }

        HB_SCRIPT_ADLAM => &_binary_NotoSansAdlam_Regular_otf,
        HB_SCRIPT_AHOM => &_binary_NotoSerifAhom_Regular_otf,
        HB_SCRIPT_ANATOLIAN_HIEROGLYPHS => &_binary_NotoSansAnatolianHieroglyphs_Regular_otf,
        HB_SCRIPT_ARMENIAN => &_binary_NotoSerifArmenian_Regular_otf,
        HB_SCRIPT_AVESTAN => &_binary_NotoSansAvestan_Regular_otf,
        HB_SCRIPT_BALINESE => &_binary_NotoSerifBalinese_Regular_otf,
        HB_SCRIPT_BAMUM => &_binary_NotoSansBamum_Regular_otf,
        HB_SCRIPT_BASSA_VAH => &_binary_NotoSansBassaVah_Regular_otf,
        HB_SCRIPT_BATAK => &_binary_NotoSansBatak_Regular_otf,
        HB_SCRIPT_BENGALI => &_binary_NotoSerifBengali_Regular_otf,
        HB_SCRIPT_BHAIKSUKI => &_binary_NotoSansBhaiksuki_Regular_otf,
        HB_SCRIPT_BRAHMI => &_binary_NotoSansBrahmi_Regular_otf,
        HB_SCRIPT_BUGINESE => &_binary_NotoSansBuginese_Regular_otf,
        HB_SCRIPT_BUHID => &_binary_NotoSansBuhid_Regular_otf,
        HB_SCRIPT_CANADIAN_SYLLABICS => &_binary_NotoSansCanadianAboriginal_Regular_otf,
        HB_SCRIPT_CARIAN => &_binary_NotoSansCarian_Regular_otf,
        HB_SCRIPT_CAUCASIAN_ALBANIAN => &_binary_NotoSansCaucasianAlbanian_Regular_otf,
        HB_SCRIPT_CHAKMA => &_binary_NotoSansChakma_Regular_otf,
        HB_SCRIPT_CHAM => &_binary_NotoSansCham_Regular_otf,
        HB_SCRIPT_CHEROKEE => &_binary_NotoSansCherokee_Regular_otf,
        HB_SCRIPT_CHORASMIAN => &_binary_NotoSansChorasmian_Regular_otf,
        HB_SCRIPT_COPTIC => &_binary_NotoSansCoptic_Regular_otf,
        HB_SCRIPT_CUNEIFORM => &_binary_NotoSansCuneiform_Regular_otf,
        HB_SCRIPT_CYPRIOT => &_binary_NotoSansCypriot_Regular_otf,
        HB_SCRIPT_CYPRO_MINOAN => &_binary_NotoSansCyproMinoan_Regular_otf,
        HB_SCRIPT_DESERET => &_binary_NotoSansDeseret_Regular_otf,
        HB_SCRIPT_DEVANAGARI => &_binary_NotoSerifDevanagari_Regular_otf,
        HB_SCRIPT_DIVES_AKURU => &_binary_NotoSerifDivesAkuru_Regular_otf,
        HB_SCRIPT_DOGRA => &_binary_NotoSerifDogra_Regular_otf,
        HB_SCRIPT_DUPLOYAN => &_binary_NotoSansDuployan_Regular_otf,
        HB_SCRIPT_EGYPTIAN_HIEROGLYPHS => &_binary_NotoSansEgyptianHieroglyphs_Regular_otf,
        HB_SCRIPT_ELBASAN => &_binary_NotoSansElbasan_Regular_otf,
        HB_SCRIPT_ELYMAIC => &_binary_NotoSansElymaic_Regular_otf,
        HB_SCRIPT_ETHIOPIC => &_binary_NotoSerifEthiopic_Regular_otf,
        HB_SCRIPT_GEORGIAN => &_binary_NotoSerifGeorgian_Regular_otf,
        HB_SCRIPT_GLAGOLITIC => &_binary_NotoSansGlagolitic_Regular_otf,
        HB_SCRIPT_GOTHIC => &_binary_NotoSansGothic_Regular_otf,
        HB_SCRIPT_GRANTHA => &_binary_NotoSerifGrantha_Regular_otf,
        HB_SCRIPT_GUJARATI => &_binary_NotoSerifGujarati_Regular_otf,
        HB_SCRIPT_GUNJALA_GONDI => &_binary_NotoSansGunjalaGondi_Regular_otf,
        HB_SCRIPT_GURMUKHI => &_binary_NotoSerifGurmukhi_Regular_otf,
        HB_SCRIPT_HANIFI_ROHINGYA => &_binary_NotoSansHanifiRohingya_Regular_otf,
        HB_SCRIPT_HANUNOO => &_binary_NotoSansHanunoo_Regular_otf,
        HB_SCRIPT_HATRAN => &_binary_NotoSansHatran_Regular_otf,
        HB_SCRIPT_HEBREW => &_binary_NotoSerifHebrew_Regular_otf,
        HB_SCRIPT_IMPERIAL_ARAMAIC => &_binary_NotoSansImperialAramaic_Regular_otf,
        HB_SCRIPT_INSCRIPTIONAL_PAHLAVI => &_binary_NotoSansInscriptionalPahlavi_Regular_otf,
        HB_SCRIPT_INSCRIPTIONAL_PARTHIAN => &_binary_NotoSansInscriptionalParthian_Regular_otf,
        HB_SCRIPT_JAVANESE => &_binary_NotoSansJavanese_Regular_otf,
        HB_SCRIPT_KAITHI => &_binary_NotoSansKaithi_Regular_otf,
        HB_SCRIPT_KANNADA => &_binary_NotoSerifKannada_Regular_otf,
        HB_SCRIPT_KAWI => &_binary_NotoSansKawi_Regular_otf,
        HB_SCRIPT_KAYAH_LI => &_binary_NotoSansKayahLi_Regular_otf,
        HB_SCRIPT_KHAROSHTHI => &_binary_NotoSansKharoshthi_Regular_otf,
        HB_SCRIPT_KHITAN_SMALL_SCRIPT => &_binary_NotoSerifKhitanSmallScript_Regular_otf,
        HB_SCRIPT_KHMER => &_binary_NotoSerifKhmer_Regular_otf,
        HB_SCRIPT_KHOJKI => &_binary_NotoSerifKhojki_Regular_otf,
        HB_SCRIPT_KHUDAWADI => &_binary_NotoSansKhudawadi_Regular_otf,
        HB_SCRIPT_LAO => &_binary_NotoSerifLao_Regular_otf,
        HB_SCRIPT_LEPCHA => &_binary_NotoSansLepcha_Regular_otf,
        HB_SCRIPT_LIMBU => &_binary_NotoSansLimbu_Regular_otf,
        HB_SCRIPT_LINEAR_A => &_binary_NotoSansLinearA_Regular_otf,
        HB_SCRIPT_LINEAR_B => &_binary_NotoSansLinearB_Regular_otf,
        HB_SCRIPT_LISU => &_binary_NotoSansLisu_Regular_otf,
        HB_SCRIPT_LYCIAN => &_binary_NotoSansLycian_Regular_otf,
        HB_SCRIPT_LYDIAN => &_binary_NotoSansLydian_Regular_otf,
        HB_SCRIPT_MAHAJANI => &_binary_NotoSansMahajani_Regular_otf,
        HB_SCRIPT_MAKASAR => &_binary_NotoSerifMakasar_Regular_otf,
        HB_SCRIPT_MALAYALAM => &_binary_NotoSerifMalayalam_Regular_otf,
        HB_SCRIPT_MANDAIC => &_binary_NotoSansMandaic_Regular_otf,
        HB_SCRIPT_MANICHAEAN => &_binary_NotoSansManichaean_Regular_otf,
        HB_SCRIPT_MARCHEN => &_binary_NotoSansMarchen_Regular_otf,
        HB_SCRIPT_MASARAM_GONDI => &_binary_NotoSansMasaramGondi_Regular_otf,
        HB_SCRIPT_MEDEFAIDRIN => &_binary_NotoSansMedefaidrin_Regular_otf,
        HB_SCRIPT_MEETEI_MAYEK => &_binary_NotoSansMeeteiMayek_Regular_otf,
        HB_SCRIPT_MENDE_KIKAKUI => &_binary_NotoSansMendeKikakui_Regular_otf,
        HB_SCRIPT_MIAO => &_binary_NotoSansMiao_Regular_otf,
        HB_SCRIPT_MODI => &_binary_NotoSansModi_Regular_otf,
        HB_SCRIPT_MONGOLIAN => &_binary_NotoSansMongolian_Regular_otf,
        HB_SCRIPT_MRO => &_binary_NotoSansMro_Regular_otf,
        HB_SCRIPT_MULTANI => &_binary_NotoSansMultani_Regular_otf,
        HB_SCRIPT_MYANMAR => &_binary_NotoSerifMyanmar_Regular_otf,
        HB_SCRIPT_NABATAEAN => &_binary_NotoSansNabataean_Regular_otf,
        HB_SCRIPT_NAG_MUNDARI => &_binary_NotoSansNagMundari_Regular_otf,
        HB_SCRIPT_NANDINAGARI => &_binary_NotoSansNandinagari_Regular_otf,
        HB_SCRIPT_NEWA => &_binary_NotoSansNewa_Regular_otf,
        HB_SCRIPT_NEW_TAI_LUE => &_binary_NotoSansNewTaiLue_Regular_otf,
        HB_SCRIPT_NKO => &_binary_NotoSansNKo_Regular_otf,
        HB_SCRIPT_NUSHU => &_binary_NotoSansNushu_Regular_otf,
        HB_SCRIPT_NYIAKENG_PUACHUE_HMONG => &_binary_NotoSerifNyiakengPuachueHmong_Regular_otf,
        HB_SCRIPT_OGHAM => &_binary_NotoSansOgham_Regular_otf,
        HB_SCRIPT_OLD_HUNGARIAN => &_binary_NotoSansOldHungarian_Regular_otf,
        HB_SCRIPT_OLD_ITALIC => &_binary_NotoSansOldItalic_Regular_otf,
        HB_SCRIPT_OLD_NORTH_ARABIAN => &_binary_NotoSansOldNorthArabian_Regular_otf,
        HB_SCRIPT_OLD_PERMIC => &_binary_NotoSansOldPermic_Regular_otf,
        HB_SCRIPT_OLD_PERSIAN => &_binary_NotoSansOldPersian_Regular_otf,
        HB_SCRIPT_OLD_SOGDIAN => &_binary_NotoSansOldSogdian_Regular_otf,
        HB_SCRIPT_OLD_SOUTH_ARABIAN => &_binary_NotoSansOldSouthArabian_Regular_otf,
        HB_SCRIPT_OLD_TURKIC => &_binary_NotoSansOldTurkic_Regular_otf,
        HB_SCRIPT_OLD_UYGHUR => &_binary_NotoSerifOldUyghur_Regular_otf,
        HB_SCRIPT_OL_CHIKI => &_binary_NotoSansOlChiki_Regular_otf,
        HB_SCRIPT_ORIYA => &_binary_NotoSerifOriya_Regular_otf,
        HB_SCRIPT_OSAGE => &_binary_NotoSansOsage_Regular_otf,
        HB_SCRIPT_OSMANYA => &_binary_NotoSansOsmanya_Regular_otf,
        HB_SCRIPT_PAHAWH_HMONG => &_binary_NotoSansPahawhHmong_Regular_otf,
        HB_SCRIPT_PALMYRENE => &_binary_NotoSansPalmyrene_Regular_otf,
        HB_SCRIPT_PAU_CIN_HAU => &_binary_NotoSansPauCinHau_Regular_otf,
        HB_SCRIPT_PHAGS_PA => &_binary_NotoSansPhagsPa_Regular_otf,
        HB_SCRIPT_PHOENICIAN => &_binary_NotoSansPhoenician_Regular_otf,
        HB_SCRIPT_PSALTER_PAHLAVI => &_binary_NotoSansPsalterPahlavi_Regular_otf,
        HB_SCRIPT_REJANG => &_binary_NotoSansRejang_Regular_otf,
        HB_SCRIPT_RUNIC => &_binary_NotoSansRunic_Regular_otf,
        HB_SCRIPT_SAMARITAN => &_binary_NotoSansSamaritan_Regular_otf,
        HB_SCRIPT_SAURASHTRA => &_binary_NotoSansSaurashtra_Regular_otf,
        HB_SCRIPT_SHARADA => &_binary_NotoSansSharada_Regular_otf,
        HB_SCRIPT_SHAVIAN => &_binary_NotoSansShavian_Regular_otf,
        HB_SCRIPT_SIDDHAM => &_binary_NotoSansSiddham_Regular_otf,
        HB_SCRIPT_SIGNWRITING => &_binary_NotoSansSignWriting_Regular_otf,
        HB_SCRIPT_SINHALA => &_binary_NotoSerifSinhala_Regular_otf,
        HB_SCRIPT_SOGDIAN => &_binary_NotoSansSogdian_Regular_otf,
        HB_SCRIPT_SORA_SOMPENG => &_binary_NotoSansSoraSompeng_Regular_otf,
        HB_SCRIPT_SOYOMBO => &_binary_NotoSansSoyombo_Regular_otf,
        HB_SCRIPT_SUNDANESE => &_binary_NotoSansSundanese_Regular_otf,
        HB_SCRIPT_SYLOTI_NAGRI => &_binary_NotoSansSylotiNagri_Regular_otf,
        HB_SCRIPT_TAGALOG => &_binary_NotoSansTagalog_Regular_otf,
        HB_SCRIPT_TAGBANWA => &_binary_NotoSansTagbanwa_Regular_otf,
        HB_SCRIPT_TAI_LE => &_binary_NotoSansTaiLe_Regular_otf,
        HB_SCRIPT_TAI_THAM => &_binary_NotoSansTaiTham_Regular_otf,
        HB_SCRIPT_TAI_VIET => &_binary_NotoSansTaiViet_Regular_otf,
        HB_SCRIPT_TAKRI => &_binary_NotoSansTakri_Regular_otf,
        HB_SCRIPT_TAMIL => &_binary_NotoSerifTamil_Regular_otf,
        HB_SCRIPT_TANGSA => &_binary_NotoSansTangsa_Regular_otf,
        HB_SCRIPT_TELUGU => &_binary_NotoSerifTelugu_Regular_otf,
        HB_SCRIPT_THAANA => &_binary_NotoSansThaana_Regular_otf,
        HB_SCRIPT_THAI => &_binary_NotoSerifThai_Regular_otf,
        HB_SCRIPT_TIBETAN => &_binary_NotoSerifTibetan_Regular_otf,
        HB_SCRIPT_TIFINAGH => &_binary_NotoSansTifinagh_Regular_otf,
        HB_SCRIPT_TIRHUTA => &_binary_NotoSansTirhuta_Regular_otf,
        HB_SCRIPT_TOTO => &_binary_NotoSerifToto_Regular_otf,
        HB_SCRIPT_UGARITIC => &_binary_NotoSansUgaritic_Regular_otf,
        HB_SCRIPT_VAI => &_binary_NotoSansVai_Regular_otf,
        HB_SCRIPT_VITHKUQI => &_binary_NotoSerifVithkuqi_Regular_otf,
        HB_SCRIPT_WANCHO => &_binary_NotoSansWancho_Regular_otf,
        HB_SCRIPT_WARANG_CITI => &_binary_NotoSansWarangCiti_Regular_otf,
        HB_SCRIPT_YEZIDI => &_binary_NotoSerifYezidi_Regular_otf,
        HB_SCRIPT_YI => &_binary_NotoSansYi_Regular_otf,
        HB_SCRIPT_ZANABAZAR_SQUARE => &_binary_NotoSansZanabazarSquare_Regular_otf,

        HB_SYMBOL_MATHS => &_binary_NotoSansMath_Regular_otf,
        HB_SYMBOL_MUSIC => &_binary_NotoMusic_Regular_otf,
        HB_SYMBOL_MISC_ONE => &_binary_NotoSansSymbols_Regular_otf,
        HB_SCRIPT_BRAILLE | HB_SYMBOL_MISC_TWO => &_binary_NotoSansSymbols2_Regular_otf,
        HB_SYMBOL_EMOJI => &_binary_NotoEmoji_Regular_ttf,

        _ => &_binary_DroidSansFallback_ttf,
    }

    #[cfg(all(target_os = "linux", not(target_arch = "arm")))]
    match script {
        HB_SCRIPT_HANGUL | HB_SCRIPT_HIRAGANA | HB_SCRIPT_KATAKANA | HB_SCRIPT_BOPOMOFO
        | HB_SCRIPT_HAN => &_binary_resources_fonts_droid_DroidSansFallback_ttf_start,

        HB_SCRIPT_ARABIC => &_binary_resources_fonts_noto_NotoNaskhArabic_Regular_otf_start,
        HB_SCRIPT_SYRIAC => &_binary_resources_fonts_noto_NotoSansSyriac_Regular_otf_start,
        HB_SCRIPT_MEROITIC_CURSIVE | HB_SCRIPT_MEROITIC_HIEROGLYPHS => {
            &_binary_resources_fonts_noto_NotoSansMeroitic_Regular_otf_start
        }

        HB_SCRIPT_ADLAM => &_binary_resources_fonts_noto_NotoSansAdlam_Regular_otf_start,
        HB_SCRIPT_AHOM => &_binary_resources_fonts_noto_NotoSerifAhom_Regular_otf_start,
        HB_SCRIPT_ANATOLIAN_HIEROGLYPHS => {
            &_binary_resources_fonts_noto_NotoSansAnatolianHieroglyphs_Regular_otf_start
        }
        HB_SCRIPT_ARMENIAN => &_binary_resources_fonts_noto_NotoSerifArmenian_Regular_otf_start,
        HB_SCRIPT_AVESTAN => &_binary_resources_fonts_noto_NotoSansAvestan_Regular_otf_start,
        HB_SCRIPT_BALINESE => &_binary_resources_fonts_noto_NotoSerifBalinese_Regular_otf_start,
        HB_SCRIPT_BAMUM => &_binary_resources_fonts_noto_NotoSansBamum_Regular_otf_start,
        HB_SCRIPT_BASSA_VAH => &_binary_resources_fonts_noto_NotoSansBassaVah_Regular_otf_start,
        HB_SCRIPT_BATAK => &_binary_resources_fonts_noto_NotoSansBatak_Regular_otf_start,
        HB_SCRIPT_BENGALI => &_binary_resources_fonts_noto_NotoSerifBengali_Regular_otf_start,
        HB_SCRIPT_BHAIKSUKI => &_binary_resources_fonts_noto_NotoSansBhaiksuki_Regular_otf_start,
        HB_SCRIPT_BRAHMI => &_binary_resources_fonts_noto_NotoSansBrahmi_Regular_otf_start,
        HB_SCRIPT_BUGINESE => &_binary_resources_fonts_noto_NotoSansBuginese_Regular_otf_start,
        HB_SCRIPT_BUHID => &_binary_resources_fonts_noto_NotoSansBuhid_Regular_otf_start,
        HB_SCRIPT_CANADIAN_SYLLABICS => {
            &_binary_resources_fonts_noto_NotoSansCanadianAboriginal_Regular_otf_start
        }
        HB_SCRIPT_CARIAN => &_binary_resources_fonts_noto_NotoSansCarian_Regular_otf_start,
        HB_SCRIPT_CAUCASIAN_ALBANIAN => {
            &_binary_resources_fonts_noto_NotoSansCaucasianAlbanian_Regular_otf_start
        }
        HB_SCRIPT_CHAKMA => &_binary_resources_fonts_noto_NotoSansChakma_Regular_otf_start,
        HB_SCRIPT_CHAM => &_binary_resources_fonts_noto_NotoSansCham_Regular_otf_start,
        HB_SCRIPT_CHEROKEE => &_binary_resources_fonts_noto_NotoSansCherokee_Regular_otf_start,
        HB_SCRIPT_CHORASMIAN => &_binary_resources_fonts_noto_NotoSansChorasmian_Regular_otf_start,
        HB_SCRIPT_COPTIC => &_binary_resources_fonts_noto_NotoSansCoptic_Regular_otf_start,
        HB_SCRIPT_CUNEIFORM => &_binary_resources_fonts_noto_NotoSansCuneiform_Regular_otf_start,
        HB_SCRIPT_CYPRIOT => &_binary_resources_fonts_noto_NotoSansCypriot_Regular_otf_start,
        HB_SCRIPT_CYPRO_MINOAN => {
            &_binary_resources_fonts_noto_NotoSansCyproMinoan_Regular_otf_start
        }
        HB_SCRIPT_DESERET => &_binary_resources_fonts_noto_NotoSansDeseret_Regular_otf_start,
        HB_SCRIPT_DEVANAGARI => &_binary_resources_fonts_noto_NotoSerifDevanagari_Regular_otf_start,
        HB_SCRIPT_DIVES_AKURU => {
            &_binary_resources_fonts_noto_NotoSerifDivesAkuru_Regular_otf_start
        }
        HB_SCRIPT_DOGRA => &_binary_resources_fonts_noto_NotoSerifDogra_Regular_otf_start,
        HB_SCRIPT_DUPLOYAN => &_binary_resources_fonts_noto_NotoSansDuployan_Regular_otf_start,
        HB_SCRIPT_EGYPTIAN_HIEROGLYPHS => {
            &_binary_resources_fonts_noto_NotoSansEgyptianHieroglyphs_Regular_otf_start
        }
        HB_SCRIPT_ELBASAN => &_binary_resources_fonts_noto_NotoSansElbasan_Regular_otf_start,
        HB_SCRIPT_ELYMAIC => &_binary_resources_fonts_noto_NotoSansElymaic_Regular_otf_start,
        HB_SCRIPT_ETHIOPIC => &_binary_resources_fonts_noto_NotoSerifEthiopic_Regular_otf_start,
        HB_SCRIPT_GEORGIAN => &_binary_resources_fonts_noto_NotoSerifGeorgian_Regular_otf_start,
        HB_SCRIPT_GLAGOLITIC => &_binary_resources_fonts_noto_NotoSansGlagolitic_Regular_otf_start,
        HB_SCRIPT_GOTHIC => &_binary_resources_fonts_noto_NotoSansGothic_Regular_otf_start,
        HB_SCRIPT_GRANTHA => &_binary_resources_fonts_noto_NotoSerifGrantha_Regular_otf_start,
        HB_SCRIPT_GUJARATI => &_binary_resources_fonts_noto_NotoSerifGujarati_Regular_otf_start,
        HB_SCRIPT_GUNJALA_GONDI => {
            &_binary_resources_fonts_noto_NotoSansGunjalaGondi_Regular_otf_start
        }
        HB_SCRIPT_GURMUKHI => &_binary_resources_fonts_noto_NotoSerifGurmukhi_Regular_otf_start,
        HB_SCRIPT_HANIFI_ROHINGYA => {
            &_binary_resources_fonts_noto_NotoSansHanifiRohingya_Regular_otf_start
        }
        HB_SCRIPT_HANUNOO => &_binary_resources_fonts_noto_NotoSansHanunoo_Regular_otf_start,
        HB_SCRIPT_HATRAN => &_binary_resources_fonts_noto_NotoSansHatran_Regular_otf_start,
        HB_SCRIPT_HEBREW => &_binary_resources_fonts_noto_NotoSerifHebrew_Regular_otf_start,
        HB_SCRIPT_IMPERIAL_ARAMAIC => {
            &_binary_resources_fonts_noto_NotoSansImperialAramaic_Regular_otf_start
        }
        HB_SCRIPT_INSCRIPTIONAL_PAHLAVI => {
            &_binary_resources_fonts_noto_NotoSansInscriptionalPahlavi_Regular_otf_start
        }
        HB_SCRIPT_INSCRIPTIONAL_PARTHIAN => {
            &_binary_resources_fonts_noto_NotoSansInscriptionalParthian_Regular_otf_start
        }
        HB_SCRIPT_JAVANESE => &_binary_resources_fonts_noto_NotoSansJavanese_Regular_otf_start,
        HB_SCRIPT_KAITHI => &_binary_resources_fonts_noto_NotoSansKaithi_Regular_otf_start,
        HB_SCRIPT_KANNADA => &_binary_resources_fonts_noto_NotoSerifKannada_Regular_otf_start,
        HB_SCRIPT_KAWI => &_binary_resources_fonts_noto_NotoSansKawi_Regular_otf_start,
        HB_SCRIPT_KAYAH_LI => &_binary_resources_fonts_noto_NotoSansKayahLi_Regular_otf_start,
        HB_SCRIPT_KHAROSHTHI => &_binary_resources_fonts_noto_NotoSansKharoshthi_Regular_otf_start,
        HB_SCRIPT_KHITAN_SMALL_SCRIPT => {
            &_binary_resources_fonts_noto_NotoSerifKhitanSmallScript_Regular_otf_start
        }
        HB_SCRIPT_KHMER => &_binary_resources_fonts_noto_NotoSerifKhmer_Regular_otf_start,
        HB_SCRIPT_KHOJKI => &_binary_resources_fonts_noto_NotoSerifKhojki_Regular_otf_start,
        HB_SCRIPT_KHUDAWADI => &_binary_resources_fonts_noto_NotoSansKhudawadi_Regular_otf_start,
        HB_SCRIPT_LAO => &_binary_resources_fonts_noto_NotoSerifLao_Regular_otf_start,
        HB_SCRIPT_LEPCHA => &_binary_resources_fonts_noto_NotoSansLepcha_Regular_otf_start,
        HB_SCRIPT_LIMBU => &_binary_resources_fonts_noto_NotoSansLimbu_Regular_otf_start,
        HB_SCRIPT_LINEAR_A => &_binary_resources_fonts_noto_NotoSansLinearA_Regular_otf_start,
        HB_SCRIPT_LINEAR_B => &_binary_resources_fonts_noto_NotoSansLinearB_Regular_otf_start,
        HB_SCRIPT_LISU => &_binary_resources_fonts_noto_NotoSansLisu_Regular_otf_start,
        HB_SCRIPT_LYCIAN => &_binary_resources_fonts_noto_NotoSansLycian_Regular_otf_start,
        HB_SCRIPT_LYDIAN => &_binary_resources_fonts_noto_NotoSansLydian_Regular_otf_start,
        HB_SCRIPT_MAHAJANI => &_binary_resources_fonts_noto_NotoSansMahajani_Regular_otf_start,
        HB_SCRIPT_MAKASAR => &_binary_resources_fonts_noto_NotoSerifMakasar_Regular_otf_start,
        HB_SCRIPT_MALAYALAM => &_binary_resources_fonts_noto_NotoSerifMalayalam_Regular_otf_start,
        HB_SCRIPT_MANDAIC => &_binary_resources_fonts_noto_NotoSansMandaic_Regular_otf_start,
        HB_SCRIPT_MANICHAEAN => &_binary_resources_fonts_noto_NotoSansManichaean_Regular_otf_start,
        HB_SCRIPT_MARCHEN => &_binary_resources_fonts_noto_NotoSansMarchen_Regular_otf_start,
        HB_SCRIPT_MASARAM_GONDI => {
            &_binary_resources_fonts_noto_NotoSansMasaramGondi_Regular_otf_start
        }
        HB_SCRIPT_MEDEFAIDRIN => {
            &_binary_resources_fonts_noto_NotoSansMedefaidrin_Regular_otf_start
        }
        HB_SCRIPT_MEETEI_MAYEK => {
            &_binary_resources_fonts_noto_NotoSansMeeteiMayek_Regular_otf_start
        }
        HB_SCRIPT_MENDE_KIKAKUI => {
            &_binary_resources_fonts_noto_NotoSansMendeKikakui_Regular_otf_start
        }
        HB_SCRIPT_MIAO => &_binary_resources_fonts_noto_NotoSansMiao_Regular_otf_start,
        HB_SCRIPT_MODI => &_binary_resources_fonts_noto_NotoSansModi_Regular_otf_start,
        HB_SCRIPT_MONGOLIAN => &_binary_resources_fonts_noto_NotoSansMongolian_Regular_otf_start,
        HB_SCRIPT_MRO => &_binary_resources_fonts_noto_NotoSansMro_Regular_otf_start,
        HB_SCRIPT_MULTANI => &_binary_resources_fonts_noto_NotoSansMultani_Regular_otf_start,
        HB_SCRIPT_MYANMAR => &_binary_resources_fonts_noto_NotoSerifMyanmar_Regular_otf_start,
        HB_SCRIPT_NABATAEAN => &_binary_resources_fonts_noto_NotoSansNabataean_Regular_otf_start,
        HB_SCRIPT_NAG_MUNDARI => &_binary_resources_fonts_noto_NotoSansNagMundari_Regular_otf_start,
        HB_SCRIPT_NANDINAGARI => {
            &_binary_resources_fonts_noto_NotoSansNandinagari_Regular_otf_start
        }
        HB_SCRIPT_NEWA => &_binary_resources_fonts_noto_NotoSansNewa_Regular_otf_start,
        HB_SCRIPT_NEW_TAI_LUE => &_binary_resources_fonts_noto_NotoSansNewTaiLue_Regular_otf_start,
        HB_SCRIPT_NKO => &_binary_resources_fonts_noto_NotoSansNKo_Regular_otf_start,
        HB_SCRIPT_NUSHU => &_binary_resources_fonts_noto_NotoSansNushu_Regular_otf_start,
        HB_SCRIPT_NYIAKENG_PUACHUE_HMONG => {
            &_binary_resources_fonts_noto_NotoSerifNyiakengPuachueHmong_Regular_otf_start
        }
        HB_SCRIPT_OGHAM => &_binary_resources_fonts_noto_NotoSansOgham_Regular_otf_start,
        HB_SCRIPT_OLD_HUNGARIAN => {
            &_binary_resources_fonts_noto_NotoSansOldHungarian_Regular_otf_start
        }
        HB_SCRIPT_OLD_ITALIC => &_binary_resources_fonts_noto_NotoSansOldItalic_Regular_otf_start,
        HB_SCRIPT_OLD_NORTH_ARABIAN => {
            &_binary_resources_fonts_noto_NotoSansOldNorthArabian_Regular_otf_start
        }
        HB_SCRIPT_OLD_PERMIC => &_binary_resources_fonts_noto_NotoSansOldPermic_Regular_otf_start,
        HB_SCRIPT_OLD_PERSIAN => &_binary_resources_fonts_noto_NotoSansOldPersian_Regular_otf_start,
        HB_SCRIPT_OLD_SOGDIAN => &_binary_resources_fonts_noto_NotoSansOldSogdian_Regular_otf_start,
        HB_SCRIPT_OLD_SOUTH_ARABIAN => {
            &_binary_resources_fonts_noto_NotoSansOldSouthArabian_Regular_otf_start
        }
        HB_SCRIPT_OLD_TURKIC => &_binary_resources_fonts_noto_NotoSansOldTurkic_Regular_otf_start,
        HB_SCRIPT_OLD_UYGHUR => &_binary_resources_fonts_noto_NotoSerifOldUyghur_Regular_otf_start,
        HB_SCRIPT_OL_CHIKI => &_binary_resources_fonts_noto_NotoSansOlChiki_Regular_otf_start,
        HB_SCRIPT_ORIYA => &_binary_resources_fonts_noto_NotoSerifOriya_Regular_otf_start,
        HB_SCRIPT_OSAGE => &_binary_resources_fonts_noto_NotoSansOsage_Regular_otf_start,
        HB_SCRIPT_OSMANYA => &_binary_resources_fonts_noto_NotoSansOsmanya_Regular_otf_start,
        HB_SCRIPT_PAHAWH_HMONG => {
            &_binary_resources_fonts_noto_NotoSansPahawhHmong_Regular_otf_start
        }
        HB_SCRIPT_PALMYRENE => &_binary_resources_fonts_noto_NotoSansPalmyrene_Regular_otf_start,
        HB_SCRIPT_PAU_CIN_HAU => &_binary_resources_fonts_noto_NotoSansPauCinHau_Regular_otf_start,
        HB_SCRIPT_PHAGS_PA => &_binary_resources_fonts_noto_NotoSansPhagsPa_Regular_otf_start,
        HB_SCRIPT_PHOENICIAN => &_binary_resources_fonts_noto_NotoSansPhoenician_Regular_otf_start,
        HB_SCRIPT_PSALTER_PAHLAVI => {
            &_binary_resources_fonts_noto_NotoSansPsalterPahlavi_Regular_otf_start
        }
        HB_SCRIPT_REJANG => &_binary_resources_fonts_noto_NotoSansRejang_Regular_otf_start,
        HB_SCRIPT_RUNIC => &_binary_resources_fonts_noto_NotoSansRunic_Regular_otf_start,
        HB_SCRIPT_SAMARITAN => &_binary_resources_fonts_noto_NotoSansSamaritan_Regular_otf_start,
        HB_SCRIPT_SAURASHTRA => &_binary_resources_fonts_noto_NotoSansSaurashtra_Regular_otf_start,
        HB_SCRIPT_SHARADA => &_binary_resources_fonts_noto_NotoSansSharada_Regular_otf_start,
        HB_SCRIPT_SHAVIAN => &_binary_resources_fonts_noto_NotoSansShavian_Regular_otf_start,
        HB_SCRIPT_SIDDHAM => &_binary_resources_fonts_noto_NotoSansSiddham_Regular_otf_start,
        HB_SCRIPT_SIGNWRITING => {
            &_binary_resources_fonts_noto_NotoSansSignWriting_Regular_otf_start
        }
        HB_SCRIPT_SINHALA => &_binary_resources_fonts_noto_NotoSerifSinhala_Regular_otf_start,
        HB_SCRIPT_SOGDIAN => &_binary_resources_fonts_noto_NotoSansSogdian_Regular_otf_start,
        HB_SCRIPT_SORA_SOMPENG => {
            &_binary_resources_fonts_noto_NotoSansSoraSompeng_Regular_otf_start
        }
        HB_SCRIPT_SOYOMBO => &_binary_resources_fonts_noto_NotoSansSoyombo_Regular_otf_start,
        HB_SCRIPT_SUNDANESE => &_binary_resources_fonts_noto_NotoSansSundanese_Regular_otf_start,
        HB_SCRIPT_SYLOTI_NAGRI => {
            &_binary_resources_fonts_noto_NotoSansSylotiNagri_Regular_otf_start
        }
        HB_SCRIPT_TAGALOG => &_binary_resources_fonts_noto_NotoSansTagalog_Regular_otf_start,
        HB_SCRIPT_TAGBANWA => &_binary_resources_fonts_noto_NotoSansTagbanwa_Regular_otf_start,
        HB_SCRIPT_TAI_LE => &_binary_resources_fonts_noto_NotoSansTaiLe_Regular_otf_start,
        HB_SCRIPT_TAI_THAM => &_binary_resources_fonts_noto_NotoSansTaiTham_Regular_otf_start,
        HB_SCRIPT_TAI_VIET => &_binary_resources_fonts_noto_NotoSansTaiViet_Regular_otf_start,
        HB_SCRIPT_TAKRI => &_binary_resources_fonts_noto_NotoSansTakri_Regular_otf_start,
        HB_SCRIPT_TAMIL => &_binary_resources_fonts_noto_NotoSerifTamil_Regular_otf_start,
        HB_SCRIPT_TANGSA => &_binary_resources_fonts_noto_NotoSansTangsa_Regular_otf_start,
        HB_SCRIPT_TELUGU => &_binary_resources_fonts_noto_NotoSerifTelugu_Regular_otf_start,
        HB_SCRIPT_THAANA => &_binary_resources_fonts_noto_NotoSansThaana_Regular_otf_start,
        HB_SCRIPT_THAI => &_binary_resources_fonts_noto_NotoSerifThai_Regular_otf_start,
        HB_SCRIPT_TIBETAN => &_binary_resources_fonts_noto_NotoSerifTibetan_Regular_otf_start,
        HB_SCRIPT_TIFINAGH => &_binary_resources_fonts_noto_NotoSansTifinagh_Regular_otf_start,
        HB_SCRIPT_TIRHUTA => &_binary_resources_fonts_noto_NotoSansTirhuta_Regular_otf_start,
        HB_SCRIPT_TOTO => &_binary_resources_fonts_noto_NotoSerifToto_Regular_otf_start,
        HB_SCRIPT_UGARITIC => &_binary_resources_fonts_noto_NotoSansUgaritic_Regular_otf_start,
        HB_SCRIPT_VAI => &_binary_resources_fonts_noto_NotoSansVai_Regular_otf_start,
        HB_SCRIPT_VITHKUQI => &_binary_resources_fonts_noto_NotoSerifVithkuqi_Regular_otf_start,
        HB_SCRIPT_WANCHO => &_binary_resources_fonts_noto_NotoSansWancho_Regular_otf_start,
        HB_SCRIPT_WARANG_CITI => &_binary_resources_fonts_noto_NotoSansWarangCiti_Regular_otf_start,
        HB_SCRIPT_YEZIDI => &_binary_resources_fonts_noto_NotoSerifYezidi_Regular_otf_start,
        HB_SCRIPT_YI => &_binary_resources_fonts_noto_NotoSansYi_Regular_otf_start,
        HB_SCRIPT_ZANABAZAR_SQUARE => {
            &_binary_resources_fonts_noto_NotoSansZanabazarSquare_Regular_otf_start
        }

        HB_SYMBOL_MATHS => &_binary_resources_fonts_noto_NotoSansMath_Regular_otf_start,
        HB_SYMBOL_MUSIC => &_binary_resources_fonts_noto_NotoMusic_Regular_otf_start,
        HB_SYMBOL_MISC_ONE => &_binary_resources_fonts_noto_NotoSansSymbols_Regular_otf_start,
        HB_SCRIPT_BRAILLE | HB_SYMBOL_MISC_TWO => {
            &_binary_resources_fonts_noto_NotoSansSymbols2_Regular_otf_start
        }
        HB_SYMBOL_EMOJI => &_binary_resources_fonts_noto_NotoEmoji_Regular_ttf_start,

        _ => &_binary_resources_fonts_droid_DroidSansFallback_ttf_start,
    }
}

#[inline]
fn script_from_code(code: u32) -> HbScript {
    // Can be updated when the font changes by comparing the expanded output of
    // `ttfdump -t cmap` for each font.
    match code {
        0x2032..=0x2037
        | 0x2057
        | 0x20D0..=0x20DC
        | 0x20E1
        | 0x20E5..=0x20EF
        | 0x2102
        | 0x210A..=0x210E
        | 0x2110..=0x2112
        | 0x2115
        | 0x2119..=0x211D
        | 0x2124
        | 0x2128
        | 0x212C
        | 0x212D
        | 0x212F..=0x2131
        | 0x2133..=0x2138
        | 0x213C..=0x2140
        | 0x2145..=0x2149
        | 0x2190..=0x21AE
        | 0x21B0..=0x21E5
        | 0x21F1
        | 0x21F2
        | 0x21F4..=0x22FF
        | 0x2308..=0x230B
        | 0x2310
        | 0x2319
        | 0x231C..=0x2321
        | 0x2336..=0x237A
        | 0x237C
        | 0x2395
        | 0x239B..=0x23B6
        | 0x23D0
        | 0x23DC..=0x23E1
        | 0x2474
        | 0x2475
        | 0x25AF
        | 0x25B3
        | 0x25B7
        | 0x25BD
        | 0x25C1
        | 0x25CA
        | 0x25CC
        | 0x25FB
        | 0x266D..=0x266F
        | 0x27C0..=0x27FF
        | 0x2900..=0x2AFF
        | 0x2B0E..=0x2B11
        | 0x2B30..=0x2B4C
        | 0x2BFE
        | 0xFF5B
        | 0xFF5D
        | 0x1D400..=0x1D454
        | 0x1D456..=0x1D49C
        | 0x1D49E
        | 0x1D49F
        | 0x1D4A2
        | 0x1D4A5
        | 0x1D4A6
        | 0x1D4A9..=0x1D4AC
        | 0x1D4AE..=0x1D4B9
        | 0x1D4BB
        | 0x1D4BD..=0x1D4C3
        | 0x1D4C5..=0x1D505
        | 0x1D507..=0x1D50A
        | 0x1D50D..=0x1D514
        | 0x1D516..=0x1D51C
        | 0x1D51E..=0x1D539
        | 0x1D53B..=0x1D53E
        | 0x1D540..=0x1D544
        | 0x1D546
        | 0x1D54A..=0x1D550
        | 0x1D552..=0x1D6A5
        | 0x1D6A8..=0x1D7CB
        | 0x1D7CE..=0x1D7FF
        | 0x1EE00..=0x1EE03
        | 0x1EE05..=0x1EE1F
        | 0x1EE21
        | 0x1EE22
        | 0x1EE24
        | 0x1EE27
        | 0x1EE29..=0x1EE32
        | 0x1EE34..=0x1EE37
        | 0x1EE39
        | 0x1EE3B
        | 0x1EE42
        | 0x1EE47
        | 0x1EE49
        | 0x1EE4B
        | 0x1EE4D..=0x1EE4F
        | 0x1EE51
        | 0x1EE52
        | 0x1EE54
        | 0x1EE57
        | 0x1EE59
        | 0x1EE5B
        | 0x1EE5D
        | 0x1EE5F
        | 0x1EE61
        | 0x1EE62
        | 0x1EE64
        | 0x1EE67..=0x1EE6A
        | 0x1EE6C..=0x1EE72
        | 0x1EE74..=0x1EE77
        | 0x1EE79..=0x1EE7C
        | 0x1EE7E
        | 0x1EE80..=0x1EE89
        | 0x1EE8B..=0x1EE9B
        | 0x1EEA1..=0x1EEA3
        | 0x1EEA5..=0x1EEA9
        | 0x1EEAB..=0x1EEBB
        | 0x1EEF0
        | 0x1EEF1 => HB_SYMBOL_MATHS,

        0x1D000..=0x1D0F5 | 0x1D100..=0x1D126 | 0x1D129..=0x1D1E8 | 0x1D200..=0x1D245 => {
            HB_SYMBOL_MUSIC
        }

        0x20DD..=0x20E0
        | 0x20E2..=0x20E4
        | 0x2160..=0x2183
        | 0x2185..=0x2188
        | 0x218A
        | 0x218B
        | 0x2300..=0x230F
        | 0x2311..=0x2315
        | 0x2317
        | 0x2322
        | 0x2323
        | 0x2329
        | 0x232A
        | 0x232C..=0x2335
        | 0x2380..=0x2394
        | 0x2396..=0x239A
        | 0x23BE..=0x23CD
        | 0x23D1..=0x23DB
        | 0x23E2..=0x23E8
        | 0x2460..=0x24FF
        | 0x260A..=0x260D
        | 0x2613
        | 0x2624..=0x262F
        | 0x2638..=0x263B
        | 0x263D..=0x2653
        | 0x2669..=0x267E
        | 0x2690..=0x269D
        | 0x26A2..=0x26A9
        | 0x26AD..=0x26BC
        | 0x26CE
        | 0x26E2..=0x26FF
        | 0x271D..=0x2721
        | 0x2776..=0x2793
        | 0x1F100..=0x1F10C
        | 0x1F110..=0x1F12F
        | 0x1F130..=0x1F16C
        | 0x1F170..=0x1F190
        | 0x1F19B..=0x1F1AC
        | 0x1F546..=0x1F549
        | 0x1F54F
        | 0x1F610
        | 0x1F700..=0x1F773 => HB_SYMBOL_MISC_ONE,

        0x2022
        | 0x21AF
        | 0x21E6..=0x21F0
        | 0x21F3
        | 0x2316
        | 0x2318
        | 0x231A
        | 0x231B
        | 0x2324..=0x2328
        | 0x232B
        | 0x237B
        | 0x237D..=0x237F
        | 0x23CE
        | 0x23CF
        | 0x23E9
        | 0x23ED..=0x23EF
        | 0x23F1..=0x23FF
        | 0x2400..=0x2426
        | 0x2440..=0x244A
        | 0x25A0..=0x2609
        | 0x260E..=0x2612
        | 0x2614..=0x2623
        | 0x2630..=0x2637
        | 0x263C
        | 0x2654..=0x2668
        | 0x267F..=0x268F
        | 0x269E..=0x26A1
        | 0x26AA..=0x26AC
        | 0x26BD..=0x26CD
        | 0x26CF..=0x26E1
        | 0x2700..=0x2704
        | 0x2706..=0x2709
        | 0x270B..=0x271C
        | 0x2722..=0x2727
        | 0x2729..=0x274B
        | 0x274D
        | 0x274F..=0x2752
        | 0x2756..=0x2775
        | 0x2794
        | 0x2798..=0x27AF
        | 0x27B1..=0x27BE
        | 0x2800..=0x28FF
        | 0x2B00..=0x2B0D
        | 0x2B12..=0x2B2F
        | 0x2B4D..=0x2B73
        | 0x2B76..=0x2B95
        | 0x2B97..=0x2BFD
        | 0x2BFF
        | 0x4DC0..=0x4DFF
        | 0xFFF9..=0xFFFB
        | 0x10140..=0x1018E
        | 0x10190..=0x1019C
        | 0x101A0
        | 0x101D0..=0x101FD
        | 0x102E0..=0x102FB
        | 0x10E60..=0x10E7E
        | 0x1D2E0..=0x1D2F3
        | 0x1D300..=0x1D356
        | 0x1D360..=0x1D378
        | 0x1F000..=0x1F02B
        | 0x1F030..=0x1F093
        | 0x1F0A0..=0x1F0AE
        | 0x1F0B1..=0x1F0BF
        | 0x1F0C1..=0x1F0CF
        | 0x1F0D1..=0x1F0F5
        | 0x1F30D..=0x1F30F
        | 0x1F315
        | 0x1F31C
        | 0x1F321..=0x1F32C
        | 0x1F336
        | 0x1F378
        | 0x1F37D
        | 0x1F394..=0x1F39F
        | 0x1F3A7
        | 0x1F3AC..=0x1F3AE
        | 0x1F3C2
        | 0x1F3CB..=0x1F3CE
        | 0x1F3D4..=0x1F3DF
        | 0x1F3ED
        | 0x1F3F1..=0x1F3F3
        | 0x1F3F5..=0x1F3F7
        | 0x1F408
        | 0x1F415
        | 0x1F41F
        | 0x1F426
        | 0x1F43F
        | 0x1F441
        | 0x1F446..=0x1F449
        | 0x1F44C..=0x1F44E
        | 0x1F453
        | 0x1F46A
        | 0x1F47D
        | 0x1F4A3
        | 0x1F4B0
        | 0x1F4B3
        | 0x1F4B9
        | 0x1F4BB
        | 0x1F4BF
        | 0x1F4C8..=0x1F4CB
        | 0x1F4DA
        | 0x1F4DF
        | 0x1F4E4..=0x1F4E6
        | 0x1F4EA..=0x1F4ED
        | 0x1F4F9..=0x1F4FB
        | 0x1F4FD
        | 0x1F4FE
        | 0x1F503
        | 0x1F507..=0x1F50A
        | 0x1F50D
        | 0x1F512
        | 0x1F513
        | 0x1F53E..=0x1F545
        | 0x1F54A
        | 0x1F550..=0x1F579
        | 0x1F57B..=0x1F594
        | 0x1F597..=0x1F5A3
        | 0x1F5A5..=0x1F5FA
        | 0x1F650..=0x1F67F
        | 0x1F687
        | 0x1F68D
        | 0x1F691
        | 0x1F694
        | 0x1F698
        | 0x1F6AD
        | 0x1F6B2
        | 0x1F6B9
        | 0x1F6BA
        | 0x1F6BC
        | 0x1F6C6..=0x1F6CB
        | 0x1F6CD..=0x1F6CF
        | 0x1F6D3..=0x1F6D7
        | 0x1F6E0..=0x1F6EA
        | 0x1F6F0..=0x1F6F3
        | 0x1F6F7..=0x1F6FC
        | 0x1F780..=0x1F7D8
        | 0x1F7E0..=0x1F7EB
        | 0x1F800..=0x1F80B
        | 0x1F810..=0x1F847
        | 0x1F850..=0x1F859
        | 0x1F860..=0x1F887
        | 0x1F890..=0x1F8AD
        | 0x1F8B0..=0x1F8B1
        | 0x1F93B
        | 0x1F946
        | 0x1FA00..=0x1FA53
        | 0x1FA60..=0x1FA6D
        | 0x1FA70..=0x1FA74
        | 0x1FA78..=0x1FA7A
        | 0x1FA80..=0x1FA86
        | 0x1FA90..=0x1FAA8
        | 0x1FAB0..=0x1FAB6
        | 0x1FAC0..=0x1FAC2
        | 0x1FAD0..=0x1FAD6
        | 0x1FB00..=0x1FBCA
        | 0x1FBF0..=0x1FBF9 => HB_SYMBOL_MISC_TWO,

        0x2049
        | 0x2122
        | 0x2139
        | 0x23EA..=0x23EC
        | 0x23F0
        | 0x2705
        | 0x2708..=0x270C
        | 0x2728
        | 0x274C
        | 0x274E
        | 0x2753..=0x2755
        | 0x2795..=0x2797
        | 0x27B0
        | 0x27BF
        | 0x3030
        | 0x303D
        | 0x3297
        | 0x3299
        | 0xFEFF
        | 0x1F191..=0x1F19A
        | 0x1F1E6..=0x1F1FF
        | 0x1F201
        | 0x1F202
        | 0x1F21A
        | 0x1F22F
        | 0x1F232..=0x1F23A
        | 0x1F250
        | 0x1F251
        | 0x1F300..=0x1F320
        | 0x1F330..=0x1F335
        | 0x1F337..=0x1F37C
        | 0x1F380..=0x1F393
        | 0x1F3A0..=0x1F3C4
        | 0x1F3C6..=0x1F3CA
        | 0x1F3E0..=0x1F3F0
        | 0x1F400..=0x1F429
        | 0x1F42B..=0x1F43E
        | 0x1F440
        | 0x1F442..=0x1F4F7
        | 0x1F4F9..=0x1F4FC
        | 0x1F500..=0x1F53D
        | 0x1F5FB..=0x1F640
        | 0x1F645..=0x1F64F
        | 0x1F680..=0x1F697
        | 0x1F699..=0x1F6C5
        | 0xFE4E5..=0xFE4EE
        | 0xFE82C
        | 0xFE82E..=0xFE837 => HB_SYMBOL_EMOJI,

        _ => HB_SCRIPT_UNKNOWN,
    }
}
