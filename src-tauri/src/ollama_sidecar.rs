use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// GPU acceleration backend type
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GpuBackend {
    /// NVIDIA CUDA (Windows, Linux)
    Cuda,
    /// AMD ROCm/HIP (Windows, Linux)
    Rocm,
    /// Apple Metal (macOS)
    Metal,
    /// CPU-only (fallback)
    Cpu,
}

impl std::fmt::Display for GpuBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuBackend::Cuda => write!(f, "CUDA (NVIDIA)"),
            GpuBackend::Rocm => write!(f, "ROCm (AMD)"),
            GpuBackend::Metal => write!(f, "Metal (Apple)"),
            GpuBackend::Cpu => write!(f, "CPU"),
        }
    }
}

/// GPU information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpuInfo {
    pub backend: GpuBackend,
    pub available: bool,
    pub device_name: Option<String>,
}

/// Detect available GPU acceleration backend
fn detect_gpu() -> GpuInfo {
    log::info!("Detecting GPU acceleration capabilities...");

    // macOS: Metal is always available on modern Macs
    #[cfg(target_os = "macos")]
    {
        log::info!("✓ macOS detected - Metal acceleration available");
        return GpuInfo {
            backend: GpuBackend::Metal,
            available: true,
            device_name: Some("Apple GPU".to_string()),
        };
    }

    // Windows/Linux: Check for NVIDIA CUDA
    #[cfg(not(target_os = "macos"))]
    {
        // Check for NVIDIA GPU (CUDA)
        if let Some(gpu_name) = detect_nvidia_gpu() {
            log::info!("✓ NVIDIA GPU detected: {}", gpu_name);
            log::info!("  Using CUDA acceleration");
            return GpuInfo {
                backend: GpuBackend::Cuda,
                available: true,
                device_name: Some(gpu_name),
            };
        }

        // Check for AMD GPU (ROCm/HIP)
        if let Some(gpu_name) = detect_amd_gpu() {
            log::info!("✓ AMD GPU detected: {}", gpu_name);
            log::info!("  Using ROCm/HIP acceleration");
            return GpuInfo {
                backend: GpuBackend::Rocm,
                available: true,
                device_name: Some(gpu_name),
            };
        }

        // No GPU detected, fallback to CPU
        log::info!("ℹ No compatible GPU detected, using CPU");
        GpuInfo {
            backend: GpuBackend::Cpu,
            available: false,
            device_name: None,
        }
    }
}

/// Detect NVIDIA GPU (Windows/Linux)
#[cfg(not(target_os = "macos"))]
fn detect_nvidia_gpu() -> Option<String> {
    // Try nvidia-smi command to detect NVIDIA GPU
    let output = Command::new("nvidia-smi")
        .arg("--query-gpu=name")
        .arg("--format=csv,noheader")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let gpu_name = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            if !gpu_name.is_empty() {
                return Some(gpu_name);
            }
        }
    }

    // Windows: Check for CUDA DLLs in bundled resources
    #[cfg(target_os = "windows")]
    {
        // Check if CUDA libraries exist (indicates NVIDIA GPU support)
        if std::path::Path::new("lib/ollama/cuda_v12/ggml-cuda.dll").exists() ||
           std::path::Path::new("lib/ollama/cuda_v13/ggml-cuda.dll").exists() {
            log::debug!("CUDA libraries found in bundle, assuming NVIDIA GPU");
            return Some("NVIDIA GPU (detected via drivers)".to_string());
        }
    }

    // Linux: Check for CUDA runtime
    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new("/usr/local/cuda").exists() ||
           std::path::Path::new("/usr/lib/cuda").exists() {
            log::debug!("CUDA installation found, assuming NVIDIA GPU");
            return Some("NVIDIA GPU (detected via CUDA)".to_string());
        }
    }

    None
}

/// Detect AMD GPU (Windows/Linux)
#[cfg(not(target_os = "macos"))]
fn detect_amd_gpu() -> Option<String> {
    // Windows: Check for AMD drivers via registry or ROCm libraries
    #[cfg(target_os = "windows")]
    {
        // Try AMD's rocm-smi equivalent on Windows
        let output = Command::new("rocm-smi")
            .arg("--showproductname")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let gpu_name = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .find(|line| line.contains("GPU"))
                    .map(|s| s.trim().to_string());
                if let Some(name) = gpu_name {
                    return Some(name);
                }
            }
        }

        // Check for HIP libraries in bundled resources
        if std::path::Path::new("lib/ollama/ggml-hip.dll").exists() {
            log::debug!("HIP libraries found in bundle, checking for AMD GPU");
            return Some("AMD GPU (detected via drivers)".to_string());
        }
    }

    // Linux: Check for ROCm installation
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("rocm-smi")
            .arg("--showproductname")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let gpu_name = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .find(|line| line.contains("GPU"))
                    .map(|s| s.trim().to_string());
                if let Some(name) = gpu_name {
                    return Some(name);
                }
            }
        }

        // Check for ROCm installation
        if std::path::Path::new("/opt/rocm").exists() {
            log::debug!("ROCm installation found, assuming AMD GPU");
            return Some("AMD GPU (detected via ROCm)".to_string());
        }
    }

    None
}

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
    gpu_info: GpuInfo,
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

        // Detect available GPU
        let gpu_info = detect_gpu();
        log::info!("GPU Detection Result:");
        log::info!("  Backend: {}", gpu_info.backend);
        log::info!("  Available: {}", gpu_info.available);
        if let Some(ref device_name) = gpu_info.device_name {
            log::info!("  Device: {}", device_name);
        }

        Ok(OllamaSidecar {
            process: None,
            binary_path,
            models_path,
            host,
            gpu_info,
        })
    }

    /// Get GPU information
    pub fn gpu_info(&self) -> &GpuInfo {
        &self.gpu_info
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
        log::info!("  Acceleration: {} ({})",
                   self.gpu_info.backend,
                   if self.gpu_info.available { "enabled" } else { "CPU fallback" });

        // Spawn Ollama server process
        // Note: Ollama automatically detects and uses available GPU acceleration
        // based on bundled libraries (CUDA, HIP, Metal)
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
