//! Output Log Panel - Tool execution output (SPEC-015)

use ratatui::Frame;
use ratatui::layout::Rect;

#[derive(Debug)]
pub struct OutputLogPanel {
    lines: Vec<String>,
    scroll_offset: usize,
}

impl OutputLogPanel {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            scroll_offset: 0,
        }
    }

    pub fn add_line(&mut self, line: String) {
        self.lines.push(line);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Render output log
    }
}
