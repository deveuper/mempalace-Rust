#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use colored::Colorize;
use crate::config::Config;
use crate::storage::{Embedder, VectorStore};

/// Handle the search command
pub async fn handle_search(
    config: &Config,
    query: &str,
    wing: Option<String>,
    room: Option<String>,
    n: usize,
) -> Result<()> {
    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!("{}", "║           Searching Palace                ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();

    println!("🔍 Query: {}", query.yellow().bold());
    if let Some(ref w) = wing {
        println!("📂 Wing: {}", w.green());
    }
    if let Some(ref r) = room {
        println!("🚪 Room: {}", r.green());
    }
    println!();

    // Initialize vector store
    let embedder = Embedder::new(config.embedding_model())?;
    let mut store = VectorStore::open(&config.db_path, embedder)?;

    // Perform search
    let results = store.search(
        query,
        n,
        wing.as_deref(),
        room.as_deref(),
    )?;

    if results.is_empty() {
        println!("{}", "❌ No results found.".yellow());
        println!();
        println!("Tips:");
        println!("  • Try a different query");
        println!("  • Check if the palace has been mined: {}", "mempalace status".cyan());
        return Ok(());
    }

    println!("{}", format!("Found {} results:", results.len()).green().bold());
    println!();

    for (idx, result) in results.iter().enumerate() {
        let doc = &result.document;
        let score_pct = (result.score * 100.0).min(100.0);

        // Result header
        println!("{}", format!("┌─ Result {} ─{:.1}% match ───────────────┐", idx + 1, score_pct).cyan());

        // Metadata
        if let Some(w) = doc.metadata.get("wing") {
            println!("│ 📂 Wing: {}", w.green());
        }
        if let Some(r) = doc.metadata.get("room") {
            println!("│ 🚪 Room: {}", r.green());
        }
        if let Some(source) = doc.metadata.get("source_file") {
            println!("│ 📄 Source: {}", source.dimmed());
        }

        // Content preview
        println!("│");
        let content = &doc.content;
        let preview: String = content
            .lines()
            .take(10)
            .collect::<Vec<_>>()
            .join("\n│ ");
        println!("│ {}", preview);

        if content.lines().count() > 10 {
            println!("│ ... ({} more lines)", content.lines().count() - 10);
        }

        println!("{}", "└─────────────────────────────────────────┘".cyan());
        println!();
    }

    Ok(())
}
