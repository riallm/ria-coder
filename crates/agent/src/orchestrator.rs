//! Agent Orchestrator (SPEC-020)

use crate::context::ContextManager;
use crate::execution::{ChangeSet, ExecutionEngine};
use crate::history::{ConversationHistory, MessageRole};
use crate::llm::LLMEngine;
use crate::planning::TaskPlanner;
use crate::plugins::PluginManager;
use crate::state::{
    AgentResult, AgentState, ErrorState, ExecutingState, PlanningState, PresentingState,
    UnderstandingState, VerifyingState,
};
use crate::task::TaskParser;
use crate::task::{Task, TaskIntent};
use crate::verification::VerificationEngine;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

/// Central agent orchestrator
pub struct AgentOrchestrator {
    pub state: AgentState,
    pub planner: TaskPlanner,
    pub executor: ExecutionEngine,
    pub verifier: VerificationEngine,
    pub llm: Box<dyn LLMEngine>,
    pub context: ContextManager,
    pub history: ConversationHistory,
    pub plugins: PluginManager,
    pub max_iterations: usize,
    pub last_changes: Option<ChangeSet>,
    pub last_task_summary: Option<String>,
    pub undo_stack: Vec<ChangeSet>,
    pub redo_stack: Vec<ChangeSet>,
    pub session_history: Vec<SessionChangeRecord>,
}

/// Output from agent processing
pub struct AgentOutput {
    pub success: bool,
    pub message: String,
    pub changes_made: usize,
    pub tests_passed: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionChangeRecord {
    pub summary: String,
    pub status: SessionChangeStatus,
    pub file_count: usize,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionChangeStatus {
    Presented,
    Accepted,
    Rejected,
    Undone,
    Redone,
}

impl AgentOrchestrator {
    pub fn new(llm: Box<dyn LLMEngine>) -> Self {
        let orchestrator = Self {
            state: AgentState::Idle,
            planner: TaskPlanner::new(),
            executor: ExecutionEngine::new(),
            verifier: VerificationEngine::new(),
            llm,
            context: ContextManager::new(),
            history: ConversationHistory::new(),
            plugins: PluginManager::new(),
            max_iterations: 5,
            last_changes: None,
            last_task_summary: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            session_history: Vec::new(),
        };

        orchestrator
    }

    /// Initialize with project root
    pub fn init(&mut self, root: PathBuf) -> Result<()> {
        self.context.init(root.clone())?;
        self.executor.set_root(root.clone());
        self.verifier.set_root(root);
        self.history = ConversationHistory::load_default().unwrap_or_else(|error| {
            warn!("Failed to load conversation history: {error}");
            ConversationHistory::new()
        });
        self.plugins.init_all()?;
        self.plugins.register_all_tools(&mut self.executor.tools);
        self.plugins.register_all_tools(&mut self.verifier.tools);
        Ok(())
    }

    /// Process a user request (SPEC-020 Section 4)
    pub async fn process_request(&mut self, input: &str) -> Result<AgentOutput> {
        self.history.add(MessageRole::User, input.to_string());
        self.persist_history();

        // 1. Understanding
        info!("Step 1: understanding request");
        self.update_state(AgentState::Understanding(UnderstandingState {
            request: input.to_string(),
            parsed: None,
        }));

        // Try LLM parsing, fallback to simple parsing
        let task = match TaskParser::parse_llm(input, self.llm.as_ref()).await {
            Ok(t) => {
                info!(?t.intent, "Parsed task via LLM");
                t
            }
            Err(e) => {
                warn!("LLM parsing failed: {e}. Falling back to simple parser.");
                self.planner.parse(input)?
            }
        };

        self.update_state(AgentState::Understanding(UnderstandingState {
            request: input.to_string(),
            parsed: Some(task.clone()),
        }));
        self.last_task_summary = Some(summarize_task(&task));

        if task_may_change_files(&task) {
            // Safety Layer 0: Auto-stash (SPEC-040)
            info!("Safety Layer 0: auto-stashing");
            self.auto_stash("ria-coder pre-task backup")?;
        }

        // 2. Planning
        info!("Step 2: planning");
        self.update_state(AgentState::Planning(PlanningState {
            task: task.clone(),
            plan: None,
        }));

        let context = self.context.build_for_task(&task)?;
        let plan_result = match self
            .planner
            .generate_with_llm(&task, &context, self.llm.as_ref())
            .await
        {
            Ok(plan) => {
                info!(steps = plan.steps.len(), "Generated LLM-backed plan");
                Ok(plan)
            }
            Err(e) => {
                info!("LLM-backed planning unavailable: {e}. Falling back to deterministic plan.");
                self.planner.generate(&task, &context)
            }
        };

        let plan = match plan_result {
            Ok(p) => {
                info!(steps = p.steps.len(), "Generated plan");
                self.update_state(AgentState::Planning(PlanningState {
                    task: task.clone(),
                    plan: Some(p.clone()),
                }));
                p
            }
            Err(e) => {
                self.handle_error(format!("Failed to generate plan: {}", e), false);
                return Err(e);
            }
        };

        // 3. Executing
        info!("Step 3: executing plan");
        self.update_state(AgentState::Executing(ExecutingState {
            plan: plan.clone(),
            current_step: 0,
        }));

        let changes = match self.executor.execute(&plan).await {
            Ok(c) => {
                info!(changes = c.file_count(), "Execution complete");
                c
            }
            Err(e) => {
                self.handle_error(format!("Execution failed: {}", e), true);
                return Err(e);
            }
        };
        self.last_changes = Some(changes.clone());

        // 4. Verifying
        info!("Step 4: verifying results");
        self.update_state(AgentState::Verifying(VerifyingState {
            changes: changes.clone(),
        }));

        let verification = match self.verifier.verify(&changes).await {
            Ok(v) => {
                info!(summary = %v.summary(), "Verification complete");
                v
            }
            Err(e) => {
                self.handle_error(format!("Verification failed: {}", e), true);
                return Err(e);
            }
        };

        // 5. Presenting
        let success = verification.passed();
        let message = verification.summary();
        self.history.add(MessageRole::Agent, message.clone());
        self.persist_history();
        if changes.file_count() > 0 {
            self.record_session_change(SessionChangeStatus::Presented, changes.file_count());
        }

        self.update_state(AgentState::Presenting(PresentingState {
            result: AgentResult {
                success,
                message: message.clone(),
            },
        }));

        Ok(AgentOutput {
            success,
            message,
            changes_made: changes.file_count(),
            tests_passed: verification.test_count(),
        })
    }

    /// Accept and commit changes (SPEC-041)
    pub fn accept_changes(&mut self) -> Result<()> {
        if let Some(changes) = self.last_changes.clone() {
            if !changes.changes.is_empty() {
                let paths = changes
                    .changes
                    .iter()
                    .map(|change| change.path.clone())
                    .collect::<Vec<_>>();

                let mut add_args = HashMap::new();
                add_args.insert("action".to_string(), "add".to_string());
                add_args.insert("paths_json".to_string(), serde_json::to_string(&paths)?);
                let add_output = self.executor.tools.execute("git", &add_args)?;
                if add_output.exit_code != 0 {
                    return Err(anyhow::anyhow!("git add failed: {}", add_output.stderr));
                }

                let mut commit_args = HashMap::new();
                commit_args.insert("action".to_string(), "commit".to_string());
                commit_args.insert(
                    "message".to_string(),
                    self.commit_message_for_changes(&changes),
                );
                let commit_output = self.executor.tools.execute("git", &commit_args)?;
                if commit_output.exit_code != 0 {
                    return Err(anyhow::anyhow!(
                        "git commit failed: {}",
                        commit_output.stderr
                    ));
                }
            }
            self.undo_stack.push(changes.clone());
            self.redo_stack.clear();
            self.record_session_change(SessionChangeStatus::Accepted, changes.file_count());
            self.last_changes = None;
            self.last_task_summary = None;
            self.update_state(AgentState::Idle);
        }
        Ok(())
    }

    /// Reject and rollback changes (SPEC-041, SPEC-042)
    pub fn reject_changes(&mut self) -> Result<()> {
        if let Some(changes) = self.last_changes.clone() {
            changes.rollback()?;
            self.redo_stack.push(changes.clone());
            self.record_session_change(SessionChangeStatus::Rejected, changes.file_count());
            self.last_changes = None;
            self.last_task_summary = None;
            self.update_state(AgentState::Idle);
        }
        Ok(())
    }

    /// Undo the latest pending or accepted change set (SPEC-042).
    pub fn undo_last(&mut self) -> Result<()> {
        if self.last_changes.is_some() {
            return self.reject_changes();
        }

        let Some(changes) = self.undo_stack.pop() else {
            return Err(anyhow::anyhow!("No change set available to undo"));
        };
        changes.rollback()?;
        self.record_session_change(SessionChangeStatus::Undone, changes.file_count());
        self.redo_stack.push(changes);
        self.update_state(AgentState::Idle);
        Ok(())
    }

    /// Re-apply the most recently undone change set (SPEC-042).
    pub fn redo_last(&mut self) -> Result<()> {
        let Some(changes) = self.redo_stack.pop() else {
            return Err(anyhow::anyhow!("No change set available to redo"));
        };
        changes.apply()?;
        self.record_session_change(SessionChangeStatus::Redone, changes.file_count());
        self.undo_stack.push(changes);
        self.update_state(AgentState::Idle);
        Ok(())
    }

    /// Reset all accepted changes tracked in this session.
    pub fn reset_session_changes(&mut self) -> Result<()> {
        if let Some(changes) = self.last_changes.clone() {
            changes.rollback()?;
            self.redo_stack.push(changes);
            self.last_changes = None;
        }

        while let Some(changes) = self.undo_stack.pop() {
            changes.rollback()?;
            self.redo_stack.push(changes);
        }
        self.last_task_summary = None;
        self.update_state(AgentState::Idle);
        Ok(())
    }

    /// Update agent state
    pub fn update_state(&mut self, state: AgentState) {
        self.state = state;
    }

    /// Handle error state
    fn handle_error(&mut self, message: String, can_retry: bool) {
        self.update_state(AgentState::Error(ErrorState { message, can_retry }));
    }

    /// Safety Layer 0: Auto-stash (SPEC-040)
    fn auto_stash(&self, message: &str) -> Result<()> {
        let mut args = HashMap::new();
        args.insert("action".to_string(), "stash_push".to_string());
        args.insert("message".to_string(), message.to_string());
        let _ = self.executor.tools.execute("git", &args); // Ignore errors if not a git repo
        Ok(())
    }

    fn commit_message_for_changes(&self, changes: &ChangeSet) -> String {
        let summary = self
            .last_task_summary
            .as_deref()
            .unwrap_or("applied changes");
        let change_summary = changes
            .changes
            .iter()
            .map(|change| format!("- {:?}: {}", change.change_type, change.path))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "ai: {summary}\n\nChanges:\n{change_summary}\n\nFiles modified: {}",
            changes.file_count()
        )
    }

    fn record_session_change(&mut self, status: SessionChangeStatus, file_count: usize) {
        self.session_history.push(SessionChangeRecord {
            summary: self
                .last_task_summary
                .clone()
                .unwrap_or_else(|| "session change".to_string()),
            status,
            file_count,
            timestamp: Utc::now(),
        });
    }

    fn persist_history(&self) {
        if let Err(error) = self.history.save_default() {
            warn!("Failed to persist conversation history: {error}");
        }
    }
}

fn task_may_change_files(task: &Task) -> bool {
    !matches!(task.intent, TaskIntent::Explain | TaskIntent::Review { .. })
}

fn summarize_task(task: &Task) -> String {
    match &task.intent {
        TaskIntent::Explain => "explain code".to_string(),
        TaskIntent::Modify { description }
        | TaskIntent::Refactor { description }
        | TaskIntent::Create { description, .. } => description.clone(),
        TaskIntent::Delete { path } => format!("delete {}", path),
        TaskIntent::Debug { symptom } => symptom.clone(),
        TaskIntent::Test { target } => format!("test {}", target),
        TaskIntent::Review { target } => format!("review {}", target),
        TaskIntent::Document { target } => format!("document {}", target),
    }
}
