//! Claude API client configuration.

use std::fmt;

use crate::AiError;

/// How the client authenticates with the Claude API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthMethod {
    /// Anthropic API key (`x-api-key` header, `api.anthropic.com`).
    ApiKey,
    /// OAuth Bearer token (`Authorization: Bearer`, `api.claude.ai`).
    OAuth,
}

/// Claude API client configuration.
#[derive(Clone)]
pub struct ClaudeConfig {
    pub token: String,
    pub auth_method: AuthMethod,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
    pub system_prompt: Option<String>,
}

impl fmt::Debug for ClaudeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClaudeConfig")
            .field("auth_method", &self.auth_method)
            .field("token", &"[REDACTED]")
            .field("model", &self.model)
            .field("max_tokens", &self.max_tokens)
            .field("temperature", &self.temperature)
            .field("system_prompt", &self.system_prompt)
            .finish()
    }
}

impl ClaudeConfig {
    pub fn new(token: impl Into<String>, auth_method: AuthMethod) -> Self {
        Self {
            token: token.into(),
            auth_method,
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            system_prompt: None,
        }
    }

    /// Create config from environment or Claude Code CLI credentials.
    ///
    /// Resolution order:
    /// 1. `ANTHROPIC_API_KEY` env var (API key auth)
    /// 2. `CLAUDE_CODE_OAUTH_TOKEN` env var (OAuth auth)
    /// 3. `~/.claude/.credentials.json` (OAuth, written by `claude auth login`)
    pub fn from_env() -> Result<Self, AiError> {
        // 1. Anthropic API key
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            return Ok(Self::new(key, AuthMethod::ApiKey));
        }

        // 2. OAuth env var
        if let Ok(token) = std::env::var("CLAUDE_CODE_OAUTH_TOKEN") {
            return Ok(Self::new(token, AuthMethod::OAuth));
        }

        // 3. Claude Code CLI credentials file
        if let Some(token) = Self::read_claude_credentials() {
            return Ok(Self::new(token, AuthMethod::OAuth));
        }

        Err(AiError::ApiError(
            "Claude API not configured. Set ANTHROPIC_API_KEY, \
             CLAUDE_CODE_OAUTH_TOKEN, or run `claude auth login`."
                .into(),
        ))
    }

    /// Read the OAuth access token from `~/.claude/.credentials.json`.
    fn read_claude_credentials() -> Option<String> {
        let home = dirs::home_dir()?;
        let path = home.join(".claude").join(".credentials.json");
        let data = std::fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&data).ok()?;
        json.get("claudeAiOauth")?
            .get("accessToken")?
            .as_str()
            .map(|s| s.to_string())
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
