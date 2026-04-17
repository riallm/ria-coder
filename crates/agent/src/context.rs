//! Context Manager (SPEC-027)

use anyhow::Result;
use crate::task::Task;

/// Agent working memory
pub struct ContextManager {
    max_context_files: usize,
    budget_tokens: usize,
}

/// Context for a task
pub struct AgentContext {
    pub system_prompt: String,
    pub file_contents: Vec<FileContext>,
    pub symbol_index: Vec<SymbolInfo>,
}

pub struct FileContext {
    pub path: String,
    pub content: String,
    pub relevance: f32,
}

pub struct SymbolInfo {
    pub name: String,
    pub path: String,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Constant,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            max_context_files: 20,
            budget_tokens: 128_000,
        }
    }

    /// Build context for a task
    pub fn build_for_task(&self, task: &Task) -> Result<AgentContext> {
        Ok(AgentContext {
            system_prompt: "You are a coding assistant in a terminal environment.".to_string(),
            file_contents: Vec::new(),
            symbol_index: Vec::new(),
        })
    }
}
