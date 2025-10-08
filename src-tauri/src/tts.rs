use rodio::{OutputStream, Sink};
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Text-to-Speech engine for Aura using subprocess-based Piper TTS
///
/// This module handles converting text responses to spoken audio using
/// the high-quality Piper neural TTS engine via subprocess execution.
///
/// Architecture:
/// - Spawns piper executable as child process
/// - Pipes text to stdin
/// - Captures raw PCM audio from stdout
/// - Converts PCM to WAV format in-memory using hound
/// - Plays audio through speakers using rodio
/// - 100% offline, stable subprocess architecture
pub struct TextToSpeech {
    piper_path: PathBuf,
    model_path: PathBuf,
    espeak_data_path: PathBuf,
}

impl TextToSpeech {
    /// Create a new TextToSpeech instance with subprocess-based Piper integration
    ///
    /// # Arguments
    /// * `piper_path` - Path to the piper executable binary
    /// * `model_path` - Path to the Piper voice model (.onnx file)
    /// * `espeak_data_path` - Path to the espeak-ng data directory
    ///
    /// # Returns
    /// A configured TTS engine ready for speech synthesis
    ///
    /// # Errors
    /// Returns error if:
    /// - Piper binary doesn't exist
    /// - Model file doesn't exist
    /// - Model config (.json) doesn't exist
    /// - eSpeak-NG data directory doesn't exist
    pub fn new(piper_path: PathBuf, model_path: PathBuf, espeak_data_path: PathBuf) -> Result<Self, String> {
        log::info!("Initializing subprocess-based Piper TTS engine...");
        log::info!("  Piper binary: {:?}", piper_path);
        log::info!("  Voice model: {:?}", model_path);

        // Verify piper binary exists
        if !piper_path.exists() {
            return Err(format!(
                "Piper binary not found at: {:?}. Please install Piper TTS.",
                piper_path
            ));
        }

        // Verify model exists
        if !model_path.exists() {
            return Err(format!(
                "Piper voice model not found at: {:?}. Please download a voice model.",
                model_path
            ));
        }

        // Verify model has .onnx extension
        if model_path.extension().and_then(|s| s.to_str()) != Some("onnx") {
            return Err(format!(
                "Invalid model file: {:?}. Expected .onnx file.",
                model_path
            ));
        }

        // Construct config path (model_path with .onnx.json extension)
        let config_path = model_path.with_extension("onnx.json");
        if !config_path.exists() {
            return Err(format!(
                "Piper model config not found at: {:?}. Please download the .onnx.json file.",
                config_path
            ));
        }

        // Verify espeak-ng data directory exists
        if !espeak_data_path.exists() {
            return Err(format!(
                "eSpeak-NG data directory not found at: {:?}",
                espeak_data_path
            ));
        }

        log::info!("  Config file: {:?}", config_path);
        log::info!("  eSpeak-NG data: {:?}", espeak_data_path);
        log::info!("✓ Subprocess-based Piper TTS engine initialized successfully");
        log::info!("  - Using piper binary for synthesis");
        log::info!("  - 100% offline, stable subprocess architecture");

        Ok(TextToSpeech {
            piper_path,
            model_path,
            espeak_data_path,
        })
    }

    /// Speak the given text using subprocess-based Piper synthesis
    ///
    /// This spawns the piper executable, pipes the text to stdin,
    /// captures raw PCM audio from stdout, converts to WAV format,
    /// and plays through the default audio device.
    ///
    /// # Arguments
    /// * `text` - The text to speak
    ///
    /// # Returns
    /// Ok(()) if synthesis and playback succeeded, Err with details if failed
    pub fn speak(&mut self, text: &str) -> Result<(), String> {
        if text.is_empty() {
            return Err("Cannot speak empty text".to_string());
        }

        log::info!(
            "Synthesizing speech: '{}' ({} chars)",
            if text.len() > 50 {
                format!("{}...", &text[..50])
            } else {
                text.to_string()
            },
            text.len()
        );

        // Spawn piper subprocess
        log::debug!("Spawning piper subprocess...");
        let mut child = Command::new(&self.piper_path)
            .arg("--model")
            .arg(&self.model_path)
            .arg("--output-raw")  // Output raw PCM instead of WAV
            .arg("--espeak_data")
            .arg(&self.espeak_data_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn piper process: {}", e))?;

        // Write text to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| format!("Failed to write text to piper stdin: {}", e))?;
            // Drop stdin to signal EOF
            drop(stdin);
        } else {
            return Err("Failed to open piper stdin".to_string());
        }

        // Wait for process to complete and capture output
        log::debug!("Waiting for piper synthesis to complete...");
        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait for piper process: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!("Piper process failed with exit code: {:?}", output.status.code());
            log::error!("Piper stderr: {}", stderr);
            return Err(format!("Piper process failed: {}", stderr));
        }

        log::debug!("Piper process completed successfully");

        // Raw PCM data from piper
        let pcm_data = output.stdout;
        if pcm_data.is_empty() {
            return Err("Piper produced no audio (empty output)".to_string());
        }

        log::debug!(
            "Synthesis complete: {} bytes of raw PCM data generated",
            pcm_data.len()
        );

        // Convert raw PCM to i16 samples
        // Piper outputs 16-bit signed PCM, so we need to convert bytes to i16
        let mut samples = Vec::with_capacity(pcm_data.len() / 2);
        for chunk in pcm_data.chunks_exact(2) {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            samples.push(sample);
        }

        // Create in-memory WAV file using hound
        log::debug!("Converting PCM to WAV format...");
        let wav_data = self.create_wav(&samples)?;

        // Play the synthesized audio
        self.play_audio(&wav_data)?;

        log::info!("Finished speaking");
        Ok(())
    }

    /// Convert PCM samples to WAV format in-memory
    ///
    /// # Arguments
    /// * `samples` - 16-bit PCM audio samples
    ///
    /// # Returns
    /// In-memory WAV file as Vec<u8>
    fn create_wav(&self, samples: &[i16]) -> Result<Vec<u8>, String> {
        let mut wav_buffer = Cursor::new(Vec::new());

        // Piper typically outputs at 22050 Hz, mono, 16-bit
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 22050,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::new(&mut wav_buffer, spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

        for &sample in samples {
            writer
                .write_sample(sample)
                .map_err(|e| format!("Failed to write WAV sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {}", e))?;

        Ok(wav_buffer.into_inner())
    }

    /// Play WAV audio data through the default audio device
    ///
    /// # Arguments
    /// * `wav_data` - In-memory WAV file data
    ///
    /// # Returns
    /// Ok(()) if playback succeeded, Err with details if failed
    fn play_audio(&self, wav_data: &[u8]) -> Result<(), String> {
        log::debug!("Initializing audio playback...");

        // Get default audio output device
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to open audio output device: {}", e))?;

        // Create audio sink for playback
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("Failed to create audio sink: {}", e))?;

        // Decode WAV from memory (clone data to give it 'static lifetime)
        let cursor = Cursor::new(wav_data.to_vec());
        let source = rodio::Decoder::new(cursor)
            .map_err(|e| format!("Failed to decode WAV audio: {}", e))?;

        log::debug!("Audio config: playing WAV file");

        // Play and wait for completion
        sink.append(source);
        sink.sleep_until_end();

        log::debug!("Audio playback complete");
        Ok(())
    }

    /// Get information about the loaded voice model
    pub fn model_info(&self) -> String {
        format!(
            "Subprocess Piper TTS: binary={:?}, model={:?}",
            self.piper_path, self.model_path
        )
    }
}
