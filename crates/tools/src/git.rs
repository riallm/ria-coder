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

    fn run_git(&self, args: &[&str], cwd: Option<&str>) -> Result<(i32, String, String)> {
        let mut command = Command::new("git");
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
        let cwd = args.get("cwd").map(|s| s.as_str());

        let (exit_code, stdout, stderr) = match action.as_str() {
            "status" => self.run_git(&["status", "--short"], cwd)?,
            "diff" => {
                let path = args.get("path").map(|s| s.as_str()).unwrap_or(".");
                self.run_git(&["diff", path], cwd)?
            }
            "diff_staged" => self.run_git(&["diff", "--staged"], cwd)?,
            "root" => self.run_git(&["rev-parse", "--show-toplevel"], cwd)?,
            "branch" => {
                if let Some(name) = args.get("name") {
                    self.run_git(&["checkout", "-b", name], cwd)?
                } else {
                    self.run_git(&["branch", "--show-current"], cwd)?
                }
            }
            "checkout" => {
                let name = args
                    .get("name")
                    .ok_or_else(|| anyhow::anyhow!("Missing branch name"))?;
                self.run_git(&["checkout", name], cwd)?
            }
            "add" => {
                let path_values = if let Some(paths_json) = args.get("paths_json") {
                    serde_json::from_str::<Vec<String>>(paths_json)?
                } else {
                    let paths = args
                        .get("paths")
                        .ok_or_else(|| anyhow::anyhow!("Missing paths"))?;
                    paths.split_whitespace().map(ToString::to_string).collect()
                };
                let mut git_args = vec!["add".to_string()];
                git_args.extend(path_values);
                let git_args = git_args.iter().map(String::as_str).collect::<Vec<_>>();
                self.run_git(&git_args, cwd)?
            }
            "commit" => {
                let message = args
                    .get("message")
                    .ok_or_else(|| anyhow::anyhow!("Missing message"))?;
                self.run_git(&["commit", "-m", message], cwd)?
            }
            "log" => {
                let count = args.get("count").map(|s| s.as_str()).unwrap_or("10");
                self.run_git(&["log", "-n", count, "--oneline"], cwd)?
            }
            "stash_push" => {
                let message = args
                    .get("message")
                    .map(|s| s.as_str())
                    .unwrap_or("ria-coder backup");
                self.run_git(&["stash", "push", "-m", message], cwd)?
            }
            "stash_pop" => self.run_git(&["stash", "pop"], cwd)?,
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
