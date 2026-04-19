//! Plugin System (SPEC-090)

use anyhow::Result;
use ria_tools::registry::ToolRegistry;

/// Plugin trait for extending agent capabilities
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn register_tools(&self, registry: &mut ToolRegistry);
    fn on_startup(&self) -> Result<()>;
    fn on_shutdown(&self);
}

/// Registry for active plugins
pub struct PluginManager {
    pub plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
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
}
