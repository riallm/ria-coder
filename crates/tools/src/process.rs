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

    fn is_dangerous(command: &str, args: &[&str]) -> bool {
        let joined = args.join(" ");
        (command == "rm" && args.iter().any(|arg| *arg == "-rf" || *arg == "-fr"))
            || (command == "git" && joined.contains("push") && joined.contains("--force"))
            || (command == "chmod" && args.iter().any(|arg| *arg == "777"))
            || joined.contains("rm -rf")
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
        let command_args = args_str.split_whitespace().collect::<Vec<_>>();
        let allow_dangerous = args
            .get("allow_dangerous")
            .map(|value| value == "true")
            .unwrap_or(false);

        let start = std::time::Instant::now();
        if Self::is_dangerous(command_str, &command_args) && !allow_dangerous {
            return Ok(ToolOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: format!("Blocked dangerous command: {} {}", command_str, args_str),
                duration: start.elapsed(),
            });
        }

        let mut command = Command::new(command_str);
        if let Some(cwd) = args.get("cwd") {
            command.current_dir(cwd);
        }
        for arg in command_args {
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
