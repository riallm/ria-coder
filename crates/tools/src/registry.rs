//! Tool Registry (SPEC-030)

use anyhow::Result;
use std::collections::HashMap;

/// Tool categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display)]
pub enum ToolCategory {
    FileSystem,
    VersionControl,
    Build,
    Test,
    Analysis,
    Search,
}

/// Tool output
#[derive(Debug)]
pub struct ToolOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: std::time::Duration,
}

/// Tool trait
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn category(&self) -> ToolCategory;
    fn execute(&self, args: &HashMap<String, String>) -> Result<ToolOutput>;
    fn is_available(&self) -> bool;
}

/// Tool Registry
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    categories: HashMap<ToolCategory, Vec<String>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            categories: HashMap::new(),
        };

        registry.register(Box::new(crate::filesystem::FileSystemTools::new()));
        registry.register(Box::new(crate::process::ProcessTools::new()));
        registry.register(Box::new(crate::git::GitTools::new()));
        registry.register(Box::new(crate::build::BuildTools::new()));
        registry.register(Box::new(crate::test::TestTools::new()));

        registry
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        let category = tool.category();
        self.categories
            .entry(category)
            .or_default()
            .push(name.clone());
        self.tools.insert(name, tool);
    }

    pub fn execute(&self, name: &str, args: &HashMap<String, String>) -> Result<ToolOutput> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;
        tool.execute(args)
    }
}
