//! File Preview Panel - Syntax-highlighted code view (SPEC-012)

use crate::syntax::{Language, SyntaxHighlighter};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
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
        self.cursor_line = 0;
        self.scroll_offset = 0;
    }

    pub fn scroll_down(&mut self) {
        let line_count = self.content.lines().count();
        if self.cursor_line + 1 < line_count {
            self.cursor_line += 1;
        }
        if self.cursor_line >= self.scroll_offset + 1 {
            self.scroll_offset = self.scroll_offset.max(self.cursor_line.saturating_sub(1));
        }
    }

    pub fn scroll_up(&mut self) {
        self.cursor_line = self.cursor_line.saturating_sub(1);
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        }
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
        let visible_height = area.height.saturating_sub(2).max(1) as usize;
        let highlighted_lines = highlighter.highlight(&self.content, &language);
        let numbered_lines = highlighted_lines
            .into_iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(visible_height)
            .map(|(index, mut line)| {
                let number = format!("{:>4}  ", index + 1);
                line.spans.insert(
                    0,
                    Span::styled(number, Style::default().fg(Color::DarkGray)),
                );
                if index == self.cursor_line {
                    line = line.style(Style::default().bg(Color::DarkGray));
                }
                line
            })
            .collect::<Vec<_>>();

        let paragraph = Paragraph::new(numbered_lines).block(block);
        frame.render_widget(paragraph, area);
    }
}
