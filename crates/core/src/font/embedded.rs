//! Embedded Font Data
//!
//! This module contains the embedded font data declarations for platform-specific
//! linking with the MuPDF library. These are external static arrays that reference
//! font data embedded in the library.

#[cfg(all(not(target_os = "ios"), any(not(target_os = "linux"), target_arch = "arm")))]
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

#[cfg(all(not(target_os = "ios"), target_os = "linux", not(target_arch = "arm")))]
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
