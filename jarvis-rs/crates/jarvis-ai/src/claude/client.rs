//! Claude API client struct, request building, and response parsing.

use crate::tools::to_claude_tool;
use crate::{AiError, AiResponse, Message, Role, TokenUsage, ToolCall, ToolDefinition};

use super::config::ClaudeConfig;

pub(crate) const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
pub(crate) const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Claude API client.
pub struct ClaudeClient {
    pub(crate) config: ClaudeConfig,
    pub(crate) http: reqwest::Client,
}

impl ClaudeClient {
    pub fn new(config: ClaudeConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    /// Return the API URL (both auth methods use api.anthropic.com).
    pub(crate) fn api_url(&self) -> &'static str {
        ANTHROPIC_API_URL
    }

    /// Build auth headers for the configured auth method.
    pub(crate) fn auth_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        match self.config.auth_method {
            super::config::AuthMethod::ApiKey => {
                headers.insert(
                    "x-api-key",
                    self.config.token.parse().expect("invalid API key header"),
                );
            }
            super::config::AuthMethod::OAuth => {
                headers.insert(
                    "Authorization",
                    format!("Bearer {}", self.config.token)
                        .parse()
                        .expect("invalid OAuth header"),
                );
            }
        }
        headers.insert(
            "anthropic-version",
            ANTHROPIC_VERSION.parse().expect("invalid version header"),
        );
        headers
    }

    /// Build the JSON request body for the Messages API.
    pub(crate) fn build_request_body(
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
    pub(crate) fn parse_response(&self, json: serde_json::Value) -> Result<AiResponse, AiError> {
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
            input_tokens: json["usage"]["input_tokens"].as_u64().unwrap_or(0),
            output_tokens: json["usage"]["output_tokens"].as_u64().unwrap_or(0),
        };

        Ok(AiResponse {
            content,
            tool_calls,
            usage,
        })
    }
}
