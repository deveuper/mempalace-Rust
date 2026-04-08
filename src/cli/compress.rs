#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use tokio::fs;

use crate::config::Config;
use crate::dialect::{AaakCompressor, CompressionStats};

/// Handle the compress command
pub async fn handle_compress(
    _config: &Config,
    file: PathBuf,
    output: Option<PathBuf>,
) -> Result<()> {
    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!("{}", "║         AAAK Compression                  ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();

    let content = fs::read_to_string(&file)
        .await
        .context("Failed to read input file")?;

    println!("📄 Input: {}", file.display().to_string().yellow());
    println!("📊 Original size: {} characters", content.len().to_string().cyan());
    println!();

    // Compress using AAAK
    let compressor = AaakCompressor::new();
    let (compressed, stats) = compressor.compress(&content)?;

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let mut path = file.clone();
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        path.set_file_name(format!("{}_aaak.txt", stem));
        path
    });

    // Write output
    fs::write(&output_path, &compressed).await?;

    println!("{}", "✅ Compression complete!".green().bold());
    println!();
    println!("📄 Output: {}", output_path.display().to_string().yellow());
    println!("📊 Compressed size: {} characters", compressed.len().to_string().cyan());
    println!();
    println!("Compression statistics:");
    println!("  📉 Ratio: {:.1}%", stats.compression_ratio);
    println!("  📝 Entities encoded: {}", stats.entities_encoded);
    println!("  ✂️  Sentences truncated: {}", stats.sentences_truncated);
    println!("  🏷️  Emotion codes: {}", stats.emotion_codes);

    Ok(())
}
