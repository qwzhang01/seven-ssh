use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};

use super::gateway::{ChatMessage, LlmProvider};

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
}

impl OpenAiProvider {
    pub fn new(
        api_key: &str,
        base_url: &str,
        model: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            temperature,
            max_tokens,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        on_chunk: Box<dyn Fn(String) + Send>,
    ) -> Result<String, String> {
        let url = format!("{}/v1/chat/completions", self.base_url);

        let msgs: Vec<Value> = messages
            .iter()
            .map(|m| json!({ "role": m.role, "content": m.content }))
            .collect();

        let body = json!({
            "model": self.model,
            "messages": msgs,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
            "stream": true,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("API error ({status}): {text}"));
        }

        let mut full_content = String::new();
        let mut stream = response.bytes_stream();

        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream error: {e}"))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                        if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                            on_chunk(content.to_string());
                            full_content.push_str(content);
                        }
                    }
                }
            }
        }

        Ok(full_content)
    }
}
