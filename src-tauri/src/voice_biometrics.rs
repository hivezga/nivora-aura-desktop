/// Voice Biometrics Module - Speaker Recognition with sherpa-rs Integration
///
/// Provides speaker enrollment and identification using voice embeddings.
/// Uses sherpa-rs with WeSpeaker ECAPA-TDNN model for real-time speaker recognition.

use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;
use crate::database::Database;
use sherpa_rs::speaker_id::{EmbeddingExtractor, ExtractorConfig};
use std::path::PathBuf;

/// Standard embedding dimension for WeSpeaker ECAPA-TDNN model
const EMBEDDING_DIM: usize = 192;

/// Similarity threshold for speaker recognition (cosine similarity)
/// Values above this threshold indicate a match
const RECOGNITION_THRESHOLD: f32 = 0.70;

/// Maximum variance allowed during enrollment
/// Ensures consistent voice samples
const ENROLLMENT_VARIANCE_THRESHOLD: f32 = 0.15;

/// User profile with voice biometric data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserProfile {
    pub id: i64,
    pub name: String,
    #[serde(skip)]  // Don't serialize embedding in JSON responses
    pub voice_print_embedding: Vec<f32>,
    pub enrollment_date: String,
    pub last_recognized: Option<String>,
    pub recognition_count: i64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Voice biometrics error types
#[derive(Debug, thiserror::Error)]
pub enum BiometricsError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Insufficient voice samples (need at least 3, got {0})")]
    InsufficientSamples(usize),

    #[error("Inconsistent voice samples (variance: {0:.3}, threshold: {1:.3})")]
    InconsistentSamples(f32, f32),

    #[error("Invalid embedding dimension (expected {0}, got {1})")]
    InvalidEmbeddingDim(usize, usize),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("User profile already exists: {0}")]
    DuplicateUser(String),

    #[error("Model not loaded")]
    ModelNotLoaded,

    #[error("Audio processing error: {0}")]
    AudioProcessing(String),
}

/// Voice biometrics engine for speaker recognition
pub struct VoiceBiometrics {
    database: Arc<Mutex<Database>>,
    speaker_model: Arc<Mutex<Option<EmbeddingExtractor>>>,
    model_path: PathBuf,
}

impl VoiceBiometrics {
    /// Create a new voice biometrics engine
    pub fn new(database: Arc<Mutex<Database>>, model_path: PathBuf) -> Self {
        Self {
            database,
            speaker_model: Arc::new(Mutex::new(None)),
            model_path,
        }
    }

    /// Initialize the speaker embedding model
    pub async fn initialize_model(&self) -> Result<(), BiometricsError> {
        let model_file = self.model_path.join("wespeaker_en_voxceleb_CAM++.onnx");
        
        if !model_file.exists() {
            log::error!("Speaker model not found at: {:?}", model_file);
            log::info!("Please download the model using:");
            log::info!("wget https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recongition-models/wespeaker_en_voxceleb_CAM++.onnx -O {:?}", model_file);
            return Err(BiometricsError::ModelNotLoaded);
        }

        log::info!("Loading WeSpeaker ECAPA-TDNN model from: {:?}", model_file);

        let config = ExtractorConfig {
            model: model_file.to_string_lossy().to_string(),
            provider: Some("cpu".to_string()),
            num_threads: Some(1),
            debug: false,
        };

        match EmbeddingExtractor::new(config) {
            Ok(extractor) => {
                log::info!("✓ WeSpeaker ECAPA-TDNN model loaded successfully");
                log::info!("  - Embedding dimension: {}", extractor.embedding_size);
                
                // Verify embedding dimension matches expected
                if extractor.embedding_size != EMBEDDING_DIM {
                    log::warn!("Model embedding dimension ({}) differs from expected ({})", 
                              extractor.embedding_size, EMBEDDING_DIM);
                }
                
                let mut model_lock = self.speaker_model.lock().await;
                *model_lock = Some(extractor);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to load speaker model: {:?}", e);
                Err(BiometricsError::ModelNotLoaded)
            }
        }
    }

    /// Check if the model is loaded and ready
    pub async fn is_model_loaded(&self) -> bool {
        self.speaker_model.lock().await.is_some()
    }

    /// Enroll a new user with voice samples
    ///
    /// # Arguments
    /// * `user_name` - Name of the user to enroll
    /// * `audio_samples` - 3-5 audio recordings (PCM f32 samples at 16kHz)
    ///
    /// # Returns
    /// User ID of the newly enrolled profile
    pub async fn enroll_user(
        &self,
        user_name: String,
        audio_samples: Vec<Vec<f32>>,
    ) -> Result<i64, BiometricsError> {
        // Validate input
        if audio_samples.len() < 3 {
            return Err(BiometricsError::InsufficientSamples(audio_samples.len()));
        }

        // Ensure model is loaded
        if !self.is_model_loaded().await {
            return Err(BiometricsError::ModelNotLoaded);
        }

        // Extract embeddings from each sample using the real model
        let mut embeddings = Vec::new();
        for (i, audio) in audio_samples.iter().enumerate() {
            log::debug!("Extracting embedding for sample {} ({} samples)", i + 1, audio.len());
            let embedding = self.extract_embedding(audio).await?;
            embeddings.push(embedding);
        }

        // Average embeddings to create robust voice print
        let voice_print = Self::average_embeddings(&embeddings);

        // Validate enrollment quality
        let variance = Self::calculate_embedding_variance(&embeddings);
        if variance > ENROLLMENT_VARIANCE_THRESHOLD {
            return Err(BiometricsError::InconsistentSamples(
                variance,
                ENROLLMENT_VARIANCE_THRESHOLD,
            ));
        }

        // Store in database
        let user_id = self.create_user_profile(&user_name, &voice_print).await?;

        log::info!("✓ User '{}' enrolled successfully (ID: {}, variance: {:.3})",
                   user_name, user_id, variance);

        Ok(user_id)
    }

    /// Identify speaker from audio sample
    ///
    /// # Arguments
    /// * `audio` - Audio recording (PCM f32 samples at 16kHz)
    ///
    /// # Returns
    /// Matched user profile if similarity exceeds threshold, None otherwise
    pub async fn identify_speaker(
        &self,
        audio: &[f32],
    ) -> Result<Option<UserProfile>, BiometricsError> {
        // Ensure model is loaded
        if !self.is_model_loaded().await {
            return Err(BiometricsError::ModelNotLoaded);
        }

        // Extract embedding from audio using the real model
        let query_embedding = self.extract_embedding(audio).await?;

        // Load all active user profiles
        let profiles = self.get_active_user_profiles().await?;

        if profiles.is_empty() {
            return Ok(None);  // No enrolled users
        }

        // Compare with stored voice prints using cosine similarity
        let mut best_match: Option<UserProfile> = None;
        let mut best_similarity = 0.0;

        for profile in profiles {
            let similarity = Self::cosine_similarity(&query_embedding, &profile.voice_print_embedding);

            log::debug!("User '{}': similarity = {:.3}", profile.name, similarity);

            if similarity > best_similarity {
                best_similarity = similarity;
                best_match = Some(profile);
            }
        }

        // Threshold-based matching
        if best_similarity >= RECOGNITION_THRESHOLD {
            if let Some(mut profile) = best_match {
                // Update recognition stats
                self.increment_recognition_count(profile.id).await?;
                profile.recognition_count += 1;
                profile.last_recognized = Some(Utc::now().to_rfc3339());

                log::info!("✓ Speaker identified: {} (similarity: {:.3})",
                           profile.name, best_similarity);

                Ok(Some(profile))
            } else {
                Ok(None)
            }
        } else {
            log::debug!("No confident match (best similarity: {:.3} < threshold: {:.3})",
                       best_similarity, RECOGNITION_THRESHOLD);
            Ok(None)
        }
    }

    /// Extract speaker embedding from audio using WeSpeaker ECAPA-TDNN model
    ///
    /// # Arguments
    /// * `audio` - Audio samples (PCM f32, 16kHz mono)
    ///
    /// # Returns
    /// 192-dimensional embedding vector representing speaker characteristics
    async fn extract_embedding(&self, audio: &[f32]) -> Result<Vec<f32>, BiometricsError> {
        let mut model_lock = self.speaker_model.lock().await;
        let model = model_lock.as_mut()
            .ok_or(BiometricsError::ModelNotLoaded)?;

        // The sherpa-rs API expects 16kHz mono audio
        let sample_rate = 16000;
        
        log::debug!("Extracting embedding from {} samples ({:.2}s of audio)", 
                   audio.len(), audio.len() as f32 / sample_rate as f32);

        // Create embedding using sherpa-rs
        match model.compute_speaker_embedding(audio.to_vec(), sample_rate) {
            Ok(embedding) => {
                // Verify embedding dimension
                if embedding.len() != model.embedding_size {
                    return Err(BiometricsError::InvalidEmbeddingDim(
                        model.embedding_size,
                        embedding.len(),
                    ));
                }

                log::debug!("✓ Extracted {}-dimensional embedding", embedding.len());
                Ok(embedding)
            }
            Err(e) => {
                log::error!("Failed to compute embedding: {:?}", e);
                Err(BiometricsError::AudioProcessing(format!("Embedding computation failed: {:?}", e)))
            }
        }
    }

    /// Average multiple embeddings to create a robust voice print
    fn average_embeddings(embeddings: &[Vec<f32>]) -> Vec<f32> {
        let dim = embeddings[0].len();
        let mut avg = vec![0.0; dim];

        for emb in embeddings {
            for (i, val) in emb.iter().enumerate() {
                avg[i] += val;
            }
        }

        for val in &mut avg {
            *val /= embeddings.len() as f32;
        }

        // Normalize the averaged embedding
        Self::normalize_embedding(&mut avg);

        avg
    }

    /// Calculate variance of embeddings (quality check)
    fn calculate_embedding_variance(embeddings: &[Vec<f32>]) -> f32 {
        if embeddings.len() < 2 {
            return 0.0;
        }

        let avg = Self::average_embeddings(embeddings);
        let mut total_variance = 0.0;

        for emb in embeddings {
            let diff: f32 = emb
                .iter()
                .zip(avg.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum();
            total_variance += diff;
        }

        (total_variance / embeddings.len() as f32).sqrt()
    }

    /// Compute cosine similarity between two embeddings
    ///
    /// Returns value in range [-1, 1], where 1 means identical
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len(), "Embeddings must have same dimension");

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Normalize embedding to unit vector (L2 normalization)
    fn normalize_embedding(embedding: &mut [f32]) {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in embedding.iter_mut() {
                *val /= norm;
            }
        }
    }

    /// Serialize embedding to BLOB for database storage
    pub fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
        embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect()
    }

    /// Deserialize embedding from database BLOB
    pub fn deserialize_embedding(blob: &[u8]) -> Result<Vec<f32>, BiometricsError> {
        if blob.len() != EMBEDDING_DIM * 4 {
            return Err(BiometricsError::InvalidEmbeddingDim(
                EMBEDDING_DIM * 4,
                blob.len(),
            ));
        }

        let mut embedding = Vec::with_capacity(EMBEDDING_DIM);
        for chunk in blob.chunks_exact(4) {
            let bytes: [u8; 4] = chunk.try_into().unwrap();
            embedding.push(f32::from_le_bytes(bytes));
        }
        Ok(embedding)
    }

    // ========================================================================
    // Database Operations
    // ========================================================================

    /// Create a new user profile in the database
    async fn create_user_profile(
        &self,
        user_name: &str,
        voice_print: &[f32],
    ) -> Result<i64, BiometricsError> {
        let db = self.database.lock().await;
        let now = Utc::now().to_rfc3339();
        let embedding_blob = Self::serialize_embedding(voice_print);

        let user_id = db.execute_and_get_last_id(
            "INSERT INTO user_profiles
             (name, voice_print_embedding, enrollment_date, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            &[
                &user_name as &dyn rusqlite::ToSql,
                &embedding_blob,
                &now,
                &true,
                &now,
                &now,
            ],
        )
        .map_err(|e| BiometricsError::Database(e))?;

        Ok(user_id)
    }

    /// Get all active user profiles
    async fn get_active_user_profiles(&self) -> Result<Vec<UserProfile>, BiometricsError> {
        let db = self.database.lock().await;

        let profiles = db.query_rows(
            "SELECT id, name, voice_print_embedding, enrollment_date, last_recognized,
                    recognition_count, is_active, created_at, updated_at
             FROM user_profiles
             WHERE is_active = 1",
            &[],
            |row| {
                let embedding_blob: Vec<u8> = row.get(2)?;
                let voice_print = Self::deserialize_embedding(&embedding_blob)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok(UserProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    voice_print_embedding: voice_print,
                    enrollment_date: row.get(3)?,
                    last_recognized: row.get(4)?,
                    recognition_count: row.get(5)?,
                    is_active: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            }
        )
        .map_err(|e| BiometricsError::Database(e))?;

        Ok(profiles)
    }

    /// Increment recognition count for a user
    async fn increment_recognition_count(&self, user_id: i64) -> Result<(), BiometricsError> {
        let db = self.database.lock().await;
        let now = Utc::now().to_rfc3339();

        db.execute_query(
            "UPDATE user_profiles
             SET recognition_count = recognition_count + 1,
                 last_recognized = ?1,
                 updated_at = ?2
             WHERE id = ?3",
            &[&now as &dyn rusqlite::ToSql, &now, &user_id],
        )
        .map_err(|e| BiometricsError::Database(e))?;

        Ok(())
    }

    /// Delete a user profile
    pub async fn delete_user_profile(&self, user_id: i64) -> Result<(), BiometricsError> {
        let db = self.database.lock().await;

        db.execute_query("DELETE FROM user_profiles WHERE id = ?1", &[&user_id])
            .map_err(|e| BiometricsError::Database(e))?;

        log::info!("✓ User profile deleted (ID: {})", user_id);
        Ok(())
    }

    /// Get user profile by ID
    pub async fn get_user_profile(&self, user_id: i64) -> Result<Option<UserProfile>, BiometricsError> {
        let profiles = self.get_active_user_profiles().await?;
        Ok(profiles.into_iter().find(|p| p.id == user_id))
    }

    /// List all enrolled users (for UI display)
    pub async fn list_all_users(&self) -> Result<Vec<UserProfile>, BiometricsError> {
        self.get_active_user_profiles().await
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        // Identical vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((VoiceBiometrics::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        // Orthogonal vectors
        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!((VoiceBiometrics::cosine_similarity(&c, &d) - 0.0).abs() < 0.001);

        // Opposite vectors
        let e = vec![1.0, 0.0, 0.0];
        let f = vec![-1.0, 0.0, 0.0];
        assert!((VoiceBiometrics::cosine_similarity(&e, &f) + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_embedding_serialization() {
        let embedding = vec![0.5f32; EMBEDDING_DIM];
        let blob = VoiceBiometrics::serialize_embedding(&embedding);
        assert_eq!(blob.len(), EMBEDDING_DIM * 4);

        let recovered = VoiceBiometrics::deserialize_embedding(&blob).unwrap();
        assert_eq!(recovered.len(), EMBEDDING_DIM);
        assert!((recovered[0] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_normalize_embedding() {
        let mut embedding = vec![3.0, 4.0, 0.0];
        VoiceBiometrics::normalize_embedding(&mut embedding);

        // Check magnitude is 1.0
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_average_embeddings() {
        let emb1 = vec![1.0, 0.0, 0.0];
        let emb2 = vec![0.0, 1.0, 0.0];
        let emb3 = vec![0.0, 0.0, 1.0];

        let avg = VoiceBiometrics::average_embeddings(&[emb1, emb2, emb3]);

        // Average should be normalized
        let norm: f32 = avg.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }
}
