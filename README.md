# Nivora Aura

**A privacy-focused, local-first voice and text AI assistant with comprehensive smart home integration.**

Nivora Aura is a fully open-source, multi-user voice assistant that runs 100% offline for all core processing. Built with Tauri (Rust + React), it features voice biometric user recognition, smart home control, music integration, web search capabilities, and connects to any Ollama-compatible LLM server.

## Features

### 🎤 **Core Voice Intelligence**
- **Wake Word Activation**: Hands-free activation with energy-based voice activity detection
- **Speech-to-Text**: Local transcription powered by Whisper.cpp (multiple model sizes)
- **High-Quality TTS**: Premium neural voice synthesis using Piper TTS
- **Voice Biometrics**: Multi-user speaker recognition and personalized experiences
- **Conversation Management**: Intelligent chat with full conversation history

### 🏠 **Smart Home Integration**
- **Home Assistant Control**: Real-time WebSocket integration with natural language commands
- **Device Management**: Control lights, thermostats, locks, covers, and more
- **Scene Activation**: Trigger complex automations with simple voice commands
- **Entity Discovery**: Automatic device discovery and organization by room/area

### 🎵 **Music & Entertainment**
- **Spotify Integration**: Full OAuth2 authentication with premium account support
- **Voice Music Control**: "Play my music", "next song", "pause" with natural language
- **Personal Playlists**: Access your personal Spotify library and playlists
- **Device Selection**: Multi-device Spotify Connect support

### 🌐 **Web Integration & Search**
- **Real-time Web Search**: Privacy-focused search via SearXNG instances
- **RAG (Retrieval-Augmented Generation)**: Enhanced responses with current web information
- **Multiple Search Backends**: SearXNG (default) and Brave Search API support
- **Privacy-First**: No tracking, user-controlled search preferences

### 🔒 **Privacy & Security**
- **100% Local Processing**: All voice recognition and biometrics stay on-device
- **Secure Credential Storage**: OS-native keyring integration (Keychain/Credential Manager)
- **Multi-User Privacy**: Individual user profiles with separate data isolation
- **No Cloud Dependencies**: Core functionality works completely offline
- **User-Controlled Data**: Full control over conversation history and personal data

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         Nivora Aura Desktop App                                │
│                      (Tauri + React + Rust)                                    │
│                                                                                 │
│  ┌───────────────────────────────────────────────────────────────────────┐   │
│  │                    Frontend (React + TypeScript)                      │   │
│  │  • Multi-conversation chat UI        • Voice biometrics enrollment    │   │
│  │  • Smart home device controls        • Music integration UI           │   │
│  │  │  • Settings & configuration       • User profile management        │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                    │                                          │
│                                    │ Tauri IPC                                │
│                                    ▼                                          │
│  ┌───────────────────────────────────────────────────────────────────────┐   │
│  │                        Backend (Rust)                                 │   │
│  │                                                                       │   │
│  │  ┌─────────────────┐  ┌──────────────┐  ┌─────────────────────────┐  │   │
│  │  │ Native Voice    │  │ TTS Engine   │  │ Voice Biometrics        │  │   │
│  │  │ Pipeline        │  │              │  │                         │  │   │
│  │  │                 │  │ • Piper TTS  │  │ • Speaker Recognition   │  │   │
│  │  │ • whisper-rs    │  │   (subprocess)│  │ • User Enrollment       │  │   │
│  │  │ • cpal audio    │  │ • espeak-ng  │  │ • Cosine Similarity     │  │   │
│  │  │ • Energy VAD    │  │ • rodio      │  │ • Secure Voice Storage  │  │   │
│  │  └─────────────────┘  └──────────────┘  └─────────────────────────┘  │   │
│  │                                                                       │   │
│  │  ┌─────────────────┐  ┌──────────────┐  ┌─────────────────────────┐  │   │
│  │  │ Smart Home      │  │ Music        │  │ Web Search              │  │   │
│  │  │ Integration     │  │ Integration  │  │                         │  │   │
│  │  │                 │  │              │  │ • SearXNG Client        │  │   │
│  │  │ • Home Assistant│  │ • Spotify    │  │ • Brave Search API      │  │   │
│  │  │   WebSocket     │  │   OAuth2     │  │ • RAG Processing        │  │   │
│  │  │ • Device Control│  │ • Playback   │  │ • Privacy-Focused       │  │   │
│  │  │ • Natural Lang. │  │   Control    │  │                         │  │   │
│  │  └─────────────────┘  └──────────────┘  └─────────────────────────┘  │   │
│  │                                                                       │   │
│  │  ┌─────────────────┐  ┌──────────────┐  ┌─────────────────────────┐  │   │
│  │  │ LLM Client      │  │ Database     │  │ Security & Storage      │  │   │
│  │  │                 │  │              │  │                         │  │   │
│  │  │ • reqwest HTTP  │  │ • SQLite     │  │ • OS Keyring            │  │   │
│  │  │ • OpenAI API    │  │ • Messages   │  │ • API Key Storage       │  │   │
│  │  │   compatible    │  │ • Settings   │  │ • Multi-user Profiles   │  │   │
│  │  │ • Streaming     │  │ • User Data  │  │ • Secure Voice Prints   │  │   │
│  │  └─────────────────┘  └──────────────┘  └─────────────────────────┘  │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
                                    │
                ┌─────────────────────┼─────────────────────────────────────┐
                │                     │                                     │
                ▼                     ▼                                     ▼
     ┌───────────────────┐ ┌─────────────────────┐              ┌─────────────────────┐
     │ External LLM      │ │ Home Assistant      │              │ Music & Web         │
     │ Server            │ │ Server              │              │ Services            │
     │                   │ │                     │              │                     │
     │ • Ollama         │ │ • WebSocket API     │              │ • Spotify Web API   │
     │ • LM Studio      │ │ • Device Control    │              │ • SearXNG Instances │
     │ • LocalAI        │ │ • Automation        │              │ • Brave Search API  │
     │ • OpenAI API     │ │ • Real-time Events  │              │                     │
     └───────────────────┘ └─────────────────────┘              └─────────────────────┘
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

On first launch, Aura will guide you through the setup wizard, or you can manually configure via Settings:

#### 🤖 **LLM Configuration**
1. **LLM Provider**: Select "Local Model" (Ollama) or "Third-Party API"
2. **API Base URL**: `http://localhost:11434/v1` (default Ollama endpoint)
3. **Model Name**: `llama3.2:3b` (or your chosen model)
4. **API Key**: Required only for third-party APIs (stored securely in OS keyring)

#### 🎤 **Voice Configuration**
1. **STT Model**: `ggml-base.en.bin` (or your chosen Whisper model)
2. **TTS Voice**: Choose between male/female voices (Piper models)
3. **Wake Word**: Enable/disable voice activation
4. **VAD Sensitivity**: Adjust microphone sensitivity (0.5% - 15%, default: 2.0%)
5. **Silence Timeout**: Configure end-of-speech detection (0.5s - 3.0s)

#### 🎵 **Spotify Integration** (Premium Account Required)
1. **Client ID**: Enter your Spotify App credentials from [Spotify Dashboard](https://developer.spotify.com/dashboard)
2. **Authentication**: Complete OAuth2 flow via system browser
3. **Auto-Play**: Enable automatic music playback from voice commands

#### 🏠 **Home Assistant Integration**
1. **Server URL**: Your Home Assistant instance (e.g., `http://homeassistant.local:8123`)
2. **Access Token**: Long-lived access token from HA profile
3. **Device Discovery**: Automatic entity and area discovery

#### 👥 **Voice Biometrics** (Multi-User Support)
1. **User Enrollment**: Create voice profiles via guided enrollment process
2. **Speaker Recognition**: Automatic user identification during conversations
3. **Privacy Controls**: All voice data stored locally, user-controlled deletion

#### 🌐 **Web Search & RAG**
1. **Online Mode**: Enable web search augmentation (disabled by default)
2. **Search Backend**: Choose SearXNG (privacy-focused) or Brave Search API
3. **SearXNG Instance**: Custom instance URL or use default public instances
4. **Search Results**: Control number of results used for context (1-20)

Click **Save** to apply changes. The voice pipeline and integrations will reload automatically.

## Usage

### 🎤 **Voice Interaction**

#### Wake Word Mode
- Say anything clearly to activate listening (energy-based detection)
- Status indicator turns blue when actively listening
- Speak your question, command, or request
- Aura responds with personalized voice and text based on recognized user

#### Push-to-Talk Mode  
- Click and hold the microphone button
- Speak while the button is active
- Release or wait for automatic silence detection
- Perfect for noisy environments or precise control

### 💬 **Text Interaction**
- Type messages directly in the chat input field
- Press Enter or click Send to submit
- Full conversation context maintained across sessions
- Supports markdown formatting in responses

### 👥 **Multi-User Experience**
- **Voice Enrollment**: Set up individual voice profiles through Settings → User Profiles
- **Automatic Recognition**: Aura identifies speakers and provides personalized responses
- **Individual Contexts**: Each user gets separate conversation history and preferences
- **Privacy Protection**: Voice prints stored locally, never transmitted

### 🏠 **Smart Home Control**

#### Natural Language Commands
- *"Turn on the living room lights"*
- *"Set the bedroom temperature to 72 degrees"* 
- *"Lock the front door"*
- *"Open the garage door"*
- *"Turn off all the lights"*
- *"Good morning"* (trigger morning scene)

#### Device Management
- **Entity Discovery**: Automatic detection of Home Assistant devices
- **Room-Based Control**: Commands organized by areas/rooms
- **Scene Activation**: Trigger complex automations with simple phrases
- **Real-time Status**: Get current device states and sensor readings

### 🎵 **Music Control**

#### Spotify Integration (Premium Required)
- *"Play my music"* - Start your personal music based on listening history
- *"Play some jazz"* - Genre-based playback
- *"Next song"* / *"Previous song"* - Track navigation  
- *"Pause"* / *"Resume"* - Playback control
- *"Play [playlist name]"* - Access your personal playlists
- *"What's playing?"* - Get current track information

#### Multi-Device Support
- **Spotify Connect**: Control playback on any connected device
- **Device Selection**: Switch between speakers, phones, computers
- **Volume Control**: Adjust playback volume via voice commands

### 🌐 **Web-Enhanced Responses**

#### RAG (Retrieval-Augmented Generation)
- **Current Information**: Get up-to-date facts and data
- **Enhanced Context**: Web search results augment LLM responses
- **Privacy-Focused**: Uses SearXNG or Brave Search (user controlled)
- **Source Attribution**: See sources used for web-enhanced answers

### 📱 **Conversation Management**

- **New Conversation**: Click "+" button in sidebar or use Ctrl+N
- **Switch Conversations**: Click any conversation in the history list
- **Conversation Titles**: Auto-generated or manually editable
- **Search History**: Find past conversations by content or title
- **Export/Import**: Full conversation backup and restore capabilities
- **Delete Conversations**: Remove individual chats with confirmation

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

### 🎤 **Voice & Audio Issues**

#### "Microphone not working" or "No audio input detected"
1. **Check Permissions**: Ensure microphone access is granted (system will prompt on first use)
2. **Verify Models**: Confirm models are downloaded to `~/.local/share/nivora-aura/models/`
3. **Model Filenames**: Check Settings for correct Whisper model names
4. **Audio Levels**: Adjust VAD sensitivity in Settings (try 1.0% - 5.0% range)
5. **System Audio**: Verify microphone works in other applications

#### "Voice recognition not working" or "Can't understand speech"
1. **Model Quality**: Try a larger Whisper model (base.en → small.en → medium.en)
2. **Audio Environment**: Ensure quiet environment during speech
3. **Speaking Pace**: Speak clearly at normal conversational pace
4. **Model Location**: Verify STT model path in Settings matches downloaded file

### 🏠 **Smart Home Integration Issues**

#### "Failed to connect to Home Assistant" 
1. **Network Access**: Verify HA instance is reachable from Aura device
2. **Access Token**: Generate new Long-Lived Access Token in HA profile
3. **URL Format**: Use full URL with port (e.g., `http://homeassistant.local:8123`)
4. **Firewall**: Check firewall settings aren't blocking WebSocket connections
5. **HA Version**: Ensure Home Assistant version supports WebSocket API (2023.1+)

#### "Devices not responding to voice commands"
1. **Entity Discovery**: Check that devices appear in Settings → Smart Home
2. **Entity Names**: Use exact entity names or friendly names from Home Assistant  
3. **Area Configuration**: Ensure devices are assigned to areas/rooms in HA
4. **Device State**: Verify devices are available and not disabled in Home Assistant

### 🎵 **Spotify Integration Issues**

#### "Spotify login failed" or "Authorization error"
1. **Premium Account**: Spotify Premium subscription required for API access
2. **Client ID**: Verify Client ID from [Spotify Dashboard](https://developer.spotify.com/dashboard) is correct
3. **Redirect URI**: Ensure `http://localhost:8080/callback` is added to Spotify app settings
4. **Browser**: Complete OAuth flow may require default system browser
5. **Network**: Check network connectivity during authentication process

#### "No music playing" or "Playback control not working"
1. **Active Device**: Ensure at least one Spotify Connect device is active
2. **Device Selection**: Try selecting different playback device in Spotify app
3. **Premium Status**: Verify Spotify Premium subscription is active
4. **API Limits**: Check if you've exceeded Spotify API rate limits (rare)

### 👥 **Voice Biometrics Issues**

#### "Voice enrollment failed" or "Cannot create voice profile"
1. **Audio Quality**: Ensure clear, quiet environment during enrollment
2. **Multiple Samples**: Complete all 3 voice samples for accurate enrollment
3. **Consistent Voice**: Use same tone/volume throughout enrollment process
4. **Storage Space**: Verify sufficient disk space for voice profile storage
5. **Database Access**: Check that database file is writable

#### "User not recognized" or "Wrong user identified"
1. **Re-enrollment**: Consider re-enrolling voice profile with better audio quality
2. **Audio Consistency**: Speak in similar manner as during enrollment
3. **Background Noise**: Minimize background noise during recognition
4. **Multiple Users**: Ensure different users have distinct voice characteristics
5. **Model Sensitivity**: Check voice recognition sensitivity settings

### 🌐 **Web Search & Network Issues**

#### "Failed to search web" or "No search results"  
1. **Internet Connection**: Verify internet connectivity for web search
2. **Search Backend**: Try switching between SearXNG and Brave Search in Settings
3. **Instance Availability**: Test SearXNG instance URL in browser 
4. **API Key**: For Brave Search, verify API key is valid and has quota
5. **Firewall/Proxy**: Check network restrictions aren't blocking search requests

### 🖥️ **Application Issues**

#### "Blank window" or "App won't start" (Linux)
```bash
# Use required environment variable for Linux
WEBKIT_DISABLE_COMPOSITING_MODE=1 pnpm tauri dev
```

#### "Failed to connect to Ollama server"
1. **Server Status**: Verify Ollama is running: `ollama list` should show models
2. **Server Address**: Check Settings URL matches running server (default: `http://localhost:11434/v1`)
3. **Manual Test**: Test connection: `curl http://localhost:11434/api/tags`
4. **Model Availability**: Ensure target model is pulled: `ollama pull llama3.2:3b`
5. **Port Conflicts**: Check no other service is using port 11434

#### "libpiper_phonemize.so.1: cannot open shared object file" (Linux)
```bash
# Configure system linker for Piper libraries
echo "/usr/local/lib" | sudo tee /etc/ld.so.conf.d/piper.conf
sudo ldconfig
```

#### "Database locked" or "SQLite errors"
1. **File Permissions**: Ensure write access to `~/.local/share/nivora-aura/`
2. **Disk Space**: Verify sufficient storage space for database operations
3. **Concurrent Access**: Only run one instance of Aura at a time
4. **Corrupted DB**: Backup and delete database file to regenerate (loses history)

### 📋 **Getting More Help**

1. **Console Logs**: Check browser developer tools (F12) for frontend errors
2. **Rust Logs**: Run with `RUST_LOG=debug pnpm tauri dev` for detailed backend logging  
3. **Model Downloads**: Verify model files are complete and not corrupted
4. **System Resources**: Ensure sufficient RAM for chosen models (2GB+ recommended)
5. **GitHub Issues**: Report bugs with logs at https://github.com/nivora/aura-desktop/issues

## Development

### Project Structure

```
aura-desktop/
├── src/                       # React frontend source
│   ├── App.tsx               # Main application component
│   ├── store.ts              # Zustand state management
│   ├── components/           # UI components
│   │   ├── ChatView.tsx      # Main conversation interface
│   │   ├── Sidebar.tsx       # Navigation and conversation history
│   │   ├── SettingsModal.tsx # Configuration interface
│   │   ├── UserProfilesSettings.tsx  # Voice biometrics management
│   │   ├── EnrollmentModal.tsx       # Voice enrollment wizard
│   │   ├── DevicesView.tsx   # Smart home device management
│   │   ├── SpotifySettings.tsx       # Music integration setup
│   │   ├── HomeAssistantSettings.tsx # Smart home setup
│   │   └── WelcomeWizard.tsx # First-run setup guide
│   └── hooks/                # Custom React hooks
│       └── useAudioRecording.ts # Browser audio recording
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── lib.rs           # Tauri commands and app initialization
│   │   ├── native_voice.rs  # Voice pipeline (STT, VAD, audio)
│   │   ├── voice_biometrics.rs  # Speaker recognition system
│   │   ├── tts.rs           # Piper TTS integration
│   │   ├── llm.rs           # OpenAI-compatible LLM client
│   │   ├── database.rs      # SQLite persistence layer
│   │   ├── secrets.rs       # OS keyring integration
│   │   ├── spotify_client.rs    # Spotify Web API integration
│   │   ├── spotify_auth.rs      # OAuth2 authentication flow
│   │   ├── ha_client.rs         # Home Assistant WebSocket client
│   │   ├── web_search.rs        # SearXNG and Brave Search clients
│   │   ├── entity_manager.rs    # Smart home entity management
│   │   └── intent_parser.rs     # Natural language intent parsing
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
└── Documentation/           # Architecture & requirements docs
    ├── PRD.md              # Product Requirements Document
    ├── TAD.md              # Technical Architecture Document
    └── VOICE_BIOMETRICS_*  # Voice biometrics specifications
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

- **Framework**: Tauri v2 (Rust + WebView2/WKWebView)
- **Frontend**: React 19 + TypeScript + Vite + Tailwind CSS
- **State Management**: Zustand (lightweight alternative to Redux)
- **Backend**: Rust with Tokio async runtime
- **Database**: SQLite (rusqlite) with full-text search capabilities
- **HTTP Client**: reqwest with async/streaming support
- **Audio I/O**: cpal (cross-platform audio library)
- **Speech-to-Text**: whisper-rs (Whisper.cpp Rust bindings)
- **Text-to-Speech**: Piper neural TTS (subprocess) + rodio (audio playback)
- **Voice Biometrics**: Custom speaker recognition with cosine similarity
- **Secure Storage**: keyring (OS native credential storage)
- **Smart Home**: Home Assistant WebSocket API integration
- **Music**: Spotify Web API with OAuth2 PKCE authentication
- **Web Search**: SearXNG REST API and Brave Search API clients

## Privacy & Security

### 🔒 **Local-First Architecture**
- **Voice Processing**: All STT, TTS, and voice biometrics processing happens on-device
- **Speaker Recognition**: Voice prints stored locally in encrypted SQLite database
- **No Cloud Dependencies**: Core functionality works completely offline
- **User Data Ownership**: Complete control over conversation history and personal information

### 🛡️ **Multi-User Privacy Protection**  
- **Isolated User Profiles**: Each user's data stored separately with no cross-contamination
- **Secure Voice Storage**: Voice biometric data encrypted and stored using OS-native security
- **Personalized Access**: Users only see their own conversations and preferences
- **Granular Permissions**: Individual control over data sharing and integration access

### 🔐 **Secure Credential Management**
- **OS-Native Storage**: API keys and tokens stored in system keychain/credential manager
- **Encrypted at Rest**: All sensitive data encrypted using OS security frameworks
- **No Plain Text Secrets**: Credentials never stored in configuration files or logs  
- **Secure Transmission**: All network communications use HTTPS/WSS encryption

### 🚫 **Zero Telemetry Policy**
- **No Analytics**: Zero data collection, tracking, or usage analytics
- **No Phone Home**: Application never contacts Nivora servers for any purpose
- **User-Chosen Connections**: You control which external services to integrate
- **Audit Trail**: All network connections clearly documented and user-initiated

### 🔍 **Transparency & Control**
- **Open Source**: Complete codebase available for inspection and audit
- **Local Database**: SQLite database stored at `~/.local/share/nivora-aura/`
- **Configuration Transparency**: All settings stored in plaintext JSON (except secrets)
- **Export Capabilities**: Full data export and backup functionality
- **Deletion Rights**: Complete profile and data deletion with secure cleanup

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
