# **Product Requirements Document: Nivora Aura (Desktop MVP)**

Version: 1.1 (Final)  
Date: October 2, 2025  
Author: Alex Vance, Project Manager  
Status: Finalized

### **1\. Introduction & Vision**

Nivora Aura is a revolutionary voice and text-based AI assistant built on a foundation of absolute privacy and user control. Unlike existing commercial assistants that process user data in the cloud, Aura is designed to be **"local first"** and **"100% offline capable"** for all core functions. Our vision is to deliver a powerful, intuitive AI experience that does not require users to trade their privacy for convenience. This document outlines the requirements for the Minimum Viable Product (MVP) of the Nivora Aura desktop application.

### **2\. Core Principles (Non-Negotiable)**

* **Local First:** All core processing (wake word, speech-to-text, core commands, text-to-speech) happens on the user's device. The application must be fully functional without an internet connection.  
* **Absolute Transparency:** The entire software stack is Free and Open-Source Software (FOSS). Users can inspect, modify, and verify the code.  
* **User-Owned Data:** The user is in complete control of their data. No voice recordings or personal information are ever sent to a server without explicit, opt-in consent.

### **3\. Target Audience**

* **Privacy-Conscious Individuals:** Users who are disillusioned with the data collection practices of Big Tech and are seeking secure alternatives.  
* **Tech Enthusiasts & Developers:** Users who value open-source software, enjoy customization, and want to tinker with their technology.  
* **Smart Home Hobbyists:** Users (particularly of platforms like Home Assistant) who want a private voice interface to control their local smart home network.

### **4\. MVP Feature Set & User Stories**

The MVP will deliver a polished, chat-based experience that is immediately familiar to users of modern AI tools.

| Feature ID | Feature Name | User Story |
| :---- | :---- | :---- |
| **UI-101** | **Chat-Style Interface** | As a user, I want a familiar interface with a conversation list on the side and a main chat window, so I can easily navigate my interactions. |
| **UI-102** | **Branding Integration** | As a user, I want to see the clean and professional Nivora branding, so the application feels trustworthy and polished. |
| **IN-201** | **Wake Word Activation** | As a user, I want to say "Hey Aura" to activate the assistant without needing to click a button, so I can interact hands-free. |
| **IN-202** | **Voice & Text Input** | As a user, I want to be able to speak my commands or type them into an input box, so I can choose the most convenient method of interaction. |
| **FUNC-301** | **Local Command: Timers** | As a user, I want to be able to set a timer (e.g., "set a timer for 10 minutes") that works completely offline. |
| **FUNC-302** | **Local Command: LLM** | As a user, I want to ask general knowledge questions and get answers from a local LLM, so I can get information without an internet connection. |
| **DATA-401** | **Conversation History** | As a user, I want my conversations to be saved locally, so I can review my past interactions with Aura. |
| **CONF-501** | **Optional API Keys** | As a power user, I want the option to add my own third-party API key (e.g., for OpenAI), so I can enhance Aura with more powerful models at my own discretion. |
| **CONF-502** | **Optional Remote Server** | As a power user, I want the option to connect the Aura desktop app to my own powerful home server for processing, so I can leverage my own hardware. |

### **5\. First-Run User Experience**

* The application will be distributed as an **all-in-one installer** containing all necessary AI models.  
* The first launch will request microphone permission and then be immediately ready for use with the "Hey Aura" wake word.

### **6\. Success Metrics**

* Successful, stable builds for Windows, macOS, and Linux.  
* Core voice interaction (Wake Word \-\> STT \-\> LLM \-\> TTS) latency is consistently low on target hardware.  
* Positive community reception regarding privacy features and offline functionality.