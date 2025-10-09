# **Product Requirements Document: Nivora Aura (Desktop MVP)**

Version: 2.0 (MVP v1.0 Release)
Date: October 8, 2025
Author: Alex Vance, Project Manager
Status: Released

### **1. Introduction & Vision**

Nivora Aura is a revolutionary voice and text-based AI assistant built on a foundation of absolute privacy and user control. Unlike existing commercial assistants that process user data in the cloud, Aura is designed to be **"local first"** and **"100% offline capable"** for all core voice processing functions. Our vision is to deliver a powerful, intuitive AI experience that does not require users to trade their privacy for convenience. This document outlines the requirements and delivered features for the v1.0 Minimum Viable Product (MVP) of the Nivora Aura desktop application.

### **2. Core Principles (Non-Negotiable)**

* **Local First:** All core voice processing (voice activity detection, speech-to-text, text-to-speech) happens on the user's device. Voice processing is fully functional without an internet connection.
* **Absolute Transparency:** The entire software stack is Free and Open-Source Software (FOSS). Users can inspect, modify, and verify the code.
* **User-Owned Data:** The user is in complete control of their data. Voice recordings are processed locally and conversation history is stored in a local SQLite database that users fully control.

### **3. Target Audience**

* **Privacy-Conscious Individuals:** Users who are disillusioned with the data collection practices of Big Tech and are seeking secure alternatives.
* **Tech Enthusiasts & Developers:** Users who value open-source software, enjoy customization, and want to tinker with their technology.
* **AI Hobbyists:** Users who want to experiment with local LLMs (via Ollama or similar tools) and maintain full control over their AI interactions.
* **Smart Home Hobbyists:** Users (particularly of platforms like Home Assistant) who want a private voice interface to control their local smart home network (future expansion).

### **4. MVP v1.0 Feature Set & Implementation Status**

The MVP delivers a polished, chat-based experience that is immediately familiar to users of modern AI tools.

| Feature ID | Feature Name | User Story | Status |
| :---- | :---- | :---- | :---- |
| **UI-101** | **Chat-Style Interface** | As a user, I want a familiar interface with a conversation list on the side and a main chat window, so I can easily navigate my interactions. | ✅ **Implemented** |
| **UI-102** | **Branding Integration** | As a user, I want to see the clean and professional Nivora branding, so the application feels trustworthy and polished. | ✅ **Implemented** |
| **IN-201** | **Voice Activity Detection** | As a user, I want hands-free activation using voice activity detection, so I can interact naturally without clicking buttons. | ✅ **Implemented** (Energy-based VAD) |
| **IN-202** | **Voice & Text Input** | As a user, I want to be able to speak my commands (Push-to-Talk) or type them into an input box, so I can choose the most convenient method of interaction. | ✅ **Implemented** |
| **IN-203** | **Speech-to-Text** | As a user, I want my voice to be accurately transcribed locally using Whisper, so I can communicate with Aura naturally. | ✅ **Implemented** (whisper-rs) |
| **IN-204** | **Text-to-Speech** | As a user, I want Aura to respond with natural-sounding voice using high-quality TTS, so I can hear responses. | ✅ **Implemented** (Piper TTS) |
| **FUNC-301** | **LLM Integration** | As a user, I want to ask questions and get intelligent responses from a local or remote LLM server, so I can get helpful information and assistance. | ✅ **Implemented** (Ollama/OpenAI-compatible) |
| **DATA-401** | **Conversation History** | As a user, I want my conversations to be saved locally in SQLite, so I can review my past interactions with Aura. | ✅ **Implemented** |
| **DATA-402** | **Multi-Conversation Management** | As a user, I want to create, switch between, and delete multiple conversations, so I can organize different topics. | ✅ **Implemented** |
| **DATA-403** | **Conversation Export** | As a user, I want to export my conversations for backup and portability. | ✅ **Implemented** |
| **CONF-501** | **Flexible LLM Configuration** | As a power user, I want to configure different LLM providers (Ollama, OpenAI, custom), so I can use my preferred AI backend. | ✅ **Implemented** |
| **CONF-502** | **Secure API Key Storage** | As a user, I want my API keys stored securely in the OS native keychain, so my credentials are protected. | ✅ **Implemented** |
| **CONF-503** | **Voice Model Configuration** | As a user, I want to select which Whisper and Piper models to use, so I can balance quality vs. performance. | ✅ **Implemented** |
| **CONF-504** | **VAD Sensitivity Settings** | As a user, I want to adjust the voice detection sensitivity to match my environment and microphone. | ✅ **Implemented** |

### **5. First-Run User Experience**

**Current Implementation (v1.0):**
* The application is distributed as platform-specific installers (`.deb`, `.AppImage`, `.dmg`, `.exe`, `.msi`)
* Users must manually download AI models (Whisper for STT, Piper voices for TTS) to `~/.local/share/nivora-aura/models/`
* Users must install and configure an external LLM server (Ollama recommended for local operation, or any OpenAI-compatible API)
* First launch opens Settings modal to configure:
  - LLM provider and model
  - Whisper STT model path
  - Piper TTS voice model path
  - Voice detection settings
* After configuration, voice interaction is immediately available

**Future Enhancement:**
* All-in-one installer option bundling common models for out-of-box experience

### **6. Voice Interaction Flow**

**Wake Word Mode (VAD-based):**
1. User enables "Wake Word" mode in settings
2. Application continuously monitors audio for voice activity using energy-based detection
3. When voice activity is detected (user speaks above threshold), listening activates
4. User speaks their query
5. Audio is transcribed using local Whisper model
6. Query is sent to configured LLM server (Ollama or compatible API)
7. Response is displayed in chat and spoken via Piper TTS

**Push-to-Talk Mode:**
1. User clicks microphone button
2. User speaks while button is held/active
3. Recording stops on button release or silence detection
4. Rest of flow same as wake word mode

**Text Mode:**
1. User types message in chat input
2. Query is sent to LLM
3. Response displayed in chat and spoken via TTS

### **7. Technical Requirements**

**Supported Platforms:**
* Windows 10/11 (x64)
* macOS 11+ (Intel and Apple Silicon)
* Linux (x64) - Ubuntu 20.04+, Fedora, Arch, etc.

**System Requirements:**
* **Minimum:**
  - 4GB RAM
  - 2GB free disk space (for app + models)
  - Microphone for voice input
  - Speakers/headphones for audio output
  - Internet connection for LLM server (if using Ollama locally, this is just initial model download)

* **Recommended:**
  - 8GB+ RAM
  - Dedicated GPU for faster Whisper transcription (optional)
  - Local Ollama installation for best privacy

**Dependencies:**
* Piper TTS binary (`piper` command-line tool)
* espeak-ng (dependency for Piper)
* Whisper model file (ggml format)
* Piper voice model (.onnx + .onnx.json)

### **8. Success Metrics (v1.0 Achievement)**

* ✅ Successful stable builds for Windows, macOS, and Linux
* ✅ Core voice interaction (VAD → STT → LLM → TTS) with acceptable latency on consumer hardware
* ✅ 100% local voice processing (no cloud required for STT/TTS)
* ✅ Secure credential storage using OS native keychains
* ✅ Full conversation management and persistence
* ✅ Flexible LLM provider support (Ollama, OpenAI, custom servers)
* ✅ GPL-3.0 licensed with full source code transparency

### **9. Known Limitations & Future Roadmap**

**Current Limitations:**
* Wake word detection uses energy-based VAD (any loud sound triggers), not keyword-specific detection (e.g., "Hey Aura")
* LLM requires external server; no embedded LLM inference in v1.0
* Manual model downloads required; no bundled installer option
* No built-in local command execution (timers, alarms, etc.) - all queries go to LLM
* No plugin/extension system for third-party integrations

**Future Roadmap:**
* Keyword-specific wake word detection (e.g., "Hey Aura" using openwakeword)
* Embedded LLM option for fully offline operation
* All-in-one installer with bundled models
* Local command framework for timers, reminders, calculations
* Smart home integration (Home Assistant, MQTT)
* Plugin system for extensibility
* Mobile companion app
* Standalone hardware device

### **10. Privacy & Security Guarantees**

* ✅ All voice processing (STT, TTS, VAD) executes locally
* ✅ No telemetry, analytics, or tracking of any kind
* ✅ No data sent to external servers except user-configured LLM API calls
* ✅ API keys stored in OS-native secure storage (not in plaintext)
* ✅ Conversation database stored locally with full user control
* ✅ Full source code transparency (GPL-3.0)
* ✅ No embedded third-party trackers or advertisements

### **Appendix: Deferred Features**

The following features from earlier planning phases have been deferred to post-MVP releases:

* **Keyword Wake Word (openwakeword):** Deferred in favor of simpler VAD for v1.0
* **Embedded LLM (llama.cpp):** External server approach chosen for flexibility
* **gRPC Client Protocol:** HTTP/REST used for broader compatibility
* **Built-in Timer/Alarm Commands:** Focus on general LLM chat for MVP
* **Smart Home Integration:** Planned for v1.1+
* **Hardware Device:** Desktop MVP validates approach before hardware development

**End of Document**
