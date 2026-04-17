//! Agent Status Panel - Real-time state visualization (SPEC-013)

use ratatui::Frame;
use ratatui::layout::Rect;

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
        // Render status bar
    }
}
