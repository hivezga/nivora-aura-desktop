# Ollama Models Directory

> **⚠️ MODELS NOT INCLUDED IN GIT REPOSITORY**
>
> Ollama models are **NOT** tracked in Git due to size constraints. The Gemma 2B model
> is 1.7GB, which exceeds GitHub's 2GB LFS file limit.

## Required for Production Builds

This directory should contain the Ollama model files in the following structure:

```
models/
├── manifests/
│   └── registry.ollama.ai/
│       └── library/
│           └── gemma/
│               └── 2b
└── blobs/
    └── sha256-*
```

## How to Download the Model

### Option 1: Pull via Ollama (Recommended)

```bash
# Install Ollama (if not already installed)
curl -fsSL https://ollama.com/install.sh | sh

# Pull the Gemma 2B model
ollama pull gemma:2b

# Copy the model to this directory
cp -r ~/.ollama/models/* resources/ollama/models/
```

### Option 2: Download from GitHub Release

Download the pre-packaged model archive from the [Releases page](https://github.com/hivezga/nivora-aura-desktop/releases):

```bash
# Download the model archive (when available)
wget https://github.com/hivezga/nivora-aura-desktop/releases/download/v0.1.0/ollama-gemma-model.tar.gz

# Extract to this directory
tar -xzf ollama-gemma-model.tar.gz -C resources/ollama/models/
```

## Model Details

- **Model:** Gemma-2-2b-it
- **Size:** ~1.7GB (Q4_K_M quantized)
- **Parameters:** 2B
- **License:** Gemma Terms of Use (Google)
- **Quantization:** Q4_K_M for efficiency

## For Developers

The model files are automatically bundled into production builds when present in this
directory. The build process will package them into the AppImage/DEB/DMG/MSI.

See `OLLAMA_SETUP.md` in the project root for detailed setup instructions.
