//! Native Voice Pipeline
//!
//! Unified voice module using:
//! - whisper-rs for speech-to-text transcription
//! - cpal for audio input
//! - Energy-based VAD for wake word simulation
//!
//! Architecture:
//! 1. Single audio stream (16kHz mono) from microphone via cpal
//! 2. Continuous energy-based voice activity detection
//! 3. On-demand STT transcription (activated by voice activity or Push-to-Talk)
//! 4. Built-in VAD using RMS energy for end-of-speech detection

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use log::{debug, error, info, warn};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

// Audio configuration constants
const SAMPLE_RATE: u32 = 16000; // Required by whisper-rs
const CHANNELS: u16 = 1; // Mono audio
const CHUNK_SIZE: usize = 512; // Process in small chunks for responsiveness

// VAD (Voice Activity Detection) constants
const VOICE_FRAMES_REQUIRED: usize = 10; // Require consistent voice energy for wake word
const MAX_RECORDING_SECONDS: usize = 30; // Maximum 30 seconds per transcription
const SKIP_FRAMES_AFTER_WAKE_WORD: usize = 15; // Skip ~500ms of audio after wake word to prevent capturing it
const MIN_RECORDING_FRAMES: usize = 30; // Minimum 30 frames (~1 second) before allowing silence detection

/// Voice pipeline state machine
///
/// This enum represents the current operational state of the voice pipeline.
/// State transitions are explicit and controlled to prevent race conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum VoiceState {
    /// Pipeline is inactive (not listening for anything)
    Idle,
    /// Actively listening for wake word via energy-based VAD
    ListeningForWakeWord,
    /// Recording and transcribing user speech
    Transcribing,
    /// Assistant is speaking (TTS active) - wake word detection disabled to prevent feedback loop
    Speaking,
}

/// Voice pipeline state
pub struct NativeVoicePipeline {
    app_handle: AppHandle,
    model_path: PathBuf,
    stt_model_name: String, // STT model filename (e.g., "ggml-base.en.bin")

    // State machine (thread-safe, accessible from both audio thread and command handlers)
    state: Arc<Mutex<VoiceState>>,

    // Audio stream control
    wake_word_active: Arc<AtomicBool>, // Set to false to stop the entire audio loop

    // Recording buffers and signals
    recording_buffer: Arc<Mutex<Vec<f32>>>, // Dedicated buffer for active recordings
    recording_complete: Arc<AtomicBool>,    // Signal when recording finished (silence detected)
    skip_frames_counter: Arc<AtomicUsize>, // Skip N frames after transitioning to prevent wake word capture

    // Wake word detection
    voice_detected: Arc<AtomicBool>, // Track if voice activity detected for wake word

    // VAD Configuration (shared with audio thread via Arc<Mutex>)
    vad_sensitivity: Arc<Mutex<f32>>, // Voice energy threshold (0.0-1.0), controls microphone sensitivity
    vad_timeout_ms: Arc<Mutex<u32>>,  // Silence timeout in milliseconds before ending recording
}

/// Service status for frontend
#[derive(serde::Serialize, Clone)]
struct ServiceStatus {
    service: String,
    connected: bool,
}

impl NativeVoicePipeline {
    /// Create a new native voice pipeline
    pub fn new(
        app_handle: AppHandle,
        model_path: PathBuf,
        stt_model_name: String,
        vad_sensitivity: f32,
        vad_timeout_ms: u32,
    ) -> Result<Self, String> {
        info!("Initializing native voice pipeline: stt_model={}, vad_sensitivity={}, vad_timeout_ms={}",
              stt_model_name, vad_sensitivity, vad_timeout_ms);

        Ok(Self {
            app_handle,
            model_path,
            stt_model_name,
            state: Arc::new(Mutex::new(VoiceState::Idle)),
            wake_word_active: Arc::new(AtomicBool::new(false)),
            recording_buffer: Arc::new(Mutex::new(Vec::with_capacity(
                SAMPLE_RATE as usize * MAX_RECORDING_SECONDS,
            ))),
            recording_complete: Arc::new(AtomicBool::new(false)),
            skip_frames_counter: Arc::new(AtomicUsize::new(0)),
            voice_detected: Arc::new(AtomicBool::new(false)),
            vad_sensitivity: Arc::new(Mutex::new(vad_sensitivity)),
            vad_timeout_ms: Arc::new(Mutex::new(vad_timeout_ms)),
        })
    }

    /// Start the voice pipeline
    ///
    /// This spawns a background thread that:
    /// 1. Captures audio from the microphone via cpal
    /// 2. Detects voice activity using energy-based VAD
    /// 3. Emits wake_word_detected event when voice is detected
    pub fn start(&self) -> Result<(), String> {
        info!("Starting native voice pipeline");

        // Check if models exist
        if !self.check_readiness() {
            warn!("Voice models not found - voice features will be limited");
            warn!("Please download models to enable full voice functionality");
        }

        // Set initial state to ListeningForWakeWord
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| format!("Failed to lock state: {}", e))?;
            *state = VoiceState::ListeningForWakeWord;
            info!("Voice state: Idle -> ListeningForWakeWord");
        }

        // Mark wake word as active
        self.wake_word_active.store(true, Ordering::Relaxed);

        // Clone references for the audio thread
        let app_handle = self.app_handle.clone();
        let state = self.state.clone();
        let wake_word_active = self.wake_word_active.clone();
        let recording_buffer = self.recording_buffer.clone();
        let recording_complete = self.recording_complete.clone();
        let skip_frames_counter = self.skip_frames_counter.clone();
        let voice_detected = self.voice_detected.clone();
        let vad_sensitivity = self.vad_sensitivity.clone();
        let vad_timeout_ms = self.vad_timeout_ms.clone();

        // Spawn background audio processing thread
        std::thread::spawn(move || {
            if let Err(e) = Self::run_audio_loop(
                app_handle.clone(),
                state,
                wake_word_active,
                recording_buffer,
                recording_complete,
                skip_frames_counter,
                voice_detected,
                vad_sensitivity,
                vad_timeout_ms,
            ) {
                error!("Audio loop error: {}", e);
            }
        });

        // Emit initial status
        self.emit_status("voice_pipeline", true);

        info!("✓ Native voice pipeline started");
        Ok(())
    }

    /// Main audio processing loop (runs in background thread)
    fn run_audio_loop(
        app_handle: AppHandle,
        state: Arc<Mutex<VoiceState>>,
        wake_word_active: Arc<AtomicBool>,
        recording_buffer: Arc<Mutex<Vec<f32>>>,
        recording_complete: Arc<AtomicBool>,
        skip_frames_counter: Arc<AtomicUsize>,
        voice_detected: Arc<AtomicBool>,
        vad_sensitivity: Arc<Mutex<f32>>,
        vad_timeout_ms: Arc<Mutex<u32>>,
    ) -> Result<(), String> {
        // Initialize audio device
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        info!("Audio device: {}", device.name().unwrap_or_default());

        // Configure audio stream (16kHz mono)
        let config = StreamConfig {
            channels: CHANNELS,
            sample_rate: cpal::SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Fixed(CHUNK_SIZE as u32),
        };

        // Wake word detection state (only used when NOT recording)
        // Use atomics for interior mutability in Fn callback
        let voice_frame_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let last_wake_emission = Arc::new(Mutex::new(std::time::Instant::now()));

        // Recording state (only used when IS recording)
        let silence_frame_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let has_detected_speech = Arc::new(AtomicBool::new(false));
        let recording_frame_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        // Clone for audio callback
        let voice_frame_count_clone = voice_frame_count.clone();
        let last_wake_emission_clone = last_wake_emission.clone();
        let silence_frame_count_clone = silence_frame_count.clone();
        let has_detected_speech_clone = has_detected_speech.clone();
        let recording_frame_count_clone = recording_frame_count.clone();
        let state_clone = state.clone();
        let recording_buffer_clone = recording_buffer.clone();
        let recording_complete_clone = recording_complete.clone();
        let skip_frames_clone = skip_frames_counter.clone();
        let wake_word_clone = wake_word_active.clone();
        let voice_clone = voice_detected.clone();
        let app_clone = app_handle.clone();
        let vad_sensitivity_clone = vad_sensitivity.clone();
        let vad_timeout_ms_clone = vad_timeout_ms.clone();

        // Build audio input stream
        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Check if we should still be processing
                    if !wake_word_clone.load(Ordering::Relaxed) {
                        return;
                    }

                    // Get current state
                    let current_state = state_clone.lock().map(|s| *s).unwrap_or(VoiceState::Idle);

                    // Load current VAD settings
                    let sensitivity = vad_sensitivity_clone.lock().map(|s| *s).unwrap_or(0.02);
                    let timeout_ms = vad_timeout_ms_clone.lock().map(|t| *t).unwrap_or(1280);

                    // Calculate silence_frames from timeout_ms
                    // Each chunk is 512 samples at 16kHz = 32ms
                    let silence_frames = (timeout_ms as f32 / 32.0).round() as usize;

                    // Wake word threshold is 2.5x the sensitivity for more reliable detection
                    let wake_threshold = sensitivity * 2.5;

                    // Calculate energy level of this chunk
                    let energy = calculate_rms_energy(data);

                    // STATE MACHINE: Process audio based on current state
                    match current_state {
                        VoiceState::Speaking => {
                            // ===== SPEAKING MODE =====
                            // Completely ignore audio input to prevent feedback loop
                            // The assistant's TTS output won't trigger the wake word detector
                            return;
                        }
                        VoiceState::Transcribing => {
                            // ===== TRANSCRIBING MODE =====

                            // Skip initial frames after wake word to prevent capturing the wake word itself
                            let skip_count = skip_frames_clone.load(Ordering::Relaxed);
                            if skip_count > 0 {
                                skip_frames_clone.fetch_sub(1, Ordering::Relaxed);
                                return; // Discard this frame
                            }

                            // Increment recording frame counter
                            let frame_count = recording_frame_count_clone.fetch_add(1, Ordering::Relaxed) + 1;

                            // Add all samples to recording buffer
                            if let Ok(mut buffer) = recording_buffer_clone.lock() {
                                buffer.extend_from_slice(data);
                            }

                            // VAD: Detect speech start and end
                            if energy > sensitivity {
                                // Voice detected
                                let was_speech = has_detected_speech_clone.swap(true, Ordering::Relaxed);
                                silence_frame_count_clone.store(0, Ordering::Relaxed);

                                // Log first speech detection
                                if !was_speech {
                                    debug!("Speech detected! (energy: {:.4} > threshold: {:.4})", energy, sensitivity);
                                }
                            } else {
                                // Silence detected
                                if has_detected_speech_clone.load(Ordering::Relaxed) {
                                    // Only allow silence detection after minimum recording duration
                                    if frame_count >= MIN_RECORDING_FRAMES {
                                        let silence_count = silence_frame_count_clone.fetch_add(1, Ordering::Relaxed) + 1;

                                        // Check if we've had enough silence to end recording
                                        if silence_count >= silence_frames {
                                            debug!("Silence detected after speech - ending recording (frames: {}, silence_frames: {})",
                                                   frame_count, silence_count);
                                            recording_complete_clone.store(true, Ordering::Relaxed);

                                            // Reset recording state for next time
                                            has_detected_speech_clone.store(false, Ordering::Relaxed);
                                            silence_frame_count_clone.store(0, Ordering::Relaxed);
                                            recording_frame_count_clone.store(0, Ordering::Relaxed);
                                        }
                                    }
                                }
                            }
                        }
                        VoiceState::ListeningForWakeWord => {
                            // ===== WAKE WORD DETECTION MODE =====
                            // Energy-based wake word detection
                            if energy > wake_threshold {
                                let count = voice_frame_count_clone.fetch_add(1, Ordering::Relaxed) + 1;

                                // Require consistent voice activity to avoid false positives
                                if count >= VOICE_FRAMES_REQUIRED {
                                    // Only emit wake word event every 3 seconds to avoid spam
                                    let now = std::time::Instant::now();
                                    if let Ok(mut last_emission) = last_wake_emission_clone.lock() {
                                        if now.duration_since(*last_emission).as_secs() >= 3 {
                                            debug!("Voice activity detected (energy: {:.4}, threshold: {:.4})", energy, wake_threshold);

                                            // Emit wake word detected event
                                            if let Err(e) = app_clone.emit("wake_word_detected", ()) {
                                                error!("Failed to emit wake_word_detected: {}", e);
                                            }

                                            voice_clone.store(true, Ordering::Relaxed);
                                            *last_emission = now;
                                            voice_frame_count_clone.store(0, Ordering::Relaxed);
                                        }
                                    }
                                }
                            } else if energy < sensitivity {
                                // Reset counter if we detect silence
                                voice_frame_count_clone.fetch_update(
                                    Ordering::Relaxed,
                                    Ordering::Relaxed,
                                    |x| Some(x.saturating_sub(1))
                                ).ok();
                            }
                        }
                        VoiceState::Idle => {
                            // ===== IDLE MODE =====
                            // Do nothing, pipeline is not active
                            return;
                        }
                    }
                },
                move |err| {
                    error!("Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| format!("Failed to build input stream: {}", e))?;

        // Start the audio stream
        stream
            .play()
            .map_err(|e| format!("Failed to play stream: {}", e))?;

        info!("✓ Audio stream started (wake word: energy-based VAD)");

        // Keep the stream alive
        loop {
            if !wake_word_active.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        // Clean shutdown
        stream.pause().ok();
        drop(stream);

        info!("Audio loop stopped");
        Ok(())
    }

    /// Manually trigger transcription (Push-to-Talk or after wake word)
    ///
    /// This method:
    /// 1. Checks if pipeline is in valid state (not already transcribing)
    /// 2. Transitions state to Transcribing
    /// 3. Audio callback handles VAD and buffering in Transcribing state
    /// 4. Waits for recording to complete (silence detected) or timeout
    /// 5. Feeds accumulated audio to whisper-rs
    /// 6. Returns state to ListeningForWakeWord
    /// 7. Returns the transcribed text
    ///
    /// This function is idempotent and thread-safe - it can be safely called
    /// multiple times in a row without state corruption.
    pub fn start_transcription(&self) -> Result<String, String> {
        info!("Transcription triggered");

        if !self.wake_word_active.load(Ordering::Relaxed) {
            return Err("Voice pipeline not active".to_string());
        }

        // Check if configured Whisper model is present
        let whisper_model = self.model_path.join(&self.stt_model_name);
        if !whisper_model.exists() {
            return Err(format!(
                "Whisper model not found: {:?}. Please download {}",
                whisper_model, self.stt_model_name
            ));
        }

        // STATE TRANSITION: Move to Transcribing state (with guard against concurrent calls)
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| format!("Failed to lock state: {}", e))?;

            // Guard: Don't allow transcription if already transcribing
            if *state == VoiceState::Transcribing {
                return Err(
                    "Already transcribing - please wait for current recording to complete"
                        .to_string(),
                );
            }

            let prev_state = *state;
            *state = VoiceState::Transcribing;
            info!("Voice state: {:?} -> Transcribing", prev_state);
        }

        // CRITICAL: Prepare for new recording by resetting ALL state
        // This ensures idempotency - each transcription starts with a clean slate
        info!("Resetting recording state for new transcription...");
        {
            let mut buffer = self.recording_buffer.lock().unwrap();
            buffer.clear();
        }
        self.recording_complete.store(false, Ordering::Relaxed);
        self.voice_detected.store(false, Ordering::Relaxed);

        // Set skip counter to discard initial frames and prevent wake word capture
        self.skip_frames_counter
            .store(SKIP_FRAMES_AFTER_WAKE_WORD, Ordering::Relaxed);
        info!("Recording started - speak now...");
        debug!(
            "Recording config: skip_frames={}, min_frames={}, vad_sensitivity={:.4}, vad_timeout_ms={}",
            SKIP_FRAMES_AFTER_WAKE_WORD,
            MIN_RECORDING_FRAMES,
            *self.vad_sensitivity.lock().unwrap(),
            *self.vad_timeout_ms.lock().unwrap()
        );

        // Wait for recording to complete (VAD detects end of speech) or timeout
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(MAX_RECORDING_SECONDS as u64);

        loop {
            // Check if recording completed (silence detected by audio callback)
            if self.recording_complete.load(Ordering::Relaxed) {
                info!("Recording completed (silence detected)");
                break;
            }

            // Check for timeout
            if start_time.elapsed() > timeout {
                warn!("Recording timeout after {} seconds", MAX_RECORDING_SECONDS);
                break;
            }

            // Small sleep to avoid busy loop
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // STATE TRANSITION: Return to ListeningForWakeWord
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| format!("Failed to lock state: {}", e))?;
            *state = VoiceState::ListeningForWakeWord;
            info!("Voice state: Transcribing -> ListeningForWakeWord");
        }

        // Extract recorded audio
        let recording_samples: Vec<f32> = {
            let buffer = self.recording_buffer.lock().unwrap();
            buffer.clone()
        };

        let duration_seconds = recording_samples.len() as f32 / SAMPLE_RATE as f32;
        info!(
            "Captured {} samples ({:.2}s of audio)",
            recording_samples.len(),
            duration_seconds
        );

        // Check if we got any audio
        if recording_samples.is_empty() {
            // Cleanup before returning error
            warn!("No audio captured - microphone may not be working or lacks permissions");
            {
                let mut buffer = self.recording_buffer.lock().unwrap();
                buffer.clear();
            }
            self.recording_complete.store(false, Ordering::Relaxed);
            self.voice_detected.store(false, Ordering::Relaxed);
            self.skip_frames_counter.store(0, Ordering::Relaxed);

            return Err("No audio captured. Please check:\n1. Microphone permissions\n2. Microphone is connected and working\n3. Correct input device is selected".to_string());
        }

        // Check if recording is too short to be useful
        if duration_seconds < 0.5 {
            warn!(
                "Recording too short ({:.2}s) - likely no speech detected. Consider adjusting VAD sensitivity.",
                duration_seconds
            );
            // Continue anyway in case there's actual speech
        }

        // Calculate average energy to diagnose audio issues
        let avg_energy = calculate_rms_energy(&recording_samples);
        let vad_sens = self.vad_sensitivity.lock().unwrap();
        debug!(
            "Audio analysis: avg_energy={:.4}, vad_threshold={:.4}, ratio={:.2}x",
            avg_energy,
            *vad_sens,
            avg_energy / *vad_sens
        );

        // Transcribe using whisper-rs (with automatic cleanup via defer-like pattern)
        let transcription_result = self.transcribe_with_whisper(&recording_samples, &whisper_model);

        // CRITICAL: Full cleanup after transcription (whether it succeeded or failed)
        // This ensures the pipeline is ready for the next transcription
        info!("Performing final cleanup after transcription...");
        {
            let mut buffer = self.recording_buffer.lock().unwrap();
            buffer.clear();
        }
        self.recording_complete.store(false, Ordering::Relaxed);
        self.voice_detected.store(false, Ordering::Relaxed);
        self.skip_frames_counter.store(0, Ordering::Relaxed);

        info!("Pipeline reset complete - ready for next transcription");

        // Return the transcription result (propagate any errors)
        transcription_result
    }

    /// Transcribe audio samples using Whisper
    fn transcribe_with_whisper(
        &self,
        samples: &[f32],
        model_path: &PathBuf,
    ) -> Result<String, String> {
        info!("Initializing Whisper for transcription...");

        // Load Whisper model
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().unwrap(),
            WhisperContextParameters::default(),
        )
        .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

        // Configure transcription parameters
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Set language to English
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Create Whisper state
        let mut state = ctx
            .create_state()
            .map_err(|e| format!("Failed to create Whisper state: {}", e))?;

        // Run transcription
        info!("Running Whisper transcription...");
        state
            .full(params, samples)
            .map_err(|e| format!("Whisper transcription failed: {}", e))?;

        // Extract transcribed text
        let num_segments = state.full_n_segments();

        let mut transcription = String::new();
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                let text = segment
                    .to_str()
                    .map_err(|e| format!("Failed to get segment {} text: {}", i, e))?;
                transcription.push_str(text);
                transcription.push(' ');
            }
        }

        let transcription = transcription.trim().to_string();

        if transcription.is_empty() {
            info!("Whisper returned empty transcription - no speech detected");
            return Err("No speech detected in audio".to_string());
        }

        info!("Transcription complete: '{}'", transcription);
        Ok(transcription)
    }

    /// Update VAD settings in real-time
    ///
    /// This allows the user to tune the microphone sensitivity and silence timeout
    /// without restarting the application
    pub fn update_vad_settings(&self, sensitivity: f32, timeout_ms: u32) -> Result<(), String> {
        info!(
            "Updating VAD settings: sensitivity={}, timeout_ms={}",
            sensitivity, timeout_ms
        );

        // Validate sensitivity range
        if !(0.001..=1.0).contains(&sensitivity) {
            return Err(format!(
                "Invalid sensitivity: {}. Must be between 0.001 and 1.0",
                sensitivity
            ));
        }

        // Validate timeout range (minimum 100ms, maximum 10 seconds)
        if !(100..=10000).contains(&timeout_ms) {
            return Err(format!(
                "Invalid timeout: {}ms. Must be between 100ms and 10000ms",
                timeout_ms
            ));
        }

        // Update settings
        {
            let mut sens = self
                .vad_sensitivity
                .lock()
                .map_err(|e| format!("Failed to lock vad_sensitivity: {}", e))?;
            *sens = sensitivity;
        }

        {
            let mut timeout = self
                .vad_timeout_ms
                .lock()
                .map_err(|e| format!("Failed to lock vad_timeout_ms: {}", e))?;
            *timeout = timeout_ms;
        }

        info!("✓ VAD settings updated successfully");
        Ok(())
    }

    /// Set the voice pipeline state
    ///
    /// This is used to control the state machine from external commands.
    /// Key use case: Setting to Speaking when TTS starts to prevent feedback loop.
    pub fn set_state(&self, new_state: VoiceState) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| format!("Failed to lock state: {}", e))?;

        let prev_state = *state;
        *state = new_state;
        info!("Voice state: {:?} -> {:?}", prev_state, new_state);

        Ok(())
    }

    /// Get the current voice pipeline state
    pub fn get_state(&self) -> Result<VoiceState, String> {
        let state = self
            .state
            .lock()
            .map_err(|e| format!("Failed to lock state: {}", e))?;
        Ok(*state)
    }

    /// Cancel the current operation and reset to clean state
    ///
    /// This forcefully resets the state machine, ensuring the pipeline is
    /// immediately ready for the next operation without any lingering state issues.
    pub fn cancel_and_reset(&self) -> Result<(), String> {
        info!("Cancelling voice operation and resetting state");

        // Reset recording signals
        self.recording_complete.store(false, Ordering::Relaxed);
        self.voice_detected.store(false, Ordering::Relaxed);
        self.skip_frames_counter.store(0, Ordering::Relaxed);

        // Clear recording buffer
        {
            let mut buffer = self
                .recording_buffer
                .lock()
                .map_err(|e| format!("Failed to lock recording buffer: {}", e))?;
            buffer.clear();
        }

        // STATE TRANSITION: Force reset to ListeningForWakeWord
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| format!("Failed to lock state: {}", e))?;
            let prev_state = *state;
            *state = VoiceState::ListeningForWakeWord;
            info!(
                "Voice state: {:?} -> ListeningForWakeWord (forced reset)",
                prev_state
            );
        }

        info!("✓ Voice pipeline reset complete - ready for next operation");
        Ok(())
    }

    /// Stop the voice pipeline
    pub fn stop(&self) {
        info!("Stopping native voice pipeline");
        self.wake_word_active.store(false, Ordering::Relaxed);

        // Transition to Idle state
        if let Ok(mut state) = self.state.lock() {
            *state = VoiceState::Idle;
            info!("Voice state: -> Idle (stopped)");
        }

        self.emit_status("voice_pipeline", false);
    }

    /// Check if the voice pipeline is ready
    pub fn check_readiness(&self) -> bool {
        // Check if configured whisper model exists
        // Wake word uses energy-based VAD (no model needed)
        let whisper_model_path = self.model_path.join(&self.stt_model_name);
        let whisper_exists = whisper_model_path.exists();

        if !whisper_exists {
            warn!("Whisper model not found: {:?}", whisper_model_path);
        }

        whisper_exists
    }

    /// Get the full path to the configured STT model
    pub fn get_stt_model_path(&self) -> PathBuf {
        self.model_path.join(&self.stt_model_name)
    }

    /// Emit service status to frontend
    fn emit_status(&self, service: &str, connected: bool) {
        let status = ServiceStatus {
            service: service.to_string(),
            connected,
        };

        if let Err(e) = self.app_handle.emit("service_status", status) {
            error!("Failed to emit service status: {}", e);
        }
    }
}

impl Drop for NativeVoicePipeline {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Calculate RMS energy of audio samples
fn calculate_rms_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_squares / samples.len() as f32).sqrt()
}
