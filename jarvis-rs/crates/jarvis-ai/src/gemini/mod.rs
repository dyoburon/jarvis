//! Google Gemini API client.
//!
//! Implements the `AiClient` trait for Gemini models via the
//! Generative Language API.

mod api;
mod client;
mod config;

pub use client::GeminiClient;
pub use config::GeminiConfig;
