# Nivora Aura - Project Status Report

**Last Updated:** October 15, 2025
**Project Phase:** Post-MVP Enhancement
**Version:** v1.5 (Advanced Integrations)

---

## Executive Summary

Nivora Aura has evolved from a basic MVP voice assistant into a **full-featured multi-user AI assistant platform** with advanced integrations for music control, smart home automation, web search, and speaker recognition. The project maintains its core privacy-first principles while adding powerful opt-in features.

### Key Metrics

| Metric | MVP (Documented) | Current (Actual) | Growth |
|--------|------------------|------------------|--------|
| **Backend Code** | ~2,000 LOC | **10,132 LOC** | **+406%** |
| **Frontend Code** | ~1,500 LOC | **3,144 LOC** | **+110%** |
| **Rust Modules** | 7 modules | **17 modules** | **+143%** |
| **Tauri Commands** | ~10 commands | **45 commands** | **+350%** |
| **Database Tables** | 3 tables | **4 tables** | +33% |
| **Integrations** | 0 | **4 major** | +∞ |
| **Documentation** | 3 files | **23 files** | **+667%** |

---

## Feature Matrix

### ✅ Production-Ready Features

| Feature Category | Feature | Status | Commands | Documentation |
|------------------|---------|--------|----------|---------------|
| **Core Voice** | Speech-to-Text (Whisper) | ✅ Complete | `listen_and_transcribe` | README.md |
| | Text-to-Speech (Piper) | ✅ Complete | `speak_text` | README.md |
| | Voice Activity Detection | ✅ Complete | `set_voice_state` | README.md |
| | Wake Word Mode | ✅ Complete | - | README.md |
| **Chat & LLM** | Multi-conversation UI | ✅ Complete | `load_conversations` | README.md |
| | LLM Integration | ✅ Complete | `handle_user_prompt` | README.md |
| | Ollama/OpenAI Support | ✅ Complete | - | README.md |
| | Conversation History | ✅ Complete | `save_message` | README.md |
| **Settings** | Configuration UI | ✅ Complete | `load_settings`, `save_settings` | README.md |
| | API Key Storage | ✅ Complete | `save_api_key`, `load_api_key` | README.md |
| | Model Management | ✅ Complete | `fetch_available_models` | README.md |
| **Spotify** | OAuth2 Authentication | ✅ Complete | `spotify_start_auth` | SPOTIFY_ARCHITECTURE.md |
| | Music Search & Play | ✅ Complete | `spotify_handle_music_command` | SPOTIFY_USER_GUIDE.md |
| | Playback Control | ✅ Complete | `spotify_control_playback` | SPOTIFY_TESTING_GUIDE.md |
| | Device Selection | ✅ Complete | `spotify_get_devices` | - |
| | Auto Token Refresh | ✅ Complete | - | - |
| **Home Assistant** | WebSocket Connection | ✅ Complete | `ha_connect` | HOMEASSISTANT_ARCHITECTURE.md |
| | Entity Synchronization | ✅ Complete | `ha_get_entities` | - |
| | Service Calls | ✅ Complete | `ha_call_service` | - |
| | Smart Home NLU | ✅ Complete | `ha_handle_smart_home_command` | - |
| | Devices View UI | ✅ Complete | - | - |
| | Guided Onboarding | ✅ Complete | `ha_dismiss_onboarding` | - |
| **Web Search RAG** | SearXNG Support | ✅ Complete | Integrated in `handle_user_prompt` | RAG_ARCHITECTURE.md |
| | Brave Search Support | ✅ Complete | Integrated in `handle_user_prompt` | ONLINE_MODE_GUIDE.md |
| | Context Augmentation | ✅ Complete | - | - |
| | Offline Fallback | ✅ Complete | - | RAG_ACCEPTANCE_CRITERIA.md |
| **Voice Biometrics** | Speaker Enrollment | ✅ Complete | `voice_biometrics_enroll_user` | VOICE_BIOMETRICS_ARCHITECTURE.md |
| | Speaker Recognition | ✅ Complete | Integrated in `listen_and_transcribe` | VOICE_BIOMETRICS_POC_RESULTS.md |
| | User Management | ✅ Complete | `voice_biometrics_list_users` | - |
| | Voice Print Storage | ✅ Complete | - | - |
| **Infrastructure** | SQLite Database | ✅ Complete | - | - |
| | OS Keyring Security | ✅ Complete | - | - |
| | CI/CD Pipeline | ✅ Complete | - | CI_CD_GUIDE.md |
| | Multi-platform Builds | ✅ Complete | - | WINDOWS_BUILD_GUIDE.md |

---

## 🔄 In-Progress Features

| Feature | Status | Progress | Target | Notes |
|---------|--------|----------|--------|-------|
| **Multi-User Spotify** | 🔄 Design Complete | 80% | v1.6 | Architecture designed, UI implementation pending |
| **Voice Biometrics UI** | 🔄 Backend Complete | 70% | v1.6 | Enrollment flow needs frontend integration |
| **GPU Acceleration** | 🔄 Design Complete | 50% | v1.7 | Architecture documented, implementation pending |
| **Context-Aware Commands** | 🔄 Research | 30% | v1.7 | Requires multi-user UI completion |

---

## 📋 Planned Features (Roadmap)

### v1.6 (Multi-User Completion) - Target: Q4 2025
- [ ] Multi-user Spotify UI (per-user OAuth tokens)
- [ ] Voice biometrics enrollment UI component
- [ ] User profile management dashboard
- [ ] Context-aware music commands ("play *my* playlist")
- [ ] User-specific Home Assistant scenes

### v1.7 (Performance & Intelligence) - Target: Q1 2026
- [ ] GPU acceleration implementation (CUDA/Metal)
- [ ] Advanced NLU engine (replace regex with ML-based)
- [ ] Custom wake word training ("Hey Aura")
- [ ] Embedded LLM option (llama.cpp integration)
- [ ] Smart command shortcuts (timers, alarms, calculations)

### v1.8 (Ecosystem Expansion) - Target: Q2 2026
- [ ] Plugin/extension system for third-party integrations
- [ ] Calendar integration (Google Calendar, Outlook)
- [ ] Email integration (read/send via voice)
- [ ] Mobile companion app (iOS/Android)
- [ ] Hardware device prototype

---

## Backend Architecture (17 Modules)

| Module | LOC | Purpose | Key Functions | Status |
|--------|-----|---------|---------------|--------|
| `lib.rs` | 2,543 | Main Tauri app logic, all command handlers | 45 Tauri commands | ✅ Stable |
| `database.rs` | 1,032 | SQLite persistence | 4 tables: conversations, messages, settings, user_profiles | ✅ Stable |
| `native_voice.rs` | 807 | Voice activity detection, STT pipeline | Audio capture, Whisper integration | ✅ Stable |
| `spotify_client.rs` | 779 | Spotify Web API client | Search, playback, playlists | ✅ Stable |
| `smarthome_intent.rs` | 680 | Natural language parsing for HA | Regex-based intent matching | ✅ Stable |
| `secrets.rs` | 558 | Secure credential storage | OS keyring integration | ✅ Stable |
| `voice_biometrics.rs` | 544 | Speaker recognition | WeSpeaker ECAPA-TDNN embeddings | ✅ Stable |
| `spotify_auth.rs` | 439 | OAuth2 flow with PKCE | Authorization, token refresh | ✅ Stable |
| `ha_client.rs` | 429 | Home Assistant client | WebSocket + REST API | ✅ Stable |
| `ollama_sidecar.rs` | 420 | Bundled LLM server | Process management | ✅ Experimental |
| `entity_manager.rs` | 419 | HA entity state tracking | Real-time sync | ✅ Stable |
| `web_search.rs` | 412 | Privacy-focused RAG | SearXNG/Brave backends | ✅ Stable |
| `music_intent.rs` | 375 | Music command parsing | Regex-based NLU | ✅ Stable |
| `llm.rs` | 309 | LLM engine | OpenAI-compatible API | ✅ Stable |
| `tts.rs` | 277 | Text-to-speech | Piper subprocess | ✅ Stable |
| `error.rs` | 103 | Custom error types | Error handling | ✅ Stable |
| `main.rs` | 6 | Entry point | App initialization | ✅ Stable |

**Total:** 10,132 LOC

---

## All 45 Tauri Commands

### Core Commands (13)
1. `greet` - Example command
2. `handle_user_prompt` - Main LLM query + RAG
3. `listen_and_transcribe` - Push-to-talk STT
4. `speak_text` - TTS output
5. `cancel_generation` - Stop LLM
6. `cancel_recording` - Stop voice recording
7. `load_conversations` - Retrieve all conversations
8. `load_messages` - Get messages for conversation
9. `create_new_conversation` - New chat
10. `save_message` - Persist message
11. `delete_conversation` - Remove conversation
12. `update_conversation_title` - Rename conversation
13. `generate_conversation_title` - Auto-title via LLM

### Settings Commands (8)
14. `load_settings` - Retrieve app settings
15. `save_settings` - Store configuration
16. `save_api_key` - Store LLM API key
17. `load_api_key` - Load from keyring
18. `update_vad_settings` - VAD parameters
19. `reload_voice_pipeline` - Reinitialize audio
20. `check_setup_status` - First-run wizard
21. `mark_setup_complete` - Finalize onboarding

### Voice Commands (3)
22. `set_voice_state` - Control wake word listening
23. `download_whisper_model` - Download STT model
24. `fetch_available_models` - List LLM models
25. `get_gpu_info` - Hardware acceleration status

### Spotify Commands (8)
26. `spotify_start_auth` - OAuth2 authorization
27. `spotify_disconnect` - Revoke tokens
28. `spotify_get_status` - Connection status
29. `spotify_save_client_id` - Store app credentials
30. `spotify_handle_music_command` - Process voice commands
31. `spotify_control_playback` - Play/pause/skip
32. `spotify_get_current_track` - Now playing info
33. `spotify_get_devices` - Available Spotify devices

### Home Assistant Commands (8)
34. `ha_connect` - WebSocket + REST setup
35. `ha_disconnect` - Shutdown connection
36. `ha_get_status` - Connection status
37. `ha_get_entities` - List smart home devices
38. `ha_get_entity` - Single entity state
39. `ha_call_service` - Execute automation
40. `ha_handle_smart_home_command` - Process voice commands
41. `ha_dismiss_onboarding` - Skip setup wizard

### Voice Biometrics Commands (6)
42. `voice_biometrics_status` - Model load status
43. `voice_biometrics_list_users` - Enrolled users
44. `voice_biometrics_enroll_user` - Add new speaker
45. `voice_biometrics_delete_user` - Remove enrollment
46. `voice_biometrics_test_enrollment` - Verify quality

---

## Database Schema

### Table 1: `conversations`
```sql
CREATE TABLE conversations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL
);
```

### Table 2: `messages`
```sql
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id INTEGER NOT NULL,
    role TEXT NOT NULL,  -- 'user' | 'assistant'
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);
CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
```

### Table 3: `settings`
```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT
);
```

**Stored Settings:**
- LLM provider, server URL, model name
- VAD sensitivity, timeout
- STT model path, TTS voice path
- Voice preference (enabled/disabled)
- Online mode (RAG enabled/disabled)
- RAG backend (SearXNG/Brave), search result count
- Spotify client ID, connection status
- Home Assistant server URL, connection status

### Table 4: `user_profiles` (Voice Biometrics)
```sql
CREATE TABLE user_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    voice_print_embedding BLOB NOT NULL,  -- 192-dim f32 array (768 bytes)
    enrollment_date TEXT NOT NULL,
    last_recognized TEXT,
    recognition_count INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_user_profiles_name ON user_profiles(name);
CREATE INDEX idx_user_profiles_active ON user_profiles(is_active);
```

**Future Columns (Multi-User Spotify):**
- `spotify_connected BOOLEAN DEFAULT FALSE`
- `spotify_client_id TEXT`
- `spotify_user_id TEXT`
- `spotify_display_name TEXT`
- `spotify_email TEXT`
- `auto_play_enabled BOOLEAN DEFAULT TRUE`

---

## Frontend Architecture (3,144 LOC)

### Main Components

| Component | Purpose | Lines | Key Features |
|-----------|---------|-------|--------------|
| `App.tsx` | Root component | ~300 | Wake word listener, app state sync |
| `ChatView.tsx` | Message display | ~250 | Markdown rendering, code highlighting |
| `Sidebar.tsx` | Conversation list | ~150 | New chat, conversation switching |
| `InputBar.tsx` | Voice/text input | ~200 | Push-to-talk, text chat toggle |
| `SettingsModal.tsx` | Settings UI | ~400 | Collapsible sections, live reload |
| `DevicesView.tsx` | HA devices UI | ~300 | Real-time entity cards, filtering |
| `SpotifySettings.tsx` | Spotify auth | ~200 | OAuth flow, connection UI |
| `HomeAssistantSettings.tsx` | HA connection | ~200 | Token input, WebSocket setup |
| `WelcomeWizard.tsx` | First-run setup | ~250 | Model download, integration setup |
| `ErrorBoundary.tsx` | Error fallback | ~100 | Error display |

### UI Component Library (Radix UI)
- `button`, `dialog`, `input`, `label`, `select`, `switch`, `tooltip`
- TailwindCSS styling
- Dark theme (gray-950/gray-900 base)

### State Management (Zustand)
```typescript
useChatStore() {
  conversations: Conversation[]
  currentConversationId: number | null
  messages: Message[]
  status: 'idle' | 'listening' | 'processing' | 'speaking'
  inputMethod: 'voice' | 'text'
  // ... 15+ state variables
}
```

---

## Integration Details

### 🎵 Spotify Integration

**Authentication:** OAuth2 PKCE flow (no client secret)
- Redirect URI: `http://localhost:8888/callback`
- Scopes: `user-read-playback-state`, `user-modify-playback-state`, `user-read-currently-playing`, `playlist-read-private`
- Token refresh: Automatic with 5-minute buffer

**Music Intent Parser (Regex-based NLU):**
- "Play [track] by [artist]" → `PlayTrack`
- "Play my [playlist] playlist" → `PlayPlaylist`
- "Pause" / "Resume" → `Pause` / `Resume`
- "Next" / "Previous" → `Next` / `Previous`
- "What's playing?" → `NowPlaying`

**API Endpoints Used:**
- `/v1/me/player/play` - Start playback
- `/v1/me/player/pause` - Pause playback
- `/v1/me/player/next` - Skip to next
- `/v1/me/player/previous` - Skip to previous
- `/v1/me/player/currently-playing` - Get current track
- `/v1/me/playlists` - List user playlists
- `/v1/search` - Search tracks

**Multi-User Architecture (Designed):**
- Per-user OAuth tokens in keyring (user-scoped keys)
- User context in `SpotifyClient` (`user_id` field)
- Migration path from global to per-user tokens
- Fallback for unknown speakers

---

### 🏠 Home Assistant Integration

**Communication:** WebSocket + REST API
- WebSocket: Real-time state updates, event subscriptions
- REST: Service calls, entity queries

**Entity Manager:**
- Thread-safe state tracking (`Arc<RwLock<HashMap>>`)
- Auto-sync on connection
- Support for all HA domains: `light`, `climate`, `lock`, `cover`, `media_player`, `switch`, etc.

**Smart Home Intent Parser (Regex-based NLU):**
- "Turn on [device] in [room]" → `TurnOn`
- "Turn off [device]" → `TurnOff`
- "Set [device] to [brightness]%" → `SetBrightness`
- "Set temperature to [X] degrees" → `SetTemperature`
- "Lock/unlock [lock]" → `Lock` / `Unlock`
- "What's the status of [device]?" → `GetStatus`

**WebSocket Events:**
- `auth_required` → Send access token
- `state_changed` → Update entity manager
- `pong` → Heartbeat response

**REST Endpoints Used:**
- `/api/states` - Get all entities
- `/api/states/<entity_id>` - Get entity state
- `/api/services/<domain>/<service>` - Call service

**Guided Onboarding:**
- Auto-triggered when < 5 devices detected
- 6 integration recommendations (Hue, Chromecast, Nest, TP-Link, Google Assistant, Ring)
- One-click deep links to HA setup pages

---

### 🌐 Web Search RAG

**Backends:**

1. **SearXNG** (Privacy-focused meta-search)
   - No API key required
   - User-selectable instances
   - Default: `https://searx.be`
   - 20+ search engines aggregated

2. **Brave Search** (Independent index)
   - Requires API key
   - Direct search results
   - Rate limit: 2,000 queries/month (free tier)

**Context Augmentation Flow:**
```
User Query → Web Search (if enabled) → Extract Snippets → Augment LLM Prompt → LLM Response
```

**Privacy Guarantees:**
- Opt-in only (disabled by default)
- User-selected search providers
- No telemetry or tracking
- Graceful fallback to offline mode

---

### 👤 Voice Biometrics

**Model:** WeSpeaker ECAPA-TDNN
- Architecture: Emphasized Channel Attention, Propagation and Aggregation in TDNN
- Input: 16kHz audio (variable length)
- Output: 192-dimensional embedding vector
- Training Data: VoxCeleb2 (6,112 speakers)
- Performance: EER 0.8% on VoxCeleb1-O test
- Model Size: ~7MB
- Inference Time: ~10ms per utterance (CPU)

**Enrollment Flow:**
1. User records 3-5 voice samples (each 3-5 seconds)
2. Extract 192-dim embeddings from each sample
3. Average embeddings to create robust voice print
4. Validate quality (variance check)
5. Store in SQLite `user_profiles` table

**Recognition Flow:**
1. Capture audio from microphone (after VAD)
2. Extract embedding (192-dim vector)
3. Compare with all stored voice prints (cosine similarity)
4. Threshold match (similarity > 0.70)
5. Return matched user profile (or None)

**Integration:** Runs during push-to-talk STT with 500ms timeout

---

## Dependencies

### Rust Crates (35+)

**Core:**
- `tauri = "2"` - Desktop framework
- `tokio = { version = "1.40", features = ["full"] }` - Async runtime
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `serde_json = "1.0"` - JSON handling

**Database:**
- `rusqlite = { version = "0.37", features = ["bundled"] }` - SQLite
- `dirs = "6.0"` - User directories
- `keyring = "3.6"` - OS keyring

**Voice/Audio:**
- `whisper-rs = "0.15"` - Speech-to-text
- `cpal = "0.16"` - Audio I/O
- `rodio = "0.21"` - Audio playback
- `sherpa-rs = { version = "0.6", features = ["download-binaries"] }` - Speaker recognition

**HTTP/API:**
- `reqwest = "0.12"` - HTTP client
- `oauth2 = "4.4"` - OAuth2 auth
- `tokio-tungstenite = "0.21"` - WebSocket

**Integrations:**
- `searxng = "0.1.0"` - SearXNG client
- `tungstenite = "0.21"` - WebSocket protocol
- `futures-util = "0.3"` - Async utilities
- `async-trait = "0.1"` - Async traits

**Music/Spotify:**
- `regex = "1.10"` - Pattern matching
- `sha2 = "0.10"` - PKCE hashing
- `base64 = "0.21"` - PKCE encoding
- `rand = "0.8"` - Random generation
- `urlencoding = "2.1"` - URL encoding
- `open = "5.0"` - Open system browser

**Utilities:**
- `chrono = "0.4"` - Date/time
- `log = "0.4"` + `env_logger = "0.11"` - Logging
- `thiserror = "2.0"` - Error handling
- `ndarray = "0.16"` - Numerical arrays

### NPM Packages (15+)

**Core:**
- `react@19.1.0` - UI framework
- `react-dom@19.1.0`
- `@tauri-apps/api@2` - Tauri IPC

**UI:**
- `@radix-ui/*` - Accessible component primitives (10+ packages)
- `tailwindcss@3.4.18` - CSS framework
- `lucide-react@0.545.0` - Icons

**State & Utils:**
- `zustand@5.0.8` - State management
- `react-markdown@10.1.0` - Markdown rendering
- `react-hot-toast@2.6.0` - Toast notifications
- `highlight.js@11.11.1` - Code highlighting
- `date-fns@4.1.0` - Date utilities

---

## Documentation (23 Files)

### Architecture & Design
1. `Technical Architecture Document_ Nivora Aura (Desktop MVP & Hardware Foundation).md` - System overview
2. `SPOTIFY_ARCHITECTURE.md` - Multi-user Spotify design (v2.0)
3. `HOMEASSISTANT_ARCHITECTURE.md` - HA integration + entity management
4. `VOICE_BIOMETRICS_ARCHITECTURE.md` - Speaker recognition design
5. `RAG_ARCHITECTURE.md` - Web search + context augmentation

### User Guides
6. `SPOTIFY_USER_GUIDE.md` - Spotify setup instructions
7. `SPOTIFY_TESTING_GUIDE.md` - Test scenarios
8. `ONLINE_MODE_GUIDE.md` - RAG/online mode configuration
9. `VOICE_BIOMETRICS_POC_RESULTS.md` - Speaker ID accuracy metrics
10. `VOICE_BIOMETRICS_FRONTEND_PLAN.md` - UI implementation roadmap

### Build & Deployment
11. `CI_CD_GUIDE.md` - GitHub Actions workflow
12. `CI_CD_IMPROVEMENTS.md` - Recent CI enhancements
13. `WINDOWS_BUILD_GUIDE.md` - Windows-specific build steps
14. `WINDOWS_BUILD_STATUS.md` - Known Windows issues
15. `WINDOWS_VM_VERIFICATION.md` - Win11 VM testing
16. `GPU_ACCELERATION.md` - CUDA/GPU support
17. `GPU_FEATURE_SUMMARY.md` - GPU capabilities
18. `GPU_UI_INTEGRATION.md` - GPU selection UI

### Requirements
19. `Product Requirements Document_ Nivora Aura (Desktop MVP).md` - Feature spec
20. `RAG_ACCEPTANCE_CRITERIA.md` - Testing requirements
21. `SPOTIFY_ACCEPTANCE_CRITERIA.md` - Integration testing
22. `Nivora Aura_ Reclaiming Your Digital Sanctuary.md` - Vision statement
23. `SPOTIFY_ARCHITECTURE_v1_backup.md` - Legacy architecture

---

## Recent Commits (Last 2 Weeks)

| Date | Commit | Feature |
|------|--------|---------|
| Oct 15 | `0075808` | Merge multi-user Spotify branch |
| Oct 12 | `d2a24a0` | Update spotify_client.rs (token refresh) |
| Oct 11 | `57e6db0` | Add Claude Code GitHub Workflow |
| Oct 10 | `3d908bd` | Design multi-user Spotify architecture (AC1) |
| Oct 9 | `1e2b91e` | Real-time audio pipeline integration |
| Oct 8 | `3252d2b` | Integrate speaker recognition |
| Sep 28 | `5f7f97a` | Complete Spotify Music Integration |
| Sep 27 | `83f8ca9` | Complete Home Assistant Integration |
| Sep 26 | `ed8234b` | Add privacy-focused web search (RAG) |

---

## Privacy & Security

### Privacy Guarantees

✅ **100% Offline Core Processing**
- Voice activity detection
- Speech-to-text (Whisper)
- Text-to-speech (Piper)
- Speaker recognition (sherpa-rs)
- All run entirely on-device

✅ **No Telemetry or Tracking**
- Zero analytics
- No usage data collection
- No cloud uploads without explicit consent

✅ **User-Controlled Integrations**
- Spotify: Opt-in, OAuth2 tokens in keyring
- Home Assistant: Opt-in, local network only
- Web Search RAG: Opt-in, disabled by default
- All integrations can be disconnected anytime

✅ **Secure Credential Storage**
- API keys in OS native keyring
- OAuth tokens in OS native keyring
- Never stored in plaintext or database

✅ **Local Data Storage**
- Conversation history: SQLite database
- Voice embeddings: SQLite database
- Settings: SQLite database
- All data at `~/.local/share/nivora-aura/`

### What Data is Stored

- Conversation messages (user + assistant)
- User voice embeddings (192-dim vectors)
- Settings and preferences
- Integration connection states

### What Data is NEVER Stored

- Raw audio recordings (processed and discarded)
- Passwords or API keys (keyring only)
- Telemetry or usage analytics
- Personal information beyond user input

---

## Known Limitations

### Current Limitations

1. **Wake Word Detection:** Energy-based VAD (any loud sound triggers), not keyword-specific ("Hey Aura")
2. **LLM Dependency:** Requires external server (Ollama/OpenAI), no embedded inference
3. **Manual Model Downloads:** Users must manually download Whisper/Piper models
4. **Single-User Spotify:** Multi-user architecture designed but not in UI yet
5. **Regex-based NLU:** Music/smart home intents use regex (not ML-based)

### Technical Debt

1. **Error Handling:** Some error paths could be more graceful
2. **Code Duplication:** Intent parsers (`music_intent.rs`, `smarthome_intent.rs`) share patterns
3. **Testing Coverage:** Unit tests exist but integration tests needed
4. **Documentation Gaps:** Some Tauri commands lack inline docs

---

## Future Roadmap

### v1.6 - Multi-User Completion (Q4 2025)
- Multi-user Spotify UI
- Voice biometrics enrollment UI
- User profile management
- Context-aware commands

### v1.7 - Performance & Intelligence (Q1 2026)
- GPU acceleration (CUDA/Metal)
- Advanced NLU (ML-based)
- Custom wake word ("Hey Aura")
- Embedded LLM (llama.cpp)

### v1.8 - Ecosystem Expansion (Q2 2026)
- Plugin system
- Calendar integration
- Email integration
- Mobile companion app
- Hardware device prototype

---

## Success Metrics

### MVP Goals (v1.0) - ✅ ACHIEVED

- ✅ Stable builds for Windows, macOS, Linux
- ✅ Core voice interaction (VAD → STT → LLM → TTS)
- ✅ 100% local voice processing
- ✅ Secure credential storage
- ✅ Full conversation management
- ✅ GPL-3.0 licensed with source transparency

### Post-MVP Goals (v1.5) - ✅ ACHIEVED

- ✅ Spotify music control integration
- ✅ Home Assistant smart home control
- ✅ Privacy-focused web search RAG
- ✅ Voice biometrics infrastructure
- ✅ CI/CD pipeline
- ✅ Comprehensive documentation (23 files)

### Upcoming Goals (v1.6+) - 🔄 IN PROGRESS

- 🔄 Multi-user support (Spotify, HA)
- 🔄 GPU acceleration
- 📋 Advanced NLU engine
- 📋 Mobile companion app

---

## Contributor Guidelines

### Code Quality Standards

- **Rust:** Follow Rust 2021 edition conventions, use `cargo clippy`
- **TypeScript:** Follow React 19 patterns, use ESLint
- **Documentation:** Update docs when adding features
- **Testing:** Add unit tests for new modules

### Privacy Principles

- **Local-First:** Default to on-device processing
- **Opt-In:** All cloud features require explicit user consent
- **Transparency:** Document all data collection
- **User Control:** Users must be able to disable/delete data

### Pull Request Process

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request with description
6. Wait for review and CI/CD checks

---

## Support & Community

- **GitHub Issues:** https://github.com/nivora/aura-desktop/issues
- **Documentation:** https://github.com/nivora/aura-desktop/wiki
- **License:** GPL-3.0

---

**Document Version:** 1.0
**Last Updated:** October 15, 2025
**Next Review:** November 1, 2025
