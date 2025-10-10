// Spotify OAuth2 Authentication Module
//
// This module implements the OAuth2 Authorization Code Flow with PKCE (Proof Key for Code Exchange)
// for securely authenticating with Spotify without requiring a client secret.
//
// PKCE is mandatory for native/desktop applications as of April 2025 per Spotify's security requirements.

use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use rand::Rng;
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose, Engine as _};
use tiny_http::{Server, Response};
use chrono::{Utc, DateTime};

/// OAuth2 authorization endpoint
const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";

/// OAuth2 token endpoint
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";

/// Redirect URI for OAuth callback (localhost loopback for PKCE)
const REDIRECT_URI: &str = "http://127.0.0.1:8888/callback";

/// Required OAuth2 scopes for Spotify integration
const REQUIRED_SCOPES: &[&str] = &[
    "user-modify-playback-state",  // Control playback
    "user-read-playback-state",    // Read playback state
    "user-read-currently-playing", // Get current track
    "user-read-email",             // Profile info
    "user-read-private",           // Profile info
    "playlist-read-private",       // Read user playlists
    "playlist-read-collaborative", // Read collaborative playlists
];

/// Spotify authentication error types
#[derive(Debug, thiserror::Error)]
pub enum SpotifyAuthError {
    #[error("OAuth2 authorization failed: {0}")]
    AuthFailed(String),

    #[error("Token exchange failed: {0}")]
    TokenExchangeFailed(String),

    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),

    #[error("HTTP server error: {0}")]
    ServerError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout waiting for OAuth callback")]
    CallbackTimeout,

    #[error("Invalid OAuth response: {0}")]
    InvalidResponse(String),
}

/// OAuth2 token response from Spotify
#[derive(Debug, serde::Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,         // Seconds until expiry
    pub refresh_token: Option<String>,
    pub scope: String,
}

/// Spotify OAuth2 authentication manager
pub struct SpotifyAuth {
    client_id: String,
    http_client: reqwest::Client,
}

impl SpotifyAuth {
    /// Create a new Spotify authentication manager
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            http_client: reqwest::Client::new(),
        }
    }

    /// Start the OAuth2 PKCE authorization flow
    ///
    /// This will:
    /// 1. Generate a PKCE code verifier and challenge
    /// 2. Start a local HTTP server to receive the OAuth callback
    /// 3. Open the user's browser to the Spotify authorization page
    /// 4. Wait for the user to authorize the app
    /// 5. Exchange the authorization code for access/refresh tokens
    ///
    /// Returns the token response with access_token, refresh_token, and expiry
    pub async fn start_authorization(&self) -> Result<TokenResponse, SpotifyAuthError> {
        log::info!("Starting Spotify OAuth2 PKCE authorization flow");

        // 1. Generate PKCE challenge
        let pkce_verifier = generate_pkce_verifier();
        let pkce_challenge = generate_pkce_challenge(&pkce_verifier);

        log::debug!("Generated PKCE verifier and challenge");

        // 2. Start local HTTP server for OAuth callback
        let (server, callback_receiver) = start_callback_server()
            .map_err(|e| SpotifyAuthError::ServerError(e))?;

        log::info!("Started local HTTP server on http://127.0.0.1:8888");

        // 3. Build authorization URL
        let auth_url = self.build_authorization_url(&pkce_challenge);

        log::info!("Opening browser to Spotify authorization page");
        log::debug!("Authorization URL: {}", auth_url);

        // 4. Open system browser
        if let Err(e) = open::that(&auth_url) {
            log::error!("Failed to open browser: {}", e);
            return Err(SpotifyAuthError::AuthFailed(
                format!("Failed to open browser: {}", e)
            ));
        }

        // 5. Wait for OAuth callback (with 120 second timeout)
        log::info!("Waiting for user authorization (timeout: 120s)...");

        let auth_code = wait_for_callback(server, callback_receiver, 120)
            .map_err(|e| {
                log::error!("OAuth callback failed: {}", e);
                SpotifyAuthError::CallbackTimeout
            })?;

        log::info!("Received authorization code from Spotify");

        // 6. Exchange authorization code for tokens
        let token_response = self.exchange_code_for_token(&auth_code, &pkce_verifier).await?;

        log::info!("✓ Spotify authorization successful!");

        Ok(token_response)
    }

    /// Refresh an expired access token using the refresh token
    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, SpotifyAuthError> {
        log::info!("Refreshing Spotify access token");

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.client_id),
        ];

        let response = self.http_client
            .post(SPOTIFY_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| SpotifyAuthError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Token refresh failed: {}", error_text);
            return Err(SpotifyAuthError::TokenRefreshFailed(error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| SpotifyAuthError::InvalidResponse(e.to_string()))?;

        log::info!("✓ Access token refreshed successfully");

        Ok(token_response)
    }

    /// Build the Spotify authorization URL with PKCE parameters
    fn build_authorization_url(&self, pkce_challenge: &str) -> String {
        let scopes = REQUIRED_SCOPES.join(" ");

        format!(
            "{}?client_id={}&response_type=code&redirect_uri={}&code_challenge_method=S256&code_challenge={}&scope={}",
            SPOTIFY_AUTH_URL,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(REDIRECT_URI),
            urlencoding::encode(pkce_challenge),
            urlencoding::encode(&scopes)
        )
    }

    /// Exchange authorization code for access/refresh tokens
    async fn exchange_code_for_token(
        &self,
        auth_code: &str,
        pkce_verifier: &str,
    ) -> Result<TokenResponse, SpotifyAuthError> {
        log::info!("Exchanging authorization code for tokens");

        let params = [
            ("grant_type", "authorization_code"),
            ("code", auth_code),
            ("redirect_uri", REDIRECT_URI),
            ("client_id", &self.client_id),
            ("code_verifier", pkce_verifier),
        ];

        let response = self.http_client
            .post(SPOTIFY_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| SpotifyAuthError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Token exchange failed: {}", error_text);
            return Err(SpotifyAuthError::TokenExchangeFailed(error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| SpotifyAuthError::InvalidResponse(e.to_string()))?;

        log::info!("✓ Token exchange successful");

        Ok(token_response)
    }
}

/// Generate a random PKCE code verifier
///
/// Per RFC 7636: A cryptographically random string using [A-Z] / [a-z] / [0-9] / "-" / "." / "_" / "~"
/// with a minimum length of 43 characters and a maximum length of 128 characters.
fn generate_pkce_verifier() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    const VERIFIER_LENGTH: usize = 128; // Maximum length for best security

    let mut rng = rand::thread_rng();
    (0..VERIFIER_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a PKCE code challenge from the verifier
///
/// Per RFC 7636: BASE64URL(SHA256(ASCII(code_verifier)))
fn generate_pkce_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();

    // Base64 URL-safe encoding (no padding)
    general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

/// Start a local HTTP server to receive the OAuth callback
///
/// Returns the server handle and a receiver channel for the authorization code
fn start_callback_server() -> Result<(Server, Arc<StdMutex<Option<String>>>), String> {
    let server = Server::http("127.0.0.1:8888")
        .map_err(|e| format!("Failed to start HTTP server: {}", e))?;

    let callback_code = Arc::new(StdMutex::new(None));

    Ok((server, callback_code))
}

/// Wait for the OAuth callback and extract the authorization code
fn wait_for_callback(
    server: Server,
    callback_code: Arc<StdMutex<Option<String>>>,
    timeout_secs: u64,
) -> Result<String, String> {
    let timeout = Duration::from_secs(timeout_secs);
    let start = std::time::Instant::now();

    // Handle incoming requests
    for request in server.incoming_requests() {
        // Check for timeout
        if start.elapsed() > timeout {
            return Err("Timeout waiting for OAuth callback".to_string());
        }

        let url = request.url();
        log::debug!("Received HTTP request: {}", url);

        // Parse query parameters
        if url.starts_with("/callback") {
            // Extract authorization code from query string
            let query = url.split('?').nth(1).unwrap_or("");

            let mut auth_code: Option<String> = None;
            let mut error_code: Option<String> = None;

            for param in query.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    match key {
                        "code" => auth_code = Some(urlencoding::decode(value).unwrap_or_default().to_string()),
                        "error" => error_code = Some(urlencoding::decode(value).unwrap_or_default().to_string()),
                        _ => {}
                    }
                }
            }

            // Send success response to browser
            let response_html = if auth_code.is_some() {
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Aura - Spotify Connected</title>
                    <style>
                        body {
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                            display: flex;
                            justify-content: center;
                            align-items: center;
                            height: 100vh;
                            margin: 0;
                            background: linear-gradient(135deg, #1DB954 0%, #191414 100%);
                        }
                        .container {
                            text-align: center;
                            background: white;
                            padding: 3rem;
                            border-radius: 1rem;
                            box-shadow: 0 10px 30px rgba(0,0,0,0.3);
                        }
                        h1 { color: #1DB954; margin-bottom: 1rem; }
                        p { color: #666; }
                    </style>
                </head>
                <body>
                    <div class="container">
                        <h1>✓ Spotify Connected!</h1>
                        <p>You can close this window and return to Aura.</p>
                    </div>
                </body>
                </html>
                "#
            } else {
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Aura - Authorization Failed</title>
                    <style>
                        body {
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                            display: flex;
                            justify-content: center;
                            align-items: center;
                            height: 100vh;
                            margin: 0;
                            background: linear-gradient(135deg, #ff4444 0%, #191414 100%);
                        }
                        .container {
                            text-align: center;
                            background: white;
                            padding: 3rem;
                            border-radius: 1rem;
                            box-shadow: 0 10px 30px rgba(0,0,0,0.3);
                        }
                        h1 { color: #ff4444; margin-bottom: 1rem; }
                        p { color: #666; }
                    </style>
                </head>
                <body>
                    <div class="container">
                        <h1>✗ Authorization Failed</h1>
                        <p>You can close this window and try again in Aura.</p>
                    </div>
                </body>
                </html>
                "#
            };

            let _ = request.respond(Response::from_string(response_html).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap(),
            ));

            // Store authorization code and exit
            if let Some(code) = auth_code {
                *callback_code.lock().unwrap() = Some(code.clone());
                return Ok(code);
            } else {
                return Err(format!("Authorization failed: {}", error_code.unwrap_or_else(|| "unknown error".to_string())));
            }
        }
    }

    Err("Server stopped unexpectedly".to_string())
}

/// Calculate token expiry datetime from expires_in seconds
pub fn calculate_token_expiry(expires_in: u64) -> DateTime<Utc> {
    Utc::now() + chrono::Duration::seconds(expires_in as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_verifier_length() {
        let verifier = generate_pkce_verifier();
        assert_eq!(verifier.len(), 128);
    }

    #[test]
    fn test_pkce_challenge_generation() {
        let verifier = "test_verifier_12345";
        let challenge = generate_pkce_challenge(verifier);

        // Verify challenge is Base64 URL-safe encoded
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
        assert!(!challenge.contains('='));
    }

    #[test]
    fn test_calculate_token_expiry() {
        let expires_in = 3600; // 1 hour
        let expiry = calculate_token_expiry(expires_in);

        let now = Utc::now();
        let diff = (expiry - now).num_seconds();

        // Should be approximately 3600 seconds (within 5 second margin)
        assert!((diff - 3600).abs() < 5);
    }
}
