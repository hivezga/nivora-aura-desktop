/// Unified error handling for Nivora Aura
///
/// This module provides a centralized error type that encompasses all possible
/// errors that can occur in the application. Using `thiserror`, we derive the
/// Error trait and provide clean, descriptive error messages.

use thiserror::Error;

/// Main error type for all Aura operations
#[derive(Error, Debug)]
pub enum AuraError {
    /// LLM-related errors (API calls, server connectivity, response parsing)
    #[error("LLM error: {0}")]
    Llm(String),

    /// Database errors (SQLite operations, schema issues)
    #[error("Database error: {0}")]
    Database(String),

    /// Voice pipeline errors (STT, recording, audio capture)
    #[error("Voice pipeline error: {0}")]
    VoicePipeline(String),

    /// Text-to-Speech errors (Piper subprocess, audio playback)
    #[error("TTS error: {0}")]
    Tts(String),

    /// Configuration and settings errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Secrets management errors (keyring access, API key storage)
    #[error("Secrets error: {0}")]
    Secrets(String),

    /// Spotify integration errors (OAuth, API calls, playback)
    #[error("Spotify error: {0}")]
    Spotify(String),

    /// Home Assistant integration errors (WebSocket, REST API, authentication)
    #[error("Home Assistant error: {0}")]
    HomeAssistant(String),

    /// File I/O errors (model loading, directory access)
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP request errors (LLM API calls)
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Generic errors for edge cases
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Helper trait to convert external error types to AuraError
pub trait IntoAuraError<T> {
    fn map_aura_err<F>(self, f: F) -> Result<T, AuraError>
    where
        F: FnOnce(String) -> AuraError;
}

impl<T, E: std::fmt::Display> IntoAuraError<T> for Result<T, E> {
    fn map_aura_err<F>(self, f: F) -> Result<T, AuraError>
    where
        F: FnOnce(String) -> AuraError,
    {
        self.map_err(|e| f(e.to_string()))
    }
}

/// Implement Serialize for AuraError so it can be sent to the frontend
impl serde::Serialize for AuraError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AuraError::Llm("Connection failed".to_string());
        assert_eq!(err.to_string(), "LLM error: Connection failed");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let aura_err: AuraError = io_err.into();
        assert!(aura_err.to_string().contains("file not found"));
    }
}
