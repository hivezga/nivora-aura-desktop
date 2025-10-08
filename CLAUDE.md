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
- **Frontend:** React 19 + TypeScript + Vite + Tailwind CSS
- **Backend:** Rust with Tauri commands, Tokio async runtime
- **Database:** SQLite (rusqlite) for conversation persistence
- **HTTP Client:** reqwest for OpenAI-compatible LLM API calls
- **AI Stack:**
  - Speech-to-Text: whisper-rs (Whisper.cpp bindings)
  - Text-to-Speech: Piper TTS (subprocess-based)
  - Audio I/O: cpal (cross-platform audio)
  - LLM: Ollama or any OpenAI-compatible server

### Project Structure
```
aura-desktop/
├── src/                    # React frontend source
│   ├── App.tsx            # Main app component
│   └── main.tsx           # React entry point
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Core Tauri app logic (invoke handlers, plugins)
│   │   └── main.rs        # Entry point (calls lib.rs::run())
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── Documentation/         # Architecture & requirements docs (PRD, TAD)
└── public/               # Static assets
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
The current version includes:
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

Refer to `Documentation/` for detailed PRD and technical architecture specifications, and the main `README.md` for setup and usage instructions.

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
