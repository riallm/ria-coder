//! Build Tools (SPEC-033)

use crate::registry::{Tool, ToolCategory, ToolOutput};
use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;

pub struct BuildTools;

impl BuildTools {
    pub fn new() -> Self {
        Self
    }

    fn run_cargo(&self, args: &[&str], cwd: Option<&str>) -> Result<(i32, String, String)> {
        let mut command = Command::new("cargo");
        command.args(args);
        if let Some(cwd) = cwd {
            command.current_dir(cwd);
        }
        let output = command.output()?;

        Ok((
            output.status.code().unwrap_or(0),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

impl Tool for BuildTools {
    fn name(&self) -> &str {
        "build"
    }
    fn description(&self) -> &str {
        "Build system operations"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Build
    }
    fn is_available(&self) -> bool {
        Command::new("cargo").arg("--version").output().is_ok()
    }

    fn execute(&self, args: &HashMap<String, String>) -> Result<ToolOutput> {
        let action = args.get("action").map(|s| s.as_str()).unwrap_or("build");

        let start = std::time::Instant::now();
        let cwd = args.get("cwd").map(|s| s.as_str());

        let (exit_code, stdout, stderr) = match action {
            "build" => self.run_cargo(&["build"], cwd)?,
            "check" => self.run_cargo(&["check"], cwd)?,
            "clean" => self.run_cargo(&["clean"], cwd)?,
            _ => return Err(anyhow::anyhow!("Unknown build action: {}", action)),
        };

        Ok(ToolOutput {
            exit_code,
            stdout,
            stderr,
            duration: start.elapsed(),
        })
    }
}
