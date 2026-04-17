//! Main application state and event loop
//!
//! SPEC-010: TUI Overview

use anyhow::Result;
use crossterm::event::{Event, KeyEvent};
use ratatui::Frame;

use crate::panels::{ChatPanel, FilePreviewPanel, AgentStatusPanel, OutputLogPanel};
use crate::input::CommandBar;
use crate::theme::Theme;
use crate::keybindings::KeyBindings;

/// Main application struct
pub struct App {
    /// Current screen mode
    pub mode: AppMode,
    /// Panels
    pub chat_panel: ChatPanel,
    pub file_panel: FilePreviewPanel,
    pub status_panel: AgentStatusPanel,
    pub output_panel: OutputLogPanel,
    /// Input
    pub command_bar: CommandBar,
    /// Theme
    pub theme: Theme,
    /// Key bindings
    pub keybindings: KeyBindings,
    /// Running state
    pub running: bool,
}

/// Application screen modes (SPEC-010 Section 5)
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    /// Default layout with all panels
    Default,
    /// Full-screen chat
    FullChat,
    /// Full-screen file preview
    FullFile,
    /// Full-screen diff
    FullDiff,
    /// File browser (F2)
    FileBrowser,
    /// Git status (F3)
    GitStatus,
    /// Help screen (F1)
    Help,
}

impl App {
    /// Create a new application instance
    pub fn new(theme: Theme) -> Self {
        Self {
            mode: AppMode::Default,
            chat_panel: ChatPanel::new(),
            file_panel: FilePreviewPanel::new(),
            status_panel: AgentStatusPanel::new(),
            output_panel: OutputLogPanel::new(),
            command_bar: CommandBar::new(),
            theme,
            keybindings: KeyBindings::default(),
            running: true,
        }
    }

    /// Run the main event loop
    pub fn run(&mut self) -> Result<()> {
        // Initialize terminal
        // Setup crossterm
        // Main loop: poll events → handle → render
        Ok(())
    }

    /// Handle a key event
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Route key event to appropriate handler
    }

    /// Render the UI
    pub fn render(&self, frame: &mut Frame) {
        match self.mode {
            AppMode::Default => self.render_default(frame),
            AppMode::FullChat => self.render_full_chat(frame),
            AppMode::FullFile => self.render_full_file(frame),
            AppMode::FullDiff => self.render_full_diff(frame),
            AppMode::FileBrowser => self.render_file_browser(frame),
            AppMode::GitStatus => self.render_git_status(frame),
            AppMode::Help => self.render_help(frame),
        }
    }

    fn render_default(&self, frame: &mut Frame) {
        // Layout: Chat (40%) | File Preview (35%) / Diff (35%)
        // Bottom: Status bar + Command bar + Help bar
    }

    fn render_full_chat(&self, frame: &mut Frame) {}
    fn render_full_file(&self, frame: &mut Frame) {}
    fn render_full_diff(&self, frame: &mut Frame) {}
    fn render_file_browser(&self, frame: &mut Frame) {}
    fn render_git_status(&self, frame: &mut Frame) {}
    fn render_help(&self, frame: &mut Frame) {}
}
