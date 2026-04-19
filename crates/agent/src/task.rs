//! Task Parser (SPEC-022)

use anyhow::Result;

/// Structured task representation
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub intent: TaskIntent,
    pub targets: Vec<TargetSpec>,
    pub constraints: Vec<Constraint>,
    pub context_hints: Vec<String>,
}

/// What the user wants to do
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct TargetSpec {
    pub path: Option<String>,
    pub symbol: Option<String>,
    pub line_range: Option<(usize, usize)>,
}

/// Task constraints
#[derive(Debug, Clone, PartialEq)]
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
        let input_lower = input.to_lowercase();

        let intent = if input_lower.starts_with("explain") {
            TaskIntent::Explain
        } else if input_lower.starts_with("create") {
            let path = input
                .split_whitespace()
                .nth(1)
                .unwrap_or("new_file.rs")
                .to_string();
            TaskIntent::Create {
                path,
                description: input.to_string(),
            }
        } else if input_lower.starts_with("refactor") {
            TaskIntent::Refactor {
                description: input.to_string(),
            }
        } else if input_lower.starts_with("debug") {
            TaskIntent::Debug {
                symptom: input.to_string(),
            }
        } else if input_lower.starts_with("test") {
            let target = input.split_whitespace().nth(1).unwrap_or("").to_string();
            TaskIntent::Test { target }
        } else {
            TaskIntent::Modify {
                description: input.to_string(),
            }
        };

        Ok(Task {
            intent,
            targets: Vec::new(),
            constraints: Vec::new(),
            context_hints: Vec::new(),
        })
    }
}
