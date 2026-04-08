#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use chrono::Utc;

use crate::storage::{Document, VectorStore};

/// Layer 1: Essential Story
/// ~500-800 tokens, always loaded
/// Contains top moments from the palace
pub struct Layer1 {
    top_moments_count: usize,
}

/// A moment in the essential story
#[derive(Debug, Clone)]
pub struct Moment {
    pub content: String,
    pub wing: Option<String>,
    pub room: Option<String>,
    pub importance: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Layer1 {
    /// Create a new Layer1
    pub fn new(top_moments_count: usize) -> Self {
        Self { top_moments_count }
    }

    /// Render the essential story
    pub fn render(&self, store: &mut VectorStore) -> Result<String> {
        let moments = self.get_top_moments(store)?;

        if moments.is_empty() {
            return Ok(String::new());
        }

        let mut output = String::new();

        for (idx, moment) in moments.iter().enumerate() {
            output.push_str(&format!("{}. ", idx + 1));

            if let Some(ref wing) = moment.wing {
                output.push_str(&format!("[{}] ", wing));
            }

            // Truncate content to fit token budget
            let content = self.truncate_to_tokens(&moment.content, 100);
            output.push_str(&content);
            output.push('\n');
        }

        Ok(output)
    }

    /// Render essential story for a specific wing
    pub fn render_for_wing(&self, wing: &str, store: &mut VectorStore) -> Result<String> {
        let moments = self.get_top_moments_for_wing(store, wing)?;

        if moments.is_empty() {
            return Ok(format!("No essential story found for wing: {}", wing));
        }

        let mut output = String::new();

        for (idx, moment) in moments.iter().enumerate() {
            output.push_str(&format!("{}. ", idx + 1));

            if let Some(ref room) = moment.room {
                output.push_str(&format!("[{}] ", room));
            }

            let content = self.truncate_to_tokens(&moment.content, 100);
            output.push_str(&content);
            output.push('\n');
        }

        Ok(output)
    }

    /// Get top moments from the palace
    fn get_top_moments(&self, store: &mut VectorStore) -> Result<Vec<Moment>> {
        // For now, return recent documents as "moments"
        // In a full implementation, this would use importance scoring

        let wings = store.get_wings()?;
        let mut moments = Vec::new();

        for wing in wings.iter().take(self.top_moments_count) {
            // Get one document per wing as a representative moment
            if let Ok(results) = store.search(&format!("important decision {}", wing), 1, Some(wing), None) {
                for result in results {
                    moments.push(Moment {
                        content: result.document.content.clone(),
                        wing: result.document.metadata.get("wing").cloned(),
                        room: result.document.metadata.get("room").cloned(),
                        importance: result.score,
                        timestamp: result.document.created_at,
                    });
                }
            }
        }

        // Sort by importance
        moments.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        moments.truncate(self.top_moments_count);

        Ok(moments)
    }

    /// Get top moments for a specific wing
    fn get_top_moments_for_wing(&self, store: &mut VectorStore, wing: &str) -> Result<Vec<Moment>> {
        let rooms = store.get_rooms(wing)?;
        let mut moments = Vec::new();

        // Get one moment per room
        for room in rooms.iter().take(self.top_moments_count) {
            if let Ok(results) = store.search("important", 1, Some(wing), Some(room)) {
                for result in results {
                    moments.push(Moment {
                        content: result.document.content.clone(),
                        wing: Some(wing.to_string()),
                        room: Some(room.clone()),
                        importance: result.score,
                        timestamp: result.document.created_at,
                    });
                }
            }
        }

        // Sort by importance
        moments.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        moments.truncate(self.top_moments_count);

        Ok(moments)
    }

    /// Truncate content to approximate token count
    fn truncate_to_tokens(&self, content: &str, max_tokens: usize) -> String {
        let max_chars = max_tokens * 4; // Rough estimate

        if content.len() <= max_chars {
            return content.to_string();
        }

        // Find a good breaking point
        let truncate_at = content
            .char_indices()
            .take_while(|(i, _)| *i < max_chars)
            .last()
            .map(|(i, _)| i)
            .unwrap_or(max_chars);

        format!("{}...", &content[..truncate_at])
    }

    /// Estimate token count
    pub fn estimate_tokens(&self) -> usize {
        // Layer 1 typically uses 500-800 tokens
        600
    }
}
