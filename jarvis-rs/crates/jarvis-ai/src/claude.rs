//! Anthropic Claude API client.
//!
//! Implements the `AiClient` trait for Claude models via the
//! Anthropic Messages API (https://api.anthropic.com/v1/messages).
//!
//! Uses Cloud OAuth tokens (via `CLAUDE_CODE_OAUTH_TOKEN`) for
//! authentication, matching the Python Jarvis implementation.

use async_trait::async_trait;
use tracing::{debug, warn};

use crate::streaming::{parse_sse_stream, SseEvent};
use crate::tools::to_claude_tool;
use crate::{AiClient, AiError, AiResponse, Message, Role, ToolCall, ToolDefinition, TokenUsage};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Claude API client configuration.
#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    pub oauth_token: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
    pub system_prompt: Option<String>,
}

impl ClaudeConfig {
    pub fn new(oauth_token: impl Into<String>) -> Self {
        Self {
            oauth_token: oauth_token.into(),
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            system_prompt: None,
        }
    }

    /// Create config from the `CLAUDE_CODE_OAUTH_TOKEN` environment variable.
    pub fn from_env() -> Result<Self, AiError> {
        let token = std::env::var("CLAUDE_CODE_OAUTH_TOKEN").map_err(|_| {
            AiError::ApiError(
                "CLAUDE_CODE_OAUTH_TOKEN not set — required for Cloud OAuth auth".into(),
            )
        })?;
        Ok(Self::new(token))
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

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
}

/// Claude API client.
pub struct ClaudeClient {
    config: ClaudeConfig,
    http: reqwest::Client,
}

impl ClaudeClient {
    pub fn new(config: ClaudeConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    /// Build the JSON request body for the Messages API.
    fn build_request_body(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        stream: bool,
    ) -> serde_json::Value {
        let mut msgs = Vec::new();
        for msg in messages {
            let role = match msg.role {
                Role::User | Role::Tool => "user",
                Role::Assistant => "assistant",
                Role::System => continue, // system is separate in Claude API
            };
            msgs.push(serde_json::json!({
                "role": role,
                "content": msg.content,
            }));
        }

        let mut body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "messages": msgs,
        });

        if let Some(ref system) = self.config.system_prompt {
            body["system"] = serde_json::json!(system);
        } else {
            // Check for system message in the messages list
            for msg in messages {
                if msg.role == Role::System {
                    body["system"] = serde_json::json!(msg.content);
                    break;
                }
            }
        }

        if !tools.is_empty() {
            let tool_defs: Vec<_> = tools.iter().map(to_claude_tool).collect();
            body["tools"] = serde_json::json!(tool_defs);
        }

        if stream {
            body["stream"] = serde_json::json!(true);
        }

        body
    }

    /// Parse a non-streaming response.
    fn parse_response(&self, json: serde_json::Value) -> Result<AiResponse, AiError> {
        let content = json["content"]
            .as_array()
            .and_then(|blocks| {
                blocks.iter().find_map(|b| {
                    if b["type"] == "text" {
                        b["text"].as_str().map(String::from)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_default();

        let tool_calls = json["content"]
            .as_array()
            .map(|blocks| {
                blocks
                    .iter()
                    .filter(|b| b["type"] == "tool_use")
                    .map(|b| ToolCall {
                        id: b["id"].as_str().unwrap_or("").to_string(),
                        name: b["name"].as_str().unwrap_or("").to_string(),
                        arguments: b["input"].clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let usage = TokenUsage {
            input_tokens: json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
        };

        Ok(AiResponse {
            content,
            tool_calls,
            usage,
        })
    }
}

#[async_trait]
impl AiClient for ClaudeClient {
    async fn send_message(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<AiResponse, AiError> {
        let body = self.build_request_body(messages, tools, false);

        debug!(model = %self.config.model, "Claude API request");

        let response = self
            .http
            .post(CLAUDE_API_URL)
            .header("x-api-key", &self.config.oauth_token)
            .header("anthropic-version", ANTHROPIC_VERSION)
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
        let body = self.build_request_body(messages, tools, true);

        debug!(model = %self.config.model, "Claude API streaming request");

        let response = self
            .http
            .post(CLAUDE_API_URL)
            .header("x-api-key", &self.config.oauth_token)
            .header("anthropic-version", ANTHROPIC_VERSION)
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
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut usage = TokenUsage::default();

        // Current tool_use block being built
        let mut current_tool_id = String::new();
        let mut current_tool_name = String::new();
        let mut current_tool_json = String::new();

        parse_sse_stream(response, |event: SseEvent| {
            let event_type = event.event.as_deref().unwrap_or("");

            // Extract text chunk outside data's scope to avoid lifetime issues
            let mut chunk = String::new();

            match event_type {
                "content_block_delta" => {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&event.data) {
                        let delta_type = data["delta"]["type"].as_str().unwrap_or("");
                        match delta_type {
                            "text_delta" => {
                                if let Some(t) = data["delta"]["text"].as_str() {
                                    chunk = t.to_string();
                                    full_content.push_str(&chunk);
                                }
                            }
                            "input_json_delta" => {
                                if let Some(json_part) =
                                    data["delta"]["partial_json"].as_str()
                                {
                                    current_tool_json.push_str(json_part);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "content_block_start" => {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&event.data) {
                        if data["content_block"]["type"] == "tool_use" {
                            current_tool_id = data["content_block"]["id"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            current_tool_name = data["content_block"]["name"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            current_tool_json.clear();
                        }
                    }
                }
                "content_block_stop" => {
                    if !current_tool_name.is_empty() {
                        let arguments = serde_json::from_str(&current_tool_json)
                            .unwrap_or(serde_json::Value::Null);
                        tool_calls.push(ToolCall {
                            id: std::mem::take(&mut current_tool_id),
                            name: std::mem::take(&mut current_tool_name),
                            arguments,
                        });
                        current_tool_json.clear();
                    }
                }
                "message_delta" => {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&event.data) {
                        if let Some(u) = data.get("usage") {
                            usage.output_tokens =
                                u["output_tokens"].as_u64().unwrap_or(0) as u32;
                        }
                    }
                }
                "message_start" => {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&event.data) {
                        if let Some(u) = data["message"].get("usage") {
                            usage.input_tokens =
                                u["input_tokens"].as_u64().unwrap_or(0) as u32;
                        }
                    }
                }
                _ => {}
            }

            if !chunk.is_empty() {
                on_chunk(chunk);
            }
        })
        .await?;

        if usage.input_tokens == 0 && usage.output_tokens == 0 {
            warn!("No usage data received in streaming response");
        }

        Ok(AiResponse {
            content: full_content,
            tool_calls,
            usage,
        })
    }
}
