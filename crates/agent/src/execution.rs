//! Execution Engine (SPEC-024)

use crate::planning::{Plan, PlanAction};
use anyhow::Result;
use ria_tools::registry::ToolRegistry;
use std::collections::HashMap;

/// Change tracking
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeSet {
    pub changes: Vec<FileChange>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileChange {
    pub path: String,
    pub original: String,
    pub modified: String,
}

impl ChangeSet {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    pub fn file_count(&self) -> usize {
        self.changes.len()
    }

    /// Rollback changes in this set (SPEC-042)
    pub fn rollback(&self) -> Result<()> {
        for change in &self.changes {
            if change.original.is_empty() {
                // Was a new file, delete it
                if std::path::Path::new(&change.path).exists() {
                    std::fs::remove_file(&change.path)?;
                }
            } else {
                // Restore original content
                std::fs::write(&change.path, &change.original)?;
            }
        }
        Ok(())
    }
}

/// Execution engine
pub struct ExecutionEngine {
    pub tools: ToolRegistry,
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self {
            tools: ToolRegistry::new(),
        }
    }

    /// Execute a plan
    pub async fn execute(&self, plan: &Plan) -> Result<ChangeSet> {
        let mut changeset = ChangeSet::new();

        for step in &plan.steps {
            // println!("Executing: {}", step.description);
            match &step.action {
                PlanAction::ReadFile { path } => {
                    let mut args = HashMap::new();
                    args.insert("path".to_string(), path.clone());
                    args.insert("action".to_string(), "read".to_string());
                    self.tools.execute("filesystem", &args)?;
                }
                PlanAction::CreateFile { path, content } => {
                    // Before creating, check if it exists for rollback info
                    let original = if std::path::Path::new(path).exists() {
                        std::fs::read_to_string(path).unwrap_or_default()
                    } else {
                        String::new()
                    };

                    let mut args = HashMap::new();
                    args.insert("path".to_string(), path.clone());
                    args.insert("content".to_string(), content.clone());
                    args.insert("action".to_string(), "write".to_string());
                    self.tools.execute("filesystem", &args)?;

                    changeset.changes.push(FileChange {
                        path: path.clone(),
                        original,
                        modified: content.clone(),
                    });
                }
                PlanAction::RunCommand { command, args } => {
                    let mut tool_args = HashMap::new();
                    tool_args.insert("command".to_string(), command.clone());
                    tool_args.insert("args".to_string(), args.join(" "));
                    self.tools.execute("process", &tool_args)?;
                }
                _ => {
                    // Other actions not yet implemented
                }
            }
        }

        Ok(changeset)
    }
}
