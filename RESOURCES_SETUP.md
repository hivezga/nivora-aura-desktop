# Resources Setup Guide - Phase 2: Self-Contained Voice

This document describes how to obtain and organize the bundled resources for Aura's self-contained voice capabilities.

## Directory Structure

```
resources/
└── piper/
    ├── bin/
    │   ├── piper-linux-x86_64         # Linux binary
    │   ├── piper-windows-x86_64.exe   # Windows binary
    │   ├── piper-macos-x86_64         # macOS Intel binary
    │   └── piper-macos-arm64          # macOS Apple Silicon binary
    ├── voices/
    │   ├── en_US-lessac-medium.onnx       # Male voice model
    │   ├── en_US-lessac-medium.onnx.json  # Male voice config
    │   ├── en_US-amy-medium.onnx          # Female voice model
    │   └── en_US-amy-medium.onnx.json     # Female voice config
    └── espeak-ng-data/                    # eSpeak phoneme data directory
```

## Download Instructions

### 1. Piper Binaries

**Official releases:** https://github.com/rhasspy/piper/releases/latest

Download the following:

**Linux (x86_64):**
```bash
wget https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_x86_64.tar.gz
tar -xzf piper_linux_x86_64.tar.gz
cp piper/piper resources/piper/bin/piper-linux-x86_64
chmod +x resources/piper/bin/piper-linux-x86_64
```

**Windows (x86_64):**
```bash
wget https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_windows_amd64.zip
unzip piper_windows_amd64.zip
cp piper/piper.exe resources/piper/bin/piper-windows-x86_64.exe
```

**macOS (x86_64):**
```bash
wget https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_macos_x64.tar.gz
tar -xzf piper_macos_x64.tar.gz
cp piper/piper resources/piper/bin/piper-macos-x86_64
chmod +x resources/piper/bin/piper-macos-x86_64
```

**macOS (ARM64):**
```bash
wget https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_macos_aarch64.tar.gz
tar -xzf piper_macos_aarch64.tar.gz
cp piper/piper resources/piper/bin/piper-macos-arm64
chmod +x resources/piper/bin/piper-macos-arm64
```

### 2. Voice Models

**Official models:** https://huggingface.co/rhasspy/piper-voices/tree/main/en/en_US

**Male Voice (lessac-medium):**
```bash
cd resources/piper/voices
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx.json
```

**Female Voice (amy-medium):**
```bash
cd resources/piper/voices
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/amy/medium/en_US-amy-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/amy/medium/en_US-amy-medium.onnx.json
```

### 3. eSpeak-NG Data

**From Piper release** (included in all Piper downloads):
```bash
# After extracting any Piper archive (from step 1):
cp -r piper/espeak-ng-data resources/piper/
```

## Verification

After setup, verify the structure:

```bash
ls -R resources/
```

Expected output:
```
resources/:
piper

resources/piper:
bin  espeak-ng-data  voices

resources/piper/bin:
piper-linux-x86_64  piper-macos-arm64  piper-macos-x86_64  piper-windows-x86_64.exe

resources/piper/voices:
en_US-amy-medium.onnx  en_US-amy-medium.onnx.json  en_US-lessac-medium.onnx  en_US-lessac-medium.onnx.json

resources/piper/espeak-ng-data:
[many phoneme data files]
```

## File Sizes

- Piper binaries: ~5-15 MB each
- Voice models: ~25-30 MB each
- espeak-ng-data: ~5 MB
- **Total bundle size: ~120-150 MB**

## Notes

- All resources are open-source (MIT/Apache 2.0 licenses)
- Resources are platform-specific and will only be bundled for the target platform during build
- Tauri's bundler handles platform selection automatically
