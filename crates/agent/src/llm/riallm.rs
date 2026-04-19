//! RIA LLM Engine implementation (SPEC-026)

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use ria_core::TokenizerConfig;
use ria_tokenizer::RiaTokenizer;
use riallm::{config::ModelOptions, AirLLMBaseModel, AutoModel};

use crate::llm::{GenConfig, LLMEngine, ModelInfo, TokenStream};

pub struct RiaLLMEngine {
    model: Arc<Mutex<AirLLMBaseModel>>,
    tokenizer: Arc<RiaTokenizer>,
    info: ModelInfo,
}

impl RiaLLMEngine {
    pub async fn new(model_path: &str) -> Result<Self> {
        let options = ModelOptions {
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
                name: "RIA-8B".to_string(),
                parameters: "8B".to_string(),
                context_length: 128_000,
            },
        })
    }
}

#[async_trait]
impl LLMEngine for RiaLLMEngine {
    async fn generate(&self, prompt: &str, config: &GenConfig) -> Result<String> {
        let input_tokens = self
            .tokenizer
            .encode(prompt)
            .map_err(|e| anyhow::anyhow!("Tokenization error: {:?}", e))?;

        use candle_core::{Device, Tensor};
        let device = Device::Cpu; // Default for now
        let input_ids = Tensor::new(
            input_tokens.iter().map(|&id| id as u32).collect::<Vec<_>>(),
            &device,
        )?;

        let mut model = self.model.lock().await;
        let output_tokens = model
            .generate(
                &input_ids,
                config.max_tokens,
                config.temperature,
                Some(config.top_p),
            )
            .map_err(|e| anyhow::anyhow!("Inference error: {:?}", e))?;

        let decoded = self
            .tokenizer
            .decode(
                &output_tokens
                    .iter()
                    .map(|&id| id as usize)
                    .collect::<Vec<_>>(),
            )
            .map_err(|e| anyhow::anyhow!("Decoding error: {:?}", e))?;

        Ok(decoded)
    }

    async fn generate_stream(&self, _prompt: &str, _config: &GenConfig) -> Result<TokenStream> {
        Err(anyhow::anyhow!(
            "Streaming not yet implemented for RiaLLMEngine"
        ))
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.0; 384])
    }

    fn model_info(&self) -> ModelInfo {
        self.info.clone()
    }
}
