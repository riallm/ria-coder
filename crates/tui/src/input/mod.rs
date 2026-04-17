//! Command Bar - Input area (SPEC-014)

use reedline::{Reedline, Signal};
use anyhow::Result;

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
}

impl CommandBar {
    pub fn new() -> Self {
        Self {
            mode: InputMode::NaturalLanguage,
            editor: Reedline::create(),
            history: Vec::new(),
            history_index: 0,
        }
    }

    pub fn read_line(&mut self) -> Result<Option<String>> {
        let signal = self.editor.read_line()?;
        match signal {
            Signal::Success(buffer) => {
                self.history.push(buffer.clone());
                Ok(Some(buffer))
            }
            Signal::CtrlD | Signal::CtrlC => Ok(None),
        }
    }
}
