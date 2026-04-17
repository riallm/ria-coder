//! Diff Panel - Before/after code comparison

use ratatui::Frame;
use ratatui::layout::Rect;

#[derive(Debug)]
pub struct DiffPanel {
    pub original: String,
    pub modified: String,
    pub file_path: String,
}

impl DiffPanel {
    pub fn new() -> Self {
        Self {
            original: String::new(),
            modified: String::new(),
            file_path: String::new(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Render unified diff
    }
}
