//! Ria Coder Terminal User Interface
//!
//! Provides a ratatui-based TUI with:
//! - Chat Panel (SPEC-011)
//! - File Preview Panel (SPEC-012)
//! - Agent Status Panel (SPEC-013)
//! - Command Bar (SPEC-014)
//! - Output Log Panel (SPEC-015)
//! - Key Bindings (SPEC-016)
//! - Color Themes (SPEC-017)

pub mod app;
pub mod input;
pub mod keybindings;
pub mod panels;
pub mod syntax;
pub mod theme;

pub use app::App;
pub use keybindings::KeyBindings;
pub use theme::Theme;
