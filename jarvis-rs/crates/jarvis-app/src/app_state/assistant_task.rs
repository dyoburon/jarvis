//! Background async task that manages the Claude AI session.

use super::types::AssistantEvent;

/// Background task that manages the Claude AI session.
pub(super) async fn assistant_task(
    user_rx: std::sync::mpsc::Receiver<String>,
    event_tx: std::sync::mpsc::Sender<AssistantEvent>,
) {
    let config = match jarvis_ai::ClaudeConfig::from_env() {
        Ok(c) => c.with_system_prompt(
            "You are Jarvis, an AI assistant embedded in a terminal emulator. \
             Be concise and helpful. Use plain text, not markdown.",
        ),
        Err(e) => {
            let _ = event_tx.send(AssistantEvent::Error(format!(
                "Claude API not configured: {e}"
            )));
            return;
        }
    };

    let _ = event_tx.send(AssistantEvent::Initialized {
        model_name: config.model.clone(),
    });

    let client = jarvis_ai::ClaudeClient::new(config);
    let mut session = jarvis_ai::Session::new("claude").with_system_prompt(
        "You are Jarvis, an AI assistant embedded in a terminal emulator. \
         Be concise and helpful. Use plain text, not markdown.",
    );

    while let Ok(msg) = tokio::task::block_in_place(|| user_rx.recv()) {
        let tx = event_tx.clone();
        let on_chunk = Box::new(move |chunk: String| {
            let _ = tx.send(AssistantEvent::StreamChunk(chunk));
        });

        match session.chat_streaming(&client, &msg, on_chunk).await {
            Ok(_) => {
                let _ = event_tx.send(AssistantEvent::Done);
            }
            Err(e) => {
                let _ = event_tx.send(AssistantEvent::Error(e.to_string()));
            }
        }
    }
}
