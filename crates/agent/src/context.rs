//! Context Manager (SPEC-027)

use crate::task::Task;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

/// Project Information (SPEC-027 Section 7)
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub name: String,
    pub language: String,
    pub build_system: String,
    pub test_framework: String,
    pub dependencies: Vec<String>,
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

        let language = detect_language(&root);
        let build_system = detect_build_system(&root);
        let test_framework = detect_test_framework(&root);
        let dependencies = detect_dependencies(&root).unwrap_or_default();

        self.project_info = Some(ProjectInfo {
            root,
            name,
            language,
            build_system,
            test_framework,
            dependencies,
        });

        Ok(())
    }

    /// Build context for a task (SPEC-027 Section 6)
    pub fn build_for_task(&self, task: &Task) -> Result<AgentContext> {
        let project_info = self
            .project_info
            .clone()
            .ok_or_else(|| anyhow::anyhow!("ContextManager not initialized with project root"))?;

        let mut files = HashMap::new();
        let mut symbols = Vec::new();
        let mut candidates = Vec::new();
        let mut seen = HashSet::new();

        for target in &task.targets {
            if let Some(path) = &target.path {
                let path = PathBuf::from(path);
                if seen.insert(path.clone()) {
                    candidates.push((path, 1.0));
                }
            }
        }

        for standard_file in ["Cargo.toml", "README.md", "src/lib.rs", "src/main.rs"] {
            let path = PathBuf::from(standard_file);
            if project_info.root.join(&path).exists() && seen.insert(path.clone()) {
                candidates.push((path, 0.5));
            }
        }

        if candidates.len() < self.max_context_files {
            for path in self.scan_project()? {
                if candidates.len() >= self.max_context_files {
                    break;
                }
                let Ok(relative) = path.strip_prefix(&project_info.root) else {
                    continue;
                };
                let relative = relative.to_path_buf();
                if seen.insert(relative.clone()) && is_context_file(&relative) {
                    candidates.push((relative, 0.25));
                }
            }
        }

        let mut approx_tokens = 0usize;
        for (relative_path, relevance) in candidates {
            if files.len() >= self.max_context_files || approx_tokens >= self.budget_tokens {
                break;
            }

            let absolute_path = if relative_path.is_absolute() {
                relative_path.clone()
            } else {
                project_info.root.join(&relative_path)
            };
            let Ok(metadata) = std::fs::metadata(&absolute_path) else {
                continue;
            };
            if metadata.len() > 1_000_000 {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&absolute_path) else {
                continue;
            };

            approx_tokens += content.len() / 4;
            symbols.extend(extract_symbols(&relative_path, &content));
            files.insert(
                relative_path.clone(),
                FileContext {
                    path: relative_path.clone(),
                    content,
                    language: language_for_path(&relative_path),
                    relevance,
                    last_accessed: Utc::now(),
                },
            );
        }

        Ok(AgentContext {
            system_prompt: format!(
                "You are a coding assistant in a terminal environment. Project: {}. Language: {}. Build: {}. Tests: {}.",
                project_info.name,
                project_info.language,
                project_info.build_system,
                project_info.test_framework
            ),
            files,
            symbols,
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
        for entry in ignore::WalkBuilder::new(&info.root)
            .hidden(false)
            .git_ignore(true)
            .build()
            .filter_map(|entry| entry.ok())
        {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                files.push(entry.path().to_path_buf());
            }
        }

        Ok(files)
    }
}

fn detect_language(root: &PathBuf) -> String {
    if root.join("Cargo.toml").exists() {
        "rust".to_string()
    } else if root.join("package.json").exists() {
        "javascript".to_string()
    } else if root.join("go.mod").exists() {
        "go".to_string()
    } else if root.join("pyproject.toml").exists() {
        "python".to_string()
    } else {
        "unknown".to_string()
    }
}

fn detect_build_system(root: &PathBuf) -> String {
    if root.join("Cargo.toml").exists() {
        "cargo".to_string()
    } else if root.join("Makefile").exists() {
        "make".to_string()
    } else if root.join("package.json").exists() {
        "npm".to_string()
    } else if root.join("go.mod").exists() {
        "go".to_string()
    } else {
        "none".to_string()
    }
}

fn detect_test_framework(root: &PathBuf) -> String {
    if root.join("Cargo.toml").exists() {
        "cargo test".to_string()
    } else if root.join("pytest.ini").exists() || root.join("pyproject.toml").exists() {
        "pytest".to_string()
    } else if root.join("package.json").exists() {
        "npm test".to_string()
    } else if root.join("go.mod").exists() {
        "go test".to_string()
    } else {
        "unknown".to_string()
    }
}

fn detect_dependencies(root: &PathBuf) -> Result<Vec<String>> {
    let cargo = root.join("Cargo.toml");
    if !cargo.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(cargo)?;
    let value: toml::Value = toml::from_str(&content)?;
    let dependencies = value
        .get("dependencies")
        .and_then(|deps| deps.as_table())
        .map(|deps| deps.keys().cloned().collect())
        .unwrap_or_default();
    Ok(dependencies)
}

fn is_context_file(path: &std::path::Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some(
            "rs" | "toml"
                | "md"
                | "json"
                | "yaml"
                | "yml"
                | "py"
                | "go"
                | "ts"
                | "tsx"
                | "js"
                | "jsx"
        )
    )
}

fn language_for_path(path: &std::path::Path) -> String {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("rs") => "rust",
        Some("py") => "python",
        Some("go") => "go",
        Some("ts" | "tsx") => "typescript",
        Some("js" | "jsx") => "javascript",
        Some("toml") => "toml",
        Some("md") => "markdown",
        Some("json") => "json",
        Some("yaml" | "yml") => "yaml",
        _ => "text",
    }
    .to_string()
}

fn extract_symbols(path: &PathBuf, content: &str) -> Vec<SymbolInfo> {
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim_start();
            let (kind, prefix) = if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
                (SymbolKind::Function, "fn ")
            } else if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
                (SymbolKind::Struct, "struct ")
            } else if trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ") {
                (SymbolKind::Enum, "enum ")
            } else if trimmed.starts_with("pub trait ") || trimmed.starts_with("trait ") {
                (SymbolKind::Trait, "trait ")
            } else if trimmed.starts_with("pub const ") || trimmed.starts_with("const ") {
                (SymbolKind::Constant, "const ")
            } else {
                return None;
            };

            let name = trimmed
                .split(prefix)
                .nth(1)?
                .split(|ch: char| !(ch.is_alphanumeric() || ch == '_'))
                .next()?
                .to_string();
            Some(SymbolInfo {
                name,
                path: path.clone(),
                kind,
                line: index + 1,
            })
        })
        .collect()
}
