# **Technical Architecture Document: Nivora Aura (Desktop MVP & Hardware Foundation)**

Version: 1.4 (Final)  
Date: October 2, 2025  
Author: Alex Vance, Project Manager  
Status: Finalized  
Related PRD: Nivora\_PRD\_v1.1.md

### **1\. Overview**

This document outlines the technical architecture for the **Nivora Aura** desktop MVP. The architecture is designed to be robust, performant, and modular, adhering to a strict **"100% Offline Capable"** principle for all core functions. This ensures the software can power a future standalone device that works out of the box, with no internet required. Optional, user-initiated online enhancements are supported.

We will use the **Tauri framework**, building a lightweight, secure, cross-platform application using a **Rust backend** and a web-based frontend.

### **2\. Technology Stack**

| Component | Technology | Rationale |
| :---- | :---- | :---- |
| **Application Framework** | Tauri | Secure, lightweight, cross-platform, Rust-native backend. |
| **Backend Language** | Rust | Performance, memory safety, and concurrency. Ideal for AI models. |
| **Client-Server Protocol** | **gRPC** | **High-performance, low-latency communication with support for bi-directional streaming. Ideal for real-time voice and data transfer.** |
| **Frontend Framework** | React with TypeScript | Modern, component-based UI development with strong typing. |
| **Frontend Styling** | Tailwind CSS | Utility-first CSS framework for rapid, consistent UI development. |
| **State Management** | Zustand (React) | Simple, unopinionated state management. |
| **Wake Word Engine** | openwakeword | High-performance, on-device wake word detection. |
| **Speech-to-Text (STT)** | whisper.cpp | High-performance, on-device transcription. |
| **Text-to-Speech (TTS)** | piper | High-quality, fast, on-device voice synthesis. |
| **Local LLM Engine** | llama.cpp / Ollama | Efficient engine for running quantized LLMs locally. |
| **Data Persistence** | SQLite | Robust, serverless, single-file database for history and settings. |

### **3\. High-Level Architecture Diagram**

\+-------------------------------------------------------------------------+  
|                    Nivora Aura Engine (100% Offline Core)               |  
|-------------------------------------------------------------------------|  
|       \+-----------------------------+      \+------------------------+   |  
|       |     Frontend (Tauri WebView)  |      |   Rust Backend (Core)  |   |  
|       |-----------------------------|      |------------------------|   |  
|       | \- React (TSX) Components    |      | \- Tauri Command Handlers|   |  
|       |   \- Sidebar (Chat List)     |      | \- SQLite DB Interface   |   |  
|       |   \- Chat View (Messages)    |\<----\>| \- AI Processing Orchestrator|   |  
|       |   \- Settings (API Keys)     |      | \- gRPC Client Module    |   |  
|       \+-----------------------------+      \+-----------+-------------+   |  
|                                            |   AI Core (Local)   |       |  
|                                            \+---------------------+       |  
|                                            | \- Wake Word Engine  |       |  
|                                            | \- whisper.cpp       |       |  
|                                            | \- piper             |       |  
|                                            | \- Local LLM         |       |  
|                                            \+---------------------+       |  
\+-------------------------------------------------------------------------+  
       ^                                               |  
       | (gRPC, via Settings)                          | (Optional Network I/O for \*Enhancements\*)  
       v                                               v  
\+-----------------------------+              \+-----------------------------+  
|    Remote "Power" Server    |              |   External APIs (User Keys) |  
|-----------------------------|              |-----------------------------|  
| \- Full AI Core instance     |              | \- OpenAI / Anthropic / etc. |  
| \- (gRPC Server)             |              | \- Weather / News / etc.     |  
\+-----------------------------+              \+-----------------------------+

### **4\. Component Breakdown & Settings Management**

* **Settings/Secrets Manager (Rust Backend):** A module responsible for securely storing user settings, including API keys and the address of the remote Aura-Server.  
* **gRPC Client Module (Rust Backend):** A new module, built using libraries like tonic, responsible for managing the connection and communication with the remote Aura-Server.  
* **Settings UI (React Frontend):** A section in the UI where users can input and manage their API keys and configure the remote server connection.

### **5\. Data Flow for a Voice Command (Wake Word Activated)**

*This flow remains the same as v1.3.*

### **6\. Client/Server and Third-Party API Logic**

* The application operates in **Local Mode** by default.  
* The **AI Processing Orchestrator** will be the central hub for deciding where to route a user's request based on their settings.  
* When **Remote Mode** is enabled, the Orchestrator will use the **gRPC Client Module** to establish a high-performance connection to the user's Aura-Server for processing. This replaces the previous HTTP-based logic.

**End of Document**