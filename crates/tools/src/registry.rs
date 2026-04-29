//! Tool Registry (SPEC-030)

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

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

/// Tool argument set (SPEC-030)
#[derive(Debug, Clone, Default)]
pub struct ToolArgs {
    pub named: HashMap<String, String>,
    pub positional: Vec<String>,
    pub flags: HashSet<String>,
}

impl ToolArgs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_named(named: HashMap<String, String>) -> Self {
        Self {
            named,
            ..Self::default()
        }
    }
}

/// Tool parameter metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolParam {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Tool output
#[derive(Debug, Clone)]
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
    fn parameters(&self) -> Vec<ToolParam> {
        Vec::new()
    }
}

/// Tool Registry
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    categories: HashMap<ToolCategory, Vec<String>>,
    working_dir: Option<PathBuf>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            categories: HashMap::new(),
            working_dir: None,
        };

        registry.register(Box::new(crate::filesystem::FileSystemTools::new()));
        registry.register(Box::new(crate::process::ProcessTools::new()));
        registry.register(Box::new(crate::git::GitTools::new()));
        registry.register(Box::new(crate::build::BuildTools::new()));
        registry.register(Box::new(crate::test::TestTools::new()));
        registry.register(Box::new(crate::lint::LintTools::new()));
        registry.register(Box::new(crate::search::SearchTools::new()));

        registry
    }

    pub fn set_working_dir(&mut self, working_dir: PathBuf) {
        self.working_dir = Some(working_dir);
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

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|tool| tool.as_ref())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub fn list(&self, category: Option<ToolCategory>) -> Vec<&dyn Tool> {
        match category {
            Some(category) => self
                .categories
                .get(&category)
                .into_iter()
                .flatten()
                .filter_map(|name| self.get(name))
                .collect(),
            None => self.tools.values().map(|tool| tool.as_ref()).collect(),
        }
    }

    pub fn execute(&self, name: &str, args: &HashMap<String, String>) -> Result<ToolOutput> {
        self.execute_named(name, args)
    }

    pub fn execute_args(&self, name: &str, args: &ToolArgs) -> Result<ToolOutput> {
        let mut named = args.named.clone();
        if !args.positional.is_empty() {
            named.insert("args".to_string(), args.positional.join(" "));
        }
        for flag in &args.flags {
            named.insert(flag.clone(), "true".to_string());
        }
        self.execute_named(name, &named)
    }

    fn execute_named(&self, name: &str, args: &HashMap<String, String>) -> Result<ToolOutput> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;

        if !tool.is_available() {
            return Err(anyhow::anyhow!("Tool not available: {}", name));
        }

        let mut effective_args = args.clone();
        if let Some(working_dir) = &self.working_dir {
            effective_args
                .entry("cwd".to_string())
                .or_insert_with(|| working_dir.to_string_lossy().to_string());
        }

        tool.execute(&effective_args)
    }
}
