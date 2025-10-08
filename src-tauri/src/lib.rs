mod native_voice;
mod tts;
mod llm;
mod database;
mod secrets;

use native_voice::NativeVoicePipeline;
use tts::TextToSpeech;
use llm::LLMEngine;
use database::{Database, DatabaseState, Conversation, Message, Settings, get_database_path};
use std::sync::{Arc, Mutex};
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
async fn handle_user_prompt(prompt: String, llm_engine: State<'_, Arc<Mutex<LLMEngine>>>) -> Result<String, String> {
    log::info!("Tauri command: handle_user_prompt called with: '{}'", prompt);

    // Clone the Arc to use in the async block
    let llm_engine_clone = llm_engine.inner().clone();

    // Spawn a blocking task to handle the mutex lock and async call
    let result = tokio::task::spawn_blocking(move || {
        let runtime = tokio::runtime::Handle::current();
        let llm = llm_engine_clone.lock()
            .map_err(|e| format!("Failed to lock LLM engine: {}", e))?;

        runtime.block_on(llm.generate_response(&prompt))
    }).await
    .map_err(|e| format!("Task panicked: {}", e))??;

    Ok(result)
}

#[tauri::command]
async fn listen_and_transcribe(voice_pipeline: State<'_, Arc<Mutex<NativeVoicePipeline>>>) -> Result<String, String> {
    log::info!("Tauri command: listen_and_transcribe called (Push-to-Talk)");

    let voice_pipeline_clone = voice_pipeline.inner().clone();

    // Spawn blocking task for sync operation
    let result = tokio::task::spawn_blocking(move || {
        let voice_pipeline = voice_pipeline_clone.lock()
            .map_err(|e| format!("Failed to lock voice pipeline: {}", e))?;

        voice_pipeline.start_transcription()
    }).await
    .map_err(|e| format!("Task panicked: {}", e))??;

    Ok(result)
}

#[tauri::command]
fn speak_text(text: String, tts_engine: State<'_, Arc<Mutex<TextToSpeech>>>) -> Result<(), String> {
    log::info!("Tauri command: speak_text called ({} chars)", text.len());
    log::info!("Text to speak: '{}'", if text.len() > 100 { format!("{}...", &text[..100]) } else { text.clone() });

    let mut tts = tts_engine.inner().lock().map_err(|e| {
        log::error!("Failed to lock TTS engine: {}", e);
        format!("Failed to lock TTS engine: {}", e)
    })?;

    log::info!("TTS engine locked, calling speak()...");
    let result = tts.speak(&text);

    match &result {
        Ok(_) => log::info!("✓ TTS speak() completed successfully"),
        Err(e) => log::error!("✗ TTS speak() failed: {}", e),
    }

    result
}

#[tauri::command]
async fn cancel_generation(llm_engine: State<'_, Arc<Mutex<LLMEngine>>>) -> Result<(), String> {
    log::info!("Tauri command: cancel_generation called");

    // Clone the Arc to use in async block
    let llm_engine_clone = llm_engine.inner().clone();

    // Spawn a blocking task to lock the mutex, then call async cancel_generation
    tokio::task::spawn_blocking(move || {
        let runtime = tokio::runtime::Handle::current();
        let llm = llm_engine_clone.lock()
            .map_err(|e| format!("Failed to lock LLM engine: {}", e))?;

        // Call the async cancel_generation method
        runtime.block_on(llm.cancel_generation());

        Ok::<(), String>(())
    }).await
    .map_err(|e| format!("Task panicked: {}", e))??;

    Ok(())
}

#[tauri::command]
fn cancel_recording(voice_pipeline: State<'_, Arc<Mutex<NativeVoicePipeline>>>) -> Result<(), String> {
    log::info!("Tauri command: cancel_recording called");

    let voice_pipeline = voice_pipeline.inner().lock()
        .map_err(|e| format!("Failed to lock voice pipeline: {}", e))?;

    // Use the new cancel_and_reset method for clean state reset
    voice_pipeline.cancel_and_reset()
}

// Database Commands

#[tauri::command]
fn load_conversations(db: State<'_, DatabaseState>) -> Result<Vec<Conversation>, String> {
    log::info!("Tauri command: load_conversations called");

    let db = db.inner().lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.load_conversations()
}

#[tauri::command]
fn load_messages(conversation_id: i64, db: State<'_, DatabaseState>) -> Result<Vec<Message>, String> {
    log::info!("Tauri command: load_messages called for conversation {}", conversation_id);

    let db = db.inner().lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.load_messages(conversation_id)
}

#[tauri::command]
fn create_new_conversation(db: State<'_, DatabaseState>) -> Result<i64, String> {
    log::info!("Tauri command: create_new_conversation called");

    let db = db.inner().lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.create_conversation(None)
}

#[tauri::command]
fn save_message(
    conversation_id: i64,
    role: String,
    content: String,
    db: State<'_, DatabaseState>
) -> Result<i64, String> {
    log::debug!("Tauri command: save_message called (conversation: {}, role: {})", conversation_id, role);

    let db = db.inner().lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.save_message(conversation_id, &role, &content)
}

#[tauri::command]
fn delete_conversation(conversation_id: i64, db: State<'_, DatabaseState>) -> Result<(), String> {
    log::info!("Tauri command: delete_conversation called for conversation {}", conversation_id);

    let db = db.inner().lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.delete_conversation(conversation_id)
}

#[tauri::command]
fn update_conversation_title(
    conversation_id: i64,
    title: String,
    db: State<'_, DatabaseState>
) -> Result<(), String> {
    log::info!("Tauri command: update_conversation_title called for conversation {} with title: {}", conversation_id, title);

    let db = db.inner().lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.update_conversation_title(conversation_id, &title)
}

#[tauri::command]
async fn generate_conversation_title(
    prompt: String,
    llm_engine: State<'_, Arc<Mutex<LLMEngine>>>
) -> Result<String, String> {
    log::info!("Tauri command: generate_conversation_title called");

    // Clone the Arc to use in the async block
    let llm_engine_clone = llm_engine.inner().clone();
    let title_prompt = format!(
        "Create a very short title (maximum 5 words, no quotes) for a conversation starting with: '{}'",
        prompt.chars().take(150).collect::<String>()
    );

    // Spawn a blocking task to handle the mutex lock and async call
    let raw_title = tokio::task::spawn_blocking(move || {
        let runtime = tokio::runtime::Handle::current();
        let llm = llm_engine_clone.lock()
            .map_err(|e| format!("Failed to lock LLM engine: {}", e))?;

        runtime.block_on(llm.generate_response(&title_prompt))
    }).await
    .map_err(|e| format!("Task panicked: {}", e))??;

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
fn load_settings(db: State<'_, DatabaseState>) -> Result<Settings, String> {
    log::info!("Tauri command: load_settings called");

    let db = db.inner().lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.load_settings()
}

#[tauri::command]
fn save_settings(
    llm_provider: String,
    server_address: String,
    wake_word_enabled: bool,
    api_base_url: String,
    model_name: String,
    vad_sensitivity: f32,
    vad_timeout_ms: u32,
    stt_model_name: String,
    db: State<'_, DatabaseState>
) -> Result<(), String> {
    log::info!("Tauri command: save_settings called (provider: {}, server: {}, wake_word: {}, api_base_url: {}, model: {}, vad_sensitivity: {}, vad_timeout_ms: {}, stt_model: {})",
               llm_provider, server_address, wake_word_enabled, api_base_url, model_name, vad_sensitivity, vad_timeout_ms, stt_model_name);

    let db = db.inner().lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let settings = Settings {
        llm_provider,
        server_address,
        wake_word_enabled,
        api_base_url,
        model_name,
        vad_sensitivity,
        vad_timeout_ms,
        stt_model_name,
    };

    db.save_settings(&settings)
}

#[tauri::command]
fn save_api_key(api_key: String) -> Result<(), String> {
    log::info!("Tauri command: save_api_key called");

    secrets::save_api_key(&api_key)
}

#[tauri::command]
fn load_api_key() -> Result<String, String> {
    log::info!("Tauri command: load_api_key called");

    secrets::load_api_key()
}

#[tauri::command]
fn update_vad_settings(
    sensitivity: f32,
    timeout_ms: u32,
    voice_pipeline: State<'_, Arc<Mutex<NativeVoicePipeline>>>
) -> Result<(), String> {
    log::info!("Tauri command: update_vad_settings called (sensitivity: {}, timeout_ms: {})", sensitivity, timeout_ms);

    let voice_pipeline = voice_pipeline.inner().lock()
        .map_err(|e| format!("Failed to lock voice pipeline: {}", e))?;

    voice_pipeline.update_vad_settings(sensitivity, timeout_ms)
}

#[tauri::command]
fn set_voice_state(
    state: String,
    voice_pipeline: State<'_, Arc<Mutex<NativeVoicePipeline>>>
) -> Result<(), String> {
    log::info!("Tauri command: set_voice_state called (state: {})", state);

    let voice_pipeline = voice_pipeline.inner().lock()
        .map_err(|e| format!("Failed to lock voice pipeline: {}", e))?;

    // Parse state string to VoiceState enum
    use native_voice::VoiceState;
    let voice_state = match state.as_str() {
        "idle" => VoiceState::Idle,
        "listening_for_wake_word" => VoiceState::ListeningForWakeWord,
        "transcribing" => VoiceState::Transcribing,
        "speaking" => VoiceState::Speaking,
        _ => return Err(format!("Invalid voice state: {}. Must be one of: idle, listening_for_wake_word, transcribing, speaking", state)),
    };

    voice_pipeline.set_state(voice_state)
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
    voice_pipeline: State<'_, Arc<Mutex<NativeVoicePipeline>>>,
    database: State<'_, DatabaseState>,
) -> Result<(), String> {
    log::info!("Reloading voice pipeline with new settings...");
    log::info!("  STT model: {}", stt_model_name);
    log::info!("  VAD sensitivity: {}", vad_sensitivity);
    log::info!("  VAD timeout: {}ms", vad_timeout_ms);
    log::info!("  Wake word enabled: {}", wake_word_enabled);

    // Save settings to database first
    {
        let settings_to_save = Settings {
            llm_provider: llm_provider.clone(),
            server_address: server_address.clone(),
            wake_word_enabled,
            api_base_url: api_base_url.clone(),
            model_name: model_name.clone(),
            vad_sensitivity,
            vad_timeout_ms,
            stt_model_name: stt_model_name.clone(),
        };

        let db = database.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
        db.save_settings(&settings_to_save)?;
    }

    // Determine model path
    let model_path = dirs::data_local_dir()
        .map(|p| p.join("nivora-aura").join("models"))
        .unwrap_or_else(|| std::path::PathBuf::from("./models"));

    // Stop the old pipeline and create a new one
    let new_pipeline = {
        // First, stop the old pipeline
        let old_pipeline = voice_pipeline
            .lock()
            .map_err(|e| format!("Failed to lock voice pipeline: {}", e))?;

        log::info!("Stopping current voice pipeline...");
        old_pipeline.stop();

        // Drop the old pipeline before creating the new one
        drop(old_pipeline);

        // Create new pipeline with updated settings
        log::info!("Creating new voice pipeline...");
        NativeVoicePipeline::new(
            app_handle.clone(),
            model_path.clone(),
            stt_model_name.clone(),
            vad_sensitivity,
            vad_timeout_ms,
        )?
    };

    // Start the new pipeline
    log::info!("Starting new voice pipeline...");
    new_pipeline.start()?;

    // Replace the old pipeline with the new one
    {
        let mut pipeline = voice_pipeline
            .lock()
            .map_err(|e| format!("Failed to lock voice pipeline for replacement: {}", e))?;
        *pipeline = new_pipeline;
    }

    log::info!("✓ Voice pipeline reloaded successfully");
    Ok(())
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
                    Arc::new(Mutex::new(db))
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
    let db_for_llm = database.lock().unwrap();
    let settings = db_for_llm.load_settings().unwrap_or_else(|e| {
        log::warn!("Failed to load settings, using defaults: {}", e);
        Settings {
            llm_provider: "local".to_string(),
            server_address: "".to_string(),
            wake_word_enabled: false,
            api_base_url: "http://localhost:11434/v1".to_string(),
            model_name: "llama3".to_string(),
            vad_sensitivity: 0.02,
            vad_timeout_ms: 1280,
            stt_model_name: "ggml-base.en.bin".to_string(),
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
            Arc::new(Mutex::new(llm))
        }
        Err(e) => {
            log::error!("✗ Failed to initialize LLM engine: {}", e);
            log::error!("  Make sure you have configured the API Base URL and Model Name in settings");
            log::error!("  Example: Ollama at http://localhost:11434/v1 with model 'llama3'");
            panic!("Cannot start without LLM configuration. Error: {}", e);
        }
    };

    // Initialize Subprocess-based Piper TTS engine
    log::info!("Initializing subprocess-based Piper TTS engine...");

    // Determine piper binary path
    let piper_binary = std::env::var("PIPER_BINARY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/usr/local/bin/piper"));

    // Determine voice model path
    let voice_model_path = dirs::data_local_dir()
        .map(|p| p.join("nivora-aura").join("voices").join("en_US-lessac-medium.onnx"))
        .unwrap_or_else(|| std::path::PathBuf::from("./voices/en_US-lessac-medium.onnx"));

    // Determine espeak-ng data path
    // Use absolute path to the project's piper directory
    let espeak_data_path = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .and_then(|exe_dir| {
            // In dev mode, executable is in src-tauri/target/debug/
            // Go up to project root: target/debug/ -> target/ -> src-tauri/ -> project_root/
            exe_dir
                .parent() // target/debug -> target
                .and_then(|target| target.parent()) // target -> src-tauri
                .and_then(|src_tauri| src_tauri.parent()) // src-tauri -> project_root
                .map(|root| root.join("piper").join("espeak-ng-data"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("/storage/dev/aura-desktop/piper/espeak-ng-data"));

    // Create voices directory if it doesn't exist
    if let Some(parent) = voice_model_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::warn!("Failed to create voices directory: {}", e);
        }
    }

    log::info!("Piper binary path: {:?}", piper_binary);
    log::info!("Voice model path: {:?}", voice_model_path);
    log::info!("eSpeak-NG data path: {:?}", espeak_data_path);

    let tts_engine = match TextToSpeech::new(piper_binary.clone(), voice_model_path.clone(), espeak_data_path.clone()) {
        Ok(tts) => {
            log::info!("✓ Subprocess-based Piper TTS engine initialized successfully");
            log::info!("  - Piper binary: {:?}", piper_binary);
            log::info!("  - Voice model: {:?}", voice_model_path);
            log::info!("  - Using stable subprocess architecture");
            Arc::new(Mutex::new(tts))
        }
        Err(e) => {
            log::error!("✗ Failed to initialize subprocess-based Piper TTS engine: {}", e);
            log::error!("  Please install Piper TTS and download a voice model (.onnx file)");
            log::error!("  See README.md for installation instructions");
            panic!("Cannot start TTS engine. Error: {}", e);
        }
    };

    log::info!("=== Starting Tauri Application ===");

    // Clone database for use in setup closure
    let database_for_setup = database.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(database.clone())
        .manage(llm_engine)
        .manage(tts_engine)
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
            reload_voice_pipeline
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

            // Load settings again for voice pipeline configuration
            let vad_settings = {
                let db = database_for_setup.lock().unwrap();
                db.load_settings().ok()
            };

            let vad_sensitivity = vad_settings.as_ref().map(|s| s.vad_sensitivity).unwrap_or(0.02);
            let vad_timeout_ms = vad_settings.as_ref().map(|s| s.vad_timeout_ms).unwrap_or(1280);
            let stt_model_name = vad_settings.as_ref().map(|s| s.stt_model_name.clone()).unwrap_or_else(|| "ggml-base.en.bin".to_string());

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

                    Arc::new(Mutex::new(pipeline))
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
                let pipeline = voice_pipeline.lock().unwrap();
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
                        let db = database_for_status.lock().unwrap();
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
