//! Output Log Panel - Tool execution output (SPEC-015)

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

#[derive(Debug)]
pub struct OutputLogPanel {
    pub lines: Vec<LogLine>,
    pub scroll_offset: usize,
    pub max_lines: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    pub kind: LogKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogKind {
    Command,
    Stdout,
    Stderr,
    Success,
    Failure,
    Info,
}

impl OutputLogPanel {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            scroll_offset: 0,
            max_lines: 10_000,
        }
    }

    pub fn add_line(&mut self, line: String) {
        self.add(LogKind::Info, line);
    }

    pub fn add(&mut self, kind: LogKind, text: impl Into<String>) {
        self.lines.push(LogLine {
            kind,
            text: text.into(),
        });
        if self.lines.len() > self.max_lines {
            let overflow = self.lines.len() - self.max_lines;
            self.lines.drain(0..overflow);
            self.scroll_offset = self.scroll_offset.saturating_sub(overflow);
        }
        self.scroll_to_bottom();
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = (self.scroll_offset + 1).min(self.lines.len().saturating_sub(1));
    }

    pub fn page_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount.max(1));
    }

    pub fn page_down(&mut self, amount: usize) {
        self.scroll_offset =
            (self.scroll_offset + amount.max(1)).min(self.lines.len().saturating_sub(1));
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.lines.len().saturating_sub(1);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Output Log");
        let visible_height = area.height.saturating_sub(2).max(1) as usize;
        let start = if self.lines.len() <= visible_height {
            0
        } else {
            self.scroll_offset
                .saturating_sub(visible_height.saturating_sub(1))
                .min(self.lines.len().saturating_sub(visible_height))
        };

        let items: Vec<ListItem> = self
            .lines
            .iter()
            .skip(start)
            .take(visible_height)
            .map(|line| ListItem::new(line.text.as_str()).style(style_for_kind(&line.kind)))
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}

fn style_for_kind(kind: &LogKind) -> Style {
    match kind {
        LogKind::Command => Style::default().fg(Color::DarkGray),
        LogKind::Stdout => Style::default().fg(Color::White),
        LogKind::Stderr => Style::default().fg(Color::Yellow),
        LogKind::Success => Style::default().fg(Color::Green),
        LogKind::Failure => Style::default().fg(Color::Red),
        LogKind::Info => Style::default().fg(Color::Cyan),
    }
}
