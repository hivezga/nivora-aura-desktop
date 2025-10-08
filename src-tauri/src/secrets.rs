use keyring::Entry;

/// Service name for keyring storage
const SERVICE_NAME: &str = "com.nivora.aura-desktop";

/// Key name for API key in keyring
const API_KEY_NAME: &str = "llm_api_key";

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

    match entry.delete_password() {
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
