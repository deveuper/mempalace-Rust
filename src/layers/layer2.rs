#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;

use crate::storage::{SearchResult, VectorStore};

/// Layer 2: On-Demand Context
/// ~200-500 tokens each, loaded when a topic/wing comes up
pub struct Layer2 {
    context_size: usize,
}

impl Layer2 {
    /// Create a new Layer2
    pub fn new(context_size: usize) -> Self {
        Self { context_size }
    }

    /// Load context for a specific query
    pub fn load_context(
        &self,
        store: &mut VectorStore,
        query: &str,
        wing: Option<&str>,
    ) -> Result<String> {
        let results = store.search(query, 3, wing, None)?;

        if results.is_empty() {
            return Ok(String::new());
        }

        let mut output = String::new();
        output.push_str(&format!("Context for '{}':\n", query));

        for (idx, result) in results.iter().enumerate() {
            output.push_str(&format!("\n{}. ", idx + 1));

            if let Some(ref room) = result.document.metadata.get("room") {
                output.push_str(&format!("[{}] ", room));
            }

            let content = self.truncate_to_tokens(&result.document.content, self.context_size / 3);
            output.push_str(&content);
        }

        Ok(output)
    }

    /// Load context for a specific room
    pub fn load_room_context(
        &self,
        store: &mut VectorStore,
        wing: &str,
        room: &str,
    ) -> Result<String> {
        // Get documents from the room
        let results = store.search("*", 5, Some(wing), Some(room))?;

        if results.is_empty() {
            return Ok(format!("No context found for {}/{}", wing, room));
        }

        let mut output = String::new();
        output.push_str(&format!("Context for {}/{}:\n", wing, room));

        for (idx, result) in results.iter().enumerate() {
            output.push_str(&format!("\n{}. ", idx + 1));

            let content = self.truncate_to_tokens(&result.document.content, self.context_size / 5);
            output.push_str(&content);
        }

        Ok(output)
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

    /// Estimate token count for a typical context load
    pub fn estimate_tokens(&self) -> usize {
        self.context_size
    }
}
