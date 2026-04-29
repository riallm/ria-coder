//! Plugin System (SPEC-090)

use anyhow::Result;
use ria_tools::registry::ToolRegistry;
use std::collections::HashMap;

/// Plugin trait for extending agent capabilities
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn register_tools(&self, registry: &mut ToolRegistry);
    fn register_commands(&self, commands: &mut CommandRegistry);
    fn on_startup(&self) -> Result<()>;
    fn on_shutdown(&self);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub tool: String,
    pub args: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct CommandRegistry {
    commands: HashMap<String, PluginCommand>,
}

impl CommandRegistry {
    pub fn register(&mut self, command: PluginCommand) {
        self.commands.insert(command.name.clone(), command);
    }

    pub fn get(&self, name: &str) -> Option<&PluginCommand> {
        self.commands.get(name)
    }

    pub fn list(&self) -> Vec<&PluginCommand> {
        let mut commands = self.commands.values().collect::<Vec<_>>();
        commands.sort_by(|left, right| left.name.cmp(&right.name));
        commands
    }
}

/// Registry for active plugins
pub struct PluginManager {
    pub plugins: Vec<Box<dyn Plugin>>,
    pub commands: CommandRegistry,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: Vec::new(),
            commands: CommandRegistry::default(),
        };
        manager.register(Box::new(LanguagePlugin::rust()));
        manager.register(Box::new(LanguagePlugin::python()));
        manager.register(Box::new(LanguagePlugin::go()));
        manager.register(Box::new(LanguagePlugin::typescript()));
        manager.register(Box::new(LanguagePlugin::docker()));
        manager
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        plugin.register_commands(&mut self.commands);
        self.plugins.push(plugin);
    }

    pub fn init_all(&self) -> Result<()> {
        for plugin in &self.plugins {
            plugin.on_startup()?;
        }
        Ok(())
    }

    pub fn register_all_tools(&self, registry: &mut ToolRegistry) {
        for plugin in &self.plugins {
            plugin.register_tools(registry);
        }
    }

    pub fn command(&self, name: &str) -> Option<&PluginCommand> {
        self.commands.get(name)
    }

    pub fn commands(&self) -> Vec<&PluginCommand> {
        self.commands.list()
    }
}

struct LanguagePlugin {
    name: &'static str,
    description: &'static str,
    file_patterns: &'static [&'static str],
    commands: Vec<PluginCommand>,
}

impl LanguagePlugin {
    fn rust() -> Self {
        Self {
            name: "rust",
            description: "Rust project workflows",
            file_patterns: &["*.rs", "Cargo.toml"],
            commands: vec![
                command(
                    "rust-check",
                    "Run cargo check",
                    "build",
                    &[("action", "check")],
                ),
                command("rust-test", "Run cargo test", "test", &[]),
                command(
                    "rust-fmt",
                    "Run cargo fmt --check",
                    "lint",
                    &[("action", "fmt")],
                ),
                command(
                    "rust-clippy",
                    "Run cargo clippy",
                    "lint",
                    &[("action", "clippy")],
                ),
            ],
        }
    }

    fn python() -> Self {
        Self {
            name: "python",
            description: "Python project workflows",
            file_patterns: &["*.py", "pyproject.toml"],
            commands: vec![command(
                "python-test",
                "Run pytest or unittest-compatible tests",
                "test",
                &[("system", "pytest")],
            )],
        }
    }

    fn go() -> Self {
        Self {
            name: "go",
            description: "Go project workflows",
            file_patterns: &["*.go", "go.mod"],
            commands: vec![
                command(
                    "go-build",
                    "Run go build ./...",
                    "build",
                    &[("system", "go")],
                ),
                command("go-test", "Run go test ./...", "test", &[("system", "go")]),
            ],
        }
    }

    fn typescript() -> Self {
        Self {
            name: "typescript",
            description: "TypeScript and JavaScript project workflows",
            file_patterns: &["*.ts", "*.tsx", "*.js", "*.jsx", "package.json"],
            commands: vec![
                command(
                    "npm-build",
                    "Run npm build script",
                    "build",
                    &[("system", "npm")],
                ),
                command("npm-test", "Run npm test", "test", &[("system", "npm")]),
            ],
        }
    }

    fn docker() -> Self {
        Self {
            name: "docker",
            description: "Dockerfile helper workflows",
            file_patterns: &["Dockerfile", "docker-compose.yml"],
            commands: Vec::new(),
        }
    }
}

impl Plugin for LanguagePlugin {
    fn name(&self) -> &str {
        self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        self.description
    }

    fn register_tools(&self, _registry: &mut ToolRegistry) {}

    fn register_commands(&self, commands: &mut CommandRegistry) {
        for command in &self.commands {
            commands.register(command.clone());
        }
    }

    fn on_startup(&self) -> Result<()> {
        let _ = self.file_patterns;
        Ok(())
    }

    fn on_shutdown(&self) {}
}

fn command(name: &str, description: &str, tool: &str, args: &[(&str, &str)]) -> PluginCommand {
    PluginCommand {
        name: name.to_string(),
        description: description.to_string(),
        tool: tool.to_string(),
        args: args
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_plugins_register_commands() {
        let plugins = PluginManager::new();
        assert!(plugins.command("rust-check").is_some());
        assert!(plugins.command("python-test").is_some());
        assert!(plugins.command("go-test").is_some());
        assert!(plugins.command("npm-test").is_some());
    }
}
