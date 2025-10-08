# Nivora Aura

**A privacy-focused, local-first voice and text AI assistant for your desktop.**

Nivora Aura is a fully open-source voice assistant that runs 100% offline for all core processing. Built with Tauri (Rust + React), it features wake word detection, speech-to-text, high-quality text-to-speech, and connects to any Ollama-compatible LLM server.

## Features

- **🎤 Wake Word Activation**: Hands-free activation with voice activity detection
- **🗣️ Speech-to-Text**: Local transcription powered by Whisper
- **🔊 High-Quality TTS**: Premium voice synthesis using Piper neural TTS
- **💬 Intelligent Chat**: Connect to Ollama or any OpenAI-compatible LLM server
- **🔒 Privacy First**: All voice processing happens locally on your device
- **💾 Conversation History**: SQLite-based persistence with full user control
- **⚙️ Flexible Configuration**: Support for multiple LLM providers and voice models

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Nivora Aura Desktop App                  │
│                     (Tauri + React + Rust)                  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Frontend (React + TypeScript)          │   │
│  │  • Chat UI                                          │   │
│  │  • Settings modal                                   │   │
│  │  • Voice status indicator                           │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          │ Tauri IPC                        │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Backend (Rust)                         │   │
│  │                                                     │   │
│  │  ┌─────────────────┐  ┌──────────────┐            │   │
│  │  │ Native Voice    │  │ TTS Engine   │            │   │
│  │  │ Pipeline        │  │              │            │   │
│  │  │                 │  │ Piper TTS    │            │   │
│  │  │ • whisper-rs    │  │ (subprocess) │            │   │
│  │  │ • cpal audio    │  │ • espeak-ng  │            │   │
│  │  │ • Energy VAD    │  │ • rodio      │            │   │
│  │  └─────────────────┘  └──────────────┘            │   │
│  │                                                     │   │
│  │  ┌─────────────────┐  ┌──────────────┐            │   │
│  │  │ LLM Client      │  │ Database     │            │   │
│  │  │                 │  │              │            │   │
│  │  │ • reqwest HTTP  │  │ SQLite       │            │   │
│  │  │ • OpenAI API    │  │ • Messages   │            │   │
│  │  │   compatible    │  │ • Settings   │            │   │
│  │  └─────────────────┘  └──────────────┘            │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ HTTP (OpenAI-compatible API)
                          ▼
          ┌───────────────────────────────┐
          │   External LLM Server         │
          │                               │
          │   • Ollama (recommended)      │
          │   • LM Studio                 │
          │   • LocalAI                   │
          │   • OpenAI API                │
          │   • Any OpenAI-compatible     │
          └───────────────────────────────┘
```

## Getting Started

### Prerequisites

- **Node.js** (v18+) and **pnpm**
- **Rust** (1.70+) and **Cargo**
- **Tauri CLI** (installed via `pnpm install`)

### System Dependencies

#### Linux (Debian/Ubuntu)

```bash
# Install Piper TTS and its dependencies
sudo apt-get update
sudo apt-get install -y espeak-ng

# Install Piper TTS (download latest release from GitHub)
wget https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_amd64.tar.gz
tar -xzf piper_amd64.tar.gz
sudo cp piper/piper /usr/local/bin/
sudo chmod +x /usr/local/bin/piper

# Configure system linker for Piper libraries (one-time setup)
echo "/usr/local/lib" | sudo tee /etc/ld.so.conf.d/piper.conf
sudo ldconfig
```

#### macOS

```bash
# Install espeak-ng via Homebrew
brew install espeak-ng

# Download Piper for macOS
curl -L https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_arm64.tar.gz -o piper.tar.gz
tar -xzf piper.tar.gz
sudo cp piper/piper /usr/local/bin/
sudo chmod +x /usr/local/bin/piper
```

#### Windows

1. Download and install [espeak-ng](https://github.com/espeak-ng/espeak-ng/releases)
2. Download [Piper for Windows](https://github.com/rhasspy/piper/releases)
3. Add Piper to your system PATH

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/nivora/aura-desktop.git
   cd aura-desktop
   ```

2. **Install dependencies**
   ```bash
   pnpm install
   ```

3. **Download AI Models**

   Create the models directory:
   ```bash
   mkdir -p ~/.local/share/nivora-aura/models
   cd ~/.local/share/nivora-aura/models
   ```

   **Whisper Model (Speech-to-Text):**
   ```bash
   # Download Whisper base.en model (~140MB)
   wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
   ```

   Alternative models (choose based on accuracy vs. speed tradeoff):
   - `ggml-tiny.en.bin` - Fastest, least accurate (~75MB)
   - `ggml-base.en.bin` - Recommended balance (~140MB)
   - `ggml-small.en.bin` - Higher accuracy, slower (~465MB)

   **Piper Voice Model (Text-to-Speech):**
   ```bash
   # Download a high-quality English voice (e.g., Lessac medium quality)
   wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx
   wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx.json
   ```

   Browse more voices at: https://huggingface.co/rhasspy/piper-voices

4. **Set up Ollama (LLM Server)**

   **Install Ollama:**
   ```bash
   # Linux/macOS
   curl -fsSL https://ollama.com/install.sh | sh

   # Or download from: https://ollama.com/download
   ```

   **Download a model and start the server:**
   ```bash
   # Download a recommended model (llama3.2 is fast and capable)
   ollama pull llama3.2:3b

   # Start Ollama server (runs on http://localhost:11434 by default)
   ollama serve
   ```

   **Note:** Keep `ollama serve` running in a separate terminal while using Aura.

### Running Aura

Launch the application in development mode:

```bash
# Linux
WEBKIT_DISABLE_COMPOSITING_MODE=1 pnpm tauri dev

# macOS/Windows
pnpm tauri dev
```

**Important:** On Linux, the `WEBKIT_DISABLE_COMPOSITING_MODE=1` environment variable is required for proper rendering.

### Configuration

On first launch, open Settings (gear icon) and configure:

1. **LLM Provider**: Select "Ollama" (or "Custom" for other providers)
2. **Server Address**: `http://localhost:11434/v1` (default Ollama)
3. **Model Name**: `llama3.2:3b` (or whichever model you downloaded)
4. **STT Model**: `ggml-base.en.bin` (or your chosen Whisper model)
5. **TTS Voice**: `en_US-lessac-medium.onnx` (or your chosen Piper voice)
6. **Wake Word**: Enable/disable voice activation
7. **VAD Sensitivity**: Adjust microphone sensitivity (0.01-0.1, default: 0.02)

Click **Save** to apply changes. The voice pipeline will reload automatically.

## Usage

### Voice Interaction

1. **Wake Word Mode**: Say anything loudly or clearly to activate listening
   - Status indicator turns blue when listening
   - Speak your question or command
   - Aura responds with voice and text

2. **Push-to-Talk Mode**: Click the microphone button
   - Speak while the button is active
   - Release or wait for silence detection
   - Aura processes and responds

### Text Interaction

- Type messages directly in the chat input
- Press Enter to send
- Aura responds with both text and voice

### Conversation Management

- **New Conversation**: Click "+" button in sidebar
- **Switch Conversations**: Click any conversation in the list
- **Delete Conversation**: Click trash icon (requires confirmation)

## Building for Production

Build platform-specific installers:

```bash
pnpm tauri build
```

This creates optimized bundles in `src-tauri/target/release/bundle/`:
- **Linux**: `.deb`, `.AppImage`
- **macOS**: `.dmg`, `.app`
- **Windows**: `.exe`, `.msi`

## Troubleshooting

### "libpiper_phonemize.so.1: cannot open shared object file" (Linux)

Ensure you've completed the linker configuration step:
```bash
echo "/usr/local/lib" | sudo tee /etc/ld.so.conf.d/piper.conf
sudo ldconfig
```

### Blank Window or Rendering Issues (Linux)

Launch with the required environment variable:
```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 pnpm tauri dev
```

### "Failed to connect to Ollama server"

1. Verify Ollama is running: `ollama list` should show installed models
2. Check server address in Settings: default is `http://localhost:11434/v1`
3. Test manually: `curl http://localhost:11434/api/tags`

### Voice Not Working

1. Check microphone permissions (system will prompt on first use)
2. Verify models are downloaded to `~/.local/share/nivora-aura/models/`
3. Check Settings for correct model filenames
4. Review console logs for detailed error messages

### Piper TTS Not Found

1. Verify Piper installation: `which piper` should show `/usr/local/bin/piper`
2. Check espeak-ng: `espeak-ng --version` should succeed
3. Ensure voice model and config (.onnx + .onnx.json) are both present

## Development

### Project Structure

```
aura-desktop/
├── src/                    # React frontend source
│   ├── App.tsx            # Main app component
│   ├── store.ts           # Zustand state management
│   └── components/        # UI components
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Tauri commands and app initialization
│   │   ├── native_voice.rs # Voice pipeline (STT, VAD, audio)
│   │   ├── tts.rs         # Piper TTS subprocess integration
│   │   ├── llm.rs         # OpenAI-compatible LLM client
│   │   ├── database.rs    # SQLite persistence
│   │   └── secrets.rs     # OS keyring integration
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
└── Documentation/         # Architecture & requirements docs
```

### Available Commands

```bash
# Frontend development
pnpm dev              # Start Vite dev server (port 1420)
pnpm build            # Build frontend
pnpm preview          # Preview production build

# Tauri development
pnpm tauri dev        # Run app in development mode
pnpm tauri build      # Build production app bundles

# Rust backend
cd src-tauri
cargo build           # Build Rust backend
cargo test            # Run Rust tests
cargo clippy          # Lint Rust code
```

### Tech Stack

- **Framework**: Tauri v2 (Rust + WebView)
- **Frontend**: React 19 + TypeScript + Vite + Tailwind CSS
- **State Management**: Zustand
- **Backend**: Rust with Tokio async runtime
- **Database**: SQLite (rusqlite)
- **HTTP Client**: reqwest
- **Audio I/O**: cpal
- **Speech-to-Text**: whisper-rs (Whisper.cpp bindings)
- **Text-to-Speech**: Piper (subprocess) + rodio (playback)
- **Secure Storage**: keyring (OS native keychain)

## Privacy & Security

- **Local-First**: All voice processing (STT, TTS, wake word detection) happens on your device
- **No Telemetry**: Zero analytics, tracking, or data collection
- **User Control**: You choose which LLM server to connect to
- **Secure Credentials**: API keys stored in OS native keychain (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux)
- **Data Ownership**: Conversation history stored locally in SQLite database at `~/.local/share/nivora-aura/`

## License

[Your License Here]

## Support

For issues, questions, and feature requests, please visit:
- **GitHub Issues**: https://github.com/nivora/aura-desktop/issues
- **Documentation**: https://github.com/nivora/aura-desktop/wiki

## Acknowledgments

- **Whisper.cpp**: Speech recognition by OpenAI (ggerganov's C++ implementation)
- **Piper TTS**: Neural text-to-speech by Rhasspy
- **Ollama**: Local LLM inference server
- **Tauri**: Cross-platform desktop framework
