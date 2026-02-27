//! Server-Sent Events (SSE) streaming parser.
//!
//! Both Claude and Gemini APIs support SSE streaming for real-time
//! token-by-token responses. This module provides a generic SSE parser
//! that can be used with any reqwest response stream.

use futures_util::StreamExt;
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;

/// A single SSE event parsed from the stream.
#[derive(Debug, Clone)]
pub struct SseEvent {
    /// The event type (e.g., "message_start", "content_block_delta").
    pub event: Option<String>,
    /// The event data (JSON string).
    pub data: String,
}

/// Parse an SSE stream from a reqwest response, calling `on_event` for each event.
pub async fn parse_sse_stream(
    response: reqwest::Response,
    mut on_event: impl FnMut(SseEvent),
) -> Result<(), crate::AiError> {
    let byte_stream = response
        .bytes_stream()
        .map(|result| result.map_err(std::io::Error::other));
    let reader = tokio::io::BufReader::new(StreamReader::new(byte_stream));
    let mut lines = reader.lines();

    let mut current_event: Option<String> = None;
    let mut current_data = String::new();

    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|e| crate::AiError::NetworkError(e.to_string()))?
    {
        if line.is_empty() {
            // Empty line = end of event
            if !current_data.is_empty() {
                on_event(SseEvent {
                    event: current_event.take(),
                    data: std::mem::take(&mut current_data),
                });
            }
            current_event = None;
            continue;
        }

        if let Some(event_type) = line.strip_prefix("event: ") {
            current_event = Some(event_type.to_string());
        } else if let Some(data) = line.strip_prefix("data: ") {
            if !current_data.is_empty() {
                current_data.push('\n');
            }
            current_data.push_str(data);
        }
        // Ignore other fields (id:, retry:, comments)
    }

    // Flush any remaining event
    if !current_data.is_empty() {
        on_event(SseEvent {
            event: current_event,
            data: current_data,
        });
    }

    Ok(())
}
