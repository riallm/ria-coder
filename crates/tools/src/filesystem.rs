//! File System Tools (SPEC-031)

use anyhow::Result;
use std::collections::HashMap;
use crate::registry::ToolOutput;

pub struct FileSystemTools;

impl FileSystemTools {
    pub fn new() -> Self { Self }

    pub fn read(&self, path: &str) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    pub fn write(&self, path: &str, content: &str) -> Result<()> {
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn execute(&self, _args: &HashMap<String, String>) -> Result<ToolOutput> {
        Ok(ToolOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration: std::time::Duration::ZERO,
        })
    }
}
