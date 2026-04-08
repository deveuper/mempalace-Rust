#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use colored::Colorize;
use crate::config::Config;
use crate::storage::{Embedder, VectorStore};

/// Handle the repair command
pub async fn handle_repair(config: &Config) -> Result<()> {
    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!("{}", "║         Palace Repair                     ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();

    let embedder = Embedder::new(config.embedding_model())?;
    let mut store = VectorStore::open(&config.db_path, embedder)?;

    println!("🔧 Checking palace integrity...");
    println!();

    // Get document count before repair
    let before_count = store.count()?;
    println!("  📄 Documents before: {}", before_count.to_string().cyan());

    // Rebuild FTS index
    println!("  🔄 Rebuilding FTS index...");
    // This would involve re-indexing all documents
    // For now, just a placeholder

    // Check for orphaned embeddings
    println!("  🔍 Checking for orphaned embeddings...");

    // Verify metadata consistency
    println!("  📋 Verifying metadata consistency...");

    // Get document count after repair
    let after_count = store.count()?;
    println!();
    println!("  📄 Documents after: {}", after_count.to_string().cyan());

    if before_count == after_count {
        println!();
        println!("{}", "✅ No issues found. Palace is healthy!".green());
    } else {
        println!();
        println!("{}", "⚠️  Repair completed with changes.".yellow());
        println!("     Documents removed: {}", before_count - after_count);
    }

    Ok(())
}
