//! AiClient trait implementation for GeminiClient (send_message + streaming).

use async_trait::async_trait;
use tracing::debug;

use crate::streaming::{parse_sse_stream, SseEvent};
use crate::{AiClient, AiError, AiResponse, Message, TokenUsage, ToolCall, ToolDefinition};

use super::client::GeminiClient;

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
            .header("x-goog-api-key", &self.config.api_key)
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
        let url = format!("{}?alt=sse", self.api_url(true));

        debug!(model = %self.config.model, "Gemini API streaming request");

        let response = self
            .http
            .post(&url)
            .header("content-type", "application/json")
            .header("x-goog-api-key", &self.config.api_key)
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
                                        name: fc["name"].as_str().unwrap_or("").to_string(),
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
                        meta["promptTokenCount"].as_u64().unwrap_or(0);
                    usage.output_tokens =
                        meta["candidatesTokenCount"].as_u64().unwrap_or(0);
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
