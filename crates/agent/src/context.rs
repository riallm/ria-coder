//! Context Manager (SPEC-027)

use crate::task::Task;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;

/// Project Information (SPEC-027 Section 7)
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub name: String,
    pub language: String,
}

/// File Context (SPEC-027 Section 4)
#[derive(Debug, Clone)]
pub struct FileContext {
    pub path: PathBuf,
    pub content: String,
    pub language: String,
    pub relevance: f32,
    pub last_accessed: DateTime<Utc>,
}

/// Symbol Information
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolInfo {
    pub name: String,
    pub path: PathBuf,
    pub kind: SymbolKind,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Constant,
}

/// Context for a task (SPEC-027 Section 3)
pub struct AgentContext {
    pub system_prompt: String,
    pub files: HashMap<PathBuf, FileContext>,
    pub symbols: Vec<SymbolInfo>,
    pub project_info: ProjectInfo,
}

/// Agent working memory
pub struct ContextManager {
    pub project_info: Option<ProjectInfo>,
    pub max_context_files: usize,
    pub budget_tokens: usize,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            project_info: None,
            max_context_files: 20,
            budget_tokens: 128_000,
        }
    }

    /// Initialize with project root
    pub fn init(&mut self, root: PathBuf) -> Result<()> {
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.project_info = Some(ProjectInfo {
            root,
            name,
            language: "rust".to_string(), // Default to Rust for now
        });

        Ok(())
    }

    /// Build context for a task (SPEC-027 Section 6)
    pub fn build_for_task(&self, _task: &Task) -> Result<AgentContext> {
        let project_info = self
            .project_info
            .clone()
            .ok_or_else(|| anyhow::anyhow!("ContextManager not initialized with project root"))?;

        Ok(AgentContext {
            system_prompt: format!(
                "You are a coding assistant in a terminal environment. Working on project: {}",
                project_info.name
            ),
            files: HashMap::new(),
            symbols: Vec::new(),
            project_info,
        })
    }

    /// Scan project for files (SPEC-050 Section 2)
    pub fn scan_project(&self) -> Result<Vec<PathBuf>> {
        let info = self
            .project_info
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not initialized"))?;

        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(&info.root)
            .into_iter()
            .filter_map(|e: Result<walkdir::DirEntry, walkdir::Error>| e.ok())
            .filter(|e: &walkdir::DirEntry| e.file_type().is_file())
        {
            files.push(entry.path().to_path_buf());
        }

        Ok(files)
    }
}
