#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use regex::Regex;

pub mod normalize;
pub mod spellcheck;

/// Text processing utilities
pub struct TextUtils;

impl TextUtils {
    /// Clean and normalize text
    pub fn clean_text(text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Truncate text to a maximum length
    pub fn truncate(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            let truncate_at = text
                .char_indices()
                .take_while(|(i, _)| *i < max_len)
                .last()
                .map(|(i, _)| i)
                .unwrap_or(max_len);

            format!("{}...", &text[..truncate_at])
        }
    }

    /// Count words in text
    pub fn word_count(text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Estimate token count (rough approximation)
    pub fn estimate_tokens(text: &str) -> usize {
        // Rough estimate: 1 token ≈ 4 characters for English
        text.len() / 4
    }

    /// Extract sentences from text
    pub fn extract_sentences(text: &str) -> Vec<String> {
        let sentence_regex = Regex::new(r"[.!?]+\s+").unwrap();
        sentence_regex
            .split(text)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Extract paragraphs from text
    pub fn extract_paragraphs(text: &str) -> Vec<String> {
        text.split("\n\n")
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect()
    }

    /// Check if text is likely code
    pub fn is_code(text: &str) -> bool {
        let code_indicators = [
            "fn ", "def ", "class ", "import ", "use ", "let ", "const ",
            "var ", "function ", "=>", "::", "->", "{}", "[]", "();",
        ];

        code_indicators
            .iter()
            .any(|indicator| text.contains(indicator))
    }

    /// Detect language from content
    pub fn detect_language(text: &str) -> Option<String> {
        if text.contains("fn ") && text.contains("{") {
            Some("rust".to_string())
        } else if text.contains("def ") && text.contains(":") {
            Some("python".to_string())
        } else if text.contains("function ") || text.contains("const ") || text.contains("let ") {
            Some("javascript".to_string())
        } else if text.contains("package ") || text.contains("import ") && text.contains(";") {
            Some("java".to_string())
        } else {
            None
        }
    }
}

/// File utilities
pub struct FileUtils;

impl FileUtils {
    /// Get file extension
    pub fn extension(path: &std::path::Path) -> Option<String> {
        path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
    }

    /// Check if file is text
    pub fn is_text_file(path: &std::path::Path) -> bool {
        let text_extensions: std::collections::HashSet<&str> = [
            "txt", "md", "rst", "rs", "py", "js", "ts", "go", "java", "cpp", "c", "h",
            "hpp", "rb", "php", "swift", "kt", "scala", "r", "m", "mm", "sql", "sh",
            "bash", "zsh", "fish", "ps1", "bat", "cmd", "json", "yaml", "yml", "toml",
            "xml", "html", "css", "scss", "sass", "less", "vue", "jsx", "tsx",
        ]
        .iter()
        .cloned()
        .collect();

        if let Some(ext) = Self::extension(path) {
            text_extensions.contains(ext.as_str())
        } else {
            false
        }
    }

    /// Format file size
    pub fn format_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut s = size as f64;
        let mut unit_idx = 0;

        while s >= 1024.0 && unit_idx < UNITS.len() - 1 {
            s /= 1024.0;
            unit_idx += 1;
        }

        format!("{:.2} {}", s, UNITS[unit_idx])
    }
}

/// Hash utilities
pub struct HashUtils;

impl HashUtils {
    /// Compute Blake3 hash of content
    pub fn blake3(content: &[u8]) -> String {
        blake3::hash(content).to_hex().to_string()
    }

    /// Compute XXH3 hash of content
    pub fn xxh3(content: &[u8]) -> u64 {
        use xxhash_rust::xxh3::xxh3_64;
        xxh3_64(content)
    }
}
