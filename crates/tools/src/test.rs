//! Test Tools (SPEC-034)

use crate::registry::{Tool, ToolCategory, ToolOutput};
use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;

pub struct TestTools;

impl TestTools {
    pub fn new() -> Self {
        Self
    }

    fn run_test(&self, args: &[&str]) -> Result<(i32, String, String)> {
        let output = Command::new("cargo").arg("test").args(args).output()?;

        Ok((
            output.status.code().unwrap_or(0),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
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
        Command::new("cargo").arg("--version").output().is_ok()
    }

    fn execute(&self, args: &HashMap<String, String>) -> Result<ToolOutput> {
        let target = args.get("target").map(|s| s.as_str()).unwrap_or("");

        let start = std::time::Instant::now();

        let mut test_args = Vec::new();
        if !target.is_empty() {
            test_args.push(target);
        }

        let (exit_code, stdout, stderr) = self.run_test(&test_args)?;

        Ok(ToolOutput {
            exit_code,
            stdout,
            stderr,
            duration: start.elapsed(),
        })
    }
}
