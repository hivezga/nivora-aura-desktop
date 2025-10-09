use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Ollama sidecar process manager
///
/// Manages the lifecycle of a bundled Ollama server as a sidecar process.
/// The server is started automatically when the app launches and gracefully
/// terminated when the app closes.
pub struct OllamaSidecar {
    process: Option<Child>,
    binary_path: PathBuf,
    models_path: PathBuf,
    host: String,
}

impl OllamaSidecar {
    /// Create a new Ollama sidecar manager
    ///
    /// # Arguments
    /// * `binary_path` - Path to the Ollama executable
    /// * `models_path` - Path to the directory containing Ollama models
    /// * `host` - Host and port to bind to (e.g., "127.0.0.1:11434")
    pub fn new(binary_path: PathBuf, models_path: PathBuf, host: String) -> Result<Self, String> {
        log::info!("Initializing Ollama sidecar manager");
        log::info!("  Binary: {:?}", binary_path);
        log::info!("  Models: {:?}", models_path);
        log::info!("  Host: {}", host);

        // Verify binary exists
        if !binary_path.exists() {
            return Err(format!(
                "Ollama binary not found at: {:?}",
                binary_path
            ));
        }

        // Create models directory if it doesn't exist
        if !models_path.exists() {
            std::fs::create_dir_all(&models_path)
                .map_err(|e| format!("Failed to create models directory: {}", e))?;
        }

        Ok(OllamaSidecar {
            process: None,
            binary_path,
            models_path,
            host,
        })
    }

    /// Start the Ollama server process
    ///
    /// This spawns the Ollama server as a background process with the appropriate
    /// environment variables configured.
    ///
    /// # Returns
    /// Ok(()) if the process started successfully, Err with details if failed
    pub fn start(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            log::warn!("Ollama sidecar already running");
            return Ok(());
        }

        log::info!("Starting Ollama sidecar process...");

        // Spawn Ollama server process
        let child = Command::new(&self.binary_path)
            .arg("serve")
            .env("OLLAMA_MODELS", &self.models_path)
            .env("OLLAMA_HOST", &self.host)
            .env("OLLAMA_KEEP_ALIVE", "5m")
            .env("OLLAMA_NUM_PARALLEL", "1") // Limit to 1 request at a time for resource efficiency
            .stdout(Stdio::null()) // Suppress stdout
            .stderr(Stdio::null()) // Suppress stderr (change to inherit() for debugging)
            .spawn()
            .map_err(|e| format!("Failed to spawn Ollama process: {}", e))?;

        let pid = child.id();
        self.process = Some(child);

        log::info!("✓ Ollama sidecar started successfully (PID: {})", pid);
        log::info!("  Waiting for server to become ready...");

        Ok(())
    }

    /// Wait for the Ollama server to become ready
    ///
    /// This polls the Ollama API endpoint until it responds successfully or times out.
    ///
    /// # Arguments
    /// * `timeout_secs` - Maximum number of seconds to wait
    ///
    /// # Returns
    /// Ok(()) if server becomes ready, Err if timeout or server fails
    pub async fn wait_for_ready(&self, timeout_secs: u64) -> Result<(), String> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let api_url = format!("http://{}/api/tags", self.host);

        log::info!("Polling Ollama API at: {}", api_url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        loop {
            // Check if we've exceeded the timeout
            if start.elapsed() > timeout {
                return Err(format!(
                    "Ollama server did not become ready within {} seconds",
                    timeout_secs
                ));
            }

            // Try to connect to the API
            match client.get(&api_url).send().await {
                Ok(response) if response.status().is_success() => {
                    log::info!("✓ Ollama server is ready! (took {:.1}s)", start.elapsed().as_secs_f32());
                    return Ok(());
                }
                Ok(response) => {
                    log::debug!(
                        "Ollama API returned non-success status: {}",
                        response.status()
                    );
                }
                Err(e) => {
                    log::debug!("Ollama API not ready yet: {}", e);
                }
            }

            // Wait a bit before retrying
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    /// Check if the Ollama server process is running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(None) => true, // Still running
                Ok(Some(status)) => {
                    log::warn!("Ollama process exited with status: {}", status);
                    false
                }
                Err(e) => {
                    log::error!("Error checking Ollama process status: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Gracefully stop the Ollama server
    ///
    /// This sends a termination signal to the Ollama process and waits for it to exit.
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.process.take() {
            log::info!("Stopping Ollama sidecar process...");

            // Try to terminate gracefully
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                let pid = Pid::from_raw(child.id() as i32);
                if let Err(e) = kill(pid, Signal::SIGTERM) {
                    log::warn!("Failed to send SIGTERM to Ollama: {}", e);
                }
            }

            #[cfg(windows)]
            {
                if let Err(e) = child.kill() {
                    log::warn!("Failed to terminate Ollama process: {}", e);
                }
            }

            // Wait for process to exit (with timeout)
            let wait_result = std::thread::spawn(move || child.wait())
                .join()
                .map_err(|_| "Failed to join wait thread".to_string())?;

            match wait_result {
                Ok(status) => {
                    log::info!("✓ Ollama sidecar stopped (exit status: {})", status);
                    Ok(())
                }
                Err(e) => {
                    log::error!("Error waiting for Ollama to exit: {}", e);
                    Err(format!("Failed to wait for process exit: {}", e))
                }
            }
        } else {
            log::warn!("Ollama sidecar not running");
            Ok(())
        }
    }
}

impl Drop for OllamaSidecar {
    fn drop(&mut self) {
        if self.process.is_some() {
            log::info!("OllamaSidecar being dropped, stopping process...");
            let _ = self.stop();
        }
    }
}
