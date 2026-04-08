#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use crate::storage::SearchResult;

/// Result ranker for advanced ranking
pub struct ResultRanker {
    /// Boost for recent documents
    recency_boost: f32,
    /// Boost for exact matches
    exact_match_boost: f32,
    /// Boost for title matches
    title_boost: f32,
}

impl ResultRanker {
    /// Create a new ranker
    pub fn new() -> Self {
        Self {
            recency_boost: 0.1,
            exact_match_boost: 0.2,
            title_boost: 0.15,
        }
    }

    /// Rank search results
    pub fn rank(&self, mut results: Vec<SearchResult>, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        for result in &mut results {
            let mut boost = 0.0f32;

            // Recency boost
            let age_days = (chrono::Utc::now() - result.document.created_at).num_days();
            if age_days < 30 {
                boost += self.recency_boost * (1.0 - age_days as f32 / 30.0);
            }

            // Exact match boost
            let content_lower = result.document.content.to_lowercase();
            if content_lower.contains(&query_lower) {
                boost += self.exact_match_boost;
            }

            // Word match boost
            let word_matches = query_words
                .iter()
                .filter(|word| content_lower.contains(**word))
                .count();
            boost += 0.05 * word_matches as f32;

            // Title/source boost
            if let Some(source) = result.document.metadata.get("source_file") {
                let source_lower = source.to_lowercase();
                if query_words.iter().any(|w| source_lower.contains(w)) {
                    boost += self.title_boost;
                }
            }

            // Apply boost
            result.score = (result.score + boost).min(1.0);
        }

        // Re-sort by boosted score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        results
    }

    /// Set recency boost
    pub fn with_recency_boost(mut self, boost: f32) -> Self {
        self.recency_boost = boost;
        self
    }

    /// Set exact match boost
    pub fn with_exact_match_boost(mut self, boost: f32) -> Self {
        self.exact_match_boost = boost;
        self
    }

    /// Set title boost
    pub fn with_title_boost(mut self, boost: f32) -> Self {
        self.title_boost = boost;
        self
    }
}

impl Default for ResultRanker {
    fn default() -> Self {
        Self::new()
    }
}
