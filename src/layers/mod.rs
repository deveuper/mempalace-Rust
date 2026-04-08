#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use std::path::Path;

use crate::config::Config;
use crate::storage::{Embedder, VectorStore};

pub mod layer0;
pub mod layer1;
pub mod layer2;

pub use layer0::Layer0;
pub use layer1::Layer1;
pub use layer2::Layer2;

/// 4-Layer Memory Stack
/// Layer 0: Identity (~100 tokens) - Always loaded
/// Layer 1: Essential Story (~500-800) - Always loaded
/// Layer 2: On-Demand (~200-500 each) - Loaded when topic comes up
/// Layer 3: Deep Search (unlimited) - Full semantic search
pub struct LayerStack {
    pub layer0: Layer0,
    pub layer1: Layer1,
    pub layer2: Layer2,
    pub store: VectorStore,
}

impl LayerStack {
    /// Create a new layer stack
    pub fn new(config: &Config) -> Result<Self> {
        let embedder = Embedder::new(config.embedding_model())?;
        let store = VectorStore::open(&config.db_path, embedder)?;

        Ok(Self {
            layer0: Layer0::new(&config.layers.identity_file),
            layer1: Layer1::new(config.layers.top_moments_count),
            layer2: Layer2::new(config.layers.on_demand_context_size),
            store,
        })
    }

    /// Estimate total token count
    pub fn estimate_tokens(&self) -> usize {
        let l0_tokens = self.layer0.estimate_tokens();
        let l1_tokens = self.layer1.estimate_tokens();

        l0_tokens + l1_tokens
    }

    /// Render the full wake-up context
    pub fn render_wakeup(&mut self) -> Result<String> {
        let mut output = String::new();

        // Layer 0: Identity
        let identity = self.layer0.render();
        if !identity.is_empty() {
            output.push_str("=== Identity ===\n");
            output.push_str(&identity);
            output.push_str("\n\n");
        }

        // Layer 1: Essential Story
        let essential = self.layer1.render(&mut self.store)?;
        if !essential.is_empty() {
            output.push_str("=== Essential Story ===\n");
            output.push_str(&essential);
            output.push_str("\n\n");
        }

        Ok(output)
    }

    /// Render wake-up context for a specific wing
    pub fn render_wakeup_for_wing(&mut self, wing: &str) -> Result<String> {
        let mut output = String::new();

        // Layer 0: Identity
        let identity = self.layer0.render();
        if !identity.is_empty() {
            output.push_str("=== Identity ===\n");
            output.push_str(&identity);
            output.push_str("\n\n");
        }

        // Layer 1: Essential Story for wing
        let essential = self.layer1.render_for_wing(wing, &mut self.store)?;
        if !essential.is_empty() {
            output.push_str(&format!("=== Essential Story: {} ===\n", wing));
            output.push_str(&essential);
            output.push_str("\n\n");
        }

        Ok(output)
    }
}
