//! Conversation History

use chrono::DateTime;

/// Message in conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Agent,
    System,
}

/// Conversation history tracker
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
}
