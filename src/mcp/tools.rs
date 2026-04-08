#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use serde_json::Value;

use crate::mcp::protocol::{ToolContent, ToolResult};
use crate::storage::VectorStore;

/// MCP tools implementation
pub struct McpTools {
    store: VectorStore,
}

impl McpTools {
    /// Create a new tools instance
    pub fn new(store: VectorStore) -> Self {
        Self { store }
    }

    /// Search the palace
    pub async fn search(&self, arguments: Value) -> Result<Value> {
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;

        let wing = arguments.get("wing").and_then(|v| v.as_str());
        let room = arguments.get("room").and_then(|v| v.as_str());
        let n_results = arguments
            .get("n_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as usize;

        // Note: This requires mutable store, so we need to handle this differently
        // For now, return a placeholder
        let results = serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": format!("Search query: {}\nWing: {:?}\nRoom: {:?}", query, wing, room)
                }
            ]
        });

        Ok(results)
    }

    /// Get wake-up context
    pub async fn wake_up(&self, arguments: Value) -> Result<Value> {
        let wing = arguments.get("wing").and_then(|v| v.as_str());

        let context = if let Some(w) = wing {
            format!("Wake-up context for wing: {}\n\n[Context would be loaded here]", w)
        } else {
            "General wake-up context:\n\n[Context would be loaded here]".to_string()
        };

        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": context
                }
            ]
        }))
    }

    /// Get palace status
    pub async fn status(&self) -> Result<Value> {
        let doc_count = self.store.count()?;
        let wings = self.store.get_wings()?;

        let status_text = format!(
            "MemPalace Status:\n\nDocuments: {}\nWings: {}\n",
            doc_count,
            wings.len()
        );

        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": status_text
                }
            ]
        }))
    }
}
