//! Git Tools (SPEC-032)

use crate::registry::{Tool, ToolCategory, ToolOutput};
use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;

pub struct GitTools;

impl GitTools {
    pub fn new() -> Self {
        Self
    }

    fn run_git(&self, args: &[&str]) -> Result<(i32, String, String)> {
        let output = Command::new("git").args(args).output()?;

        Ok((
            output.status.code().unwrap_or(0),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

impl Tool for GitTools {
    fn name(&self) -> &str {
        "git"
    }
    fn description(&self) -> &str {
        "Version control operations"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::VersionControl
    }
    fn is_available(&self) -> bool {
        Command::new("git").arg("--version").output().is_ok()
    }

    fn execute(&self, args: &HashMap<String, String>) -> Result<ToolOutput> {
        let action = args
            .get("action")
            .ok_or_else(|| anyhow::anyhow!("Missing action argument"))?;

        let start = std::time::Instant::now();

        let (exit_code, stdout, stderr) = match action.as_str() {
            "status" => self.run_git(&["status", "--short"])?,
            "diff" => {
                let path = args.get("path").map(|s| s.as_str()).unwrap_or(".");
                self.run_git(&["diff", path])?
            }
            "add" => {
                let paths = args
                    .get("paths")
                    .ok_or_else(|| anyhow::anyhow!("Missing paths"))?;
                self.run_git(&["add", paths])?
            }
            "commit" => {
                let message = args
                    .get("message")
                    .ok_or_else(|| anyhow::anyhow!("Missing message"))?;
                self.run_git(&["commit", "-m", message])?
            }
            "log" => {
                let count = args.get("count").map(|s| s.as_str()).unwrap_or("10");
                self.run_git(&["log", "-n", count, "--oneline"])?
            }
            "stash_push" => {
                let message = args
                    .get("message")
                    .map(|s| s.as_str())
                    .unwrap_or("ria-coder backup");
                self.run_git(&["stash", "push", "-m", message])?
            }
            "stash_pop" => self.run_git(&["stash", "pop"])?,
            _ => return Err(anyhow::anyhow!("Unknown git action: {}", action)),
        };

        Ok(ToolOutput {
            exit_code,
            stdout,
            stderr,
            duration: start.elapsed(),
        })
    }
}
