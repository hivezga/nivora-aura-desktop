# Spotify Music Integration - Technical Architecture

## Overview

This document describes the technical architecture for Spotify music integration in Nivora Aura. The implementation follows Aura's core principles of privacy, transparency, and local-first design while enabling seamless music control via voice commands.

---

## Architecture Principles

1. **Privacy-First**: OAuth2 tokens stored securely in OS keyring, no credentials in logs
2. **Transparent**: User explicitly authorizes Spotify access, clear scope permissions
3. **Secure**: PKCE flow (no client secret), token refresh mechanism, secure storage
4. **Resilient**: Graceful error handling, automatic token refresh, offline fallback
5. **Lightweight**: Custom client using `oauth2` + `reqwest` (no heavy dependencies)

---

## System Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend (React)                          │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐  │
│  │ Settings Modal   │  │ Now Playing (opt)│  │ Chat View    │  │
│  │ - Connect Spotify│  │ - Track info     │  │ - Music      │  │
│  │ - Status display │  │ - Album art      │  │   commands   │  │
│  └──────────────────┘  └──────────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ▼ Tauri IPC
┌─────────────────────────────────────────────────────────────────┐
│                       Backend (Rust/Tauri)                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Tauri Commands (lib.rs)                                  │   │
│  │ - spotify_start_auth()                                   │   │
│  │ - spotify_get_status()                                   │   │
│  │ - spotify_play_track(query, artist)                      │   │
│  │ - spotify_control_playback(action)                       │   │
│  │ - spotify_get_current_track()                            │   │
│  └──────────────────────────────────────────────────────────┘   │
│         ▼                    ▼                      ▼            │
│  ┌─────────────┐  ┌──────────────────┐  ┌──────────────────┐   │
│  │ spotify_    │  │ spotify_client   │  │ music_intent     │   │
│  │ auth.rs     │  │ .rs              │  │ .rs              │   │
│  │             │  │                  │  │                  │   │
│  │ - OAuth2    │  │ - Search API     │  │ - Parse commands │   │
│  │   PKCE flow │  │ - Playback API   │  │ - Extract        │   │
│  │ - Token     │  │ - Playlist API   │  │   entities       │   │
│  │   refresh   │  │ - Device API     │  │ - Intent types   │   │
│  │ - Local     │  │                  │  │                  │   │
│  │   callback  │  │                  │  │                  │   │
│  │   server    │  │                  │  │                  │   │
│  └─────────────┘  └──────────────────┘  └──────────────────┘   │
│         ▼                    ▼                                   │
│  ┌─────────────┐  ┌──────────────────┐                          │
│  │ secrets.rs  │  │ database.rs      │                          │
│  │ (Extended)  │  │ (Extended)       │                          │
│  │             │  │                  │                          │
│  │ - Save/load │  │ - Connection     │                          │
│  │   tokens    │  │   state          │                          │
│  │ - OS keyring│  │ - User prefs     │                          │
│  └─────────────┘  └──────────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
                              ▼
                  ┌───────────────────────┐
                  │ Spotify Web API       │
                  │ (HTTPS REST)          │
                  └───────────────────────┘
                              ▼
                  ┌───────────────────────┐
                  │ Spotify Connect       │
                  │ (User's devices)      │
                  └───────────────────────┘
```

---

## OAuth2 Authentication Flow (PKCE)

### 1. Authorization Request

```rust
// spotify_auth.rs

pub struct SpotifyAuthManager {
    client_id: String,
    redirect_uri: String,
    pkce_verifier: Option<String>,
    local_server: Option<TinyHttpServer>,
}

impl SpotifyAuthManager {
    pub async fn start_authorization() -> Result<String, SpotifyError> {
        // 1. Generate PKCE challenge
        let pkce_verifier = generate_pkce_verifier();
        let pkce_challenge = sha256_base64(&pkce_verifier);

        // 2. Start local HTTP server on http://127.0.0.1:8888
        let server = start_local_callback_server().await?;

        // 3. Build authorization URL
        let auth_url = format!(
            "https://accounts.spotify.com/authorize?\
             client_id={}&\
             response_type=code&\
             redirect_uri={}&\
             code_challenge_method=S256&\
             code_challenge={}&\
             scope={}",
            client_id,
            "http://127.0.0.1:8888/callback",
            pkce_challenge,
            REQUIRED_SCOPES.join("%20")
        );

        // 4. Open system browser
        open_browser(&auth_url)?;

        // 5. Wait for callback (with timeout)
        let auth_code = server.wait_for_callback(120).await?;

        Ok(auth_code)
    }
}
```

### 2. Token Exchange

```rust
pub async fn exchange_code_for_token(
    auth_code: String,
    pkce_verifier: String,
) -> Result<TokenResponse, SpotifyError> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://accounts.spotify.com/api/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &auth_code),
            ("redirect_uri", "http://127.0.0.1:8888/callback"),
            ("client_id", &self.client_id),
            ("code_verifier", &pkce_verifier),
        ])
        .send()
        .await?;

    let token_data: TokenResponse = response.json().await?;

    // Store tokens in OS keyring
    save_spotify_tokens(&token_data)?;

    Ok(token_data)
}
```

### 3. Token Refresh

```rust
pub async fn refresh_access_token() -> Result<String, SpotifyError> {
    let refresh_token = secrets::load_spotify_refresh_token()?;

    let client = reqwest::Client::new();
    let response = client
        .post("https://accounts.spotify.com/api/token")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh_token),
            ("client_id", &self.client_id),
        ])
        .send()
        .await?;

    let token_data: TokenResponse = response.json().await?;

    // Update access token in keyring
    secrets::save_spotify_access_token(&token_data.access_token)?;

    Ok(token_data.access_token)
}
```

---

## Spotify API Client

### API Endpoints Used

| Endpoint | Method | Purpose | Required Scope |
|----------|--------|---------|----------------|
| `/v1/me/player/play` | PUT | Start/resume playback | `user-modify-playback-state` |
| `/v1/me/player/pause` | PUT | Pause playback | `user-modify-playback-state` |
| `/v1/me/player/next` | POST | Skip to next track | `user-modify-playback-state` |
| `/v1/me/player/previous` | POST | Skip to previous track | `user-modify-playback-state` |
| `/v1/me/player` | GET | Get playback state | `user-read-playback-state` |
| `/v1/me/player/currently-playing` | GET | Get current track | `user-read-currently-playing` |
| `/v1/search` | GET | Search tracks/artists/playlists | None |
| `/v1/me/playlists` | GET | Get user playlists | `playlist-read-private` |
| `/v1/me/player/devices` | GET | Get available devices | `user-read-playback-state` |

### Client Implementation

```rust
// spotify_client.rs

pub struct SpotifyClient {
    http_client: reqwest::Client,
    access_token: String,
}

impl SpotifyClient {
    pub async fn new() -> Result<Self, SpotifyError> {
        let access_token = Self::get_valid_token().await?;

        Ok(Self {
            http_client: reqwest::Client::new(),
            access_token,
        })
    }

    async fn get_valid_token() -> Result<String, SpotifyError> {
        let token = secrets::load_spotify_access_token()?;
        let expiry = secrets::load_spotify_token_expiry()?;

        // Refresh if expired or expiring soon (5 min buffer)
        if expiry < Utc::now() + Duration::minutes(5) {
            SpotifyAuthManager::refresh_access_token().await
        } else {
            Ok(token)
        }
    }

    pub async fn search_track(
        &self,
        query: &str,
        artist: Option<&str>,
    ) -> Result<Vec<Track>, SpotifyError> {
        let search_query = if let Some(artist) = artist {
            format!("track:{} artist:{}", query, artist)
        } else {
            format!("track:{}", query)
        };

        let response = self.http_client
            .get("https://api.spotify.com/v1/search")
            .bearer_auth(&self.access_token)
            .query(&[
                ("q", search_query),
                ("type", "track".to_string()),
                ("limit", "10".to_string()),
            ])
            .send()
            .await?;

        let data: SearchResponse = response.json().await?;
        Ok(data.tracks.items)
    }

    pub async fn play_track(&self, uri: &str) -> Result<(), SpotifyError> {
        self.http_client
            .put("https://api.spotify.com/v1/me/player/play")
            .bearer_auth(&self.access_token)
            .json(&PlayRequest {
                uris: vec![uri.to_string()],
                position_ms: 0,
            })
            .send()
            .await?;

        Ok(())
    }

    pub async fn control_playback(
        &self,
        action: PlaybackAction,
    ) -> Result<(), SpotifyError> {
        let endpoint = match action {
            PlaybackAction::Pause => "/v1/me/player/pause",
            PlaybackAction::Resume => "/v1/me/player/play",
            PlaybackAction::Next => "/v1/me/player/next",
            PlaybackAction::Previous => "/v1/me/player/previous",
        };

        let method = match action {
            PlaybackAction::Next | PlaybackAction::Previous => Method::POST,
            _ => Method::PUT,
        };

        self.http_client
            .request(method, format!("https://api.spotify.com{}", endpoint))
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_current_track(&self) -> Result<CurrentlyPlaying, SpotifyError> {
        let response = self.http_client
            .get("https://api.spotify.com/v1/me/player/currently-playing")
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        if response.status() == 204 {
            return Err(SpotifyError::NothingPlaying);
        }

        let data: CurrentlyPlaying = response.json().await?;
        Ok(data)
    }
}
```

---

## Music Intent Recognition

### Intent Types

```rust
// music_intent.rs

#[derive(Debug, Clone, PartialEq)]
pub enum MusicIntent {
    PlaySong { song: String, artist: Option<String> },
    PlayPlaylist { playlist_name: String },
    PlayArtist { artist: String },
    Pause,
    Resume,
    Next,
    Previous,
    GetCurrentTrack,
    Unknown,
}

pub struct MusicIntentParser;

impl MusicIntentParser {
    pub fn parse(text: &str) -> MusicIntent {
        let text_lower = text.to_lowercase();

        // Pattern matching for intents
        if text_lower.contains("pause") || text_lower.contains("stop") {
            return MusicIntent::Pause;
        }

        if text_lower.contains("resume") || text_lower.contains("continue") || text_lower.contains("unpause") {
            return MusicIntent::Resume;
        }

        if text_lower.contains("next") || text_lower.contains("skip") {
            return MusicIntent::Next;
        }

        if text_lower.contains("previous") || text_lower.contains("back") {
            return MusicIntent::Previous;
        }

        if text_lower.contains("what") && (text_lower.contains("playing") || text_lower.contains("song")) {
            return MusicIntent::GetCurrentTrack;
        }

        // Parse "play <song> by <artist>"
        if text_lower.contains("play") {
            if let Some(playlist_match) = Self::extract_playlist(&text_lower) {
                return MusicIntent::PlayPlaylist { playlist_name: playlist_match };
            }

            if let Some((song, artist)) = Self::extract_song_and_artist(&text_lower) {
                return MusicIntent::PlaySong { song, artist };
            }

            if let Some(artist) = Self::extract_artist(&text_lower) {
                return MusicIntent::PlayArtist { artist };
            }
        }

        MusicIntent::Unknown
    }

    fn extract_song_and_artist(text: &str) -> Option<(String, Option<String>)> {
        // Pattern: "play <song> by <artist>"
        let re_with_artist = regex::Regex::new(r"play\s+(.+?)\s+by\s+(.+)").ok()?;

        if let Some(caps) = re_with_artist.captures(text) {
            let song = caps.get(1)?.as_str().trim().to_string();
            let artist = caps.get(2)?.as_str().trim().to_string();
            return Some((song, Some(artist)));
        }

        // Pattern: "play <song>"
        let re_song_only = regex::Regex::new(r"play\s+(.+)").ok()?;

        if let Some(caps) = re_song_only.captures(text) {
            let song = caps.get(1)?.as_str().trim().to_string();
            return Some((song, None));
        }

        None
    }

    fn extract_playlist(text: &str) -> Option<String> {
        // Pattern: "play my <playlist> playlist"
        let re = regex::Regex::new(r"play\s+(?:my\s+)?(.+?)\s+playlist").ok()?;

        if let Some(caps) = re.captures(text) {
            return Some(caps.get(1)?.as_str().trim().to_string());
        }

        None
    }

    fn extract_artist(text: &str) -> Option<String> {
        // Pattern: "play <artist>"
        let re = regex::Regex::new(r"play\s+(.+)").ok()?;

        if let Some(caps) = re.captures(text) {
            return Some(caps.get(1)?.as_str().trim().to_string());
        }

        None
    }
}
```

### Integration with LLM (Alternative Approach)

For more sophisticated intent recognition, we can augment the user's prompt with music-specific instructions:

```rust
pub async fn handle_music_command_with_llm(
    user_text: &str,
    llm_engine: &LLMEngine,
) -> Result<MusicIntent, AuraError> {
    let augmented_prompt = format!(
        r#"Parse the following music command and extract the intent and entities.

User command: "{}"

Respond in JSON format:
{{
  "intent": "play_song" | "play_playlist" | "play_artist" | "pause" | "resume" | "next" | "previous" | "get_current",
  "song": "<song name if applicable>",
  "artist": "<artist name if applicable>",
  "playlist": "<playlist name if applicable>"
}}

Only respond with the JSON, no additional text."#,
        user_text
    );

    let llm_response = llm_engine.generate_response(&augmented_prompt).await?;
    let intent_data: IntentJson = serde_json::from_str(&llm_response)?;

    Ok(convert_to_music_intent(intent_data))
}
```

---

## Data Storage

### OS Keyring (secrets.rs)

```rust
// Extended secrets.rs

const SPOTIFY_ACCESS_TOKEN: &str = "spotify_access_token";
const SPOTIFY_REFRESH_TOKEN: &str = "spotify_refresh_token";
const SPOTIFY_TOKEN_EXPIRY: &str = "spotify_token_expiry";

pub fn save_spotify_access_token(token: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, SPOTIFY_ACCESS_TOKEN)?;
    entry.set_password(token)?;
    Ok(())
}

pub fn load_spotify_access_token() -> Result<String, String> {
    let entry = Entry::new(SERVICE_NAME, SPOTIFY_ACCESS_TOKEN)?;
    entry.get_password().map_err(|e| e.to_string())
}

pub fn save_spotify_refresh_token(token: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, SPOTIFY_REFRESH_TOKEN)?;
    entry.set_password(token)?;
    Ok(())
}

pub fn load_spotify_refresh_token() -> Result<String, String> {
    let entry = Entry::new(SERVICE_NAME, SPOTIFY_REFRESH_TOKEN)?;
    entry.get_password().map_err(|e| e.to_string())
}

pub fn save_spotify_token_expiry(expiry: &DateTime<Utc>) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, SPOTIFY_TOKEN_EXPIRY)?;
    entry.set_password(&expiry.to_rfc3339())?;
    Ok(())
}

pub fn load_spotify_token_expiry() -> Result<DateTime<Utc>, String> {
    let entry = Entry::new(SERVICE_NAME, SPOTIFY_TOKEN_EXPIRY)?;
    let expiry_str = entry.get_password().map_err(|e| e.to_string())?;
    DateTime::parse_from_rfc3339(&expiry_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| e.to_string())
}

pub fn delete_spotify_tokens() -> Result<(), String> {
    // Best-effort deletion
    let _ = Entry::new(SERVICE_NAME, SPOTIFY_ACCESS_TOKEN)?.delete_password();
    let _ = Entry::new(SERVICE_NAME, SPOTIFY_REFRESH_TOKEN)?.delete_password();
    let _ = Entry::new(SERVICE_NAME, SPOTIFY_TOKEN_EXPIRY)?.delete_password();
    Ok(())
}
```

### Database (database.rs)

```rust
// Extended Settings struct in database.rs

pub struct Settings {
    // ... existing fields ...

    // Spotify Integration
    pub spotify_connected: bool,           // Whether Spotify is connected
    pub spotify_client_id: String,         // Spotify app client ID (user-provided)
    pub spotify_auto_play_enabled: bool,   // Auto-play music via voice commands
}

// Database initialization (add to create_tables_if_not_exists)
self.conn.execute(
    "INSERT OR IGNORE INTO settings (key, value) VALUES ('spotify_connected', 'false')",
    [],
)?;

self.conn.execute(
    "INSERT OR IGNORE INTO settings (key, value) VALUES ('spotify_client_id', '')",
    [],
)?;

self.conn.execute(
    "INSERT OR IGNORE INTO settings (key, value) VALUES ('spotify_auto_play_enabled', 'true')",
    [],
)?;
```

---

## Tauri Commands

### Authentication Commands

```rust
// lib.rs

#[tauri::command]
async fn spotify_start_auth(
    client_id: String,
    state: State<'_, SpotifyAuthState>,
) -> Result<(), AuraError> {
    log::info!("Starting Spotify OAuth2 PKCE flow");

    let mut auth_manager = state.lock().await;
    auth_manager.set_client_id(client_id);

    // This will open browser and wait for callback
    auth_manager.start_authorization().await?;

    Ok(())
}

#[tauri::command]
async fn spotify_disconnect(db: State<'_, DatabaseState>) -> Result<(), AuraError> {
    log::info!("Disconnecting Spotify");

    // Delete tokens from keyring
    secrets::delete_spotify_tokens()?;

    // Update database
    let database = db.lock().await;
    let mut settings = database.load_settings()?;
    settings.spotify_connected = false;
    database.save_settings(&settings)?;

    Ok(())
}

#[tauri::command]
async fn spotify_get_status() -> Result<SpotifyStatus, AuraError> {
    let connected = secrets::load_spotify_access_token().is_ok();

    Ok(SpotifyStatus {
        connected,
        current_track: if connected {
            Some(SpotifyClient::new().await?.get_current_track().await.ok())
        } else {
            None
        },
    })
}
```

### Playback Control Commands

```rust
#[tauri::command]
async fn spotify_play_track(
    query: String,
    artist: Option<String>,
) -> Result<String, AuraError> {
    log::info!("Playing track: {} by {:?}", query, artist);

    let client = SpotifyClient::new().await?;

    // Search for track
    let tracks = client.search_track(&query, artist.as_deref()).await?;

    if tracks.is_empty() {
        return Err(AuraError::Spotify("No tracks found".to_string()));
    }

    let track = &tracks[0];

    // Play track
    client.play_track(&track.uri).await?;

    Ok(format!("Now playing: {} by {}", track.name, track.artists[0].name))
}

#[tauri::command]
async fn spotify_control_playback(action: String) -> Result<(), AuraError> {
    log::info!("Spotify playback control: {}", action);

    let client = SpotifyClient::new().await?;

    let playback_action = match action.as_str() {
        "pause" => PlaybackAction::Pause,
        "resume" => PlaybackAction::Resume,
        "next" => PlaybackAction::Next,
        "previous" => PlaybackAction::Previous,
        _ => return Err(AuraError::InvalidInput(format!("Unknown action: {}", action))),
    };

    client.control_playback(playback_action).await?;

    Ok(())
}

#[tauri::command]
async fn spotify_get_current_track() -> Result<TrackInfo, AuraError> {
    let client = SpotifyClient::new().await?;
    let current = client.get_current_track().await?;

    Ok(TrackInfo {
        name: current.item.name,
        artist: current.item.artists[0].name.clone(),
        album: current.item.album.name,
        is_playing: current.is_playing,
        progress_ms: current.progress_ms,
        duration_ms: current.item.duration_ms,
    })
}
```

---

## Frontend Integration

### Settings Modal UI

```typescript
// SettingsModal.tsx (Spotify section)

const [spotifyConnected, setSpotifyConnected] = useState(false);
const [spotifyClientId, setSpotifyClientId] = useState("");
const [isConnectingSpotify, setIsConnectingSpotify] = useState(false);

const handleConnectSpotify = async () => {
  setIsConnectingSpotify(true);
  try {
    await invoke("spotify_start_auth", {
      clientId: spotifyClientId,
    });

    // Check status after auth
    const status = await invoke<SpotifyStatus>("spotify_get_status");
    setSpotifyConnected(status.connected);

    if (status.connected) {
      alert("Spotify connected successfully!");
    }
  } catch (error) {
    showErrorToast(error, "Failed to connect Spotify");
  } finally {
    setIsConnectingSpotify(false);
  }
};

const handleDisconnectSpotify = async () => {
  try {
    await invoke("spotify_disconnect");
    setSpotifyConnected(false);
    alert("Spotify disconnected");
  } catch (error) {
    showErrorToast(error, "Failed to disconnect Spotify");
  }
};

// Render
<div className="border-t border-gray-800 pt-4">
  <h3 className="text-lg font-semibold text-gray-200 mb-3">
    Spotify Integration
  </h3>

  {!spotifyConnected ? (
    <div className="space-y-3">
      <div className="space-y-2">
        <Label htmlFor="spotify-client-id" className="text-gray-300">
          Spotify Client ID
        </Label>
        <Input
          type="text"
          id="spotify-client-id"
          value={spotifyClientId}
          onChange={(e) => setSpotifyClientId(e.target.value)}
          placeholder="Enter your Spotify app client ID"
          className="bg-gray-800 text-gray-100 border-gray-700"
        />
        <p className="text-xs text-gray-500">
          Create a Spotify app at{" "}
          <a
            href="https://developer.spotify.com/dashboard"
            target="_blank"
            className="text-blue-400 hover:underline"
          >
            developer.spotify.com
          </a>{" "}
          and copy the Client ID. Set redirect URI to{" "}
          <code className="bg-gray-800 px-1">http://127.0.0.1:8888/callback</code>
        </p>
      </div>

      <Button
        onClick={handleConnectSpotify}
        disabled={!spotifyClientId || isConnectingSpotify}
        className="w-full bg-green-600 hover:bg-green-700 text-white"
      >
        {isConnectingSpotify ? "Connecting..." : "Connect Spotify"}
      </Button>
    </div>
  ) : (
    <div className="space-y-3">
      <div className="flex items-center justify-between bg-gray-800 p-3 rounded">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 bg-green-500 rounded-full"></div>
          <span className="text-gray-300">Connected</span>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={handleDisconnectSpotify}
          className="text-red-400 border-red-400 hover:bg-red-400 hover:text-white"
        >
          Disconnect
        </Button>
      </div>
    </div>
  )}
</div>
```

### Now Playing Component (Optional)

```typescript
// components/NowPlaying.tsx

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface TrackInfo {
  name: string;
  artist: string;
  album: string;
  is_playing: boolean;
  progress_ms: number;
  duration_ms: number;
}

export const NowPlaying: React.FC = () => {
  const [track, setTrack] = useState<TrackInfo | null>(null);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const currentTrack = await invoke<TrackInfo>("spotify_get_current_track");
        setTrack(currentTrack);
      } catch {
        setTrack(null);
      }
    }, 5000); // Poll every 5 seconds

    return () => clearInterval(interval);
  }, []);

  if (!track) return null;

  return (
    <div className="fixed bottom-4 right-4 bg-gray-800 rounded-lg p-4 shadow-lg">
      <div className="flex items-center gap-3">
        <div className="w-12 h-12 bg-gray-700 rounded flex items-center justify-center">
          {track.is_playing ? "▶️" : "⏸️"}
        </div>
        <div>
          <div className="text-sm font-semibold text-gray-100">{track.name}</div>
          <div className="text-xs text-gray-400">{track.artist}</div>
        </div>
      </div>
    </div>
  );
};
```

---

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum SpotifyError {
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),

    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("No active Spotify device found")]
    NoActiveDevice,

    #[error("Spotify Premium required")]
    PremiumRequired,

    #[error("Nothing currently playing")]
    NothingPlaying,

    #[error("Rate limit exceeded, retry after {0} seconds")]
    RateLimited(u64),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}
```

### Graceful Degradation

```rust
pub async fn handle_music_command_gracefully(
    user_text: &str,
) -> Result<String, AuraError> {
    // Check if Spotify is connected
    let spotify_connected = secrets::load_spotify_access_token().is_ok();

    if !spotify_connected {
        return Ok("Spotify is not connected. Please connect your Spotify account in Settings.".to_string());
    }

    // Parse intent
    let intent = MusicIntentParser::parse(user_text);

    match intent {
        MusicIntent::PlaySong { song, artist } => {
            match spotify_play_track(song, artist).await {
                Ok(msg) => Ok(msg),
                Err(SpotifyError::NoActiveDevice) => {
                    Ok("No active Spotify device found. Please open Spotify on one of your devices.".to_string())
                }
                Err(SpotifyError::PremiumRequired) => {
                    Ok("Spotify Premium is required for playback control.".to_string())
                }
                Err(e) => Err(AuraError::Spotify(e.to_string())),
            }
        }
        MusicIntent::Unknown => {
            Ok("I didn't understand that music command. Try 'play <song> by <artist>'.".to_string())
        }
        _ => {
            // Handle other intents...
            Ok(String::new())
        }
    }
}
```

---

## Security Considerations

### 1. PKCE Flow (No Client Secret)

- ✅ Uses PKCE extension (required for native apps)
- ✅ No client secret stored or transmitted
- ✅ SHA256 code challenge

### 2. Token Storage

- ✅ Access token stored in OS keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- ✅ Refresh token stored in OS keyring
- ✅ Never logged or exposed in error messages

### 3. Redirect URI

- ✅ Uses loopback address (http://127.0.0.1:8888)
- ✅ Temporary local HTTP server (only active during auth)
- ✅ Server shut down after callback received

### 4. Scopes

Only request necessary scopes:
- `user-modify-playback-state` - Playback control
- `user-read-playback-state` - Read state
- `user-read-currently-playing` - Current track
- `user-read-email`, `user-read-private` - Profile (for display)
- `playlist-read-private` - User playlists

### 5. Token Refresh

- ✅ Automatic refresh before expiry (5-minute buffer)
- ✅ Retry logic for network failures
- ✅ Graceful fallback if refresh fails

---

## Dependencies

### Cargo.toml Additions

```toml
[dependencies]
# Existing dependencies...

# OAuth2 PKCE flow
oauth2 = "4.4"
url = "2.5"

# HTTP server for OAuth callback
tiny_http = "0.12"

# Regex for intent parsing
regex = "1.10"

# Cryptography for PKCE
sha2 = "0.10"
base64 = "0.21"
rand = "0.8"
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_play_song_with_artist() {
        let intent = MusicIntentParser::parse("play Despacito by Luis Fonsi");
        assert_eq!(
            intent,
            MusicIntent::PlaySong {
                song: "despacito".to_string(),
                artist: Some("luis fonsi".to_string())
            }
        );
    }

    #[test]
    fn test_parse_pause() {
        let intent = MusicIntentParser::parse("pause the music");
        assert_eq!(intent, MusicIntent::Pause);
    }

    // ... more tests
}
```

### Integration Tests

1. **OAuth2 Flow Test**: Manual test with real Spotify account
2. **Token Refresh Test**: Verify automatic refresh works
3. **Playback Control Test**: Test play, pause, next, previous
4. **Device Discovery Test**: Test with Spotify Connect devices
5. **Error Handling Test**: Test with no premium, no devices, etc.

---

## Migration Path

### Phase 1: Core Infrastructure (AC1)
- OAuth2 PKCE authentication
- Token storage in OS keyring
- Database schema updates

### Phase 2: API Client (AC3)
- Spotify API client
- Search, playback control, playlists

### Phase 3: Intent Recognition (AC2)
- Music intent parser
- Integration with voice commands

### Phase 4: Playback Control (AC4)
- Spotify Connect integration
- Device management

### Phase 5: Frontend UI (AC5)
- Settings modal integration
- Optional: Now Playing component

---

## Future Enhancements

1. **Playlist Management**: Create/edit playlists via voice
2. **Queue Management**: Add songs to queue
3. **Favorites/Likes**: Like/save songs
4. **Radio/Discover**: Play artist radio or discover weekly
5. **Multi-room**: Control playback on specific devices
6. **Lyrics**: Display synchronized lyrics
7. **Smart Recommendations**: AI-powered music suggestions

---

## References

- [Spotify Web API Documentation](https://developer.spotify.com/documentation/web-api)
- [OAuth2 PKCE Flow](https://developer.spotify.com/documentation/web-api/tutorials/code-pkce-flow)
- [Spotify API Scopes](https://developer.spotify.com/documentation/web-api/concepts/scopes)
- [oauth2 Rust Crate](https://docs.rs/oauth2/latest/oauth2/)
- [reqwest Documentation](https://docs.rs/reqwest/latest/reqwest/)

---

## Document Version

- **Version**: 1.0
- **Last Updated**: 2025-10-10
- **Author**: Claude Code (AuraPM)
- **Status**: Draft - Ready for Implementation
