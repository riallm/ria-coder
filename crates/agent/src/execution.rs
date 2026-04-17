//! Execution Engine (SPEC-024)

use anyhow::Result;
use crate::planning::Plan;

/// Change tracking
#[derive(Debug)]
pub struct ChangeSet {
    changes: Vec<FileChange>,
}

#[derive(Debug)]
pub struct FileChange {
    pub path: String,
    pub original: String,
    pub modified: String,
}

impl ChangeSet {
    pub fn file_count(&self) -> usize { self.changes.len() }
}

/// Execution engine
pub struct ExecutionEngine;

impl ExecutionEngine {
    pub fn new() -> Self { Self }

    /// Execute a plan
    pub fn execute(&self, plan: &Plan) -> Result<ChangeSet> {
        Ok(ChangeSet { changes: Vec::new() })
    }
}
