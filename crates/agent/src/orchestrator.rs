//! Agent Orchestrator (SPEC-020)

use anyhow::Result;
use crate::state::AgentState;
use crate::task::Task;
use crate::planning::{Plan, TaskPlanner};
use crate::execution::ExecutionEngine;
use crate::verification::VerificationEngine;
use crate::llm::LLMEngine;
use crate::context::ContextManager;
use crate::history::ConversationHistory;

/// Central agent orchestrator
pub struct AgentOrchestrator {
    pub state: AgentState,
    pub planner: TaskPlanner,
    pub executor: ExecutionEngine,
    pub verifier: VerificationEngine,
    pub llm: Box<dyn LLMEngine>,
    pub context: ContextManager,
    pub history: ConversationHistory,
    pub max_iterations: usize,
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
        Self {
            state: AgentState::Idle,
            planner: TaskPlanner::new(),
            executor: ExecutionEngine::new(),
            verifier: VerificationEngine::new(),
            llm,
            context: ContextManager::new(),
            history: ConversationHistory::new(),
            max_iterations: 5,
        }
    }

    /// Process a user request
    pub async fn process_request(&mut self, input: &str) -> Result<AgentOutput> {
        // SPEC-020: Main loop
        // 1. Parse request
        let task = self.planner.parse(input)?;
        
        // 2. Build context
        let context = self.context.build_for_task(&task)?;
        
        // 3. Generate plan
        let plan = self.planner.generate(&task, &context)?;
        
        // 4. Execute plan
        let changes = self.executor.execute(&plan)?;
        
        // 5. Verify results
        let verification = self.verifier.verify(&changes)?;
        
        // 6. Return output
        Ok(AgentOutput {
            success: verification.passed(),
            message: verification.summary(),
            changes_made: changes.file_count(),
            tests_passed: verification.test_count(),
        })
    }

    /// Update agent state
    pub fn update_state(&mut self, state: AgentState) {
        self.state = state;
    }
}
