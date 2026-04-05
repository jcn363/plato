use fxhash::FxHashMap;
use lazy_static::lazy_static;
use regex::Regex;

pub const DEFAULT_CONTRAST_EXPONENT: f32 = 1.0;
pub const DEFAULT_CONTRAST_GRAY: f32 = 224.0;

lazy_static! {
    pub static ref TITLE_PREFIXES: FxHashMap<&'static str, Regex> = {
        let mut p = FxHashMap::default();
        p.insert("en", Regex::new(r"^(The|An?)\s").unwrap());
        p.insert(
            "fr",
            Regex::new(r"^(Les?\s|La\s|L’|Une?\s|Des?\s|Du\s)").unwrap(),
        );
        p
    };
}
