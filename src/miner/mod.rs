#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::config::Config;
use crate::entity::{detect_entities, scan_for_detection, DetectedEntities, EntityRegistry};
use crate::storage::{Document, Embedder, VectorStore};

pub mod convo;
pub mod general;

pub use convo::ConversationMiner;
pub use general::GeneralExtractor;

/// Mining mode
#[derive(Debug, Clone)]
pub enum MineMode {
    /// Mine project files (code, docs, notes)
    Projects,
    /// Mine conversation exports
    Conversations,
    /// General mode with optional extraction type
    General(Option<String>),
}

/// Mining statistics
#[derive(Debug, Clone, Default)]
pub struct MineStats {
    pub files_scanned: usize,
    pub files_processed: usize,
    pub documents_created: usize,
    pub total_bytes: u64,
    pub errors: usize,
}

/// Main miner structure
pub struct Miner {
    config: Config,
    mode: MineMode,
    store: VectorStore,
    embedder: Embedder,
    registry: EntityRegistry,
}

impl Miner {
    /// Create a new miner
    pub async fn new(config: Config, mode: MineMode) -> Result<Self> {
        let embedder = Embedder::new(config.embedding_model())?;
        let store = VectorStore::open(&config.db_path, Embedder::new(config.embedding_model())?)?;

        let registry_path = config.palace_path.join("entities.json");
        let registry = EntityRegistry::load(&registry_path).await?;

        Ok(Self {
            config,
            mode,
            store,
            embedder,
            registry,
        })
    }

    /// Mine a directory
    pub async fn mine_directory<F>(&mut self, dir: &Path, progress: F) -> Result<MineStats>
    where
        F: Fn(&str) + Send + Sync,
    {
        let mut stats = MineStats::default();

        // Clone the mode to avoid borrowing issues
        let mode = self.mode.clone();
        
        match mode {
            MineMode::Projects => {
                self.mine_projects(dir, &mut stats, progress).await?;
            }
            MineMode::Conversations => {
                self.mine_conversations(dir, &mut stats, progress).await?;
            }
            MineMode::General(extract) => {
                self.mine_general(dir, extract.as_deref(), &mut stats, progress).await?;
            }
        }

        // Save registry
        let registry_path = self.config.palace_path.join("entities.json");
        self.registry.save(&registry_path).await?;

        Ok(stats)
    }

    /// Mine project files
    async fn mine_projects<F>(&mut self, dir: &Path, stats: &mut MineStats, progress: F) -> Result<()>
    where
        F: Fn(&str),
    {
        progress("Scanning project files...");

        // Detect entities first
        let files = scan_for_detection(dir, &self.config).await?;
        let detected = detect_entities(&files, &self.config).await?;
        self.registry.merge_detected(&detected);

        // Walk directory
        for entry in WalkDir::new(dir)
            .follow_links(false)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_file() {
                stats.files_scanned += 1;

                // Check if should process
                if !self.should_process_file(path) {
                    continue;
                }

                progress(&format!("Processing: {}", path.display()));

                match self.process_file(path).await {
                    Ok(doc) => {
                        stats.files_processed += 1;
                        stats.documents_created += 1;
                        stats.total_bytes += doc.content.len() as u64;

                        // Add to store
                        self.store.add(doc)?;
                    }
                    Err(e) => {
                        warn!("Failed to process {}: {}", path.display(), e);
                        stats.errors += 1;
                    }
                }
            }
        }

        Ok(())
    }

    /// Mine conversation exports
    async fn mine_conversations<F>(&mut self, dir: &Path, stats: &mut MineStats, progress: F) -> Result<()>
    where
        F: Fn(&str),
    {
        progress("Scanning conversation files...");

        let convo_miner = ConversationMiner::new(&self.config);

        for entry in WalkDir::new(dir)
            .follow_links(false)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_file() {
                stats.files_scanned += 1;

                progress(&format!("Processing: {}", path.display()));

                match convo_miner.mine_file(path).await {
                    Ok(docs) => {
                        stats.files_processed += 1;
                        stats.documents_created += docs.len();

                        for doc in docs {
                            stats.total_bytes += doc.content.len() as u64;
                            self.store.add(doc)?;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to process {}: {}", path.display(), e);
                        stats.errors += 1;
                    }
                }
            }
        }

        Ok(())
    }

    /// Mine in general mode
    async fn mine_general<F>(
        &mut self,
        dir: &Path,
        extract: Option<&str>,
        stats: &mut MineStats,
        progress: F,
    ) -> Result<()>
    where
        F: Fn(&str),
    {
        progress("Mining in general mode...");

        let extractor = GeneralExtractor::new(extract);

        for entry in WalkDir::new(dir)
            .follow_links(false)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_file() {
                stats.files_scanned += 1;

                if !self.should_process_file(path) {
                    continue;
                }

                progress(&format!("Processing: {}", path.display()));

                match extractor.extract_file(path).await {
                    Ok(docs) => {
                        stats.files_processed += 1;
                        stats.documents_created += docs.len();

                        for doc in docs {
                            stats.total_bytes += doc.content.len() as u64;
                            self.store.add(doc)?;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to process {}: {}", path.display(), e);
                        stats.errors += 1;
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a file should be processed
    fn should_process_file(&self, path: &Path) -> bool {
        // Check file name
        if let Some(name) = path.file_name() {
            let name = name.to_string_lossy();
            if self.config.should_exclude_file(&name) {
                return false;
            }
        }

        // Check extension
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if !self.config.should_include_extension(&ext) {
                return false;
            }
        } else {
            return false;
        }

        // Check parent directories
        for ancestor in path.ancestors().skip(1).take(5) {
            if let Some(dir_name) = ancestor.file_name() {
                let dir_name = dir_name.to_string_lossy();
                if self.config.should_exclude_dir(&dir_name) {
                    return false;
                }
            }
        }

        true
    }

    /// Process a single file
    async fn process_file(&mut self, path: &Path) -> Result<Document> {
        let content = fs::read_to_string(path).await?;

        // Build metadata
        let mut metadata = HashMap::new();

        // Extract wing from path
        if let Some(wing) = self.infer_wing_from_path(path) {
            metadata.insert("wing".to_string(), wing);
        }

        // Extract room from path
        if let Some(room) = self.infer_room_from_path(path) {
            metadata.insert("room".to_string(), room);
        }

        metadata.insert("source_file".to_string(), path.to_string_lossy().to_string());
        metadata.insert(
            "file_type".to_string(),
            path.extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        Ok(Document::new(content, metadata))
    }

    /// Infer wing from file path
    fn infer_wing_from_path(&self, path: &Path) -> Option<String> {
        let path_str = path.to_string_lossy();

        // Check for project names in path
        for (name, project) in &self.registry.projects {
            if path_str.contains(&project.name) {
                return Some(project.name.clone());
            }
        }

        // Infer from parent directory
        if let Some(parent) = path.parent() {
            if let Some(dir_name) = parent.file_name() {
                return Some(dir_name.to_string_lossy().to_string());
            }
        }

        None
    }

    /// Infer room from file path
    fn infer_room_from_path(&self, path: &Path) -> Option<String> {
        let file_name = path.file_name()?.to_string_lossy();

        // Infer room from file name patterns
        if file_name.contains("README") || file_name.contains("readme") {
            return Some("documentation".to_string());
        }

        if file_name.contains("test") || file_name.contains("spec") {
            return Some("tests".to_string());
        }

        if file_name.contains("config") || file_name.contains("settings") {
            return Some("configuration".to_string());
        }

        // Infer from extension
        if let Some(ext) = path.extension() {
            match ext.to_string_lossy().as_ref() {
                "md" | "rst" | "txt" => return Some("documentation".to_string()),
                "rs" | "py" | "js" | "ts" => return Some("source".to_string()),
                "json" | "yaml" | "yml" | "toml" => return Some("configuration".to_string()),
                _ => {}
            }
        }

        None
    }
}
