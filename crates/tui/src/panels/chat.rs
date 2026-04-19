//! Chat Panel - Conversation with agent (SPEC-011)

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

#[derive(Debug)]
pub struct ChatPanel {
    pub messages: Vec<ChatMessage>,
    pub scroll_offset: usize,
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
        let block = Block::default().borders(Borders::ALL).title("Chat");

        let items: Vec<ListItem> = self
            .messages
            .iter()
            .map(|msg| {
                let prefix = match msg.sender {
                    MessageSender::User => "👤 You: ",
                    MessageSender::Agent => "🤖 RIA: ",
                    MessageSender::Status => "ℹ️ ",
                    MessageSender::Success => "✅ ",
                    MessageSender::Error => "❌ ",
                };
                ListItem::new(format!("{}{}", prefix, msg.content))
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}
