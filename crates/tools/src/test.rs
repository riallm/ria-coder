//! Test Tools (SPEC-034)

use crate::registry::{Tool, ToolCategory, ToolOutput};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct TestTools;

impl TestTools {
    pub fn new() -> Self {
        Self
    }

    fn run_command(
        &self,
        program: &str,
        args: &[&str],
        cwd: Option<&str>,
    ) -> Result<(i32, String, String)> {
        let mut command = Command::new(program);
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

    fn detect_system(cwd: Option<&str>) -> TestSystem {
        let root = cwd
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        if root.join("Cargo.toml").exists() {
            TestSystem::Cargo
        } else if root.join("package.json").exists() {
            TestSystem::Npm
        } else if root.join("go.mod").exists() {
            TestSystem::Go
        } else if root.join("pytest.ini").exists()
            || root.join("pyproject.toml").exists()
            || root.join("setup.py").exists()
        {
            TestSystem::Pytest
        } else if root.join("Makefile").exists() || root.join("makefile").exists() {
            TestSystem::Make
        } else {
            TestSystem::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestSystem {
    Cargo,
    Npm,
    Go,
    Pytest,
    Make,
    Unknown,
}

fn command_exists(program: &str) -> bool {
    Command::new(program).arg("--version").output().is_ok()
        || Command::new(program).arg("version").output().is_ok()
}

impl Tool for TestTools {
    fn name(&self) -> &str {
        "test"
    }
    fn description(&self) -> &str {
        "Test system operations"
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Test
    }
    fn is_available(&self) -> bool {
        ["cargo", "npm", "go", "python3", "pytest", "make"]
            .iter()
            .any(|program| command_exists(program))
    }

    fn execute(&self, args: &HashMap<String, String>) -> Result<ToolOutput> {
        let target = args.get("target").map(|s| s.as_str()).unwrap_or("");

        let start = std::time::Instant::now();
        let cwd = args.get("cwd").map(|s| s.as_str());
        let system = args
            .get("system")
            .map(|system| match system.as_str() {
                "cargo" => TestSystem::Cargo,
                "npm" => TestSystem::Npm,
                "go" => TestSystem::Go,
                "pytest" | "python" => TestSystem::Pytest,
                "make" => TestSystem::Make,
                _ => TestSystem::Unknown,
            })
            .unwrap_or_else(|| Self::detect_system(cwd));

        let (program, command_args): (&str, Vec<&str>) = match system {
            TestSystem::Cargo => {
                let mut args = vec!["test"];
                if !target.is_empty() {
                    args.push(target);
                }
                ("cargo", args)
            }
            TestSystem::Npm => ("npm", vec!["test"]),
            TestSystem::Go => ("go", vec!["test", "./..."]),
            TestSystem::Pytest => {
                if command_exists("pytest") {
                    let mut args = Vec::new();
                    if !target.is_empty() {
                        args.push(target);
                    }
                    ("pytest", args)
                } else {
                    let mut args = vec!["-m", "pytest"];
                    if !target.is_empty() {
                        args.push(target);
                    }
                    ("python3", args)
                }
            }
            TestSystem::Make => ("make", vec!["test"]),
            TestSystem::Unknown => {
                return Ok(ToolOutput {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: format!(
                        "No supported test system found in {}",
                        cwd.map(Path::new)
                            .unwrap_or_else(|| Path::new("."))
                            .display()
                    ),
                    duration: start.elapsed(),
                });
            }
        };
        let (exit_code, stdout, stderr) = self.run_command(program, &command_args, cwd)?;

        Ok(ToolOutput {
            exit_code,
            stdout,
            stderr,
            duration: start.elapsed(),
        })
    }
}
