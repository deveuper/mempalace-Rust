#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::{Context, Result};
use ndarray::Array1;

/// Embedding model configuration
#[derive(Debug, Clone)]
pub enum EmbeddingModel {
    /// All-MiniLM-L6-v2 (default, 384 dimensions)
    AllMiniLmL6V2,
    /// All-MiniLM-L12-v2 (384 dimensions, slightly better quality)
    AllMiniLmL12V2,
    /// All-MPNet-Base-v2 (768 dimensions, best quality)
    AllMpNetBaseV2,
    /// Custom model with dimensions
    Custom { name: String, dimensions: usize },
}

impl EmbeddingModel {
    pub fn dimensions(&self) -> usize {
        match self {
            EmbeddingModel::AllMiniLmL6V2 => 384,
            EmbeddingModel::AllMiniLmL12V2 => 384,
            EmbeddingModel::AllMpNetBaseV2 => 768,
            EmbeddingModel::Custom { dimensions, .. } => *dimensions,
        }
    }

    pub fn model_name(&self) -> String {
        match self {
            EmbeddingModel::AllMiniLmL6V2 => "all-MiniLM-L6-v2".to_string(),
            EmbeddingModel::AllMiniLmL12V2 => "all-MiniLM-L12-v2".to_string(),
            EmbeddingModel::AllMpNetBaseV2 => "all-mpnet-base-v2".to_string(),
            EmbeddingModel::Custom { name, .. } => name.clone(),
        }
    }
}

impl Default for EmbeddingModel {
    fn default() -> Self {
        EmbeddingModel::AllMiniLmL6V2
    }
}

/// Embedder for generating text embeddings
/// 
/// Note: This is a simplified implementation. In production, you should:
/// 1. Use an external embedding service (OpenAI, Cohere, etc.)
/// 2. Or integrate with a local embedding library like fastembed or rust-bert
/// 3. Or use a pre-built embedding server
pub struct Embedder {
    model: EmbeddingModel,
    dimensions: usize,
}

impl Embedder {
    /// Create a new embedder with the specified model
    pub fn new(model: EmbeddingModel) -> Result<Self> {
        let dimensions = model.dimensions();
        
        Ok(Self {
            model,
            dimensions,
        })
    }

    /// Get the embedding dimensions
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Embed a single text
    /// 
    /// This simplified implementation creates a deterministic hash-based embedding.
    /// For real semantic search, replace this with actual embedding generation.
    pub fn embed(&mut self, text: &str) -> Result<Array1<f32>> {
        // Create a deterministic embedding based on text hash
        // This is NOT a real embedding - it's just for demonstration
        // In production, replace with actual embedding model
        let hash = Self::hash_text(text);
        
        // Generate pseudo-random but deterministic embedding
        let mut embedding = Vec::with_capacity(self.dimensions);
        let mut rng = hash;
        
        for _ in 0..self.dimensions {
            // Simple LCG random number generator
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let value = ((rng % 1000) as f32 / 1000.0) * 2.0 - 1.0; // Range [-1, 1]
            embedding.push(value);
        }
        
        // Normalize the embedding
        let arr = Array1::from_vec(embedding);
        let norm = arr.dot(&arr).sqrt();
        
        if norm > 0.0 {
            Ok(&arr / norm)
        } else {
            Ok(arr)
        }
    }

    /// Embed multiple texts (batch processing)
    pub fn embed_batch(&mut self, texts: &[String]) -> Result<Vec<Array1<f32>>> {
        texts.iter().map(|text| self.embed(text)).collect()
    }

    /// Compute cosine similarity between two embeddings
    pub fn cosine_similarity(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
        let dot = a.dot(b);
        let norm_a = a.dot(a).sqrt();
        let norm_b = b.dot(b).sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
        }
    }

    /// Compute Euclidean distance between two embeddings
    pub fn euclidean_distance(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
        (a - b).mapv(|x| x * x).sum().sqrt()
    }
    
    /// Hash text to a seed value
    fn hash_text(text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let b = Array1::from_vec(vec![0.0, 1.0, 0.0]);
        let c = Array1::from_vec(vec![1.0, 0.0, 0.0]);

        assert!((Embedder::cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
        assert!((Embedder::cosine_similarity(&a, &c) - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_embed() {
        let mut embedder = Embedder::new(EmbeddingModel::AllMiniLmL6V2).unwrap();
        let embedding = embedder.embed("test text").unwrap();
        
        assert_eq!(embedding.len(), 384);
        
        // Check normalization
        let norm = embedding.dot(&embedding).sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }
    
    #[test]
    fn test_embed_deterministic() {
        let mut embedder1 = Embedder::new(EmbeddingModel::AllMiniLmL6V2).unwrap();
        let mut embedder2 = Embedder::new(EmbeddingModel::AllMiniLmL6V2).unwrap();
        
        let e1 = embedder1.embed("same text").unwrap();
        let e2 = embedder2.embed("same text").unwrap();
        
        // Same text should produce same embedding
        assert!(e1.iter().zip(e2.iter()).all(|(a, b)| (a - b).abs() < 1e-6));
    }
}
