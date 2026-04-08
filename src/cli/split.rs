#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::path::PathBuf;
use tokio::fs;

use crate::config::Config;

/// Handle the split command
pub async fn handle_split(
    _config: &Config,
    file: PathBuf,
    output: Option<PathBuf>,
) -> Result<()> {
    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!("{}", "║      Split Mega-Files                     ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();

    let content = fs::read_to_string(&file)
        .await
        .context("Failed to read input file")?;

    println!("📄 Input: {}", file.display().to_string().yellow());
    println!("📊 Size: {} characters", content.len().to_string().cyan());
    println!();

    // Determine output directory
    let output_dir = output.unwrap_or_else(|| {
        let mut path = file.clone();
        path.set_extension("");
        path
    });

    // Create output directory
    fs::create_dir_all(&output_dir).await?;

    // Split by session markers
    // Common patterns for conversation exports
    let session_patterns = [
        // Claude exports
        r"(?m)^={50,}\s*\n",
        // ChatGPT exports
        r"(?m)^Conversation with",
        // Generic date patterns
        r"(?m)^\d{4}-\d{2}-\d{2}[T ]",
        // Slack exports
        r"(?m)^\[\d{4}-\d{2}-\d{2}\]",
    ];

    let mut sessions: Vec<String> = Vec::new();
    let mut current_session = String::new();

    for line in content.lines() {
        let is_new_session = session_patterns.iter().any(|pattern| {
            Regex::new(pattern).unwrap().is_match(line)
        });

        if is_new_session && !current_session.is_empty() {
            sessions.push(current_session.clone());
            current_session.clear();
        }

        current_session.push_str(line);
        current_session.push('\n');
    }

    // Don't forget the last session
    if !current_session.is_empty() {
        sessions.push(current_session);
    }

    // If no sessions were detected, split by size
    if sessions.len() <= 1 && content.len() > 10000 {
        println!("  No session markers found. Splitting by size...");
        sessions = split_by_size(&content, 5000);
    }

    // Write sessions to files
    let file_stem = file.file_stem().unwrap_or_default().to_string_lossy();

    for (idx, session) in sessions.iter().enumerate() {
        let output_file = output_dir.join(format!("{}_{:04}.txt", file_stem, idx + 1));
        fs::write(&output_file, session).await?;
    }

    println!("{}", format!("✅ Split into {} files!", sessions.len()).green().bold());
    println!();
    println!("📁 Output directory: {}", output_dir.display().to_string().yellow());

    Ok(())
}

fn split_by_size(content: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for line in content.lines() {
        if current_chunk.len() + line.len() > chunk_size && !current_chunk.is_empty() {
            chunks.push(current_chunk.clone());
            current_chunk.clear();
        }
        current_chunk.push_str(line);
        current_chunk.push('\n');
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}
