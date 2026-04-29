//! Command Bar - Input area (SPEC-014)

use anyhow::Result;
use reedline::{DefaultPrompt, Reedline, Signal};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    NaturalLanguage,
    Command,
    SlashCommand,
    Search,
}

pub struct CommandBar {
    pub mode: InputMode,
    pub editor: Reedline,
    pub history: Vec<String>,
    pub history_index: usize,
    pub prompt: DefaultPrompt,
}

impl CommandBar {
    pub fn new() -> Self {
        let history = Self::load_history_default().unwrap_or_default();
        let history_index = history.len();
        Self {
            mode: InputMode::NaturalLanguage,
            editor: Reedline::create(),
            history,
            history_index,
            prompt: DefaultPrompt::default(),
        }
    }

    pub fn read_line(&mut self) -> Result<Option<String>> {
        let signal = self.editor.read_line(&self.prompt)?;
        match signal {
            Signal::Success(buffer) => {
                self.history.push(buffer.clone());
                self.history_index = self.history.len();
                let _ = self.save_history_default();
                Ok(Some(buffer))
            }
            Signal::CtrlD | Signal::CtrlC => Ok(None),
        }
    }

    pub fn push_history(&mut self, command: String) {
        self.history.push(command);
        if self.history.len() > 1000 {
            let overflow = self.history.len() - 1000;
            self.history.drain(0..overflow);
        }
        self.history_index = self.history.len();
        let _ = self.save_history_default();
    }

    pub fn history_path() -> PathBuf {
        let config_path = ria_config::Config::default_path();
        config_path
            .parent()
            .map(|parent| parent.join("command-history.json"))
            .unwrap_or_else(|| PathBuf::from("command-history.json"))
    }

    pub fn load_history(path: &Path) -> Result<Vec<String>> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save_history(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(&self.history)?)?;
        Ok(())
    }

    fn load_history_default() -> Result<Vec<String>> {
        Self::load_history(&Self::history_path())
    }

    fn save_history_default(&self) -> Result<()> {
        self.save_history(&Self::history_path())
    }
}
