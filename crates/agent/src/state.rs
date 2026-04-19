//! Agent State Machine (SPEC-021)

use crate::execution::ChangeSet;
use crate::planning::Plan;
use crate::task::Task;

/// Agent states (SPEC-021 Section 2)
#[derive(Debug, Clone, PartialEq)]
pub enum AgentState {
    /// Waiting for user input
    Idle,
    /// Parsing and understanding request
    Understanding(UnderstandingState),
    /// Generating execution plan
    Planning(PlanningState),
    /// Executing plan steps
    Executing(ExecutingState),
    /// Verifying execution results
    Verifying(VerifyingState),
    /// Presenting results to user
    Presenting(PresentingState),
    /// Error state
    Error(ErrorState),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnderstandingState {
    pub request: String,
    pub parsed: Option<Task>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanningState {
    pub task: Task,
    pub plan: Option<Plan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutingState {
    pub plan: Plan,
    pub current_step: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifyingState {
    pub changes: ChangeSet,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PresentingState {
    pub result: AgentResult,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorState {
    pub message: String,
    pub can_retry: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentResult {
    pub message: String,
    pub success: bool,
}

impl AgentState {
    /// Get human-readable status
    pub fn status_text(&self) -> &str {
        match self {
            Self::Idle => "Idle",
            Self::Understanding(_) => "Understanding request...",
            Self::Planning(_) => "Planning steps...",
            Self::Executing(_s) => "Executing...",
            Self::Verifying(_) => "Verifying changes...",
            Self::Presenting(_) => "Review changes?",
            Self::Error(_) => "Error",
        }
    }

    /// Check if agent is busy
    pub fn is_busy(&self) -> bool {
        !matches!(self, Self::Idle | Self::Error(_))
    }
}
