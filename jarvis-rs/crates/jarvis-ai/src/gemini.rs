//! Google Gemini API client.
//!
//! Implements the `AiClient` trait for Gemini models via the
//! Generative Language API.

use async_trait::async_trait;
use tracing::debug;

use crate::streaming::{parse_sse_stream, SseEvent};
use crate::tools::to_gemini_tool;
use crate::{AiClient, AiError, AiResponse, Message, Role, ToolCall, ToolDefinition, TokenUsage};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// Gemini API client configuration.
#[derive(Debug, Clone)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
}

impl GeminiConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "gemini-2.0-flash".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }
}

/// Gemini API client.
pub struct GeminiClient {
    config: GeminiConfig,
    http: reqwest::Client,
}

impl GeminiClient {
    pub fn new(config: GeminiConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    fn api_url(&self, stream: bool) -> String {
        let method = if stream {
            "streamGenerateContent"
        } else {
            "generateContent"
        };
        format!(
            "{}/{}:{}?key={}",
            GEMINI_API_BASE, self.config.model, method, self.config.api_key
        )
    }

    /// Build the JSON request body for the Gemini API.
    fn build_request_body(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> serde_json::Value {
        let mut contents = Vec::new();

        for msg in messages {
            let role = match msg.role {
                Role::User | Role::Tool => "user",
                Role::Assistant => "model",
                Role::System => continue, // handled via systemInstruction
            };
            contents.push(serde_json::json!({
                "role": role,
                "parts": [{ "text": msg.content }]
            }));
        }

        let mut body = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": self.config.max_tokens,
                "temperature": self.config.temperature,
            }
        });

        // System instruction
        for msg in messages {
            if msg.role == Role::System {
                body["systemInstruction"] = serde_json::json!({
                    "parts": [{ "text": msg.content }]
                });
                break;
            }
        }

        if !tools.is_empty() {
            let tool_defs: Vec<_> = tools.iter().map(to_gemini_tool).collect();
            body["tools"] = serde_json::json!([{
                "functionDeclarations": tool_defs
            }]);
        }

        body
    }

    /// Parse a Gemini response.
    fn parse_response(&self, json: serde_json::Value) -> Result<AiResponse, AiError> {
        let candidates = json["candidates"]
            .as_array()
            .ok_or_else(|| AiError::ParseError("no candidates in response".to_string()))?;

        let first = candidates
            .first()
            .ok_or_else(|| AiError::ParseError("empty candidates".to_string()))?;

        let parts = first["content"]["parts"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for part in &parts {
            if let Some(text) = part["text"].as_str() {
                content.push_str(text);
            }
            if let Some(fc) = part.get("functionCall") {
                tool_calls.push(ToolCall {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: fc["name"].as_str().unwrap_or("").to_string(),
                    arguments: fc["args"].clone(),
                });
            }
        }

        let usage = TokenUsage {
            input_tokens: json["usageMetadata"]["promptTokenCount"]
                .as_u64()
                .unwrap_or(0) as u32,
            output_tokens: json["usageMetadata"]["candidatesTokenCount"]
                .as_u64()
                .unwrap_or(0) as u32,
        };

        Ok(AiResponse {
            content,
            tool_calls,
            usage,
        })
    }
}

#[async_trait]
impl AiClient for GeminiClient {
    async fn send_message(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<AiResponse, AiError> {
        let body = self.build_request_body(messages, tools);
        let url = self.api_url(false);

        debug!(model = %self.config.model, "Gemini API request");

        let response = self
            .http
            .post(&url)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        let status = response.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AiError::RateLimited);
        }
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AiError::ApiError(format!("HTTP {status}: {text}")));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AiError::ParseError(e.to_string()))?;

        self.parse_response(json)
    }

    async fn send_message_streaming(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
    ) -> Result<AiResponse, AiError> {
        let body = self.build_request_body(messages, tools);
        let url = format!("{}&alt=sse", self.api_url(true));

        debug!(model = %self.config.model, "Gemini API streaming request");

        let response = self
            .http
            .post(&url)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        let status = response.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AiError::RateLimited);
        }
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AiError::ApiError(format!("HTTP {status}: {text}")));
        }

        let mut full_content = String::new();
        let mut tool_calls = Vec::new();
        let mut usage = TokenUsage::default();

        parse_sse_stream(response, |event: SseEvent| {
            let mut chunk = String::new();

            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&event.data) {
                // Extract text from candidates
                if let Some(candidates) = data["candidates"].as_array() {
                    for candidate in candidates {
                        if let Some(parts) = candidate["content"]["parts"].as_array() {
                            for part in parts {
                                if let Some(t) = part["text"].as_str() {
                                    if !t.is_empty() {
                                        chunk.push_str(t);
                                        full_content.push_str(t);
                                    }
                                }
                                if let Some(fc) = part.get("functionCall") {
                                    tool_calls.push(ToolCall {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        name: fc["name"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                        arguments: fc["args"].clone(),
                                    });
                                }
                            }
                        }
                    }
                }

                // Extract usage
                if let Some(meta) = data.get("usageMetadata") {
                    usage.input_tokens =
                        meta["promptTokenCount"].as_u64().unwrap_or(0) as u32;
                    usage.output_tokens =
                        meta["candidatesTokenCount"].as_u64().unwrap_or(0) as u32;
                }
            }

            if !chunk.is_empty() {
                on_chunk(chunk);
            }
        })
        .await?;

        Ok(AiResponse {
            content: full_content,
            tool_calls,
            usage,
        })
    }
}
