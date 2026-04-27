//! Verification Engine (SPEC-025)

use crate::execution::ChangeSet;
use anyhow::Result;
use ria_tools::registry::ToolRegistry;
use std::collections::HashMap;
use std::path::PathBuf;

/// Verification status
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationStatus {
    Pass,
    Warning { messages: Vec<String> },
    Fail { errors: Vec<String> },
}

/// Verification result
#[derive(Debug)]
pub struct VerificationResult {
    pub status: VerificationStatus,
    pub build_pass: bool,
    pub test_pass: bool,
    pub lint_pass: bool,
    pub test_count: Option<usize>,
}

impl VerificationResult {
    pub fn passed(&self) -> bool {
        matches!(
            self.status,
            VerificationStatus::Pass | VerificationStatus::Warning { .. }
        )
    }

    pub fn summary(&self) -> String {
        match &self.status {
            VerificationStatus::Pass => "All checks passed".to_string(),
            VerificationStatus::Warning { messages } => {
                format!("Passed with {} warnings", messages.len())
            }
            VerificationStatus::Fail { errors } => format!("Failed with {} errors", errors.len()),
        }
    }

    pub fn test_count(&self) -> Option<usize> {
        self.test_count
    }
}

/// Verification engine
pub struct VerificationEngine {
    pub tools: ToolRegistry,
    root: Option<PathBuf>,
}

impl VerificationEngine {
    pub fn new() -> Self {
        Self {
            tools: ToolRegistry::new(),
            root: None,
        }
    }

    pub fn set_root(&mut self, root: PathBuf) {
        self.tools.set_working_dir(root.clone());
        self.root = Some(root);
    }

    /// Verify a change set
    pub async fn verify(&self, changes: &ChangeSet) -> Result<VerificationResult> {
        if changes.changes.is_empty() {
            return Ok(VerificationResult {
                status: VerificationStatus::Pass,
                build_pass: true,
                test_pass: true,
                lint_pass: true,
                test_count: Some(0),
            });
        }

        if !self.has_cargo_project() {
            return Ok(VerificationResult {
                status: VerificationStatus::Warning {
                    messages: vec![
                        "No Cargo.toml found; skipped Rust build, test, and lint checks"
                            .to_string(),
                    ],
                },
                build_pass: true,
                test_pass: true,
                lint_pass: true,
                test_count: None,
            });
        }

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let build_output = self.run_tool("build", &[("action", "check")])?;
        let build_pass = build_output.exit_code == 0;
        if !build_pass {
            errors.push(format!(
                "cargo check failed: {}{}",
                build_output.stderr, build_output.stdout
            ));
        }

        let test_output = self.run_tool("test", &[])?;
        let test_pass = test_output.exit_code == 0;
        let test_count =
            parse_cargo_test_count(&format!("{}\n{}", test_output.stdout, test_output.stderr));
        if !test_pass {
            errors.push(format!(
                "cargo test failed: {}{}",
                test_output.stderr, test_output.stdout
            ));
        }

        let fmt_output = self.run_tool("lint", &[("action", "fmt")])?;
        let lint_pass = fmt_output.exit_code == 0;
        if !lint_pass {
            warnings.push(format!(
                "cargo fmt --check reported formatting changes: {}{}",
                fmt_output.stderr, fmt_output.stdout
            ));
        }

        let status = if !errors.is_empty() {
            VerificationStatus::Fail { errors }
        } else if !warnings.is_empty() {
            VerificationStatus::Warning { messages: warnings }
        } else {
            VerificationStatus::Pass
        };

        Ok(VerificationResult {
            status,
            build_pass,
            test_pass,
            lint_pass,
            test_count,
        })
    }

    fn has_cargo_project(&self) -> bool {
        self.root
            .as_ref()
            .map(|root| root.join("Cargo.toml").exists())
            .unwrap_or_else(|| PathBuf::from("Cargo.toml").exists())
    }

    fn run_tool(
        &self,
        tool: &str,
        pairs: &[(&str, &str)],
    ) -> Result<ria_tools::registry::ToolOutput> {
        let mut args = HashMap::new();
        for (key, value) in pairs {
            args.insert((*key).to_string(), (*value).to_string());
        }
        self.tools.execute(tool, &args)
    }
}

fn parse_cargo_test_count(output: &str) -> Option<usize> {
    output.lines().find_map(|line| {
        let line = line.trim();
        if !line.starts_with("test result:") {
            return None;
        }

        let passed_marker = " passed";
        let passed_index = line.find(passed_marker)?;
        let before_passed = &line[..passed_index];
        before_passed
            .split_whitespace()
            .last()
            .and_then(|value| value.parse::<usize>().ok())
    })
}
