//! AiClient trait implementation for ClaudeClient (send_message + streaming).

use async_trait::async_trait;
use tracing::{debug, warn};

use crate::streaming::{parse_sse_stream, SseEvent};
use crate::{AiClient, AiError, AiResponse, Message, TokenUsage, ToolCall, ToolDefinition};

use super::client::{ClaudeClient, ANTHROPIC_VERSION, CLAUDE_API_URL};

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
            .header(
                "Authorization",
                format!("Bearer {}", self.config.oauth_token),
            )
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
            let text = text.chars().take(200).collect::<String>();
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
            .header(
                "Authorization",
                format!("Bearer {}", self.config.oauth_token),
            )
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
            let text = text.chars().take(200).collect::<String>();
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
                                if let Some(json_part) = data["delta"]["partial_json"].as_str() {
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
                            usage.output_tokens = u["output_tokens"].as_u64().unwrap_or(0);
                        }
                    }
                }
                "message_start" => {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&event.data) {
                        if let Some(u) = data["message"].get("usage") {
                            usage.input_tokens = u["input_tokens"].as_u64().unwrap_or(0);
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
