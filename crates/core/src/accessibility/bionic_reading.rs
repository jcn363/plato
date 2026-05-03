//! Bionic Reading Implementation
//!
//! Implements Bionic Reading technique which bolds the first half of words
//! to increase reading speed and focus. Based on the Bionic Reading API
//! (https://bionic-reading.com/).

use std::borrow::Cow;

/// Apply bionic reading transformation to text
///
/// Bolds the first half of each word to guide the eye and increase reading speed.
/// The `intensity` parameter controls how much of each word to bold (0.0 to 1.0).
///
/// # Arguments
/// * `text` - The input text to transform
/// * `intensity` - How much of each word to bold (0.0 = none, 1.0 = entire word)
///
/// # Returns
/// Transformed text with bold markers (uses ** for bold)
pub fn apply_bionic_reading(text: &str, intensity: f32) -> String {
    if intensity <= 0.0 {
        return text.to_string();
    }

    let intensity = intensity.clamp(0.0, 1.0);
    let mut result = String::with_capacity(text.len() + text.len() / 10);

    for word in text.split_inclusive(|c: char| !c.is_alphabetic()) {
        // Check if the word contains alphabetic characters
        let alpha_count = word.chars().filter(|c| c.is_alphabetic()).count();
        if alpha_count == 0 {
            // Not a word, just append
            result.push_str(word);
            continue;
        }

        // Find the alphabetic part
        let start = word.chars().position(|c| c.is_alphabetic()).unwrap_or(0);

        // Find the end of alphabetic part (last alphabetic char)
        let mut end = word.len();
        for (i, c) in word.char_indices().rev() {
            if c.is_alphabetic() {
                end = i + c.len_utf8();
                break;
            }
        }

        if start >= end || alpha_count < 2 {
            // Too short or no alphabetic chars
            result.push_str(word);
            continue;
        }

        // Split into prefix (non-alpha) + word + suffix (non-alpha)
        let prefix = &word[..start];
        let word_part = &word[start..end];
        let suffix = &word[end..];

        let bold_len = ((word_part.chars().count() as f32 * intensity).ceil() as usize)
            .clamp(1, word_part.len());

        // Find the byte index for the bold section
        let mut char_count = 0;
        let mut bold_end = 0;
        for (i, ch) in word_part.char_indices() {
            if char_count >= bold_len {
                break;
            }
            bold_end = i + ch.len_utf8();
            char_count += 1;
        }

        // Append: prefix + **bold_part** + rest + suffix
        result.push_str(prefix);
        result.push_str("**");
        result.push_str(&word_part[..bold_end]);
        result.push_str("**");
        result.push_str(&word_part[bold_end..]);
        result.push_str(suffix);
    }

    result
}

/// Simple bionic reading that returns (bold_part, rest_part) for each word
///
/// This is easier to use with rendering systems that don't support markup
pub fn split_word_bionic(word: &str, intensity: f32) -> (Cow<str>, Cow<str>) {
    if intensity <= 0.0 || word.len() < 2 {
        return (Cow::Borrowed(word), Cow::Borrowed(""));
    }

    let intensity = intensity.clamp(0.0, 1.0);
    let char_count = word.chars().count();

    if char_count < 2 {
        return (Cow::Borrowed(word), Cow::Borrowed(""));
    }

    let bold_count = ((char_count as f32 * intensity).ceil() as usize).clamp(1, char_count);

    // Find the split point in bytes
    let mut current_char = 0;
    let mut split_byte = 0;
    for (i, ch) in word.char_indices() {
        if current_char >= bold_count {
            break;
        }
        split_byte = i + ch.len_utf8();
        current_char += 1;
    }

    let (bold_part, rest_part) = word.split_at(split_byte);
    (Cow::Borrowed(bold_part), Cow::Borrowed(rest_part))
}

/// Process text for bionic reading
///
/// Returns a vector of (text, is_bold) tuples for rendering
pub fn process_bionic_text(text: &str, intensity: f32) -> Vec<(String, bool)> {
    if intensity <= 0.0 {
        return vec![(text.to_string(), false)];
    }

    let intensity = intensity.clamp(0.0, 1.0);
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut in_word = false;

    for ch in text.chars() {
        let is_alpha = ch.is_alphabetic();

        if is_alpha {
            if !in_word {
                // Starting a new word - process any accumulated non-word
                if !current_word.is_empty() {
                    result.push((current_word.clone(), false));
                    current_word.clear();
                }
                in_word = true;
            }
            current_word.push(ch);
        } else {
            if in_word {
                // End of word - process with bionic
                if current_word.len() >= 2 {
                    let (bold, rest) = split_word_bionic(&current_word, intensity);
                    result.push((bold.into_owned(), true));
                    if !rest.is_empty() {
                        result.push((rest.into_owned(), false));
                    }
                } else {
                    result.push((current_word.clone(), false));
                }
                current_word.clear();
                in_word = false;
            }
            current_word.push(ch);
        }
    }

    // Process remaining
    if in_word {
        if current_word.len() >= 2 {
            let (bold, rest) = split_word_bionic(&current_word, intensity);
            result.push((bold.into_owned(), true));
            if !rest.is_empty() {
                result.push((rest.into_owned(), false));
            }
        } else {
            result.push((current_word, false));
        }
    } else if !current_word.is_empty() {
        result.push((current_word, false));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bionic_simple() {
        let result = apply_bionic_reading("Hello world", 0.5);
        assert!(result.contains("**"), "Should contain bold markers");
        // Check that parts of the words are present (bionic reading transforms them)
        assert!(result.contains("Hel"), "Should contain start of 'Hello'");
        assert!(result.contains("wor"), "Should contain start of 'world'");
    }

    #[test]
    fn test_bionic_intensity_0() {
        let text = "Hello world";
        let result = apply_bionic_reading(text, 0.0);
        assert_eq!(result, text);
    }

    #[test]
    fn test_split_word() {
        let (bold, rest) = split_word_bionic("Hello", 0.5);
        assert!(!bold.is_empty());
        // Bold part should be the first half
        assert_eq!(bold, "Hel");
        assert_eq!(rest, "lo");
    }

    #[test]
    fn test_process_bionic_text() {
        let result = process_bionic_text("Hello world", 0.5);
        assert!(!result.is_empty());
        // Check that at least one part is marked as bold
        assert!(result.iter().any(|(_, is_bold)| *is_bold));
    }

    #[test]
    fn test_short_word() {
        let result = process_bionic_text("I am", 0.5);
        // Short words might not be processed
        assert!(!result.is_empty());
    }
}
