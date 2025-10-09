mod native_voice;
mod tts;
mod llm;
mod database;
mod secrets;
mod error;
mod ollama_sidecar;

use native_voice::NativeVoicePipeline;
use tts::TextToSpeech;
use llm::LLMEngine;
use ollama_sidecar::OllamaSidecar;
use database::{Database, DatabaseState, Conversation, Message, Settings, get_database_path};
use error::AuraError;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use std::sync::Mutex as StdMutex;
use tauri::{Manager, State, Emitter};
use serde::Serialize;

/// System status payload for frontend (service health check)
#[derive(Serialize, Clone, Debug)]
struct SystemStatus {
    stt_connected: bool,
    llm_connected: bool,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn handle_user_prompt(prompt: String, llm_engine: State<'_, Arc<TokioMutex<LLMEngine>>>) -> Result<String, AuraError> {
    log::info!("Tauri command: handle_user_prompt called with: '{}'", prompt);

    let llm = llm_engine.inner().lock().await;
    let result = llm.generate_response(&prompt).await
        .map_err(|e| AuraError::Llm(e))?;

    Ok(result)
}

#[tauri::command]
async fn listen_and_transcribe(voice_pipeline: State<'_, Arc<StdMutex<NativeVoicePipeline>>>) -> Result<String, AuraError> {
    log::info!("Tauri command: listen_and_transcribe called (Push-to-Talk)");

    // Use spawn_blocking because NativeVoicePipeline uses std::sync::Mutex internally
    // (required for audio thread compatibility)
    let voice_pipeline_clone = voice_pipeline.inner().clone();

    let result = tokio::task::spawn_blocking(move || {
        let pipeline = voice_pipeline_clone.lock()
            .map_err(|e| AuraError::Internal(format!("Failed to lock voice pipeline: {}", e)))?;

        pipeline.start_transcription()
            .map_err(|e| AuraError::VoicePipeline(e))
    }).await
    .map_err(|e| AuraError::Internal(format!("Task panicked: {}", e)))??;

    Ok(result)
}

#[tauri::command]
async fn speak_text(text: String, tts_engine: State<'_, Arc<TokioMutex<TextToSpeech>>>) -> Result<(), AuraError> {
    log::info!("Tauri command: speak_text called ({} chars)", text.len());
    log::info!("Text to speak: '{}'", if text.len() > 100 { format!("{}...", &text[..100]) } else { text.clone() });

    let mut tts = tts_engine.inner().lock().await;

    log::info!("TTS engine locked, calling speak()...");
    let result = tts.speak(&text)
        .map_err(|e| AuraError::Tts(e));

    match &result {
        Ok(_) => log::info!("✓ TTS speak() completed successfully"),
        Err(e) => log::error!("✗ TTS speak() failed: {}", e),
    }

    result
}

#[tauri::command]
async fn cancel_generation(llm_engine: State<'_, Arc<TokioMutex<LLMEngine>>>) -> Result<(), AuraError> {
    log::info!("Tauri command: cancel_generation called");

    let llm = llm_engine.inner().lock().await;
    llm.cancel_generation().await;

    Ok(())
}

#[tauri::command]
async fn cancel_recording(voice_pipeline: State<'_, Arc<StdMutex<NativeVoicePipeline>>>) -> Result<(), AuraError> {
    log::info!("Tauri command: cancel_recording called");

    let voice_pipeline_clone = voice_pipeline.inner().clone();

    tokio::task::spawn_blocking(move || {
        let pipeline = voice_pipeline_clone.lock()
            .map_err(|e| AuraError::Internal(format!("Failed to lock voice pipeline: {}", e)))?;

        // Use the new cancel_and_reset method for clean state reset
        pipeline.cancel_and_reset()
            .map_err(|e| AuraError::VoicePipeline(e))
    }).await
    .map_err(|e| AuraError::Internal(format!("Task panicked: {}", e)))??;

    Ok(())
}

// Database Commands

#[tauri::command]
async fn load_conversations(db: State<'_, DatabaseState>) -> Result<Vec<Conversation>, AuraError> {
    log::info!("Tauri command: load_conversations called");

    let db = db.inner().lock().await;
    db.load_conversations()
        .map_err(|e| AuraError::Database(e))
}

#[tauri::command]
async fn load_messages(conversation_id: i64, db: State<'_, DatabaseState>) -> Result<Vec<Message>, AuraError> {
    log::info!("Tauri command: load_messages called for conversation {}", conversation_id);

    let db = db.inner().lock().await;
    db.load_messages(conversation_id)
        .map_err(|e| AuraError::Database(e))
}

#[tauri::command]
async fn create_new_conversation(db: State<'_, DatabaseState>) -> Result<i64, AuraError> {
    log::info!("Tauri command: create_new_conversation called");

    let db = db.inner().lock().await;
    db.create_conversation(None)
        .map_err(|e| AuraError::Database(e))
}

#[tauri::command]
async fn save_message(
    conversation_id: i64,
    role: String,
    content: String,
    db: State<'_, DatabaseState>
) -> Result<i64, AuraError> {
    log::debug!("Tauri command: save_message called (conversation: {}, role: {})", conversation_id, role);

    let db = db.inner().lock().await;
    db.save_message(conversation_id, &role, &content)
        .map_err(|e| AuraError::Database(e))
}

#[tauri::command]
async fn delete_conversation(conversation_id: i64, db: State<'_, DatabaseState>) -> Result<(), AuraError> {
    log::info!("Tauri command: delete_conversation called for conversation {}", conversation_id);

    let db = db.inner().lock().await;
    db.delete_conversation(conversation_id)
        .map_err(|e| AuraError::Database(e))
}

#[tauri::command]
async fn update_conversation_title(
    conversation_id: i64,
    title: String,
    db: State<'_, DatabaseState>
) -> Result<(), AuraError> {
    log::info!("Tauri command: update_conversation_title called for conversation {} with title: {}", conversation_id, title);

    let db = db.inner().lock().await;
    db.update_conversation_title(conversation_id, &title)
        .map_err(|e| AuraError::Database(e))
}

#[tauri::command]
async fn generate_conversation_title(
    prompt: String,
    llm_engine: State<'_, Arc<TokioMutex<LLMEngine>>>
) -> Result<String, AuraError> {
    log::info!("Tauri command: generate_conversation_title called");

    let title_prompt = format!(
        "Create a very short title (maximum 5 words, no quotes) for a conversation starting with: '{}'",
        prompt.chars().take(150).collect::<String>()
    );

    let llm = llm_engine.inner().lock().await;
    let raw_title = llm.generate_response(&title_prompt).await
        .map_err(|e| AuraError::Llm(e))?;

    // Clean up the title (remove quotes, trim, limit length)
    let clean_title = raw_title
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .split('\n')
        .next()
        .unwrap_or(&raw_title)
        .chars()
        .take(60)
        .collect::<String>()
        .trim()
        .to_string();

    log::info!("Generated title: '{}'", clean_title);

    Ok(clean_title)
}

// Settings Commands

#[tauri::command]
async fn load_settings(db: State<'_, DatabaseState>) -> Result<Settings, AuraError> {
    log::info!("Tauri command: load_settings called");

    let db = db.inner().lock().await;

    db.load_settings()
        .map_err(|e| AuraError::Database(e))
}

#[tauri::command]
async fn save_settings(
    llm_provider: String,
    server_address: String,
    wake_word_enabled: bool,
    api_base_url: String,
    model_name: String,
    vad_sensitivity: f32,
    vad_timeout_ms: u32,
    stt_model_name: String,
    voice_preference: String,
    db: State<'_, DatabaseState>
) -> Result<(), AuraError> {
    log::info!("Tauri command: save_settings called (provider: {}, server: {}, wake_word: {}, api_base_url: {}, model: {}, vad_sensitivity: {}, vad_timeout_ms: {}, stt_model: {}, voice: {})",
               llm_provider, server_address, wake_word_enabled, api_base_url, model_name, vad_sensitivity, vad_timeout_ms, stt_model_name, voice_preference);

    let db = db.inner().lock().await;

    let settings = Settings {
        llm_provider,
        server_address,
        wake_word_enabled,
        api_base_url,
        model_name,
        vad_sensitivity,
        vad_timeout_ms,
        stt_model_name,
        voice_preference,
    };

    db.save_settings(&settings)
        .map_err(|e| AuraError::Database(e))
}

#[tauri::command]
async fn save_api_key(api_key: String) -> Result<(), AuraError> {
    log::info!("Tauri command: save_api_key called");

    secrets::save_api_key(&api_key)
        .map_err(|e| AuraError::Secrets(e))
}

#[tauri::command]
async fn load_api_key() -> Result<String, AuraError> {
    log::info!("Tauri command: load_api_key called");

    secrets::load_api_key()
        .map_err(|e| AuraError::Secrets(e))
}

#[tauri::command]
async fn update_vad_settings(
    sensitivity: f32,
    timeout_ms: u32,
    voice_pipeline: State<'_, Arc<StdMutex<NativeVoicePipeline>>>
) -> Result<(), AuraError> {
    log::info!("Tauri command: update_vad_settings called (sensitivity: {}, timeout_ms: {})", sensitivity, timeout_ms);

    let voice_pipeline_clone = voice_pipeline.inner().clone();

    tokio::task::spawn_blocking(move || {
        let pipeline = voice_pipeline_clone.lock()
            .map_err(|e| AuraError::Internal(format!("Failed to lock voice pipeline: {}", e)))?;

        pipeline.update_vad_settings(sensitivity, timeout_ms)
            .map_err(|e| AuraError::VoicePipeline(e))
    }).await
    .map_err(|e| AuraError::Internal(format!("Task panicked: {}", e)))??;

    Ok(())
}

#[tauri::command]
async fn set_voice_state(
    state: String,
    voice_pipeline: State<'_, Arc<StdMutex<NativeVoicePipeline>>>
) -> Result<(), AuraError> {
    log::info!("Tauri command: set_voice_state called (state: {})", state);

    // Parse state string to VoiceState enum
    use native_voice::VoiceState;
    let voice_state = match state.as_str() {
        "idle" => VoiceState::Idle,
        "listening_for_wake_word" => VoiceState::ListeningForWakeWord,
        "transcribing" => VoiceState::Transcribing,
        "speaking" => VoiceState::Speaking,
        _ => return Err(AuraError::Config(format!("Invalid voice state: {}. Must be one of: idle, listening_for_wake_word, transcribing, speaking", state))),
    };

    let voice_pipeline_clone = voice_pipeline.inner().clone();

    tokio::task::spawn_blocking(move || {
        let pipeline = voice_pipeline_clone.lock()
            .map_err(|e| AuraError::Internal(format!("Failed to lock voice pipeline: {}", e)))?;

        pipeline.set_state(voice_state)
            .map_err(|e| AuraError::VoicePipeline(e))
    }).await
    .map_err(|e| AuraError::Internal(format!("Task panicked: {}", e)))??;

    Ok(())
}

/// Check if the configured STT model is present
fn check_stt_model_ready(model_path: &std::path::Path, stt_model_name: &str) -> bool {
    let stt_model = model_path.join(stt_model_name);
    let exists = stt_model.exists();

    log::trace!(
        "STT model check: {} exists={}",
        stt_model_name,
        exists
    );

    exists
}

/// Check if an HTTP-based service is reachable and responding (used for LLM/Ollama)
async fn check_http_service(base_url: &str) -> bool {
    let timeout = std::time::Duration::from_secs(2);

    // Extract just the base URL without the /v1 path for health check
    let health_check_url = if base_url.ends_with("/v1") {
        base_url.trim_end_matches("/v1").to_string()
    } else {
        base_url.to_string()
    };

    // Create a simple HTTP client
    let client = match reqwest::Client::builder()
        .timeout(timeout)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::trace!("Failed to create HTTP client for service check: {}", e);
            return false;
        }
    };

    // Try to GET the base URL
    match client.get(&health_check_url).send().await {
        Ok(response) => {
            let is_ok = response.status().is_success() || response.status().is_client_error();
            // Accept any response (even 404) as long as the server is responding
            // This is because some servers might return 404 on root but still be operational
            log::trace!(
                "HTTP service {} responded with status: {} (connected: {})",
                health_check_url,
                response.status(),
                is_ok
            );
            is_ok
        }
        Err(e) => {
            log::trace!("HTTP service {} connection failed: {}", health_check_url, e);
            false
        }
    }
}

/// Reload the voice pipeline with new settings (enables live model switching without restart)
///
/// This command safely shuts down the current voice pipeline and starts a new one
/// with the updated settings. This allows users to change STT models, VAD settings,
/// and wake word configuration without restarting the entire application.
#[tauri::command]
async fn reload_voice_pipeline(
    app_handle: tauri::AppHandle,
    llm_provider: String,
    server_address: String,
    wake_word_enabled: bool,
    api_base_url: String,
    model_name: String,
    vad_sensitivity: f32,
    vad_timeout_ms: u32,
    stt_model_name: String,
    voice_pipeline: State<'_, Arc<StdMutex<NativeVoicePipeline>>>,
    database: State<'_, DatabaseState>,
) -> Result<(), AuraError> {
    log::info!("Reloading voice pipeline with new settings...");
    log::info!("  STT model: {}", stt_model_name);
    log::info!("  VAD sensitivity: {}", vad_sensitivity);
    log::info!("  VAD timeout: {}ms", vad_timeout_ms);
    log::info!("  Wake word enabled: {}", wake_word_enabled);

    // Save settings to database first
    {
        let db = database.lock().await;

        // Load existing voice preference (this command doesn't modify it)
        let existing_voice_preference = db.load_settings()
            .ok()
            .map(|s| s.voice_preference)
            .unwrap_or_else(|| "male".to_string());

        let settings_to_save = Settings {
            llm_provider: llm_provider.clone(),
            server_address: server_address.clone(),
            wake_word_enabled,
            api_base_url: api_base_url.clone(),
            model_name: model_name.clone(),
            vad_sensitivity,
            vad_timeout_ms,
            stt_model_name: stt_model_name.clone(),
            voice_preference: existing_voice_preference,
        };

        db.save_settings(&settings_to_save)
            .map_err(|e| AuraError::Database(e))?;
    }

    // Determine model path
    let model_path = dirs::data_local_dir()
        .map(|p| p.join("nivora-aura").join("models"))
        .unwrap_or_else(|| std::path::PathBuf::from("./models"));

    let voice_pipeline_clone = voice_pipeline.inner().clone();

    // Stop the old pipeline and create a new one using spawn_blocking
    let new_pipeline = tokio::task::spawn_blocking(move || {
        // First, stop the old pipeline
        let old_pipeline = voice_pipeline_clone.lock()
            .map_err(|e| AuraError::Internal(format!("Failed to lock voice pipeline: {}", e)))?;

        log::info!("Stopping current voice pipeline...");
        old_pipeline.stop();

        // Drop the old pipeline before creating the new one
        drop(old_pipeline);

        // Create new pipeline with updated settings
        log::info!("Creating new voice pipeline...");
        let pipeline = NativeVoicePipeline::new(
            app_handle.clone(),
            model_path.clone(),
            stt_model_name.clone(),
            vad_sensitivity,
            vad_timeout_ms,
        )
        .map_err(|e| AuraError::VoicePipeline(e))?;

        // Start the new pipeline
        log::info!("Starting new voice pipeline...");
        pipeline.start()
            .map_err(|e| AuraError::VoicePipeline(e))?;

        Ok::<_, AuraError>(pipeline)
    }).await
    .map_err(|e| AuraError::Internal(format!("Task panicked: {}", e)))??;

    // Replace the old pipeline with the new one
    let voice_pipeline_clone2 = voice_pipeline.inner().clone();
    tokio::task::spawn_blocking(move || {
        let mut pipeline = voice_pipeline_clone2.lock()
            .map_err(|e| AuraError::Internal(format!("Failed to lock voice pipeline: {}", e)))?;
        *pipeline = new_pipeline;
        Ok::<_, AuraError>(())
    }).await
    .map_err(|e| AuraError::Internal(format!("Task panicked: {}", e)))??;

    log::info!("✓ Voice pipeline reloaded successfully");
    Ok(())
}

// =============================================================================
// FIRST-RUN WIZARD COMMANDS
// =============================================================================

/// Status of a dependency for the first-run wizard
#[derive(Debug, Clone, serde::Serialize)]
pub struct SetupStatus {
    pub first_run_complete: bool,
    pub whisper_model_exists: bool,
    pub whisper_model_path: String,
}

/// Check the status of all required dependencies
#[tauri::command]
async fn check_setup_status(
    database: State<'_, DatabaseState>,
) -> Result<SetupStatus, AuraError> {
    log::info!("Checking setup status for first-run wizard");

    // Check if first-run wizard has been completed
    let first_run_complete = {
        let db = database.lock().await;
        db.is_first_run_complete()
            .map_err(|e| AuraError::Database(e))?
    };

    // Check Whisper model existence
    let model_path = dirs::data_local_dir()
        .map(|p| p.join("nivora-aura").join("models"))
        .unwrap_or_else(|| std::path::PathBuf::from("./models"));

    let whisper_model_path = model_path.join("ggml-tiny.bin");
    let whisper_model_exists = whisper_model_path.exists();

    let status = SetupStatus {
        first_run_complete,
        whisper_model_exists,
        whisper_model_path: whisper_model_path.to_string_lossy().to_string(),
    };

    log::info!("Setup status: first_run={}, whisper={}",
               status.first_run_complete, status.whisper_model_exists);

    Ok(status)
}

/// Download progress event
#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    percentage: f32,
}

/// Download the Whisper tiny model from HuggingFace
#[tauri::command]
async fn download_whisper_model(
    app_handle: tauri::AppHandle,
) -> Result<String, AuraError> {
    log::info!("Starting Whisper model download");

    // Determine download path
    let model_path = dirs::data_local_dir()
        .map(|p| p.join("nivora-aura").join("models"))
        .unwrap_or_else(|| std::path::PathBuf::from("./models"));

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&model_path)
        .map_err(|e| AuraError::Internal(format!("Failed to create models directory: {}", e)))?;

    let dest_path = model_path.join("ggml-tiny.bin");

    // URL for ggml-tiny.bin from HuggingFace
    let url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin";

    // Download with progress
    let client = reqwest::Client::new();
    let mut response = client.get(url).send().await
        .map_err(|e| AuraError::Internal(format!("Failed to start download: {}", e)))?;

    let total_size = response.content_length();

    let mut file = tokio::fs::File::create(&dest_path).await
        .map_err(|e| AuraError::Internal(format!("Failed to create file: {}", e)))?;

    let mut downloaded: u64 = 0;

    while let Some(chunk) = response.chunk().await
        .map_err(|e| AuraError::Internal(format!("Failed to download chunk: {}", e)))? {

        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await
            .map_err(|e| AuraError::Internal(format!("Failed to write chunk: {}", e)))?;

        downloaded += chunk.len() as u64;

        let percentage = if let Some(total) = total_size {
            (downloaded as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        // Emit progress event
        let progress = DownloadProgress {
            downloaded_bytes: downloaded,
            total_bytes: total_size,
            percentage,
        };

        app_handle.emit("download_progress", progress)
            .map_err(|e| AuraError::Internal(format!("Failed to emit progress: {}", e)))?;
    }

    log::info!("✓ Whisper model downloaded successfully to: {}", dest_path.display());

    Ok(dest_path.to_string_lossy().to_string())
}

/// Mark the first-run wizard as complete
#[tauri::command]
async fn mark_setup_complete(
    database: State<'_, DatabaseState>,
) -> Result<(), AuraError> {
    log::info!("Marking first-run setup as complete");

    let db = database.lock().await;
    db.mark_first_run_complete()
        .map_err(|e| AuraError::Database(e))?;

    log::info!("✓ First-run setup marked complete");

    Ok(())
}

/// Fetch list of available Ollama models from the running server
#[tauri::command]
async fn fetch_available_models(db: State<'_, DatabaseState>) -> Result<Vec<String>, AuraError> {
    log::info!("Tauri command: fetch_available_models called");

    // Load settings to get API base URL
    let settings = {
        let db = db.inner().lock().await;
        db.load_settings()
            .map_err(|e| AuraError::Database(e))?
    };

    let api_url = format!("{}/tags", settings.api_base_url);
    log::info!("Fetching models from: {}", api_url);

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| AuraError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    // Fetch models list
    let response = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| AuraError::Internal(format!("Failed to fetch models: {}", e)))?;

    if !response.status().is_success() {
        return Err(AuraError::Internal(format!(
            "Ollama API returned error status: {}",
            response.status()
        )));
    }

    // Parse response
    #[derive(serde::Deserialize)]
    struct TagsResponse {
        models: Vec<ModelInfo>,
    }

    #[derive(serde::Deserialize)]
    struct ModelInfo {
        name: String,
    }

    let tags_response: TagsResponse = response
        .json()
        .await
        .map_err(|e| AuraError::Internal(format!("Failed to parse models response: {}", e)))?;

    // Extract model names
    let mut model_names: Vec<String> = tags_response
        .models
        .into_iter()
        .map(|m| m.name)
        .collect();

    // Always include bundled gemma:2b if not already in list
    if !model_names.contains(&"gemma:2b".to_string()) {
        model_names.push("gemma:2b".to_string());
        log::info!("Added bundled gemma:2b to model list");
    }

    // Sort alphabetically for better UX
    model_names.sort();

    log::info!("✓ Found {} available models", model_names.len());

    Ok(model_names)
}

/// Wait for Ollama server to become ready
///
/// Polls the Ollama API until it responds or times out
async fn wait_for_ollama_ready(host: &str, timeout_secs: u64) -> Result<(), String> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);
    let api_url = format!("http://{}/api/tags", host);

    log::info!("Waiting for Ollama API at: {}", api_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    loop {
        if start.elapsed() > timeout {
            return Err(format!(
                "Ollama server did not become ready within {} seconds",
                timeout_secs
            ));
        }

        match client.get(&api_url).send().await {
            Ok(response) if response.status().is_success() => {
                log::info!("✓ Ollama server ready (took {:.1}s)", start.elapsed().as_secs_f32());
                return Ok(());
            }
            Ok(response) => {
                log::debug!("Ollama API returned: {}", response.status());
            }
            Err(e) => {
                log::debug!("Ollama not ready yet: {}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    env_logger::init();

    log::info!("=== Aura Desktop Initialization ===");

    // Initialize database
    log::info!("Initializing database...");
    let database = match get_database_path() {
        Ok(db_path) => {
            match Database::new(db_path) {
                Ok(db) => {
                    let conv_count = db.count_conversations().unwrap_or(0);
                    let msg_count = db.count_messages().unwrap_or(0);
                    log::info!("✓ Database initialized successfully");
                    log::info!("  - {} conversations, {} messages", conv_count, msg_count);
                    Arc::new(TokioMutex::new(db))
                }
                Err(e) => {
                    log::error!("✗ Failed to initialize database: {}", e);
                    panic!("Cannot start without database. Error: {}", e);
                }
            }
        }
        Err(e) => {
            log::error!("✗ Failed to get database path: {}", e);
            panic!("Cannot start without database. Error: {}", e);
        }
    };

    // Initialize LLM engine (OpenAI-compatible API client)
    log::info!("Loading LLM engine configuration...");

    // Load settings to get API configuration
    let db_for_llm = database.blocking_lock();
    let settings = db_for_llm.load_settings().unwrap_or_else(|e| {
        log::warn!("Failed to load settings, using defaults: {}", e);
        Settings {
            llm_provider: "local".to_string(),
            server_address: "".to_string(),
            wake_word_enabled: false,
            api_base_url: "http://localhost:11434/v1".to_string(),
            model_name: "gemma:2b".to_string(),
            vad_sensitivity: 0.02,
            vad_timeout_ms: 1280,
            stt_model_name: "ggml-base.en.bin".to_string(),
            voice_preference: "male".to_string(),
        }
    });
    drop(db_for_llm); // Release the lock

    // Load API key from keyring (optional)
    let api_key = secrets::load_api_key().ok();

    let llm_engine = match LLMEngine::new(
        settings.api_base_url.clone(),
        settings.model_name.clone(),
        api_key.clone(),
        None
    ) {
        Ok(llm) => {
            log::info!("✓ LLM engine initialized successfully");
            let info = llm.model_info();
            log::info!("  - API Base URL: {}", info.api_base_url);
            log::info!("  - Model: {}", info.model_name);
            log::info!("  - System prompt: {}", info.system_prompt);
            log::info!("  - API Key: {}", if api_key.is_some() { "provided" } else { "not provided" });
            Arc::new(TokioMutex::new(llm))
        }
        Err(e) => {
            log::error!("✗ Failed to initialize LLM engine: {}", e);
            log::error!("  Make sure you have configured the API Base URL and Model Name in settings");
            log::error!("  Example: Ollama at http://localhost:11434/v1 with model 'llama3'");
            panic!("Cannot start without LLM configuration. Error: {}", e);
        }
    };

    // TTS will be initialized in setup closure after app handle is available

    log::info!("=== Starting Tauri Application ===");

    // Clone database for use in setup closure
    let database_for_setup = database.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(database.clone())
        .manage(llm_engine)
        .invoke_handler(tauri::generate_handler![
            greet,
            handle_user_prompt,
            listen_and_transcribe,
            cancel_recording,
            speak_text,
            cancel_generation,
            load_conversations,
            load_messages,
            create_new_conversation,
            save_message,
            delete_conversation,
            update_conversation_title,
            generate_conversation_title,
            load_settings,
            save_settings,
            save_api_key,
            load_api_key,
            update_vad_settings,
            set_voice_state,
            reload_voice_pipeline,
            check_setup_status,
            download_whisper_model,
            mark_setup_complete,
            fetch_available_models
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Determine model path
            let model_path = dirs::data_local_dir()
                .map(|p| p.join("nivora-aura").join("models"))
                .unwrap_or_else(|| std::path::PathBuf::from("./models"));

            // Create model directory if it doesn't exist
            if let Err(e) = std::fs::create_dir_all(&model_path) {
                log::warn!("Failed to create model directory: {}", e);
            }

            log::info!("Model directory: {:?}", model_path);

            // Initialize Ollama sidecar process (bundled LLM server)
            log::info!("Initializing Ollama sidecar...");

            // Try bundled Ollama first (production mode)
            let resource_dir_for_ollama = app_handle.path().resource_dir()
                .map_err(|e| format!("Failed to get resource directory: {}", e))
                .unwrap();

            // Determine platform-specific Ollama binary name
            let ollama_binary_name = if cfg!(target_os = "windows") {
                "ollama-windows-amd64.exe"
            } else if cfg!(target_os = "macos") {
                if cfg!(target_arch = "aarch64") {
                    "ollama-darwin-arm64"
                } else {
                    "ollama-darwin-amd64"
                }
            } else {
                "ollama-linux-amd64"
            };

            let bundled_ollama_binary = resource_dir_for_ollama.join("ollama").join("bin").join(ollama_binary_name);
            let bundled_ollama_models = resource_dir_for_ollama.join("ollama").join("models");

            // Check if bundled Ollama exists (production build)
            let use_bundled_ollama = bundled_ollama_binary.exists();

            let (ollama_binary, ollama_models) = if use_bundled_ollama {
                log::info!("Using bundled Ollama (production mode)");
                (bundled_ollama_binary, bundled_ollama_models)
            } else {
                log::warn!("Bundled Ollama not found, falling back to system Ollama (dev mode)");

                // Fallback to system-installed Ollama for development
                let system_ollama = which::which("ollama")
                    .map_err(|e| e.to_string())
                    .unwrap_or_else(|_| std::path::PathBuf::from("/usr/local/bin/ollama"));

                let system_models = dirs::home_dir()
                    .map(|h| h.join(".ollama").join("models"))
                    .unwrap_or_else(|| std::path::PathBuf::from("./.ollama/models"));

                (system_ollama, system_models)
            };

            let ollama_host = "127.0.0.1:11434".to_string();

            let mut ollama_sidecar = match OllamaSidecar::new(
                ollama_binary.clone(),
                ollama_models.clone(),
                ollama_host.clone(),
            ) {
                Ok(sidecar) => {
                    log::info!("✓ Ollama sidecar manager created");
                    sidecar
                }
                Err(e) => {
                    if use_bundled_ollama {
                        log::error!("✗ Failed to create Ollama sidecar: {}", e);
                        panic!("Cannot start bundled Ollama. Error: {}", e);
                    } else {
                        log::warn!("✗ Failed to create Ollama sidecar: {}", e);
                        log::warn!("  Will rely on external Ollama server");
                        log::warn!("  Make sure Ollama is installed and running: ollama serve");
                        // Create a dummy sidecar that won't be started
                        OllamaSidecar::new(
                            std::path::PathBuf::from("dummy"),
                            std::path::PathBuf::from("dummy"),
                            ollama_host.clone(),
                        ).unwrap()
                    }
                }
            };

            // Start the Ollama server if we have bundled resources
            if use_bundled_ollama {
                match ollama_sidecar.start() {
                    Ok(()) => {
                        log::info!("Ollama server starting...");

                        // Wait for readiness in background (non-blocking)
                        let ollama_host_clone = ollama_host.clone();
                        tokio::spawn(async move {
                            // Give it 30 seconds to start
                            match wait_for_ollama_ready(&ollama_host_clone, 30).await {
                                Ok(()) => {
                                    log::info!("✓ Ollama server is ready!");
                                }
                                Err(e) => {
                                    log::error!("✗ Ollama server failed to become ready: {}", e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("✗ Failed to start Ollama server: {}", e);
                        panic!("Cannot start Ollama sidecar. Error: {}", e);
                    }
                }
            } else {
                log::info!("Skipping Ollama sidecar start (using external Ollama)");
            }

            // Register Ollama sidecar as managed state for shutdown
            app.manage(Arc::new(StdMutex::new(ollama_sidecar)));

            // Load settings again for voice pipeline configuration
            let vad_settings = {
                let db = database_for_setup.blocking_lock();
                db.load_settings().ok()
            };

            let vad_sensitivity = vad_settings.as_ref().map(|s| s.vad_sensitivity).unwrap_or(0.02);
            let vad_timeout_ms = vad_settings.as_ref().map(|s| s.vad_timeout_ms).unwrap_or(1280);
            let stt_model_name = vad_settings.as_ref().map(|s| s.stt_model_name.clone()).unwrap_or_else(|| "ggml-tiny.bin".to_string());
            let voice_preference = vad_settings.as_ref().map(|s| s.voice_preference.clone()).unwrap_or_else(|| "male".to_string());

            // Initialize Subprocess-based Piper TTS engine with bundled resources
            log::info!("Initializing subprocess-based Piper TTS engine...");

            // Try bundled resources first (production mode)
            let resource_dir = app_handle.path().resource_dir()
                .map_err(|e| format!("Failed to get resource directory: {}", e))
                .unwrap();

            // Determine platform-specific Piper binary name
            let piper_binary_name = if cfg!(target_os = "windows") {
                "piper-windows-x86_64.exe"
            } else if cfg!(target_os = "macos") {
                if cfg!(target_arch = "aarch64") {
                    "piper-macos-arm64"
                } else {
                    "piper-macos-x86_64"
                }
            } else {
                "piper-linux-x86_64"
            };

            let bundled_piper_binary = resource_dir.join("piper").join("bin").join(piper_binary_name);

            // Determine voice model based on user preference
            let voice_model_file = if voice_preference == "female" {
                "en_US-amy-medium.onnx"
            } else {
                "en_US-lessac-medium.onnx"
            };

            let bundled_voice_model = resource_dir.join("piper").join("voices").join(voice_model_file);
            let bundled_espeak_data = resource_dir.join("piper").join("espeak-ng-data");

            // Check if bundled resources exist (production build)
            let use_bundled = bundled_piper_binary.exists() && bundled_voice_model.exists() && bundled_espeak_data.exists();

            let (piper_binary, voice_model_path, espeak_data_path) = if use_bundled {
                log::info!("Using bundled Piper resources (production mode)");
                (bundled_piper_binary, bundled_voice_model, bundled_espeak_data)
            } else {
                log::warn!("Bundled resources not detected via resource_dir, checking source tree for dev mode...");

                // In dev mode, try to use resources from source tree
                let dev_resources_root = std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                    .and_then(|exe_dir| {
                        exe_dir
                            .parent()
                            .and_then(|target| target.parent())
                            .and_then(|src_tauri| src_tauri.parent())
                            .map(|root| root.join("resources"))
                    })
                    .unwrap_or_else(|| std::path::PathBuf::from("./resources"));

                let dev_piper_binary = dev_resources_root.join("piper").join("bin").join(piper_binary_name);
                let dev_voice_model = dev_resources_root.join("piper").join("voices").join(voice_model_file);
                let dev_espeak_data = dev_resources_root.join("piper").join("espeak-ng-data");

                // Check if dev resources exist
                if dev_piper_binary.exists() && dev_voice_model.exists() && dev_espeak_data.exists() {
                    log::info!("Using Piper resources from source tree (dev mode)");
                    (dev_piper_binary, dev_voice_model, dev_espeak_data)
                } else {
                    log::warn!("Dev resources not found, falling back to system Piper (may require installation)");

                    // Last resort: system-installed Piper
                    let system_piper = std::env::var("PIPER_BINARY")
                        .map(std::path::PathBuf::from)
                        .or_else(|_| which::which("piper").map_err(|e| e.to_string()))
                        .unwrap_or_else(|_| std::path::PathBuf::from("/usr/local/bin/piper"));

                    let system_voice = dirs::data_local_dir()
                        .map(|p| p.join("nivora-aura").join("voices").join(voice_model_file))
                        .unwrap_or_else(|| std::path::PathBuf::from("./voices").join(voice_model_file));

                    let system_espeak = std::env::var("ESPEAK_DATA_PATH")
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|_| std::path::PathBuf::from("/usr/share/espeak-ng-data"));

                    (system_piper, system_voice, system_espeak)
                }
            };

            log::info!("Piper binary path: {:?}", piper_binary);
            log::info!("Voice model path: {:?} (preference: {})", voice_model_path, voice_preference);
            log::info!("eSpeak-NG data path: {:?}", espeak_data_path);

            let tts_engine = match TextToSpeech::new(piper_binary.clone(), voice_model_path.clone(), espeak_data_path.clone()) {
                Ok(tts) => {
                    log::info!("✓ Subprocess-based Piper TTS engine initialized successfully");
                    log::info!("  - Piper binary: {:?}", piper_binary);
                    log::info!("  - Voice model: {:?}", voice_model_path);
                    log::info!("  - Voice: {} ({})", voice_preference, voice_model_file);
                    log::info!("  - Mode: {}", if use_bundled { "bundled (production)" } else { "system (dev)" });
                    Arc::new(TokioMutex::new(tts))
                }
                Err(e) => {
                    log::error!("✗ Failed to initialize subprocess-based Piper TTS engine: {}", e);
                    if use_bundled {
                        log::error!("  Bundled resources may be corrupted");
                    } else {
                        log::error!("  Please install Piper TTS or ensure voice models are downloaded");
                    }
                    panic!("Cannot start TTS engine. Error: {}", e);
                }
            };

            // Register TTS engine as managed state
            app.manage(tts_engine.clone());

            // Initialize Native Voice Pipeline
            log::info!("Initializing native voice pipeline...");
            let voice_pipeline = match NativeVoicePipeline::new(
                app_handle.clone(),
                model_path.clone(),
                stt_model_name.clone(),
                vad_sensitivity,
                vad_timeout_ms,
            ) {
                Ok(pipeline) => {
                    log::info!("✓ Native voice pipeline initialized");
                    log::info!("  - Audio device: configured for 16kHz mono");
                    log::info!("  - Wake word: energy-based VAD");
                    log::info!("  - STT: whisper-rs (Whisper.cpp)");
                    log::info!("  - STT model: {}", stt_model_name);
                    log::info!("  - Model path: {:?}", model_path);
                    log::info!("  - VAD sensitivity: {}", vad_sensitivity);
                    log::info!("  - VAD timeout: {}ms", vad_timeout_ms);

                    // Check if models are present
                    if !pipeline.check_readiness() {
                        log::warn!("⚠ STT model not found!");
                        log::warn!("  Please download: {}", stt_model_name);
                        log::warn!("  Place it in: {:?}", model_path);
                    }

                    Arc::new(StdMutex::new(pipeline))
                }
                Err(e) => {
                    log::error!("✗ Failed to initialize voice pipeline: {}", e);
                    log::error!("  Make sure you have a microphone connected");
                    panic!("Cannot start voice pipeline. Error: {}", e);
                }
            };

            // Register voice pipeline as managed state
            app.manage(voice_pipeline.clone());

            // Start voice pipeline
            log::info!("Starting voice pipeline...");
            {
                let pipeline = voice_pipeline.lock().expect("Failed to lock voice pipeline");
                if let Err(e) = pipeline.start() {
                    log::error!("Failed to start voice pipeline: {}", e);
                } else {
                    log::info!("✓ Voice pipeline started");
                }
            }

            // Start background service status checker thread
            log::info!("Starting service status checker...");
            let app_handle_for_status = app_handle.clone();
            let database_for_status = database_for_setup.clone();
            let model_path_for_status = model_path.clone();
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();

                loop {
                    // Load current settings to get API base URL and STT model name
                    let (api_base_url, stt_model_name) = {
                        let db = database_for_status.blocking_lock();
                        if let Ok(settings) = db.load_settings() {
                            (settings.api_base_url, settings.stt_model_name)
                        } else {
                            ("http://localhost:11434/v1".to_string(), "ggml-base.en.bin".to_string())
                        }
                    };

                    // Check if configured STT model file exists on disk
                    let stt_connected = check_stt_model_ready(&model_path_for_status, &stt_model_name);

                    // Check LLM service connection (HTTP-based API)
                    let llm_connected = runtime.block_on(async {
                        check_http_service(&api_base_url).await
                    });

                    // Emit system status update
                    let status = SystemStatus {
                        stt_connected,
                        llm_connected,
                    };

                    if let Err(e) = app_handle_for_status.emit("system_status_update", status.clone()) {
                        log::error!("Failed to emit system_status_update: {}", e);
                    } else {
                        log::debug!("System status: STT={}, LLM={}", stt_connected, llm_connected);
                    }

                    // Wait 5 seconds before next check
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            });
            log::info!("✓ Service status checker started");

            log::info!("=== Aura Desktop Ready ===");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
