//! Agent Status Panel - Real-time state visualization (SPEC-013)

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug)]
pub struct AgentStatusPanel {
    pub token_speed: f64,
    pub memory_mb: f64,
    pub files_in_context: usize,
    pub pending_changes: usize,
    pub state: String,
}

impl AgentStatusPanel {
    pub fn new() -> Self {
        Self {
            token_speed: 0.0,
            memory_mb: 0.0,
            files_in_context: 0,
            pending_changes: 0,
            state: "Ready".to_string(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let content = format!(
            "📊 Tokens: {:.1}/s  │  🧠 Mem: {:.1}GB  │  ⚡ Files: {}  │  🔄 Changes: {}",
            self.token_speed,
            self.memory_mb / 1024.0,
            self.files_in_context,
            self.pending_changes
        );
        frame.render_widget(
            Paragraph::new(content).block(Block::default().borders(Borders::NONE)),
            area,
        );
    }
}
