# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nivora Aura is a privacy-focused, local-first voice and text-based AI assistant desktop application. The project is built with Tauri (Rust backend + React/TypeScript frontend) and adheres to a **"100% Offline Capable"** principle for all core functions.

**Core Principles:**
- Local First: All core processing happens on-device without requiring internet
- Absolute Transparency: FOSS stack, user-inspectable code
- User-Owned Data: Complete user control over data, no cloud uploads without explicit consent

## Development Commands

**Package Manager:** This project uses `pnpm` (not npm or yarn)

### Frontend Development
```bash
pnpm dev              # Start Vite dev server (port 1420)
pnpm build            # Build frontend (TypeScript compilation + Vite build)
pnpm preview          # Preview production build
```

### Tauri Development
```bash
pnpm tauri dev        # Run Tauri app in development mode (runs pnpm dev + Rust backend)
pnpm tauri build      # Build production app bundles for current platform
```

### Rust Backend
```bash
cd src-tauri
cargo build           # Build Rust backend
cargo test            # Run Rust tests
cargo clippy          # Lint Rust code
```

## Architecture Overview

### Tech Stack
- **Framework:** Tauri v2 (Rust + WebView)
- **Frontend:** React 19 + TypeScript + Vite + Tailwind CSS + Radix UI
- **Backend:** Rust with 45 Tauri commands, Tokio async runtime, 10,132 LOC across 17 modules
- **Database:** SQLite (rusqlite) - 4 tables (conversations, messages, settings, user_profiles)
- **HTTP Client:** reqwest for OpenAI-compatible LLM API calls
- **State Management:** Zustand
- **AI Stack:**
  - Speech-to-Text: whisper-rs (Whisper.cpp bindings)
  - Text-to-Speech: Piper TTS (subprocess-based)
  - Audio I/O: cpal (cross-platform audio)
  - Speaker Recognition: sherpa-rs (WeSpeaker ECAPA-TDNN, 192-dim embeddings)
  - LLM: Ollama or any OpenAI-compatible server
- **Integrations:**
  - Spotify: OAuth2 (PKCE flow), music playback control
  - Home Assistant: WebSocket + REST API, real-time entity sync
  - Web Search RAG: SearXNG/Brave Search for context augmentation
- **Security:** keyring (OS native credential storage)

### Project Structure
```
aura-desktop/
├── src/                         # React frontend source (3,144 LOC)
│   ├── App.tsx                 # Main app component
│   ├── main.tsx                # React entry point
│   ├── store.ts                # Zustand state management
│   └── components/             # UI components
│       ├── ChatView.tsx        # Message display
│       ├── Sidebar.tsx         # Conversation list
│       ├── InputBar.tsx        # Voice/text input
│       ├── SettingsModal.tsx   # Settings UI
│       ├── DevicesView.tsx     # Home Assistant devices UI
│       ├── SpotifySettings.tsx # Spotify connection UI
│       ├── HomeAssistantSettings.tsx # HA connection UI
│       ├── WelcomeWizard.tsx   # First-run setup
│       └── ui/                 # Radix UI components
├── src-tauri/                   # Rust backend (10,132 LOC)
│   ├── src/
│   │   ├── lib.rs              # 45 Tauri commands, app initialization (2,543 LOC)
│   │   ├── database.rs         # SQLite persistence (1,032 LOC)
│   │   ├── native_voice.rs     # Voice pipeline (STT, VAD, audio) (807 LOC)
│   │   ├── spotify_client.rs   # Spotify Web API client (779 LOC)
│   │   ├── smarthome_intent.rs # HA command NLU (680 LOC)
│   │   ├── secrets.rs          # OS keyring integration (558 LOC)
│   │   ├── voice_biometrics.rs # Speaker recognition (544 LOC)
│   │   ├── spotify_auth.rs     # OAuth2 PKCE flow (439 LOC)
│   │   ├── ha_client.rs        # Home Assistant WebSocket + REST (429 LOC)
│   │   ├── ollama_sidecar.rs   # Bundled LLM server (420 LOC)
│   │   ├── entity_manager.rs   # HA entity state tracking (419 LOC)
│   │   ├── web_search.rs       # RAG with SearXNG/Brave (412 LOC)
│   │   ├── music_intent.rs     # Music command NLU (375 LOC)
│   │   ├── llm.rs              # LLM API client (309 LOC)
│   │   ├── tts.rs              # Piper TTS subprocess (277 LOC)
│   │   ├── error.rs            # Custom error types (103 LOC)
│   │   └── main.rs             # Entry point (6 LOC)
│   ├── Cargo.toml              # Rust dependencies (35+ crates)
│   └── tauri.conf.json         # Tauri configuration
├── Documentation/               # Architecture & requirements docs (23 files)
│   ├── Product Requirements Document_ Nivora Aura (Desktop MVP).md
│   ├── Technical Architecture Document_ Nivora Aura (Desktop MVP & Hardware Foundation).md
│   ├── SPOTIFY_ARCHITECTURE.md (v2.0 - Multi-User)
│   ├── HOMEASSISTANT_ARCHITECTURE.md
│   ├── VOICE_BIOMETRICS_ARCHITECTURE.md
│   ├── RAG_ARCHITECTURE.md
│   ├── CI_CD_GUIDE.md
│   └── ... (16 more)
└── public/                     # Static assets
```

### Communication Pattern
- **Frontend → Backend:** Uses Tauri's `invoke()` API to call Rust commands
- **Commands:** Defined with `#[tauri::command]` macro in `src-tauri/src/lib.rs`
- **Registration:** Commands registered via `tauri::generate_handler![]` in lib.rs
- **Example:** The `greet` command demonstrates the pattern

### Rust Backend Details
- Library crate name: `aura_desktop_lib` (see Cargo.toml `[lib]` section)
- Crate types: `staticlib`, `cdylib`, `rlib` (for Tauri requirements)
- Main entry point calls `aura_desktop_lib::run()` to start Tauri app

### Configuration
- **Dev server:** Fixed port 1420 (strictPort: true)
- **HMR port:** 1421
- **Windows subsystem:** GUI mode (no console window in release builds on Windows)
- **App identifier:** com.nivora.aura-desktop

## Development Guidelines

### Frontend
- Use TypeScript for all React components
- State management will use Zustand (not yet implemented)
- Styling will use Tailwind CSS (not yet integrated)
- Follow React 19 patterns (no legacy hooks)

### Backend (Rust)
- Add new Tauri commands in `src-tauri/src/lib.rs`
- Register commands in `.invoke_handler(tauri::generate_handler![...])`
- Use `serde` for data serialization between frontend and backend
- Follow Rust 2021 edition conventions

### Implemented Features

#### Core Voice Assistant (MVP v1.0)
- ✅ Chat-style UI (sidebar + main conversation view)
- ✅ Voice activity detection (energy-based wake word simulation)
- ✅ Voice & text input modes (Push-to-Talk and text chat)
- ✅ LLM integration (Ollama/OpenAI-compatible servers via HTTP)
- ✅ SQLite-based conversation history
- ✅ Settings modal with live configuration reload
- ✅ Secure API key storage (OS native keyring)
- ✅ Speech-to-Text (Whisper.cpp via whisper-rs)
- ✅ Text-to-Speech (Piper via subprocess)
- ✅ Multi-conversation management
- ✅ Conversation export/persistence

#### Advanced Integrations (Post-MVP)
- ✅ **Spotify Music Control** (8 Tauri commands)
  - OAuth2 PKCE authentication flow
  - Search and play tracks, playlists, albums
  - Playback control (play/pause/next/previous)
  - Device selection and currently playing info
  - Natural language music intent parser (regex-based)
  - Automatic token refresh (5-min buffer)
  - Multi-user architecture designed (not yet in UI)

- ✅ **Home Assistant Integration** (8 Tauri commands)
  - WebSocket real-time connection with auto-reconnect
  - REST API for service calls
  - Entity manager with state synchronization
  - Support for all HA domains (lights, climate, locks, covers, media)
  - Natural language smart home intent parser
  - Devices view UI with real-time entity cards
  - Guided onboarding for new users

- ✅ **Web Search RAG** (Privacy-focused)
  - SearXNG meta-search support (no API key required)
  - Brave Search API support (requires API key)
  - Context augmentation for LLM prompts
  - Configurable result count (1-20)
  - Opt-in only (disabled by default)
  - Graceful offline fallback

- ✅ **Voice Biometrics** (6 Tauri commands)
  - Speaker recognition using WeSpeaker ECAPA-TDNN
  - User enrollment with 3-5 voice samples
  - Real-time speaker identification (<20ms latency)
  - 192-dimensional voice embeddings (0.8% EER)
  - SQLite storage of voice prints
  - Integration with push-to-talk STT pipeline

#### Build & Infrastructure
- ✅ CI/CD pipeline (GitHub Actions)
- ✅ Multi-platform builds (Windows, macOS, Linux)
- ✅ GPU acceleration architecture (designed)
- ✅ Comprehensive documentation (23 files)

Refer to `Documentation/` for detailed architecture specifications (SPOTIFY_ARCHITECTURE.md, HOMEASSISTANT_ARCHITECTURE.md, VOICE_BIOMETRICS_ARCHITECTURE.md, RAG_ARCHITECTURE.md), and the main `README.md` for setup and usage instructions.

### Testing
- The app should be fully functional offline for all voice processing (core principle)
- LLM requires external server (Ollama recommended for local operation)
- Test on Windows, macOS, and Linux targets
- Target low latency for voice interactions (Voice Detection → STT → LLM → TTS)
- Ensure proper handling of missing models and graceful degradation

## Important Notes

- **DO NOT** add telemetry, analytics, or any cloud-dependent features without explicit user opt-in
- **DO NOT** implement features that send user data to external servers by default
- All AI processing should prioritize local/on-device execution
- When adding new dependencies, prefer lightweight, privacy-respecting libraries
- The application must remain fully functional without internet connectivity for core features
