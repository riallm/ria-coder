//! File Preview Panel - Syntax-highlighted code view (SPEC-012)

use crate::syntax::{Language, SyntaxHighlighter};
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

    pub fn render(&self, frame: &mut Frame, area: Rect, highlighter: &SyntaxHighlighter) {
        let title = format!(
            "Preview: {}",
            self.current_file.as_deref().unwrap_or("No file")
        );
        let block = Block::default().borders(Borders::ALL).title(title);

        if self.content.is_empty() {
            frame.render_widget(Paragraph::new("No content").block(block), area);
            return;
        }

        let extension = self
            .current_file
            .as_ref()
            .and_then(|path| std::path::Path::new(path).extension())
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let language = Language::from_extension(extension);
        let highlighted_lines = highlighter.highlight(&self.content, &language);

        let paragraph = Paragraph::new(highlighted_lines).block(block);
        frame.render_widget(paragraph, area);
    }
}
