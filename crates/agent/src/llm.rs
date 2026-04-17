//! LLM Interface (SPEC-026)

use anyhow::Result;
use async_trait::async_trait;

/// Generation configuration
#[derive(Debug, Clone)]
pub struct GenConfig {
    pub max_tokens: usize,
    pub temperature: f64,
    pub top_p: f64,
    pub stop_sequences: Vec<String>,
    pub system_prompt: Option<String>,
}

/// LLM engine trait
#[async_trait]
pub trait LLMEngine: Send + Sync {
    async fn generate(&self, prompt: &str, config: &GenConfig) -> Result<String>;
    fn model_info(&self) -> ModelInfo;
}

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub parameters: String,
    pub context_length: usize,
}
