//! Color Themes (SPEC-017)

use serde::{Deserialize, Serialize};

/// Built-in color themes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeName {
    Default,
    Dark,
    Monokai,
    Gruvbox,
    Nord,
    Solarized,
    Light,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: ThemeName,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub user_message: String,
    pub agent_message: String,
    pub status_text: String,
    pub success: String,
    pub error: String,
    pub warning: String,
    pub file_path: String,
    pub code_added: String,
    pub code_removed: String,
    pub border: String,
    pub background: String,
    pub foreground: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: ThemeName::Default,
            colors: ThemeColors {
                user_message: "white".to_string(),
                agent_message: "cyan".to_string(),
                status_text: "yellow".to_string(),
                success: "green".to_string(),
                error: "red".to_string(),
                warning: "yellow".to_string(),
                file_path: "blue".to_string(),
                code_added: "green".to_string(),
                code_removed: "red".to_string(),
                border: "bright_black".to_string(),
                background: "reset".to_string(),
                foreground: "reset".to_string(),
            },
        }
    }
}

impl Theme {
    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Default => Self::default(),
            _ => Self::default(),
        }
    }
}
