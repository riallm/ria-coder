//! Diff Panel - Before/after code comparison (SPEC-012 Section 4.3)

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

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
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Diff: {}", self.file_path));

        // Simple line-by-line diff for prototype
        // In a real app we'd use a diffing library
        let mut items = Vec::new();

        if self.original.is_empty() && !self.modified.is_empty() {
            // New file
            for line in self.modified.lines() {
                items.push(
                    ListItem::new(format!("+{}", line)).style(Style::default().fg(Color::Green)),
                );
            }
        } else {
            // Show modified
            let orig_lines: Vec<&str> = self.original.lines().collect();
            let mod_lines: Vec<&str> = self.modified.lines().collect();

            // Very basic matching for prototype
            for (i, line) in mod_lines.iter().enumerate() {
                if i < orig_lines.len() {
                    if line == &orig_lines[i] {
                        items.push(ListItem::new(format!(" {}", line)));
                    } else {
                        items.push(
                            ListItem::new(format!("-{}", orig_lines[i]))
                                .style(Style::default().fg(Color::Red)),
                        );
                        items.push(
                            ListItem::new(format!("+{}", line))
                                .style(Style::default().fg(Color::Green)),
                        );
                    }
                } else {
                    items.push(
                        ListItem::new(format!("+{}", line))
                            .style(Style::default().fg(Color::Green)),
                    );
                }
            }
        }

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}
