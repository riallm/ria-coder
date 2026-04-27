//! File Browser Panel - Project file navigation (SPEC-010 Section 5)

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use std::path::PathBuf;

pub struct FileBrowserPanel {
    pub root: PathBuf,
    pub items: Vec<PathBuf>,
    pub state: ListState,
}

impl FileBrowserPanel {
    pub fn new(root: PathBuf) -> Self {
        let mut panel = Self {
            root,
            items: Vec::new(),
            state: ListState::default(),
        };
        panel.refresh();
        panel
    }

    pub fn refresh(&mut self) {
        self.items.clear();
        let walker = ignore::WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker.filter_map(|e: Result<ignore::DirEntry, ignore::Error>| e.ok()) {
            if entry
                .file_type()
                .map(|ft: std::fs::FileType| ft.is_file())
                .unwrap_or(false)
            {
                if let Ok(rel) = entry.path().strip_prefix(&self.root) {
                    self.items.push(rel.to_path_buf());
                }
            }
        }
        self.items.sort();
        if !self.items.is_empty() && self.state.selected().is_none() {
            self.state.select(Some(0));
        }
    }

    pub fn next(&mut self) {
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

    pub fn selected_item(&self) -> Option<&PathBuf> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("File Browser (F2 to close)");

        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|path| ListItem::new(path.to_string_lossy().to_string()))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}
