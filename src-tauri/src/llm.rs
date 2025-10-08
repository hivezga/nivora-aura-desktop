use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

// LLM Configuration Constants
const DEFAULT_MAX_TOKENS: u32 = 512;
const DEFAULT_TEMPERATURE: f32 = 0.7;
const DEFAULT_TIMEOUT_SECS: u64 = 120; // 2 minutes for LLM generation

/// OpenAI-compatible Chat Completion Request
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

/// Chat message in OpenAI format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String, // "system", "user", or "assistant"
    content: String,
}

/// OpenAI-compatible Chat Completion Response
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}

/// Universal LLM Client for OpenAI-compatible APIs
///
/// This module provides a "bring your own backend" approach, allowing
/// users to connect to any local AI server that exposes an OpenAI-compatible
/// API endpoint (e.g., Ollama, LM Studio, Jan.ai, LocalAI, etc.)
pub struct LLMEngine {
    client: Client,
    api_base_url: String,
    model_name: String,
    api_key: Option<String>,
    system_prompt: String,
    /// Abort handle for the current generation task (allows immediate cancellation)
    current_task: Arc<TokioMutex<Option<tokio::task::AbortHandle>>>,
}

impl LLMEngine {
    /// Create a new LLM engine configured for an OpenAI-compatible API
    ///
    /// # Arguments
    /// * `api_base_url` - Base URL of the API (e.g., "http://localhost:1234/v1")
    /// * `model_name` - Name of the model to use (e.g., "llama3", "phi3:instruct")
    /// * `api_key` - Optional API key (some local servers don't require this)
    /// * `system_prompt` - Optional custom system prompt
    pub fn new(
        api_base_url: String,
        model_name: String,
        api_key: Option<String>,
        system_prompt: Option<String>,
    ) -> Result<Self, String> {
        log::info!("Initializing LLM engine (OpenAI-compatible API)");
        log::info!("  API Base URL: {}", api_base_url);
        log::info!("  Model: {}", model_name);
        log::info!("  API Key: {}", if api_key.is_some() { "provided" } else { "not provided" });

        // Validate base URL format
        if !api_base_url.starts_with("http://") && !api_base_url.starts_with("https://") {
            return Err(format!(
                "Invalid API Base URL: '{}'. Must start with http:// or https://",
                api_base_url
            ));
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let system_prompt = system_prompt.unwrap_or_else(|| {
            "You are Aura, a helpful AI assistant. Provide concise, accurate, and friendly responses."
                .to_string()
        });

        log::info!("LLM engine initialized successfully");

        Ok(LLMEngine {
            client,
            api_base_url,
            model_name,
            api_key,
            system_prompt,
            current_task: Arc::new(TokioMutex::new(None)),
        })
    }

    /// Generate a response to a user prompt
    ///
    /// This method sends a request to the configured OpenAI-compatible API
    /// and returns the generated response.
    ///
    /// The generation can be immediately cancelled by calling `cancel_generation()`,
    /// which will abort the HTTP request and return control to the caller.
    pub async fn generate_response(&self, user_prompt: &str) -> Result<String, String> {
        log::info!("Generating response for prompt: '{}'", user_prompt);

        // Construct messages array with system prompt and user message
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: self.system_prompt.clone(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ];

        // Build the request
        let request = ChatCompletionRequest {
            model: self.model_name.clone(),
            messages,
            max_tokens: Some(DEFAULT_MAX_TOKENS),
            temperature: Some(DEFAULT_TEMPERATURE),
            stream: false, // Non-streaming for simplicity
        };

        // Construct the full endpoint URL
        let endpoint = format!("{}/chat/completions", self.api_base_url.trim_end_matches('/'));

        log::info!("Sending request to: {}", endpoint);
        log::debug!("Request payload: {:?}", request);

        // Clone data needed for the spawned task
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let endpoint_clone = endpoint.clone();

        // Spawn the HTTP request in an abortable task
        let task_handle = tokio::spawn(async move {
            // Build HTTP request
            let mut http_request = client.post(&endpoint_clone).json(&request);

            // Add API key to headers if provided
            if let Some(ref api_key) = api_key {
                http_request = http_request.header("Authorization", format!("Bearer {}", api_key));
            }

            // Send request
            let response = http_request
                .send()
                .await
                .map_err(|e| {
                    format!(
                        "Failed to connect to LLM API at {}: {}. Make sure your AI server is running.",
                        endpoint_clone, e
                    )
                })?;

            // Check for HTTP errors
            if !response.status().is_success() {
                let status = response.status();
                let error_body = response.text().await.unwrap_or_else(|_| "Unable to read error body".to_string());
                return Err(format!(
                    "LLM API returned error {}: {}",
                    status, error_body
                ));
            }

            // Parse response
            let completion: ChatCompletionResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse API response: {}. Make sure the server is OpenAI-compatible.", e))?;

            // Extract the assistant's message
            let assistant_message = completion
                .choices
                .first()
                .ok_or("No response choices returned from API")?
                .message
                .content
                .clone();

            Ok(assistant_message)
        });

        // Store the abort handle so cancel_generation can abort this task
        {
            let mut current_task = self.current_task.lock().await;
            *current_task = Some(task_handle.abort_handle());
        }

        // Wait for the task to complete (or be aborted)
        let result = match task_handle.await {
            Ok(response_result) => {
                // Task completed successfully, return the response (or error from HTTP/parsing)
                response_result
            }
            Err(join_error) if join_error.is_cancelled() => {
                // Task was aborted via cancel_generation
                log::info!("Generation cancelled by user");
                Err("Generation cancelled by user".to_string())
            }
            Err(join_error) => {
                // Task panicked (shouldn't happen, but handle it)
                log::error!("Generation task panicked: {}", join_error);
                Err(format!("Generation task panicked: {}", join_error))
            }
        };

        // Clear the abort handle now that the task is done
        {
            let mut current_task = self.current_task.lock().await;
            *current_task = None;
        }

        // Log result and return
        match &result {
            Ok(message) => log::info!("Response received: {} characters", message.len()),
            Err(e) => log::warn!("Generation failed: {}", e),
        }

        result
    }

    /// Cancel the current generation immediately
    ///
    /// This aborts the HTTP request task, causing generate_response to return
    /// with a cancellation error. The UI will immediately return to idle state.
    pub async fn cancel_generation(&self) {
        log::info!("Cancellation requested - aborting current generation task");

        let mut current_task = self.current_task.lock().await;
        if let Some(abort_handle) = current_task.take() {
            abort_handle.abort();
            log::info!("✓ Generation task aborted successfully");
        } else {
            log::warn!("No active generation task to cancel");
        }
    }

    /// Get information about the configured LLM
    pub fn model_info(&self) -> ModelInfo {
        ModelInfo {
            api_base_url: self.api_base_url.clone(),
            model_name: self.model_name.clone(),
            system_prompt: self.system_prompt.clone(),
        }
    }

    /// Update the model configuration
    pub fn update_config(
        &mut self,
        api_base_url: String,
        model_name: String,
        api_key: Option<String>,
    ) {
        log::info!("Updating LLM configuration");
        log::info!("  New API Base URL: {}", api_base_url);
        log::info!("  New Model: {}", model_name);

        self.api_base_url = api_base_url;
        self.model_name = model_name;
        self.api_key = api_key;
    }
}

/// Model information
pub struct ModelInfo {
    pub api_base_url: String,
    pub model_name: String,
    pub system_prompt: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_engine_creation() {
        let engine = LLMEngine::new(
            "http://localhost:1234/v1".to_string(),
            "llama3".to_string(),
            None,
            None,
        );
        assert!(engine.is_ok());
    }

    #[test]
    fn test_invalid_url() {
        let engine = LLMEngine::new(
            "invalid-url".to_string(),
            "llama3".to_string(),
            None,
            None,
        );
        assert!(engine.is_err());
    }
}
