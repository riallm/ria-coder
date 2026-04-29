//! RIA LLM Engine implementation (SPEC-026)

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use ria_core::TokenizerConfig;
use ria_tokenizer::RiaTokenizer;
use riallm::{
    config::{DeviceSpec, ModelOptions},
    AirLLMBaseModel, AutoModel,
};

use crate::llm::{apply_stop_sequences, GenConfig, LLMEngine, ModelInfo, Token, TokenStream};

pub struct RiaLLMEngine {
    model: Arc<Mutex<AirLLMBaseModel>>,
    tokenizer: Arc<RiaTokenizer>,
    info: ModelInfo,
}

impl RiaLLMEngine {
    pub async fn new(model_path: &str) -> Result<Self> {
        let options = ModelOptions {
            device: DeviceSpec::Cpu,
            profiling_mode: true,
            prefetch_layers: true,
            ..Default::default()
        };

        // We use AutoModel::from_pretrained to get the AirLLMBaseModel
        let model = AutoModel::from_pretrained(model_path, Some(options))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load model: {}", e))?;

        // Initialize tokenizer with standard RIA config
        let tok_config = TokenizerConfig::default();
        let tokenizer = RiaTokenizer::new(tok_config);

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            tokenizer: Arc::new(tokenizer),
            info: ModelInfo {
                name: model_path.to_string(),
                parameters: "unknown".to_string(),
                context_length: 128_000,
            },
        })
    }
}

#[async_trait]
impl LLMEngine for RiaLLMEngine {
    async fn generate(&self, prompt: &str, config: &GenConfig) -> Result<String> {
        let prompt = match &config.system_prompt {
            Some(system_prompt) if !system_prompt.is_empty() => {
                format!("{system_prompt}\n\n{prompt}")
            }
            _ => prompt.to_string(),
        };

        let input_tokens = self
            .tokenizer
            .encode(&prompt)
            .map_err(|e| anyhow::anyhow!("Tokenization error: {:?}", e))?;
        let input_len = input_tokens.len();

        use candle_core::{Device, Tensor};
        let device = Device::Cpu; // Default for now
        let input_ids = Tensor::new(
            input_tokens.iter().map(|&id| id as u32).collect::<Vec<_>>(),
            &device,
        )?
        .reshape((1, input_len))?;

        let mut model = self.model.lock().await;
        let output_tokens = model
            .generate(
                &input_ids,
                config.max_tokens,
                config.temperature,
                Some(config.top_p),
            )
            .map_err(|e| anyhow::anyhow!("Inference error: {:?}", e))?;
        let completion_tokens = if output_tokens.len() > input_len {
            &output_tokens[input_len..]
        } else {
            output_tokens.as_slice()
        };

        let decoded = self
            .tokenizer
            .decode(
                &completion_tokens
                    .iter()
                    .map(|&id| id as usize)
                    .collect::<Vec<_>>(),
            )
            .map_err(|e| anyhow::anyhow!("Decoding error: {:?}", e))?;

        Ok(apply_stop_sequences(decoded, &config.stop_sequences))
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
        let tokens = parts.into_iter().enumerate().map(move |(index, text)| {
            Ok(Token {
                text,
                is_last: index == last_index,
            })
        });

        Ok(Box::pin(stream::iter(tokens)))
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.0; 384])
    }

    fn model_info(&self) -> ModelInfo {
        self.info.clone()
    }
}
