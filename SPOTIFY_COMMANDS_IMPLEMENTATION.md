# Spotify Multi-User Commands Implementation Plan

## Overview

This document outlines the specific modifications needed for Tauri commands to support multi-user Spotify authentication. The changes enable user-scoped Spotify operations while maintaining backward compatibility during the migration period.

---

## 🔄 **Command Modifications Required**

### 1. Enhanced Authentication Commands

#### `spotify_start_auth` (Modified)
**Current Signature:**
```rust
async fn spotify_start_auth(
    client_id: String,
    state: State<'_, SpotifyAuthState>,
) -> Result<(), AuraError>
```

**New Signature:**
```rust
async fn spotify_start_auth(
    user_id: i64,
    client_id: String,
    state: State<'_, SpotifyAuthState>,
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError>
```

**Key Changes:**
- Accept `user_id` parameter to associate tokens with specific user
- Save tokens using `secrets::save_user_spotify_*` functions
- Update user profile in database with Spotify connection status
- Fetch Spotify user info and store in database

#### `spotify_disconnect` (Modified)
**Current Signature:**
```rust
async fn spotify_disconnect(
    db: State<'_, DatabaseState>
) -> Result<(), AuraError>
```

**New Signature:**
```rust
async fn spotify_disconnect_user(
    user_id: i64,
    db: State<'_, DatabaseState>
) -> Result<(), AuraError>
```

**Key Changes:**
- Delete user-specific tokens: `secrets::delete_user_spotify_tokens(user_id)`
- Update user profile to mark Spotify as disconnected
- Preserve other users' connections

### 2. Status and Information Commands

#### `spotify_get_status` (Modified)
**Current Signature:**
```rust
async fn spotify_get_status(
    db: State<'_, DatabaseState>
) -> Result<SpotifyStatusResponse, AuraError>
```

**New Signature:**
```rust
async fn spotify_get_user_status(
    user_id: Option<i64>,
    db: State<'_, DatabaseState>
) -> Result<SpotifyUserStatusResponse, AuraError>
```

**Key Changes:**
- Return user-specific connection status
- If `user_id` is None, return status for "active" user or aggregate info
- Include user's Spotify display name and email if connected

#### NEW: `spotify_list_user_connections`
**New Command:**
```rust
async fn spotify_list_user_connections(
    db: State<'_, DatabaseState>
) -> Result<Vec<UserSpotifyConnection>, AuraError>
```

**Purpose:**
- List all users who have connected Spotify accounts
- Return user profiles with Spotify connection status
- Used by frontend to show per-user connection management

### 3. Music Control Commands

#### `spotify_handle_music_command` (Enhanced)
**Current Signature:**
```rust
async fn spotify_handle_music_command(
    text: String,
    db: State<'_, DatabaseState>,
) -> Result<String, AuraError>
```

**New Signature:**
```rust
async fn spotify_handle_music_command(
    text: String,
    speaker_context: Option<SpeakerInfo>,
    db: State<'_, DatabaseState>,
) -> Result<String, AuraError>
```

**Key Changes:**
- Accept speaker context from voice biometrics
- Route to user-specific Spotify account based on speaker identification
- Enhanced error messages with user context
- Support for "my playlist" vs "John's playlist" disambiguation

#### `spotify_control_playback` (Enhanced)
**Current Signature:**
```rust
async fn spotify_control_playback(
    action: String,
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError>
```

**New Signature:**
```rust
async fn spotify_control_playback(
    action: String,
    user_id: Option<i64>,
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError>
```

**Key Changes:**
- Use user-specific tokens for playback control
- If `user_id` is None, attempt to use last identified speaker
- Graceful fallback to "primary" user if configured

---

## 📊 **New Data Structures**

### Enhanced Response Types

```rust
#[derive(Debug, Serialize)]
pub struct SpotifyUserStatusResponse {
    pub connected: bool,
    pub user_id: Option<i64>,
    pub user_name: Option<String>,
    pub spotify_display_name: Option<String>,
    pub spotify_email: Option<String>,
    pub current_track: Option<serde_json::Value>,
    pub connected_at: Option<String>,
    pub last_refresh: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserSpotifyConnection {
    pub user_id: i64,
    pub user_name: String,
    pub spotify_connected: bool,
    pub spotify_display_name: Option<String>,
    pub spotify_email: Option<String>,
    pub connected_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpeakerInfo {
    pub user_id: Option<i64>,
    pub user_name: Option<String>,
    pub similarity_score: f32,
    pub identified: bool,
}
```

### Token Management Types

```rust
#[derive(Debug, Clone)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct SpotifyTokenManager;

impl SpotifyTokenManager {
    pub async fn get_valid_tokens_for_user(user_id: i64) -> Result<TokenSet, SpotifyError> {
        // Implementation as defined in architecture
    }
    
    pub async fn refresh_tokens_for_user(user_id: i64) -> Result<TokenSet, SpotifyError> {
        // User-specific token refresh
    }
}
```

---

## 🔧 **Implementation Steps**

### Step 1: Database Schema Updates
```rust
// Add to database.rs migration
impl Database {
    pub fn add_spotify_columns_to_user_profiles(&self) -> Result<(), String> {
        // Add columns as defined in architecture document
        self.conn.execute(
            "ALTER TABLE user_profiles ADD COLUMN spotify_connected BOOLEAN DEFAULT FALSE",
            []
        )?;
        // ... other columns
        Ok(())
    }
}
```

### Step 2: Enhanced Secrets Integration
```rust
// Update lib.rs imports
use crate::secrets::{
    // Existing imports...
    save_user_spotify_access_token,
    load_user_spotify_access_token,
    save_user_spotify_refresh_token,
    load_user_spotify_refresh_token,
    save_user_spotify_token_expiry,
    load_user_spotify_token_expiry,
    delete_user_spotify_tokens,
    is_user_spotify_connected,
};
```

### Step 3: Update Command Registration
```rust
// In lib.rs generate_handler![] macro
tauri::generate_handler![
    // ... existing commands ...
    
    // Enhanced Spotify commands
    spotify_start_auth,                  // Modified
    spotify_disconnect_user,             // Modified (renamed)
    spotify_get_user_status,            // Modified (renamed)
    spotify_list_user_connections,      // NEW
    spotify_handle_music_command,       // Enhanced
    spotify_control_playback,           // Enhanced
    
    // ... other commands ...
]
```

### Step 4: Migration Support
```rust
// In lib.rs initialization
async fn initialize_spotify_multi_user_support(
    db: Arc<TokioMutex<Database>>
) -> Result<(), String> {
    log::info!("Initializing multi-user Spotify support...");
    
    let database = db.lock().await;
    
    // Add new columns if they don't exist
    database.add_spotify_columns_to_user_profiles()?;
    
    // Migrate existing global Spotify connection
    database.migrate_spotify_to_multi_user()?;
    
    log::info!("✓ Multi-user Spotify support initialized");
    Ok(())
}
```

---

## 🎯 **Enhanced Client Architecture**

### User-Contextual Client Creation

```rust
// Enhanced spotify_client.rs modifications
impl SpotifyClient {
    /// Create client for specific user (new primary constructor)
    pub async fn new_for_user(user_id: i64) -> Result<Self, SpotifyError> {
        let tokens = SpotifyTokenManager::get_valid_tokens_for_user(user_id).await?;
        
        Ok(Self {
            http_client: reqwest::Client::new(),
            access_token: tokens.access_token,
            user_id: Some(user_id),
        })
    }
    
    /// Legacy constructor for backward compatibility
    pub async fn new() -> Result<Self, SpotifyError> {
        // Try to find a "primary" user or use global tokens during migration
        if let Ok(access_token) = crate::secrets::load_spotify_access_token() {
            Ok(Self {
                http_client: reqwest::Client::new(),
                access_token,
                user_id: None, // Legacy mode
            })
        } else {
            Err(SpotifyError::NotAuthenticated)
        }
    }
}
```

---

## ⚡ **Performance Considerations**

### Token Caching Strategy
```rust
// In-memory token cache for performance
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TokenCache {
    cache: Arc<RwLock<HashMap<i64, (TokenSet, chrono::DateTime<chrono::Utc>)>>>,
}

impl TokenCache {
    pub async fn get_or_fetch_tokens(&self, user_id: i64) -> Result<TokenSet, SpotifyError> {
        let cache = self.cache.read().await;
        
        if let Some((tokens, cached_at)) = cache.get(&user_id) {
            // Return cached if not expired (5 min cache TTL)
            if chrono::Utc::now() - *cached_at < chrono::Duration::minutes(5) {
                return Ok(tokens.clone());
            }
        }
        
        drop(cache);
        
        // Fetch fresh tokens and cache them
        let fresh_tokens = SpotifyTokenManager::get_valid_tokens_for_user(user_id).await?;
        
        let mut cache = self.cache.write().await;
        cache.insert(user_id, (fresh_tokens.clone(), chrono::Utc::now()));
        
        Ok(fresh_tokens)
    }
}
```

---

## 🧪 **Testing Strategy**

### Unit Tests for Multi-User Scenarios
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_scoped_authentication() {
        // Test user-specific token storage and retrieval
    }
    
    #[tokio::test]
    async fn test_speaker_context_routing() {
        // Test music command routing based on speaker identification
    }
    
    #[tokio::test]
    async fn test_migration_from_global() {
        // Test migration from global to per-user tokens
    }
    
    #[tokio::test]
    async fn test_multi_user_isolation() {
        // Test that users cannot access each other's tokens
    }
}
```

### Integration Test Scenarios
1. **Multi-User Enrollment**: Multiple users enroll and connect Spotify
2. **Context Switching**: Commands automatically use correct account based on speaker
3. **Fallback Behavior**: Anonymous speakers get appropriate error messages
4. **Token Isolation**: User deletion cleans up only their tokens
5. **Migration**: Existing single-user setup migrates smoothly

---

## 📋 **Implementation Checklist**

### Phase 1: Foundation (Current Sprint)
- [ ] ✅ Architecture document completed
- [ ] ✅ Enhanced secrets.rs with user-scoped functions
- [ ] 🔄 Database schema updates for user profiles
- [ ] 🔄 Modified Tauri command signatures
- [ ] 🔄 Updated SpotifyClient for user context
- [ ] 🔄 Migration logic for existing users

### Phase 2: Integration (Next Sprint)
- [ ] Enhanced music intent parsing
- [ ] Speaker context integration
- [ ] Context-aware error messages
- [ ] Frontend UI updates
- [ ] End-to-end testing

### Phase 3: Polish (Final Sprint)
- [ ] Performance optimization (token caching)
- [ ] Enhanced user experience flows
- [ ] Documentation and user guides
- [ ] Production deployment preparation

---

## Document Metadata

- **Version**: 1.0
- **Last Updated**: 2025-10-13
- **Author**: Claude Code (AuraPM)
- **Status**: Implementation Ready
- **Dependencies**: Enhanced secrets.rs, Voice Biometrics System