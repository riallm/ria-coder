//! Chat Panel - Conversation with agent (SPEC-011)

use ratatui::Frame;
use ratatui::layout::Rect;

#[derive(Debug)]
pub struct ChatPanel {
    messages: Vec<ChatMessage>,
    scroll_offset: usize,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: MessageSender,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageSender {
    User,
    Agent,
    Status,
    Success,
    Error,
}

impl ChatPanel {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
        }
    }

    pub fn add_message(&mut self, sender: MessageSender, content: String) {
        self.messages.push(ChatMessage { sender, content });
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Render chat panel with messages
    }
}
