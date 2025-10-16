# Nivora Aura

**A privacy-focused, local-first voice and text AI assistant for your desktop.**

Nivora Aura is a fully open-source voice assistant that runs 100% offline for all core processing. Built with Tauri (Rust + React), it features wake word detection, speech-to-text, high-quality text-to-speech, and connects to any Ollama-compatible LLM server.

## Features

### Core Voice Assistant
- **🎤 Wake Word Activation**: Hands-free activation with voice activity detection
- **🗣️ Speech-to-Text**: Local transcription powered by Whisper
- **🔊 High-Quality TTS**: Premium voice synthesis using Piper neural TTS
- **💬 Intelligent Chat**: Connect to Ollama or any OpenAI-compatible LLM server
- **🔒 Privacy First**: All voice processing happens locally on your device
- **💾 Conversation History**: SQLite-based persistence with full user control
- **⚙️ Flexible Configuration**: Support for multiple LLM providers and voice models

### Advanced Integrations
- **🎵 Spotify Music Control**: OAuth2-based music playback with voice commands
  - Search and play tracks, playlists, and albums
  - Playback control (play/pause/next/previous)
  - Device selection and currently playing info
  - Natural language commands ("play my workout playlist")

- **🏠 Home Assistant Integration**: Real-time smart home control via WebSocket
  - Control lights, climate, locks, covers, and media players
  - Real-time entity state synchronization
  - Natural language commands ("turn on kitchen lights")
  - Device filtering and management UI

- **🌐 Web Search RAG**: Privacy-focused retrieval-augmented generation
  - Dual backend support (SearXNG, Brave Search)
  - Context augmentation for smarter responses
  - Opt-in only (disabled by default)
  - Graceful offline fallback

- **👤 Voice Biometrics**: Multi-user speaker recognition
  - Offline speaker identification using WeSpeaker ECAPA-TDNN
  - User enrollment with voice samples
  - Personalized responses based on speaker identity
  - <20ms latency overhead

## Architecture

```
┌───────────────────────────────────────────────────────────────────────────┐
│                      Nivora Aura Desktop App                              │
│                       (Tauri + React + Rust)                              │
│                                                                           │
│  ┌───────────────────────────────────────────────────────────────────┐   │
│  │                Frontend (React + TypeScript)                      │   │
│  │  • Chat UI                  • Home Assistant Devices View        │   │
│  │  • Settings modal           • Spotify Settings                   │   │
│  │  • Voice status indicator   • User Profile Management            │   │
│  └───────────────────────────────────────────────────────────────────┘   │
│                              │                                            │
│                              │ Tauri IPC (45 commands)                    │
│                              ▼                                            │
│  ┌───────────────────────────────────────────────────────────────────┐   │
│  │                    Backend (Rust - 10,132 LOC)                    │   │
│  │                                                                   │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐       │   │
│  │  │ Voice        │  │ TTS Engine   │  │ Voice Biometrics │       │   │
│  │  │ Pipeline     │  │              │  │                  │       │   │
│  │  │ • whisper-rs │  │ • Piper TTS  │  │ • sherpa-rs      │       │   │
│  │  │ • cpal audio │  │ • espeak-ng  │  │ • ECAPA-TDNN     │       │   │
│  │  │ • Energy VAD │  │ • rodio      │  │ • Speaker ID     │       │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘       │   │
│  │                                                                   │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐       │   │
│  │  │ LLM Client   │  │ Web Search   │  │ Database         │       │   │
│  │  │              │  │ RAG          │  │                  │       │   │
│  │  │ • reqwest    │  │ • SearXNG    │  │ SQLite           │       │   │
│  │  │ • OpenAI API │  │ • Brave API  │  │ • Conversations  │       │   │
│  │  │   compatible │  │ • Context    │  │ • User Profiles  │       │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘       │   │
│  │                                                                   │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐       │   │
│  │  │ Spotify      │  │ Home         │  │ Secrets          │       │   │
│  │  │ Integration  │  │ Assistant    │  │ Management       │       │   │
│  │  │              │  │              │  │                  │       │   │
│  │  │ • OAuth2     │  │ • WebSocket  │  │ • OS Keyring     │       │   │
│  │  │ • Music NLU  │  │ • REST API   │  │ • API Keys       │       │   │
│  │  │ • Playback   │  │ • Entity Mgr │  │ • OAuth Tokens   │       │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘       │   │
│  └───────────────────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────────────────────┘
           │                    │                    │
           │                    │                    │
           ▼                    ▼                    ▼
    ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
    │ LLM Server  │     │  Spotify    │     │    Home     │
    │             │     │  Web API    │     │  Assistant  │
    │ • Ollama    │     │             │     │             │
    │ • LM Studio │     │ • OAuth2    │     │ • WebSocket │
    │ • LocalAI   │     │ • Playback  │     │ • REST API  │
    │ • OpenAI    │     │ • Playlists │     │ • Entities  │
    └─────────────┘     └─────────────┘     └─────────────┘
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

#### Core Settings
1. **LLM Provider**: Select "Ollama" (or "Custom" for other providers)
2. **Server Address**: `http://localhost:11434/v1` (default Ollama)
3. **Model Name**: `llama3.2:3b` (or whichever model you downloaded)
4. **STT Model**: `ggml-base.en.bin` (or your chosen Whisper model)
5. **TTS Voice**: `en_US-lessac-medium.onnx` (or your chosen Piper voice)
6. **Wake Word**: Enable/disable voice activation
7. **VAD Sensitivity**: Adjust microphone sensitivity (0.01-0.1, default: 0.02)

#### Optional Integrations

**Spotify Music Control:**
1. Create a Spotify app at [developer.spotify.com/dashboard](https://developer.spotify.com/dashboard)
2. Add `http://localhost:8888/callback` as a Redirect URI
3. Copy your Client ID and paste it in Settings → Spotify
4. Click "Connect to Spotify" and authorize the app
5. Use voice commands like "play Shape of You by Ed Sheeran"

**Home Assistant Integration:**
1. In your Home Assistant, create a Long-Lived Access Token (Profile → Security)
2. In Aura Settings → Home Assistant, enter:
   - Server URL: `http://your-ha-ip:8123` (or `https://` if using SSL)
   - Access Token: Paste the token from step 1
3. Click "Connect" to establish WebSocket connection
4. View and control devices in the "Devices" tab
5. Use voice commands like "turn on the living room lights"

**Web Search (RAG):**
1. In Settings → Online Mode, enable "Online Mode"
2. Choose backend:
   - **SearXNG**: Select a public instance from the dropdown (no API key needed)
   - **Brave Search**: Enter your Brave Search API key
3. Configure result count (1-20 results)
4. Aura will now augment responses with web search context

**Voice Biometrics:**
1. In Settings → Voice Biometrics, click "Enroll New User"
2. Record 3-5 voice samples when prompted
3. Aura will identify you automatically during voice interactions
4. Multi-user support for personalized Spotify/HA commands (coming soon)

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
├── src/                         # React frontend source (3,144 LOC)
│   ├── App.tsx                 # Main app component
│   ├── store.ts                # Zustand state management
│   ├── components/             # UI components
│   │   ├── ChatView.tsx        # Message display
│   │   ├── DevicesView.tsx     # Home Assistant devices UI
│   │   ├── SpotifySettings.tsx # Spotify connection UI
│   │   └── HomeAssistantSettings.tsx # HA connection UI
│   └── ...
├── src-tauri/                   # Rust backend (10,132 LOC)
│   ├── src/
│   │   ├── lib.rs              # 45 Tauri commands, app initialization
│   │   ├── database.rs         # SQLite (4 tables: conversations, messages, settings, user_profiles)
│   │   ├── native_voice.rs     # Voice pipeline (STT, VAD, audio)
│   │   ├── voice_biometrics.rs # Speaker recognition (sherpa-rs)
│   │   ├── tts.rs              # Piper TTS subprocess
│   │   ├── llm.rs              # OpenAI-compatible LLM client
│   │   ├── web_search.rs       # RAG with SearXNG/Brave
│   │   ├── spotify_client.rs   # Spotify Web API client
│   │   ├── spotify_auth.rs     # OAuth2 PKCE flow
│   │   ├── music_intent.rs     # Music command NLU
│   │   ├── ha_client.rs        # Home Assistant WebSocket + REST
│   │   ├── entity_manager.rs   # HA entity state tracking
│   │   ├── smarthome_intent.rs # Smart home command NLU
│   │   ├── secrets.rs          # OS keyring integration
│   │   ├── ollama_sidecar.rs   # Bundled LLM server process
│   │   ├── error.rs            # Custom error types
│   │   └── main.rs             # Entry point
│   ├── Cargo.toml              # Rust dependencies (35+ crates)
│   └── tauri.conf.json         # Tauri configuration
└── Documentation/               # Architecture & requirements docs (23 files)
    ├── SPOTIFY_ARCHITECTURE.md
    ├── HOMEASSISTANT_ARCHITECTURE.md
    ├── VOICE_BIOMETRICS_ARCHITECTURE.md
    ├── RAG_ARCHITECTURE.md
    └── ...
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

**Framework & Core:**
- **Framework**: Tauri v2 (Rust + WebView)
- **Frontend**: React 19 + TypeScript + Vite + Tailwind CSS
- **UI Components**: Radix UI primitives
- **State Management**: Zustand
- **Backend**: Rust with Tokio async runtime
- **Database**: SQLite (rusqlite) - 4 tables
- **HTTP Client**: reqwest

**Voice & AI:**
- **Speech-to-Text**: whisper-rs (Whisper.cpp bindings)
- **Text-to-Speech**: Piper (subprocess) + rodio (playback)
- **Audio I/O**: cpal (cross-platform audio)
- **Speaker Recognition**: sherpa-rs (WeSpeaker ECAPA-TDNN)

**Integrations:**
- **Spotify**: oauth2 (PKCE flow) + regex (music NLU)
- **Home Assistant**: tokio-tungstenite (WebSocket) + async-trait
- **Web Search**: searxng crate (SearXNG client)
- **Secure Storage**: keyring (OS native keychain)

**Additional Libraries:**
- **Utilities**: chrono, serde, serde_json, thiserror, log, env_logger
- **Crypto**: sha2, base64 (OAuth2 PKCE)
- **Numerical**: ndarray (voice embeddings)

## Privacy & Security

### Core Privacy Guarantees
- **Local-First**: All voice processing (STT, TTS, wake word detection, speaker recognition) happens on your device
- **No Telemetry**: Zero analytics, tracking, or data collection
- **User Control**: You choose which services to connect to (all integrations are opt-in)
- **Secure Credentials**: API keys and OAuth tokens stored in OS native keychain (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux)
- **Data Ownership**: All data stored locally in SQLite database at `~/.local/share/nivora-aura/`

### Integration Privacy
- **Spotify**: OAuth2 tokens stored securely, no client secret required (PKCE flow)
- **Home Assistant**: WebSocket connections stay on local network, access tokens in keyring
- **Web Search RAG**: Opt-in only (disabled by default), user-selectable search providers
- **Voice Biometrics**: Voice embeddings stored locally, never transmitted to cloud
- **LLM Queries**: Only sent to your configured server (recommend local Ollama for privacy)

### What Data is Stored
- Conversation history (messages, timestamps)
- User voice embeddings (for speaker recognition)
- Settings and preferences
- Integration connection states

### What Data is NEVER Stored
- Raw audio recordings (processed and discarded immediately)
- Passwords or API keys (keyring only)
- Telemetry or usage analytics
- Personal information beyond what you explicitly provide

## License

This project is licensed under the **GNU General Public License v3.0** (GPL-3.0).

You are free to use, modify, and distribute this software under the terms of the GPL-3.0 license. See the [LICENSE](LICENSE) file for the full license text.

**In summary:**
- ✅ You can use this software for any purpose
- ✅ You can modify the source code
- ✅ You can distribute copies
- ✅ You must disclose the source code when distributing
- ✅ Any modifications must also be GPL-3.0 licensed
- ⚠️ The software is provided without warranty

## Support

For issues, questions, and feature requests, please visit:
- **GitHub Issues**: https://github.com/nivora/aura-desktop/issues
- **Documentation**: https://github.com/nivora/aura-desktop/wiki

## Acknowledgments

- **Whisper.cpp**: Speech recognition by OpenAI (ggerganov's C++ implementation)
- **Piper TTS**: Neural text-to-speech by Rhasspy
- **Ollama**: Local LLM inference server
- **Tauri**: Cross-platform desktop framework
