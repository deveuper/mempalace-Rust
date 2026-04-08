#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;

use crate::storage::{SearchResult, VectorStore};

pub mod filters;
pub mod ranking;

use filters::SearchFilters;
use ranking::ResultRanker;

/// Advanced search functionality
pub struct SearchEngine {
    ranker: ResultRanker,
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new() -> Self {
        Self {
            ranker: ResultRanker::new(),
        }
    }

    /// Search with advanced ranking
    pub fn search(
        &self,
        store: &mut VectorStore,
        query: &str,
        filters: SearchFilters,
    ) -> Result<Vec<SearchResult>> {
        let results = store.search(
            query,
            filters.limit.unwrap_or(10),
            filters.wing.as_deref(),
            filters.room.as_deref(),
        )?;

        // Apply additional ranking
        let ranked = self.ranker.rank(results, query);

        Ok(ranked)
    }

    /// Hybrid search combining semantic and keyword search
    pub fn hybrid_search(
        &self,
        store: &mut VectorStore,
        query: &str,
        filters: SearchFilters,
    ) -> Result<Vec<SearchResult>> {
        let semantic_results = store.search(
            query,
            filters.limit.unwrap_or(10) * 2,
            filters.wing.as_deref(),
            filters.room.as_deref(),
        )?;

        let fts_results = store.search_fts(query, filters.limit.unwrap_or(10) * 2)?;

        // Merge results
        let mut combined: std::collections::HashMap<String, SearchResult> =
            std::collections::HashMap::new();

        for r in semantic_results {
            combined.insert(r.document.id.clone(), r);
        }

        for r in fts_results {
            combined
                .entry(r.document.id.clone())
                .and_modify(|e| e.score = e.score.max(r.score * 0.8)) // Weight FTS slightly lower
                .or_insert(r);
        }

        let mut results: Vec<SearchResult> = combined.into_values().collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        if let Some(limit) = filters.limit {
            results.truncate(limit);
        }

        Ok(results)
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
