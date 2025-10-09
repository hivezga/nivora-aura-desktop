# Ollama Resources Setup Guide - Phase 3: Integrated Inference Engine

> **⚠️ IMPORTANT: Models NOT Included in Git Repository**
>
> Ollama models are **NOT tracked in Git** due to size constraints. The Gemma 2B model is 1.7GB,
> which exceeds GitHub's 2GB LFS file limit. You **must** download the model separately
> following the instructions below before building the application.

This document describes how to obtain and organize the bundled Ollama server and LLM model for Aura's self-contained AI capabilities.

## Directory Structure

```
resources/
├── piper/                    # (Phase 2 - already complete)
└── ollama/
    ├── bin/
    │   ├── ollama-linux-amd64           # Linux binary
    │   ├── ollama-windows-amd64.exe     # Windows binary
    │   ├── ollama-darwin-amd64          # macOS Intel binary
    │   └── ollama-darwin-arm64          # macOS Apple Silicon binary
    └── models/
        ├── manifests/
        │   └── registry.ollama.ai/
        │       └── library/
        │           └── gemma/
        │               └── 2b           # Model manifest
        └── blobs/
            └── sha256-*                 # Model blob files
```

## Download Instructions

### 1. Ollama Binaries

**Official releases:** https://github.com/ollama/ollama/releases/latest

Download the standalone binaries (not installers):

**Linux (x86_64):**
```bash
wget https://github.com/ollama/ollama/releases/download/v0.5.4/ollama-linux-amd64
chmod +x ollama-linux-amd64
mkdir -p resources/ollama/bin
mv ollama-linux-amd64 resources/ollama/bin/
```

**Windows (x86_64):**
```bash
wget https://github.com/ollama/ollama/releases/download/v0.5.4/ollama-windows-amd64.exe
mkdir -p resources/ollama/bin
mv ollama-windows-amd64.exe resources/ollama/bin/
```

**macOS (x86_64):**
```bash
wget https://github.com/ollama/ollama/releases/download/v0.5.4/ollama-darwin-amd64
chmod +x ollama-darwin-amd64
mkdir -p resources/ollama/bin
mv ollama-darwin-amd64 resources/ollama/bin/
```

**macOS (ARM64):**
```bash
wget https://github.com/ollama/ollama/releases/download/v0.5.4/ollama-darwin-arm64
chmod +x ollama-darwin-arm64
mkdir -p resources/ollama/bin
mv ollama-darwin-arm64 resources/ollama/bin/
```

### 2. Gemma 2B Model

The model must be pulled using Ollama, then copied from the system location.

**Step 1: Install Ollama temporarily (if not already installed)**
```bash
curl -fsSL https://ollama.com/install.sh | sh
```

**Step 2: Pull the Gemma 2B model**
```bash
ollama pull gemma:2b
```

**Step 3: Locate and copy the model files**

The model is stored in `~/.ollama/models` (macOS/Linux) or `C:\Users\%username%\.ollama\models` (Windows).

**Linux/macOS:**
```bash
# Create destination directory
mkdir -p resources/ollama/models

# Copy the entire models directory structure
cp -r ~/.ollama/models/* resources/ollama/models/
```

**Windows (PowerShell):**
```powershell
# Create destination directory
New-Item -ItemType Directory -Force -Path resources\ollama\models

# Copy the models directory
Copy-Item -Recurse $env:USERPROFILE\.ollama\models\* resources\ollama\models\
```

**Step 4: Verify the structure**
```bash
ls -R resources/ollama/models/
```

Expected output:
```
resources/ollama/models/:
blobs  manifests

resources/ollama/models/manifests:
registry.ollama.ai

resources/ollama/models/manifests/registry.ollama.ai:
library

resources/ollama/models/manifests/registry.ollama.ai/library:
gemma

resources/ollama/models/manifests/registry.ollama.ai/library/gemma:
2b

resources/ollama/models/blobs:
sha256-* (multiple files, ~1.7 GB total)
```

## File Sizes

- Ollama binaries: ~50-100 MB each
- Gemma 2B model: ~1.7 GB (Q4_K_M quantization)
- **Total bundle size per platform: ~1.8 GB**

## Environment Configuration

The bundled Ollama will be configured with:
- `OLLAMA_MODELS`: Points to bundled models directory
- `OLLAMA_HOST`: 127.0.0.1:11434 (localhost only)
- `OLLAMA_KEEP_ALIVE`: 5m (keep model loaded for 5 minutes)

## Notes

- Ollama binaries are CPU-only (GPU support deferred to future optimization)
- The bundled model (gemma:2b) is quantized to Q4_K_M for efficiency (~2B parameters)
- Ollama runs as a managed sidecar process, not a system service
- The process is automatically started on app launch and stopped on app close
- All resources are bundled only for the target platform during build

## License

- Ollama: MIT License
- Gemma 2B: Gemma Terms of Use (Google)
