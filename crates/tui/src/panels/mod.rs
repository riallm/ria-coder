//! UI Panels
//!
//! SPEC-010 through SPEC-015

pub mod agent_status;
pub mod chat;
pub mod diff;
pub mod file_browser;
pub mod file_preview;
pub mod git_status;
pub mod output_log;

pub use agent_status::AgentStatusPanel;
pub use chat::{ChatPanel, MessageSender};
pub use diff::DiffPanel;
pub use file_browser::FileBrowserPanel;
pub use file_preview::FilePreviewPanel;
pub use git_status::GitStatusPanel;
pub use output_log::{LogKind, OutputLogPanel};
