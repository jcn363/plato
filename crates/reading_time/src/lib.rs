//! Plato Reading Time Module
//!
//! This crate provides reading time estimation for Plato.

pub use plato_core::reading_time::{
    count_words, estimate_from_page_count, estimate_from_word_count, format_duration,
    remaining_time, ReadingSpeed,
};
