//! Main application state and event loop
//!
//! SPEC-010: TUI Overview

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::path::PathBuf;

use crate::input::CommandBar;
use crate::keybindings::{Action, KeyBindings};
use crate::panels::{
    AgentStatusPanel, ChatPanel, DiffPanel, FilePreviewPanel, MessageSender, OutputLogPanel,
};
use crate::theme::Theme;
use ria_agent::orchestrator::AgentOrchestrator;
use ria_agent::state::AgentState;
use ria_tools::FileWatcher;

/// Main application struct
pub struct App {
    /// Project root
    pub project_root: PathBuf,
    /// Current screen mode
    pub mode: AppMode,
    /// Current input buffer
    pub input_buffer: String,
    /// Panels
    pub chat_panel: ChatPanel,
    pub file_panel: FilePreviewPanel,
    pub diff_panel: DiffPanel,
    pub status_panel: AgentStatusPanel,
    pub output_panel: OutputLogPanel,
    /// Input
    pub command_bar: CommandBar,
    /// Theme
    pub theme: Theme,
    /// Key bindings
    pub keybindings: KeyBindings,
    /// Agent
    pub orchestrator: AgentOrchestrator,
    /// File Watcher
    pub watcher: FileWatcher,
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
    pub fn new(
        theme: Theme,
        project_root: PathBuf,
        orchestrator: AgentOrchestrator,
    ) -> Result<Self> {
        let watcher = FileWatcher::new()?;

        Ok(Self {
            project_root,
            mode: AppMode::Default,
            input_buffer: String::new(),
            chat_panel: ChatPanel::new(),
            file_panel: FilePreviewPanel::new(),
            diff_panel: DiffPanel::new(),
            status_panel: AgentStatusPanel::new(),
            output_panel: OutputLogPanel::new(),
            command_bar: CommandBar::new(),
            theme,
            keybindings: KeyBindings::default(),
            orchestrator,
            watcher,
            running: true,
        })
    }

    /// Run the main event loop
    pub fn run(&mut self) -> Result<()> {
        use crossterm::{
            event::{self, Event},
            execute,
            terminal::{
                disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
            },
        };
        use ratatui::backend::CrosstermBackend;
        use std::io;

        // Initialize orchestrator
        self.orchestrator.init(self.project_root.clone())?;

        // Start watching project root
        self.watcher.watch(&self.project_root)?;

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)?;

        while self.running {
            // Update UI from agent state
            self.update_ui_from_agent();

            terminal.draw(|f| self.render(f))?;

            // Check for file events
            while let Ok(Ok(event)) = self.watcher.rx.try_recv() {
                self.handle_file_event(event);
            }

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn update_ui_from_agent(&mut self) {
        self.status_panel.state = self.orchestrator.state.status_text().to_string();

        if let AgentState::Verifying(s) = &self.orchestrator.state {
            if let Some(change) = s.changes.changes.first() {
                self.diff_panel.file_path = change.path.clone();
                self.diff_panel.original = change.original.clone();
                self.diff_panel.modified = change.modified.clone();
            }
        } else if let AgentState::Presenting(_s) = &self.orchestrator.state {
            if let Some(changes) = &self.orchestrator.last_changes {
                if let Some(change) = changes.changes.first() {
                    self.diff_panel.file_path = change.path.clone();
                    self.diff_panel.original = change.original.clone();
                    self.diff_panel.modified = change.modified.clone();
                }
            }
        }
    }

    fn handle_file_event(&mut self, event: notify::Event) {
        // Update file preview if current file changed
        if let Some(current) = &self.file_panel.current_file {
            let current_path = self.project_root.join(current);
            if event.paths.iter().any(|p| p == &current_path) {
                // Reload file
                if let Ok(content) = std::fs::read_to_string(&current_path) {
                    self.file_panel.content = content;
                }
            }
        }

        // Add to output log for visibility in prototype
        self.output_panel
            .add_line(format!("File event: {:?}", event.kind));
    }

    /// Handle a key event
    pub fn handle_key(&mut self, key: KeyEvent) {
        if let Some(action) = self.keybindings.find_action(&key) {
            match action {
                Action::Quit => self.running = false,
                Action::Help => self.mode = AppMode::Help,
                Action::FileBrowser => self.mode = AppMode::FileBrowser,
                Action::GitStatus => self.mode = AppMode::GitStatus,
                Action::Cancel => self.mode = AppMode::Default,
                Action::AcceptChanges => {
                    if let AgentState::Presenting(_) = &self.orchestrator.state {
                        if let Err(e) = self.orchestrator.accept_changes() {
                            self.chat_panel.add_message(
                                MessageSender::Error,
                                format!("Failed to apply changes: {}", e),
                            );
                        } else {
                            self.chat_panel.add_message(
                                MessageSender::Success,
                                "Changes applied and committed.".to_string(),
                            );
                        }
                    }
                }
                Action::RejectChanges => {
                    if let AgentState::Presenting(_) = &self.orchestrator.state {
                        if let Err(e) = self.orchestrator.reject_changes() {
                            self.chat_panel.add_message(
                                MessageSender::Error,
                                format!("Failed to rollback changes: {}", e),
                            );
                        } else {
                            self.chat_panel.add_message(
                                MessageSender::Status,
                                "Changes rejected and rolled back.".to_string(),
                            );
                        }
                    }
                }
                Action::SendMessage => {
                    let input = self.input_buffer.drain(..).collect::<String>();
                    if !input.is_empty() {
                        self.process_input(input);
                    }
                }
                _ => {}
            }
            return;
        }

        // Fallback for regular typing
        match key.code {
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            _ => {}
        }
    }

    fn process_input(&mut self, input: String) {
        self.chat_panel
            .add_message(MessageSender::User, input.clone());

        // Prototype: handle basic internal commands directly
        if input.starts_with(":open ") {
            let path = &input[6..];
            let full_path = self.project_root.join(path);
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                self.file_panel.open_file(path);
                self.file_panel.content = content;
                self.chat_panel
                    .add_message(MessageSender::Success, format!("Opened {}", path));
            } else {
                self.chat_panel
                    .add_message(MessageSender::Error, format!("Could not open {}", path));
            }
            return;
        }

        match tokio::runtime::Handle::current().block_on(self.orchestrator.process_request(&input))
        {
            Ok(output) => {
                self.chat_panel
                    .add_message(MessageSender::Agent, output.message);
                if let AgentState::Presenting(_) = &self.orchestrator.state {
                    self.chat_panel.add_message(
                        MessageSender::Status,
                        "Accept changes? (F6: yes, F7: no)".to_string(),
                    );
                }
            }
            Err(e) => {
                self.chat_panel
                    .add_message(MessageSender::Error, format!("Error: {}", e));
            }
        }
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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(1), // Status bar
                Constraint::Length(1), // Command bar
                Constraint::Length(1), // Help bar
            ])
            .split(frame.area());

        // Header
        self.render_header(frame, chunks[0]);

        // Main content
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Chat
                Constraint::Percentage(60), // Preview + Diff
            ])
            .split(chunks[1]);

        self.chat_panel.render(frame, main_chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // File Preview
                Constraint::Percentage(50), // Diff
            ])
            .split(main_chunks[1]);

        self.file_panel.render(frame, right_chunks[0]);
        self.diff_panel.render(frame, right_chunks[1]);

        // Status bar
        self.status_panel.render(frame, chunks[2]);

        // Command bar
        let input_display = format!("> {}", self.input_buffer);
        frame.render_widget(
            Paragraph::new(input_display).block(Block::default().borders(Borders::NONE)),
            chunks[3],
        );

        // Help bar
        frame.render_widget(
            Paragraph::new("F1:Help  F2:Files  F3:Git  F4:Build  F10:Quit"),
            chunks[4],
        );
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(area);

        let project_name = self
            .project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        frame.render_widget(
            Paragraph::new(format!("📁 {}", project_name))
                .block(Block::default().borders(Borders::ALL).title("Project")),
            chunks[0],
        );
        frame.render_widget(
            Paragraph::new("🤖 RIA-8B")
                .block(Block::default().borders(Borders::ALL).title("Model")),
            chunks[1],
        );
        frame.render_widget(
            Paragraph::new(format!("🧠 {}", self.status_panel.state))
                .block(Block::default().borders(Borders::ALL).title("Status")),
            chunks[2],
        );
    }

    fn render_full_chat(&self, _frame: &mut Frame) {}
    fn render_full_file(&self, _frame: &mut Frame) {}
    fn render_full_diff(&self, _frame: &mut Frame) {}
    fn render_file_browser(&self, _frame: &mut Frame) {}
    fn render_git_status(&self, _frame: &mut Frame) {}
    fn render_help(&self, _frame: &mut Frame) {}
}
