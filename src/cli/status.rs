#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use colored::Colorize;
use crate::config::{Config, PalaceMetadata};
use crate::storage::{Embedder, VectorStore};

/// Handle the status command
pub async fn handle_status(config: &Config) -> Result<()> {
    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!("{}", "║           Palace Status                   ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();

    // Palace location
    println!("📁 Palace location: {}", config.palace_path.display().to_string().yellow());
    println!("💾 Database: {}", config.db_path.display().to_string().yellow());
    println!();

    // Load vector store stats
    let embedder = Embedder::new(config.embedding_model())?;
    let store = VectorStore::open(&config.db_path, embedder)?;

    let doc_count = store.count()?;
    let wings = store.get_wings()?;

    println!("{}", "Storage Statistics:".blue().bold());
    println!("  📝 Total documents: {}", doc_count.to_string().cyan());
    println!("  📂 Wings: {}", wings.len().to_string().cyan());

    if !wings.is_empty() {
        println!();
        println!("{}", "Wings and Rooms:".blue().bold());

        for wing in &wings {
            println!("  📂 {}", wing.green());

            let rooms = store.get_rooms(wing)?;
            for room in &rooms {
                println!("    🚪 {}", room.cyan());
            }
        }
    }

    // Load metadata
    let metadata_path = config.metadata_path();
    if metadata_path.exists() {
        let metadata = PalaceMetadata::load(&metadata_path).await?;
        println!();
        println!("{}", "Metadata:".blue().bold());
        println!("  🏷️  Version: {}", metadata.version.cyan());
        println!("  📅 Created: {}", metadata.created_at.to_rfc2822().cyan());
        if let Some(last_mined) = metadata.last_mined {
            println!("  ⏰ Last mined: {}", last_mined.to_rfc2822().cyan());
        }
    }

    println!();

    if doc_count == 0 {
        println!("{}", "⚠️  Palace is empty!".yellow());
        println!();
        println!("To start mining:");
        println!("  1. Initialize: {} <directory>", "mempalace init".cyan());
        println!("  2. Mine: {} <directory>", "mempalace mine".cyan());
    } else {
        println!("{}", "✅ Palace is ready for search!".green());
        println!();
        println!("Try: {}", "mempalace search \"your query\"".cyan());
    }

    Ok(())
}
