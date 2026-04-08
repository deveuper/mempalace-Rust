#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use tokio::fs;

use crate::knowledge_graph::{
    Edge, EdgeType, Fact, KnowledgeGraphOps, Node, NodeType, Relationship,
};

/// In-memory knowledge graph
#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Vec<Edge>>,
    facts: Vec<Fact>,
    name_index: HashMap<String, Vec<String>>, // name -> node_ids
}

impl KnowledgeGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            facts: Vec::new(),
            name_index: HashMap::new(),
        }
    }

    /// Load from file
    pub async fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path).await?;
            let graph: SerializedGraph = serde_json::from_str(&content)?;

            let mut kg = Self::new();

            for node in graph.nodes {
                kg.add_node(node)?;
            }

            for edge in graph.edges {
                kg.add_edge(edge)?;
            }

            kg.facts = graph.facts;

            Ok(kg)
        } else {
            Ok(Self::new())
        }
    }

    /// Save to file
    pub async fn save(&self, path: &Path) -> Result<()> {
        let serialized = SerializedGraph {
            nodes: self.nodes.values().cloned().collect(),
            edges: self.edges.values().flatten().cloned().collect(),
            facts: self.facts.clone(),
        };

        let content = serde_json::to_string_pretty(&serialized)?;
        fs::write(path, content).await?;

        Ok(())
    }

    /// Index a node by name
    fn index_node(&mut self, node: &Node) {
        let name_lower = node.name.to_lowercase();
        self.name_index
            .entry(name_lower)
            .or_default()
            .push(node.id.clone());
    }

    /// Build the palace graph from documents
    pub fn build_from_documents(&mut self, documents: &[crate::storage::Document]) -> Result<()> {
        for doc in documents {
            // Create document node
            let doc_node = Node::new(
                doc.id.clone(),
                format!("Document: {}", doc.id[..8].to_string()),
                NodeType::Document,
            );
            self.add_node(doc_node)?;

            // Extract entities and create nodes
            let entities = self.extract_entities(&doc.content);
            for entity in entities {
                let entity_id = format!("entity:{}", entity.to_lowercase());

                if !self.nodes.contains_key(&entity_id) {
                    let node_type = self.infer_entity_type(&entity);
                    let node = Node::new(entity_id.clone(), entity.clone(), node_type);
                    self.add_node(node)?;
                }

                // Create mention edge
                let edge_id = format!("{}->mentions->{}", doc.id, entity_id);
                let edge = Edge::new(
                    edge_id,
                    doc.id.clone(),
                    entity_id,
                    EdgeType::Mentions,
                );
                self.add_edge(edge)?;
            }

            // Add wing/room relationships
            if let Some(wing) = doc.metadata.get("wing") {
                let wing_id = format!("wing:{}", wing.to_lowercase());
                if !self.nodes.contains_key(&wing_id) {
                    let wing_node = Node::new(wing_id.clone(), wing.clone(), NodeType::Wing);
                    self.add_node(wing_node)?;
                }

                let edge_id = format!("{}->in_wing->{}", doc.id, wing_id);
                let edge = Edge::new(edge_id, doc.id.clone(), wing_id, EdgeType::InWing);
                self.add_edge(edge)?;
            }

            if let Some(room) = doc.metadata.get("room") {
                let room_id = format!("room:{}", room.to_lowercase());
                if !self.nodes.contains_key(&room_id) {
                    let room_node = Node::new(room_id.clone(), room.clone(), NodeType::Room);
                    self.add_node(room_node)?;
                }

                let edge_id = format!("{}->belongs_to->{}", doc.id, room_id);
                let edge = Edge::new(edge_id, doc.id.clone(), room_id, EdgeType::BelongsTo);
                self.add_edge(edge)?;
            }
        }

        Ok(())
    }

    /// Extract entities from text
    fn extract_entities(&self, text: &str) -> Vec<String> {
        let mut entities = Vec::new();
        let regex = regex::Regex::new(r"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b").unwrap();

        for cap in regex.captures_iter(text) {
            if let Some(matched) = cap.get(0) {
                let entity = matched.as_str().to_string();
                if entity.len() > 2 && !entities.contains(&entity) {
                    entities.push(entity);
                }
            }
        }

        entities.truncate(20);
        entities
    }

    /// Infer entity type from name
    fn infer_entity_type(&self, name: &str) -> NodeType {
        let name_lower = name.to_lowercase();

        // Check for person indicators
        if name_lower.contains("inc.")
            || name_lower.contains("corp")
            || name_lower.contains("llc")
            || name_lower.contains("ltd")
        {
            return NodeType::Organization;
        }

        // Check for technology indicators
        let tech_keywords = ["api", "framework", "library", "database", "server", "client"];
        if tech_keywords.iter().any(|kw| name_lower.contains(kw)) {
            return NodeType::Technology;
        }

        // Default to concept
        NodeType::Concept
    }

    /// Get graph statistics
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.values().map(|v| v.len()).sum(),
            fact_count: self.facts.len(),
        }
    }
}

/// Graph statistics
#[derive(Debug, Clone)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub fact_count: usize,
}

/// Serializable graph structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SerializedGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    facts: Vec<Fact>,
}

impl KnowledgeGraphOps for KnowledgeGraph {
    fn add_node(&mut self, node: Node) -> Result<()> {
        self.index_node(&node);
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    fn add_edge(&mut self, edge: Edge) -> Result<()> {
        self.edges
            .entry(edge.from.clone())
            .or_default()
            .push(edge.clone());

        // Also add reverse edge for undirected relationships
        match edge.edge_type {
            EdgeType::Knows | EdgeType::Connects | EdgeType::Links => {
                let reverse_edge = Edge::new(
                    format!("{}_reverse", edge.id),
                    edge.to.clone(),
                    edge.from.clone(),
                    edge.edge_type.clone(),
                )
                .with_weight(edge.weight);
                self.edges
                    .entry(edge.to.clone())
                    .or_default()
                    .push(reverse_edge);
            }
            _ => {}
        }

        Ok(())
    }

    fn add_fact(&mut self, fact: Fact) -> Result<()> {
        self.facts.push(fact);
        Ok(())
    }

    fn get_node(&self, id: &str) -> Result<Option<Node>> {
        Ok(self.nodes.get(id).cloned())
    }

    fn get_edges_from(&self, node_id: &str) -> Result<Vec<Edge>> {
        Ok(self.edges.get(node_id).cloned().unwrap_or_default())
    }

    fn get_edges_to(&self, node_id: &str) -> Result<Vec<Edge>> {
        let mut incoming = Vec::new();
        for edges in self.edges.values() {
            for edge in edges {
                if edge.to == node_id {
                    incoming.push(edge.clone());
                }
            }
        }
        Ok(incoming)
    }

    fn search_nodes(&self, query: &str) -> Result<Vec<Node>> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<Node> = self
            .nodes
            .values()
            .filter(|n| n.matches(&query_lower))
            .cloned()
            .collect();

        results.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(results)
    }

    fn get_related(&self, node_id: &str, depth: usize) -> Result<Vec<Node>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut related = Vec::new();

        queue.push_back((node_id.to_string(), 0));
        visited.insert(node_id.to_string());

        while let Some((current_id, current_depth)) = queue.pop_front() {
            if current_depth >= depth {
                continue;
            }

            if let Some(edges) = self.edges.get(&current_id) {
                for edge in edges {
                    let other_id = edge.other(&current_id).unwrap();

                    if !visited.contains(other_id) {
                        visited.insert(other_id.clone());
                        queue.push_back((other_id.clone(), current_depth + 1));

                        if let Some(node) = self.nodes.get(other_id) {
                            related.push(node.clone());
                        }
                    }
                }
            }
        }

        Ok(related)
    }

    fn check_contradictions(&self) -> Result<Vec<(Fact, Fact)>> {
        let mut contradictions = Vec::new();

        // Simple contradiction detection:
        // Facts with same subject and predicate but different objects
        for (i, fact1) in self.facts.iter().enumerate() {
            for fact2 in self.facts.iter().skip(i + 1) {
                if fact1.subject == fact2.subject
                    && fact1.predicate == fact2.predicate
                    && fact1.object != fact2.object
                {
                    // Check if confidence allows contradiction
                    if fact1.confidence > 0.7 && fact2.confidence > 0.7 {
                        contradictions.push((fact1.clone(), fact2.clone()));
                    }
                }
            }
        }

        Ok(contradictions)
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}
