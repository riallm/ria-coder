//! Process Execution Tools

use crate::registry::{Tool, ToolCategory, ToolOutput};
use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;

pub struct ProcessTools;

impl ProcessTools {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for ProcessTools {
    fn name(&self) -> &str {
        "process"
    }
    fn description(&self) -> &str {
        "Execute system commands"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Build
    }
    fn is_available(&self) -> bool {
        true
    }

    fn execute(&self, args: &HashMap<String, String>) -> Result<ToolOutput> {
        let command_str = args
            .get("command")
            .ok_or_else(|| anyhow::anyhow!("Missing command argument"))?;
        let args_str = args.get("args").cloned().unwrap_or_default();

        let start = std::time::Instant::now();

        let mut command = Command::new(command_str);
        for arg in args_str.split_whitespace() {
            command.arg(arg);
        }

        match command.output() {
            Ok(output) => Ok(ToolOutput {
                exit_code: output.status.code().unwrap_or(0),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration: start.elapsed(),
            }),
            Err(e) => Ok(ToolOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: e.to_string(),
                duration: start.elapsed(),
            }),
        }
    }
}
