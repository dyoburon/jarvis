//! Assistant panel state for the AI chat overlay.

/// Role of a chat message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
}

/// A single chat message for display.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

/// Visual state for the assistant overlay panel.
pub struct AssistantPanel {
    messages: Vec<ChatMessage>,
    input_text: String,
    streaming_text: String,
    is_streaming: bool,
    scroll_offset: usize,
    error: Option<String>,
}

impl Default for AssistantPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl AssistantPanel {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_text: String::new(),
            streaming_text: String::new(),
            is_streaming: false,
            scroll_offset: 0,
            error: None,
        }
    }

    pub fn append_char(&mut self, c: char) {
        self.input_text.push(c);
    }

    pub fn backspace(&mut self) {
        self.input_text.pop();
    }

    /// Take the current input text, clearing the buffer.
    pub fn take_input(&mut self) -> String {
        std::mem::take(&mut self.input_text)
    }

    pub fn push_user_message(&mut self, text: String) {
        self.messages.push(ChatMessage {
            role: ChatRole::User,
            content: text,
        });
        self.scroll_offset = 0;
    }

    pub fn push_assistant_message(&mut self, text: String) {
        self.messages.push(ChatMessage {
            role: ChatRole::Assistant,
            content: text,
        });
    }

    pub fn append_streaming_chunk(&mut self, chunk: &str) {
        self.is_streaming = true;
        self.streaming_text.push_str(chunk);
        self.error = None;
    }

    /// Commit the streaming text as a completed assistant message.
    pub fn finish_streaming(&mut self) {
        if !self.streaming_text.is_empty() {
            let text = std::mem::take(&mut self.streaming_text);
            self.push_assistant_message(text);
        }
        self.is_streaming = false;
    }

    pub fn set_error(&mut self, msg: String) {
        self.error = Some(msg);
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(n);
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    // -- Getters --

    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn input_text(&self) -> &str {
        &self.input_text
    }

    pub fn streaming_text(&self) -> &str {
        &self.streaming_text
    }

    pub fn is_streaming(&self) -> bool {
        self.is_streaming
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }
}
