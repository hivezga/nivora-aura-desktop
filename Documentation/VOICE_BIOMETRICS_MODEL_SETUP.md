# Voice Biometrics Model Setup Guide

**Epic:** Voice Biometrics (Speaker Recognition)
**Phase:** Model Integration
**Date:** 2025-10-11
**Status:** Implementation Guide

---

## Overview

This document provides instructions for downloading and setting up the speaker recognition model required for Voice Biometrics in Aura.

---

## Model Selection

**Selected Model:** WeSpeaker ECAPA-TDNN (CAM++)
**Architecture:** ECAPA-TDNN (Emphasized Channel Attention, Propagation and Aggregation in TDNN)
**Training Dataset:** VoxCeleb2 (6,112 speakers)
**Performance:** EER 0.8% on VoxCeleb1-O test set
**Embedding Size:** 192 dimensions
**Model Size:** ~7MB

---

## Download Options

### Option 1: Official sherpa-onnx Release (Recommended)

**Download Location:**
https://github.com/k2-fsa/sherpa-onnx/releases/tag/speaker-recongition-models

**Steps:**
1. Navigate to the releases page
2. Download `wespeaker_en_voxceleb_CAM++.onnx` (or similar WeSpeaker model)
3. Place in Aura models directory (see Installation section)

### Option 2: Hugging Face Mirror

**Alternative Download:**
https://huggingface.co/vibeus/sherpa-onnx-int8

**Model File:** `voxceleb_CAM++_LM.onnx`

**Direct Download URL:**
```bash
wget https://huggingface.co/vibeus/sherpa-onnx-int8/resolve/main/voxceleb_CAM++_LM.onnx
```

---

## Installation

### 1. Determine Model Directory

Aura stores models in the platform-specific local data directory:

**Linux:**
`~/.local/share/com.nivora.aura-desktop/models/speaker-id/`

**macOS:**
`~/Library/Application Support/com.nivora.aura-desktop/models/speaker-id/`

**Windows:**
`C:\Users\<username>\AppData\Local\com.nivora.aura-desktop\models\speaker-id\`

### 2. Create Directory

```bash
# Linux/macOS
mkdir -p ~/.local/share/com.nivora.aura-desktop/models/speaker-id

# Windows PowerShell
New-Item -ItemType Directory -Force -Path "$env:LOCALAPPDATA\com.nivora.aura-desktop\models\speaker-id"
```

### 3. Download and Place Model

**Using wget (Linux/macOS):**
```bash
cd ~/.local/share/com.nivora.aura-desktop/models/speaker-id
wget https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recongition-models/wespeaker_en_voxceleb_CAM++.onnx
```

**OR using Hugging Face mirror:**
```bash
cd ~/.local/share/com.nivora.aura-desktop/models/speaker-id
wget https://huggingface.co/vibeus/sherpa-onnx-int8/resolve/main/voxceleb_CAM++_LM.onnx
```

### 4. Verify Installation

Check that the model file exists:

```bash
# Linux/macOS
ls -lh ~/.local/share/com.nivora.aura-desktop/models/speaker-id/

# Windows PowerShell
Get-ChildItem "$env:LOCALAPPDATA\com.nivora.aura-desktop\models\speaker-id\"
```

Expected output:
- File size: ~7MB
- Extension: `.onnx`

---

## Model Configuration

The model is automatically loaded by Aura's `VoiceBiometrics` engine on initialization. No additional configuration is required after placing the file in the correct directory.

**Model Path (in code):**
```rust
let model_dir = dirs::data_local_dir()
    .ok_or("Failed to get local data directory")?
    .join("com.nivora.aura-desktop/models/speaker-id");

let model_path = model_dir.join("wespeaker_en_voxceleb_CAM++.onnx");
// OR
let model_path = model_dir.join("voxceleb_CAM++_LM.onnx");
```

---

## Troubleshooting

### Model Not Found Error

**Symptom:** Application logs show "Model not found" or "Failed to load speaker recognition model"

**Solution:**
1. Verify model file exists in correct directory (see step 4 above)
2. Check file permissions (should be readable by current user)
3. Ensure filename matches exactly (case-sensitive on Linux/macOS)

### Invalid Model Format Error

**Symptom:** Application fails to initialize with "Invalid ONNX model" error

**Solution:**
1. Re-download the model (file may be corrupted)
2. Verify file size is approximately 7MB
3. Check SHA256 checksum against official release

### Performance Issues

**Symptom:** Slow embedding extraction (>500ms per sample)

**Optimization:**
1. Enable multi-threading in `ExtractorConfig`:
   ```rust
   ExtractorConfig {
       num_threads: Some(4),  // Adjust based on CPU cores
       ...
   }
   ```
2. Consider using GPU acceleration (if available):
   ```rust
   ExtractorConfig {
       provider: Some("cuda".to_string()),  // or "directml" on Windows
       ...
   }
   ```

---

## Alternative Models

If the recommended model is unavailable, these alternatives can be used:

### 3D-Speaker Model

**Download:**
https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recongition-models/3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k.onnx

**Pros:** Good accuracy, multilingual support
**Cons:** Larger file size (~10MB), optimized for Chinese

### Custom Model

To use a custom ONNX model:
1. Ensure model accepts 16kHz audio input
2. Verify output is a fixed-size embedding vector
3. Update `ExtractorConfig.model` path to point to custom model
4. Adjust `EMBEDDING_DIM` constant in `voice_biometrics.rs` if needed

---

## Model Licensing

**WeSpeaker Models:** Apache 2.0 License
**Commercial Use:** Permitted
**Attribution:** Required (see model documentation)

**Note:** Always review the license agreement of the specific model before commercial deployment.

---

## Performance Benchmarks

### Expected Performance

| Operation | Time (CPU) | Time (GPU) |
|-----------|------------|------------|
| Model initialization | ~100ms | ~50ms |
| Embedding extraction (3s audio) | ~15ms | ~5ms |
| Cosine similarity (per comparison) | <1ms | N/A |

### System Requirements

**Minimum:**
- CPU: x86_64 with SSE4.2
- RAM: 512MB available
- Storage: 10MB free

**Recommended:**
- CPU: Modern multi-core x86_64
- RAM: 1GB available
- GPU: CUDA-compatible (optional, for acceleration)

---

## Additional Resources

- **sherpa-onnx Documentation:** https://k2-fsa.github.io/sherpa/onnx/speaker-identification/
- **sherpa-rs Rust API:** https://docs.rs/sherpa-rs/latest/sherpa_rs/
- **WeSpeaker Project:** https://github.com/wenet-e2e/wespeaker
- **ECAPA-TDNN Paper:** https://arxiv.org/abs/2005.07143

---

**Document Version:** 1.0
**Last Updated:** 2025-10-11
**Status:** Ready for Use
