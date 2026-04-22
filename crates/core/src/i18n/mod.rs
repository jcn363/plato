use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{LazyLock, RwLock};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum Language {
    #[default]
    English,
    Spanish,
    French,
    German,
    Italian,
    Portuguese,
    Russian,
    Chinese,
    Japanese,
    Korean,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Italian => "it",
            Language::Portuguese => "pt",
            Language::Russian => "ru",
            Language::Chinese => "zh",
            Language::Japanese => "ja",
            Language::Korean => "ko",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::German => "Deutsch",
            Language::Italian => "Italiano",
            Language::Portuguese => "Português",
            Language::Russian => "Русский",
            Language::Chinese => "中文",
            Language::Japanese => "日本語",
            Language::Korean => "한국어",
        }
    }

    pub fn from_code(code: &str) -> Option<Language> {
        match code {
            "en" => Some(Language::English),
            "es" => Some(Language::Spanish),
            "fr" => Some(Language::French),
            "de" => Some(Language::German),
            "it" => Some(Language::Italian),
            "pt" => Some(Language::Portuguese),
            "ru" => Some(Language::Russian),
            "zh" => Some(Language::Chinese),
            "ja" => Some(Language::Japanese),
            "ko" => Some(Language::Korean),
            _ => None,
        }
    }

    pub fn all() -> Vec<Language> {
        vec![
            Language::English,
            Language::Spanish,
            Language::French,
            Language::German,
            Language::Italian,
            Language::Portuguese,
            Language::Russian,
            Language::Chinese,
            Language::Japanese,
            Language::Korean,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEntry {
    pub value: String,
    pub plural: Option<PluralForms>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluralForms {
    pub zero: Option<String>,
    pub one: Option<String>,
    pub two: Option<String>,
    pub few: Option<String>,
    pub many: Option<String>,
    pub other: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationData {
    pub language: String,
    pub region: Option<String>,
    pub translations: HashMap<String, TranslationEntry>,
    pub metadata: TranslationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationMetadata {
    pub version: String,
    pub last_modified: String,
    pub translators: Vec<String>,
    pub completeness: f64,
}

pub type TranslationMap = HashMap<String, TranslationEntry>;

pub static CURRENT_LANGUAGE: LazyLock<RwLock<Language>> =
    LazyLock::new(|| RwLock::new(Language::English));

pub static TRANSLATIONS: LazyLock<RwLock<HashMap<String, TranslationData>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));


pub fn set_language(lang: Language) {
    *CURRENT_LANGUAGE
        .write()
        .expect("CURRENT_LANGUAGE lock poisoned") = lang;
}

pub fn get_language() -> Language {
    *CURRENT_LANGUAGE
        .read()
        .expect("CURRENT_LANGUAGE lock poisoned")
}

/// Load translations from JSON files in a directory
pub fn load_translations_from_dir<P: AsRef<Path>>(dir: P) -> Result<(), Error> {
    let dir = dir.as_ref();
    if !dir.exists() {
        return Ok(()); // No translations directory is fine
    }

    let mut translations = TRANSLATIONS
        .write()
        .expect("TRANSLATIONS lock poisoned");

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read translation file {}", path.display()))?;
            
            let data: TranslationData = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse translation file {}", path.display()))?;
            
            translations.insert(data.language.clone(), data);
        }
    }

    Ok(())
}

/// Simple translation function
pub fn t(key: &str) -> String {
    t_with_context(key, None)
}

/// Translation with context
pub fn t_with_context(key: &str, context: Option<&str>) -> String {
    let lang = get_language();
    let lang_code = lang.code();
    
    let translations = TRANSLATIONS
        .read()
        .expect("TRANSLATIONS lock poisoned");
    
    // Try to get translation from loaded files
    if let Some(data) = translations.get(lang_code) {
        if let Some(entry) = data.translations.get(key) {
            // Check context if provided
            if let Some(req_context) = context {
                if let Some(entry_context) = &entry.context {
                    if entry_context != req_context {
                        // Context mismatch, try fallback
                        return get_fallback_translation(key);
                    }
                }
            }
            return entry.value.clone();
        }
    }
    
    // Fallback to hardcoded translations
    get_fallback_translation(key)
}

/// Pluralized translation
pub fn tn(key: &str, count: usize) -> String {
    tn_with_context(key, count, None)
}

/// Pluralized translation with context
pub fn tn_with_context(key: &str, count: usize, context: Option<&str>) -> String {
    let lang = get_language();
    let lang_code = lang.code();
    
    let translations = TRANSLATIONS
        .read()
        .expect("TRANSLATIONS lock poisoned");
    
    // Try to get translation from loaded files
    if let Some(data) = translations.get(lang_code) {
        if let Some(entry) = data.translations.get(key) {
            if let Some(ref plural) = entry.plural {
                let plural_form = get_plural_form(lang, count);
                return match plural_form {
                    PluralForm::Zero => plural.zero.as_ref().unwrap_or(&plural.other).clone(),
                    PluralForm::One => plural.one.as_ref().unwrap_or(&plural.other).clone(),
                    PluralForm::Two => plural.two.as_ref().unwrap_or(&plural.other).clone(),
                    PluralForm::Few => plural.few.as_ref().unwrap_or(&plural.other).clone(),
                    PluralForm::Many => plural.many.as_ref().unwrap_or(&plural.other).clone(),
                    PluralForm::Other => plural.other.clone(),
                };
            }
        }
    }
    
    // Fallback to simple translation
    t_with_context(key, context)
}

/// Interpolated translation with variables
pub fn ti(key: &str, vars: &[(&str, &str)]) -> String {
    let mut result = t(key);
    
    for (var_name, var_value) in vars {
        result = result.replace(&format!("{{{}}}", var_name), var_value);
    }
    
    result
}

/// Interpolated pluralized translation
pub fn tni(key: &str, count: usize, vars: &[(&str, &str)]) -> String {
    let mut result = tn(key, count);
    
    for (var_name, var_value) in vars {
        result = result.replace(&format!("{{{}}}", var_name), var_value);
    }
    
    result
}

fn get_fallback_translation(key: &str) -> String {
    // Return the key as last resort - no hardcoded fallbacks
    format!("[{}]", key)
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum PluralForm {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

fn get_plural_form(lang: Language, count: usize) -> PluralForm {
    match lang {
        Language::English | Language::German | Language::Italian | Language::Portuguese | Language::Chinese | Language::Japanese | Language::Korean => {
            if count == 1 {
                PluralForm::One
            } else {
                PluralForm::Other
            }
        },
        Language::Spanish => {
            if count == 1 {
                PluralForm::One
            } else {
                PluralForm::Other
            }
        },
        Language::French => {
            if count == 0 || count == 1 {
                PluralForm::One
            } else {
                PluralForm::Other
            }
        },
        Language::Russian => {
            let tens = count % 100;
            let ones = count % 10;
            
            if tens >= 10 && tens <= 20 {
                PluralForm::Many
            } else if ones == 1 {
                PluralForm::One
            } else if ones >= 2 && ones <= 4 {
                PluralForm::Few
            } else {
                PluralForm::Many
            }
        },
    }
}

/// Get available languages with their translation completeness
pub fn get_available_languages() -> Vec<(Language, f64)> {
    let translations = TRANSLATIONS
        .read()
        .expect("TRANSLATIONS lock poisoned");
    
    Language::all()
        .into_iter()
        .map(|lang| {
            let completeness = translations
                .get(lang.code())
                .map(|data| data.metadata.completeness)
                .unwrap_or(0.0);
            (lang, completeness)
        })
        .collect()
}

/// Export current translations to JSON format
pub fn export_translations<P: AsRef<Path>>(output_dir: P) -> Result<(), Error> {
    let translations = TRANSLATIONS
        .read()
        .expect("TRANSLATIONS lock poisoned");
    
    fs::create_dir_all(&output_dir)?;
    
    for (lang_code, data) in translations.iter() {
        let file_path = output_dir.as_ref().join(format!("{}.json", lang_code));
        let json = serde_json::to_string_pretty(data)?;
        fs::write(file_path, json)?;
    }
    
    Ok(())
}


