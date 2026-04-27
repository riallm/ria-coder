//! File System Tools (SPEC-031)

use crate::registry::{Tool, ToolCategory, ToolOutput, ToolParam};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct FileSystemTools;

impl FileSystemTools {
    pub fn new() -> Self {
        Self
    }

    fn resolve_path(args: &HashMap<String, String>) -> Result<PathBuf> {
        let path = args
            .get("path")
            .ok_or_else(|| anyhow::anyhow!("Missing path argument"))?;
        let path = PathBuf::from(path);
        if path.is_absolute() {
            Ok(path)
        } else if let Some(cwd) = args.get("cwd") {
            Ok(Path::new(cwd).join(path))
        } else {
            Ok(path)
        }
    }

    fn read_range(content: &str, start: Option<usize>, end: Option<usize>) -> String {
        match (start, end) {
            (Some(start), Some(end)) => content
                .lines()
                .enumerate()
                .filter_map(|(index, line)| {
                    let line_no = index + 1;
                    (line_no >= start && line_no <= end)
                        .then(|| format!("{:>4}: {}", line_no, line))
                })
                .collect::<Vec<_>>()
                .join("\n"),
            _ => content.to_string(),
        }
    }

    fn simple_diff(original: &str, modified: &str) -> String {
        let mut diff = String::new();
        for line in original.lines() {
            if !modified.lines().any(|candidate| candidate == line) {
                diff.push_str(&format!("-{}\n", line));
            }
        }
        for line in modified.lines() {
            if !original.lines().any(|candidate| candidate == line) {
                diff.push_str(&format!("+{}\n", line));
            }
        }
        diff
    }
}

impl Tool for FileSystemTools {
    fn name(&self) -> &str {
        "filesystem"
    }
    fn description(&self) -> &str {
        "Read and write files"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }
    fn is_available(&self) -> bool {
        true
    }

    fn parameters(&self) -> Vec<ToolParam> {
        vec![
            ToolParam {
                name: "action".to_string(),
                description: "read, write, edit, create, delete, or list".to_string(),
                required: true,
            },
            ToolParam {
                name: "path".to_string(),
                description: "Path relative to the project root unless absolute".to_string(),
                required: true,
            },
        ]
    }

    fn execute(&self, args: &HashMap<String, String>) -> Result<ToolOutput> {
        let action = args
            .get("action")
            .ok_or_else(|| anyhow::anyhow!("Missing action argument"))?;
        let path = Self::resolve_path(args)?;

        let start = std::time::Instant::now();

        let (stdout, stderr, exit_code) = match action.as_str() {
            "read" => match std::fs::read_to_string(&path) {
                Ok(content) => {
                    let range_start = args.get("start").and_then(|value| value.parse().ok());
                    let range_end = args.get("end").and_then(|value| value.parse().ok());
                    (
                        Self::read_range(&content, range_start, range_end),
                        String::new(),
                        0,
                    )
                }
                Err(e) => (String::new(), e.to_string(), 1),
            },
            "write" => {
                let content = args
                    .get("content")
                    .ok_or_else(|| anyhow::anyhow!("Missing content argument"))?;
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                match std::fs::write(&path, content) {
                    Ok(_) => (format!("Wrote to {}", path.display()), String::new(), 0),
                    Err(e) => (String::new(), e.to_string(), 1),
                }
            }
            "create" => {
                if path.exists() {
                    (
                        String::new(),
                        format!("File already exists: {}", path.display()),
                        1,
                    )
                } else {
                    let content = args.get("content").cloned().unwrap_or_default();
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    match std::fs::write(&path, content) {
                        Ok(_) => (format!("Created {}", path.display()), String::new(), 0),
                        Err(e) => (String::new(), e.to_string(), 1),
                    }
                }
            }
            "edit" => {
                let old_text = args
                    .get("old_text")
                    .ok_or_else(|| anyhow::anyhow!("Missing old_text argument"))?;
                let new_text = args
                    .get("new_text")
                    .ok_or_else(|| anyhow::anyhow!("Missing new_text argument"))?;
                let replace_all = args
                    .get("replace_all")
                    .map(|value| value == "true")
                    .unwrap_or(false);

                match std::fs::read_to_string(&path) {
                    Ok(original) => {
                        if !original.contains(old_text) {
                            (
                                String::new(),
                                format!("Text to replace was not found in {}", path.display()),
                                1,
                            )
                        } else {
                            let modified = if replace_all {
                                original.replace(old_text, new_text)
                            } else {
                                original.replacen(old_text, new_text, 1)
                            };
                            std::fs::write(&path, &modified)?;
                            (Self::simple_diff(&original, &modified), String::new(), 0)
                        }
                    }
                    Err(e) => (String::new(), e.to_string(), 1),
                }
            }
            "delete" => {
                let force = args
                    .get("force")
                    .map(|value| value == "true")
                    .unwrap_or(false);
                if path.is_dir() && !force {
                    (
                        String::new(),
                        format!(
                            "Refusing to delete directory without force: {}",
                            path.display()
                        ),
                        1,
                    )
                } else if path.is_dir() {
                    match std::fs::remove_dir_all(&path) {
                        Ok(_) => (format!("Deleted {}", path.display()), String::new(), 0),
                        Err(e) => (String::new(), e.to_string(), 1),
                    }
                } else {
                    match std::fs::remove_file(&path) {
                        Ok(_) => (format!("Deleted {}", path.display()), String::new(), 0),
                        Err(e) => (String::new(), e.to_string(), 1),
                    }
                }
            }
            "list" => {
                let pattern = args.get("pattern").map(|value| value.as_str());
                match std::fs::read_dir(&path) {
                    Ok(entries) => {
                        let mut paths = Vec::new();
                        for entry in entries.flatten() {
                            let display = entry.path().display().to_string();
                            if pattern
                                .map(|pattern| display.contains(pattern))
                                .unwrap_or(true)
                            {
                                paths.push(display);
                            }
                        }
                        paths.sort();
                        (paths.join("\n"), String::new(), 0)
                    }
                    Err(e) => (String::new(), e.to_string(), 1),
                }
            }
            _ => return Err(anyhow::anyhow!("Unknown action: {}", action)),
        };

        Ok(ToolOutput {
            exit_code,
            stdout,
            stderr,
            duration: start.elapsed(),
        })
    }
}
