#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use ndarray::Array1;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::embedding::Embedder;

/// A document stored in the vector database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

impl Document {
    pub fn new(content: String, metadata: HashMap<String, String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            embedding: None,
            metadata,
            created_at: Utc::now(),
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

/// Search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
    pub distance: f32,
}

/// Vector store using SQLite for metadata and flat index for vectors
/// For production, this can be upgraded to HNSW or IVF indexing
pub struct VectorStore {
    conn: Connection,
    embedder: Embedder,
    collection_name: String,
}

impl VectorStore {
    /// Open or create a vector store at the given path
    pub fn open<P: AsRef<Path>>(path: P, embedder: Embedder) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open SQLite database")?;

        let store = Self {
            conn,
            embedder,
            collection_name: "mempalace_drawers".to_string(),
        };

        store.init_schema()?;
        info!("Vector store initialized");

        Ok(store)
    }

    /// Create an in-memory vector store for testing
    pub fn new_in_memory(embedder: Embedder) -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to create in-memory database")?;

        let store = Self {
            conn,
            embedder,
            collection_name: "mempalace_drawers".to_string(),
        };

        store.init_schema()?;
        Ok(store)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        // Documents table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                embedding BLOB,
                metadata TEXT NOT NULL,
                created_at TEXT NOT NULL,
                wing TEXT,
                room TEXT,
                hall TEXT,
                source_file TEXT
            )",
            [],
        )?;

        // Indexes for metadata filtering
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wing ON documents(wing)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_room ON documents(room)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_hall ON documents(hall)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON documents(created_at)",
            [],
        )?;

        // FTS (Full Text Search) virtual table for hybrid search
        self.conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
                content,
                content_rowid=id
            )",
            [],
        )?;

        // Collection metadata
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS collection_info (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Insert default collection info
        self.conn.execute(
            "INSERT OR IGNORE INTO collection_info (key, value) VALUES ('name', 'mempalace_drawers')",
            [],
        )?;
        self.conn.execute(
            "INSERT OR IGNORE INTO collection_info (key, value) VALUES ('version', '3.0.0')",
            [],
        )?;

        Ok(())
    }

    /// Add a document to the store
    pub fn add(&mut self, mut doc: Document) -> Result<String> {
        // Generate embedding if not present
        if doc.embedding.is_none() {
            let embedding = self.embedder.embed(&doc.content)?;
            doc.embedding = Some(embedding.to_vec());
        }

        let embedding_bytes = doc
            .embedding
            .as_ref()
            .map(|e| Self::embedding_to_bytes(e));

        let metadata_json = serde_json::to_string(&doc.metadata)?;

        let wing = doc.metadata.get("wing").cloned();
        let room = doc.metadata.get("room").cloned();
        let hall = doc.metadata.get("hall").cloned();
        let source_file = doc.metadata.get("source_file").cloned();

        self.conn.execute(
            "INSERT INTO documents (id, content, embedding, metadata, created_at, wing, room, hall, source_file)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                content = excluded.content,
                embedding = excluded.embedding,
                metadata = excluded.metadata,
                wing = excluded.wing,
                room = excluded.room,
                hall = excluded.hall,
                source_file = excluded.source_file",
            params![
                doc.id,
                doc.content,
                embedding_bytes,
                metadata_json,
                doc.created_at.to_rfc3339(),
                wing,
                room,
                hall,
                source_file,
            ],
        )?;

        // Update FTS index
        self.conn.execute(
            "INSERT OR REPLACE INTO documents_fts (rowid, content) VALUES (?1, ?2)",
            params![doc.id, doc.content],
        )?;

        debug!("Added document: {}", doc.id);
        Ok(doc.id)
    }

    /// Add multiple documents (batch insert)
    pub fn add_batch(&mut self, docs: Vec<Document>) -> Result<Vec<String>> {
        let mut ids = Vec::with_capacity(docs.len());

        // Collect texts for batch embedding
        let texts: Vec<String> = docs.iter().map(|d| d.content.clone()).collect();
        let embeddings = self.embedder.embed_batch(&texts)?;

        for (mut doc, embedding) in docs.into_iter().zip(embeddings) {
            doc.embedding = Some(embedding.to_vec());
            let id = self.add(doc)?;
            ids.push(id);
        }

        info!("Added {} documents", ids.len());

        Ok(ids)
    }

    /// Get a document by ID
    pub fn get(&self, id: &str) -> Result<Option<Document>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, embedding, metadata, created_at FROM documents WHERE id = ?1"
        )?;

        let doc = stmt
            .query_row([id], |row| {
                let embedding_bytes: Option<Vec<u8>> = row.get(2)?;
                let metadata_json: String = row.get(3)?;

                Ok(Document {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    embedding: embedding_bytes.map(|b| Self::bytes_to_embedding(&b)),
                    metadata: serde_json::from_str(&metadata_json).unwrap_or_default(),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })
            .optional()?;

        Ok(doc)
    }

    /// Delete a document by ID
    pub fn delete(&self, id: &str) -> Result<bool> {
        let changed = self.conn.execute("DELETE FROM documents WHERE id = ?1", [id])?;
        self.conn
            .execute("DELETE FROM documents_fts WHERE rowid = ?1", [id])?;

        Ok(changed > 0)
    }

    /// Search for similar documents
    pub fn search(
        &mut self,
        query: &str,
        n_results: usize,
        wing: Option<&str>,
        room: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedder.embed(query)?;

        // Build WHERE clause and params
        let mut where_parts = Vec::new();
        let mut wing_owned = None;
        let mut room_owned = None;

        if let Some(w) = wing {
            where_parts.push("wing = ?".to_string());
            wing_owned = Some(w.to_string());
        }
        if let Some(r) = room {
            where_parts.push("room = ?".to_string());
            room_owned = Some(r.to_string());
        }

        let where_sql = if where_parts.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_parts.join(" AND "))
        };

        // Fetch all documents with embeddings (for now, brute force)
        // TODO: Implement HNSW or IVF for approximate search
        let sql = format!(
            "SELECT id, content, embedding, metadata, created_at FROM documents {}",
            where_sql
        );

        let mut stmt = self.conn.prepare(&sql)?;

        // Bind parameters
        let mut param_idx = 1;
        if let Some(ref w) = wing_owned {
            stmt.raw_bind_parameter(param_idx, w.as_str())?;
            param_idx += 1;
        }
        if let Some(ref r) = room_owned {
            stmt.raw_bind_parameter(param_idx, r.as_str())?;
        }

        let docs: Vec<Document> = stmt
            .query_map([], |row| {
                let embedding_bytes: Option<Vec<u8>> = row.get(2)?;
                let metadata_json: String = row.get(3)?;

                Ok(Document {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    embedding: embedding_bytes.map(|b| Self::bytes_to_embedding(&b)),
                    metadata: serde_json::from_str(&metadata_json).unwrap_or_default(),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .filter_map(|r| r.ok())
            .filter(|d| d.embedding.is_some())
            .collect();

        // Compute similarities and rank
        let mut results: Vec<SearchResult> = docs
            .into_iter()
            .filter_map(|doc| {
                let embedding = Array1::from_vec(doc.embedding.clone().unwrap());
                let similarity = super::embedding::Embedder::cosine_similarity(
                    &query_embedding,
                    &embedding,
                );
                let distance = 1.0 - similarity;

                Some(SearchResult {
                    document: doc,
                    score: similarity,
                    distance,
                })
            })
            .collect();

        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Take top n
        results.truncate(n_results);

        Ok(results)
    }

    /// Full-text search using SQLite FTS
    pub fn search_fts(&self, query: &str, n_results: usize) -> Result<Vec<SearchResult>> {
        let sql = r#"
            SELECT d.id, d.content, d.embedding, d.metadata, d.created_at,
                   rank as score
            FROM documents_fts fts
            JOIN documents d ON fts.rowid = d.id
            WHERE documents_fts MATCH ?1
            ORDER BY rank
            LIMIT ?2
        "#;

        let mut stmt = self.conn.prepare(sql)?;

        let results = stmt
            .query_map(params![query, n_results], |row| {
                let embedding_bytes: Option<Vec<u8>> = row.get(2)?;
                let metadata_json: String = row.get(3)?;
                let score: f64 = row.get(5)?;

                Ok(SearchResult {
                    document: Document {
                        id: row.get(0)?,
                        content: row.get(1)?,
                        embedding: embedding_bytes.map(|b| Self::bytes_to_embedding(&b)),
                        metadata: serde_json::from_str(&metadata_json).unwrap_or_default(),
                        created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                            .unwrap()
                            .with_timezone(&Utc),
                    },
                    score: score as f32,
                    distance: 0.0,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    /// Hybrid search: combine semantic and FTS results
    pub fn search_hybrid(
        &mut self,
        query: &str,
        n_results: usize,
        wing: Option<&str>,
        room: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let semantic_results = self.search(query, n_results, wing, room)?;
        let fts_results = self.search_fts(query, n_results)?;

        // Merge and deduplicate results
        let mut combined: HashMap<String, SearchResult> = HashMap::new();

        for r in semantic_results {
            combined.insert(r.document.id.clone(), r);
        }

        for r in fts_results {
            combined
                .entry(r.document.id.clone())
                .and_modify(|e| e.score = e.score.max(r.score))
                .or_insert(r);
        }

        let mut results: Vec<SearchResult> = combined.into_values().collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(n_results);

        Ok(results)
    }

    /// Count documents in the store
    pub fn count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Get all unique wings
    pub fn get_wings(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT wing FROM documents WHERE wing IS NOT NULL")?;

        let wings = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(wings)
    }

    /// Get all rooms in a wing
    pub fn get_rooms(&self, wing: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT room FROM documents WHERE wing = ?1 AND room IS NOT NULL"
        )?;

        let rooms = stmt
            .query_map([wing], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(rooms)
    }

    /// Convert embedding vector to bytes for storage
    fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
        embedding
            .iter()
            .flat_map(|&f| f.to_le_bytes())
            .collect()
    }

    /// Convert bytes back to embedding vector
    fn bytes_to_embedding(bytes: &[u8]) -> Vec<f32> {
        bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_bytes_conversion() {
        let embedding = vec![1.0f32, 2.0, 3.0, 4.0];
        let bytes = VectorStore::embedding_to_bytes(&embedding);
        let recovered = VectorStore::bytes_to_embedding(&bytes);
        assert_eq!(embedding, recovered);
    }
}
