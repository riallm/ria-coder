//! Main application state and event loop
//!
//! SPEC-010: TUI Overview

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::input::{CommandBar, InputMode};
use crate::keybindings::{Action, KeyBindings};
use crate::panels::{
    AgentStatusPanel, ChatPanel, DiffPanel, FileBrowserPanel, FilePreviewPanel, GitStatusPanel,
    LogKind, MessageSender, OutputLogPanel,
};
use crate::syntax::SyntaxHighlighter;
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
    pub file_browser: FileBrowserPanel,
    pub git_status: GitStatusPanel,
    /// Input
    pub command_bar: CommandBar,
    /// Theme
    pub theme: Theme,
    /// Key bindings
    pub keybindings: KeyBindings,
    /// Syntax Highlighter
    pub highlighter: SyntaxHighlighter,
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
    /// Output log (F9)
    OutputLog,
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
        let highlighter = SyntaxHighlighter::new("base16-ocean.dark");

        Ok(Self {
            project_root: project_root.clone(),
            mode: AppMode::Default,
            input_buffer: String::new(),
            chat_panel: ChatPanel::new(),
            file_panel: FilePreviewPanel::new(),
            diff_panel: DiffPanel::new(),
            status_panel: AgentStatusPanel::new(),
            output_panel: OutputLogPanel::new(),
            file_browser: FileBrowserPanel::new(project_root),
            git_status: GitStatusPanel::new(),
            command_bar: CommandBar::new(),
            theme,
            keybindings: KeyBindings::default(),
            highlighter,
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
        self.status_panel.pending_changes = self
            .orchestrator
            .last_changes
            .as_ref()
            .map(|changes| changes.file_count())
            .unwrap_or(0);

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
            .add(LogKind::Info, format!("File event: {:?}", event.kind));
    }

    /// Handle a key event
    pub fn handle_key(&mut self, key: KeyEvent) {
        if let Some(action) = self.keybindings.find_action(&key) {
            match action {
                Action::Quit => self.running = false,
                Action::Help => self.mode = AppMode::Help,
                Action::Build => self.run_build(),
                Action::Test => self.run_tests(),
                Action::ToggleLayout => self.cycle_layout(),
                Action::OutputLog => {
                    self.mode = if self.mode == AppMode::OutputLog {
                        AppMode::Default
                    } else {
                        AppMode::OutputLog
                    };
                }
                Action::FileBrowser => {
                    if self.mode == AppMode::FileBrowser {
                        self.mode = AppMode::Default;
                    } else {
                        self.file_browser.refresh();
                        self.mode = AppMode::FileBrowser;
                    }
                }
                Action::GitStatus => {
                    if self.mode == AppMode::GitStatus {
                        self.mode = AppMode::Default;
                    } else {
                        self.git_status.refresh(&self.orchestrator.executor.tools);
                        self.mode = AppMode::GitStatus;
                    }
                }
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
                    self.sync_input_mode();
                }
                _ => {}
            }
            return;
        }

        // Special handling for panels when active
        match self.mode {
            AppMode::FileBrowser => match key.code {
                KeyCode::Down => self.file_browser.next(),
                KeyCode::Up => self.file_browser.previous(),
                KeyCode::Enter => {
                    if let Some(path) = self.file_browser.selected_item() {
                        let rel_path = path.to_string_lossy().to_string();
                        let full_path = self.project_root.join(path);
                        if let Ok(content) = std::fs::read_to_string(&full_path) {
                            self.file_panel.open_file(&rel_path);
                            self.file_panel.content = content;
                            self.mode = AppMode::Default;
                        }
                    }
                }
                _ => {}
            },
            AppMode::GitStatus => match key.code {
                KeyCode::Down => self.git_status.next(),
                KeyCode::Up => self.git_status.previous(),
                _ => {}
            },
            AppMode::FullFile => match key.code {
                KeyCode::Down | KeyCode::Char('j') => self.file_panel.scroll_down(),
                KeyCode::Up | KeyCode::Char('k') => self.file_panel.scroll_up(),
                _ => {}
            },
            AppMode::OutputLog => match key.code {
                KeyCode::Down | KeyCode::Char('j') => self.output_panel.scroll_down(),
                KeyCode::Up | KeyCode::Char('k') => self.output_panel.scroll_up(),
                KeyCode::PageDown => self.output_panel.page_down(10),
                KeyCode::PageUp => self.output_panel.page_up(10),
                KeyCode::Char('q') => self.mode = AppMode::Default,
                _ => {}
            },
            _ => {
                // Fallback for regular typing
                match key.code {
                    KeyCode::Char(c) => {
                        self.input_buffer.push(c);
                        self.sync_input_mode();
                    }
                    KeyCode::Backspace => {
                        self.input_buffer.pop();
                        self.sync_input_mode();
                    }
                    KeyCode::Up => {
                        self.previous_history();
                    }
                    KeyCode::Down => {
                        self.next_history();
                    }
                    _ => {}
                }
            }
        }
    }

    fn process_input(&mut self, input: String) {
        self.command_bar.push_history(input.clone());

        self.chat_panel
            .add_message(MessageSender::User, input.clone());

        if input.starts_with(':') {
            self.process_direct_command(&input);
            return;
        }

        if let Some(mapped) = self.map_slash_command(&input) {
            self.process_agent_request(mapped);
            return;
        }

        self.process_agent_request(input);
    }

    fn process_agent_request(&mut self, input: String) {
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

    fn process_direct_command(&mut self, input: &str) {
        let command = input.trim_start_matches(':').trim();
        if command.is_empty() {
            return;
        }

        match command {
            "build" => self.run_build(),
            "test" => self.run_tests(),
            "lint" => self.run_lint(),
            "files" => {
                self.file_browser.refresh();
                self.mode = AppMode::FileBrowser;
            }
            "clear" => {
                self.chat_panel.messages.clear();
                self.output_panel.lines.clear();
            }
            "quit" | "q" => self.running = false,
            "undo" => match self.orchestrator.undo_last() {
                Ok(()) => self.chat_panel.add_message(
                    MessageSender::Status,
                    "Rolled back last change.".to_string(),
                ),
                Err(e) => self
                    .chat_panel
                    .add_message(MessageSender::Error, format!("Rollback failed: {}", e)),
            },
            "redo" => match self.orchestrator.redo_last() {
                Ok(()) => self
                    .chat_panel
                    .add_message(MessageSender::Status, "Re-applied last change.".to_string()),
                Err(e) => self
                    .chat_panel
                    .add_message(MessageSender::Error, format!("Redo failed: {}", e)),
            },
            "reset" => match self.orchestrator.reset_session_changes() {
                Ok(()) => self.chat_panel.add_message(
                    MessageSender::Status,
                    "Reset tracked session changes.".to_string(),
                ),
                Err(e) => self
                    .chat_panel
                    .add_message(MessageSender::Error, format!("Reset failed: {}", e)),
            },
            "history" => self.show_session_history(),
            "log" | "output" => self.mode = AppMode::OutputLog,
            "plugins" => self.show_plugin_commands(),
            "git status" => self.run_git("status", None),
            "git diff" => self.run_git("diff", None),
            other if other.starts_with("open ") || other.starts_with("e ") => {
                let path = other
                    .split_once(' ')
                    .map(|(_, path)| path.trim())
                    .unwrap_or_default();
                self.open_file(path);
            }
            other if other.starts_with("grep ") => {
                let pattern = other.trim_start_matches("grep").trim().trim_matches('"');
                self.run_search(pattern);
            }
            other if other.starts_with("git log") => self.run_git("log", Some(("count", "20"))),
            other => {
                if self.run_plugin_command(other) {
                    return;
                }
                self.chat_panel
                    .add_message(MessageSender::Error, format!("Unknown command: {}", other))
            }
        }
    }

    fn map_slash_command(&self, input: &str) -> Option<String> {
        let rest = input.strip_prefix('/')?;
        let (command, args) = rest.split_once(' ').unwrap_or((rest, ""));
        let mapped = match command {
            "explain" => format!("explain {}", args),
            "refactor" => format!("refactor {}", args),
            "test" => format!("write tests for {}", args),
            "fix" => format!("fix {}", args),
            "doc" => format!("document {}", args),
            "review" => format!("review {}", args),
            _ => return None,
        };
        Some(mapped.trim().to_string())
    }

    fn run_build(&mut self) {
        let mut args = HashMap::new();
        args.insert("action".to_string(), "check".to_string());
        self.run_tool("build", args, "build");
    }

    fn run_tests(&mut self) {
        self.run_tool("test", HashMap::new(), "test");
    }

    fn run_lint(&mut self) {
        let mut args = HashMap::new();
        args.insert("action".to_string(), "clippy".to_string());
        self.run_tool("lint", args, "lint");
    }

    fn run_git(&mut self, action: &str, extra: Option<(&str, &str)>) {
        let mut args = HashMap::new();
        args.insert("action".to_string(), action.to_string());
        if let Some((key, value)) = extra {
            args.insert(key.to_string(), value.to_string());
        }
        self.run_tool("git", args, &format!("git {}", action));
    }

    fn run_search(&mut self, pattern: &str) {
        if pattern.is_empty() {
            self.chat_panel
                .add_message(MessageSender::Error, "Usage: :grep <pattern>".to_string());
            return;
        }

        let mut args = HashMap::new();
        args.insert("action".to_string(), "content".to_string());
        args.insert("pattern".to_string(), pattern.to_string());
        args.insert("max_results".to_string(), "50".to_string());
        self.run_tool("search", args, "search");
    }

    fn run_tool(&mut self, tool: &str, args: HashMap<String, String>, label: &str) {
        self.output_panel
            .add(LogKind::Command, format!("$ {}", label));
        match self.orchestrator.executor.tools.execute(tool, &args) {
            Ok(output) => {
                let status = if output.exit_code == 0 {
                    MessageSender::Success
                } else {
                    MessageSender::Error
                };
                let log_kind = if output.exit_code == 0 {
                    LogKind::Success
                } else {
                    LogKind::Failure
                };
                self.chat_panel.add_message(
                    status,
                    format!("{} exited with {}", label, output.exit_code),
                );
                self.output_panel.add(
                    log_kind,
                    format!("{} exited with {}", label, output.exit_code),
                );
                self.append_tool_output(LogKind::Stdout, output.stdout);
                self.append_tool_output(LogKind::Stderr, output.stderr);
            }
            Err(e) => self
                .chat_panel
                .add_message(MessageSender::Error, format!("{} failed: {}", label, e)),
        }
    }

    fn append_tool_output(&mut self, kind: LogKind, output: String) {
        for line in output.lines().take(200) {
            self.output_panel.add(kind.clone(), line.to_string());
        }
    }

    fn show_session_history(&mut self) {
        if self.orchestrator.session_history.is_empty() {
            self.output_panel
                .add(LogKind::Info, "Session history is empty");
        } else {
            self.output_panel
                .add(LogKind::Command, "$ session history".to_string());
            for (index, record) in self.orchestrator.session_history.iter().enumerate() {
                self.output_panel.add(
                    LogKind::Info,
                    format!(
                        "{}. {:?}: {} ({} files)",
                        index + 1,
                        record.status,
                        record.summary,
                        record.file_count
                    ),
                );
            }
        }
        self.mode = AppMode::OutputLog;
    }

    fn show_plugin_commands(&mut self) {
        self.output_panel
            .add(LogKind::Command, "$ plugin commands".to_string());
        for command in self.orchestrator.plugins.commands() {
            self.output_panel.add(
                LogKind::Info,
                format!(":{} - {}", command.name, command.description),
            );
        }
        self.mode = AppMode::OutputLog;
    }

    fn run_plugin_command(&mut self, command_name: &str) -> bool {
        let Some(command) = self.orchestrator.plugins.command(command_name).cloned() else {
            return false;
        };
        self.run_tool(&command.tool, command.args, &command.name);
        true
    }

    fn open_file(&mut self, path: &str) {
        if path.is_empty() {
            self.chat_panel
                .add_message(MessageSender::Error, "Usage: :open <path>".to_string());
            return;
        }

        let full_path = self.project_root.join(path);
        if let Ok(content) = std::fs::read_to_string(&full_path) {
            self.file_panel.open_file(path);
            self.file_panel.content = content;
            self.chat_panel
                .add_message(MessageSender::Success, format!("Opened {}", path));
            self.mode = AppMode::Default;
        } else {
            self.chat_panel
                .add_message(MessageSender::Error, format!("Could not open {}", path));
        }
    }

    fn sync_input_mode(&mut self) {
        self.command_bar.mode = if self.input_buffer.starts_with(':') {
            InputMode::Command
        } else if self.input_buffer.starts_with('/') {
            InputMode::SlashCommand
        } else {
            InputMode::NaturalLanguage
        };
    }

    fn previous_history(&mut self) {
        if self.command_bar.history.is_empty() {
            return;
        }
        self.command_bar.history_index = self.command_bar.history_index.saturating_sub(1);
        if let Some(value) = self.command_bar.history.get(self.command_bar.history_index) {
            self.input_buffer = value.clone();
            self.sync_input_mode();
        }
    }

    fn next_history(&mut self) {
        if self.command_bar.history.is_empty() {
            return;
        }
        self.command_bar.history_index =
            (self.command_bar.history_index + 1).min(self.command_bar.history.len());
        if self.command_bar.history_index == self.command_bar.history.len() {
            self.input_buffer.clear();
        } else if let Some(value) = self.command_bar.history.get(self.command_bar.history_index) {
            self.input_buffer = value.clone();
        }
        self.sync_input_mode();
    }

    fn cycle_layout(&mut self) {
        self.mode = match self.mode {
            AppMode::Default => AppMode::FullChat,
            AppMode::FullChat => AppMode::FullFile,
            AppMode::FullFile => AppMode::FullDiff,
            _ => AppMode::Default,
        };
    }

    /// Render the UI
    pub fn render(&mut self, frame: &mut Frame) {
        match self.mode {
            AppMode::Default => self.render_default(frame),
            AppMode::FullChat => self.render_full_chat(frame),
            AppMode::FullFile => self.render_full_file(frame),
            AppMode::FullDiff => self.render_full_diff(frame),
            AppMode::FileBrowser => self.render_file_browser_overlay(frame),
            AppMode::GitStatus => self.render_git_status_overlay(frame),
            AppMode::OutputLog => self.render_output_log(frame),
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

        if main_chunks[1].height >= 24 {
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40), // File Preview
                    Constraint::Percentage(35), // Diff
                    Constraint::Percentage(25), // Output Log
                ])
                .split(main_chunks[1]);

            self.file_panel
                .render(frame, right_chunks[0], &self.highlighter);
            self.diff_panel.render(frame, right_chunks[1]);
            self.output_panel.render(frame, right_chunks[2]);
        } else {
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50), // File Preview
                    Constraint::Percentage(50), // Diff
                ])
                .split(main_chunks[1]);

            self.file_panel
                .render(frame, right_chunks[0], &self.highlighter);
            self.diff_panel.render(frame, right_chunks[1]);
        }

        // Status bar
        self.status_panel.render(frame, chunks[2]);

        // Command bar
        let prompt = match self.command_bar.mode {
            InputMode::NaturalLanguage => ">",
            InputMode::Command => ":",
            InputMode::SlashCommand => "/",
            InputMode::Search => "?",
        };
        let input_text = match self.command_bar.mode {
            InputMode::Command | InputMode::SlashCommand => self
                .input_buffer
                .get(1..)
                .unwrap_or(self.input_buffer.as_str()),
            _ => self.input_buffer.as_str(),
        };
        let input_display = format!("{} {}", prompt, input_text);
        frame.render_widget(
            Paragraph::new(input_display).block(Block::default().borders(Borders::NONE)),
            chunks[3],
        );

        // Help bar
        frame.render_widget(
            Paragraph::new(
                "F1:Help  F2:Files  F3:Git  F4:Build  F5:Test  F8:Layout  F9:Log  F10:Quit",
            ),
            chunks[4],
        );
    }

    fn render_file_browser_overlay(&mut self, frame: &mut Frame) {
        self.render_default(frame);
        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(Clear, area);
        self.file_browser.render(frame, area);
    }

    fn render_git_status_overlay(&mut self, frame: &mut Frame) {
        self.render_default(frame);
        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(Clear, area);
        self.git_status.render(frame, area);
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

    fn render_full_chat(&self, frame: &mut Frame) {
        self.chat_panel.render(frame, frame.area());
    }

    fn render_full_file(&self, frame: &mut Frame) {
        self.file_panel
            .render(frame, frame.area(), &self.highlighter);
    }

    fn render_full_diff(&self, frame: &mut Frame) {
        self.diff_panel.render(frame, frame.area());
    }

    fn render_output_log(&self, frame: &mut Frame) {
        self.output_panel.render(frame, frame.area());
    }

    fn render_help(&self, frame: &mut Frame) {
        let help = [
            "Ria Coder Help",
            "",
            "F1 Help    F2 Files    F3 Git    F4 Build    F5 Test    F8 Layout    F9 Log    F10 Quit",
            "F6 Accept proposed changes    F7 Reject proposed changes",
            "",
            "Commands:",
            ":build       Run project check/build",
            ":test        Run test suite",
            ":lint        Run linter",
            ":git status  Show working tree status",
            ":git diff    Show unstaged diff",
            ":files       Open file browser",
            ":open PATH   Open a file",
            ":grep TEXT   Search project files",
            ":undo        Roll back the last pending or accepted change",
            ":redo        Re-apply the last undone change",
            ":reset       Roll back tracked session changes",
            ":history     Show session change history",
            ":log         Open output log",
            ":clear       Clear chat and output",
            ":quit        Exit",
            "",
            "Slash workflows: /explain, /refactor, /test, /fix, /doc, /review",
            "",
            "Press Esc to return.",
        ]
        .join("\n");

        frame.render_widget(
            Paragraph::new(help).block(Block::default().borders(Borders::ALL).title("Help")),
            frame.area(),
        );
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
