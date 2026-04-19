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
}

impl MockLLMEngine {
    pub fn new() -> Self {
        Self {
            info: ModelInfo {
                name: "mock-ria-8b".to_string(),
                parameters: "8B".to_string(),
                context_length: 128_000,
            },
        }
    }
}

#[async_trait]
impl LLMEngine for MockLLMEngine {
    async fn generate(&self, _prompt: &str, _config: &GenConfig) -> Result<String> {
        Ok("Mock response from RIA-8B".to_string())
    }

    async fn generate_stream(&self, _prompt: &str, _config: &GenConfig) -> Result<TokenStream> {
        use futures::stream;
        let tokens = vec![
            Ok(Token {
                text: "Mock ".to_string(),
                is_last: false,
            }),
            Ok(Token {
                text: "response ".to_string(),
                is_last: false,
            }),
            Ok(Token {
                text: "from ".to_string(),
                is_last: false,
            }),
            Ok(Token {
                text: "RIA-8B".to_string(),
                is_last: true,
            }),
        ];
        Ok(Box::pin(stream::iter(tokens)))
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3])
    }

    fn model_info(&self) -> ModelInfo {
        self.info.clone()
    }
}
