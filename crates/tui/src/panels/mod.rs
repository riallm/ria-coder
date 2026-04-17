//! UI Panels
//!
//! SPEC-010 through SPEC-015

pub mod chat;
pub mod file_preview;
pub mod agent_status;
pub mod output_log;
pub mod diff;

pub use chat::ChatPanel;
pub use file_preview::FilePreviewPanel;
pub use agent_status::AgentStatusPanel;
pub use output_log::OutputLogPanel;
pub use diff::DiffPanel;
