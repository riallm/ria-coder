//! Syntax Highlighting (SPEC-012)

use syntect::highlighting::Theme as SyntectTheme;

/// Supported languages for syntax highlighting
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Rust,
    Python,
    Go,
    TypeScript,
    JavaScript,
    C,
    Cpp,
    Java,
    Shell,
    Toml,
    Json,
    Yaml,
    Markdown,
    Unknown,
}

impl Language {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => Self::Rust,
            "py" => Self::Python,
            "go" => Self::Go,
            "ts" | "tsx" => Self::TypeScript,
            "js" | "jsx" => Self::JavaScript,
            "c" | "h" => Self::C,
            "cpp" | "cc" | "cxx" => Self::Cpp,
            "java" => Self::Java,
            "sh" | "bash" | "zsh" => Self::Shell,
            "toml" => Self::Toml,
            "json" => Self::Json,
            "yml" | "yaml" => Self::Yaml,
            "md" => Self::Markdown,
            _ => Self::Unknown,
        }
    }
}

/// Syntax highlighter
pub struct SyntaxHighlighter {
    theme: SyntectTheme,
}

impl SyntaxHighlighter {
    pub fn new(_theme: &str) -> Self {
        // Initialize syntect highlighter with theme
        Self {
            theme: SyntectTheme::default(),
        }
    }

    pub fn highlight(&self, _code: &str, _language: &Language) -> Vec<StyledLine> {
        // Return syntax-highlighted lines
        Vec::new()
    }
}

#[derive(Debug)]
pub struct StyledLine {
    pub segments: Vec<StyledSegment>,
}

#[derive(Debug)]
pub struct StyledSegment {
    pub text: String,
    pub style: TextStyle,
}

#[derive(Debug)]
pub struct TextStyle {
    pub foreground: Option<String>,
    pub bold: bool,
    pub italic: bool,
}
