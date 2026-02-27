//! OpenAI Whisper API client for speech-to-text.
//!
//! Used for voice input — records audio and transcribes it to text
//! via the Whisper API.

use tracing::debug;

use crate::AiError;

const WHISPER_API_URL: &str = "https://api.openai.com/v1/audio/transcriptions";

/// Whisper API client configuration.
#[derive(Clone)]
pub struct WhisperConfig {
    pub api_key: String,
    pub model: String,
    pub language: Option<String>,
}

impl std::fmt::Debug for WhisperConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WhisperConfig")
            .field("api_key", &"[REDACTED]")
            .field("model", &self.model)
            .field("language", &self.language)
            .finish()
    }
}

impl WhisperConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "whisper-1".to_string(),
            language: None,
        }
    }

    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }
}

/// Whisper speech-to-text client.
pub struct WhisperClient {
    config: WhisperConfig,
    http: reqwest::Client,
}

impl WhisperClient {
    pub fn new(config: WhisperConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    /// Transcribe audio bytes to text.
    ///
    /// `audio_data` should be valid audio in a supported format
    /// (mp3, mp4, mpeg, mpga, m4a, wav, webm).
    /// `filename` is used for the multipart form (e.g., "audio.wav").
    pub async fn transcribe(&self, audio_data: Vec<u8>, filename: &str) -> Result<String, AiError> {
        debug!(
            model = %self.config.model,
            size = audio_data.len(),
            "Whisper transcription request"
        );

        let mime = match filename.rsplit('.').next() {
            Some("mp3") => "audio/mpeg",
            Some("m4a") => "audio/mp4",
            Some("webm") => "audio/webm",
            Some("ogg") => "audio/ogg",
            _ => "audio/wav",
        };

        let file_part = reqwest::multipart::Part::bytes(audio_data)
            .file_name(filename.to_string())
            .mime_str(mime)
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        let mut form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", self.config.model.clone());

        if let Some(ref lang) = self.config.language {
            form = form.text("language", lang.clone());
        }

        let response = self
            .http
            .post(WHISPER_API_URL)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .multipart(form)
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

        json["text"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| AiError::ParseError("no 'text' field in response".to_string()))
    }
}
