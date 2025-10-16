mod native_voice;
mod tts;
mod llm;
mod database;
mod secrets;
mod error;
mod ollama_sidecar;
mod web_search;
mod spotify_auth;
mod spotify_client;
mod music_intent;
mod entity_manager;
mod ha_client;
mod smarthome_intent;
mod voice_biometrics;

use native_voice::{NativeVoicePipeline, TranscriptionResult, SpeakerInfo};
use tts::TextToSpeech;
use llm::LLMEngine;
use ollama_sidecar::OllamaSidecar;
use database::{Database, DatabaseState, Conversation, Message, Settings, UserHAShortcut, UserHAPreferences, get_database_path};
use voice_biometrics::{VoiceBiometrics, UserProfile};
use error::AuraError;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use std::sync::Mutex as StdMutex;
use tauri::{Manager, State, Emitter};
use serde::Serialize;

// Type aliases for state management
type VoiceBiometricsState = Arc<VoiceBiometrics>;

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
async fn handle_user_prompt(
    prompt: String,
    llm_engine: State<'_, Arc<TokioMutex<LLMEngine>>>,
    db: State<'_, DatabaseState>,
) -> Result<String, AuraError> {
    log::info!("Tauri command: handle_user_prompt called with: '{}'", prompt);

    // Load settings to check if online mode is enabled
    let settings = {
        let database = db.lock().await;
        database.load_settings()
            .map_err(|e| AuraError::Internal(format!("Failed to load settings: {}", e)))?
    };

    // Determine final prompt (with or without RAG)
    let augmented_prompt = if settings.online_mode_enabled {
        log::info!("Online mode enabled, performing web search for RAG...");

        // Determine search backend from settings
        let search_backend = match settings.search_backend.as_str() {
            "searxng" => {
                web_search::SearchBackend::SearXNG {
                    instance_url: settings.searxng_instance_url.clone(),
                }
            }
            "brave" => {
                // Get Brave API key from settings
                let api_key = settings.brave_search_api_key
                    .clone()
                    .ok_or_else(|| AuraError::Internal(
                        "Brave Search selected but no API key configured. Please set it in Settings.".to_string()
                    ))?;

                web_search::SearchBackend::BraveSearch { api_key }
            }
            backend => {
                log::warn!("Unknown search backend '{}', defaulting to SearXNG", backend);
                web_search::SearchBackend::SearXNG {
                    instance_url: "https://searx.be".to_string(),
                }
            }
        };

        // Perform web search
        match web_search::search_web(
            &prompt,
            search_backend,
            settings.max_search_results as usize,
        ).await {
            Ok(results) if !results.is_empty() => {
                log::info!("✓ Web search successful: {} results found", results.len());

                // Format search results as context
                let context = web_search::format_search_context(&results);

                // Augment prompt with search context
                format!(
                    "{}\nUser Question: {}",
                    context,
                    prompt
                )
            }
            Ok(_) => {
                log::warn!("⚠ Web search returned 0 results, using offline mode");
                prompt.clone()
            }
            Err(e) => {
                log::warn!("⚠ Web search failed: {}, falling back to offline mode", e);
                prompt.clone()
            }
        }
    } else {
        log::debug!("Online mode disabled, using offline LLM query");
        prompt.clone()
    };

    // Query LLM with (possibly augmented) prompt
    let llm = llm_engine.inner().lock().await;
    let result = llm.generate_response(&augmented_prompt).await
        .map_err(|e| AuraError::Llm(e))?;

    Ok(result)
}

#[tauri::command]
async fn listen_and_transcribe(
    voice_pipeline: State<'_, Arc<StdMutex<NativeVoicePipeline>>>,
    voice_biometrics: State<'_, VoiceBiometricsState>,
) -> Result<TranscriptionResult, AuraError> {
    log::info!("Tauri command: listen_and_transcribe called (Push-to-Talk)");

    // Use spawn_blocking because NativeVoicePipeline uses std::sync::Mutex internally
    // (required for audio thread compatibility)
    let voice_pipeline_clone = voice_pipeline.inner().clone();

    let (transcription_text, audio_samples, audio_metadata) = tokio::task::spawn_blocking(move || {
        let pipeline = voice_pipeline_clone.lock()
            .map_err(|e| AuraError::Internal(format!("Failed to lock voice pipeline: {}", e)))?;

        // Perform transcription - now returns both text and audio samples
        let (text, samples) = pipeline.start_transcription()
            .map_err(|e| AuraError::VoicePipeline(e))?;

        // Calculate audio metadata from the samples
        let duration = samples.len() as f32 / 16000.0; // SAMPLE_RATE = 16000
        let avg_energy = if !samples.is_empty() {
            let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
            (sum_squares / samples.len() as f32).sqrt()
        } else {
            0.0
        };
        let metadata = (samples.len(), duration, avg_energy);

        Ok::<_, AuraError>((text, samples, metadata))
    }).await
    .map_err(|e| AuraError::Internal(format!("Task panicked: {}", e)))??;

    log::info!("Transcription completed: \"{}\"", transcription_text);

    // **AC1: Pipeline Hook** - Perform speaker identification asynchronously 
    // **AC3: Asynchronous Operation** - Non-blocking speaker ID with timeout
    let speaker_info = if voice_biometrics.is_model_loaded().await && !audio_samples.is_empty() {
        log::debug!("Performing speaker identification on {:.2}s of audio...", audio_metadata.1);
        
        // Perform speaker identification with timeout to avoid blocking
        let identification_start = std::time::Instant::now();
        match tokio::time::timeout(
            std::time::Duration::from_millis(500), // 500ms timeout for speaker ID
            voice_biometrics.identify_speaker(&audio_samples)
        ).await {
            Ok(Ok(Some(user_profile))) => {
                let identification_time = identification_start.elapsed();
                log::info!("✅ Speaker identified: {} (took {:.1}ms)", 
                          user_profile.name, identification_time.as_millis());
                
                Some(SpeakerInfo {
                    user_id: Some(user_profile.id),
                    user_name: Some(user_profile.name),
                    similarity_score: 0.85, // TODO: Get actual similarity score from identify_speaker
                    identified: true,
                })
            }
            Ok(Ok(None)) => {
                log::debug!("No speaker recognized (no confident match)");
                Some(SpeakerInfo {
                    user_id: None,
                    user_name: None,
                    similarity_score: 0.0,
                    identified: false,
                })
            }
            Ok(Err(e)) => {
                log::warn!("Speaker identification failed: {:?}", e);
                None
            }
            Err(_timeout) => {
                log::warn!("Speaker identification timed out (>500ms)");
                None
            }
        }
    } else {
        if !voice_biometrics.is_model_loaded().await {
            log::debug!("Speaker identification not available (model not loaded)");
        } else {
            log::debug!("No audio samples available for speaker identification");
        }
        None
    };

    // **AC2: Context Passing** - Enhanced result with speaker information
    let enhanced_result = TranscriptionResult {
        text: transcription_text.clone(),
        duration_seconds: audio_metadata.1,
        sample_count: audio_metadata.0,
        speaker_info,
    };

    // Log the complete result for validation
    log::info!("Enhanced transcription result: text=\"{}\", duration={:.2}s, samples={}, speaker={:?}",
               enhanced_result.text,
               enhanced_result.duration_seconds,
               enhanced_result.sample_count,
               enhanced_result.speaker_info);

    // **AC1: Multi-User Frontend Integration** - Return full result with speaker context
    // This enables the frontend to route music commands with user_id for personalization
    Ok(enhanced_result)
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
    online_mode_enabled: bool,
    search_backend: String,
    searxng_instance_url: String,
    brave_search_api_key: Option<String>,
    max_search_results: u32,
    db: State<'_, DatabaseState>
) -> Result<(), AuraError> {
    log::info!("Tauri command: save_settings called (provider: {}, server: {}, wake_word: {}, api_base_url: {}, model: {}, vad_sensitivity: {}, vad_timeout_ms: {}, stt_model: {}, voice: {}, online_mode: {}, search_backend: {}, max_results: {})",
               llm_provider, server_address, wake_word_enabled, api_base_url, model_name, vad_sensitivity, vad_timeout_ms, stt_model_name, voice_preference, online_mode_enabled, search_backend, max_search_results);

    let db = db.inner().lock().await;

    // Load existing settings to preserve Spotify and Home Assistant configuration
    let existing_settings = db.load_settings().ok().unwrap_or_else(|| Settings {
        llm_provider: "local".to_string(),
        server_address: String::new(),
        wake_word_enabled: false,
        api_base_url: "http://localhost:11434/v1".to_string(),
        model_name: "gemma:2b".to_string(),
        vad_sensitivity: 0.02,
        vad_timeout_ms: 1280,
        stt_model_name: "ggml-base.en.bin".to_string(),
        voice_preference: "male".to_string(),
        online_mode_enabled: false,
        search_backend: "searxng".to_string(),
        searxng_instance_url: "https://searx.be".to_string(),
        brave_search_api_key: None,
        max_search_results: 5,
        spotify_connected: false,
        spotify_client_id: String::new(),
        spotify_auto_play_enabled: true,
        ha_connected: false,
        ha_base_url: String::new(),
        ha_auto_sync: true,
        ha_onboarding_dismissed: false,
    });

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
        online_mode_enabled,
        search_backend,
        searxng_instance_url,
        brave_search_api_key,
        max_search_results,
        // Preserve existing Spotify settings
        spotify_connected: existing_settings.spotify_connected,
        spotify_client_id: existing_settings.spotify_client_id,
        spotify_auto_play_enabled: existing_settings.spotify_auto_play_enabled,
        // Preserve existing Home Assistant settings
        ha_connected: existing_settings.ha_connected,
        ha_base_url: existing_settings.ha_base_url,
        ha_auto_sync: existing_settings.ha_auto_sync,
        ha_onboarding_dismissed: existing_settings.ha_onboarding_dismissed,
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

        // Load existing settings (this command doesn't modify voice preference, RAG, Spotify, or Home Assistant settings)
        let existing_settings = db.load_settings().ok().unwrap_or_else(|| Settings {
            llm_provider: "local".to_string(),
            server_address: "".to_string(),
            wake_word_enabled: false,
            api_base_url: "http://localhost:11434/v1".to_string(),
            model_name: "gemma:2b".to_string(),
            vad_sensitivity: 0.02,
            vad_timeout_ms: 1280,
            stt_model_name: "ggml-base.en.bin".to_string(),
            voice_preference: "male".to_string(),
            online_mode_enabled: false,
            search_backend: "searxng".to_string(),
            searxng_instance_url: "https://searx.be".to_string(),
            brave_search_api_key: None,
            max_search_results: 5,
            spotify_connected: false,
            spotify_client_id: String::new(),
            spotify_auto_play_enabled: true,
            ha_connected: false,
            ha_base_url: String::new(),
            ha_auto_sync: true,
            ha_onboarding_dismissed: false,
        });

        let settings_to_save = Settings {
            llm_provider: llm_provider.clone(),
            server_address: server_address.clone(),
            wake_word_enabled,
            api_base_url: api_base_url.clone(),
            model_name: model_name.clone(),
            vad_sensitivity,
            vad_timeout_ms,
            stt_model_name: stt_model_name.clone(),
            voice_preference: existing_settings.voice_preference,
            // Preserve RAG settings
            online_mode_enabled: existing_settings.online_mode_enabled,
            search_backend: existing_settings.search_backend,
            searxng_instance_url: existing_settings.searxng_instance_url,
            brave_search_api_key: existing_settings.brave_search_api_key,
            max_search_results: existing_settings.max_search_results,
            // Preserve Spotify settings
            spotify_connected: existing_settings.spotify_connected,
            spotify_client_id: existing_settings.spotify_client_id,
            spotify_auto_play_enabled: existing_settings.spotify_auto_play_enabled,
            // Preserve Home Assistant settings
            ha_connected: existing_settings.ha_connected,
            ha_base_url: existing_settings.ha_base_url,
            ha_auto_sync: existing_settings.ha_auto_sync,
            ha_onboarding_dismissed: existing_settings.ha_onboarding_dismissed,
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

/// Get GPU acceleration status
#[tauri::command]
async fn get_gpu_info(
    ollama_sidecar: State<'_, Arc<StdMutex<OllamaSidecar>>>
) -> Result<ollama_sidecar::GpuInfo, AuraError> {
    log::info!("Tauri command: get_gpu_info called");

    let sidecar = ollama_sidecar.lock()
        .map_err(|e| AuraError::Internal(format!("Failed to lock Ollama sidecar: {}", e)))?;

    let gpu_info = sidecar.gpu_info().clone();

    log::info!("GPU Info: backend={}, available={}, device={:?}",
               gpu_info.backend,
               gpu_info.available,
               gpu_info.device_name);

    Ok(gpu_info)
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

// =============================================================================
// Spotify Music Integration Commands
// =============================================================================

use spotify_auth::{SpotifyAuth, calculate_token_expiry};
use spotify_client::{SpotifyClient, format_track_info, format_currently_playing};
use music_intent::{MusicIntentParser, MusicIntent};
use entity_manager::{EntityManager, Entity, EntityFilter};
use ha_client::HomeAssistantClient;
use smarthome_intent::{SmartHomeIntentParser, SmartHomeIntent, TemperatureUnit};

/// Start Spotify OAuth2 authorization flow
///
/// Opens the user's browser to Spotify's authorization page, waits for callback,
/// and exchanges the authorization code for access/refresh tokens.
#[tauri::command]
async fn spotify_start_auth(
    client_id: String,
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError> {
    log::info!("Starting Spotify authorization for client ID: {}", client_id);

    // Create auth manager
    let auth = SpotifyAuth::new(client_id.clone());

    // Start OAuth2 flow (this will block until user authorizes or times out)
    let token_response = auth.start_authorization()
        .await
        .map_err(|e| AuraError::Spotify(e.to_string()))?;

    // Save tokens to OS keyring
    secrets::save_spotify_access_token(&token_response.access_token)
        .map_err(|e| AuraError::Secrets(e))?;

    if let Some(refresh_token) = &token_response.refresh_token {
        secrets::save_spotify_refresh_token(refresh_token)
            .map_err(|e| AuraError::Secrets(e))?;
    }

    // Calculate and save token expiry
    let expiry = calculate_token_expiry(token_response.expires_in);
    secrets::save_spotify_token_expiry(&expiry)
        .map_err(|e| AuraError::Secrets(e))?;

    // Update database: mark as connected and save client ID
    let database = db.lock().await;
    let mut settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;

    settings.spotify_connected = true;
    settings.spotify_client_id = client_id;

    database.save_settings(&settings)
        .map_err(|e| AuraError::Database(e))?;

    log::info!("✓ Spotify authorization successful and settings saved");

    Ok(())
}

/// Disconnect Spotify account
///
/// Removes all tokens from OS keyring and updates database connection status
#[tauri::command]
async fn spotify_disconnect(db: State<'_, DatabaseState>) -> Result<(), AuraError> {
    log::info!("Disconnecting Spotify");

    // Delete tokens from keyring
    secrets::delete_spotify_tokens()
        .map_err(|e| AuraError::Secrets(e))?;

    // Update database
    let database = db.lock().await;
    let mut settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;

    settings.spotify_connected = false;

    database.save_settings(&settings)
        .map_err(|e| AuraError::Database(e))?;

    log::info!("✓ Spotify disconnected successfully");

    Ok(())
}

/// Spotify connection status response
#[derive(serde::Serialize)]
struct SpotifyStatusResponse {
    connected: bool,
    client_id: String,
    auto_play_enabled: bool,
}

/// Get Spotify connection status
#[tauri::command]
async fn spotify_get_status(db: State<'_, DatabaseState>) -> Result<SpotifyStatusResponse, AuraError> {
    let database = db.lock().await;
    let settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;

    // Verify tokens actually exist in keyring
    let connected = settings.spotify_connected && secrets::is_spotify_connected();

    Ok(SpotifyStatusResponse {
        connected,
        client_id: settings.spotify_client_id,
        auto_play_enabled: settings.spotify_auto_play_enabled,
    })
}

/// Save Spotify client ID to database
#[tauri::command]
async fn spotify_save_client_id(
    client_id: String,
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError> {
    log::info!("Saving Spotify client ID");

    let database = db.lock().await;
    let mut settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;

    settings.spotify_client_id = client_id;

    database.save_settings(&settings)
        .map_err(|e| AuraError::Database(e))?;

    Ok(())
}

/// Handle music command with intent recognition and Spotify playback (Multi-User Aware)
///
/// This is the main entry point for voice/text music commands. It parses the
/// user's intent, searches Spotify, and controls playback accordingly.
///
/// **Multi-User Support (AC1):**
/// - When `user_id` is provided (speaker identified), uses per-user Spotify tokens
/// - When `user_id` is None (unknown speaker), falls back to global Spotify account
/// - Provides appropriate error messages for each scenario (AC3)
#[tauri::command]
async fn spotify_handle_music_command(
    command: String,
    user_id: Option<i64>, // NEW: User context from voice biometrics
    db: State<'_, DatabaseState>,
) -> Result<String, AuraError> {
    log::info!("Handling music command: '{}' (user_id: {:?})", command, user_id);

    // Get client ID from database
    let database = db.lock().await;
    let settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;
    drop(database); // Release lock

    let client_id = settings.spotify_client_id;

    if client_id.is_empty() {
        return Err(AuraError::Spotify(
            "Spotify client ID not configured. Please enter your client ID in Settings.".to_string()
        ));
    }

    // **AC3: Graceful Fallback Logic**
    // Determine which Spotify account to use based on speaker identification
    let (client, user_context) = if let Some(uid) = user_id {
        // User identified - check if they have Spotify connected
        if secrets::is_user_spotify_connected(uid) {
            log::info!("✓ Using user {}'s Spotify account", uid);
            let client = SpotifyClient::new_for_user(client_id, uid)
                .map_err(|e| AuraError::Spotify(e.to_string()))?;
            (client, format!("user {}", uid))
        } else {
            // User identified but not connected to Spotify
            log::warn!("User {} identified but not connected to Spotify", uid);
            return Err(AuraError::Spotify(format!(
                "You haven't connected your Spotify account yet. Please go to Settings to link your Spotify account."
            )));
        }
    } else {
        // Unknown speaker - fall back to global Spotify account (legacy mode)
        if secrets::is_spotify_connected() {
            log::info!("⚠ Unknown speaker, using global Spotify account (legacy mode)");
            let client = SpotifyClient::new(client_id)
                .map_err(|e| AuraError::Spotify(e.to_string()))?;
            (client, "global account".to_string())
        } else {
            // No Spotify account connected at all
            return Err(AuraError::Spotify(
                "Spotify is not connected. Please connect your Spotify account in Settings.".to_string()
            ));
        }
    };

    log::debug!("Spotify client created for: {}", user_context);

    // Parse music intent
    let intent = MusicIntentParser::parse(&command);
    log::info!("Parsed intent: {:?}", intent);

    // Handle intent
    match intent {
        MusicIntent::PlaySong { song, artist, is_possessive } => {
            // AC2: Log possessive context for future personalization
            if is_possessive {
                log::debug!("Possessive command detected - user-specific song search (future enhancement)");
            }

            // Search for track
            let tracks = client.search_track(&song, artist.as_deref(), 10)
                .await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;

            if tracks.is_empty() {
                let query = if let Some(artist) = artist {
                    format!("{} by {}", song, artist)
                } else {
                    song.clone()
                };
                return Err(AuraError::Spotify(format!("No tracks found for '{}'", query)));
            }

            // Play first result
            let track = &tracks[0];
            client.play_track(&track.uri)
                .await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;

            let message = format!("Now playing: {}", format_track_info(track));
            log::info!("{}", message);
            Ok(message)
        }

        MusicIntent::PlayPlaylist { playlist_name, is_possessive } => {
            // AC2: Possessive context indicates user-specific playlist
            if is_possessive && user_id.is_some() {
                log::info!("✓ User-specific playlist requested: '{}' for user {:?}", playlist_name, user_id);
            }

            // Search user playlists
            let playlists = client.get_user_playlists(50)
                .await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;

            // Find playlist by name (case-insensitive)
            let playlist = playlists.iter().find(|p| {
                p.name.to_lowercase() == playlist_name.to_lowercase()
            });

            match playlist {
                Some(playlist) => {
                    // Note: Playing playlists requires context URI, not implemented in basic client
                    // For now, return a helpful message
                    Ok(format!("Found playlist '{}' with {} tracks. Playlist playback coming soon!", playlist.name, playlist.tracks.total))
                }
                None => {
                    Err(AuraError::Spotify(format!("Playlist '{}' not found", playlist_name)))
                }
            }
        }

        MusicIntent::PlayArtist { artist, is_possessive } => {
            // AC2: Log possessive context for future personalization
            if is_possessive {
                log::debug!("Possessive command detected - user-specific artist search (future enhancement)");
            }

            // Search for artist's top tracks
            let tracks = client.search_track(&artist, Some(&artist), 10)
                .await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;

            if tracks.is_empty() {
                return Err(AuraError::Spotify(format!("No tracks found for artist '{}'", artist)));
            }

            // Play first result
            let track = &tracks[0];
            client.play_track(&track.uri)
                .await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;

            let message = format!("Playing {} by {}", track.name, track.artists[0].name);
            log::info!("{}", message);
            Ok(message)
        }

        MusicIntent::Pause => {
            client.pause()
                .await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;

            Ok("Music paused".to_string())
        }

        MusicIntent::Resume => {
            client.resume()
                .await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;

            Ok("Music resumed".to_string())
        }

        MusicIntent::Next => {
            client.next()
                .await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;

            Ok("Skipped to next track".to_string())
        }

        MusicIntent::Previous => {
            client.previous()
                .await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;

            Ok("Skipped to previous track".to_string())
        }

        MusicIntent::GetCurrentTrack => {
            let current = client.get_current_track()
                .await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;

            let message = format_currently_playing(&current);
            Ok(message)
        }

        MusicIntent::Unknown => {
            Ok("I didn't understand that music command. Try 'play [song] by [artist]', 'pause', 'next', or 'what's playing?'".to_string())
        }
    }
}

/// Control Spotify playback (pause, resume, next, previous)
#[tauri::command]
async fn spotify_control_playback(
    action: String,
    db: State<'_, DatabaseState>,
) -> Result<String, AuraError> {
    log::info!("Spotify playback control: {}", action);

    if !secrets::is_spotify_connected() {
        return Err(AuraError::Spotify("Spotify not connected".to_string()));
    }

    let database = db.lock().await;
    let settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;
    drop(database);

    let client = SpotifyClient::new(settings.spotify_client_id)
        .map_err(|e| AuraError::Spotify(e.to_string()))?;

    match action.as_str() {
        "pause" => {
            client.pause().await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;
            Ok("Paused".to_string())
        }
        "resume" | "play" => {
            client.resume().await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;
            Ok("Resumed".to_string())
        }
        "next" => {
            client.next().await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;
            Ok("Next track".to_string())
        }
        "previous" => {
            client.previous().await
                .map_err(|e| AuraError::Spotify(e.to_string()))?;
            Ok("Previous track".to_string())
        }
        _ => Err(AuraError::Spotify(format!("Unknown playback action: {}", action)))
    }
}

/// Get currently playing track info
#[tauri::command]
async fn spotify_get_current_track(db: State<'_, DatabaseState>) -> Result<serde_json::Value, AuraError> {
    if !secrets::is_spotify_connected() {
        return Err(AuraError::Spotify("Spotify not connected".to_string()));
    }

    let database = db.lock().await;
    let settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;
    drop(database);

    let client = SpotifyClient::new(settings.spotify_client_id)
        .map_err(|e| AuraError::Spotify(e.to_string()))?;

    let current = client.get_current_track().await
        .map_err(|e| AuraError::Spotify(e.to_string()))?;

    // Serialize to JSON for frontend
    serde_json::to_value(&current)
        .map_err(|e| AuraError::Serialization(e))
}

/// Get available Spotify Connect devices
#[tauri::command]
async fn spotify_get_devices(db: State<'_, DatabaseState>) -> Result<serde_json::Value, AuraError> {
    if !secrets::is_spotify_connected() {
        return Err(AuraError::Spotify("Spotify not connected".to_string()));
    }

    let database = db.lock().await;
    let settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;
    drop(database);

    let client = SpotifyClient::new(settings.spotify_client_id)
        .map_err(|e| AuraError::Spotify(e.to_string()))?;

    let devices = client.get_devices().await
        .map_err(|e| AuraError::Spotify(e.to_string()))?;

    // Serialize to JSON for frontend
    serde_json::to_value(&devices)
        .map_err(|e| AuraError::Serialization(e))
}

// =============================================================================
// Multi-User Spotify Commands (AC2 & AC3)
// =============================================================================

/// Response for user profile with Spotify status
#[derive(Serialize)]
struct UserProfileWithSpotify {
    id: i64,
    name: String,
    enrollment_date: String,
    last_recognized: Option<String>,
    recognition_count: i64,
    is_active: bool,
    spotify_connected: bool,
    spotify_display_name: Option<String>,
    spotify_email: Option<String>,
    spotify_connected_at: Option<String>,
}

/// List all user profiles with their Spotify connection status (AC2)
#[tauri::command]
async fn list_user_profiles_with_spotify(
    voice_biometrics: State<'_, VoiceBiometricsState>,
    db: State<'_, DatabaseState>,
) -> Result<Vec<UserProfileWithSpotify>, AuraError> {
    // Get all user profiles
    let profiles = voice_biometrics.list_all_users().await
        .map_err(|e| AuraError::Database(e.to_string()))?;

    // Get database connection
    let database = db.lock().await;

    let mut result = Vec::new();
    for profile in profiles {
        // Check if user has Spotify connected
        let spotify_connected = secrets::is_user_spotify_connected(profile.id);

        // Query user_profiles table for Spotify metadata
        let spotify_metadata = database.query_rows(
            "SELECT spotify_display_name, spotify_email, spotify_connected_at
             FROM user_profiles WHERE id = ?1 LIMIT 1",
            &[&profile.id as &dyn rusqlite::ToSql],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            }
        ).unwrap_or_default();

        let (spotify_display_name, spotify_email, spotify_connected_at) =
            spotify_metadata.first().cloned().unwrap_or((None, None, None));

        result.push(UserProfileWithSpotify {
            id: profile.id,
            name: profile.name,
            enrollment_date: profile.enrollment_date,
            last_recognized: profile.last_recognized,
            recognition_count: profile.recognition_count,
            is_active: profile.is_active,
            spotify_connected,
            spotify_display_name,
            spotify_email,
            spotify_connected_at,
        });
    }

    Ok(result)
}

/// Start Spotify OAuth for a specific user (AC2)
#[tauri::command]
async fn user_spotify_start_auth(
    user_id: i64,
    client_id: String,
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError> {
    log::info!("Starting Spotify OAuth for user {}", user_id);

    // Verify user exists
    let database = db.lock().await;
    let user_count = database.query_rows(
        "SELECT COUNT(*) FROM user_profiles WHERE id = ?1",
        &[&user_id as &dyn rusqlite::ToSql],
        |row| row.get::<_, i64>(0)
    ).unwrap_or_default();

    let user_exists = user_count.first().copied().unwrap_or(0) > 0;

    if !user_exists {
        return Err(AuraError::Database(format!("User {} not found", user_id)));
    }
    drop(database);

    // Use SpotifyAuth to start OAuth flow
    use crate::spotify_auth::{SpotifyAuth, calculate_token_expiry};
    let auth = SpotifyAuth::new(client_id.clone());

    // Start auth flow and get tokens
    let token_response = auth.start_authorization().await
        .map_err(|e| AuraError::Spotify(e.to_string()))?;

    let access_token = token_response.access_token;
    let refresh_token = token_response.refresh_token
        .ok_or_else(|| AuraError::Spotify("No refresh token received".to_string()))?;
    let expires_at = calculate_token_expiry(token_response.expires_in);

    // Save user-scoped tokens
    secrets::save_user_spotify_access_token(user_id, &access_token)
        .map_err(|e| AuraError::Secrets(e))?;
    secrets::save_user_spotify_refresh_token(user_id, &refresh_token)
        .map_err(|e| AuraError::Secrets(e))?;
    secrets::save_user_spotify_token_expiry(user_id, &expires_at)
        .map_err(|e| AuraError::Secrets(e))?;

    // Get user info from Spotify
    let client = SpotifyClient::new_for_user(client_id, user_id)
        .map_err(|e| AuraError::Spotify(e.to_string()))?;

    let user_info = client.get_current_user().await
        .map_err(|e| AuraError::Spotify(e.to_string()))?;

    // Update database with Spotify metadata
    let database = db.lock().await;
    let now = chrono::Utc::now().to_rfc3339();
    database.execute_query(
        "UPDATE user_profiles
         SET spotify_connected = 1,
             spotify_display_name = ?1,
             spotify_email = ?2,
             spotify_connected_at = ?3
         WHERE id = ?4",
        &[
            &user_info.display_name as &dyn rusqlite::ToSql,
            &user_info.email,
            &now,
            &user_id,
        ],
    ).map_err(|e| AuraError::Database(e))?;

    log::info!("✓ User {} connected to Spotify: {}", user_id, user_info.display_name);

    Ok(())
}

/// Disconnect Spotify for a specific user (AC2)
#[tauri::command]
async fn user_spotify_disconnect(
    user_id: i64,
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError> {
    log::info!("Disconnecting Spotify for user {}", user_id);

    // Delete user-scoped tokens from keyring
    secrets::delete_user_spotify_tokens(user_id)
        .map_err(|e| AuraError::Secrets(e))?;

    // Update database
    let database = db.lock().await;
    database.execute_query(
        "UPDATE user_profiles
         SET spotify_connected = 0,
             spotify_display_name = '',
             spotify_email = '',
             spotify_user_id = '',
             spotify_connected_at = NULL
         WHERE id = ?1",
        &[&user_id as &dyn rusqlite::ToSql],
    ).map_err(|e| AuraError::Database(e))?;

    log::info!("✓ User {} disconnected from Spotify", user_id);

    Ok(())
}

/// Check if global Spotify tokens exist and can be migrated (AC3)
#[derive(Serialize)]
struct MigrationStatus {
    global_tokens_exist: bool,
    can_migrate: bool,
    user_count: usize,
}

#[tauri::command]
async fn check_global_spotify_migration(
    voice_biometrics: State<'_, VoiceBiometricsState>,
) -> Result<MigrationStatus, AuraError> {
    let global_tokens_exist = secrets::is_spotify_connected();

    let users = voice_biometrics.list_all_users().await
        .map_err(|e| AuraError::Database(e.to_string()))?;

    let user_count = users.len();
    let can_migrate = global_tokens_exist && user_count > 0;

    Ok(MigrationStatus {
        global_tokens_exist,
        can_migrate,
        user_count,
    })
}

/// Migrate global Spotify tokens to a specific user (AC3)
#[tauri::command]
async fn migrate_global_spotify_to_user(
    user_id: i64,
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError> {
    log::info!("Migrating global Spotify tokens to user {}", user_id);

    // Verify user exists
    let database = db.lock().await;
    let user_count = database.query_rows(
        "SELECT COUNT(*) FROM user_profiles WHERE id = ?1",
        &[&user_id as &dyn rusqlite::ToSql],
        |row| row.get::<_, i64>(0)
    ).unwrap_or_default();

    let user_exists = user_count.first().copied().unwrap_or(0) > 0;

    if !user_exists {
        return Err(AuraError::Database(format!("User {} not found", user_id)));
    }

    // Check if global tokens exist
    if !secrets::is_spotify_connected() {
        return Err(AuraError::Spotify("No global Spotify tokens to migrate".to_string()));
    }

    // Load global tokens
    let access_token = secrets::load_spotify_access_token()
        .map_err(|e| AuraError::Secrets(e))?;
    let refresh_token = secrets::load_spotify_refresh_token()
        .map_err(|e| AuraError::Secrets(e))?;
    let token_expiry = secrets::load_spotify_token_expiry()
        .map_err(|e| AuraError::Secrets(e))?;

    // Get Spotify client ID from database
    let settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;
    let client_id = settings.spotify_client_id;

    if client_id.is_empty() {
        return Err(AuraError::Spotify("No Spotify client ID configured".to_string()));
    }

    // Get user info from Spotify using global tokens
    let client = SpotifyClient::new(client_id.clone())
        .map_err(|e| AuraError::Spotify(e.to_string()))?;

    let user_info = client.get_current_user().await
        .map_err(|e| AuraError::Spotify(e.to_string()))?;

    drop(database);  // Release lock before saving to keyring

    // Save tokens to user-scoped keyring entries
    secrets::save_user_spotify_access_token(user_id, &access_token)
        .map_err(|e| AuraError::Secrets(e))?;
    secrets::save_user_spotify_refresh_token(user_id, &refresh_token)
        .map_err(|e| AuraError::Secrets(e))?;
    secrets::save_user_spotify_token_expiry(user_id, &token_expiry)
        .map_err(|e| AuraError::Secrets(e))?;

    // Update database with Spotify metadata
    let database = db.lock().await;
    let now = chrono::Utc::now().to_rfc3339();
    database.execute_query(
        "UPDATE user_profiles
         SET spotify_connected = 1,
             spotify_display_name = ?1,
             spotify_email = ?2,
             spotify_connected_at = ?3
         WHERE id = ?4",
        &[
            &user_info.display_name as &dyn rusqlite::ToSql,
            &user_info.email,
            &now,
            &user_id,
        ],
    ).map_err(|e| AuraError::Database(e))?;

    drop(database);

    // Delete global tokens (migration complete)
    secrets::delete_spotify_tokens()
        .map_err(|e| AuraError::Secrets(e))?;

    log::info!("✓ Global Spotify tokens migrated to user {}", user_id);

    Ok(())
}

// =============================================================================
// Home Assistant Integration Commands
// =============================================================================

/// Type alias for Home Assistant client state (shared across commands)
pub type HAClientState = Arc<TokioMutex<Option<HomeAssistantClient>>>;

/// Type alias for Entity Manager state
pub type EntityManagerState = Arc<EntityManager>;

/// Response for Home Assistant status query
#[derive(Serialize)]
struct HAStatusResponse {
    connected: bool,
    base_url: String,
    entity_count: usize,
}

/// Connect to Home Assistant
///
/// Establishes WebSocket connection, authenticates, and syncs entities.
#[tauri::command]
async fn ha_connect(
    base_url: String,
    token: String,
    db: State<'_, DatabaseState>,
    ha_client_state: State<'_, HAClientState>,
    entity_manager: State<'_, EntityManagerState>,
) -> Result<(), AuraError> {
    log::info!("Connecting to Home Assistant at {}", base_url);

    // Create client
    let client = HomeAssistantClient::new(
        base_url.clone(),
        token.clone(),
        entity_manager.inner().clone(),
    );

    // Connect and authenticate
    client.connect().await
        .map_err(|e| AuraError::HomeAssistant(e))?;

    // Store client in state
    let mut ha_client_lock = ha_client_state.lock().await;
    *ha_client_lock = Some(client);
    drop(ha_client_lock);

    // Save token to keyring
    secrets::save_ha_access_token(&token)
        .map_err(|e| AuraError::Secrets(e))?;

    // Update database settings
    let database = db.lock().await;
    let mut settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;

    settings.ha_connected = true;
    settings.ha_base_url = base_url;

    database.save_settings(&settings)
        .map_err(|e| AuraError::Database(e))?;

    log::info!("✓ Successfully connected to Home Assistant");

    Ok(())
}

/// Disconnect from Home Assistant
#[tauri::command]
async fn ha_disconnect(
    db: State<'_, DatabaseState>,
    ha_client_state: State<'_, HAClientState>,
) -> Result<(), AuraError> {
    log::info!("Disconnecting from Home Assistant");

    // Disconnect client
    let mut ha_client_lock = ha_client_state.lock().await;
    if let Some(client) = ha_client_lock.as_ref() {
        client.disconnect().await;
    }
    *ha_client_lock = None;
    drop(ha_client_lock);

    // Delete token from keyring
    secrets::delete_ha_access_token()
        .map_err(|e| AuraError::Secrets(e))?;

    // Update database settings
    let database = db.lock().await;
    let mut settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;

    settings.ha_connected = false;
    settings.ha_base_url = String::new();

    database.save_settings(&settings)
        .map_err(|e| AuraError::Database(e))?;

    log::info!("✓ Disconnected from Home Assistant");

    Ok(())
}

/// Get Home Assistant connection status
#[tauri::command]
async fn ha_get_status(
    db: State<'_, DatabaseState>,
    ha_client_state: State<'_, HAClientState>,
    entity_manager: State<'_, EntityManagerState>,
) -> Result<HAStatusResponse, AuraError> {
    let ha_client_lock = ha_client_state.lock().await;
    let connected = ha_client_lock.is_some() && ha_client_lock.as_ref().unwrap().is_connected().await;
    drop(ha_client_lock);

    let database = db.lock().await;
    let settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;
    drop(database);

    let entity_count = entity_manager.get_entity_count().await;

    Ok(HAStatusResponse {
        connected,
        base_url: settings.ha_base_url,
        entity_count,
    })
}

/// Get all entities (optionally filtered)
#[tauri::command]
async fn ha_get_entities(
    domain: Option<String>,
    area: Option<String>,
    entity_manager: State<'_, EntityManagerState>,
) -> Result<Vec<Entity>, AuraError> {
    let filter = EntityFilter {
        domain,
        area,
        device_class: None,
        state: None,
    };

    let entities = entity_manager.query_entities(filter).await;

    Ok(entities)
}

/// Get a specific entity by ID
#[tauri::command]
async fn ha_get_entity(
    entity_id: String,
    entity_manager: State<'_, EntityManagerState>,
) -> Result<Option<Entity>, AuraError> {
    let entity = entity_manager.get_entity(&entity_id).await;
    Ok(entity)
}

/// Call a Home Assistant service
#[tauri::command]
async fn ha_call_service(
    domain: String,
    service: String,
    entity_id: String,
    data: Option<serde_json::Value>,
    ha_client_state: State<'_, HAClientState>,
) -> Result<String, AuraError> {
    let ha_client_lock = ha_client_state.lock().await;

    if let Some(client) = ha_client_lock.as_ref() {
        client.call_service(&domain, &service, &entity_id, data).await
            .map_err(|e| AuraError::HomeAssistant(e))?;

        Ok(format!("✓ Called {}.{} on {}", domain, service, entity_id))
    } else {
        Err(AuraError::HomeAssistant("Not connected to Home Assistant".to_string()))
    }
}

/// Handle a natural language smart home command
///
/// This is the main entry point for voice/text smart home control.
/// Parses the command, matches entities, and executes the action.
#[tauri::command]
async fn ha_handle_smart_home_command(
    command: String,
    user_id: Option<i64>,
    ha_client_state: State<'_, HAClientState>,
    entity_manager: State<'_, EntityManagerState>,
    db: State<'_, DatabaseState>,
) -> Result<String, AuraError> {
    log::info!("Processing smart home command: {} (user_id={:?})", command, user_id);

    // Parse intent with user context for personal shortcuts
    let intent = SmartHomeIntentParser::parse_with_user(&command, user_id);

    log::debug!("Parsed intent: {:?}", intent);

    // Fetch user preferences for contextual defaults (AC3: Contextual Control)
    let user_prefs = if let Some(uid) = user_id {
        let database = db.lock().await;
        match database.query_rows(
            "SELECT user_id, default_room, default_light_entity, default_climate_entity, default_media_player_entity, updated_at FROM user_ha_preferences WHERE user_id = ?",
            &[&uid],
            |row| {
                Ok(UserHAPreferences {
                    user_id: row.get(0)?,
                    default_room: row.get(1)?,
                    default_light_entity: row.get(2)?,
                    default_climate_entity: row.get(3)?,
                    default_media_player_entity: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            }
        ) {
            Ok(mut results) => {
                if !results.is_empty() {
                    let prefs = results.remove(0);
                    log::debug!("Loaded user preferences for user_id={}: default_room={:?}", uid, prefs.default_room);
                    Some(prefs)
                } else {
                    None
                }
            }
            Err(e) => {
                log::warn!("Failed to load user preferences: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Check if connected
    let ha_client_lock = ha_client_state.lock().await;
    if ha_client_lock.is_none() {
        return Err(AuraError::HomeAssistant("Not connected to Home Assistant".to_string()));
    }

    let client = ha_client_lock.as_ref().unwrap();

    // Execute based on intent
    match intent {
        SmartHomeIntent::TurnOn { room, device_type, device_name: _ } => {
            // AC3: Apply contextual defaults - use user's default room if not specified
            let effective_room = room.clone().or_else(|| {
                user_prefs.as_ref().and_then(|p| p.default_room.clone())
            });

            if effective_room.is_some() && effective_room.as_ref() != room.as_ref() {
                log::debug!("Applied default room: {:?}", effective_room);
            }

            let entities = entity_manager.query_entities(EntityFilter {
                domain: device_type.clone(),
                area: effective_room,
                device_class: None,
                state: None,
            }).await;

            if entities.is_empty() {
                return Ok(format!("I couldn't find any {} devices{}",
                    device_type.unwrap_or_else(|| "".to_string()),
                    room.map(|r| format!(" in the {}", r)).unwrap_or_default()
                ));
            }

            // Turn on all matched entities
            for entity in &entities {
                if let Some(domain) = entity.entity_id.split('.').next() {
                    let _ = client.call_service(domain, "turn_on", &entity.entity_id, None).await;
                }
            }

            Ok(format!("✓ Turned on {} device{}",
                entities.len(),
                if entities.len() == 1 { "" } else { "s" }
            ))
        }

        SmartHomeIntent::TurnOff { room, device_type, device_name: _ } => {
            // AC3: Apply contextual defaults - use user's default room if not specified
            let effective_room = room.clone().or_else(|| {
                user_prefs.as_ref().and_then(|p| p.default_room.clone())
            });

            let entities = entity_manager.query_entities(EntityFilter {
                domain: device_type.clone(),
                area: effective_room,
                device_class: None,
                state: None,
            }).await;

            if entities.is_empty() {
                return Ok(format!("I couldn't find any {} devices{}",
                    device_type.unwrap_or_else(|| "".to_string()),
                    room.map(|r| format!(" in the {}", r)).unwrap_or_default()
                ));
            }

            // Turn off all matched entities
            for entity in &entities {
                if let Some(domain) = entity.entity_id.split('.').next() {
                    let _ = client.call_service(domain, "turn_off", &entity.entity_id, None).await;
                }
            }

            Ok(format!("✓ Turned off {} device{}",
                entities.len(),
                if entities.len() == 1 { "" } else { "s" }
            ))
        }

        SmartHomeIntent::SetBrightness { room, device_name: _, brightness } => {
            // AC3: Apply contextual defaults - use user's default room if not specified
            let effective_room = room.clone().or_else(|| {
                user_prefs.as_ref().and_then(|p| p.default_room.clone())
            });

            let entities = entity_manager.query_entities(EntityFilter {
                domain: Some("light".to_string()),
                area: effective_room,
                device_class: None,
                state: None,
            }).await;

            if entities.is_empty() {
                return Ok("I couldn't find any lights to adjust".to_string());
            }

            // Convert percentage to 0-255 range
            let brightness_value = ((brightness as f32 / 100.0) * 255.0) as u8;

            for entity in &entities {
                let data = serde_json::json!({
                    "brightness": brightness_value
                });
                let _ = client.call_service("light", "turn_on", &entity.entity_id, Some(data)).await;
            }

            Ok(format!("✓ Set brightness to {}%", brightness))
        }

        SmartHomeIntent::SetTemperature { room, temperature, unit } => {
            // AC3: Apply contextual defaults - use user's default room if not specified
            let effective_room = room.clone().or_else(|| {
                user_prefs.as_ref().and_then(|p| p.default_room.clone())
            });

            let entities = entity_manager.query_entities(EntityFilter {
                domain: Some("climate".to_string()),
                area: effective_room,
                device_class: None,
                state: None,
            }).await;

            if entities.is_empty() {
                return Ok("I couldn't find any climate devices".to_string());
            }

            // Convert to target temperature
            let temp_value = match unit {
                TemperatureUnit::Fahrenheit => temperature,
                TemperatureUnit::Celsius => temperature,
            };

            for entity in &entities {
                let data = serde_json::json!({
                    "temperature": temp_value
                });
                let _ = client.call_service("climate", "set_temperature", &entity.entity_id, Some(data)).await;
            }

            Ok(format!("✓ Set temperature to {:.1}°{}",
                temperature,
                match unit {
                    TemperatureUnit::Fahrenheit => "F",
                    TemperatureUnit::Celsius => "C",
                }
            ))
        }

        SmartHomeIntent::GetState { room, device_type, device_name: _ } => {
            let entities = entity_manager.query_entities(EntityFilter {
                domain: device_type.clone(),
                area: room.clone(),
                device_class: None,
                state: None,
            }).await;

            if entities.is_empty() {
                return Ok("I couldn't find any matching devices".to_string());
            }

            // Report state of first entity
            let entity = &entities[0];
            Ok(format!("{} is {}",
                entity.attributes.friendly_name.clone().unwrap_or_else(|| entity.entity_id.clone()),
                entity.state
            ))
        }

        SmartHomeIntent::OpenCover { room, device_name: _ } => {
            let entities = entity_manager.query_entities(EntityFilter {
                domain: Some("cover".to_string()),
                area: room.clone(),
                device_class: None,
                state: None,
            }).await;

            if entities.is_empty() {
                return Ok("I couldn't find any covers to open".to_string());
            }

            for entity in &entities {
                let _ = client.call_service("cover", "open_cover", &entity.entity_id, None).await;
            }

            Ok(format!("✓ Opening cover{}", if entities.len() == 1 { "" } else { "s" }))
        }

        SmartHomeIntent::CloseCover { room, device_name: _ } => {
            let entities = entity_manager.query_entities(EntityFilter {
                domain: Some("cover".to_string()),
                area: room.clone(),
                device_class: None,
                state: None,
            }).await;

            if entities.is_empty() {
                return Ok("I couldn't find any covers to close".to_string());
            }

            for entity in &entities {
                let _ = client.call_service("cover", "close_cover", &entity.entity_id, None).await;
            }

            Ok(format!("✓ Closing cover{}", if entities.len() == 1 { "" } else { "s" }))
        }

        SmartHomeIntent::Lock { room, device_name: _ } => {
            let entities = entity_manager.query_entities(EntityFilter {
                domain: Some("lock".to_string()),
                area: room.clone(),
                device_class: None,
                state: None,
            }).await;

            if entities.is_empty() {
                return Ok("I couldn't find any locks".to_string());
            }

            for entity in &entities {
                let _ = client.call_service("lock", "lock", &entity.entity_id, None).await;
            }

            Ok(format!("✓ Locked {} lock{}", entities.len(), if entities.len() == 1 { "" } else { "s" }))
        }

        SmartHomeIntent::Unlock { room, device_name: _ } => {
            let entities = entity_manager.query_entities(EntityFilter {
                domain: Some("lock".to_string()),
                area: room.clone(),
                device_class: None,
                state: None,
            }).await;

            if entities.is_empty() {
                return Ok("I couldn't find any locks".to_string());
            }

            for entity in &entities {
                let _ = client.call_service("lock", "unlock", &entity.entity_id, None).await;
            }

            Ok(format!("✓ Unlocked {} lock{}", entities.len(), if entities.len() == 1 { "" } else { "s" }))
        }

        SmartHomeIntent::ActivateScene { scene_name, user_id: intent_user_id } => {
            // If user_id is provided, try to resolve personal shortcut first
            let mut resolved_entity_id: Option<String> = None;

            if let Some(uid) = intent_user_id {
                log::debug!("Checking personal shortcuts for user_id={}, shortcut_name='{}'", uid, scene_name);

                let database = db.lock().await;
                match database.query_rows(
                    "SELECT ha_entity_id FROM user_ha_shortcuts WHERE user_id = ? AND LOWER(shortcut_name) = LOWER(?)",
                    &[&uid, &scene_name as &dyn rusqlite::ToSql],
                    |row| row.get::<_, String>(0)
                ) {
                    Ok(mut results) => {
                        if !results.is_empty() {
                            resolved_entity_id = Some(results.remove(0));
                            log::debug!("✓ Resolved personal shortcut '{}' -> '{}'", scene_name, resolved_entity_id.as_ref().unwrap());
                        } else {
                            log::debug!("Personal shortcut '{}' not found for user_id={}", scene_name, uid);
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to lookup personal shortcut: {}", e);
                    }
                }
            }

            // If we have a resolved entity_id from personal shortcuts, use it
            if let Some(entity_id) = resolved_entity_id {
                // Determine service based on entity type
                let (domain, service) = if entity_id.starts_with("scene.") {
                    ("scene", "turn_on")
                } else if entity_id.starts_with("script.") {
                    ("script", "turn_on")
                } else {
                    log::warn!("Unknown entity type for personal shortcut: {}", entity_id);
                    return Ok(format!("I couldn't activate your shortcut '{}' (unsupported entity type)", scene_name));
                };

                let _ = client.call_service(domain, service, &entity_id, None).await;
                Ok(format!("✓ Activated your {} ({})", scene_name, entity_id))
            } else {
                // Fall back to searching HA scene entities
                let entities = entity_manager.get_all_entities().await;

                // Find scene entity
                let scene_entity = entities.iter().find(|e| {
                    e.entity_id.starts_with("scene.") &&
                    e.attributes.friendly_name.as_ref().map(|n| n.to_lowercase().contains(&scene_name.to_lowercase())).unwrap_or(false)
                });

                if let Some(scene) = scene_entity {
                    let _ = client.call_service("scene", "turn_on", &scene.entity_id, None).await;
                    Ok(format!("✓ Activated {} scene", scene_name))
                } else {
                    Ok(format!("I couldn't find a scene named '{}'", scene_name))
                }
            }
        }

        SmartHomeIntent::Toggle { room, device_type, device_name: _ } => {
            // AC3: Apply contextual defaults
            let effective_room = room.clone().or_else(|| {
                user_prefs.as_ref().and_then(|p| p.default_room.clone())
            });

            let entities = entity_manager.query_entities(EntityFilter {
                domain: device_type.clone(),
                area: effective_room,
                device_class: None,
                state: None,
            }).await;

            if entities.is_empty() {
                return Ok("I couldn't find any devices to toggle".to_string());
            }

            for entity in &entities {
                if let Some(domain) = entity.entity_id.split('.').next() {
                    let _ = client.call_service(domain, "toggle", &entity.entity_id, None).await;
                }
            }

            Ok(format!("✓ Toggled {} device{}", entities.len(), if entities.len() == 1 { "" } else { "s" }))
        }

        SmartHomeIntent::SetupGuide => {
            Ok("To add devices to your smart home, go to the Devices tab and click 'Guide Me' next to any integration you want to set up. I can help you control: lights, thermostats, locks, covers, media players, and more. Just ask me to 'turn on the kitchen lights' or 'set bedroom to 72 degrees' once your devices are connected!".to_string())
        }

        SmartHomeIntent::Unknown => {
            Ok("I didn't understand that command. Try something like 'turn on the kitchen lights' or 'set bedroom to 72 degrees'.".to_string())
        }
    }
}

/// Dismiss the Home Assistant onboarding guide
#[tauri::command]
async fn ha_dismiss_onboarding(
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError> {
    let database = db.lock().await;
    let mut settings = database.load_settings()
        .map_err(|e| AuraError::Database(e))?;

    settings.ha_onboarding_dismissed = true;

    database.save_settings(&settings)
        .map_err(|e| AuraError::Database(e))?;

    log::info!("✓ Home Assistant onboarding dismissed");

    Ok(())
}

// ============================================================================
// User-Specific Home Assistant Commands (Personalization)
// ============================================================================

/// List all Home Assistant shortcuts for a specific user
#[tauri::command]
async fn list_user_ha_shortcuts(
    user_id: i64,
    db: State<'_, DatabaseState>,
) -> Result<Vec<UserHAShortcut>, AuraError> {
    log::info!("Listing HA shortcuts for user_id={}", user_id);

    let database = db.lock().await;
    let shortcuts = database.query_rows(
        "SELECT id, user_id, shortcut_name, ha_entity_id, entity_type, created_at FROM user_ha_shortcuts WHERE user_id = ?",
        &[&user_id],
        |row| {
            Ok(UserHAShortcut {
                id: row.get(0)?,
                user_id: row.get(1)?,
                shortcut_name: row.get(2)?,
                ha_entity_id: row.get(3)?,
                entity_type: row.get(4)?,
                created_at: row.get(5)?,
            })
        }
    ).map_err(|e| AuraError::Database(format!("Failed to query shortcuts: {}", e)))?;

    log::info!("Found {} shortcut(s) for user_id={}", shortcuts.len(), user_id);
    Ok(shortcuts)
}

/// Create a new Home Assistant shortcut for a user
#[tauri::command]
async fn create_user_ha_shortcut(
    user_id: i64,
    shortcut_name: String,
    ha_entity_id: String,
    entity_type: String,
    db: State<'_, DatabaseState>,
) -> Result<i64, AuraError> {
    log::info!(
        "Creating HA shortcut: user_id={}, name='{}', entity='{}', type='{}'",
        user_id, shortcut_name, ha_entity_id, entity_type
    );

    // Validate entity_type
    if entity_type != "scene" && entity_type != "script" {
        return Err(AuraError::Database(format!(
            "Invalid entity_type '{}'. Must be 'scene' or 'script'",
            entity_type
        )));
    }

    let database = db.lock().await;
    let created_at = chrono::Utc::now().to_rfc3339();

    let shortcut_id = database.execute_and_get_last_id(
        "INSERT INTO user_ha_shortcuts (user_id, shortcut_name, ha_entity_id, entity_type, created_at) VALUES (?, ?, ?, ?, ?)",
        &[&user_id, &shortcut_name as &dyn rusqlite::ToSql, &ha_entity_id as &dyn rusqlite::ToSql, &entity_type as &dyn rusqlite::ToSql, &created_at as &dyn rusqlite::ToSql],
    ).map_err(|e| AuraError::Database(format!("Failed to create shortcut: {}", e)))?;

    log::info!("✓ Created shortcut with id={}", shortcut_id);

    Ok(shortcut_id)
}

/// Delete a Home Assistant shortcut
#[tauri::command]
async fn delete_user_ha_shortcut(
    shortcut_id: i64,
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError> {
    log::info!("Deleting HA shortcut id={}", shortcut_id);

    let database = db.lock().await;
    let rows_affected = database.execute_query(
        "DELETE FROM user_ha_shortcuts WHERE id = ?",
        &[&shortcut_id],
    ).map_err(|e| AuraError::Database(format!("Failed to delete shortcut: {}", e)))?;

    if rows_affected == 0 {
        return Err(AuraError::Database(format!(
            "Shortcut with id={} not found",
            shortcut_id
        )));
    }

    log::info!("✓ Deleted shortcut id={}", shortcut_id);
    Ok(())
}

/// Get Home Assistant preferences for a specific user
#[tauri::command]
async fn get_user_ha_preferences(
    user_id: i64,
    db: State<'_, DatabaseState>,
) -> Result<Option<UserHAPreferences>, AuraError> {
    log::info!("Getting HA preferences for user_id={}", user_id);

    let database = db.lock().await;
    let mut prefs_vec = database.query_rows(
        "SELECT user_id, default_room, default_light_entity, default_climate_entity, default_media_player_entity, updated_at FROM user_ha_preferences WHERE user_id = ?",
        &[&user_id],
        |row| {
            Ok(UserHAPreferences {
                user_id: row.get(0)?,
                default_room: row.get(1)?,
                default_light_entity: row.get(2)?,
                default_climate_entity: row.get(3)?,
                default_media_player_entity: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }
    ).map_err(|e| AuraError::Database(format!("Failed to query preferences: {}", e)))?;

    let prefs = if prefs_vec.is_empty() {
        None
    } else {
        Some(prefs_vec.remove(0))
    };

    if prefs.is_some() {
        log::info!("✓ Found preferences for user_id={}", user_id);
    } else {
        log::info!("No preferences found for user_id={}", user_id);
    }

    Ok(prefs)
}

/// Update Home Assistant preferences for a specific user
#[tauri::command]
async fn update_user_ha_preferences(
    user_id: i64,
    default_room: Option<String>,
    default_light_entity: Option<String>,
    default_climate_entity: Option<String>,
    default_media_player_entity: Option<String>,
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError> {
    log::info!(
        "Updating HA preferences for user_id={}: room={:?}, light={:?}, climate={:?}, media={:?}",
        user_id, default_room, default_light_entity, default_climate_entity, default_media_player_entity
    );

    let database = db.lock().await;
    let updated_at = chrono::Utc::now().to_rfc3339();

    // Use INSERT OR REPLACE to handle both create and update cases
    database.execute_query(
        "INSERT OR REPLACE INTO user_ha_preferences (user_id, default_room, default_light_entity, default_climate_entity, default_media_player_entity, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        &[&user_id, &default_room as &dyn rusqlite::ToSql, &default_light_entity as &dyn rusqlite::ToSql, &default_climate_entity as &dyn rusqlite::ToSql, &default_media_player_entity as &dyn rusqlite::ToSql, &updated_at as &dyn rusqlite::ToSql],
    ).map_err(|e| AuraError::Database(format!("Failed to update preferences: {}", e)))?;

    log::info!("✓ Updated preferences for user_id={}", user_id);
    Ok(())
}

/// Lookup a user's shortcut by name (for NLU resolution)
#[tauri::command]
async fn lookup_user_shortcut(
    user_id: i64,
    shortcut_name: String,
    db: State<'_, DatabaseState>,
) -> Result<Option<String>, AuraError> {
    log::debug!("Looking up shortcut '{}' for user_id={}", shortcut_name, user_id);

    let database = db.lock().await;
    let mut entity_ids = database.query_rows(
        "SELECT ha_entity_id FROM user_ha_shortcuts WHERE user_id = ? AND LOWER(shortcut_name) = LOWER(?)",
        &[&user_id, &shortcut_name as &dyn rusqlite::ToSql],
        |row| row.get::<_, String>(0)
    ).map_err(|e| AuraError::Database(format!("Failed to lookup shortcut: {}", e)))?;

    let entity_id = if entity_ids.is_empty() {
        None
    } else {
        Some(entity_ids.remove(0))
    };

    if let Some(ref id) = entity_id {
        log::debug!("✓ Resolved '{}' -> '{}'", shortcut_name, id);
    } else {
        log::debug!("Shortcut '{}' not found for user_id={}", shortcut_name, user_id);
    }

    Ok(entity_id)
}

// ============================================================================
// Voice Biometrics (Speaker Recognition) Commands
// ============================================================================

/// Check if voice biometrics (speaker recognition) is available
#[tauri::command]
async fn voice_biometrics_status(
    voice_biometrics: State<'_, VoiceBiometricsState>,
) -> Result<bool, AuraError> {
    let is_ready = voice_biometrics.is_model_loaded().await;
    log::debug!("Voice biometrics status check: {}", is_ready);
    Ok(is_ready)
}

/// List all enrolled users
#[tauri::command]
async fn voice_biometrics_list_users(
    voice_biometrics: State<'_, VoiceBiometricsState>,
) -> Result<Vec<UserProfile>, AuraError> {
    match voice_biometrics.list_all_users().await {
        Ok(users) => {
            log::info!("Listed {} enrolled user(s)", users.len());
            Ok(users)
        }
        Err(e) => {
            log::error!("Failed to list users: {:?}", e);
            Err(AuraError::Internal(format!("Failed to list users: {:?}", e)))
        }
    }
}

/// Enroll a new user with voice samples
/// 
/// This is a placeholder command - in a full implementation, this would
/// need to handle audio recording from the frontend
#[tauri::command]
async fn voice_biometrics_enroll_user(
    user_name: String,
    voice_biometrics: State<'_, VoiceBiometricsState>,
) -> Result<i64, AuraError> {
    log::info!("Voice enrollment request for user: {}", user_name);
    
    // For now, return an error indicating this needs to be implemented
    // In the full implementation, this would:
    // 1. Start audio recording session
    // 2. Collect 3-5 voice samples from user
    // 3. Process samples through the enrollment pipeline
    // 4. Store voice print in database
    
    Err(AuraError::Internal(
        "Voice enrollment requires audio recording integration - not yet implemented".to_string()
    ))
}

/// Test command for voice biometrics enrollment using captured audio
/// 
/// This is a test-only command that simulates enrollment with real audio samples
#[tauri::command]
async fn voice_biometrics_test_enrollment(
    user_name: String,
    voice_pipeline: State<'_, Arc<StdMutex<NativeVoicePipeline>>>,
    voice_biometrics: State<'_, VoiceBiometricsState>,
) -> Result<i64, AuraError> {
    log::info!("Test enrollment request for user: {}", user_name);
    
    if !voice_biometrics.is_model_loaded().await {
        return Err(AuraError::Internal("Voice biometrics model not loaded".to_string()));
    }

    // For testing, we'll use the last captured audio as enrollment sample
    // In a real implementation, this would capture multiple samples
    let audio_samples = tokio::task::spawn_blocking({
        let pipeline_clone = voice_pipeline.inner().clone();
        move || {
            let pipeline = pipeline_clone.lock()
                .map_err(|e| AuraError::Internal(format!("Failed to lock pipeline: {}", e)))?;
            let samples = pipeline.get_last_audio_samples();
            Ok::<Vec<f32>, AuraError>(samples)
        }
    }).await
    .map_err(|e| AuraError::Internal(format!("Task panic: {}", e)))??;

    if audio_samples.len() < 8000 { // Less than 0.5 seconds
        return Err(AuraError::Internal("Insufficient audio for enrollment. Please speak longer.".to_string()));
    }

    // For testing, create multiple "variations" by using different segments of the same audio
    let sample_size = audio_samples.len() / 3;
    let mut enrollment_samples = Vec::new();
    
    for i in 0..3 {
        let start = i * sample_size;
        let end = std::cmp::min(start + sample_size * 2, audio_samples.len());
        if end > start {
            enrollment_samples.push(audio_samples[start..end].to_vec());
        }
    }

    // Ensure we have at least 3 samples
    while enrollment_samples.len() < 3 {
        enrollment_samples.push(audio_samples.clone());
    }

    log::info!("Enrolling user '{}' with {} audio samples", user_name, enrollment_samples.len());
    for (i, sample) in enrollment_samples.iter().enumerate() {
        log::debug!("Sample {}: {} samples ({:.2}s)", i+1, sample.len(), sample.len() as f32 / 16000.0);
    }

    match voice_biometrics.enroll_user(user_name.clone(), enrollment_samples).await {
        Ok(user_id) => {
            log::info!("✅ Successfully enrolled user '{}' with ID: {}", user_name, user_id);
            Ok(user_id)
        }
        Err(e) => {
            log::error!("❌ Failed to enroll user '{}': {:?}", user_name, e);
            Err(AuraError::Internal(format!("Enrollment failed: {:?}", e)))
        }
    }
}

/// Delete a user profile
#[tauri::command]
async fn voice_biometrics_delete_user(
    user_id: i64,
    voice_biometrics: State<'_, VoiceBiometricsState>,
) -> Result<(), AuraError> {
    match voice_biometrics.delete_user_profile(user_id).await {
        Ok(()) => {
            log::info!("✓ User profile {} deleted", user_id);
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to delete user profile {}: {:?}", user_id, e);
            Err(AuraError::Internal(format!("Failed to delete user: {:?}", e)))
        }
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
            // RAG / Online Mode defaults (disabled by default for privacy)
            online_mode_enabled: false,
            search_backend: "searxng".to_string(),
            searxng_instance_url: "https://searx.be".to_string(),
            brave_search_api_key: None,
            max_search_results: 5,
            // Spotify defaults (disconnected by default)
            spotify_connected: false,
            spotify_client_id: String::new(),
            spotify_auto_play_enabled: true,
            // Home Assistant defaults (disconnected by default)
            ha_connected: false,
            ha_base_url: String::new(),
            ha_auto_sync: true,
            ha_onboarding_dismissed: false,
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

    // Initialize Home Assistant state
    let entity_manager: EntityManagerState = Arc::new(EntityManager::new());
    let ha_client_state: HAClientState = Arc::new(TokioMutex::new(None));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(database.clone())
        .manage(llm_engine)
        .manage(entity_manager)
        .manage(ha_client_state)
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
            fetch_available_models,
            get_gpu_info,
            // Spotify Music Integration commands
            spotify_start_auth,
            spotify_disconnect,
            spotify_get_status,
            spotify_save_client_id,
            spotify_handle_music_command,
            spotify_control_playback,
            spotify_get_current_track,
            spotify_get_devices,
            // Multi-User Spotify commands (AC2 & AC3)
            list_user_profiles_with_spotify,
            user_spotify_start_auth,
            user_spotify_disconnect,
            check_global_spotify_migration,
            migrate_global_spotify_to_user,
            // Home Assistant Integration commands
            ha_connect,
            ha_disconnect,
            ha_get_status,
            ha_get_entities,
            ha_get_entity,
            ha_call_service,
            ha_handle_smart_home_command,
            ha_dismiss_onboarding,
            // User-Specific Home Assistant commands (Personalization)
            list_user_ha_shortcuts,
            create_user_ha_shortcut,
            delete_user_ha_shortcut,
            get_user_ha_preferences,
            update_user_ha_preferences,
            lookup_user_shortcut,
            // Voice Biometrics commands
            voice_biometrics_status,
            voice_biometrics_list_users,
            voice_biometrics_enroll_user,
            voice_biometrics_delete_user,
            voice_biometrics_test_enrollment
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

            // Initialize Voice Biometrics (Speaker Recognition)
            log::info!("Initializing voice biometrics system...");
            let voice_biometrics = VoiceBiometrics::new(
                database_for_setup.clone(),
                model_path.clone(),
            );

            // Try to initialize the speaker recognition model
            let voice_biometrics_ready = {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                runtime.block_on(async {
                    match voice_biometrics.initialize_model().await {
                        Ok(()) => {
                            log::info!("✓ Voice biometrics system initialized successfully");
                            log::info!("  - Model: WeSpeaker ECAPA-TDNN");
                            log::info!("  - Mode: Real-time speaker recognition");
                            log::info!("  - Privacy: 100% offline processing");
                            true
                        }
                        Err(e) => {
                            log::warn!("⚠ Voice biometrics initialization failed: {:?}", e);
                            log::warn!("  Speaker recognition will be disabled");
                            log::warn!("  Users can still enroll and use basic features");
                            false
                        }
                    }
                })
            };

            if voice_biometrics_ready {
                log::info!("✓ Voice biometrics ready for real-time speaker identification");
            } else {
                log::info!("ℹ Voice biometrics running in fallback mode (no speaker model)");
            }

            // Register voice biometrics as managed state
            let voice_biometrics_state: VoiceBiometricsState = Arc::new(voice_biometrics);
            app.manage(voice_biometrics_state.clone());

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
