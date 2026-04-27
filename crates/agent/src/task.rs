//! Task Parser (SPEC-022)

use crate::llm::{GenConfig, LLMEngine};
use anyhow::Result;

/// Structured task representation
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub intent: TaskIntent,
    pub targets: Vec<TargetSpec>,
    pub constraints: Vec<Constraint>,
    pub context_hints: Vec<String>,
}

/// What the user wants to do
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TaskIntent {
    Explain,
    Modify { description: String },
    Create { path: String, description: String },
    Delete { path: String },
    Refactor { description: String },
    Debug { symptom: String },
    Test { target: String },
    Review { target: String },
    Document { target: String },
}

/// Target file or symbol
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TargetSpec {
    pub path: Option<String>,
    pub symbol: Option<String>,
    pub line_range: Option<(usize, usize)>,
}

/// Task constraints
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Constraint {
    KeepApiStable,
    Language(String),
    Style(String),
    WithTests,
    Performance,
}

/// Task parser
pub struct TaskParser;

impl TaskParser {
    pub fn parse(input: &str) -> Result<Task> {
        let input_lower = input.to_lowercase();
        let targets = Self::extract_targets(input);
        let constraints = Self::extract_constraints(input);
        let context_hints = Self::extract_context_hints(input);

        let intent = if input_lower.starts_with("explain")
            || input_lower.starts_with("what ")
            || input_lower.starts_with("how ")
        {
            TaskIntent::Explain
        } else if input_lower.starts_with("create") || input_lower.contains("new file") {
            let path = targets
                .iter()
                .find_map(|target| target.path.clone())
                .or_else(|| input.split_whitespace().nth(1).map(clean_token))
                .unwrap_or_else(|| "new_file.rs".to_string());
            TaskIntent::Create {
                path,
                description: input.to_string(),
            }
        } else if input_lower.starts_with("delete")
            || input_lower.starts_with("remove")
            || input_lower.starts_with("rm ")
        {
            let path = targets
                .iter()
                .find_map(|target| target.path.clone())
                .or_else(|| input.split_whitespace().nth(1).map(clean_token))
                .unwrap_or_default();
            TaskIntent::Delete { path }
        } else if input_lower.starts_with("refactor") {
            TaskIntent::Refactor {
                description: input.to_string(),
            }
        } else if input_lower.starts_with("debug")
            || input_lower.starts_with("fix")
            || input_lower.contains(" failing")
            || input_lower.contains(" panic")
            || input_lower.contains(" bug")
        {
            TaskIntent::Debug {
                symptom: input.to_string(),
            }
        } else if input_lower.starts_with("test")
            || input_lower.starts_with("write test")
            || input_lower.starts_with("add test")
        {
            let target = targets
                .iter()
                .find_map(|target| target.path.clone().or_else(|| target.symbol.clone()))
                .or_else(|| input.split_whitespace().nth(1).map(clean_token))
                .unwrap_or_default();
            TaskIntent::Test { target }
        } else if input_lower.starts_with("review") {
            let target = targets
                .iter()
                .find_map(|target| target.path.clone().or_else(|| target.symbol.clone()))
                .unwrap_or_default();
            TaskIntent::Review { target }
        } else if input_lower.starts_with("document")
            || input_lower.starts_with("docs")
            || input_lower.contains("documentation")
        {
            let target = targets
                .iter()
                .find_map(|target| target.path.clone().or_else(|| target.symbol.clone()))
                .unwrap_or_default();
            TaskIntent::Document { target }
        } else {
            TaskIntent::Modify {
                description: input.to_string(),
            }
        };

        Ok(Task {
            intent,
            targets,
            constraints,
            context_hints,
        })
    }

    pub async fn parse_llm(input: &str, llm: &dyn LLMEngine) -> Result<Task> {
        let system_prompt = r#"You are a Task Parser for an agentic coding system. 
Your goal is to parse user natural language requests into a structured JSON format.

JSON Structure:
{
  "intent": {
    "type": "Modify" | "Create" | "Delete" | "Refactor" | "Debug" | "Test" | "Explain",
    "data": { ... } // fields based on type
  },
  "targets": [
    { "path": "string?", "symbol": "string?", "line_range": [start, end]? }
  ],
  "constraints": [
    { "type": "Language" | "Style" | "Performance" | "WithTests" | "KeepApiStable", "data": "string?" }
  ],
  "context_hints": ["string"]
}

Respond ONLY with valid JSON."#;

        let config = GenConfig {
            max_tokens: 512,
            temperature: 0.1,
            system_prompt: Some(system_prompt.to_string()),
            ..Default::default()
        };

        let prompt = format!("{system_prompt}\n\nUser request: {input}");
        let response = llm.generate(&prompt, &config).await?;

        // Try to find JSON in response (in case of chatty model)
        let json_start = response
            .find('{')
            .ok_or_else(|| anyhow::anyhow!("No JSON found in LLM response"))?;
        let json_end = response
            .rfind('}')
            .ok_or_else(|| anyhow::anyhow!("No JSON found in LLM response"))?
            + 1;
        let json_str = &response[json_start..json_end];

        let task: Task = serde_json::from_str(json_str)?;
        Ok(task)
    }

    fn extract_targets(input: &str) -> Vec<TargetSpec> {
        let mut targets = Vec::new();
        for token in input.split_whitespace().map(clean_token) {
            if looks_like_path(&token) {
                targets.push(TargetSpec {
                    path: Some(token),
                    symbol: None,
                    line_range: None,
                });
            }
        }

        let words: Vec<String> = input.split_whitespace().map(clean_token).collect();
        for window in words.windows(2) {
            if matches!(
                window[0].to_lowercase().as_str(),
                "function" | "fn" | "struct" | "enum" | "trait" | "symbol"
            ) {
                targets.push(TargetSpec {
                    path: None,
                    symbol: Some(window[1].clone()),
                    line_range: None,
                });
            }
        }

        targets
    }

    fn extract_constraints(input: &str) -> Vec<Constraint> {
        let lower = input.to_lowercase();
        let mut constraints = Vec::new();

        if lower.contains("without changing the api")
            || lower.contains("keep api")
            || lower.contains("api stable")
        {
            constraints.push(Constraint::KeepApiStable);
        }
        if lower.contains("with tests")
            || lower.contains("add tests")
            || lower.contains("write tests")
        {
            constraints.push(Constraint::WithTests);
        }
        if lower.contains("performance")
            || lower.contains("efficient")
            || lower.contains("optimize")
            || lower.contains("fast")
        {
            constraints.push(Constraint::Performance);
        }
        if lower.contains("existing pattern") || lower.contains("existing style") {
            constraints.push(Constraint::Style("existing style".to_string()));
        }

        for language in [
            "rust",
            "python",
            "go",
            "typescript",
            "javascript",
            "java",
            "c++",
            "c",
        ] {
            if lower.contains(&format!(" in {language}"))
                || lower.contains(&format!(" {language} "))
            {
                constraints.push(Constraint::Language(language.to_string()));
            }
        }

        constraints
    }

    fn extract_context_hints(input: &str) -> Vec<String> {
        let lower = input.to_lowercase();
        let mut hints = Vec::new();
        if lower.contains("this file") {
            hints.push("current_file".to_string());
        }
        if lower.contains("this code") || lower.contains("selection") {
            hints.push("selected_code".to_string());
        }
        hints
    }
}

fn clean_token(token: &str) -> String {
    token
        .trim_matches(|ch: char| {
            matches!(
                ch,
                '`' | '"' | '\'' | ',' | '.' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}'
            )
        })
        .to_string()
}

fn looks_like_path(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }

    token.contains('/')
        || token.ends_with(".rs")
        || token.ends_with(".toml")
        || token.ends_with(".md")
        || token.ends_with(".json")
        || token.ends_with(".yaml")
        || token.ends_with(".yml")
        || token.ends_with(".py")
        || token.ends_with(".go")
        || token.ends_with(".ts")
        || token.ends_with(".tsx")
        || token.ends_with(".js")
        || token.ends_with(".jsx")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_delete_target() {
        let task = TaskParser::parse("delete crates/tools/src/old.rs").unwrap();
        assert_eq!(
            task.intent,
            TaskIntent::Delete {
                path: "crates/tools/src/old.rs".to_string()
            }
        );
        assert_eq!(
            task.targets[0].path.as_deref(),
            Some("crates/tools/src/old.rs")
        );
    }

    #[test]
    fn extracts_constraints() {
        let task = TaskParser::parse(
            "add validation to src/config.rs with tests without changing the API",
        )
        .unwrap();
        assert!(matches!(task.intent, TaskIntent::Modify { .. }));
        assert!(task.constraints.contains(&Constraint::WithTests));
        assert!(task.constraints.contains(&Constraint::KeepApiStable));
        assert_eq!(task.targets[0].path.as_deref(), Some("src/config.rs"));
    }
}
