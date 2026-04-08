#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;

use crate::config::Config;
use crate::entity::{DetectedEntities, FileInfo};

/// Detect entities (people and projects) from file content
pub async fn detect_entities(files: &[FileInfo], _config: &Config) -> Result<DetectedEntities> {
    let mut people = HashSet::new();
    let mut projects = HashSet::new();
    let mut uncertain = HashSet::new();

    // Common patterns for detecting people
    let people_patterns = [
        // Email addresses
        Regex::new(r"\b([A-Za-z][A-Za-z0-9._-]*)@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap(),
        // @mentions
        Regex::new(r"@([A-Za-z][A-Za-z0-9_-]+)").unwrap(),
        // "Name said/wrote/thinks"
        Regex::new(r"(?i)\b([A-Z][a-z]+)\s+(?:said|wrote|thinks|suggests|proposed)").unwrap(),
    ];

    // Common patterns for detecting projects
    let project_patterns = [
        // GitHub repo references
        Regex::new(r"github\.com/[^/]+/([^/\s]+)").unwrap(),
        // Project names in quotes
        Regex::new(r#""([^"]{3,30})"\s+(?:project|app|system|tool)"#).unwrap(),
        // "the X project"
        Regex::new(r"(?i)the\s+([A-Z][A-Za-z]+)\s+project").unwrap(),
    ];

    // Directory name patterns
    let dir_name_patterns = [
        Regex::new(r"^[a-z]+-[a-z]+$").unwrap(),     // kebab-case
        Regex::new(r"^[a-z]+_[a-z]+$").unwrap(),     // snake_case
        Regex::new(r"^[A-Z][a-z]+[A-Z][a-z]+$").unwrap(), // CamelCase
    ];

    for file in files {
        let content = &file.content;
        let path_str = file.path.to_string_lossy();

        // Extract directory name as potential project
        if let Some(parent) = file.path.parent() {
            if let Some(dir_name) = parent.file_name() {
                let dir_name = dir_name.to_string_lossy();
                if dir_name_patterns.iter().any(|p| p.is_match(&dir_name)) {
                    if dir_name.len() > 2 && dir_name.len() < 30 {
                        projects.insert(dir_name.to_string());
                    }
                }
            }
        }

        // Extract from content using patterns
        for pattern in &people_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(name) = cap.get(1) {
                    let name = name.as_str().to_string();
                    if is_likely_name(&name) {
                        people.insert(name);
                    }
                }
            }
        }

        for pattern in &project_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(proj) = cap.get(1) {
                    let proj = proj.as_str().to_string();
                    if is_likely_project(&proj) {
                        projects.insert(proj);
                    }
                }
            }
        }

        // Extract from file path
        if path_str.contains("/projects/") || path_str.contains("/repos/") {
            if let Some(proj) = extract_project_from_path(&path_str) {
                projects.insert(proj);
            }
        }
    }

    // Filter out common false positives
    let false_positive_names: HashSet<&str> = [
        "the", "this", "that", "these", "those", "http", "https", "www",
        "com", "org", "net", "io", "app", "api", "src", "lib", "bin",
        "test", "tests", "docs", "doc", "examples", "example",
    ].iter().cloned().collect();

    people.retain(|p| !false_positive_names.contains(p.to_lowercase().as_str()));
    projects.retain(|p| !false_positive_names.contains(p.to_lowercase().as_str()));

    // Items that could be either
    for item in people.intersection(&projects) {
        uncertain.insert(item.clone());
    }

    people.retain(|p| !uncertain.contains(p));
    projects.retain(|p| !uncertain.contains(p));

    Ok(DetectedEntities {
        people: people.into_iter().collect(),
        projects: projects.into_iter().collect(),
        uncertain: uncertain.into_iter().collect(),
    })
}

fn is_likely_name(s: &str) -> bool {
    if s.len() < 2 || s.len() > 30 {
        return false;
    }

    // Must start with uppercase
    if !s.chars().next().unwrap().is_uppercase() {
        return false;
    }

    // Should not be all uppercase (likely acronym)
    if s.chars().all(|c| c.is_uppercase()) {
        return false;
    }

    // Should not contain numbers
    if s.chars().any(|c| c.is_numeric()) {
        return false;
    }

    true
}

fn is_likely_project(s: &str) -> bool {
    if s.len() < 3 || s.len() > 40 {
        return false;
    }

    // Should not be a common word
    let common_words: HashSet<&str> = [
        "the", "and", "for", "are", "but", "not", "you", "all", "can",
        "had", "her", "was", "one", "our", "out", "day", "get", "has",
    ].iter().cloned().collect();

    if common_words.contains(s.to_lowercase().as_str()) {
        return false;
    }

    true
}

fn extract_project_from_path(path: &str) -> Option<String> {
    // Extract project name from path like /home/user/projects/my-project
    let parts: Vec<&str> = path.split('/').collect();

    for (i, part) in parts.iter().enumerate() {
        if *part == "projects" || *part == "repos" || *part == "src" {
            if i + 1 < parts.len() {
                return Some(parts[i + 1].to_string());
            }
        }
    }

    None
}
