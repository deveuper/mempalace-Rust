#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::miner::{MineMode, Miner};

/// Handle the mine command
pub async fn handle_mine(
    config: &Config,
    dir: PathBuf,
    mode: &str,
    extract: Option<String>,
) -> Result<()> {
    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!("{}", "║           Mining into Palace              ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();

    let dir = dir.canonicalize().context("Failed to resolve directory path")?;

    let mine_mode = match mode {
        "projects" => MineMode::Projects,
        "convos" => MineMode::Conversations,
        "general" => MineMode::General(extract),
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown mode: {}. Use 'projects', 'convos', or 'general'",
                mode
            ));
        }
    };

    println!("📁 Directory: {}", dir.display().to_string().yellow());
    println!("📦 Mode: {}", mode.green());
    if let MineMode::General(Some(ref ext)) = mine_mode {
        println!("🔍 Extract: {}", ext.green());
    }
    println!();

    // Create progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    // Initialize miner
    pb.set_message("Initializing miner...");
    let mut miner = Miner::new(config.clone(), mine_mode).await?;

    // Mine the directory
    pb.set_message("Scanning files...");
    let stats = miner.mine_directory(&dir, |_msg: &str| {
        // Progress callback - currently disabled due to threading issues
        // pb.set_message(msg.to_string());
    }).await?;

    pb.finish_and_clear();

    // Print results
    println!("{}", "✅ Mining complete!".green().bold());
    println!();
    println!("Statistics:");
    println!("  📄 Files scanned: {}", stats.files_scanned.to_string().cyan());
    println!("  📄 Files processed: {}", stats.files_processed.to_string().cyan());
    println!("  📝 Documents created: {}", stats.documents_created.to_string().cyan());
    println!("  💾 Total bytes: {}", format_bytes(stats.total_bytes).cyan());
    println!();

    if stats.errors > 0 {
        println!("  ⚠️  Errors: {}", stats.errors.to_string().yellow());
    }

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}
