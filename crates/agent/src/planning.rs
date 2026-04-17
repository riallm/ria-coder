//! Planning Engine (SPEC-023)

use anyhow::Result;
use crate::task::{Task, TargetSpec};
use crate::context::AgentContext;

/// Ordered execution plan
#[derive(Debug, Clone)]
pub struct Plan {
    pub task: Task,
    pub steps: Vec<PlanStep>,
    pub estimated_duration: std::time::Duration,
    pub risk_level: RiskLevel,
}

/// Single plan step
#[derive(Debug, Clone)]
pub struct PlanStep {
    pub description: String,
    pub action: PlanAction,
    pub target: Option<TargetSpec>,
    pub depends_on: Vec<usize>,
}

/// Step action type
#[derive(Debug, Clone)]
pub enum PlanAction {
    ReadFile { path: String },
    EditFile { path: String },
    CreateFile { path: String },
    RunCommand { command: String, args: Vec<String> },
    Search { pattern: String },
    Analyze { target: String },
}

/// Risk assessment
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Task planner
pub struct TaskPlanner;

impl TaskPlanner {
    pub fn new() -> Self { Self }

    /// Parse input to task
    pub fn parse(&self, input: &str) -> Result<Task> {
        crate::task::TaskParser::parse(input)
    }

    /// Generate plan from task
    pub fn generate(&self, task: &Task, context: &AgentContext) -> Result<Plan> {
        Ok(Plan {
            task: task.clone(),
            steps: Vec::new(),
            estimated_duration: std::time::Duration::from_secs(30),
            risk_level: RiskLevel::Low,
        })
    }
}
