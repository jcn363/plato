# Internationalization (i18n) Improvements

> **Last Updated**: 2026-04-22
> **Related Documents**: [GUIDE.md](./GUIDE.md) | [BUILD.md](./BUILD.md)

This document outlines the enhanced internationalization system for Plato, providing comprehensive multi-language support with advanced features.

## Overview

The enhanced i18n system provides:

- **Dynamic translation loading** from JSON files
- **Pluralization support** for multiple languages
- **Variable interpolation** in translations
- **Context-aware translations**
- **Fallback mechanisms** for missing translations
- **Translation completeness tracking**

## Supported Languages

| Language   | Code | Display Name | Status       |
|------------|------|--------------|--------------|
| English    | `en` | English      | Full Support |
| Spanish    | `es` | Español      | Full Support |
| French     | `fr` | Français     | Planned      |
| German     | `de` | Deutsch      | Planned      |
| Italian    | `it` | Italiano     | Planned      |
| Portuguese | `pt` | Português    | Planned      |
| Russian    | `ru` | Français     | Planned      |
| Chinese    | `zh` | Deutsch      | Planned      |
| Japanese   | `ja` | Italiano     | Planned      |
| Korean     | `ko` | Português    | Planned      |

## API Reference

### Basic Translation Functions

```rust
// Simple translation
let text = i18n::t("close"); // "Close" or "Cerrar"

// Translation with context
let text = i18n::t_with_context("save", Some("dialog"));

// Pluralized translation
let text = i18n::tn("book", 1); // "Book" or "Libro"
let text = i18n::tn("book", 5); // "Books" or "Libros"

// Pluralized translation with context
let text = i18n::tn_with_context("items_found", 3, Some("search"));
```

### Variable Interpolation

```rust
// Simple interpolation
let text = i18n::ti("welcome", &[("name", "John")]);
// "Welcome, John!" or "¡Bienvenido, John!"

// Pluralized interpolation
let text = i18n::tni("items_found", 5, &[("count", "5")]);
// "5 items found" or "5 elementos encontrados"
```

### Language Management

```rust
// Set current language
i18n::set_language(i18n::Language::Spanish);

// Get current language
let lang = i18n::get_language();

// Get all available languages with completeness
let languages = i18n::get_available_languages();
```

### Translation Loading

```rust
// Load translations from directory
i18n::load_translations_from_dir("translations")?;

// Export translations to JSON
i18n::export_translations("output/translations")?;
```

## Translation File Format

Translation files are stored in JSON format in the `translations/` directory:

### Example: `en.json`

```json
{
  "language": "en",
  "region": "US",
  "metadata": {
    "version": "1.0.0",
    "last_modified": "2026-04-22",
    "translators": ["Plato Team"],
    "completeness": 1.0
  },
  "translations": {
    "close": {
      "value": "Close",
      "plural": null,
      "context": null
    },
    "book": {
      "value": "Book",
      "plural": {
        "zero": "Books",
        "one": "Book",
        "other": "Books"
      },
      "context": null
    },
    "items_found": {
      "value": "{count} item found",
      "plural": {
        "zero": "No items found",
        "one": "{count} item found",
        "other": "{count} items found"
      },
      "context": "search results"
    }
  }
}
```

## Pluralization Rules

The system supports Unicode CLDR plural rules:

### English/German/Italian/Portuguese/Chinese/Japanese/Korean

- **One**: `n == 1`
- **Other**: everything else

### Spanish/French

- **One**: `n == 1` (Spanish) or `n == 0 || n == 1` (French)
- **Other**: everything else

### Russian

- **One**: ends in 1, not 11-21
- **Few**: ends in 2-4, not 12-14
- **Many**: ends in 0, 5-9, 11-19, 21-29
- **Other**: fallback

## Variable Interpolations

Variables in translations use `{variable}` syntax:

```json
{
  "welcome_user": {
    "value": "Welcome, {name}!",
    "plural": null,
    "context": null
  },
  "items_count": {
    "value": "{count} item",
    "plural": {
      "one": "{count} item",
      "other": "{count} items"
    },
    "context": null
  }
}
```

## Context-Aware Translations

Context helps disambiguate translations with same keys:

```json
{
  "file": {
    "value": "File",
    "plural": null,
    "context": "noun"
  },
  "file": {
    "value": "to file",
    "plural": null,
    "context": "verb"
  }
}
```

## Migration Guide

### From Old System

```rust
// Old system - no longer supported
use crate::i18n::t;
let text = t("close"); // This will now return "[close]" if no translation file is loaded

// New system - requires translation files to be loaded
use crate::i18n::{t, tn, ti, tni, load_translations_from_dir};

// Load translations first
load_translations_from_dir("translations")?;

// Use translation functions
let text = t("close"); // "Close" or "Cerrar" (from JSON files)
let plural_text = tn("book", count); // "Book" or "Books" with proper pluralization
let interpolated = ti("welcome", &[("name", "John")]); // "Welcome, John!"
let both = tni("items_found", count, &[("count", "5")]); // "5 items found"
```

### Adding New Translations

1. Create translation file: `translations/xx.json`
2. Add translations following the format above
3. Update `Language` enum in `i18n/mod.rs`
4. Add pluralization rules if needed

## Best Practices

### Translation Keys

- Use descriptive keys: `close_dialog`, `file_menu`, `error_network`
- Group related keys with prefixes: `menu_`, `settings_`, `error_`
- Use snake_case for consistency

### Context Usage

- Provide context for ambiguous terms
- Use context for UI elements vs. concepts
- Document context in translation files

### Variable Names

- Use descriptive variable names: `{username}`, `{page_number}`
- Avoid single-letter variables: `{x}`, `{y}`
- Keep variable names consistent across languages

### Pluralization

- Always provide plural forms for countable items
- Use appropriate plural rules for each language
- Test pluralization with various numbers

## Performance Considerations

- Translations are loaded once at startup from JSON files
- Thread-safe access using `RwLock`
- Minimal memory overhead with lazy loading
- No hardcoded fallbacks - translation files are required
- Missing translations return formatted key (e.g., "[missing_key]")

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_translation() {
        i18n::set_language(i18n::Language::English);
        assert_eq!(i18n::t("close"), "Close");
    }

    #[test]
    fn test_pluralization() {
        i18n::set_language(i18n::Language::English);
        assert_eq!(i18n::tn("book", 1), "Book");
        assert_eq!(i18n::tn("book", 2), "Books");
    }

    #[test]
    fn test_interpolation() {
        i18n::set_language(i18n::Language::English);
        let result = i18n::ti("welcome", &[("name", "John")]);
        assert_eq!(result, "Welcome, John!");
    }
}
```

## Future Enhancements

- **RTL language support** (Arabic, Hebrew)
- **Gender-aware translations**
- **Date/time localization**
- **Number formatting localization**
- **Translation validation tools**
- **Automatic translation completeness reporting**
- **Community translation platform integration**
