//! Search Tools (SPEC-030)

use crate::registry::{Tool, ToolCategory, ToolOutput, ToolParam};
use anyhow::Result;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct SearchTools;

impl SearchTools {
    pub fn new() -> Self {
        Self
    }

    fn root(args: &HashMap<String, String>) -> PathBuf {
        args.get("path")
            .or_else(|| args.get("cwd"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn relative_display(root: &Path, path: &Path) -> String {
        path.strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }
}

impl Tool for SearchTools {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "Search code and project files"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Search
    }

    fn execute(&self, args: &HashMap<String, String>) -> Result<ToolOutput> {
        let action = args.get("action").map(|s| s.as_str()).unwrap_or("content");
        let pattern = args
            .get("pattern")
            .ok_or_else(|| anyhow::anyhow!("Missing pattern argument"))?;
        let root = Self::root(args);
        let start = std::time::Instant::now();

        let mut matches = Vec::new();
        let max_results = args
            .get("max_results")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(100);

        match action {
            "files" => {
                for entry in WalkBuilder::new(&root)
                    .hidden(false)
                    .git_ignore(true)
                    .build()
                    .filter_map(|entry| entry.ok())
                {
                    if matches.len() >= max_results {
                        break;
                    }
                    if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        let path = entry.path();
                        let display = Self::relative_display(&root, path);
                        if display.contains(pattern) {
                            matches.push(display);
                        }
                    }
                }
            }
            "content" => {
                for entry in WalkBuilder::new(&root)
                    .hidden(false)
                    .git_ignore(true)
                    .build()
                    .filter_map(|entry| entry.ok())
                {
                    if matches.len() >= max_results {
                        break;
                    }
                    if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        continue;
                    }

                    let path = entry.path();
                    let Ok(content) = std::fs::read_to_string(path) else {
                        continue;
                    };
                    for (index, line) in content.lines().enumerate() {
                        if line.contains(pattern) {
                            matches.push(format!(
                                "{}:{}:{}",
                                Self::relative_display(&root, path),
                                index + 1,
                                line.trim()
                            ));
                            if matches.len() >= max_results {
                                break;
                            }
                        }
                    }
                }
            }
            _ => return Err(anyhow::anyhow!("Unknown search action: {}", action)),
        }

        Ok(ToolOutput {
            exit_code: 0,
            stdout: matches.join("\n"),
            stderr: String::new(),
            duration: start.elapsed(),
        })
    }

    fn is_available(&self) -> bool {
        true
    }

    fn parameters(&self) -> Vec<ToolParam> {
        vec![
            ToolParam {
                name: "action".to_string(),
                description: "content or files".to_string(),
                required: false,
            },
            ToolParam {
                name: "pattern".to_string(),
                description: "Substring to find".to_string(),
                required: true,
            },
        ]
    }
}
