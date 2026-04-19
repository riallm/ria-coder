//! UI Panels
//!
//! SPEC-010 through SPEC-015

pub mod agent_status;
pub mod chat;
pub mod diff;
pub mod file_preview;
pub mod output_log;

pub use agent_status::AgentStatusPanel;
pub use chat::{ChatPanel, MessageSender};
pub use diff::DiffPanel;
pub use file_preview::FilePreviewPanel;
pub use output_log::OutputLogPanel;
