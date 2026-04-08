#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, MultiSelect};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

use crate::config::{Config, PalaceMetadata};
use crate::entity::{detect_entities, scan_for_detection};
use crate::room::detect_rooms_local;

/// Initialize a new palace for a directory
pub async fn handle_init(config: &Config, dir: PathBuf, yes: bool) -> Result<()> {
    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!("{}", "║        MemPalace Initialization           ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();

    let dir = dir.canonicalize().context("Failed to resolve directory path")?;

    println!("📁 Scanning directory: {}", dir.display().to_string().yellow());
    println!();

    // Phase 1: Detect entities (people and projects)
    println!("{}", "Phase 1: Detecting entities...".blue().bold());

    let files = scan_for_detection(&dir, config).await?;
    println!("  Found {} files to analyze", files.len());

    let detected = detect_entities(&files, config).await?;

    println!();
    println!("  Detected entities:");
    println!("    👤 People: {}", detected.people.len());
    for person in &detected.people {
        println!("       - {}", person.green());
    }

    println!("    📂 Projects: {}", detected.projects.len());
    for project in &detected.projects {
        println!("       - {}", project.green());
    }

    if !detected.uncertain.is_empty() {
        println!("    ❓ Uncertain: {}", detected.uncertain.len());
    }

    // Confirm entities with user
    let confirmed = if yes {
        detected
    } else {
        confirm_entities(detected)?
    };

    // Save entities to file
    if !confirmed.people.is_empty() || !confirmed.projects.is_empty() {
        let entities_path = dir.join("entities.json");
        let entities_json = serde_json::to_string_pretty(&confirmed)?;
        fs::write(&entities_path, entities_json).await?;
        println!();
        println!("  💾 Entities saved to: {}", entities_path.display().to_string().cyan());
    }

    // Phase 2: Detect rooms from folder structure
    println!();
    println!("{}", "Phase 2: Detecting rooms from folder structure...".blue().bold());

    let rooms = detect_rooms_local(&dir, config).await?;

    println!("  Found {} rooms:", rooms.len());
    for room in &rooms {
        println!("    🚪 {}", room.name.green());
    }

    // Save rooms to file
    let rooms_path = dir.join("rooms.json");
    let rooms_json = serde_json::to_string_pretty(&rooms)?;
    fs::write(&rooms_path, rooms_json).await?;
    println!();
    println!("  💾 Rooms saved to: {}", rooms_path.display().to_string().cyan());

    // Create palace metadata
    let mut metadata = PalaceMetadata::default();
    metadata.wings = confirmed.projects.clone();
    metadata.wings.extend(confirmed.people.clone());

    let metadata_path = config.palace_path.join(format!(
        "metadata_{}.json",
        dir.file_name()
            .unwrap_or_default()
            .to_string_lossy()
    ));
    metadata.save(&metadata_path).await?;

    println!();
    println!("{}", "✅ Initialization complete!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Run: {} {}",
        "mempalace mine".yellow(),
        dir.display().to_string().cyan()
    );
    println!("  2. Search: {}",
        "mempalace search \"your query\"".yellow()
    );

    Ok(())
}

/// Confirm detected entities with the user
fn confirm_entities(detected: crate::entity::DetectedEntities) -> Result<crate::entity::DetectedEntities> {
    let mut confirmed_people = Vec::new();
    let mut confirmed_projects = Vec::new();

    // Confirm people
    if !detected.people.is_empty() {
        println!();
        println!("{}", "Confirm people:".yellow().bold());

        let selections = MultiSelect::new()
            .items(&detected.people)
            .defaults(&vec![true; detected.people.len()])
            .interact()?;

        for idx in selections {
            confirmed_people.push(detected.people[idx].clone());
        }
    }

    // Confirm projects
    if !detected.projects.is_empty() {
        println!();
        println!("{}", "Confirm projects:".yellow().bold());

        let selections = MultiSelect::new()
            .items(&detected.projects)
            .defaults(&vec![true; detected.projects.len()])
            .interact()?;

        for idx in selections {
            confirmed_projects.push(detected.projects[idx].clone());
        }
    }

    // Handle uncertain items
    if !detected.uncertain.is_empty() {
        println!();
        println!("{}", "Review uncertain items:".yellow().bold());

        for item in &detected.uncertain {
            let is_project = Confirm::new()
                .with_prompt(format!("Is '{}' a project? (No = person)", item))
                .default(true)
                .interact()?;

            if is_project {
                confirmed_projects.push(item.clone());
            } else {
                confirmed_people.push(item.clone());
            }
        }
    }

    Ok(crate::entity::DetectedEntities {
        people: confirmed_people,
        projects: confirmed_projects,
        uncertain: Vec::new(),
    })
}
