# Spotify Music Integration - Technical Architecture (Multi-User Edition)

## Overview

This document describes the enhanced technical architecture for Spotify music integration in Nivora Aura with **multi-user support**. The implementation leverages Aura's voice biometrics system to provide personalized Spotify experiences for each enrolled user while maintaining privacy, transparency, and local-first design principles.

---

## 🆕 **Multi-User Architecture Overview**

With the integration of Aura's voice biometrics system, Spotify authentication and music commands now support **per-user accounts**:

- ✅ **Individual Spotify accounts** linked to each voice-enrolled user  
- ✅ **Contextual music commands** ("play *my* workout playlist")
- ✅ **Automatic account selection** based on speaker identification
- ✅ **Secure user-scoped token storage** in OS keyring
- ✅ **Graceful fallback** for unknown or unauthenticated speakers

### 🔄 **Migration from Global to Per-User**

**Before (Global Account):**
```
Global Settings:
├── spotify_connected: bool
├── spotify_client_id: string
└── spotify_auto_play: bool

OS Keyring:
├── spotify_access_token
├── spotify_refresh_token  
└── spotify_token_expiry
```

**After (Multi-User):**
```
User Profiles:
├── user_1 (id: 1, name: "Alice")
│   ├── spotify_connected: bool
│   ├── spotify_client_id: string
│   └── voice_print_embedding: blob
├── user_2 (id: 2, name: "Bob")  
│   ├── spotify_connected: bool
│   ├── spotify_client_id: string
│   └── voice_print_embedding: blob

OS Keyring (User-Scoped):
├── spotify_access_token_1
├── spotify_refresh_token_1
├── spotify_token_expiry_1
├── spotify_access_token_2
├── spotify_refresh_token_2
└── spotify_token_expiry_2
```

---

## 🔑 **Multi-User Token Management**

### Enhanced Secrets Architecture

#### User-Scoped Keyring Storage

```rust
// Enhanced secrets.rs

/// Generate user-scoped keyring entry name
fn get_user_scoped_key(base_key: &str, user_id: i64) -> String {
    format!("{}_{}", base_key, user_id)
}

/// Keyring entry names for user-scoped Spotify tokens
fn get_spotify_access_token_key(user_id: i64) -> String {
    get_user_scoped_key(SPOTIFY_ACCESS_TOKEN, user_id)
}

// Enhanced API
pub fn save_user_spotify_access_token(user_id: i64, token: &str) -> Result<(), String>
pub fn load_user_spotify_access_token(user_id: i64) -> Result<String, String>
pub fn save_user_spotify_refresh_token(user_id: i64, token: &str) -> Result<(), String>
pub fn load_user_spotify_refresh_token(user_id: i64) -> Result<String, String>
pub fn save_user_spotify_token_expiry(user_id: i64, expiry: &DateTime<Utc>) -> Result<(), String>
pub fn load_user_spotify_token_expiry(user_id: i64) -> Result<DateTime<Utc>, String>
pub fn delete_user_spotify_tokens(user_id: i64) -> Result<(), String>
pub fn list_users_with_spotify() -> Result<Vec<i64>, String>
```

#### Secure Token Isolation

```rust
// Token isolation ensures:
// 1. User A cannot access User B's tokens
// 2. Each user has independent authentication state  
// 3. Token refresh works independently per user
// 4. Account disconnection only affects specific user

pub struct SpotifyTokenManager;

impl SpotifyTokenManager {
    /// Load tokens for specific user with automatic refresh
    pub async fn get_valid_tokens_for_user(user_id: i64) -> Result<TokenSet, SpotifyError> {
        let access_token = secrets::load_user_spotify_access_token(user_id)?;
        let expiry = secrets::load_user_spotify_token_expiry(user_id)?;

        // Refresh if expired or expiring soon (5 min buffer)  
        if expiry < Utc::now() + Duration::minutes(5) {
            let refresh_token = secrets::load_user_spotify_refresh_token(user_id)?;
            let new_tokens = refresh_user_access_token(user_id, &refresh_token).await?;
            
            // Save refreshed tokens
            secrets::save_user_spotify_access_token(user_id, &new_tokens.access_token)?;
            secrets::save_user_spotify_token_expiry(user_id, &new_tokens.expires_at)?;
            
            Ok(new_tokens)
        } else {
            Ok(TokenSet {
                access_token,
                refresh_token: secrets::load_user_spotify_refresh_token(user_id)?,
                expires_at: expiry,
            })
        }
    }
}
```

---

## 🗄️ **Enhanced Database Schema**

### User Profile Extensions

```sql
-- Enhanced user profiles table with Spotify integration
ALTER TABLE user_profiles ADD COLUMN spotify_connected BOOLEAN DEFAULT FALSE;
ALTER TABLE user_profiles ADD COLUMN spotify_client_id TEXT DEFAULT '';
ALTER TABLE user_profiles ADD COLUMN spotify_user_id TEXT DEFAULT '';
ALTER TABLE user_profiles ADD COLUMN spotify_display_name TEXT DEFAULT '';
ALTER TABLE user_profiles ADD COLUMN spotify_email TEXT DEFAULT '';
ALTER TABLE user_profiles ADD COLUMN auto_play_enabled BOOLEAN DEFAULT TRUE;
ALTER TABLE user_profiles ADD COLUMN spotify_connected_at DATETIME NULL;
ALTER TABLE user_profiles ADD COLUMN last_spotify_refresh DATETIME NULL;

-- Index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_user_profiles_spotify_connected 
ON user_profiles(spotify_connected);

CREATE INDEX IF NOT EXISTS idx_user_profiles_spotify_user_id
ON user_profiles(spotify_user_id);
```

### Migration Strategy  

```rust
// Migration from global to per-user Spotify settings

impl Database {
    pub fn migrate_spotify_to_multi_user(&self) -> Result<(), String> {
        log::info!("Migrating Spotify settings from global to multi-user...");
        
        // Check if we have existing global Spotify connection
        let settings = self.load_settings()?;
        
        if settings.spotify_connected {
            log::info!("Found existing global Spotify connection, migrating...");
            
            // Create a default "Primary User" for existing connection
            let primary_user_id = self.create_user_profile_internal(
                "Primary User",
                vec![], // Empty embedding - will need voice enrollment
            )?;
            
            // Migrate global Spotify settings to primary user
            self.conn.execute(
                "UPDATE user_profiles SET 
                 spotify_connected = ?, 
                 spotify_client_id = ?,
                 spotify_connected_at = CURRENT_TIMESTAMP
                 WHERE id = ?",
                rusqlite::params![true, &settings.spotify_client_id, primary_user_id]
            )?;
            
            // Migrate tokens in keyring (if they exist)
            if let Ok(access_token) = secrets::load_spotify_access_token() {
                secrets::save_user_spotify_access_token(primary_user_id, &access_token)?;
            }
            
            if let Ok(refresh_token) = secrets::load_spotify_refresh_token() {
                secrets::save_user_spotify_refresh_token(primary_user_id, &refresh_token)?;  
            }
            
            if let Ok(expiry) = secrets::load_spotify_token_expiry() {
                secrets::save_user_spotify_token_expiry(primary_user_id, &expiry)?;
            }
            
            // Clean up old global tokens
            let _ = secrets::delete_spotify_tokens();
            
            log::info!("✓ Spotify migration completed. Primary user created with ID: {}", primary_user_id);
        }
        
        Ok(())
    }
}
```

---

## 🎯 **Enhanced API Client Architecture**

### User-Contextual Spotify Client

```rust
// Enhanced spotify_client.rs

pub struct SpotifyClient {
    http_client: reqwest::Client,
    access_token: String,
    user_id: i64,                    // NEW: Associated user ID
    user_context: Option<UserInfo>,   // NEW: User context for logging
}

impl SpotifyClient {
    /// Create client for specific user
    pub async fn new_for_user(user_id: i64) -> Result<Self, SpotifyError> {
        let tokens = SpotifyTokenManager::get_valid_tokens_for_user(user_id).await?;
        let user_info = Database::get_user_spotify_info(user_id).await?;
        
        Ok(Self {
            http_client: reqwest::Client::new(),
            access_token: tokens.access_token,
            user_id,
            user_context: Some(user_info),
        })
    }
    
    /// Enhanced playlist search with user context
    pub async fn get_user_playlists(&self) -> Result<Vec<Playlist>, SpotifyError> {
        log::debug!("Fetching playlists for user {}", self.user_id);
        
        let response = self.http_client
            .get(&format!("{}/me/playlists", SPOTIFY_API_BASE))
            .bearer_auth(&self.access_token)
            .query(&[("limit", "50")])
            .send()
            .await?;

        let data: PlaylistsResponse = response.json().await?;
        
        log::debug!("Retrieved {} playlists for user {}", data.items.len(), self.user_id);
        
        Ok(data.items)
    }

    /// Play user's specific playlist 
    pub async fn play_playlist(&self, playlist_uri: &str) -> Result<(), SpotifyError> {
        log::info!("Playing playlist {} for user {}", playlist_uri, self.user_id);
        
        self.http_client
            .put(&format!("{}/me/player/play", SPOTIFY_API_BASE))
            .bearer_auth(&self.access_token)
            .json(&serde_json::json!({
                "context_uri": playlist_uri
            }))
            .send()
            .await?;

        Ok(())
    }
}
```

---

## 📋 **Implementation Plan Summary**

### Phase 1: Multi-Account Foundation (Current Task)
1. **✅ Architecture Design** (This Document)
2. **🔄 Enhanced Secrets Management** (User-scoped tokens)
3. **🔄 Database Schema Extensions** (User profiles + Spotify)
4. **🔄 Migration Support** (Global → Per-User)

### Phase 2: Context-Aware Commands  
1. **Enhanced Intent Parser** (Possessive pronoun detection)
2. **Speaker Integration** (Voice biometrics → User context)
3. **Smart Routing** (User-specific API calls)
4. **Graceful Fallbacks** (Anonymous speaker handling)

### Phase 3: User Experience
1. **Frontend Updates** (Per-user Spotify settings)
2. **User Profile Management** (Enrollment + Account linking)
3. **Enhanced Error Messages** (Context-aware responses)
4. **Migration UX** (Smooth transition for existing users)

---

## 🔒 **Security & Privacy Enhancements**

### Token Isolation
- ✅ **Per-user keyring entries** prevent cross-user access
- ✅ **Independent token refresh** per user account
- ✅ **Secure cleanup** when users are deleted
- ✅ **No token cross-contamination** between users

### Voice Privacy  
- ✅ **Speaker identification** happens locally (no cloud)
- ✅ **Voice prints** never leave the device
- ✅ **User consent** required for both voice enrollment and Spotify linking
- ✅ **Clear data ownership** (users control their voice + music data)

---

## Document Metadata

- **Version**: 2.0 - Multi-User Edition
- **Last Updated**: 2025-10-13  
- **Author**: Claude Code (AuraPM)
- **Status**: Ready for Implementation
- **Dependencies**: Voice Biometrics System (✅ Complete)