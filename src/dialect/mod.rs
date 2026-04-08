#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

pub mod compressor;
pub mod entity_codes;
pub mod emotion_codes;

pub use compressor::{AaakCompressor, CompressionStats};
pub use entity_codes::EntityCodeRegistry;
pub use emotion_codes::EMOTION_CODES;

/// AAAK (Abbreviated AI Knowledge) Dialect
/// A compressed symbolic format for AI memory
///
/// Format:
/// - Header: FILE_NUM|PRIMARY_ENTITY|DATE|TITLE
/// - Zettel: ZID:ENTITIES|topic_keywords|"key_quote"|WEIGHT|EMOTIONS|FLAGS
/// - Tunnel: T:ZID<->ZID|label
/// - Arc: ARC:emotion->emotion->emotion
pub struct AaakDialect {
    entity_registry: EntityCodeRegistry,
    emotion_codes: HashMap<String, String>,
}

impl AaakDialect {
    /// Create a new AAAK dialect
    pub fn new() -> Self {
        Self {
            entity_registry: EntityCodeRegistry::new(),
            emotion_codes: EMOTION_CODES.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    /// Encode a full text to AAAK format
    pub fn encode(&mut self, text: &str) -> Result<String> {
        let mut output = String::new();

        // Add header
        let header = self.encode_header(text);
        output.push_str(&header);
        output.push('\n');

        // Encode sentences as zettels
        let sentences = self.split_into_sentences(text);
        for (idx, sentence) in sentences.iter().enumerate() {
            let zettel = self.encode_zettel(idx, sentence)?;
            output.push_str(&zettel);
            output.push('\n');
        }

        Ok(output)
    }

    /// Encode header
    fn encode_header(&self, text: &str) -> String {
        // Extract primary entity (first mentioned proper noun)
        let primary_entity = self.extract_primary_entity(text);

        // Current date
        let date = chrono::Local::now().format("%Y%m%d").to_string();

        // Extract title (first sentence or first 50 chars)
        let title: String = text
            .lines()
            .next()
            .unwrap_or("Untitled")
            .chars()
            .take(50)
            .collect();

        format!("0001|{}|{}|{}", primary_entity, date, title)
    }

    /// Encode a sentence as a zettel
    fn encode_zettel(&mut self, idx: usize, sentence: &str) -> Result<String> {
        let zid = format!("{:04}", idx + 1);

        // Extract entities and encode them
        let entities = self.extract_entities(sentence);
        let entity_codes: Vec<String> = entities
            .iter()
            .map(|e| self.entity_registry.encode(e))
            .collect();

        // Extract topic keywords
        let keywords = self.extract_keywords(sentence);

        // Extract key quote (truncated sentence)
        let key_quote = self.truncate_sentence(sentence);

        // Calculate weight (importance score)
        let weight = self.calculate_weight(sentence);

        // Detect emotions
        let emotions = self.detect_emotions(sentence);

        // Detect flags
        let flags = self.detect_flags(sentence);

        Ok(format!(
            "{}:{}|{}|\"{}\"|{:.2}|{}|{}",
            zid,
            entity_codes.join(","),
            keywords.join(","),
            key_quote,
            weight,
            emotions.join(","),
            flags.join(",")
        ))
    }

    /// Extract primary entity from text
    fn extract_primary_entity(&self, text: &str) -> String {
        // Look for proper nouns
        let proper_noun_regex = Regex::new(r"\b[A-Z][a-z]+\b").unwrap();

        if let Some(matched) = proper_noun_regex.find(text) {
            matched.as_str().to_string()
        } else {
            "UNK".to_string()
        }
    }

    /// Extract entities from sentence
    fn extract_entities(&self, sentence: &str) -> Vec<String> {
        let mut entities = Vec::new();

        // Proper nouns
        let proper_noun_regex = Regex::new(r"\b[A-Z][a-zA-Z]+\b").unwrap();
        for cap in proper_noun_regex.captures_iter(sentence) {
            if let Some(matched) = cap.get(0) {
                entities.push(matched.as_str().to_string());
            }
        }

        // Technical terms
        let tech_regex = Regex::new(r"\b[A-Za-z]+[0-9]*(?:\.[A-Za-z]+)*\b").unwrap();
        for cap in tech_regex.captures_iter(sentence) {
            if let Some(matched) = cap.get(0) {
                let term = matched.as_str();
                if term.len() > 3 && !entities.contains(&term.to_string()) {
                    entities.push(term.to_string());
                }
            }
        }

        entities.truncate(5); // Limit entities per zettel
        entities
    }

    /// Extract keywords from sentence
    fn extract_keywords(&self, sentence: &str) -> Vec<String> {
        let mut keywords = Vec::new();

        // Skip common words
        let stop_words: std::collections::HashSet<&str> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will",
            "would", "could", "should", "may", "might", "must", "shall",
            "can", "need", "dare", "ought", "used", "to", "of", "in",
            "for", "on", "with", "at", "by", "from", "as", "into",
            "through", "during", "before", "after", "above", "below",
            "between", "under", "and", "but", "or", "yet", "so",
        ].iter().cloned().collect();

        // Extract significant words
        let word_regex = Regex::new(r"\b[a-zA-Z]{4,}\b").unwrap();
        for cap in word_regex.captures_iter(sentence) {
            if let Some(matched) = cap.get(0) {
                let word = matched.as_str().to_lowercase();
                if !stop_words.contains(word.as_str()) && !keywords.contains(&word) {
                    keywords.push(word);
                }
            }
        }

        keywords.truncate(5);
        keywords
    }

    /// Truncate sentence for key quote
    fn truncate_sentence(&self, sentence: &str) -> String {
        let max_len = 80;
        if sentence.len() <= max_len {
            sentence.to_string()
        } else {
            format!("{}...", &sentence[..max_len])
        }
    }

    /// Calculate importance weight
    fn calculate_weight(&self, sentence: &str) -> f32 {
        let mut weight = 0.5f32;

        // Boost for decision keywords
        if sentence.to_lowercase().contains("decided")
            || sentence.to_lowercase().contains("chose")
        {
            weight += 0.3;
        }

        // Boost for problem keywords
        if sentence.to_lowercase().contains("problem")
            || sentence.to_lowercase().contains("issue")
        {
            weight += 0.2;
        }

        // Boost for discovery keywords
        if sentence.to_lowercase().contains("discovered")
            || sentence.to_lowercase().contains("found")
        {
            weight += 0.2;
        }

        weight.min(1.0)
    }

    /// Detect emotions in sentence
    fn detect_emotions(&self, sentence: &str) -> Vec<String> {
        let mut emotions = Vec::new();
        let sentence_lower = sentence.to_lowercase();

        for (emotion, code) in &self.emotion_codes {
            if sentence_lower.contains(emotion) {
                emotions.push(code.clone());
            }
        }

        emotions.truncate(3);
        emotions
    }

    /// Detect flags in sentence
    fn detect_flags(&self, sentence: &str) -> Vec<String> {
        let mut flags = Vec::new();
        let sentence_lower = sentence.to_lowercase();

        if sentence_lower.contains("decided") || sentence_lower.contains("decision") {
            flags.push("DECISION".to_string());
        }
        if sentence_lower.contains("important") || sentence_lower.contains("critical") {
            flags.push("CORE".to_string());
        }
        if sentence_lower.contains("first") || sentence_lower.contains("initial") {
            flags.push("ORIGIN".to_string());
        }
        if sentence_lower.contains("changed") || sentence_lower.contains("shifted") {
            flags.push("PIVOT".to_string());
        }
        if sentence_lower.contains("architecture")
            || sentence_lower.contains("implementation")
        {
            flags.push("TECHNICAL".to_string());
        }

        flags
    }

    /// Split text into sentences
    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        let sentence_regex = Regex::new(r"[.!?]+\s+").unwrap();
        sentence_regex
            .split(text)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

impl Default for AaakDialect {
    fn default() -> Self {
        Self::new()
    }
}
