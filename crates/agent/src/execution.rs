//! Execution Engine (SPEC-024)

use crate::planning::{Plan, PlanAction};
use anyhow::Result;
use chrono::{DateTime, Utc};
use ria_tools::registry::ToolRegistry;
use std::collections::HashMap;
use std::path::PathBuf;

/// Change tracking
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeSet {
    pub changes: Vec<FileChange>,
    pub backup: Option<BackupInfo>,
    pub timestamp: DateTime<Utc>,
    pub root: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileChange {
    pub path: String,
    pub original: String,
    pub modified: String,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackupInfo {
    pub method: BackupMethod,
    pub reference: String,
    pub timestamp: DateTime<Utc>,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupMethod {
    GitStash,
    FileBackup,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionState {
    pub current_step: usize,
    pub total_steps: usize,
    pub changes: ChangeSet,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionRecord {
    pub step_index: usize,
    pub description: String,
    pub tool: String,
    pub status: StepStatus,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Completed,
    Failed { error: String },
    Skipped { reason: String },
}

impl ChangeSet {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            backup: None,
            timestamp: Utc::now(),
            root: None,
        }
    }

    pub fn with_root(root: Option<PathBuf>) -> Self {
        Self {
            root,
            ..Self::new()
        }
    }

    pub fn file_count(&self) -> usize {
        self.changes.len()
    }

    /// Rollback changes in this set (SPEC-042)
    pub fn rollback(&self) -> Result<()> {
        for change in &self.changes {
            let path = self.resolve_change_path(&change.path);
            match change.change_type {
                ChangeType::Created => {
                    if path.exists() {
                        std::fs::remove_file(&path)?;
                    }
                }
                ChangeType::Modified | ChangeType::Deleted => {
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&path, &change.original)?;
                }
            }
        }
        Ok(())
    }

    fn resolve_change_path(&self, path: &str) -> PathBuf {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            path
        } else if let Some(root) = &self.root {
            root.join(path)
        } else {
            path
        }
    }
}

/// Execution engine
pub struct ExecutionEngine {
    pub tools: ToolRegistry,
    pub state: ExecutionState,
    pub history: Vec<ExecutionRecord>,
    root: Option<PathBuf>,
}

impl ExecutionEngine {
    pub fn new() -> Self {
        let changes = ChangeSet::new();
        Self {
            tools: ToolRegistry::new(),
            state: ExecutionState {
                current_step: 0,
                total_steps: 0,
                changes,
                errors: Vec::new(),
            },
            history: Vec::new(),
            root: None,
        }
    }

    pub fn set_root(&mut self, root: PathBuf) {
        self.tools.set_working_dir(root.clone());
        self.root = Some(root);
    }

    /// Execute a plan
    pub async fn execute(&mut self, plan: &Plan) -> Result<ChangeSet> {
        let mut changeset = ChangeSet::with_root(self.root.clone());
        self.history.clear();
        self.state = ExecutionState {
            current_step: 0,
            total_steps: plan.steps.len(),
            changes: changeset.clone(),
            errors: Vec::new(),
        };
        let mut completed = vec![false; plan.steps.len()];

        for (index, step) in plan.steps.iter().enumerate() {
            self.state.current_step = index;
            let step_start = std::time::Instant::now();

            if let Some(missing) = step.depends_on.iter().find(|dependency| {
                **dependency >= completed.len()
                    || !completed.get(**dependency).copied().unwrap_or(false)
            }) {
                let reason = format!("Dependency step {} has not completed", missing);
                self.history.push(ExecutionRecord {
                    step_index: index,
                    description: step.description.clone(),
                    tool: "none".to_string(),
                    status: StepStatus::Skipped {
                        reason: reason.clone(),
                    },
                    duration: step_start.elapsed(),
                });
                return Err(anyhow::anyhow!(reason));
            }

            let mut tool_name = "none".to_string();
            let result: Result<()> = match &step.action {
                PlanAction::ReadFile { path } => {
                    tool_name = "filesystem".to_string();
                    let mut args = HashMap::new();
                    args.insert("path".to_string(), path.clone());
                    args.insert("action".to_string(), "read".to_string());
                    let output = self.tools.execute("filesystem", &args)?;
                    ensure_success("filesystem read", &output)?;
                    Ok(())
                }
                PlanAction::CreateFile { path, content } => {
                    tool_name = "filesystem".to_string();
                    let mut args = HashMap::new();
                    args.insert("path".to_string(), path.clone());
                    args.insert("content".to_string(), content.clone());
                    args.insert("action".to_string(), "create".to_string());
                    let output = self.tools.execute("filesystem", &args)?;
                    ensure_success("filesystem create", &output)?;

                    changeset.changes.push(FileChange {
                        path: path.clone(),
                        original: String::new(),
                        modified: content.clone(),
                        change_type: ChangeType::Created,
                    });
                    Ok(())
                }
                PlanAction::EditFile { path, edit } => {
                    tool_name = "filesystem".to_string();
                    let resolved = self.resolve_path(path);
                    let original = std::fs::read_to_string(&resolved)?;
                    let old_text = edit
                        .old_text
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Edit requires old_text"))?;
                    let new_text = edit
                        .new_text
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Edit requires new_text"))?;

                    let mut args = HashMap::new();
                    args.insert("path".to_string(), path.clone());
                    args.insert("action".to_string(), "edit".to_string());
                    args.insert("old_text".to_string(), old_text.clone());
                    args.insert("new_text".to_string(), new_text.clone());
                    args.insert("replace_all".to_string(), edit.replace_all.to_string());
                    let output = self.tools.execute("filesystem", &args)?;
                    ensure_success("filesystem edit", &output)?;
                    let modified = std::fs::read_to_string(&resolved)?;

                    changeset.changes.push(FileChange {
                        path: path.clone(),
                        original,
                        modified,
                        change_type: ChangeType::Modified,
                    });
                    Ok(())
                }
                PlanAction::DeleteFile { path } => {
                    tool_name = "filesystem".to_string();
                    let resolved = self.resolve_path(path);
                    let original = std::fs::read_to_string(&resolved)?;

                    let mut args = HashMap::new();
                    args.insert("path".to_string(), path.clone());
                    args.insert("action".to_string(), "delete".to_string());
                    let output = self.tools.execute("filesystem", &args)?;
                    ensure_success("filesystem delete", &output)?;

                    changeset.changes.push(FileChange {
                        path: path.clone(),
                        original,
                        modified: String::new(),
                        change_type: ChangeType::Deleted,
                    });
                    Ok(())
                }
                PlanAction::RunCommand { command, args } => {
                    let mut tool_args = HashMap::new();
                    match command.as_str() {
                        "cargo" if args.first().map(|value| value.as_str()) == Some("test") => {
                            tool_name = "test".to_string();
                            if let Some(target) = args.get(1) {
                                tool_args.insert("target".to_string(), target.clone());
                            }
                            let output = self.tools.execute("test", &tool_args)?;
                            ensure_success("cargo test", &output)?;
                        }
                        "cargo"
                            if matches!(
                                args.first().map(|value| value.as_str()),
                                Some("build" | "check" | "clean")
                            ) =>
                        {
                            tool_name = "build".to_string();
                            let action =
                                args.first().cloned().unwrap_or_else(|| "build".to_string());
                            tool_args.insert("action".to_string(), action);
                            let output = self.tools.execute("build", &tool_args)?;
                            ensure_success("cargo build", &output)?;
                        }
                        _ => {
                            tool_name = "process".to_string();
                            tool_args.insert("command".to_string(), command.clone());
                            tool_args.insert("args".to_string(), args.join(" "));
                            let output = self.tools.execute("process", &tool_args)?;
                            ensure_success(command, &output)?;
                        }
                    }
                    Ok(())
                }
                PlanAction::Search { pattern } => {
                    tool_name = "search".to_string();
                    let mut args = HashMap::new();
                    args.insert("action".to_string(), "content".to_string());
                    args.insert("pattern".to_string(), pattern.clone());
                    let output = self.tools.execute("search", &args)?;
                    ensure_success("search", &output)?;
                    Ok(())
                }
                PlanAction::Analyze { .. } => Ok(()),
            };

            match result {
                Ok(()) => {
                    completed[index] = true;
                    self.history.push(ExecutionRecord {
                        step_index: index,
                        description: step.description.clone(),
                        tool: tool_name,
                        status: StepStatus::Completed,
                        duration: step_start.elapsed(),
                    });
                }
                Err(error) => {
                    let message = error.to_string();
                    self.state.errors.push(message.clone());
                    self.history.push(ExecutionRecord {
                        step_index: index,
                        description: step.description.clone(),
                        tool: tool_name,
                        status: StepStatus::Failed {
                            error: message.clone(),
                        },
                        duration: step_start.elapsed(),
                    });
                    return Err(error);
                }
            }

            self.state.changes = changeset.clone();
        }

        Ok(changeset)
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            path
        } else if let Some(root) = &self.root {
            root.join(path)
        } else {
            path
        }
    }
}

fn ensure_success(label: &str, output: &ria_tools::registry::ToolOutput) -> Result<()> {
    if output.exit_code == 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "{} failed with exit code {}: {}{}",
            label,
            output.exit_code,
            output.stderr,
            output.stdout
        ))
    }
}
