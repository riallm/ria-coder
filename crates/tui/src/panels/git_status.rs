//! Git Status Panel - Version control visualization (SPEC-010 Section 5)

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use ria_tools::registry::ToolRegistry;
use std::collections::HashMap;

pub struct GitStatusPanel {
    pub items: Vec<String>,
    pub state: ListState,
}

impl GitStatusPanel {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            state: ListState::default(),
        }
    }

    pub fn refresh(&mut self, registry: &ToolRegistry) {
        self.items.clear();
        let mut args = HashMap::new();
        args.insert("action".to_string(), "status".to_string());

        if let Ok(output) = registry.execute("git", &args) {
            for line in output.stdout.lines() {
                self.items.push(line.to_string());
            }
            if self.items.is_empty() && output.exit_code == 0 {
                self.items.push("Working tree clean".to_string());
            } else if output.exit_code != 0 {
                self.items.push(output.stderr);
            }
        }

        if self.items.is_empty() {
            self.state.select(None);
        } else if self.state.selected().is_none() {
            self.state.select(Some(0));
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Git Status (F3 to close)");

        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|line| {
                let style = if line.starts_with('M') {
                    Style::default().fg(Color::Yellow)
                } else if line.starts_with('A') || line.starts_with('?') {
                    Style::default().fg(Color::Green)
                } else if line.starts_with('D') {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default()
                };
                ListItem::new(line.as_str()).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}
