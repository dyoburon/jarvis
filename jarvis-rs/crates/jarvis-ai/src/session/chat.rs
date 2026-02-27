//! Async chat methods for Session (send_message + streaming).

use crate::{AiClient, AiError, Message, Role};

use super::manager::Session;
use super::types::BusyGuard;

impl Session {
    /// Add a user message and get the assistant's response.
    /// If the AI calls tools, this runs the tool-call loop automatically.
    pub async fn chat(
        &mut self,
        client: &dyn AiClient,
        user_message: impl Into<String>,
    ) -> Result<String, AiError> {
        let _guard = BusyGuard::acquire(&self.busy)?;

        self.messages.push(Message {
            role: Role::User,
            content: user_message.into(),
        });

        let mut messages = self.build_messages();
        let mut rounds = 0;

        loop {
            let response = client.send_message(&messages, &self.tools).await?;
            self.tracker.record(&self.provider, &response.usage);

            if response.tool_calls.is_empty() || self.tool_executor.is_none() {
                // No tool calls — we have the final response
                self.messages.push(Message {
                    role: Role::Assistant,
                    content: response.content.clone(),
                });
                return Ok(response.content);
            }

            // Execute tool calls
            rounds += 1;
            if rounds > self.max_tool_rounds {
                tracing::debug!("Max tool rounds reached, returning partial response");
                self.messages.push(Message {
                    role: Role::Assistant,
                    content: response.content.clone(),
                });
                return Ok(response.content);
            }

            // Add assistant message with tool calls
            messages.push(Message {
                role: Role::Assistant,
                content: response.content.clone(),
            });

            // Execute each tool and add results
            let executor = self.tool_executor.as_ref().unwrap();
            for tool_call in &response.tool_calls {
                let result = self.execute_tool(executor, tool_call);
                messages.push(Message {
                    role: Role::Tool,
                    content: format!("[Tool Result: {}]\n{}", tool_call.name, result),
                });
            }
        }
    }

    /// Send a message with streaming, returning the full response.
    pub async fn chat_streaming(
        &mut self,
        client: &dyn AiClient,
        user_message: impl Into<String>,
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
    ) -> Result<String, AiError> {
        let _guard = BusyGuard::acquire(&self.busy)?;

        self.messages.push(Message {
            role: Role::User,
            content: user_message.into(),
        });

        let messages = self.build_messages();
        let response = client
            .send_message_streaming(&messages, &self.tools, on_chunk)
            .await?;

        self.tracker.record(&self.provider, &response.usage);
        self.messages.push(Message {
            role: Role::Assistant,
            content: response.content.clone(),
        });

        Ok(response.content)
    }
}
