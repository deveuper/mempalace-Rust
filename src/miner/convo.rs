#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use crate::config::Config;
use crate::storage::Document;

/// Conversation miner for extracting data from conversation exports
pub struct ConversationMiner<'a> {
    config: &'a Config,
}

/// Claude conversation export format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeConversation {
    name: String,
    chat_messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeMessage {
    sender: String,
    text: String,
    #[serde(default)]
    timestamp: Option<String>,
}

/// ChatGPT conversation export format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatGPTConversation {
    title: String,
    #[serde(default)]
    mapping: HashMap<String, ChatGPTNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatGPTNode {
    message: Option<ChatGPTMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatGPTMessage {
    author: ChatGPTAuthor,
    content: ChatGPTContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatGPTAuthor {
    role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatGPTContent {
    parts: Vec<String>,
}

impl<'a> ConversationMiner<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Mine a conversation file
    pub async fn mine_file(&self, path: &Path) -> Result<Vec<Document>> {
        let content = fs::read_to_string(path).await?;

        // Try to detect format
        if content.contains("chat_messages") && content.contains("Claude") {
            self.parse_claude_export(&content, path).await
        } else if content.contains("\"mapping\"") {
            self.parse_chatgpt_export(&content, path).await
        } else {
            // Generic text conversation
            self.parse_generic_conversation(&content, path).await
        }
    }

    /// Parse Claude conversation export
    async fn parse_claude_export(&self, content: &str, path: &Path) -> Result<Vec<Document>> {
        let conversations: Vec<ClaudeConversation> = serde_json::from_str(content)
            .context("Failed to parse Claude conversation export")?;

        let mut documents = Vec::new();

        for convo in conversations {
            let mut full_text = String::new();
            full_text.push_str(&format!("# {}\n\n", convo.name));

            for msg in &convo.chat_messages {
                full_text.push_str(&format!("**{}**: {}\n\n", msg.sender, msg.text));
            }

            let mut metadata = HashMap::new();
            metadata.insert("source_file".to_string(), path.to_string_lossy().to_string());
            metadata.insert("conversation_name".to_string(), convo.name.clone());
            metadata.insert("format".to_string(), "claude".to_string());

            // Infer wing from conversation name
            if let Some(wing) = self.infer_wing_from_name(&convo.name) {
                metadata.insert("wing".to_string(), wing);
            }

            documents.push(Document::new(full_text, metadata));
        }

        Ok(documents)
    }

    /// Parse ChatGPT conversation export
    async fn parse_chatgpt_export(&self, content: &str, path: &Path) -> Result<Vec<Document>> {
        let conversations: Vec<ChatGPTConversation> = serde_json::from_str(content)
            .context("Failed to parse ChatGPT conversation export")?;

        let mut documents = Vec::new();

        for convo in conversations {
            let mut full_text = String::new();
            full_text.push_str(&format!("# {}\n\n", convo.title));

            // ChatGPT exports have a mapping structure
            for (id, node) in &convo.mapping {
                if let Some(msg) = &node.message {
                    let role = &msg.author.role;
                    let text = msg.content.parts.join("\n");
                    full_text.push_str(&format!("**{}**: {}\n\n", role, text));
                }
            }

            let mut metadata = HashMap::new();
            metadata.insert("source_file".to_string(), path.to_string_lossy().to_string());
            metadata.insert("conversation_name".to_string(), convo.title.clone());
            metadata.insert("format".to_string(), "chatgpt".to_string());

            // Infer wing from conversation title
            if let Some(wing) = self.infer_wing_from_name(&convo.title) {
                metadata.insert("wing".to_string(), wing);
            }

            documents.push(Document::new(full_text, metadata));
        }

        Ok(documents)
    }

    /// Parse generic conversation format
    async fn parse_generic_conversation(&self, content: &str, path: &Path) -> Result<Vec<Document>> {
        let mut documents = Vec::new();

        // Split by session markers
        let session_regex = Regex::new(r"(?m)^={50,}\s*$|^\[?\d{4}-\d{2}-\d{2}\]?|^Conversation with").unwrap();

        let sessions: Vec<&str> = session_regex.split(content).filter(|s| !s.trim().is_empty()).collect();

        for (idx, session) in sessions.iter().enumerate() {
            let mut metadata = HashMap::new();
            metadata.insert("source_file".to_string(), path.to_string_lossy().to_string());
            metadata.insert("session_index".to_string(), idx.to_string());
            metadata.insert("format".to_string(), "generic".to_string());

            documents.push(Document::new(session.to_string(), metadata));
        }

        // If no sessions were split, treat entire file as one document
        if documents.is_empty() {
            let mut metadata = HashMap::new();
            metadata.insert("source_file".to_string(), path.to_string_lossy().to_string());
            metadata.insert("format".to_string(), "generic".to_string());

            documents.push(Document::new(content.to_string(), metadata));
        }

        Ok(documents)
    }

    /// Infer wing from conversation name
    fn infer_wing_from_name(&self, name: &str) -> Option<String> {
        // Look for project names in conversation title
        let project_patterns = [
            Regex::new(r"(?i)\b(project|app|system|tool)\s+([A-Z][a-zA-Z]+)").unwrap(),
            Regex::new(r"(?i)\b([A-Z][a-zA-Z]+)\s+(project|app|system|tool)").unwrap(),
        ];

        for pattern in &project_patterns {
            if let Some(caps) = pattern.captures(name) {
                if let Some(project) = caps.get(2) {
                    return Some(project.as_str().to_string());
                }
                if let Some(project) = caps.get(1) {
                    return Some(project.as_str().to_string());
                }
            }
        }

        None
    }
}
