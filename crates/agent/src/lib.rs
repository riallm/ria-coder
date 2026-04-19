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

pub mod context;
pub mod execution;
pub mod history;
pub mod llm;
pub mod orchestrator;
pub mod planning;
pub mod plugins;
pub mod state;
pub mod task;
pub mod verification;

pub use context::ContextManager;
pub use execution::ExecutionEngine;
pub use llm::{LLMEngine, MockLLMEngine, RiaLLMEngine};
pub use orchestrator::AgentOrchestrator;
pub use planning::Plan;
pub use state::AgentState;
pub use task::Task;
pub use verification::VerificationEngine;
