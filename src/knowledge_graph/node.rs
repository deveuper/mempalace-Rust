#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node types in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Person,
    Project,
    Organization,
    Technology,
    Concept,
    Document,
    Room,
    Wing,
    Hall,
    Custom(String),
}

impl NodeType {
    pub fn as_str(&self) -> &str {
        match self {
            NodeType::Person => "person",
            NodeType::Project => "project",
            NodeType::Organization => "organization",
            NodeType::Technology => "technology",
            NodeType::Concept => "concept",
            NodeType::Document => "document",
            NodeType::Room => "room",
            NodeType::Wing => "wing",
            NodeType::Hall => "hall",
            NodeType::Custom(s) => s.as_str(),
        }
    }
}

/// A node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Node {
    /// Create a new node
    pub fn new(id: String, name: String, node_type: NodeType) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            name,
            node_type,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Update the node
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now();
    }

    /// Check if node matches a query
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower)
            || self.metadata.values().any(|v| v.to_lowercase().contains(&query_lower))
    }
}
