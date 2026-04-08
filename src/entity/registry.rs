#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use crate::entity::DetectedEntities;

/// Entity registry for tracking people and projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRegistry {
    pub people: HashMap<String, Person>,
    pub projects: HashMap<String, Project>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub aliases: Vec<String>,
    pub email: Option<String>,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub mentions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub aliases: Vec<String>,
    pub path: Option<String>,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub documents: usize,
}

impl EntityRegistry {
    pub fn new() -> Self {
        Self {
            people: HashMap::new(),
            projects: HashMap::new(),
            version: "3.0.0".to_string(),
        }
    }

    pub async fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path).await?;
            let registry: EntityRegistry = serde_json::from_str(&content)?;
            Ok(registry)
        } else {
            Ok(Self::new())
        }
    }

    pub async fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content).await?;
        Ok(())
    }

    pub fn add_person(&mut self, name: &str) {
        let now = chrono::Utc::now();
        let key = name.to_lowercase();

        self.people
            .entry(key.clone())
            .and_modify(|p| {
                p.last_seen = now;
                p.mentions += 1;
            })
            .or_insert(Person {
                name: name.to_string(),
                aliases: Vec::new(),
                email: None,
                first_seen: now,
                last_seen: now,
                mentions: 1,
            });
    }

    pub fn add_project(&mut self, name: &str, path: Option<String>) {
        let now = chrono::Utc::now();
        let key = name.to_lowercase();

        self.projects
            .entry(key.clone())
            .and_modify(|p| {
                p.last_seen = now;
                p.documents += 1;
            })
            .or_insert(Project {
                name: name.to_string(),
                aliases: Vec::new(),
                path,
                first_seen: now,
                last_seen: now,
                documents: 1,
            });
    }

    pub fn merge_detected(&mut self, detected: &DetectedEntities) {
        for person in &detected.people {
            self.add_person(person);
        }

        for project in &detected.projects {
            self.add_project(project, None);
        }
    }

    pub fn get_person(&self, name: &str) -> Option<&Person> {
        self.people.get(&name.to_lowercase())
    }

    pub fn get_project(&self, name: &str) -> Option<&Project> {
        self.projects.get(&name.to_lowercase())
    }

    pub fn search_people(&self, query: &str) -> Vec<&Person> {
        let query_lower = query.to_lowercase();
        self.people
            .values()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.aliases.iter().any(|a| a.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn search_projects(&self, query: &str) -> Vec<&Project> {
        let query_lower = query.to_lowercase();
        self.projects
            .values()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.aliases.iter().any(|a| a.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
}

impl Default for EntityRegistry {
    fn default() -> Self {
        Self::new()
    }
}
