//! Skill router — dispatches AI requests to the appropriate provider.
//!
//! The router selects between Claude, Gemini, or other providers
//! based on the task type, cost, and availability.

use std::collections::HashMap;
use std::sync::Arc;

use crate::{AiClient, AiError, AiResponse, Message, ToolDefinition};

/// Which AI provider to use for a given task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    Claude,
    Gemini,
}

/// A skill that can be routed to a specific provider.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Unique name for this skill (e.g., "code_assist", "general_chat").
    pub name: String,
    /// Which provider to use.
    pub provider: Provider,
    /// Optional system prompt override for this skill.
    pub system_prompt: Option<String>,
}

/// Routes AI requests to registered providers based on skill type.
pub struct SkillRouter {
    /// Registered AI clients by provider.
    clients: HashMap<Provider, Arc<dyn AiClient>>,
    /// Registered skills.
    skills: HashMap<String, Skill>,
    /// Default provider when no skill matches.
    default_provider: Provider,
}

impl SkillRouter {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            skills: HashMap::new(),
            default_provider: Provider::Claude,
        }
    }

    /// Register an AI client for a provider.
    pub fn register_client(&mut self, provider: Provider, client: Arc<dyn AiClient>) {
        self.clients.insert(provider, client);
    }

    /// Register a skill with its routing configuration.
    pub fn register_skill(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Set the default provider.
    pub fn set_default_provider(&mut self, provider: Provider) {
        self.default_provider = provider;
    }

    /// Route a message to the appropriate provider based on skill name.
    pub async fn route(
        &self,
        skill_name: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<AiResponse, AiError> {
        let provider = self
            .skills
            .get(skill_name)
            .map(|s| s.provider)
            .unwrap_or(self.default_provider);

        let client = self
            .clients
            .get(&provider)
            .ok_or_else(|| {
                AiError::ApiError(format!("No client registered for provider {provider:?}"))
            })?;

        client.send_message(messages, tools).await
    }

    /// Route with streaming.
    pub async fn route_streaming(
        &self,
        skill_name: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
    ) -> Result<AiResponse, AiError> {
        let provider = self
            .skills
            .get(skill_name)
            .map(|s| s.provider)
            .unwrap_or(self.default_provider);

        let client = self
            .clients
            .get(&provider)
            .ok_or_else(|| {
                AiError::ApiError(format!("No client registered for provider {provider:?}"))
            })?;

        client
            .send_message_streaming(messages, tools, on_chunk)
            .await
    }

    /// Get a client for a specific provider.
    pub fn client(&self, provider: Provider) -> Option<&Arc<dyn AiClient>> {
        self.clients.get(&provider)
    }

    /// List all registered skills.
    pub fn skills(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }
}

impl Default for SkillRouter {
    fn default() -> Self {
        Self::new()
    }
}
