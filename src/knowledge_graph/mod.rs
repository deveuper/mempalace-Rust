#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod graph;
pub mod node;
pub mod edge;

pub use graph::KnowledgeGraph;
pub use node::{Node, NodeType};
pub use edge::{Edge, EdgeType};

/// A fact in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
    pub source: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// A relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub relation_type: String,
    pub metadata: HashMap<String, String>,
}

/// Knowledge graph operations
pub trait KnowledgeGraphOps {
    /// Add a node to the graph
    fn add_node(&mut self, node: Node) -> Result<()>;

    /// Add an edge to the graph
    fn add_edge(&mut self, edge: Edge) -> Result<()>;

    /// Add a fact to the graph
    fn add_fact(&mut self, fact: Fact) -> Result<()>;

    /// Get node by ID
    fn get_node(&self, id: &str) -> Result<Option<Node>>;

    /// Get edges from a node
    fn get_edges_from(&self, node_id: &str) -> Result<Vec<Edge>>;

    /// Get edges to a node
    fn get_edges_to(&self, node_id: &str) -> Result<Vec<Edge>>;

    /// Search for nodes by name
    fn search_nodes(&self, query: &str) -> Result<Vec<Node>>;

    /// Get related entities
    fn get_related(&self, node_id: &str, depth: usize) -> Result<Vec<Node>>;

    /// Check for contradictions
    fn check_contradictions(&self) -> Result<Vec<(Fact, Fact)>>;
}
