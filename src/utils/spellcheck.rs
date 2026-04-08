#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use std::collections::HashSet;

/// Simple spell checker
pub struct SpellChecker {
    dictionary: HashSet<String>,
}

impl SpellChecker {
    /// Create a new spell checker with default dictionary
    pub fn new() -> Self {
        let mut dictionary = HashSet::new();

        // Add common English words
        let common_words = [
            "the", "be", "to", "of", "and", "a", "in", "that", "have", "i",
            "it", "for", "not", "on", "with", "he", "as", "you", "do", "at",
            "this", "but", "his", "by", "from", "they", "we", "say", "her",
            "she", "or", "an", "will", "my", "one", "all", "would", "there",
            "their", "what", "so", "up", "out", "if", "about", "who", "get",
            "which", "go", "me", "when", "make", "can", "like", "time", "no",
            "just", "him", "know", "take", "people", "into", "year", "your",
            "good", "some", "could", "them", "see", "other", "than", "then",
            "now", "look", "only", "come", "its", "over", "think", "also",
            "back", "after", "use", "two", "how", "our", "work", "first",
            "well", "way", "even", "new", "want", "because", "any", "these",
            "give", "day", "most", "us", "is", "was", "are", "were", "been",
            "has", "had", "did", "does", "doing", "done", "being", "having",
        ];

        for word in &common_words {
            dictionary.insert(word.to_string());
        }

        Self { dictionary }
    }

    /// Check if a word is spelled correctly
    pub fn check(&self, word: &str) -> bool {
        let word_lower = word.to_lowercase();
        self.dictionary.contains(&word_lower)
    }

    /// Suggest corrections for a misspelled word
    pub fn suggest(&self, word: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let word_lower = word.to_lowercase();

        // Try edit distance 1
        for candidate in self.edit_distance_1(&word_lower) {
            if self.dictionary.contains(&candidate) && !suggestions.contains(&candidate) {
                suggestions.push(candidate);
            }
        }

        // Limit suggestions
        suggestions.truncate(5);
        suggestions
    }

    /// Generate words at edit distance 1
    fn edit_distance_1(&self, word: &str) -> Vec<String> {
        let mut results = Vec::new();
        let chars: Vec<char> = word.chars().collect();
        let alphabet = "abcdefghijklmnopqrstuvwxyz";

        // Deletions
        for i in 0..chars.len() {
            let mut candidate = String::new();
            for (j, c) in chars.iter().enumerate() {
                if i != j {
                    candidate.push(*c);
                }
            }
            results.push(candidate);
        }

        // Insertions
        for i in 0..=chars.len() {
            for c in alphabet.chars() {
                let mut candidate = String::new();
                for (j, ch) in chars.iter().enumerate() {
                    if i == j {
                        candidate.push(c);
                    }
                    candidate.push(*ch);
                }
                if i == chars.len() {
                    candidate.push(c);
                }
                results.push(candidate);
            }
        }

        // Substitutions
        for i in 0..chars.len() {
            for c in alphabet.chars() {
                let mut candidate = String::new();
                for (j, ch) in chars.iter().enumerate() {
                    if i == j {
                        candidate.push(c);
                    } else {
                        candidate.push(*ch);
                    }
                }
                results.push(candidate);
            }
        }

        // Transpositions
        for i in 0..chars.len() - 1 {
            let mut candidate = String::new();
            for (j, c) in chars.iter().enumerate() {
                if j == i {
                    candidate.push(chars[i + 1]);
                } else if j == i + 1 {
                    candidate.push(chars[i]);
                } else {
                    candidate.push(*c);
                }
            }
            results.push(candidate);
        }

        results
    }

    /// Add a word to the dictionary
    pub fn add_word(&mut self, word: &str) {
        self.dictionary.insert(word.to_lowercase());
    }

    /// Check text and return misspelled words
    pub fn check_text(&self, text: &str) -> Vec<(String, Vec<String>)> {
        let mut misspelled = Vec::new();

        for word in text.split_whitespace() {
            // Remove punctuation
            let clean_word: String = word.chars().filter(|c| c.is_alphabetic()).collect();

            if !clean_word.is_empty() && !self.check(&clean_word) {
                let suggestions = self.suggest(&clean_word);
                misspelled.push((clean_word, suggestions));
            }
        }

        misspelled
    }
}

impl Default for SpellChecker {
    fn default() -> Self {
        Self::new()
    }
}
