// Spotify Web API Client
//
// This module provides a comprehensive client for interacting with the Spotify Web API.
// It handles authentication, token refresh, search, playback control, and device management.
//
// Key features:
// - Automatic token refresh before expiry (5-minute buffer)
// - Rate limit handling with exponential backoff
// - Comprehensive error types for all API failures
// - Support for Spotify Connect device control

use chrono::{Duration, Utc};
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

use crate::secrets;
use crate::spotify_auth::{SpotifyAuth, SpotifyAuthError, calculate_token_expiry};

/// Spotify API base URL
const SPOTIFY_API_BASE: &str = "https://api.spotify.com/v1";

/// Buffer time before token expiry to trigger refresh (5 minutes)
const TOKEN_REFRESH_BUFFER: Duration = Duration::minutes(5);

/// Retry delays for rate limiting (exponential backoff)
const RETRY_DELAYS: &[u64] = &[1, 2, 4, 8, 16]; // seconds

/// Spotify API error types
#[derive(Debug, thiserror::Error)]
pub enum SpotifyError {
    #[error("Not authenticated - please connect Spotify in Settings")]
    NotAuthenticated,

    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),

    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("No active Spotify device found - please open Spotify on one of your devices")]
    NoActiveDevice,

    #[error("Spotify Premium required for playback control")]
    PremiumRequired,

    #[error("Nothing currently playing")]
    NothingPlaying,

    #[error("Track not found: {0}")]
    TrackNotFound(String),

    #[error("Playlist not found: {0}")]
    PlaylistNotFound(String),

    #[error("Rate limit exceeded, retry after {0} seconds")]
    RateLimited(u64),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

impl From<SpotifyAuthError> for SpotifyError {
    fn from(err: SpotifyAuthError) -> Self {
        SpotifyError::TokenRefreshFailed(err.to_string())
    }
}

// =============================================================================
// Spotify API Response Types
// =============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub uri: String,
    pub duration_ms: u64,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub popularity: Option<u8>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub uri: String,
    pub images: Vec<Image>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Image {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub tracks: Option<TracksPage>,
}

#[derive(Debug, Deserialize)]
pub struct TracksPage {
    pub items: Vec<Track>,
    pub total: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub uri: String,
    pub description: Option<String>,
    pub images: Vec<Image>,
    pub tracks: PlaylistTracks,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlaylistTracks {
    pub total: u32,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistsResponse {
    pub items: Vec<Playlist>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CurrentlyPlaying {
    pub is_playing: bool,
    pub progress_ms: Option<u64>,
    pub item: Option<Track>,
    pub device: Option<Device>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub r#type: String, // "Computer", "Smartphone", "Speaker", etc.
    pub is_active: bool,
    pub volume_percent: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct DevicesResponse {
    pub devices: Vec<Device>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpotifyUserInfo {
    pub id: String,
    pub display_name: String,
    pub email: String,
    pub product: Option<String>, // "premium" or "free"
}

#[derive(Debug, Serialize)]
struct PlayRequest {
    uris: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position_ms: Option<u64>,
}

// =============================================================================
// Spotify API Client
// =============================================================================

/// Spotify Web API client with automatic token refresh
pub struct SpotifyClient {
    http_client: reqwest::Client,
    client_id: String,
    user_id: Option<i64>, // NEW: User context for multi-user support
}

impl SpotifyClient {
    /// Create a new Spotify client for a specific user
    ///
    /// This is the preferred constructor for multi-user support.
    /// The client will use user-scoped tokens for all operations.
    pub fn new_for_user(client_id: String, user_id: i64) -> Result<Self, SpotifyError> {
        let http_client = reqwest::Client::builder()
            .timeout(StdDuration::from_secs(30))
            .build()
            .map_err(|e| SpotifyError::NetworkError(e.to_string()))?;

        log::debug!("Created Spotify client for user {}", user_id);

        Ok(Self {
            http_client,
            client_id,
            user_id: Some(user_id),
        })
    }

    /// Create a new Spotify client (legacy mode)
    ///
    /// This constructor maintains backward compatibility during migration.
    /// It uses global tokens and should be deprecated once all users migrate.
    pub fn new(client_id: String) -> Result<Self, SpotifyError> {
        let http_client = reqwest::Client::builder()
            .timeout(StdDuration::from_secs(30))
            .build()
            .map_err(|e| SpotifyError::NetworkError(e.to_string()))?;

        log::debug!("Created Spotify client in legacy mode (global tokens)");

        Ok(Self {
            http_client,
            client_id,
            user_id: None, // Legacy mode
        })
    }

    /// Get a valid access token, refreshing if necessary
    ///
    /// This method checks if the current token is expired or will expire soon
    /// (within 5 minutes), and automatically refreshes it if needed.
    /// Uses user-scoped tokens if user_id is present, otherwise falls back to global tokens.
    async fn get_valid_token(&self) -> Result<String, SpotifyError> {
        if let Some(user_id) = self.user_id {
            // Use user-scoped token management
            self.get_valid_user_token(user_id).await
        } else {
            // Legacy global token management
            self.get_valid_global_token().await
        }
    }

    /// Get valid token for specific user (multi-user mode)
    async fn get_valid_user_token(&self, user_id: i64) -> Result<String, SpotifyError> {
        log::debug!("Getting valid token for user {}", user_id);

        // Load current user token
        let access_token = crate::secrets::load_user_spotify_access_token(user_id)
            .map_err(|_| SpotifyError::NotAuthenticated)?;

        // Check expiry
        let expiry = crate::secrets::load_user_spotify_token_expiry(user_id)
            .map_err(|_| SpotifyError::NotAuthenticated)?;

        let now = Utc::now();

        // Refresh if expired or expiring soon
        if expiry < now + TOKEN_REFRESH_BUFFER {
            log::info!("User {} Spotify token expired or expiring soon, refreshing...", user_id);
            self.refresh_user_token(user_id).await?;

            // Load the new token
            crate::secrets::load_user_spotify_access_token(user_id)
                .map_err(|_| SpotifyError::NotAuthenticated)
        } else {
            log::debug!("User {} Spotify token is still valid (expires at {})", user_id, expiry);
            Ok(access_token)
        }
    }

    /// Get valid token for legacy global mode
    async fn get_valid_global_token(&self) -> Result<String, SpotifyError> {
        // Load current token
        let access_token = secrets::load_spotify_access_token()
            .map_err(|_| SpotifyError::NotAuthenticated)?;

        // Check expiry
        let expiry = secrets::load_spotify_token_expiry()
            .map_err(|_| SpotifyError::NotAuthenticated)?;

        let now = Utc::now();

        // Refresh if expired or expiring soon
        if expiry < now + TOKEN_REFRESH_BUFFER {
            log::info!("Spotify token expired or expiring soon, refreshing...");
            self.refresh_global_token().await?;

            // Load the new token
            secrets::load_spotify_access_token()
                .map_err(|_| SpotifyError::NotAuthenticated)
        } else {
            log::debug!("Spotify token is still valid (expires at {})", expiry);
            Ok(access_token)
        }
    }

    /// Refresh the access token using the refresh token
    async fn refresh_token(&self) -> Result<(), SpotifyError> {
        if let Some(user_id) = self.user_id {
            self.refresh_user_token(user_id).await
        } else {
            self.refresh_global_token().await
        }
    }

    /// Refresh token for specific user (multi-user mode)
    async fn refresh_user_token(&self, user_id: i64) -> Result<(), SpotifyError> {
        log::info!("Refreshing Spotify access token for user {}", user_id);

        let refresh_token = crate::secrets::load_user_spotify_refresh_token(user_id)
            .map_err(|_| SpotifyError::NotAuthenticated)?;

        let auth = SpotifyAuth::new(self.client_id.clone());
        let token_response = auth.refresh_access_token(&refresh_token).await?;

        // Save new access token
        crate::secrets::save_user_spotify_access_token(user_id, &token_response.access_token)
            .map_err(|e| SpotifyError::TokenRefreshFailed(e))?;

        // Calculate and save new expiry
        let expiry = calculate_token_expiry(token_response.expires_in);
        crate::secrets::save_user_spotify_token_expiry(user_id, &expiry)
            .map_err(|e| SpotifyError::TokenRefreshFailed(e))?;

        // Update refresh token if provided (not always included in refresh response)
        if let Some(new_refresh_token) = token_response.refresh_token {
            crate::secrets::save_user_spotify_refresh_token(user_id, &new_refresh_token)
                .map_err(|e| SpotifyError::TokenRefreshFailed(e))?;
        }

        log::info!("Successfully refreshed Spotify token for user {}", user_id);
        Ok(())
    }

    /// Refresh token for legacy global mode
    async fn refresh_global_token(&self) -> Result<(), SpotifyError> {
        log::info!("Refreshing Spotify access token");

        let refresh_token = secrets::load_spotify_refresh_token()
            .map_err(|_| SpotifyError::NotAuthenticated)?;

        let auth = SpotifyAuth::new(self.client_id.clone());
        let token_response = auth.refresh_access_token(&refresh_token).await?;

        // Save new access token
        secrets::save_spotify_access_token(&token_response.access_token)
            .map_err(|e| SpotifyError::TokenRefreshFailed(e))?;

        // Calculate and save new expiry
        let expiry = calculate_token_expiry(token_response.expires_in);
        secrets::save_spotify_token_expiry(&expiry)
            .map_err(|e| SpotifyError::TokenRefreshFailed(e))?;

        // Update refresh token if provided (not always included in refresh response)
        if let Some(new_refresh_token) = token_response.refresh_token {
            secrets::save_spotify_refresh_token(&new_refresh_token)
                .map_err(|e| SpotifyError::TokenRefreshFailed(e))?;
        }

        log::info!("✓ Spotify token refreshed successfully (expires at {})", expiry);

        Ok(())
    }

    /// Make an authenticated GET request to the Spotify API
    async fn get(&self, endpoint: &str) -> Result<reqwest::Response, SpotifyError> {
        self.request_with_retry(reqwest::Method::GET, endpoint, None::<&()>)
            .await
    }

    /// Make an authenticated POST request to the Spotify API
    async fn post<T: Serialize>(
        &self,
        endpoint: &str,
        body: Option<&T>,
    ) -> Result<reqwest::Response, SpotifyError> {
        self.request_with_retry(reqwest::Method::POST, endpoint, body)
            .await
    }

    /// Make an authenticated PUT request to the Spotify API
    async fn put<T: Serialize>(
        &self,
        endpoint: &str,
        body: Option<&T>,
    ) -> Result<reqwest::Response, SpotifyError> {
        self.request_with_retry(reqwest::Method::PUT, endpoint, body)
            .await
    }

    /// Make an HTTP request with automatic retry on rate limit
    async fn request_with_retry<T: Serialize>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<&T>,
    ) -> Result<reqwest::Response, SpotifyError> {
        let url = format!("{}{}", SPOTIFY_API_BASE, endpoint);

        for (attempt, &delay_secs) in RETRY_DELAYS.iter().enumerate() {
            let token = self.get_valid_token().await?;

            let mut request = self
                .http_client
                .request(method.clone(), &url)
                .bearer_auth(&token);

            if let Some(body) = body {
                request = request.json(body);
            }

            let response = request
                .send()
                .await
                .map_err(|e| SpotifyError::NetworkError(e.to_string()))?;

            // Check for rate limiting
            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                let retry_after = response
                    .headers()
                    .get(header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(delay_secs);

                log::warn!(
                    "Rate limited by Spotify API, retrying in {} seconds (attempt {}/{})",
                    retry_after,
                    attempt + 1,
                    RETRY_DELAYS.len()
                );

                if attempt < RETRY_DELAYS.len() - 1 {
                    tokio::time::sleep(StdDuration::from_secs(retry_after)).await;
                    continue;
                } else {
                    return Err(SpotifyError::RateLimited(retry_after));
                }
            }

            return Ok(response);
        }

        unreachable!("Retry loop should always return or continue")
    }

    // =========================================================================
    // Search API
    // =========================================================================

    /// Search for tracks by name and optional artist
    ///
    /// Returns up to 10 matching tracks, sorted by popularity.
    pub async fn search_track(
        &self,
        query: &str,
        artist: Option<&str>,
        limit: u8,
    ) -> Result<Vec<Track>, SpotifyError> {
        log::info!("Searching for track: '{}' by {:?}", query, artist);

        let search_query = if let Some(artist) = artist {
            format!("track:{} artist:{}", query, artist)
        } else {
            format!("track:{}", query)
        };

        let endpoint = format!(
            "/search?q={}&type=track&limit={}",
            urlencoding::encode(&search_query),
            limit.min(50) // Spotify max is 50
        );

        let response = self.get(&endpoint).await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Search failed: {}", error_text);
            return Err(SpotifyError::ApiError(error_text));
        }

        let search_response: SearchResponse = response
            .json()
            .await
            .map_err(|e| SpotifyError::ParseError(e.to_string()))?;

        let tracks = search_response
            .tracks
            .map(|t| t.items)
            .unwrap_or_default();

        log::info!("Found {} tracks", tracks.len());

        Ok(tracks)
    }

    /// Get user's playlists
    pub async fn get_user_playlists(&self, limit: u8) -> Result<Vec<Playlist>, SpotifyError> {
        log::info!("Fetching user playlists");

        let endpoint = format!("/me/playlists?limit={}", limit.min(50));

        let response = self.get(&endpoint).await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SpotifyError::ApiError(error_text));
        }

        let playlists_response: PlaylistsResponse = response
            .json()
            .await
            .map_err(|e| SpotifyError::ParseError(e.to_string()))?;

        log::info!("Found {} playlists", playlists_response.items.len());

        Ok(playlists_response.items)
    }

    // =========================================================================
    // Playback Control API
    // =========================================================================

    /// Play a track by URI
    ///
    /// The URI should be in the format "spotify:track:TRACK_ID"
    pub async fn play_track(&self, track_uri: &str) -> Result<(), SpotifyError> {
        log::info!("Playing track: {}", track_uri);

        let play_request = PlayRequest {
            uris: vec![track_uri.to_string()],
            position_ms: Some(0),
        };

        let response = self.put("/me/player/play", Some(&play_request)).await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::ACCEPTED => {
                log::info!("✓ Playback started");
                Ok(())
            }
            StatusCode::NOT_FOUND => Err(SpotifyError::NoActiveDevice),
            StatusCode::FORBIDDEN => Err(SpotifyError::PremiumRequired),
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(SpotifyError::ApiError(error_text))
            }
        }
    }

    /// Pause playback
    pub async fn pause(&self) -> Result<(), SpotifyError> {
        log::info!("Pausing playback");

        let response = self.put("/me/player/pause", None::<&()>).await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::ACCEPTED => {
                log::info!("✓ Playback paused");
                Ok(())
            }
            StatusCode::NOT_FOUND => Err(SpotifyError::NoActiveDevice),
            StatusCode::FORBIDDEN => Err(SpotifyError::PremiumRequired),
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(SpotifyError::ApiError(error_text))
            }
        }
    }

    /// Resume playback
    pub async fn resume(&self) -> Result<(), SpotifyError> {
        log::info!("Resuming playback");

        let response = self.put("/me/player/play", None::<&()>).await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::ACCEPTED => {
                log::info!("✓ Playback resumed");
                Ok(())
            }
            StatusCode::NOT_FOUND => Err(SpotifyError::NoActiveDevice),
            StatusCode::FORBIDDEN => Err(SpotifyError::PremiumRequired),
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(SpotifyError::ApiError(error_text))
            }
        }
    }

    /// Skip to next track
    pub async fn next(&self) -> Result<(), SpotifyError> {
        log::info!("Skipping to next track");

        let response = self.post("/me/player/next", None::<&()>).await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::ACCEPTED => {
                log::info!("✓ Skipped to next track");
                Ok(())
            }
            StatusCode::NOT_FOUND => Err(SpotifyError::NoActiveDevice),
            StatusCode::FORBIDDEN => Err(SpotifyError::PremiumRequired),
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(SpotifyError::ApiError(error_text))
            }
        }
    }

    /// Skip to previous track
    pub async fn previous(&self) -> Result<(), SpotifyError> {
        log::info!("Skipping to previous track");

        let response = self.post("/me/player/previous", None::<&()>).await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::ACCEPTED => {
                log::info!("✓ Skipped to previous track");
                Ok(())
            }
            StatusCode::NOT_FOUND => Err(SpotifyError::NoActiveDevice),
            StatusCode::FORBIDDEN => Err(SpotifyError::PremiumRequired),
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(SpotifyError::ApiError(error_text))
            }
        }
    }

    // =========================================================================
    // Playback State API
    // =========================================================================

    /// Get currently playing track
    pub async fn get_current_track(&self) -> Result<CurrentlyPlaying, SpotifyError> {
        log::info!("Fetching currently playing track");

        let response = self.get("/me/player/currently-playing").await?;

        match response.status() {
            StatusCode::OK => {
                let currently_playing: CurrentlyPlaying = response
                    .json()
                    .await
                    .map_err(|e| SpotifyError::ParseError(e.to_string()))?;

                if currently_playing.item.is_none() {
                    return Err(SpotifyError::NothingPlaying);
                }

                log::info!(
                    "✓ Currently playing: {}",
                    currently_playing
                        .item
                        .as_ref()
                        .map(|t| t.name.as_str())
                        .unwrap_or("Unknown")
                );

                Ok(currently_playing)
            }
            StatusCode::NO_CONTENT => Err(SpotifyError::NothingPlaying),
            StatusCode::NOT_FOUND => Err(SpotifyError::NoActiveDevice),
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(SpotifyError::ApiError(error_text))
            }
        }
    }

    /// Get available Spotify Connect devices
    pub async fn get_devices(&self) -> Result<Vec<Device>, SpotifyError> {
        log::info!("Fetching available Spotify devices");

        let response = self.get("/me/player/devices").await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SpotifyError::ApiError(error_text));
        }

        let devices_response: DevicesResponse = response
            .json()
            .await
            .map_err(|e| SpotifyError::ParseError(e.to_string()))?;

        log::info!("Found {} devices", devices_response.devices.len());

        Ok(devices_response.devices)
    }

    /// Get current user's Spotify profile information
    pub async fn get_current_user(&self) -> Result<SpotifyUserInfo, SpotifyError> {
        log::info!("Fetching current user info");

        let response = self.get("/me").await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SpotifyError::ApiError(error_text));
        }

        let user_info: SpotifyUserInfo = response
            .json()
            .await
            .map_err(|e| SpotifyError::ParseError(e.to_string()))?;

        log::info!("✓ Current user: {}", user_info.display_name);

        Ok(user_info)
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Format track information for display/TTS
pub fn format_track_info(track: &Track) -> String {
    let artists = track
        .artists
        .iter()
        .map(|a| a.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    format!("{} by {}", track.name, artists)
}

/// Format currently playing information for display/TTS
pub fn format_currently_playing(current: &CurrentlyPlaying) -> String {
    if let Some(track) = &current.item {
        let status = if current.is_playing {
            "Now playing"
        } else {
            "Paused"
        };

        format!("{}: {}", status, format_track_info(track))
    } else {
        "Nothing is currently playing".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_track_info() {
        let track = Track {
            id: "test_id".to_string(),
            name: "Bohemian Rhapsody".to_string(),
            uri: "spotify:track:test".to_string(),
            duration_ms: 354000,
            artists: vec![Artist {
                id: "queen".to_string(),
                name: "Queen".to_string(),
                uri: "spotify:artist:queen".to_string(),
            }],
            album: Album {
                id: "album".to_string(),
                name: "A Night at the Opera".to_string(),
                uri: "spotify:album:album".to_string(),
                images: vec![],
            },
            popularity: Some(95),
        };

        let formatted = format_track_info(&track);
        assert_eq!(formatted, "Bohemian Rhapsody by Queen");
    }

    #[test]
    fn test_format_track_info_multiple_artists() {
        let track = Track {
            id: "test_id".to_string(),
            name: "Despacito".to_string(),
            uri: "spotify:track:test".to_string(),
            duration_ms: 228000,
            artists: vec![
                Artist {
                    id: "luis".to_string(),
                    name: "Luis Fonsi".to_string(),
                    uri: "spotify:artist:luis".to_string(),
                },
                Artist {
                    id: "daddy".to_string(),
                    name: "Daddy Yankee".to_string(),
                    uri: "spotify:artist:daddy".to_string(),
                },
            ],
            album: Album {
                id: "album".to_string(),
                name: "Vida".to_string(),
                uri: "spotify:album:album".to_string(),
                images: vec![],
            },
            popularity: Some(98),
        };

        let formatted = format_track_info(&track);
        assert_eq!(formatted, "Despacito by Luis Fonsi, Daddy Yankee");
    }
}
