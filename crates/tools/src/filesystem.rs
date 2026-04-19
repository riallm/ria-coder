//! File System Tools (SPEC-031)

use crate::registry::{Tool, ToolCategory, ToolOutput};
use anyhow::Result;
use std::collections::HashMap;

pub struct FileSystemTools;

impl FileSystemTools {
    pub fn new() -> Self {
        Self
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

    fn execute(&self, args: &HashMap<String, String>) -> Result<ToolOutput> {
        let action = args
            .get("action")
            .ok_or_else(|| anyhow::anyhow!("Missing action argument"))?;
        let path = args
            .get("path")
            .ok_or_else(|| anyhow::anyhow!("Missing path argument"))?;

        let start = std::time::Instant::now();

        let (stdout, stderr, exit_code) = match action.as_str() {
            "read" => match std::fs::read_to_string(path) {
                Ok(content) => (content, String::new(), 0),
                Err(e) => (String::new(), e.to_string(), 1),
            },
            "write" => {
                let content = args
                    .get("content")
                    .ok_or_else(|| anyhow::anyhow!("Missing content argument"))?;
                match std::fs::write(path, content) {
                    Ok(_) => (format!("Wrote to {}", path), String::new(), 0),
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
