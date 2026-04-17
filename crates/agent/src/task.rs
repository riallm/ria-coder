//! Task Parser (SPEC-022)

use anyhow::Result;

/// Structured task representation
#[derive(Debug, Clone)]
pub struct Task {
    pub intent: TaskIntent,
    pub targets: Vec<TargetSpec>,
    pub constraints: Vec<Constraint>,
    pub context_hints: Vec<String>,
}

/// What the user wants to do
#[derive(Debug, Clone)]
pub enum TaskIntent {
    Explain,
    Modify { description: String },
    Create { path: String, description: String },
    Delete { path: String },
    Refactor { description: String },
    Debug { symptom: String },
    Test { target: String },
    Review { target: String },
    Document { target: String },
}

/// Target file or symbol
#[derive(Debug, Clone)]
pub struct TargetSpec {
    pub path: Option<String>,
    pub symbol: Option<String>,
    pub line_range: Option<(usize, usize)>,
}

/// Task constraints
#[derive(Debug, Clone)]
pub enum Constraint {
    KeepApiStable,
    Language(String),
    Style(String),
    WithTests,
    Performance,
}

/// Task parser
pub struct TaskParser;

impl TaskParser {
    pub fn parse(input: &str) -> Result<Task> {
        // Parse natural language to structured task
        Ok(Task {
            intent: TaskIntent::Modify { description: input.to_string() },
            targets: Vec::new(),
            constraints: Vec::new(),
            context_hints: Vec::new(),
        })
    }
}
