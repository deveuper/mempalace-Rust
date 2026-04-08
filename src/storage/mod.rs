#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

pub mod embedding;
pub mod vector_store;

pub use embedding::{EmbeddingModel, Embedder};
pub use vector_store::{Document, VectorStore, SearchResult};
