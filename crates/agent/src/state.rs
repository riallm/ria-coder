//! Agent State Machine (SPEC-021)

use crate::task::Task;
use crate::planning::Plan;

/// Agent states (SPEC-021 Section 2)
#[derive(Debug, Clone, PartialEq)]
pub enum AgentState {
    /// Waiting for user input
    Idle,
    /// Parsing and understanding request
    Understanding { request: String },
    /// Generating execution plan
    Planning { task: Task },
    /// Executing plan steps
    Executing { plan: Plan, step: usize },
    /// Verifying execution results
    Verifying { changes: usize },
    /// Presenting results to user
    Presenting { result: String },
    /// Error state
    Error { error: String },
}

impl AgentState {
    /// Get display text for status panel
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Idle => "Ready",
            Self::Understanding { .. } => "Analyzing...",
            Self::Planning { .. } => "Planning...",
            Self::Executing { .. } => "Applying...",
            Self::Verifying { .. } => "Testing...",
            Self::Presenting { .. } => "Review changes?",
            Self::Error { .. } => "Error",
        }
    }

    /// Check if agent is busy
    pub fn is_busy(&self) -> bool {
        !matches!(self, Self::Idle | Self::Error { .. })
    }
}
