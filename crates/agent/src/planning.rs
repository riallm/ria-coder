//! Planning Engine (SPEC-023)

use crate::context::AgentContext;
use crate::task::{TargetSpec, Task};
use anyhow::Result;

/// Ordered execution plan
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub task: Task,
    pub steps: Vec<PlanStep>,
    pub estimated_duration: std::time::Duration,
    pub risk_level: RiskLevel,
}

/// Single plan step
#[derive(Debug, Clone, PartialEq)]
pub struct PlanStep {
    pub description: String,
    pub action: PlanAction,
    pub target: Option<TargetSpec>,
    pub depends_on: Vec<usize>,
}

/// Edit specification
#[derive(Debug, Clone, PartialEq)]
pub struct EditSpec {
    pub description: String,
    pub diff: Option<String>,
}

/// Step action type
#[derive(Debug, Clone, PartialEq)]
pub enum PlanAction {
    ReadFile { path: String },
    EditFile { path: String, edit: EditSpec },
    CreateFile { path: String, content: String },
    RunCommand { command: String, args: Vec<String> },
    Search { pattern: String },
    Analyze { target: String },
}

/// Risk assessment
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Task planner
pub struct TaskPlanner;

impl TaskPlanner {
    pub fn new() -> Self {
        Self
    }

    /// Parse input to task
    pub fn parse(&self, input: &str) -> Result<Task> {
        crate::task::TaskParser::parse(input)
    }

    /// Generate plan from task
    pub fn generate(&self, task: &Task, _context: &AgentContext) -> Result<Plan> {
        use crate::task::TaskIntent;
        let mut steps = Vec::new();

        match &task.intent {
            TaskIntent::Explain => {
                steps.push(PlanStep {
                    description: "Analyze current code and explain".to_string(),
                    action: PlanAction::Analyze {
                        target: "current context".to_string(),
                    },
                    target: None,
                    depends_on: Vec::new(),
                });
            }
            TaskIntent::Create { path, description } => {
                steps.push(PlanStep {
                    description: format!("Create new file: {}", path),
                    action: PlanAction::CreateFile {
                        path: path.clone(),
                        content: format!("// Generated based on: {}", description),
                    },
                    target: Some(TargetSpec {
                        path: Some(path.clone()),
                        symbol: None,
                        line_range: None,
                    }),
                    depends_on: Vec::new(),
                });
                steps.push(PlanStep {
                    description: "Verify file creation".to_string(),
                    action: PlanAction::RunCommand {
                        command: "ls".to_string(),
                        args: vec![path.clone()],
                    },
                    target: None,
                    depends_on: vec![0],
                });
            }
            TaskIntent::Test { target } => {
                steps.push(PlanStep {
                    description: format!("Run tests for: {}", target),
                    action: PlanAction::RunCommand {
                        command: "cargo".to_string(),
                        args: vec!["test".to_string(), target.clone()],
                    },
                    target: None,
                    depends_on: Vec::new(),
                });
            }
            _ => {
                // Default: just a placeholder modify step
                steps.push(PlanStep {
                    description: "Analyze task and propose changes".to_string(),
                    action: PlanAction::Analyze {
                        target: "task description".to_string(),
                    },
                    target: None,
                    depends_on: Vec::new(),
                });
            }
        }

        Ok(Plan {
            task: task.clone(),
            steps,
            estimated_duration: std::time::Duration::from_secs(10),
            risk_level: RiskLevel::Low,
        })
    }
}
