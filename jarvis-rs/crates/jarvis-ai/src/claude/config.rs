//! Claude API client configuration.

use std::fmt;

use crate::AiError;

/// Claude API client configuration.
#[derive(Clone)]
pub struct ClaudeConfig {
    pub oauth_token: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
    pub system_prompt: Option<String>,
}

impl fmt::Debug for ClaudeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClaudeConfig")
            .field("oauth_token", &"[REDACTED]")
            .field("model", &self.model)
            .field("max_tokens", &self.max_tokens)
            .field("temperature", &self.temperature)
            .field("system_prompt", &self.system_prompt)
            .finish()
    }
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
