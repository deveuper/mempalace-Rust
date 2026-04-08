#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Edge types in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    /// Person works on Project
    WorksOn,
    /// Person knows Person
    Knows,
    /// Project uses Technology
    Uses,
    /// Document belongs to Room
    BelongsTo,
    /// Room is in Wing
    InWing,
    /// Hall connects Rooms
    Connects,
    /// Tunnel links Wings
    Links,
    /// Document mentions Entity
    Mentions,
    /// Fact about Entity
    FactAbout,
    /// Custom relationship
    Custom(String),
}

impl EdgeType {
    pub fn as_str(&self) -> &str {
        match self {
            EdgeType::WorksOn => "works_on",
            EdgeType::Knows => "knows",
            EdgeType::Uses => "uses",
            EdgeType::BelongsTo => "belongs_to",
            EdgeType::InWing => "in_wing",
            EdgeType::Connects => "connects",
            EdgeType::Links => "links",
            EdgeType::Mentions => "mentions",
            EdgeType::FactAbout => "fact_about",
            EdgeType::Custom(s) => s.as_str(),
        }
    }
}

/// An edge in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    pub weight: f32,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Edge {
    /// Create a new edge
    pub fn new(id: String, from: String, to: String, edge_type: EdgeType) -> Self {
        Self {
            id,
            from,
            to,
            edge_type,
            weight: 1.0,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Set weight
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Check if edge involves a node
    pub fn involves(&self, node_id: &str) -> bool {
        self.from == node_id || self.to == node_id
    }

    /// Get the other end of the edge
    pub fn other(&self, node_id: &str) -> Option<&String> {
        if self.from == node_id {
            Some(&self.to)
        } else if self.to == node_id {
            Some(&self.from)
        } else {
            None
        }
    }
}
