#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use crate::storage::Document;

/// General extractor for classifying content into categories
pub struct GeneralExtractor {
    extract_type: Option<String>,
}

/// Extracted content types
#[derive(Debug, Clone)]
pub enum ContentType {
    Decision,
    Milestone,
    Problem,
    Preference,
    Discovery,
    Advice,
    General,
}

impl GeneralExtractor {
    pub fn new(extract_type: Option<&str>) -> Self {
        Self {
            extract_type: extract_type.map(|s| s.to_string()),
        }
    }

    /// Extract content from a file
    pub async fn extract_file(&self, path: &Path) -> Result<Vec<Document>> {
        let content = fs::read_to_string(path).await?;

        match self.extract_type.as_deref() {
            Some("decisions") => self.extract_decisions(&content, path),
            Some("milestones") => self.extract_milestones(&content, path),
            Some("problems") => self.extract_problems(&content, path),
            Some("preferences") => self.extract_preferences(&content, path),
            Some("discoveries") => self.extract_discoveries(&content, path),
            Some("advice") => self.extract_advice(&content, path),
            _ => self.extract_all(&content, path),
        }
    }

    /// Extract all content types
    fn extract_all(&self, content: &str, path: &Path) -> Result<Vec<Document>> {
        let mut documents = Vec::new();

        // Extract each type
        documents.extend(self.extract_decisions(content, path)?);
        documents.extend(self.extract_milestones(content, path)?);
        documents.extend(self.extract_problems(content, path)?);
        documents.extend(self.extract_preferences(content, path)?);
        documents.extend(self.extract_discoveries(content, path)?);
        documents.extend(self.extract_advice(content, path)?);

        Ok(documents)
    }

    /// Extract decisions
    fn extract_decisions(&self, content: &str, path: &Path) -> Result<Vec<Document>> {
        let patterns = [
            Regex::new(r"(?i)(we decided|decision:|decided to|chose to|opted for|agreed on)\s+(.{10,500})").unwrap(),
            Regex::new(r"(?i)(let's go with|going with|will use|using)\s+(.{5,200})").unwrap(),
        ];

        self.extract_with_patterns(content, path, &patterns, "decision", "facts")
    }

    /// Extract milestones
    fn extract_milestones(&self, content: &str, path: &Path) -> Result<Vec<Document>> {
        let patterns = [
            Regex::new(r"(?i)(milestone|shipped|launched|released|completed|finished|done)\s+(.{10,500})").unwrap(),
            Regex::new(r"(?i)(version\s+\d+\.\d+|v\d+\.\d+)").unwrap(),
        ];

        self.extract_with_patterns(content, path, &patterns, "milestone", "events")
    }

    /// Extract problems
    fn extract_problems(&self, content: &str, path: &Path) -> Result<Vec<Document>> {
        let patterns = [
            Regex::new(r"(?i)(problem:|issue:|bug:|error:|failed|broken|not working)\s+(.{10,500})").unwrap(),
            Regex::new(r"(?i)(can't|cannot|doesn't|isn't|won't|couldn't)\s+(.{5,300})").unwrap(),
        ];

        self.extract_with_patterns(content, path, &patterns, "problem", "discoveries")
    }

    /// Extract preferences
    fn extract_preferences(&self, content: &str, path: &Path) -> Result<Vec<Document>> {
        let patterns = [
            Regex::new(r"(?i)(prefer|like|favorite|best|better than|instead of)\s+(.{5,300})").unwrap(),
            Regex::new(r"(?i)(i think|in my opinion|my preference)\s+(.{5,300})").unwrap(),
        ];

        self.extract_with_patterns(content, path, &patterns, "preference", "preferences")
    }

    /// Extract discoveries
    fn extract_discoveries(&self, content: &str, path: &Path) -> Result<Vec<Document>> {
        let patterns = [
            Regex::new(r"(?i)(discovered|found out|realized|learned that|turns out)\s+(.{10,500})").unwrap(),
            Regex::new(r"(?i)(interesting|fascinating|surprising|unexpected)\s+(.{5,300})").unwrap(),
        ];

        self.extract_with_patterns(content, path, &patterns, "discovery", "discoveries")
    }

    /// Extract advice
    fn extract_advice(&self, content: &str, path: &Path) -> Result<Vec<Document>> {
        let patterns = [
            Regex::new(r"(?i)(should|recommend|suggest|advice:|tip:)\s+(.{10,500})").unwrap(),
            Regex::new(r"(?i)(try|consider|look into|check out)\s+(.{5,300})").unwrap(),
        ];

        self.extract_with_patterns(content, path, &patterns, "advice", "advice")
    }

    /// Helper to extract with patterns
    fn extract_with_patterns(
        &self,
        content: &str,
        path: &Path,
        patterns: &[Regex],
        content_type: &str,
        hall: &str,
    ) -> Result<Vec<Document>> {
        let mut documents = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for pattern in patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(matched) = cap.get(0) {
                    let text = matched.as_str().trim();

                    // Deduplicate
                    if seen.contains(text) {
                        continue;
                    }
                    seen.insert(text.to_string());

                    let mut metadata = HashMap::new();
                    metadata.insert("source_file".to_string(), path.to_string_lossy().to_string());
                    metadata.insert("content_type".to_string(), content_type.to_string());
                    metadata.insert("hall".to_string(), hall.to_string());

                    documents.push(Document::new(text.to_string(), metadata));
                }
            }
        }

        Ok(documents)
    }
}
