#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

/// Text normalizer
pub struct Normalizer;

impl Normalizer {
    /// Normalize text for storage
    pub fn normalize(text: &str) -> String {
        let mut result = text.to_string();

        // Unicode normalization (NFC)
        result = result.nfc().collect();

        // Normalize whitespace
        result = Self::normalize_whitespace(&result);

        // Normalize line endings
        result = Self::normalize_line_endings(&result);

        // Remove control characters
        result = Self::remove_control_chars(&result);

        result
    }

    /// Normalize whitespace
    fn normalize_whitespace(text: &str) -> String {
        let re = Regex::new(r"[\s\x{00A0}]+").unwrap();
        re.replace_all(text, " ").to_string()
    }

    /// Normalize line endings to LF
    fn normalize_line_endings(text: &str) -> String {
        text.replace("\r\n", "\n").replace('\r', "\n")
    }

    /// Remove control characters
    fn remove_control_chars(text: &str) -> String {
        text.chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect()
    }

    /// Normalize for search (lowercase, remove punctuation)
    pub fn normalize_for_search(text: &str) -> String {
        let mut result = text.to_lowercase();

        // Remove punctuation
        let re = Regex::new(r"[^\w\s]").unwrap();
        result = re.replace_all(&result, " ").to_string();

        // Normalize whitespace
        result = Self::normalize_whitespace(&result);

        result.trim().to_string()
    }

    /// Normalize file path
    pub fn normalize_path(path: &str) -> String {
        path.replace('\\', "/")
    }
}
