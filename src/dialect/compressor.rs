#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;

use crate::dialect::AaakDialect;

/// Compression statistics
#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f32,
    pub entities_encoded: usize,
    pub sentences_truncated: usize,
    pub emotion_codes: usize,
}

/// AAAK Compressor
pub struct AaakCompressor {
    dialect: AaakDialect,
}

impl AaakCompressor {
    /// Create a new compressor
    pub fn new() -> Self {
        Self {
            dialect: AaakDialect::new(),
        }
    }

    /// Compress text using AAAK dialect
    pub fn compress(&self, text: &str) -> Result<(String, CompressionStats)> {
        let original_size = text.len();

        let mut dialect = AaakDialect::new();
        let compressed = dialect.encode(text)?;

        let compressed_size = compressed.len();
        let compression_ratio = if original_size > 0 {
            (1.0 - (compressed_size as f32 / original_size as f32)) * 100.0
        } else {
            0.0
        };

        // Count entities, sentences, emotions
        let entities_encoded = compressed.matches('|').count();
        let sentences_truncated = compressed.matches("...").count();
        let emotion_codes = compressed
            .lines()
            .filter(|l| l.contains("vul") || l.contains("joy") || l.contains("fear"))
            .count();

        let stats = CompressionStats {
            original_size,
            compressed_size,
            compression_ratio,
            entities_encoded,
            sentences_truncated,
            emotion_codes,
        };

        Ok((compressed, stats))
    }

    /// Decompress AAAK format (approximate)
    pub fn decompress(&self, compressed: &str) -> Result<String> {
        let mut output = String::new();

        for line in compressed.lines() {
            // Skip header
            if line.contains('|') && line.len() < 100 {
                continue;
            }

            // Extract key quote from zettel
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    let quote = &line[start + 1..start + 1 + end];
                    output.push_str(quote);
                    output.push(' ');
                }
            }
        }

        Ok(output.trim().to_string())
    }
}

impl Default for AaakCompressor {
    fn default() -> Self {
        Self::new()
    }
}
