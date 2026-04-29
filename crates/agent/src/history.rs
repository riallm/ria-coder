//! Conversation History

use chrono::DateTime;
use std::path::{Path, PathBuf};

/// Message in conversation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MessageRole {
    User,
    Agent,
    System,
}

/// Conversation history tracker
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationHistory {
    messages: Vec<Message>,
    max_messages: usize,
}

impl ConversationHistory {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 100,
        }
    }

    pub fn add(&mut self, role: MessageRole, content: String) {
        self.messages.push(Message {
            role,
            content,
            timestamp: chrono::Utc::now(),
        });
        if self.messages.len() > self.max_messages {
            let overflow = self.messages.len() - self.max_messages;
            self.messages.drain(0..overflow);
        }
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(path)?;
        let mut history: Self = serde_json::from_str(&content)?;
        if history.max_messages == 0 {
            history.max_messages = 100;
        }
        Ok(history)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn default_path() -> PathBuf {
        let config_path = ria_config::Config::default_path();
        config_path
            .parent()
            .map(|parent| parent.join("history.json"))
            .unwrap_or_else(|| PathBuf::from("history.json"))
    }

    pub fn load_default() -> anyhow::Result<Self> {
        Self::load(&Self::default_path())
    }

    pub fn save_default(&self) -> anyhow::Result<()> {
        self.save(&Self::default_path())
    }
}
