//! LLM Interface (SPEC-026)

pub mod riallm;

pub use riallm::RiaLLMEngine;

use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;

/// Generation configuration
#[derive(Debug, Clone)]
pub struct GenConfig {
    pub max_tokens: usize,
    pub temperature: f64,
    pub top_p: f64,
    pub stop_sequences: Vec<String>,
    pub system_prompt: Option<String>,
}

impl Default for GenConfig {
    fn default() -> Self {
        Self {
            max_tokens: 1024,
            temperature: 0.7,
            top_p: 0.95,
            stop_sequences: Vec::new(),
            system_prompt: None,
        }
    }
}

/// Token in a stream
#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub is_last: bool,
}

pub type TokenStream = BoxStream<'static, Result<Token>>;

/// Apply stop sequences to already generated text.
pub fn apply_stop_sequences(mut text: String, stop_sequences: &[String]) -> String {
    if stop_sequences.is_empty() {
        return text;
    }

    if let Some(index) = stop_sequences
        .iter()
        .filter(|sequence| !sequence.is_empty())
        .filter_map(|sequence| text.find(sequence))
        .min()
    {
        text.truncate(index);
    }

    text
}

/// LLM engine trait
#[async_trait]
pub trait LLMEngine: Send + Sync {
    async fn generate(&self, prompt: &str, config: &GenConfig) -> Result<String>;
    async fn generate_stream(&self, prompt: &str, config: &GenConfig) -> Result<TokenStream>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn model_info(&self) -> ModelInfo;
}

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub parameters: String,
    pub context_length: usize,
}

/// Mock LLM for testing
pub struct MockLLMEngine {
    pub info: ModelInfo,
    response: Option<String>,
}

impl MockLLMEngine {
    pub fn new() -> Self {
        Self {
            info: ModelInfo {
                name: "mock-ria-8b".to_string(),
                parameters: "8B".to_string(),
                context_length: 128_000,
            },
            response: None,
        }
    }

    pub fn with_response(response: impl Into<String>) -> Self {
        Self {
            response: Some(response.into()),
            ..Self::new()
        }
    }
}

#[async_trait]
impl LLMEngine for MockLLMEngine {
    async fn generate(&self, _prompt: &str, config: &GenConfig) -> Result<String> {
        let response = self
            .response
            .clone()
            .unwrap_or_else(|| "Mock response from RIA-8B".to_string());
        Ok(apply_stop_sequences(response, &config.stop_sequences))
    }

    async fn generate_stream(&self, prompt: &str, config: &GenConfig) -> Result<TokenStream> {
        use futures::stream;
        let response = self.generate(prompt, config).await?;
        let mut parts = response
            .split_inclusive(char::is_whitespace)
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if parts.is_empty() {
            parts.push(String::new());
        }
        let last_index = parts.len().saturating_sub(1);
        let tokens = parts
            .into_iter()
            .enumerate()
            .map(|(index, text)| {
                Ok(Token {
                    text,
                    is_last: index == last_index,
                })
            })
            .collect::<Vec<_>>();
        Ok(Box::pin(stream::iter(tokens)))
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3])
    }

    fn model_info(&self) -> ModelInfo {
        self.info.clone()
    }
}
