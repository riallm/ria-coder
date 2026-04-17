//! Ria Coder Configuration
//!
//! SPEC-093: Configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model: ModelConfig,
    pub ui: UIConfig,
    pub agent: AgentConfig,
    pub git: GitConfig,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub path: Option<String>,
    pub device: String,
    pub max_seq_len: usize,
}

/// UI configuration (SPEC-017)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: String,
    pub show_line_numbers: bool,
    pub syntax_highlight: bool,
    pub auto_refresh: bool,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub auto_test: bool,
    pub auto_build: bool,
    pub require_approval: bool,
    pub max_iterations: usize,
}

/// Git configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub auto_commit: bool,
    pub commit_template: String,
    pub auto_stash: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: ModelConfig {
                path: None,
                device: "cpu".to_string(),
                max_seq_len: 131_072,
            },
            ui: UIConfig {
                theme: "default".to_string(),
                show_line_numbers: true,
                syntax_highlight: true,
                auto_refresh: true,
            },
            agent: AgentConfig {
                auto_test: true,
                auto_build: true,
                require_approval: true,
                max_iterations: 5,
            },
            git: GitConfig {
                auto_commit: true,
                commit_template: "ai: {description}".to_string(),
                auto_stash: true,
            },
        }
    }
}

impl Config {
    /// Load from config file
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save to config file
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get default config path
    pub fn default_path() -> PathBuf {
        let dirs = directories::ProjectDirs::from("org", "riallm", "ria-coder")
            .expect("Failed to get project dirs");
        dirs.config_dir().join("config.toml")
    }
}
