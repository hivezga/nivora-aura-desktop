use rusqlite::{Connection, Result as SqlResult, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Represents a conversation in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: i64,
    pub title: String,
    pub created_at: String,
}

/// Represents a message in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: String,
}

/// Represents application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub llm_provider: String,      // "local" or "api" (kept for backward compatibility)
    pub server_address: String,    // Remote server address for gRPC (legacy field)
    pub wake_word_enabled: bool,   // Enable/disable wake word detection
    pub api_base_url: String,      // Base URL for OpenAI-compatible API (e.g., "http://localhost:1234/v1")
    pub model_name: String,        // Model name to use (e.g., "llama3", "phi3:instruct")
    pub vad_sensitivity: f32,      // Voice activity detection sensitivity (RMS energy threshold, 0.0-1.0)
    pub vad_timeout_ms: u32,       // Silence timeout in milliseconds before ending recording
    pub stt_model_name: String,    // STT (Whisper) model filename (e.g., "ggml-base.en.bin", "ggml-small.en.bin")
    pub voice_preference: String,  // TTS voice preference ("male" or "female", maps to lessac-medium or amy-medium)
}

/// Database manager for Aura Desktop
///
/// Handles all SQLite operations for conversation and message persistence
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection and initialize tables
    pub fn new(db_path: PathBuf) -> Result<Self, String> {
        log::info!("Initializing database at: {}", db_path.display());

        // Create parent directories if they don't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database directory: {}", e))?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let db = Database { conn };

        // Initialize tables
        db.init_tables()?;

        log::info!("Database initialized successfully");

        Ok(db)
    }

    /// Create database tables if they don't exist
    fn init_tables(&self) -> Result<(), String> {
        // Create conversations table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS conversations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )
            .map_err(|e| format!("Failed to create conversations table: {}", e))?;

        // Create messages table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS messages (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    conversation_id INTEGER NOT NULL,
                    role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
                    content TEXT NOT NULL,
                    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
                )",
                [],
            )
            .map_err(|e| format!("Failed to create messages table: {}", e))?;

        // Create index on conversation_id for faster message lookups
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_messages_conversation_id
                 ON messages(conversation_id)",
                [],
            )
            .map_err(|e| format!("Failed to create index: {}", e))?;

        // Create settings table (key-value store)
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                )",
                [],
            )
            .map_err(|e| format!("Failed to create settings table: {}", e))?;

        // Insert default settings if they don't exist
        self.conn
            .execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('llm_provider', 'local')",
                [],
            )
            .map_err(|e| format!("Failed to insert default llm_provider: {}", e))?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('server_address', '')",
                [],
            )
            .map_err(|e| format!("Failed to insert default server_address: {}", e))?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('wake_word_enabled', 'false')",
                [],
            )
            .map_err(|e| format!("Failed to insert default wake_word_enabled: {}", e))?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('api_base_url', 'http://localhost:11434/v1')",
                [],
            )
            .map_err(|e| format!("Failed to insert default api_base_url: {}", e))?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('model_name', 'gemma:2b')",
                [],
            )
            .map_err(|e| format!("Failed to insert default model_name: {}", e))?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('vad_sensitivity', '0.02')",
                [],
            )
            .map_err(|e| format!("Failed to insert default vad_sensitivity: {}", e))?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('vad_timeout_ms', '1280')",
                [],
            )
            .map_err(|e| format!("Failed to insert default vad_timeout_ms: {}", e))?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('stt_model_name', 'ggml-tiny.bin')",
                [],
            )
            .map_err(|e| format!("Failed to insert default stt_model_name: {}", e))?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('voice_preference', 'male')",
                [],
            )
            .map_err(|e| format!("Failed to insert default voice_preference: {}", e))?;

        // First-run wizard completion flag
        self.conn
            .execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('first_run_complete', 'false')",
                [],
            )
            .map_err(|e| format!("Failed to insert default first_run_complete: {}", e))?;

        log::info!("Database tables initialized");

        Ok(())
    }

    /// Load all conversations from the database
    pub fn load_conversations(&self) -> Result<Vec<Conversation>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, created_at FROM conversations ORDER BY created_at DESC")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let conversations = stmt
            .query_map([], |row| {
                Ok(Conversation {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })
            .map_err(|e| format!("Failed to query conversations: {}", e))?
            .collect::<SqlResult<Vec<_>>>()
            .map_err(|e| format!("Failed to collect conversations: {}", e))?;

        log::info!("Loaded {} conversations", conversations.len());

        Ok(conversations)
    }

    /// Load all messages for a specific conversation
    pub fn load_messages(&self, conversation_id: i64) -> Result<Vec<Message>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, conversation_id, role, content, timestamp
                 FROM messages
                 WHERE conversation_id = ?
                 ORDER BY timestamp ASC",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let messages = stmt
            .query_map(params![conversation_id], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    timestamp: row.get(4)?,
                })
            })
            .map_err(|e| format!("Failed to query messages: {}", e))?
            .collect::<SqlResult<Vec<_>>>()
            .map_err(|e| format!("Failed to collect messages: {}", e))?;

        log::info!(
            "Loaded {} messages for conversation {}",
            messages.len(),
            conversation_id
        );

        Ok(messages)
    }

    /// Create a new conversation
    pub fn create_conversation(&self, title: Option<String>) -> Result<i64, String> {
        let title = title.unwrap_or_else(|| {
            let now = chrono::Local::now();
            format!("New Chat - {}", now.format("%b %d, %H:%M"))
        });

        self.conn
            .execute(
                "INSERT INTO conversations (title) VALUES (?1)",
                params![title],
            )
            .map_err(|e| format!("Failed to create conversation: {}", e))?;

        let id = self.conn.last_insert_rowid();

        log::info!("Created new conversation: {} (id: {})", title, id);

        Ok(id)
    }

    /// Save a message to the database
    pub fn save_message(
        &self,
        conversation_id: i64,
        role: &str,
        content: &str,
    ) -> Result<i64, String> {
        // Validate role
        if role != "user" && role != "assistant" {
            return Err(format!("Invalid role: {}. Must be 'user' or 'assistant'", role));
        }

        self.conn
            .execute(
                "INSERT INTO messages (conversation_id, role, content) VALUES (?1, ?2, ?3)",
                params![conversation_id, role, content],
            )
            .map_err(|e| format!("Failed to save message: {}", e))?;

        let id = self.conn.last_insert_rowid();

        log::debug!(
            "Saved {} message to conversation {} (id: {})",
            role,
            conversation_id,
            id
        );

        Ok(id)
    }

    /// Update conversation title
    pub fn update_conversation_title(&self, conversation_id: i64, title: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE conversations SET title = ?1 WHERE id = ?2",
                params![title, conversation_id],
            )
            .map_err(|e| format!("Failed to update conversation title: {}", e))?;

        log::info!("Updated conversation {} title to: {}", conversation_id, title);

        Ok(())
    }

    /// Delete a conversation and all its messages
    pub fn delete_conversation(&self, conversation_id: i64) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM conversations WHERE id = ?1",
                params![conversation_id],
            )
            .map_err(|e| format!("Failed to delete conversation: {}", e))?;

        log::info!("Deleted conversation {}", conversation_id);

        Ok(())
    }

    /// Get the total number of conversations
    pub fn count_conversations(&self) -> Result<i64, String> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count conversations: {}", e))?;

        Ok(count)
    }

    /// Get the total number of messages across all conversations
    pub fn count_messages(&self) -> Result<i64, String> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count messages: {}", e))?;

        Ok(count)
    }

    /// Load application settings from the database
    pub fn load_settings(&self) -> Result<Settings, String> {
        let llm_provider: String = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'llm_provider'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to load llm_provider: {}", e))?;

        let server_address: String = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'server_address'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to load server_address: {}", e))?;

        let wake_word_enabled_str: String = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'wake_word_enabled'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "false".to_string());

        let wake_word_enabled = wake_word_enabled_str == "true";

        let api_base_url: String = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'api_base_url'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "http://localhost:11434/v1".to_string());

        let model_name: String = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'model_name'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "gemma:2b".to_string());

        let vad_sensitivity: f32 = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'vad_sensitivity'",
                [],
                |row| row.get(0),
            )
            .ok()
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(0.02);

        let vad_timeout_ms: u32 = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'vad_timeout_ms'",
                [],
                |row| row.get(0),
            )
            .ok()
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(1280);

        let stt_model_name: String = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'stt_model_name'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "ggml-base.en.bin".to_string());

        let voice_preference: String = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'voice_preference'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "male".to_string());

        log::info!("Loaded settings: provider={}, server={}, wake_word={}, api_base_url={}, model={}, vad_sensitivity={}, vad_timeout_ms={}, stt_model={}, voice={}",
                   llm_provider, server_address, wake_word_enabled, api_base_url, model_name, vad_sensitivity, vad_timeout_ms, stt_model_name, voice_preference);

        Ok(Settings {
            llm_provider,
            server_address,
            wake_word_enabled,
            api_base_url,
            model_name,
            vad_sensitivity,
            vad_timeout_ms,
            stt_model_name,
            voice_preference,
        })
    }

    /// Save application settings to the database
    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE settings SET value = ?1 WHERE key = 'llm_provider'",
                params![&settings.llm_provider],
            )
            .map_err(|e| format!("Failed to save llm_provider: {}", e))?;

        self.conn
            .execute(
                "UPDATE settings SET value = ?1 WHERE key = 'server_address'",
                params![&settings.server_address],
            )
            .map_err(|e| format!("Failed to save server_address: {}", e))?;

        let wake_word_enabled_str = if settings.wake_word_enabled { "true" } else { "false" };
        self.conn
            .execute(
                "UPDATE settings SET value = ?1 WHERE key = 'wake_word_enabled'",
                params![wake_word_enabled_str],
            )
            .map_err(|e| format!("Failed to save wake_word_enabled: {}", e))?;

        self.conn
            .execute(
                "UPDATE settings SET value = ?1 WHERE key = 'api_base_url'",
                params![&settings.api_base_url],
            )
            .map_err(|e| format!("Failed to save api_base_url: {}", e))?;

        self.conn
            .execute(
                "UPDATE settings SET value = ?1 WHERE key = 'model_name'",
                params![&settings.model_name],
            )
            .map_err(|e| format!("Failed to save model_name: {}", e))?;

        self.conn
            .execute(
                "UPDATE settings SET value = ?1 WHERE key = 'vad_sensitivity'",
                params![settings.vad_sensitivity.to_string()],
            )
            .map_err(|e| format!("Failed to save vad_sensitivity: {}", e))?;

        self.conn
            .execute(
                "UPDATE settings SET value = ?1 WHERE key = 'vad_timeout_ms'",
                params![settings.vad_timeout_ms.to_string()],
            )
            .map_err(|e| format!("Failed to save vad_timeout_ms: {}", e))?;

        self.conn
            .execute(
                "UPDATE settings SET value = ?1 WHERE key = 'stt_model_name'",
                params![&settings.stt_model_name],
            )
            .map_err(|e| format!("Failed to save stt_model_name: {}", e))?;

        self.conn
            .execute(
                "UPDATE settings SET value = ?1 WHERE key = 'voice_preference'",
                params![&settings.voice_preference],
            )
            .map_err(|e| format!("Failed to save voice_preference: {}", e))?;

        log::info!("Saved settings: provider={}, server={}, wake_word={}, api_base_url={}, model={}, vad_sensitivity={}, vad_timeout_ms={}, stt_model={}, voice={}",
                   settings.llm_provider, settings.server_address, settings.wake_word_enabled,
                   settings.api_base_url, settings.model_name, settings.vad_sensitivity, settings.vad_timeout_ms, settings.stt_model_name, settings.voice_preference);

        Ok(())
    }

    /// Check if first-run wizard has been completed
    pub fn is_first_run_complete(&self) -> Result<bool, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = 'first_run_complete'")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let result = stmt
            .query_row([], |row| {
                let value: String = row.get(0)?;
                Ok(value == "true")
            })
            .map_err(|e| format!("Failed to query first_run_complete: {}", e))?;

        Ok(result)
    }

    /// Mark first-run wizard as complete
    pub fn mark_first_run_complete(&self) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE settings SET value = 'true' WHERE key = 'first_run_complete'",
                [],
            )
            .map_err(|e| format!("Failed to update first_run_complete: {}", e))?;

        log::info!("First-run wizard marked as complete");

        Ok(())
    }
}

/// Get the path to the database file
///
/// Uses Tauri's app data directory for cross-platform compatibility
pub fn get_database_path() -> Result<PathBuf, String> {
    // For development, use a local path
    // For production, this should use Tauri's app data directory

    // Try to get the app data directory
    let mut db_path = dirs::data_local_dir()
        .ok_or("Failed to get local data directory")?;

    db_path.push("com.nivora.aura-desktop");
    db_path.push("aura_storage.db");

    Ok(db_path)
}

/// Thread-safe database wrapper for use with Tauri state
pub type DatabaseState = Arc<Mutex<Database>>;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_initialization() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();

        assert_eq!(db.count_conversations().unwrap(), 0);
        assert_eq!(db.count_messages().unwrap(), 0);
    }

    #[test]
    fn test_create_conversation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();

        let id = db.create_conversation(Some("Test Chat".to_string())).unwrap();
        assert!(id > 0);

        let conversations = db.load_conversations().unwrap();
        assert_eq!(conversations.len(), 1);
        assert_eq!(conversations[0].title, "Test Chat");
    }

    #[test]
    fn test_save_and_load_messages() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();

        let conv_id = db.create_conversation(Some("Test".to_string())).unwrap();

        db.save_message(conv_id, "user", "Hello").unwrap();
        db.save_message(conv_id, "assistant", "Hi there!").unwrap();

        let messages = db.load_messages(conv_id).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].content, "Hi there!");
    }

    #[test]
    fn test_delete_conversation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();

        let conv_id = db.create_conversation(Some("Test".to_string())).unwrap();
        db.save_message(conv_id, "user", "Hello").unwrap();

        db.delete_conversation(conv_id).unwrap();

        assert_eq!(db.count_conversations().unwrap(), 0);
        assert_eq!(db.count_messages().unwrap(), 0); // CASCADE delete
    }
}
