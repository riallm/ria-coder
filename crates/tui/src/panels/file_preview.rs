//! File Preview Panel - Syntax-highlighted code view (SPEC-012)

use ratatui::Frame;
use ratatui::layout::Rect;

#[derive(Debug)]
pub struct FilePreviewPanel {
    current_file: Option<String>,
    content: String,
    cursor_line: usize,
    scroll_offset: usize,
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
        // Render file with syntax highlighting
    }
}
