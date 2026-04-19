//! Output Log Panel - Tool execution output (SPEC-015)

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

#[derive(Debug)]
pub struct OutputLogPanel {
    pub lines: Vec<String>,
    pub scroll_offset: usize,
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
        let block = Block::default().borders(Borders::ALL).title("Output Log");

        let items: Vec<ListItem> = self
            .lines
            .iter()
            .map(|line| ListItem::new(line.as_str()))
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}
