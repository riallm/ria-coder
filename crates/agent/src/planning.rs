//! Planning Engine (SPEC-023)

use crate::context::AgentContext;
use crate::llm::{GenConfig, LLMEngine};
use crate::task::{Constraint, TargetSpec, Task, TaskIntent};
use anyhow::Result;

/// Ordered execution plan
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub task: Task,
    pub steps: Vec<PlanStep>,
    pub estimated_duration: std::time::Duration,
    pub risk_level: RiskLevel,
}

/// Single plan step
#[derive(Debug, Clone, PartialEq)]
pub struct PlanStep {
    pub description: String,
    pub action: PlanAction,
    pub target: Option<TargetSpec>,
    pub depends_on: Vec<usize>,
}

/// Edit specification
#[derive(Debug, Clone, PartialEq)]
pub struct EditSpec {
    pub description: String,
    pub diff: Option<String>,
    pub old_text: Option<String>,
    pub new_text: Option<String>,
    pub replace_all: bool,
}

/// Step action type
#[derive(Debug, Clone, PartialEq)]
pub enum PlanAction {
    ReadFile { path: String },
    EditFile { path: String, edit: EditSpec },
    CreateFile { path: String, content: String },
    DeleteFile { path: String },
    RunCommand { command: String, args: Vec<String> },
    Search { pattern: String },
    Analyze { target: String },
}

/// Risk assessment
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Task planner
pub struct TaskPlanner;

#[derive(Debug, serde::Deserialize)]
struct GeneratedPlan {
    #[allow(dead_code)]
    summary: Option<String>,
    changes: Vec<GeneratedChange>,
}

#[derive(Debug, serde::Deserialize)]
struct GeneratedChange {
    path: String,
    action: String,
    content: Option<String>,
    old_text: Option<String>,
    new_text: Option<String>,
    replace_all: Option<bool>,
}

impl TaskPlanner {
    pub fn new() -> Self {
        Self
    }

    /// Parse input to task
    pub fn parse(&self, input: &str) -> Result<Task> {
        crate::task::TaskParser::parse(input)
    }

    /// Ask the configured model to synthesize concrete file changes.
    ///
    /// This is intentionally strict: invalid or incomplete model output becomes
    /// an error and the orchestrator falls back to the deterministic planner.
    pub async fn generate_with_llm(
        &self,
        task: &Task,
        context: &AgentContext,
        llm: &dyn LLMEngine,
    ) -> Result<Plan> {
        if matches!(
            task.intent,
            TaskIntent::Explain | TaskIntent::Review { .. } | TaskIntent::Delete { .. }
        ) {
            return Err(anyhow::anyhow!(
                "LLM change synthesis is not needed for this task"
            ));
        }

        let prompt = build_llm_planning_prompt(task, context)?;
        let config = GenConfig {
            max_tokens: 4096,
            temperature: 0.2,
            top_p: 0.95,
            stop_sequences: vec!["</changes>".to_string()],
            system_prompt: Some(context.system_prompt.clone()),
        };

        let response = llm.generate(&prompt, &config).await?;
        let json = extract_json_object(&response)?;
        let generated: GeneratedPlan = serde_json::from_str(json)?;
        if generated.changes.is_empty() {
            return Err(anyhow::anyhow!("LLM returned no file changes"));
        }

        let mut steps = Vec::new();
        let mut read_steps = Vec::new();

        for target in &task.targets {
            if let Some(path) = &target.path {
                let index = steps.len();
                steps.push(PlanStep {
                    description: format!("Read target file: {}", path),
                    action: PlanAction::ReadFile { path: path.clone() },
                    target: Some(target.clone()),
                    depends_on: Vec::new(),
                });
                read_steps.push(index);
            }
        }

        for change in generated.changes {
            let target = Some(TargetSpec {
                path: Some(change.path.clone()),
                symbol: None,
                line_range: None,
            });
            let depends_on = read_steps.clone();
            let action = match change.action.as_str() {
                "create" => PlanAction::CreateFile {
                    path: change.path.clone(),
                    content: change.content.unwrap_or_default(),
                },
                "edit" => PlanAction::EditFile {
                    path: change.path.clone(),
                    edit: EditSpec {
                        description: task_description(task),
                        diff: None,
                        old_text: Some(change.old_text.ok_or_else(|| {
                            anyhow::anyhow!("edit change for {} missing old_text", change.path)
                        })?),
                        new_text: Some(change.new_text.ok_or_else(|| {
                            anyhow::anyhow!("edit change for {} missing new_text", change.path)
                        })?),
                        replace_all: change.replace_all.unwrap_or(false),
                    },
                },
                "delete" => PlanAction::DeleteFile {
                    path: change.path.clone(),
                },
                other => {
                    return Err(anyhow::anyhow!(
                        "Unknown generated change action: {}",
                        other
                    ))
                }
            };

            steps.push(PlanStep {
                description: format!("Apply {} to {}", change.action, change.path),
                action,
                target,
                depends_on,
            });
        }

        let risk_level = assess_risk(&steps);
        Ok(Plan {
            task: task.clone(),
            estimated_duration: estimate_duration(&steps),
            steps,
            risk_level,
        })
    }

    /// Generate plan from task
    pub fn generate(&self, task: &Task, _context: &AgentContext) -> Result<Plan> {
        let mut steps = Vec::new();

        match &task.intent {
            TaskIntent::Explain => {
                steps.push(PlanStep {
                    description: "Analyze current code and explain".to_string(),
                    action: PlanAction::Analyze {
                        target: "current context".to_string(),
                    },
                    target: None,
                    depends_on: Vec::new(),
                });
            }
            TaskIntent::Create { path, description } => {
                steps.push(PlanStep {
                    description: format!("Create new file: {}", path),
                    action: PlanAction::CreateFile {
                        path: path.clone(),
                        content: default_file_content(path, description),
                    },
                    target: Some(TargetSpec {
                        path: Some(path.clone()),
                        symbol: None,
                        line_range: None,
                    }),
                    depends_on: Vec::new(),
                });
                steps.push(PlanStep {
                    description: "Verify file creation".to_string(),
                    action: PlanAction::RunCommand {
                        command: "ls".to_string(),
                        args: vec![path.clone()],
                    },
                    target: None,
                    depends_on: vec![0],
                });
            }
            TaskIntent::Delete { path } => {
                if path.is_empty() {
                    steps.push(PlanStep {
                        description: "Analyze delete request and identify target".to_string(),
                        action: PlanAction::Analyze {
                            target: "delete target".to_string(),
                        },
                        target: None,
                        depends_on: Vec::new(),
                    });
                } else {
                    steps.push(PlanStep {
                        description: format!("Read file before deleting: {}", path),
                        action: PlanAction::ReadFile { path: path.clone() },
                        target: Some(TargetSpec {
                            path: Some(path.clone()),
                            symbol: None,
                            line_range: None,
                        }),
                        depends_on: Vec::new(),
                    });
                    steps.push(PlanStep {
                        description: format!("Delete file: {}", path),
                        action: PlanAction::DeleteFile { path: path.clone() },
                        target: Some(TargetSpec {
                            path: Some(path.clone()),
                            symbol: None,
                            line_range: None,
                        }),
                        depends_on: vec![0],
                    });
                }
            }
            TaskIntent::Test { target } => {
                steps.push(PlanStep {
                    description: format!("Run tests for: {}", target),
                    action: PlanAction::RunCommand {
                        command: "cargo".to_string(),
                        args: vec!["test".to_string(), target.clone()],
                    },
                    target: None,
                    depends_on: Vec::new(),
                });
            }
            TaskIntent::Review { target } | TaskIntent::Document { target } => {
                if !target.is_empty() {
                    steps.push(PlanStep {
                        description: format!("Read target: {}", target),
                        action: PlanAction::ReadFile {
                            path: target.clone(),
                        },
                        target: Some(TargetSpec {
                            path: Some(target.clone()),
                            symbol: None,
                            line_range: None,
                        }),
                        depends_on: Vec::new(),
                    });
                }
                steps.push(PlanStep {
                    description: format!("Analyze {}", target),
                    action: PlanAction::Analyze {
                        target: target.clone(),
                    },
                    target: None,
                    depends_on: if target.is_empty() {
                        Vec::new()
                    } else {
                        vec![0]
                    },
                });
            }
            _ => {
                for target in &task.targets {
                    if let Some(path) = &target.path {
                        steps.push(PlanStep {
                            description: format!("Read target file: {}", path),
                            action: PlanAction::ReadFile { path: path.clone() },
                            target: Some(target.clone()),
                            depends_on: Vec::new(),
                        });
                    } else if let Some(symbol) = &target.symbol {
                        steps.push(PlanStep {
                            description: format!("Search for symbol: {}", symbol),
                            action: PlanAction::Search {
                                pattern: symbol.clone(),
                            },
                            target: Some(target.clone()),
                            depends_on: Vec::new(),
                        });
                    }
                }

                steps.push(PlanStep {
                    description: "Analyze task and propose changes".to_string(),
                    action: PlanAction::Analyze {
                        target: "task description".to_string(),
                    },
                    target: None,
                    depends_on: (0..steps.len()).collect(),
                });
            }
        }

        let estimated_duration = estimate_duration(&steps);
        let risk_level = assess_risk(&steps);

        Ok(Plan {
            task: task.clone(),
            steps,
            estimated_duration,
            risk_level,
        })
    }
}

fn build_llm_planning_prompt(task: &Task, context: &AgentContext) -> Result<String> {
    let task_json = serde_json::to_string_pretty(task)?;
    let mut files = String::new();

    let mut file_context = context.files.values().collect::<Vec<_>>();
    file_context.sort_by(|left, right| {
        right
            .relevance
            .partial_cmp(&left.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut used_chars = 0usize;
    for file in file_context {
        if used_chars > 48_000 {
            break;
        }
        let content = truncate_for_prompt(&file.content, 12_000);
        used_chars += content.len();
        files.push_str(&format!(
            "\n--- file: {} ({}) ---\n{}\n",
            file.path.display(),
            file.language,
            content
        ));
    }

    Ok(format!(
        r#"Generate concrete file changes for this coding task.

Return ONLY JSON with this shape:
{{
  "summary": "short summary",
  "changes": [
    {{
      "path": "relative/path",
      "action": "create|edit|delete",
      "content": "full file content for create",
      "old_text": "exact text to replace for edit",
      "new_text": "replacement text for edit",
      "replace_all": false
    }}
  ]
}}

Rules:
- Use relative paths.
- For edits, old_text must exactly match the current file content.
- Keep changes small and focused on the task.
- Do not include markdown fences or commentary.

Task:
{task_json}

Project:
- name: {}
- language: {}
- build: {}
- tests: {}

Relevant files:
{files}
</changes>"#,
        context.project_info.name,
        context.project_info.language,
        context.project_info.build_system,
        context.project_info.test_framework
    ))
}

fn extract_json_object(text: &str) -> Result<&str> {
    let start = text
        .find('{')
        .ok_or_else(|| anyhow::anyhow!("No JSON object found in LLM response"))?;
    let end = text
        .rfind('}')
        .ok_or_else(|| anyhow::anyhow!("No JSON object found in LLM response"))?
        + 1;
    Ok(&text[start..end])
}

fn truncate_for_prompt(content: &str, max_chars: usize) -> &str {
    if content.chars().count() <= max_chars {
        return content;
    }

    let end = content
        .char_indices()
        .nth(max_chars)
        .map(|(index, _)| index)
        .unwrap_or(content.len());
    &content[..end]
}

fn task_description(task: &Task) -> String {
    match &task.intent {
        TaskIntent::Explain => "Explain code".to_string(),
        TaskIntent::Modify { description }
        | TaskIntent::Refactor { description }
        | TaskIntent::Create { description, .. } => description.clone(),
        TaskIntent::Delete { path } => format!("Delete {}", path),
        TaskIntent::Debug { symptom } => symptom.clone(),
        TaskIntent::Test { target } => format!("Test {}", target),
        TaskIntent::Review { target } => format!("Review {}", target),
        TaskIntent::Document { target } => format!("Document {}", target),
    }
}

fn default_file_content(path: &str, description: &str) -> String {
    if path.ends_with(".rs") {
        format!("//! {}\n\n", description)
    } else if path.ends_with(".md") {
        format!("# {}\n\n", description.trim())
    } else if path.ends_with(".toml") {
        format!("# {}\n", description)
    } else {
        format!("{}\n", description)
    }
}

fn estimate_duration(steps: &[PlanStep]) -> std::time::Duration {
    std::time::Duration::from_secs((steps.len().max(1) as u64) * 5)
}

fn assess_risk(steps: &[PlanStep]) -> RiskLevel {
    let change_count = steps
        .iter()
        .filter(|step| {
            matches!(
                step.action,
                PlanAction::EditFile { .. }
                    | PlanAction::CreateFile { .. }
                    | PlanAction::DeleteFile { .. }
            )
        })
        .count();

    if steps
        .iter()
        .any(|step| matches!(step.action, PlanAction::DeleteFile { .. }))
    {
        RiskLevel::High
    } else if change_count > 5 || steps.len() > 20 {
        RiskLevel::Medium
    } else if steps.iter().any(|step| {
        matches!(
            step.action,
            PlanAction::RunCommand { ref command, ref args }
                if command == "cargo" && args.iter().any(|arg| arg == "clean")
        )
    }) {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    }
}

#[allow(dead_code)]
fn wants_tests(task: &Task) -> bool {
    task.constraints
        .iter()
        .any(|constraint| matches!(constraint, Constraint::WithTests))
}
