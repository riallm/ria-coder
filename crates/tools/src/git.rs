//! Git Tools (SPEC-032)

use anyhow::Result;
use std::collections::HashMap;
use crate::registry::ToolOutput;

pub struct GitTools;

impl GitTools {
    pub fn new() -> Self { Self }

    pub fn status(&self) -> Result<String> { Ok(String::new()) }
    pub fn diff(&self) -> Result<String> { Ok(String::new()) }
    pub fn stash(&self, message: &str) -> Result<()> { Ok(()) }
    pub fn add(&self, paths: &[&str]) -> Result<()> { Ok(()) }
    pub fn commit(&self, message: &str) -> Result<()> { Ok(()) }

    pub fn execute(&self, _args: &HashMap<String, String>) -> Result<ToolOutput> {
        Ok(ToolOutput { exit_code: 0, stdout: String::new(), stderr: String::new(), duration: std::time::Duration::ZERO })
    }
}
