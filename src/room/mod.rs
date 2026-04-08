#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

use crate::config::Config;

/// A room in the palace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub file_patterns: Vec<String>,
}

/// Detect rooms from local directory structure
pub async fn detect_rooms_local(dir: &Path, config: &Config) -> Result<Vec<Room>> {
    let mut rooms: Vec<Room> = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    // Walk directory and detect rooms from folder names
    for entry in WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_dir() {
            if let Some(dir_name) = path.file_name() {
                let dir_name = dir_name.to_string_lossy().to_string();

                // Skip excluded directories
                if config.should_exclude_dir(&dir_name) {
                    continue;
                }

                // Create room from directory name
                if !seen_names.contains(&dir_name) {
                    seen_names.insert(dir_name.clone());

                    let room = Room {
                        name: dir_name.clone(),
                        description: format!("Content from {} directory", dir_name),
                        keywords: extract_keywords_from_name(&dir_name),
                        file_patterns: vec!["*".to_string()],
                    };

                    rooms.push(room);
                }
            }
        }
    }

    // Add standard rooms based on file types found
    let standard_rooms = detect_standard_rooms(dir, config).await?;
    for room in standard_rooms {
        if !seen_names.contains(&room.name) {
            seen_names.insert(room.name.clone());
            rooms.push(room);
        }
    }

    // Sort by name
    rooms.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(rooms)
}

/// Detect standard rooms based on file types
async fn detect_standard_rooms(dir: &Path, _config: &Config) -> Result<Vec<Room>> {
    let mut rooms = Vec::new();
    let mut has_docs = false;
    let mut has_tests = false;
    let mut has_config = false;
    let mut has_source = false;

    for entry in WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                match ext.as_str() {
                    "md" | "rst" | "txt" => has_docs = true,
                    "test" | "spec" => has_tests = true,
                    "json" | "yaml" | "yml" | "toml" => has_config = true,
                    "rs" | "py" | "js" | "ts" | "go" | "java" | "cpp" | "c" => {
                        has_source = true
                    }
                    _ => {}
                }
            }
        }
    }

    if has_docs {
        rooms.push(Room {
            name: "documentation".to_string(),
            description: "Documentation and readme files".to_string(),
            keywords: vec!["docs".to_string(), "documentation".to_string(), "readme".to_string()],
            file_patterns: vec!["*.md".to_string(), "*.rst".to_string(), "*.txt".to_string()],
        });
    }

    if has_tests {
        rooms.push(Room {
            name: "tests".to_string(),
            description: "Test files and specifications".to_string(),
            keywords: vec!["test".to_string(), "spec".to_string(), "testing".to_string()],
            file_patterns: vec!["*test*".to_string(), "*spec*".to_string()],
        });
    }

    if has_config {
        rooms.push(Room {
            name: "configuration".to_string(),
            description: "Configuration files".to_string(),
            keywords: vec!["config".to_string(), "settings".to_string(), "setup".to_string()],
            file_patterns: vec!["*.json".to_string(), "*.yaml".to_string(), "*.toml".to_string()],
        });
    }

    if has_source {
        rooms.push(Room {
            name: "source".to_string(),
            description: "Source code files".to_string(),
            keywords: vec!["code".to_string(), "source".to_string(), "implementation".to_string()],
            file_patterns: vec!["*.rs".to_string(), "*.py".to_string(), "*.js".to_string()],
        });
    }

    Ok(rooms)
}

/// Extract keywords from a directory name
fn extract_keywords_from_name(name: &str) -> Vec<String> {
    let mut keywords = Vec::new();

    // Split by common separators
    let parts: Vec<&str> = name.split(&['-', '_', '.'][..]).collect();

    for part in parts {
        if part.len() > 2 {
            keywords.push(part.to_lowercase());
        }
    }

    keywords
}
