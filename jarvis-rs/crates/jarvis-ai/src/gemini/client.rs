//! Gemini API client struct, request building, and response parsing.

use crate::tools::to_gemini_tool;
use crate::{AiError, AiResponse, Message, Role, TokenUsage, ToolCall, ToolDefinition};

use super::config::GeminiConfig;

pub(crate) const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// Gemini API client.
pub struct GeminiClient {
    pub(crate) config: GeminiConfig,
    pub(crate) http: reqwest::Client,
}

impl GeminiClient {
    pub fn new(config: GeminiConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    pub(crate) fn api_url(&self, stream: bool) -> String {
        let method = if stream {
            "streamGenerateContent"
        } else {
            "generateContent"
        };
        format!("{}/{}:{}", GEMINI_API_BASE, self.config.model, method)
    }

    /// Build the JSON request body for the Gemini API.
    pub(crate) fn build_request_body(
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
    pub(crate) fn parse_response(&self, json: serde_json::Value) -> Result<AiResponse, AiError> {
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
                .unwrap_or(0),
            output_tokens: json["usageMetadata"]["candidatesTokenCount"]
                .as_u64()
                .unwrap_or(0),
        };

        Ok(AiResponse {
            content,
            tool_calls,
            usage,
        })
    }
}
