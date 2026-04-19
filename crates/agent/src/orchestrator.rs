//! Agent Orchestrator (SPEC-020)

use crate::context::ContextManager;
use crate::execution::{ChangeSet, ExecutionEngine};
use crate::history::ConversationHistory;
use crate::llm::LLMEngine;
use crate::planning::TaskPlanner;
use crate::plugins::PluginManager;
use crate::state::{
    AgentResult, AgentState, ErrorState, ExecutingState, PlanningState, PresentingState,
    UnderstandingState, VerifyingState,
};
use crate::verification::VerificationEngine;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

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
}

/// Output from agent processing
pub struct AgentOutput {
    pub success: bool,
    pub message: String,
    pub changes_made: usize,
    pub tests_passed: Option<usize>,
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
        };

        // Initialize built-in plugins (placeholder)
        // orchestrator.plugins.register(Box::new(RustPlugin::new()));

        orchestrator
    }

    /// Initialize with project root
    pub fn init(&mut self, root: PathBuf) -> Result<()> {
        self.context.init(root)?;
        self.plugins.init_all()?;
        self.plugins.register_all_tools(&mut self.executor.tools);
        self.plugins.register_all_tools(&mut self.verifier.tools);
        Ok(())
    }

    /// Process a user request (SPEC-020 Section 4)
    pub async fn process_request(&mut self, input: &str) -> Result<AgentOutput> {
        // 0. Safety Layer 0: Auto-stash (SPEC-040)
        self.auto_stash("ria-coder pre-task backup")?;

        // 1. Understanding
        self.update_state(AgentState::Understanding(UnderstandingState {
            request: input.to_string(),
            parsed: None,
        }));

        let task = match self.planner.parse(input) {
            Ok(t) => {
                self.update_state(AgentState::Understanding(UnderstandingState {
                    request: input.to_string(),
                    parsed: Some(t.clone()),
                }));
                t
            }
            Err(e) => {
                self.handle_error(format!("Failed to parse task: {}", e), false);
                return Err(e);
            }
        };

        // 2. Planning
        self.update_state(AgentState::Planning(PlanningState {
            task: task.clone(),
            plan: None,
        }));

        let context = self.context.build_for_task(&task)?;
        let plan = match self.planner.generate(&task, &context) {
            Ok(p) => {
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
        self.update_state(AgentState::Executing(ExecutingState {
            plan: plan.clone(),
            current_step: 0,
        }));

        let changes = match self.executor.execute(&plan).await {
            Ok(c) => c,
            Err(e) => {
                self.handle_error(format!("Execution failed: {}", e), true);
                return Err(e);
            }
        };
        self.last_changes = Some(changes.clone());

        // 4. Verifying
        self.update_state(AgentState::Verifying(VerifyingState {
            changes: changes.clone(),
        }));

        let verification = match self.verifier.verify(&changes).await {
            Ok(v) => v,
            Err(e) => {
                self.handle_error(format!("Verification failed: {}", e), true);
                return Err(e);
            }
        };

        // 5. Presenting
        let success = verification.passed();
        let message = verification.summary();

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
        if let Some(_) = &self.last_changes {
            let mut args = HashMap::new();
            args.insert("action".to_string(), "commit".to_string());
            args.insert("message".to_string(), "ai: applied changes".to_string());
            let _ = self.executor.tools.execute("git", &args);
            self.last_changes = None;
            self.update_state(AgentState::Idle);
        }
        Ok(())
    }

    /// Reject and rollback changes (SPEC-041, SPEC-042)
    pub fn reject_changes(&mut self) -> Result<()> {
        if let Some(changes) = &self.last_changes {
            changes.rollback()?;
            self.last_changes = None;
            self.update_state(AgentState::Idle);
        }
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
}
