# Windows Production Build Guide

## Overview

This guide covers creating a production Windows build of Aura Desktop, including all bundled resources (Ollama, Piper TTS, Whisper models).

## Official Installer Format

**NSIS (.exe) is the official Windows installer format for Aura Desktop.**

### Why NSIS?
- **Lightweight and fast** - Quick installation process
- **Widely compatible** - Works on all Windows versions (7+)
- **No additional dependencies** - WiX toolset not required for builds
- **Better user experience** - Modern installer UI with progress indicators

### Note on MSIX
**MSIX is not currently supported by Tauri 2.** While MSIX offers benefits like package identity and app URI handlers, it's not yet available in the Tauri framework. There are active feature requests for this functionality.

### Alternative: MSI Format
MSI installers are also available via WiX toolset, but **NSIS is the officially supported and recommended format** for Aura releases.

## Prerequisites

### Build Tools

1. **Rust toolchain** (Windows target)
   ```powershell
   rustup target add x86_64-pc-windows-msvc
   ```

2. **Node.js and pnpm** (v8+)
   ```powershell
   npm install -g pnpm
   ```

3. **Windows SDK and Build Tools**
   - Install Visual Studio Build Tools 2019 or later
   - Or install Visual Studio with "Desktop development with C++" workload

4. **WiX Toolset v3** (for MSI builds, optional)
   ```powershell
   # Download from https://wixtoolset.org/releases/
   # Or use chocolatey:
   choco install wixtoolset
   ```

### Required Resources

Before building, ensure all Windows-specific resources are in place:

#### 1. Ollama Binary (v0.12.3)

```powershell
# Download
cd resources/ollama/bin
wget https://github.com/ollama/ollama/releases/download/v0.12.3/ollama-windows-amd64.zip

# Extract (creates ollama-windows-amd64.exe)
Expand-Archive ollama-windows-amd64.zip -DestinationPath .
Remove-Item ollama-windows-amd64.zip
```

**File structure after extraction:**
```
resources/ollama/bin/
├── ollama-windows-amd64.exe     (~32MB - main binary)
├── lib/
│   └── ollama/
│       ├── ggml-base.dll
│       ├── ggml-cpu-*.dll       (CPU optimizations for different architectures)
│       ├── ggml-hip.dll         (~525MB - AMD GPU support)
│       ├── cuda_v12/
│       │   └── *.dll           (NVIDIA CUDA 12.x support)
│       └── cuda_v13/
│           └── *.dll           (NVIDIA CUDA 13.x support)
└── vc_redist.x64.exe           (~25MB - Visual C++ Redistributable)

Total: ~1.9GB (includes GPU acceleration for NVIDIA and AMD)
```

#### 2. Ollama Models

The Gemma 2B model should already be present in `resources/ollama/models/`:
```
resources/ollama/models/
├── blobs/
│   └── sha256-* (multiple files)
└── manifests/
    └── registry.ollama.ai/
        └── library/
            └── gemma/
                └── 2b
```

#### 3. Piper TTS Resources

Already included in the repository:
- Binary: `resources/piper/bin/piper-windows-x86_64.exe`
- Voice models: `resources/piper/voices/*.onnx`
- eSpeak data: `resources/piper/espeak-ng-data/`

#### 4. Whisper Model

The Whisper model is downloaded during first-run setup by the application itself, but you can pre-bundle it:
```powershell
# Download ggml-tiny.bin (or other model)
$modelPath = "$env:LOCALAPPDATA\nivora-aura\models"
New-Item -ItemType Directory -Force -Path $modelPath
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin -OutFile "$modelPath\ggml-tiny.bin"
```

## Build Process

### 1. Install Dependencies

```powershell
pnpm install
```

### 2. Build Frontend

```powershell
pnpm build
```

### 3. Build Tauri Application (NSIS Official Release)

**Build the official NSIS installer:**
```powershell
pnpm tauri build --target nsis
```

This creates the production-ready installer at:
```
src-tauri/target/release/bundle/nsis/aura-desktop_0.1.0_x64-setup.exe
```

**Alternative formats (for testing only):**
```powershell
# MSI installer (requires WiX toolset)
pnpm tauri build --target msi

# Both formats
pnpm tauri build --target nsis msi
```

**Note:** Stick to NSIS for official releases. Only use MSI if you have specific enterprise deployment requirements.

### Build Output

Installers will be created in:
```
src-tauri/target/release/bundle/
├── nsis/
│   └── aura-desktop_0.1.0_x64-setup.exe
└── msi/
    └── aura-desktop_0.1.0_x64_en-US.msi
```

## Bundle Configuration

The current Tauri configuration (`src-tauri/tauri.conf.json`) includes:

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "resources": [
      "models/*",
      "../resources/piper/bin/*",
      "../resources/piper/voices/*",
      "../resources/piper/espeak-ng-data",
      "../resources/ollama/bin/*",
      "../resources/ollama/models"
    ]
  }
}
```

### Platform-Specific Binary Selection

The Rust code automatically selects the correct platform-specific binaries at runtime:

**Ollama** (src-tauri/src/lib.rs:874-884):
```rust
let ollama_binary_name = if cfg!(target_os = "windows") {
    "ollama-windows-amd64.exe"
} else if cfg!(target_os = "macos") {
    if cfg!(target_arch = "aarch64") {
        "ollama-darwin-arm64"
    } else {
        "ollama-darwin-amd64"
    }
} else {
    "ollama-linux-amd64"
};
```

**Piper** (src-tauri/src/lib.rs:991-1001):
```rust
let piper_binary_name = if cfg!(target_os = "windows") {
    "piper-windows-x86_64.exe"
} else if cfg!(target_os = "macos") {
    if cfg!(target_arch = "aarch64") {
        "piper-macos-arm64"
    } else {
        "piper-macos-x86_64"
    }
} else {
    "piper-linux-x86_64"
};
```

## Testing the Build

### 1. Install on Clean Windows 11 VM

1. Create a Windows 11 VM (VirtualBox, VMware, or Hyper-V)
2. Install the NSIS or MSI installer
3. Launch the application

### 2. Verification Checklist

- [ ] Application launches without errors
- [ ] First-run wizard appears (if no prior installation)
- [ ] Whisper model can be downloaded
- [ ] Ollama server starts automatically (check Task Manager)
- [ ] Gemma 2B model is available in settings
- [ ] Push-to-Talk voice input works
- [ ] Speech-to-text transcription works
- [ ] LLM generates responses
- [ ] Text-to-speech works (male/female voices)
- [ ] Conversation history saves and loads
- [ ] Settings persist across restarts
- [ ] Application uninstalls cleanly

### 3. Check Bundled Resources

After installation, verify resources are accessible:

```powershell
# Default install location
cd "C:\Program Files\aura-desktop"

# Check for bundled resources
dir resources\ollama\bin
dir resources\ollama\models
dir resources\piper\bin
dir resources\piper\voices
```

### 4. Monitor Ollama Process

```powershell
# Check if Ollama is running
Get-Process -Name ollama-windows-amd64 -ErrorAction SilentlyContinue

# Check Ollama server logs (if available)
# The app redirects stdout/stderr to null by default
# To debug, modify src-tauri/src/ollama_sidecar.rs:
# .stdout(Stdio::inherit())
# .stderr(Stdio::inherit())
```

## Common Issues

### Issue: Build fails with "bundled resources not found"

**Solution:** Ensure all resources are in place before building:
- Ollama Windows binary in `resources/ollama/bin/`
- Gemma model in `resources/ollama/models/`
- Piper resources in `resources/piper/`

### Issue: Installer size is too large

**Expected size:** ~3.6 GB total for NSIS installer
- **Ollama (with GPU libraries):** ~1.9GB
  - Includes CUDA (NVIDIA) and HIP (AMD) support
  - CPU-optimized libraries for various Intel/AMD processors
- **Gemma 2B model:** ~1.7GB
- **Piper TTS resources:** ~130MB
- **Application code:** ~50MB

This is expected for a fully offline-capable AI assistant with GPU acceleration support.

**Note:** GPU support is automatically utilized if compatible hardware is detected. On systems without dedicated GPUs, only CPU-optimized libraries will be active.

### Issue: Ollama doesn't start on installed app

**Debug steps:**
1. Check Windows Event Viewer for errors
2. Modify `ollama_sidecar.rs` to enable stdout/stderr logging
3. Rebuild and test again
4. Verify Windows Defender isn't blocking the process

### Issue: WiX/MSI build fails

**Solution:** Ensure WiX Toolset v3 is installed:
```powershell
wix --version  # Should show 3.x
```

## Build Time Estimates

- **Frontend build:** 1-2 minutes
- **Rust compilation (release):** 5-10 minutes
- **Bundling (with resources):** 2-5 minutes
- **Total:** ~15-20 minutes

Note: First build will be slower due to dependency compilation.

## Installer Distribution

### File Naming

- NSIS: `aura-desktop_0.1.0_x64-setup.exe`
- MSI: `aura-desktop_0.1.0_x64_en-US.msi`

### Signing (Production)

For production releases, sign the installers:

```powershell
# Using signtool.exe from Windows SDK
signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com aura-desktop_0.1.0_x64-setup.exe
```

### Checksum Generation

```powershell
# Generate SHA256 checksums
Get-FileHash .\aura-desktop_0.1.0_x64-setup.exe -Algorithm SHA256 | Format-List
Get-FileHash .\aura-desktop_0.1.0_x64_en-US.msi -Algorithm SHA256 | Format-List
```

## Next Steps

1. **Build the installer** following this guide
2. **Test on Windows 11 VM** using the verification checklist
3. **Document any issues** encountered
4. **Optimize bundle size** if needed (consider model quantization)
5. **Set up code signing** for production releases

## Resources

- [Tauri Windows Installer Documentation](https://v2.tauri.app/distribute/windows-installer/)
- [Ollama Releases](https://github.com/ollama/ollama/releases)
- [Whisper.cpp Models](https://huggingface.co/ggerganov/whisper.cpp)
- [Piper TTS](https://github.com/rhasspy/piper)

## Support

For build issues, check:
1. This guide
2. Project CLAUDE.md
3. OLLAMA_SETUP.md
4. Open a GitHub issue with build logs
