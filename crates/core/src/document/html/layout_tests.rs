use super::hyph_lang;
use crate::document::html::layout::Language;

#[test]
fn test_hyph_lang() {
    assert_eq!(hyph_lang("zh-latn-pinyin"), Some(Language::Chinese));
    assert_eq!(hyph_lang("EN"), Some(Language::EnglishUS));
    assert_eq!(hyph_lang("en-GB"), Some(Language::EnglishGB));
    assert_eq!(hyph_lang("DE-ZZZ"), Some(Language::German1996));
    assert_eq!(hyph_lang("de-CH-uuu"), Some(Language::GermanSwiss));
    assert_eq!(hyph_lang("y"), None);
}
