//! Verification Engine (SPEC-025)

use crate::execution::ChangeSet;
use anyhow::Result;
use ria_tools::registry::{ToolOutput, ToolRegistry};
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
    pub syntax_check: CheckResult,
    pub build_result: CheckResult,
    pub test_result: CheckResult,
    pub lint_result: CheckResult,
    pub format_result: CheckResult,
    pub build_pass: bool,
    pub test_pass: bool,
    pub lint_pass: bool,
    pub test_count: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warning,
    Fail,
    Skipped,
}

impl VerificationResult {
    pub fn passed(&self) -> bool {
        matches!(
            self.status,
            VerificationStatus::Pass | VerificationStatus::Warning { .. }
        )
    }

    pub fn summary(&self) -> String {
        let checks = [
            &self.syntax_check,
            &self.build_result,
            &self.test_result,
            &self.lint_result,
            &self.format_result,
        ];
        let check_lines = checks
            .iter()
            .map(|check| format!("{}: {}", check.name, check.message))
            .collect::<Vec<_>>()
            .join("; ");

        match &self.status {
            VerificationStatus::Pass => format!("All checks passed ({check_lines})"),
            VerificationStatus::Warning { messages } => {
                format!("Passed with {} warnings ({check_lines})", messages.len())
            }
            VerificationStatus::Fail { errors } => {
                format!("Failed with {} errors ({check_lines})", errors.len())
            }
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
                syntax_check: CheckResult::pass("Syntax", "No changes"),
                build_result: CheckResult::pass("Build", "No changes"),
                test_result: CheckResult::pass("Tests", "No changes"),
                lint_result: CheckResult::pass("Lint", "No changes"),
                format_result: CheckResult::pass("Format", "No changes"),
                build_pass: true,
                test_pass: true,
                lint_pass: true,
                test_count: Some(0),
            });
        }

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let syntax_check = self.verify_syntax(changes);
        collect_check(&syntax_check, &mut errors, &mut warnings);

        let build_output = self.run_tool("build", &[("action", "check")])?;
        let build_pass = build_output.exit_code == 0;
        let build_result = result_from_output("Build", &build_output, "Build succeeded");
        collect_check(&build_result, &mut errors, &mut warnings);

        let test_output = self.run_tool("test", &[])?;
        let test_pass = test_output.exit_code == 0;
        let test_count =
            parse_cargo_test_count(&format!("{}\n{}", test_output.stdout, test_output.stderr));
        let test_message = test_count
            .map(|count| format!("{count} tests passed"))
            .unwrap_or_else(|| "Tests passed".to_string());
        let test_result = result_from_output("Tests", &test_output, &test_message);
        collect_check(&test_result, &mut errors, &mut warnings);

        let (lint_result, format_result) = if self.has_cargo_project() {
            let clippy_output = self.run_tool("lint", &[("action", "clippy")])?;
            let fmt_output = self.run_tool("lint", &[("action", "fmt")])?;
            (
                result_from_output("Lint", &clippy_output, "Clippy passed"),
                warning_from_output("Format", &fmt_output, "Format clean"),
            )
        } else {
            (
                CheckResult::skipped("Lint", "No Cargo.toml; lint skipped"),
                CheckResult::skipped("Format", "No Cargo.toml; format skipped"),
            )
        };
        collect_check(&lint_result, &mut errors, &mut warnings);
        collect_check(&format_result, &mut errors, &mut warnings);

        let status = if !errors.is_empty() {
            VerificationStatus::Fail { errors }
        } else if !warnings.is_empty() {
            VerificationStatus::Warning { messages: warnings }
        } else {
            VerificationStatus::Pass
        };
        let lint_pass = matches!(
            lint_result.status,
            CheckStatus::Pass | CheckStatus::Warning | CheckStatus::Skipped
        ) && matches!(
            format_result.status,
            CheckStatus::Pass | CheckStatus::Warning | CheckStatus::Skipped
        );

        Ok(VerificationResult {
            status,
            syntax_check,
            build_result,
            test_result,
            lint_result,
            format_result,
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

    fn run_tool(&self, tool: &str, pairs: &[(&str, &str)]) -> Result<ToolOutput> {
        let mut args = HashMap::new();
        for (key, value) in pairs {
            args.insert((*key).to_string(), (*value).to_string());
        }
        self.tools.execute(tool, &args)
    }

    fn verify_syntax(&self, changes: &ChangeSet) -> CheckResult {
        for change in &changes.changes {
            if change.modified.is_empty() {
                continue;
            }

            let path = PathBuf::from(&change.path);
            match path.extension().and_then(|extension| extension.to_str()) {
                Some("json") => {
                    if let Err(error) = serde_json::from_str::<serde_json::Value>(&change.modified)
                    {
                        return CheckResult::fail(
                            "Syntax",
                            format!("{} JSON parse failed: {}", change.path, error),
                        );
                    }
                }
                Some("toml") => {
                    if let Err(error) = toml::from_str::<toml::Value>(&change.modified) {
                        return CheckResult::fail(
                            "Syntax",
                            format!("{} TOML parse failed: {}", change.path, error),
                        );
                    }
                }
                _ => {}
            }
        }

        CheckResult::pass("Syntax", "Changed structured files parse")
    }
}

impl CheckResult {
    fn pass(name: &str, message: impl Into<String>) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: message.into(),
        }
    }

    fn warning(name: &str, message: impl Into<String>) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Warning,
            message: message.into(),
        }
    }

    fn fail(name: &str, message: impl Into<String>) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: message.into(),
        }
    }

    fn skipped(name: &str, message: impl Into<String>) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Skipped,
            message: message.into(),
        }
    }
}

fn result_from_output(name: &str, output: &ToolOutput, pass_message: &str) -> CheckResult {
    if output.exit_code == 0 {
        CheckResult::pass(name, pass_message)
    } else if output.stderr.contains("No supported") {
        CheckResult::skipped(
            name,
            first_nonempty_line(&output.stderr).unwrap_or("Skipped"),
        )
    } else {
        CheckResult::fail(
            name,
            first_nonempty_line(&output.stderr)
                .or_else(|| first_nonempty_line(&output.stdout))
                .unwrap_or("Command failed"),
        )
    }
}

fn warning_from_output(name: &str, output: &ToolOutput, pass_message: &str) -> CheckResult {
    if output.exit_code == 0 {
        CheckResult::pass(name, pass_message)
    } else {
        CheckResult::warning(
            name,
            first_nonempty_line(&output.stderr)
                .or_else(|| first_nonempty_line(&output.stdout))
                .unwrap_or("Check reported warnings"),
        )
    }
}

fn collect_check(check: &CheckResult, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
    match check.status {
        CheckStatus::Fail => errors.push(format!("{}: {}", check.name, check.message)),
        CheckStatus::Warning => warnings.push(format!("{}: {}", check.name, check.message)),
        CheckStatus::Skipped => warnings.push(format!("{}: {}", check.name, check.message)),
        CheckStatus::Pass => {}
    }
}

fn first_nonempty_line(output: &str) -> Option<&str> {
    output.lines().map(str::trim).find(|line| !line.is_empty())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cargo_test_count() {
        let output = "test result: ok. 47 passed; 0 failed; 0 ignored";
        assert_eq!(parse_cargo_test_count(output), Some(47));
    }

    #[test]
    fn marks_unsupported_tools_as_skipped() {
        let output = ToolOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: "No supported build system found in .".to_string(),
            duration: std::time::Duration::from_millis(1),
        };
        let result = result_from_output("Build", &output, "Build succeeded");
        assert_eq!(result.status, CheckStatus::Skipped);
    }
}
