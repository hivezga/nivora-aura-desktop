# GPU Acceleration Guide

## Overview

Aura Desktop automatically detects and utilizes available GPU acceleration for faster LLM inference through the bundled Ollama server. This significantly improves response times for users with NVIDIA or AMD GPUs, or Apple Silicon Macs.

## Supported GPU Backends

### NVIDIA CUDA
**Platforms:** Windows, Linux
**Requirements:**
- NVIDIA GPU with CUDA support (compute capability 5.0+)
- NVIDIA drivers installed
- No additional setup required - CUDA libraries are bundled

**Bundled Libraries:**
- CUDA 12.x support (~1.3 GB)
- CUDA 13.x support (~260 MB)
- CPU-optimized fallbacks

**Detection Method:**
1. Checks for `nvidia-smi` command
2. Queries GPU name and capabilities
3. Automatically enables if NVIDIA GPU detected

### AMD ROCm/HIP
**Platforms:** Windows, Linux
**Requirements:**
- AMD GPU with ROCm support (GCN 4+ architecture)
- AMD drivers installed
- No additional setup required - HIP libraries are bundled

**Bundled Libraries:**
- HIP runtime (~550 MB)
- CPU-optimized fallbacks

**Detection Method:**
1. Checks for `rocm-smi` command
2. Queries GPU name and capabilities
3. Automatically enables if AMD GPU detected

### Apple Metal
**Platforms:** macOS
**Requirements:**
- macOS 11+ (Big Sur or later)
- M1/M2/M3 chip or Intel Mac with Metal support
- Automatic - no additional setup required

**Detection Method:**
- Automatically enabled on all modern Macs
- Uses Apple's Metal Performance Shaders (MPS)

## How It Works

### Automatic Detection
On application startup, Aura detects available GPUs:

1. **Platform Detection:** Identifies OS (Windows/Linux/macOS)
2. **GPU Probe:** Checks for NVIDIA, AMD, or Apple GPUs
3. **Library Verification:** Ensures required acceleration libraries are available
4. **Graceful Fallback:** Falls back to CPU if no compatible GPU found

### Detection Logs
Check the application logs for GPU detection results:

```
[INFO] Detecting GPU acceleration capabilities...
[INFO] ✓ NVIDIA GPU detected: NVIDIA GeForce RTX 3060
[INFO]   Using CUDA acceleration
[INFO] GPU Detection Result:
[INFO]   Backend: CUDA (NVIDIA)
[INFO]   Available: true
[INFO]   Device: NVIDIA GeForce RTX 3060
```

Or for CPU fallback:

```
[INFO] Detecting GPU acceleration capabilities...
[INFO] ℹ No compatible GPU detected, using CPU
[INFO] GPU Detection Result:
[INFO]   Backend: CPU
[INFO]   Available: false
```

### Runtime Behavior
Ollama automatically uses the detected GPU acceleration:

- **With GPU:** LLM inference runs on GPU, significantly faster
- **CPU Fallback:** LLM inference runs on CPU, slower but functional
- **Automatic Selection:** No user configuration required

## Performance Impact

### Expected Speedups (Gemma 2B model)

| Hardware | Tokens/Second | Response Time (100 tokens) |
|----------|---------------|----------------------------|
| **NVIDIA RTX 3060** | 40-60 tok/s | ~2-3 seconds |
| **NVIDIA RTX 4090** | 100-150 tok/s | ~1 second |
| **AMD RX 6800** | 30-50 tok/s | ~3-4 seconds |
| **Apple M1** | 30-45 tok/s | ~3-4 seconds |
| **Apple M2 Pro** | 50-70 tok/s | ~2 seconds |
| **CPU (8-core)** | 5-15 tok/s | ~10-20 seconds |

**Note:** Actual performance varies based on system configuration, thermal conditions, and model complexity.

### GPU Benefits
- **3-10x faster** inference vs CPU
- **Lower latency** for conversational AI
- **Better user experience** with near-instant responses
- **Reduced CPU usage** for other tasks

## Checking GPU Status

### Via Application Logs
Enable debug logging to see GPU status:

1. Set environment variable: `RUST_LOG=info`
2. Launch Aura Desktop
3. Check logs for GPU detection messages

**Windows:**
```powershell
$env:RUST_LOG="info"
.\aura-desktop.exe
```

**macOS/Linux:**
```bash
RUST_LOG=info ./aura-desktop
```

### Via Tauri Command (Programmatic)
Frontend can query GPU status using the `get_gpu_info` command:

```typescript
import { invoke } from '@tauri-apps/api/core'

interface GpuInfo {
  backend: 'Cuda' | 'Rocm' | 'Metal' | 'Cpu'
  available: boolean
  device_name: string | null
}

const gpuInfo: GpuInfo = await invoke('get_gpu_info')
console.log(`GPU: ${gpuInfo.backend}`)
console.log(`Available: ${gpuInfo.available}`)
console.log(`Device: ${gpuInfo.device_name}`)
```

### Via Settings UI (Optional)
A UI indicator in the Settings modal can display GPU status:

```
Inference Device: CUDA (NVIDIA GeForce RTX 3060)
```

Or:

```
Inference Device: CPU (No GPU detected)
```

## Troubleshooting

### GPU Not Detected

**Symptom:** Logs show "No compatible GPU detected, using CPU" despite having a GPU

**NVIDIA Solutions:**
1. **Install NVIDIA Drivers:**
   - Download latest drivers from [NVIDIA website](https://www.nvidia.com/Download/index.aspx)
   - Ensure drivers support CUDA 12.0+
   - Restart system after installation

2. **Verify NVIDIA GPU:**
   ```bash
   nvidia-smi
   ```
   Should display GPU information. If not, driver issue.

3. **Check CUDA Compatibility:**
   - GPU must support compute capability 5.0+
   - Check [NVIDIA CUDA GPUs list](https://developer.nvidia.com/cuda-gpus)

**AMD Solutions:**
1. **Install AMD Drivers:**
   - Download latest drivers from [AMD website](https://www.amd.com/en/support)
   - For ROCm support, install ROCm runtime (Linux)
   - Restart system after installation

2. **Verify AMD GPU:**
   ```bash
   rocm-smi
   ```
   Should display GPU information.

3. **Check ROCm Compatibility:**
   - GPU must be GCN 4.0 architecture or newer
   - Check [AMD ROCm support list](https://rocm.docs.amd.com/en/latest/release/gpu_os_support.html)

**macOS Solutions:**
1. **Update macOS:**
   - Ensure macOS 11+ (Big Sur or later)
   - Metal is built-in, no additional setup needed
   - If not working, update to latest macOS version

### Slow Performance Despite GPU

**Symptom:** GPU detected but performance still slow

**Possible Causes:**
1. **Thermal Throttling:**
   - Check GPU temperature
   - Ensure adequate cooling
   - Clean dust from vents

2. **Other Applications Using GPU:**
   - Close GPU-intensive applications (games, video editing, etc.)
   - Check GPU utilization with monitoring tools

3. **Model Size Too Large:**
   - Gemma 2B fits in most GPUs
   - If using larger models, may require more VRAM
   - Check GPU memory usage

4. **Driver Issues:**
   - Update to latest GPU drivers
   - Restart application after driver update

### Verification Commands

**NVIDIA:**
```bash
# Check GPU info
nvidia-smi

# Monitor GPU usage during inference
nvidia-smi dmon -s u -d 1
```

**AMD:**
```bash
# Check GPU info
rocm-smi

# Monitor GPU usage
rocm-smi --showuse
```

**macOS:**
```bash
# Check GPU usage
sudo powermetrics --samplers gpu_power -i 1000
```

## Platform-Specific Notes

### Windows
- **CUDA and HIP libraries bundled** in the application
- **No separate installation** of CUDA/ROCm required
- **Automatic detection** via nvidia-smi or rocm-smi
- **Windows Defender:** May scan DLLs on first run (one-time delay)

### Linux
- **CUDA libraries bundled**, but NVIDIA drivers must be installed
- **ROCm libraries bundled**, but AMD drivers must be installed
- **Check drivers:** Use `nvidia-smi` or `rocm-smi`
- **Wayland/X11:** No display server dependency for GPU compute

### macOS
- **Metal always available** on modern Macs (2012+)
- **M1/M2/M3 chips:** Unified memory, excellent performance
- **Intel Macs:** Discrete GPU or integrated graphics both supported
- **No configuration needed** - works out of the box

## Technical Implementation

### Detection Flow
```rust
fn detect_gpu() -> GpuInfo {
    #[cfg(target_os = "macos")]
    return GpuInfo { backend: Metal, available: true }

    #[cfg(not(target_os = "macos"))]
    {
        if detect_nvidia_gpu() { return Cuda }
        if detect_amd_gpu() { return Rocm }
        return Cpu
    }
}
```

### Ollama Integration
- **Automatic GPU Selection:** Ollama detects and uses available acceleration
- **No Environment Variables Needed:** Detection is built-in
- **Library Bundling:** All CUDA/HIP libraries included in Windows build
- **Graceful Degradation:** Falls back to CPU if GPU unavailable

### Bundled GPU Libraries

**Windows Bundle Includes:**
```
lib/ollama/
├── cuda_v12/
│   ├── cublas64_12.dll       (114 MB - NVIDIA BLAS)
│   ├── cublasLt64_12.dll     (692 MB - NVIDIA BLAS Light)
│   ├── cudart64_12.dll       (574 KB - CUDA Runtime)
│   └── ggml-cuda.dll         (1.3 GB - GGML CUDA Backend)
├── cuda_v13/
│   ├── cublas64_13.dll       (50 MB)
│   ├── cublasLt64_13.dll     (478 MB)
│   └── ggml-cuda.dll         (260 MB)
├── ggml-hip.dll              (550 MB - AMD HIP Backend)
└── ggml-cpu-*.dll            (CPU-optimized variants)
```

**Total Size:** ~3.4 GB (Windows), ~0 GB additional (macOS/Linux use system libraries)

## Future Enhancements

Planned GPU acceleration improvements:

1. **Multi-GPU Support:**
   - Detect and use multiple GPUs
   - Load balancing across GPUs
   - User-selectable GPU preference

2. **GPU Memory Management:**
   - Monitor VRAM usage
   - Warn if model too large for GPU
   - Automatic offloading to CPU if VRAM exhausted

3. **Performance Profiling:**
   - Measure actual tokens/second
   - Display real-time GPU utilization
   - Performance history graphs

4. **Advanced Settings:**
   - Manual GPU selection (if multiple GPUs)
   - GPU layer offloading (partial GPU usage)
   - Memory limit configuration

## FAQ

**Q: Do I need to install CUDA or ROCm separately?**
A: No! Windows builds include all necessary libraries. Linux/macOS users only need GPU drivers.

**Q: Will this work with older GPUs?**
A: NVIDIA GPUs from 2012+ (Kepler architecture) and AMD GPUs from 2016+ (Polaris/GCN 4) are supported. Older GPUs will fall back to CPU.

**Q: How do I force CPU mode?**
A: Currently automatic only. CPU fallback happens if no GPU detected. Manual selection planned for future release.

**Q: Does GPU acceleration work offline?**
A: Yes! All GPU libraries are bundled. No internet required for GPU acceleration.

**Q: What about Intel GPUs?**
A: Intel GPU support (via oneAPI) is not currently included but may be added in future releases.

**Q: Can I use external GPUs (eGPUs)?**
A: Yes, if properly configured with drivers. Aura treats eGPUs same as internal GPUs.

## Support

If you encounter GPU-related issues:

1. **Check logs** for GPU detection messages
2. **Verify drivers** are installed and up-to-date
3. **Run verification commands** (nvidia-smi, rocm-smi)
4. **Report issue** on GitHub with:
   - GPU model and drivers version
   - Application logs
   - Output of nvidia-smi / rocm-smi

---

**Last Updated:** October 2025
**Ollama Version:** 0.12.3
**CUDA Version:** 12.x / 13.x
**ROCm Version:** 5.x+
