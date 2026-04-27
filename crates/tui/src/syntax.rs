//! Syntax Highlighting (SPEC-012)

use ratatui::style::{Color, Modifier, Style as RatatuiStyle};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

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

    pub fn to_syntect_name(&self) -> &str {
        match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::Go => "Go",
            Self::TypeScript => "TypeScript",
            Self::JavaScript => "JavaScript",
            Self::C => "C",
            Self::Cpp => "C++",
            Self::Java => "Java",
            Self::Shell => "Bourne Again Shell (bash)",
            Self::Toml => "TOML",
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Markdown => "Markdown",
            Self::Unknown => "Plain Text",
        }
    }
}

/// Syntax highlighter
pub struct SyntaxHighlighter {
    pub ps: SyntaxSet,
    pub ts: ThemeSet,
    pub theme_name: String,
}

impl SyntaxHighlighter {
    pub fn new(theme: &str) -> Self {
        Self {
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
            theme_name: theme.to_string(),
        }
    }

    pub fn highlight(&self, code: &str, language: &Language) -> Vec<Line<'static>> {
        let syntax = self
            .ps
            .find_syntax_by_name(language.to_syntect_name())
            .unwrap_or_else(|| self.ps.find_syntax_plain_text());

        let theme = &self.ts.themes.get(&self.theme_name).unwrap_or(
            self.ts
                .themes
                .get("base16-ocean.dark")
                .unwrap_or_else(|| self.ts.themes.values().next().unwrap()),
        );

        let mut h = HighlightLines::new(syntax, theme);
        let mut lines = Vec::new();

        for line in LinesWithEndings::from(code) {
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &self.ps).unwrap_or_default();
            let spans: Vec<Span<'static>> = ranges
                .into_iter()
                .map(|(style, text)| {
                    let mut s = RatatuiStyle::default().fg(Color::Rgb(
                        style.foreground.r,
                        style.foreground.g,
                        style.foreground.b,
                    ));

                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::BOLD)
                    {
                        s = s.add_modifier(Modifier::BOLD);
                    }
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::ITALIC)
                    {
                        s = s.add_modifier(Modifier::ITALIC);
                    }
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::UNDERLINE)
                    {
                        s = s.add_modifier(Modifier::UNDERLINED);
                    }

                    Span::styled(text.to_string(), s)
                })
                .collect();

            lines.push(Line::from(spans));
        }

        lines
    }
}
