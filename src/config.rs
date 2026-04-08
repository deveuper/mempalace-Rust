#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::info;

use crate::storage::EmbeddingModel;

/// Default palace directory name
const DEFAULT_PALACE_DIR: &str = ".mempalace";

/// Configuration file name
const CONFIG_FILE: &str = "config.toml";

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Palace root directory
    #[serde(skip)]
    pub palace_path: PathBuf,

    /// Vector database path
    #[serde(skip)]
    pub db_path: PathBuf,

    /// Embedding model configuration
    pub embedding: EmbeddingConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Mining configuration
    pub mining: MiningConfig,

    /// Layer configuration
    pub layers: LayerConfig,

    /// MCP server configuration
    pub mcp: McpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model: String,
    pub dimensions: usize,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Maximum document size in bytes
    pub max_doc_size: usize,
    /// Chunk size for large documents
    pub chunk_size: usize,
    /// Chunk overlap
    pub chunk_overlap: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    /// File extensions to include
    pub include_extensions: Vec<String>,
    /// Directories to exclude
    pub exclude_dirs: Vec<String>,
    /// Files to exclude
    pub exclude_files: Vec<String>,
    /// Maximum file size to process (bytes)
    pub max_file_size: usize,
    /// Enable conversation mining
    pub enable_convo_mining: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfig {
    /// Layer 0: Identity file path
    pub identity_file: PathBuf,
    /// Layer 1: Number of top moments to keep
    pub top_moments_count: usize,
    /// Layer 2: On-demand context size
    pub on_demand_context_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub enabled: bool,
    pub transport: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        let palace_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(DEFAULT_PALACE_DIR);

        Self {
            palace_path: palace_path.clone(),
            db_path: palace_path.join("palace.db"),
            embedding: EmbeddingConfig {
                model: "all-MiniLM-L6-v2".to_string(),
                dimensions: 384,
                batch_size: 32,
            },
            storage: StorageConfig {
                max_doc_size: 1024 * 1024, // 1MB
                chunk_size: 512,
                chunk_overlap: 128,
            },
            mining: MiningConfig {
                include_extensions: vec![
                    "txt".to_string(),
                    "md".to_string(),
                    "rs".to_string(),
                    "py".to_string(),
                    "js".to_string(),
                    "ts".to_string(),
                    "json".to_string(),
                    "yaml".to_string(),
                    "yml".to_string(),
                    "toml".to_string(),
                ],
                exclude_dirs: vec![
                    "node_modules".to_string(),
                    "target".to_string(),
                    ".git".to_string(),
                    "__pycache__".to_string(),
                    ".venv".to_string(),
                    "venv".to_string(),
                    "dist".to_string(),
                    "build".to_string(),
                ],
                exclude_files: vec![
                    ".gitignore".to_string(),
                    ".DS_Store".to_string(),
                    "package-lock.json".to_string(),
                    "Cargo.lock".to_string(),
                ],
                max_file_size: 10 * 1024 * 1024, // 10MB
                enable_convo_mining: true,
            },
            layers: LayerConfig {
                identity_file: palace_path.join("identity.txt"),
                top_moments_count: 20,
                on_demand_context_size: 500,
            },
            mcp: McpConfig {
                enabled: true,
                transport: "stdio".to_string(),
                port: 8080,
            },
        }
    }
}

impl Config {
    /// Load configuration from the specified palace path or default location
    pub async fn load(palace_path: Option<PathBuf>) -> Result<Self> {
        let palace_path = palace_path.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(DEFAULT_PALACE_DIR)
        });

        let config_path = palace_path.join(CONFIG_FILE);

        let mut config = if config_path.exists() {
            info!("Loading configuration from {:?}", config_path);
            let content = fs::read_to_string(&config_path).await?;
            let mut config: Config = toml::from_str(&content)?;
            config.palace_path = palace_path.clone();
            config.db_path = palace_path.join("palace.db");
            config.layers.identity_file = palace_path.join("identity.txt");
            config
        } else {
            info!("Creating default configuration");
            let mut config = Config::default();
            config.palace_path = palace_path.clone();
            config.db_path = palace_path.join("palace.db");
            config.layers.identity_file = palace_path.join("identity.txt");
            config
        };

        // Ensure palace directory exists
        fs::create_dir_all(&config.palace_path).await?;

        Ok(config)
    }

    /// Save configuration to disk
    pub async fn save(&self) -> Result<()> {
        let config_path = self.palace_path.join(CONFIG_FILE);
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content).await?;
        info!("Configuration saved to {:?}", config_path);
        Ok(())
    }

    /// Get the embedding model
    pub fn embedding_model(&self) -> EmbeddingModel {
        match self.embedding.model.as_str() {
            "all-MiniLM-L6-v2" => EmbeddingModel::AllMiniLmL6V2,
            "all-MiniLM-L12-v2" => EmbeddingModel::AllMiniLmL12V2,
            "all-mpnet-base-v2" => EmbeddingModel::AllMpNetBaseV2,
            _ => EmbeddingModel::Custom {
                name: self.embedding.model.clone(),
                dimensions: self.embedding.dimensions,
            },
        }
    }

    /// Get the palace metadata path
    pub fn metadata_path(&self) -> PathBuf {
        self.palace_path.join("metadata.json")
    }

    /// Get the entities file path for a project
    pub fn entities_path(&self, project_dir: &Path) -> PathBuf {
        project_dir.join("entities.json")
    }

    /// Get the rooms file path for a project
    pub fn rooms_path(&self, project_dir: &Path) -> PathBuf {
        project_dir.join("rooms.json")
    }

    /// Check if a file extension should be included
    pub fn should_include_extension(&self, ext: &str) -> bool {
        self.mining.include_extensions.contains(&ext.to_lowercase())
    }

    /// Check if a directory should be excluded
    pub fn should_exclude_dir(&self, dir: &str) -> bool {
        self.mining.exclude_dirs.contains(&dir.to_string())
    }

    /// Check if a file should be excluded
    pub fn should_exclude_file(&self, file: &str) -> bool {
        self.mining.exclude_files.contains(&file.to_string())
    }
}

/// Palace metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceMetadata {
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_mined: Option<chrono::DateTime<chrono::Utc>>,
    pub document_count: usize,
    pub wings: Vec<String>,
    pub rooms: HashMap<String, Vec<String>>,
}

impl Default for PalaceMetadata {
    fn default() -> Self {
        Self {
            version: "3.0.0".to_string(),
            created_at: chrono::Utc::now(),
            last_mined: None,
            document_count: 0,
            wings: Vec::new(),
            rooms: HashMap::new(),
        }
    }
}

impl PalaceMetadata {
    /// Load metadata from disk
    pub async fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path).await?;
            let metadata: PalaceMetadata = serde_json::from_str(&content)?;
            Ok(metadata)
        } else {
            Ok(Self::default())
        }
    }

    /// Save metadata to disk
    pub async fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content).await?;
        Ok(())
    }
}
