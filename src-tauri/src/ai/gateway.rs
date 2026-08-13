use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::ollama::OllamaProvider;
use super::openai::OpenAiProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: "openai".into(),
            api_key: None,
            base_url: None,
            model: "gpt-4o-mini".into(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        on_chunk: Box<dyn Fn(String) + Send>,
    ) -> Result<String, String>;
}

pub fn create_provider(config: &AiConfig) -> Result<Box<dyn LlmProvider>, String> {
    match config.provider.as_str() {
        "openai" | "custom" => {
            let api_key = config
                .api_key
                .as_deref()
                .filter(|k| !k.is_empty())
                .ok_or_else(|| "API key is required for OpenAI provider".to_string())?;
            let base_url = config
                .base_url
                .as_deref()
                .filter(|u| !u.is_empty())
                .unwrap_or("https://api.openai.com");
            Ok(Box::new(OpenAiProvider::new(
                api_key,
                base_url,
                &config.model,
                config.temperature,
                config.max_tokens,
            )))
        }
        "ollama" => {
            let base_url = config
                .base_url
                .as_deref()
                .filter(|u| !u.is_empty())
                .unwrap_or("http://localhost:11434");
            Ok(Box::new(OllamaProvider::new(
                base_url,
                &config.model,
                config.temperature,
                config.max_tokens,
            )))
        }
        other => Err(format!("Unknown AI provider: {other}")),
    }
}
