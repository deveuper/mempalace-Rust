#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use std::path::Path;
use tokio::fs;

/// Layer 0: Identity
/// ~100 tokens, always loaded
/// Contains the AI's identity and basic context
pub struct Layer0 {
    identity_path: std::path::PathBuf,
    cached_content: Option<String>,
}

impl Layer0 {
    /// Create a new Layer0
    pub fn new<P: AsRef<Path>>(identity_path: P) -> Self {
        Self {
            identity_path: identity_path.as_ref().to_path_buf(),
            cached_content: None,
        }
    }

    /// Render the identity content
    pub fn render(&self) -> String {
        if let Some(ref content) = self.cached_content {
            return content.clone();
        }

        // Try to read from file
        if let Ok(content) = std::fs::read_to_string(&self.identity_path) {
            return content.trim().to_string();
        }

        // Return default identity
        Self::default_identity()
    }

    /// Render with async file reading
    pub async fn render_async(&mut self) -> String {
        if let Some(ref content) = self.cached_content {
            return content.clone();
        }

        // Try to read from file
        if let Ok(content) = fs::read_to_string(&self.identity_path).await {
            let content = content.trim().to_string();
            self.cached_content = Some(content.clone());
            return content;
        }

        // Return default identity
        Self::default_identity()
    }

    /// Get default identity text
    fn default_identity() -> String {
        r#"I am an AI assistant with access to a memory palace.
I remember past conversations and can search through them.
I help users recall information, decisions, and context from their work."#
            .to_string()
    }

    /// Estimate token count (rough approximation)
    pub fn estimate_tokens(&self) -> usize {
        let content = self.render();
        // Rough estimate: 1 token ≈ 4 characters for English
        content.len() / 4
    }

    /// Check if identity file exists
    pub fn exists(&self) -> bool {
        self.identity_path.exists()
    }

    /// Get the identity file path
    pub fn path(&self) -> &Path {
        &self.identity_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_identity() {
        let layer0 = Layer0::new("/nonexistent");
        let identity = layer0.render();
        assert!(!identity.is_empty());
        assert!(identity.contains("AI assistant"));
    }
}
