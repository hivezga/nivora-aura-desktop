# **Technical Architecture Document: Nivora Aura (Desktop MVP v1.0)**

Version: 2.0 (MVP v1.0 Release)
Date: October 8, 2025
Author: Alex Vance, Project Manager
Status: Released
Related PRD: Product Requirements Document v2.0

### **1. Overview**

This document outlines the implemented technical architecture for the **Nivora Aura Desktop MVP v1.0**. The architecture is designed to be robust, performant, and modular, adhering to a strict **"100% Offline Capable"** principle for all core voice processing functions (speech-to-text, text-to-speech, voice activity detection). This ensures privacy-first operation while maintaining flexibility for users who wish to leverage external LLM services.

We use the **Tauri framework**, building a lightweight, secure, cross-platform application using a **Rust backend** and a **React + TypeScript** frontend.

### **2. Technology Stack (As Implemented)**

| Component | Technology | Rationale |
| :---- | :---- | :---- |
| **Application Framework** | Tauri v2 | Secure, lightweight, cross-platform, Rust-native backend with WebView UI. |
| **Backend Language** | Rust 2021 | Performance, memory safety, and concurrency. Ideal for real-time audio and AI workloads. |
| **Frontend Framework** | React 19 + TypeScript | Modern, component-based UI development with strong typing for maintainability. |
| **Frontend Styling** | Tailwind CSS | Utility-first CSS framework for rapid, consistent UI development. |
| **State Management** | Zustand | Simple, unopinionated global state management for React. |
| **Voice Activity Detection** | Energy-based VAD (cpal) | Lightweight, real-time voice detection using audio energy levels. |
| **Speech-to-Text (STT)** | whisper-rs (Whisper.cpp bindings) | High-performance, on-device transcription using OpenAI Whisper models. |
| **Text-to-Speech (TTS)** | Piper (subprocess) + rodio | High-quality, fast, on-device neural voice synthesis via Piper CLI. |
| **Audio I/O** | cpal | Cross-platform audio capture and monitoring. |
| **Audio Playback** | rodio | Cross-platform audio playback for TTS output. |
| **LLM Communication** | reqwest (HTTP client) | HTTP-based communication with OpenAI-compatible LLM APIs. |
| **LLM Server (External)** | Ollama / OpenAI-compatible | Flexible support for local (Ollama) or cloud (OpenAI, Anthropic) LLMs. |
| **Data Persistence** | SQLite (rusqlite) | Robust, serverless, single-file database for conversation history and settings. |
| **Secure Storage** | keyring | OS-native credential storage (macOS Keychain, Windows Credential Manager, Linux Secret Service). |
| **Async Runtime** | Tokio | High-performance async runtime for concurrent voice pipeline and HTTP requests. |

### **3. High-Level Architecture Diagram**

```
┌─────────────────────────────────────────────────────────────┐
│                    Nivora Aura Desktop App                  │
│                     (Tauri + React + Rust)                  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Frontend (React + TypeScript)          │   │
│  │  • Chat UI (messages, conversation list)           │   │
│  │  • Settings modal (LLM, voice models, API keys)    │   │
│  │  • Voice status indicator                          │   │
│  │  • Zustand state management                        │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          │ Tauri IPC (invoke API)           │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Rust Backend (Core)                    │   │
│  │                                                     │   │
│  │  ┌──────────────────────────────────────────────┐  │   │
│  │  │          Tauri Command Handlers              │  │   │
│  │  │  • greet, send_message                       │  │   │
│  │  │  • start_voice_pipeline, stop_voice_pipeline │  │   │
│  │  │  • save/load settings, manage conversations  │  │   │
│  │  └──────────────────────────────────────────────┘  │   │
│  │                                                     │   │
│  │  ┌──────────────────┐  ┌───────────────────────┐  │   │
│  │  │ Native Voice     │  │ TTS Engine            │  │   │
│  │  │ Pipeline         │  │                       │  │   │
│  │  │                  │  │ • Piper TTS (CLI)     │  │   │
│  │  │ • whisper-rs     │  │ • subprocess spawn    │  │   │
│  │  │ • cpal (audio)   │  │ • WAV conversion      │  │   │
│  │  │ • Energy VAD     │  │ • rodio playback      │  │   │
│  │  │ • Wake detection │  │ • espeak-ng phonemes  │  │   │
│  │  └──────────────────┘  └───────────────────────┘  │   │
│  │                                                     │   │
│  │  ┌──────────────────┐  ┌───────────────────────┐  │   │
│  │  │ LLM Client       │  │ Database & Storage    │  │   │
│  │  │                  │  │                       │  │   │
│  │  │ • reqwest HTTP   │  │ • SQLite (rusqlite)   │  │   │
│  │  │ • OpenAI API fmt │  │ • Conversations table │  │   │
│  │  │ • JSON streaming │  │ • Messages table      │  │   │
│  │  │ • Async (Tokio)  │  │ • keyring secrets     │  │   │
│  │  └──────────────────┘  └───────────────────────┘  │   │
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
          │   • Anthropic API             │
          │   • Any OpenAI-compatible     │
          └───────────────────────────────┘
```

### **4. Component Breakdown**

#### **4.1 Frontend (React + TypeScript)**

**File:** `src/App.tsx`, `src/main.tsx`, `src/store.ts`

**Responsibilities:**
- Render chat interface with sidebar (conversation list) and main chat view
- Handle user input (text and voice button interactions)
- Communicate with Rust backend via Tauri's `invoke()` API
- Manage UI state using Zustand (current conversation, messages, voice status)
- Display settings modal for configuration

**Key Components:**
- Chat message list with user/assistant messages
- Text input with send button
- Microphone button for push-to-talk
- Conversation sidebar with create/delete/switch functionality
- Settings modal for LLM, STT, TTS, and API key configuration

**Communication Pattern:**
```typescript
// Example: Send message to LLM
await invoke('send_message', {
  conversationId: number,
  content: string
});

// Example: Start voice pipeline
await invoke('start_voice_pipeline', {
  wakeWordEnabled: boolean
});
```

#### **4.2 Rust Backend (Tauri Core)**

**File:** `src-tauri/src/lib.rs`

**Responsibilities:**
- Expose Tauri commands to frontend via `#[tauri::command]` macro
- Orchestrate voice pipeline, LLM requests, and database operations
- Manage application lifecycle and state

**Key Commands:**
- `greet(name: String) -> String` - Example/test command
- `send_message(conversation_id: i64, content: String) -> Result<Message>`
- `start_voice_pipeline(wake_word_enabled: bool) -> Result<()>`
- `stop_voice_pipeline() -> Result<()>`
- `create_conversation(title: String) -> Result<Conversation>`
- `delete_conversation(id: i64) -> Result<()>`
- `get_conversations() -> Result<Vec<Conversation>>`
- `save_settings(settings: Settings) -> Result<()>`
- `load_settings() -> Result<Settings>`

#### **4.3 Native Voice Pipeline**

**File:** `src-tauri/src/native_voice.rs`

**Responsibilities:**
- Capture audio from microphone using `cpal`
- Detect voice activity using energy-based threshold detection
- Transcribe audio to text using `whisper-rs`
- Support both wake word mode (continuous monitoring) and push-to-talk mode

**Implementation Details:**
- **Voice Activity Detection (VAD):** Calculates RMS energy of audio samples and compares against configurable threshold
- **Wake Word Mode:** Continuously monitors audio stream, activates listening when energy exceeds threshold
- **Push-to-Talk Mode:** Records audio on button press, stops on button release or silence
- **Whisper Integration:** Loads ggml model from `~/.local/share/nivora-aura/models/`, transcribes recorded audio chunks

**Key Functions:**
- `start_recording_with_wake_word()` - Begin continuous VAD monitoring
- `start_recording_push_to_talk()` - Begin manual recording session
- `stop_recording()` - Stop active recording and transcribe
- `transcribe_audio(samples: &[f32]) -> Result<String>` - Convert audio to text using Whisper

#### **4.4 Text-to-Speech Engine**

**File:** `src-tauri/src/tts.rs`

**Responsibilities:**
- Synthesize natural-sounding speech from text using Piper TTS
- Play generated audio through system speakers using `rodio`

**Implementation Details:**
- **Subprocess Approach:** Spawns `piper` CLI binary with voice model path and input text
- **Audio Format:** Piper outputs raw WAV data, converted to in-memory format using `hound`
- **Playback:** `rodio` handles cross-platform audio output to default audio device
- **Voice Models:** Users select ONNX voice models (e.g., `en_US-lessac-medium.onnx`) in settings

**Key Functions:**
- `speak(text: &str, voice_model: &str) -> Result<()>`
- Internally: `spawn_piper_process()` → `decode_wav()` → `play_audio()`

#### **4.5 LLM Client**

**File:** `src-tauri/src/llm.rs`

**Responsibilities:**
- Send user messages to configured LLM server (Ollama, OpenAI, etc.)
- Parse streaming or non-streaming responses
- Support multiple LLM providers via OpenAI-compatible API format

**Implementation Details:**
- **HTTP Client:** Uses `reqwest` with `tokio` async runtime
- **API Format:** OpenAI chat completions API (`/v1/chat/completions`)
- **Providers Supported:**
  - **Ollama:** `http://localhost:11434/v1` (local, private)
  - **OpenAI:** `https://api.openai.com/v1` (requires API key)
  - **Custom:** Any URL implementing OpenAI-compatible API

**Request Format:**
```json
{
  "model": "llama3.2:3b",
  "messages": [
    {"role": "user", "content": "What is the weather?"}
  ],
  "stream": false
}
```

**Key Functions:**
- `send_chat_message(server_url: &str, api_key: Option<&str>, model: &str, messages: Vec<Message>) -> Result<String>`

#### **4.6 Database & Persistence**

**File:** `src-tauri/src/database.rs`

**Responsibilities:**
- Store conversation history in local SQLite database
- Manage conversations and messages CRUD operations
- Persist user settings

**Schema:**
```sql
CREATE TABLE conversations (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE messages (
    id INTEGER PRIMARY KEY,
    conversation_id INTEGER NOT NULL,
    role TEXT NOT NULL,  -- 'user' or 'assistant'
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);
```

**Database Location:** `~/.local/share/nivora-aura/aura.db` (or platform-equivalent user data directory)

**Key Functions:**
- `create_conversation(title: &str) -> Result<Conversation>`
- `get_conversations() -> Result<Vec<Conversation>>`
- `delete_conversation(id: i64) -> Result<()>`
- `add_message(conversation_id: i64, role: &str, content: &str) -> Result<Message>`
- `get_messages(conversation_id: i64) -> Result<Vec<Message>>`

#### **4.7 Secrets Management**

**File:** `src-tauri/src/secrets.rs`

**Responsibilities:**
- Securely store and retrieve API keys using OS-native credential storage
- Support multiple API key types (OpenAI, Anthropic, custom)

**Implementation:**
- **Library:** `keyring` crate for cross-platform secret storage
- **Service Name:** `nivora-aura`
- **Key Format:** `{provider}_api_key` (e.g., `openai_api_key`)

**Storage Backends:**
- **macOS:** Keychain Access
- **Windows:** Credential Manager
- **Linux:** Secret Service (GNOME Keyring, KWallet)

**Key Functions:**
- `save_api_key(provider: &str, api_key: &str) -> Result<()>`
- `get_api_key(provider: &str) -> Result<Option<String>>`
- `delete_api_key(provider: &str) -> Result<()>`

### **5. Data Flow: Voice Interaction (Wake Word Mode)**

**Complete flow from voice input to spoken response:**

1. **User speaks** (above energy threshold)
2. **VAD Detection** (native_voice.rs):
   - `cpal` captures audio stream
   - Calculate RMS energy of audio samples
   - If energy > threshold: begin recording buffer
3. **Audio Recording**:
   - Buffer audio samples until silence detected (energy drops below threshold for N seconds)
   - Convert samples to WAV format
4. **Speech-to-Text** (native_voice.rs):
   - Load Whisper model via `whisper-rs`
   - Transcribe audio buffer: `transcribe_audio(samples)` → `"What is the weather?"`
   - Send transcription to frontend via Tauri event
5. **LLM Request** (llm.rs):
   - Frontend invokes `send_message(conversation_id, "What is the weather?")`
   - Backend constructs OpenAI API request with conversation history
   - Send HTTP POST to configured LLM server (e.g., Ollama)
   - Parse JSON response: `{"choices": [{"message": {"content": "..."}}]}`
6. **Database Persistence** (database.rs):
   - Save user message: `add_message(conversation_id, "user", "What is the weather?")`
   - Save assistant response: `add_message(conversation_id, "assistant", "...")`
7. **Text-to-Speech** (tts.rs):
   - Spawn Piper subprocess: `piper --model en_US-lessac-medium.onnx --output_raw`
   - Pipe response text to stdin, receive WAV audio on stdout
   - Decode WAV using `hound`, play via `rodio`
8. **Frontend Update**:
   - Display both user and assistant messages in chat UI
   - Update conversation list (last message timestamp)

### **6. Configuration & Settings Management**

**Settings Structure (JSON):**
```json
{
  "llm": {
    "provider": "ollama",  // "ollama" | "openai" | "custom"
    "server_url": "http://localhost:11434/v1",
    "model_name": "llama3.2:3b",
    "api_key_provider": null  // "openai" | "anthropic" | null
  },
  "voice": {
    "stt_model": "ggml-base.en.bin",
    "tts_voice": "en_US-lessac-medium.onnx",
    "wake_word_enabled": true,
    "vad_threshold": 0.02  // 0.01-0.1, lower = more sensitive
  }
}
```

**Storage:**
- Settings stored in SQLite database (future: may use separate JSON config file)
- API keys stored separately in OS keychain via `keyring`

**Live Reload:**
- When user saves settings in UI, backend immediately reloads voice pipeline with new models
- LLM configuration applied to next request (no restart required)

### **7. Cross-Platform Considerations**

**Audio Differences:**
- **Linux:** Requires PulseAudio or ALSA, `WEBKIT_DISABLE_COMPOSITING_MODE=1` for rendering
- **macOS:** Uses CoreAudio, requires microphone permission prompt
- **Windows:** Uses WASAPI, requires microphone permission in system settings

**File Paths:**
- Model directory: `~/.local/share/nivora-aura/models/` (Linux/macOS), `%APPDATA%/nivora-aura/models/` (Windows)
- Database: `~/.local/share/nivora-aura/aura.db`

**Piper TTS Binary:**
- Must be in system PATH or `/usr/local/bin/piper`
- Windows: typically `C:\Program Files\Piper\piper.exe`

### **8. Performance Considerations**

**Latency Targets:**
- **VAD to STT:** < 500ms (depends on Whisper model size)
- **STT to LLM request:** < 100ms
- **LLM response:** Variable (local Ollama: 1-5s, cloud APIs: 500ms-2s)
- **TTS synthesis:** < 1s (Piper is very fast)
- **Total user query → audio response:** Target 2-7 seconds

**Optimizations:**
- Use smaller Whisper models (`ggml-base.en.bin` vs `ggml-large-v3.bin`) for faster transcription
- Local Ollama with quantized models (e.g., `llama3.2:3b-q4`) reduces LLM latency
- Piper TTS is already optimized for real-time synthesis
- Audio processing uses efficient Rust async/await with Tokio

**Resource Usage:**
- **RAM:** ~300-500MB (app) + ~500MB-2GB (Whisper model) + ~2-8GB (Ollama)
- **CPU:** Moderate during voice processing, low when idle
- **GPU:** Optional for Whisper acceleration (not used in v1.0)

### **9. Security Architecture**

**Attack Surface Mitigation:**
- **No external network access** except user-configured LLM API endpoints
- **No telemetry or analytics** - zero data sent to Nivora servers
- **Tauri security:** Uses allowlist for IPC commands, no arbitrary shell execution
- **SQLite injection protection:** Parameterized queries only
- **API key encryption:** OS-native credential storage (not plaintext)

**Future Enhancements:**
- End-to-end encryption for exported conversation backups
- Certificate pinning for cloud LLM API connections
- Sandboxed execution for plugin system (future roadmap)

### **10. Testing Strategy**

**Unit Tests:**
- Database operations (CRUD for conversations/messages)
- Settings serialization/deserialization
- Audio utility functions (VAD energy calculation)

**Integration Tests:**
- Full voice pipeline (mock audio → Whisper → verify transcription)
- LLM client with mock HTTP server
- TTS subprocess spawning and audio playback

**Platform Testing:**
- Manual testing on Ubuntu 22.04, macOS 13+, Windows 11
- Build verification for `.deb`, `.AppImage`, `.dmg`, `.exe` installers

### **11. Future Architecture Enhancements**

**Deferred to Post-MVP:**

1. **Keyword Wake Word (openwakeword):**
   - Replace energy-based VAD with ML-based keyword detection
   - Detect specific phrase (e.g., "Hey Aura") instead of any loud sound

2. **Embedded LLM (llama.cpp):**
   - Add optional embedded inference via `llama-cpp-rs`
   - Fully offline operation without Ollama dependency

3. **gRPC Client Protocol:**
   - Support high-performance gRPC for future Aura server communication
   - Enable bidirectional streaming for real-time voice

4. **Local Command Framework:**
   - Built-in handlers for timers, alarms, calculations
   - Reduce reliance on LLM for simple tasks

5. **Plugin System:**
   - WebAssembly-based plugins for extensibility
   - Community-contributed integrations (Home Assistant, MQTT, etc.)

6. **Hardware Device:**
   - Port core engine to embedded Linux (Raspberry Pi, etc.)
   - Custom PCB with far-field microphone array

### **12. Deployment & Distribution**

**Build Outputs:**
- **Linux:** `.deb` (Debian/Ubuntu), `.AppImage` (universal), `.rpm` (future)
- **macOS:** `.dmg` installer, `.app` bundle
- **Windows:** `.exe` installer, `.msi` (MSI Installer)

**Build Process:**
```bash
# Clean build for release
pnpm tauri build

# Output location:
# src-tauri/target/release/bundle/
```

**Dependencies Checklist:**
- ✅ Piper TTS binary installed (`piper` in PATH)
- ✅ espeak-ng installed (Piper dependency)
- ✅ Whisper model downloaded to models directory
- ✅ Piper voice model downloaded (.onnx + .onnx.json)
- ✅ Ollama installed and running (or alternative LLM server configured)

**End of Document**
