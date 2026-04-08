#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::Config;

pub mod detector;
pub mod registry;

pub use detector::detect_entities;
pub use registry::EntityRegistry;

/// Detected entities from file scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedEntities {
    pub people: Vec<String>,
    pub projects: Vec<String>,
    pub uncertain: Vec<String>,
}

impl DetectedEntities {
    pub fn new() -> Self {
        Self {
            people: Vec::new(),
            projects: Vec::new(),
            uncertain: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.people.is_empty() && self.projects.is_empty() && self.uncertain.is_empty()
    }
}

impl Default for DetectedEntities {
    fn default() -> Self {
        Self::new()
    }
}

/// File info for entity detection
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub content: String,
    pub size: u64,
}

/// Scan directory for files to analyze
pub async fn scan_for_detection(dir: &Path, config: &Config) -> Result<Vec<FileInfo>> {
    let mut files = Vec::new();
    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_file() {
            // Check file extension
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if config.should_include_extension(&ext) {
                    // Check file size
                    let metadata = entry.metadata().await?;
                    let size = metadata.len();

                    if size <= config.mining.max_file_size as u64 {
                        // Read content
                        match tokio::fs::read_to_string(&path).await {
                            Ok(content) => {
                                files.push(FileInfo {
                                    path,
                                    content,
                                    size,
                                });
                            }
                            Err(_) => {
                                // Skip binary files
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(files)
}
