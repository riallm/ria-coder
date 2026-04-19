//! File Preview Panel - Syntax-highlighted code view (SPEC-012)

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug)]
pub struct FilePreviewPanel {
    pub current_file: Option<String>,
    pub content: String,
    pub cursor_line: usize,
    pub scroll_offset: usize,
}

impl FilePreviewPanel {
    pub fn new() -> Self {
        Self {
            current_file: None,
            content: String::new(),
            cursor_line: 0,
            scroll_offset: 0,
        }
    }

    pub fn open_file(&mut self, path: &str) {
        self.current_file = Some(path.to_string());
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let title = format!(
            "Preview: {}",
            self.current_file.as_deref().unwrap_or("No file")
        );
        let block = Block::default().borders(Borders::ALL).title(title);

        let paragraph = Paragraph::new(self.content.as_str()).block(block);
        frame.render_widget(paragraph, area);
    }
}
