use regex::Regex;
use rustc_hash::FxHashMap;
use std::sync::LazyLock;

pub const DEFAULT_CONTRAST_EXPONENT: f32 = 1.0;
pub const DEFAULT_CONTRAST_GRAY: f32 = 224.0;

pub static TITLE_PREFIXES: LazyLock<FxHashMap<&'static str, Regex>> = LazyLock::new(|| {
    let mut p = FxHashMap::default();
    p.insert("en", Regex::new(r"^(The|An?)\s").unwrap());
    p.insert(
        "fr",
        Regex::new(r"^(Les?\s|La\s|L'Une?\s|Des?\s|Du\s)").unwrap(),
    );
    p
});
