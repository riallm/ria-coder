//! Ria Coder Agent Orchestration
//!
//! Core agent components per SPEC-020 through SPEC-027:
//! - Agent Orchestrator (SPEC-020)
//! - State Machine (SPEC-021)
//! - Task Parser (SPEC-022)
//! - Planning Engine (SPEC-023)
//! - Execution Engine (SPEC-024)
//! - Verification Engine (SPEC-025)
//! - LLM Interface (SPEC-026)
//! - Context Manager (SPEC-027)

pub mod orchestrator;
pub mod state;
pub mod task;
pub mod planning;
pub mod execution;
pub mod verification;
pub mod llm;
pub mod context;
pub mod history;

pub use orchestrator::AgentOrchestrator;
pub use state::AgentState;
pub use task::Task;
pub use planning::Plan;
pub use execution::ExecutionEngine;
pub use verification::VerificationEngine;
pub use llm::LLMEngine;
pub use context::ContextManager;
