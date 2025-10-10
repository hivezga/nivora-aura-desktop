use keyring::Entry;

/// Service name for keyring storage
const SERVICE_NAME: &str = "com.nivora.aura-desktop";

/// Key name for API key in keyring
const API_KEY_NAME: &str = "llm_api_key";

/// Keyring entry names for Spotify tokens
const SPOTIFY_ACCESS_TOKEN: &str = "spotify_access_token";
const SPOTIFY_REFRESH_TOKEN: &str = "spotify_refresh_token";
const SPOTIFY_TOKEN_EXPIRY: &str = "spotify_token_expiry";

/// Save API key to the OS keyring
///
/// Uses the native credential storage:
/// - macOS: Keychain
/// - Windows: Credential Manager
/// - Linux: Secret Service (libsecret)
pub fn save_api_key(api_key: &str) -> Result<(), String> {
    log::info!("Saving API key to OS keyring");

    let entry = Entry::new(SERVICE_NAME, API_KEY_NAME)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(api_key)
        .map_err(|e| format!("Failed to save API key to keyring: {}", e))?;

    log::info!("API key saved successfully");

    Ok(())
}

/// Load API key from the OS keyring
///
/// Returns an empty string if no key is stored
pub fn load_api_key() -> Result<String, String> {
    log::info!("Loading API key from OS keyring");

    let entry = Entry::new(SERVICE_NAME, API_KEY_NAME)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(password) => {
            log::info!("API key loaded successfully");
            Ok(password)
        }
        Err(keyring::Error::NoEntry) => {
            log::info!("No API key found in keyring");
            Ok(String::new())
        }
        Err(e) => {
            log::warn!("Failed to load API key from keyring: {}", e);
            // Don't fail - just return empty string
            Ok(String::new())
        }
    }
}

/// Delete API key from the OS keyring
pub fn delete_api_key() -> Result<(), String> {
    log::info!("Deleting API key from OS keyring");

    let entry = Entry::new(SERVICE_NAME, API_KEY_NAME)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.delete_credential() {
        Ok(_) => {
            log::info!("API key deleted successfully");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            log::info!("No API key to delete");
            Ok(())
        }
        Err(e) => Err(format!("Failed to delete API key: {}", e)),
    }
}

// =============================================================================
// Spotify Token Management
// =============================================================================

/// Save Spotify access token to OS keyring
pub fn save_spotify_access_token(token: &str) -> Result<(), String> {
    log::info!("Saving Spotify access token to OS keyring");

    let entry = Entry::new(SERVICE_NAME, SPOTIFY_ACCESS_TOKEN)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(token)
        .map_err(|e| format!("Failed to save Spotify access token: {}", e))?;

    log::debug!("Spotify access token saved successfully");
    Ok(())
}

/// Load Spotify access token from OS keyring
pub fn load_spotify_access_token() -> Result<String, String> {
    log::debug!("Loading Spotify access token from OS keyring");

    let entry = Entry::new(SERVICE_NAME, SPOTIFY_ACCESS_TOKEN)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(token) => {
            log::debug!("Spotify access token loaded successfully");
            Ok(token)
        }
        Err(keyring::Error::NoEntry) => {
            Err("No Spotify access token found".to_string())
        }
        Err(e) => {
            log::warn!("Failed to load Spotify access token: {}", e);
            Err(format!("Failed to load Spotify access token: {}", e))
        }
    }
}

/// Save Spotify refresh token to OS keyring
pub fn save_spotify_refresh_token(token: &str) -> Result<(), String> {
    log::info!("Saving Spotify refresh token to OS keyring");

    let entry = Entry::new(SERVICE_NAME, SPOTIFY_REFRESH_TOKEN)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(token)
        .map_err(|e| format!("Failed to save Spotify refresh token: {}", e))?;

    log::debug!("Spotify refresh token saved successfully");
    Ok(())
}

/// Load Spotify refresh token from OS keyring
pub fn load_spotify_refresh_token() -> Result<String, String> {
    log::debug!("Loading Spotify refresh token from OS keyring");

    let entry = Entry::new(SERVICE_NAME, SPOTIFY_REFRESH_TOKEN)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(token) => {
            log::debug!("Spotify refresh token loaded successfully");
            Ok(token)
        }
        Err(keyring::Error::NoEntry) => {
            Err("No Spotify refresh token found".to_string())
        }
        Err(e) => {
            log::warn!("Failed to load Spotify refresh token: {}", e);
            Err(format!("Failed to load Spotify refresh token: {}", e))
        }
    }
}

/// Save Spotify token expiry timestamp to OS keyring
pub fn save_spotify_token_expiry(expiry: &chrono::DateTime<chrono::Utc>) -> Result<(), String> {
    log::debug!("Saving Spotify token expiry to OS keyring");

    let entry = Entry::new(SERVICE_NAME, SPOTIFY_TOKEN_EXPIRY)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    let expiry_str = expiry.to_rfc3339();

    entry
        .set_password(&expiry_str)
        .map_err(|e| format!("Failed to save Spotify token expiry: {}", e))?;

    log::debug!("Spotify token expiry saved successfully: {}", expiry_str);
    Ok(())
}

/// Load Spotify token expiry timestamp from OS keyring
pub fn load_spotify_token_expiry() -> Result<chrono::DateTime<chrono::Utc>, String> {
    log::debug!("Loading Spotify token expiry from OS keyring");

    let entry = Entry::new(SERVICE_NAME, SPOTIFY_TOKEN_EXPIRY)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(expiry_str) => {
            chrono::DateTime::parse_from_rfc3339(&expiry_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| format!("Failed to parse Spotify token expiry: {}", e))
        }
        Err(keyring::Error::NoEntry) => {
            Err("No Spotify token expiry found".to_string())
        }
        Err(e) => {
            log::warn!("Failed to load Spotify token expiry: {}", e);
            Err(format!("Failed to load Spotify token expiry: {}", e))
        }
    }
}

/// Delete all Spotify tokens from OS keyring
pub fn delete_spotify_tokens() -> Result<(), String> {
    log::info!("Deleting all Spotify tokens from OS keyring");

    // Best-effort deletion - don't fail if a token doesn't exist
    let mut errors = Vec::new();

    // Delete access token
    if let Err(e) = Entry::new(SERVICE_NAME, SPOTIFY_ACCESS_TOKEN)
        .and_then(|entry| entry.delete_credential())
    {
        if !matches!(e, keyring::Error::NoEntry) {
            errors.push(format!("access token: {}", e));
        }
    }

    // Delete refresh token
    if let Err(e) = Entry::new(SERVICE_NAME, SPOTIFY_REFRESH_TOKEN)
        .and_then(|entry| entry.delete_credential())
    {
        if !matches!(e, keyring::Error::NoEntry) {
            errors.push(format!("refresh token: {}", e));
        }
    }

    // Delete token expiry
    if let Err(e) = Entry::new(SERVICE_NAME, SPOTIFY_TOKEN_EXPIRY)
        .and_then(|entry| entry.delete_credential())
    {
        if !matches!(e, keyring::Error::NoEntry) {
            errors.push(format!("token expiry: {}", e));
        }
    }

    if errors.is_empty() {
        log::info!("All Spotify tokens deleted successfully");
        Ok(())
    } else {
        Err(format!("Failed to delete some tokens: {}", errors.join(", ")))
    }
}

/// Check if Spotify tokens are stored (i.e., user is connected)
pub fn is_spotify_connected() -> bool {
    load_spotify_access_token().is_ok() && load_spotify_refresh_token().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_api_key() {
        let test_key = "test-api-key-12345";

        // Save
        save_api_key(test_key).unwrap();

        // Load
        let loaded = load_api_key().unwrap();
        assert_eq!(loaded, test_key);

        // Cleanup
        delete_api_key().unwrap();
    }

    #[test]
    fn test_load_nonexistent_key() {
        // Ensure no key exists
        let _ = delete_api_key();

        // Should return empty string, not error
        let result = load_api_key().unwrap();
        assert_eq!(result, "");
    }
}
