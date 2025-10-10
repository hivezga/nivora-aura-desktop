# Windows 11 VM Verification Checklist

## Purpose

This document provides a comprehensive checklist for verifying the Windows production build of Aura Desktop on a clean Windows 11 virtual machine.

## VM Setup Requirements

### Recommended VM Configuration
- **OS:** Windows 11 Pro (latest stable build)
- **RAM:** 8GB minimum (16GB recommended for GPU testing)
- **Storage:** 40GB minimum (50GB+ recommended)
- **CPU:** 4 cores minimum
- **Hypervisor:** VirtualBox, VMware Workstation, or Hyper-V

### Optional: GPU Passthrough (Advanced)
If testing GPU acceleration:
- Enable GPU passthrough in your hypervisor
- Ensure NVIDIA/AMD drivers are installed in the VM
- Allocate sufficient VRAM (4GB+)

## Pre-Installation Verification

### 1. Clean Environment Check

- [ ] Fresh Windows 11 installation (no Aura previously installed)
- [ ] No Ollama installed system-wide
- [ ] No Piper TTS installed system-wide
- [ ] Windows fully updated
- [ ] Windows Defender enabled (test installer signing)

### 2. Prerequisites

- [ ] Windows 11 with latest updates installed
- [ ] User account has Administrator privileges
- [ ] At least 10GB free disk space
- [ ] Internet connection available (for optional online features)

## Installation Phase

### 1. Installer Execution

- [ ] Double-click `aura-desktop_0.1.0_x64-setup.exe`
- [ ] **Installer launches without SmartScreen warnings** (if signed)
  - If unsigned: SmartScreen warning is expected, click "More info" → "Run anyway"
- [ ] **Installation wizard appears** with Aura branding
- [ ] **License agreement** displays correctly
- [ ] **Installation directory** selection works
  - Default: `C:\Program Files\aura-desktop`
- [ ] **Create desktop shortcut** option available
- [ ] **Start menu entry** option available

### 2. Installation Progress

- [ ] **Progress bar** displays correctly
- [ ] **Estimated time** shows reasonable values
- [ ] **No error dialogs** during installation
- [ ] **Installation completes successfully** (shows "Completed" message)

### 3. Post-Installation File Verification

Navigate to `C:\Program Files\aura-desktop` and verify:

#### Application Files
- [ ] `aura-desktop.exe` exists (~5-10MB)
- [ ] `resources` directory exists

#### Bundled Resources
Check `C:\Program Files\aura-desktop\resources`:

```
resources/
├── ollama/
│   ├── bin/
│   │   ├── ollama-windows-amd64.exe (~32MB)
│   │   ├── lib/
│   │   │   └── ollama/
│   │   │       ├── ggml-*.dll (multiple files)
│   │   │       ├── cuda_v12/ (NVIDIA support)
│   │   │       └── cuda_v13/ (NVIDIA support)
│   │   └── vc_redist.x64.exe
│   └── models/
│       ├── blobs/ (Gemma 2B model files)
│       └── manifests/
├── piper/
│   ├── bin/
│   │   └── piper-windows-x86_64.exe
│   ├── voices/
│   │   ├── en_US-amy-medium.onnx
│   │   └── en_US-lessac-medium.onnx
│   └── espeak-ng-data/ (phoneme data)
└── whisper/ (may be empty initially)
```

Verification:
- [ ] All Ollama files present
- [ ] Ollama binary is ~32MB
- [ ] Ollama `lib/` directory contains DLLs (~1.9GB total)
- [ ] Gemma model blobs exist (~1.7GB)
- [ ] Piper binary exists (~500KB)
- [ ] Voice models exist (~61MB each)
- [ ] eSpeak-NG data directory populated

## First Launch

### 1. Application Startup

- [ ] **Launch Aura** from Start Menu or desktop shortcut
- [ ] **Application window opens** within 5-10 seconds
- [ ] **No crash or immediate errors**
- [ ] **First-run wizard appears** (if applicable)

### 2. System Tray / Background Processes

Open Task Manager (Ctrl+Shift+Esc):
- [ ] **`aura-desktop.exe`** process is running
- [ ] **`ollama-windows-amd64.exe`** process starts automatically (may take 10-30 seconds)
- [ ] Check **CPU usage** during Ollama startup (should spike initially, then settle)
- [ ] Check **Memory usage** (~500MB-1GB for Aura + Ollama)

### 3. First-Run Wizard (if implemented)

- [ ] **Welcome screen** displays correctly
- [ ] **Whisper model download** option available
- [ ] **Download progress** displays correctly (if downloading model)
- [ ] **Download completes successfully** (~75MB for ggml-tiny.bin)
- [ ] **Wizard completion** transitions to main app

## Core Functionality Testing

### 1. Settings Configuration

Navigate to **Settings** (gear icon or menu):

#### LLM Configuration
- [ ] **API Base URL** field shows: `http://localhost:11434/v1`
- [ ] **Model dropdown** populates with: `gemma:2b`
- [ ] **Model selection** works
- [ ] **Save settings** persists configuration

#### Voice Settings
- [ ] **STT Model** selection available (tiny/base/small)
- [ ] **Voice preference** toggle (male/female) works
- [ ] **VAD sensitivity** slider functional
- [ ] **VAD timeout** slider functional

#### API Key (Optional)
- [ ] **API key field** available (for external LLM providers)
- [ ] **Save API key** stores securely (Windows Credential Manager)
- [ ] **Load API key** retrieves from secure storage

### 2. Voice Input (Push-to-Talk)

#### Push-to-Talk Activation
- [ ] **Click microphone button** or use hotkey
- [ ] **Microphone indicator** shows "Listening..." state
- [ ] **Speak clearly** into microphone: "Hello, what is the weather today?"
- [ ] **Stop recording** (release button or timeout)
- [ ] **Transcription appears** in chat within 2-5 seconds

#### Transcription Accuracy
- [ ] **Simple phrases** transcribe correctly (90%+ accuracy)
- [ ] **Complex sentences** transcribe reasonably well
- [ ] **Background noise handling** acceptable

### 3. LLM Response Generation

After sending voice or text input:

- [ ] **Loading indicator** appears ("Generating...")
- [ ] **Response begins streaming** within 3-10 seconds (first load may be slower)
- [ ] **Response quality** is coherent and relevant
- [ ] **Response completes** without errors
- [ ] **Response appears** in chat interface

#### Ollama Server Check
Open browser to `http://localhost:11434`:
- [ ] **Ollama landing page** displays (simple text page)
- [ ] Confirms Ollama server is running

### 4. Text-to-Speech (TTS)

After LLM response:

- [ ] **"Play" or auto-play** TTS button appears
- [ ] **Click play** (if not auto-play)
- [ ] **Audio begins within 1-2 seconds**
- [ ] **Voice quality** is clear and natural
- [ ] **Playback completes** without stuttering or cutoff

#### Voice Selection
- [ ] **Switch to male voice** in settings
- [ ] **Test TTS again** - male voice plays
- [ ] **Switch to female voice** in settings
- [ ] **Test TTS again** - female voice plays

### 5. Conversation Management

#### New Conversation
- [ ] **"New Conversation" button** works
- [ ] **Previous conversation** saves to history
- [ ] **New blank chat** appears

#### Conversation History
- [ ] **Sidebar shows conversation list**
- [ ] **Click previous conversation** loads messages
- [ ] **Conversation title** auto-generates or allows editing
- [ ] **Delete conversation** works (with confirmation)

#### Persistence
- [ ] **Close application**
- [ ] **Relaunch application**
- [ ] **Previous conversations** still visible
- [ ] **Messages persist** correctly

### 6. System Status Indicators

In the UI (usually bottom left or status bar):

- [ ] **STT indicator** shows:
  - ✅ Green = Model ready
  - ❌ Red = Model missing
- [ ] **LLM indicator** shows:
  - ✅ Green = Ollama connected
  - ❌ Red = Ollama not connected

## Performance Testing

### 1. Response Times (on moderate hardware)

Expected timings:
- [ ] **Push-to-Talk activation:** <500ms
- [ ] **Speech-to-Text (STT):** 2-5 seconds for 10 seconds of audio
- [ ] **LLM first token:** 3-10 seconds (first load)
- [ ] **LLM subsequent tokens:** 1-3 seconds
- [ ] **Text-to-Speech (TTS):** 1-2 seconds to start playback

### 2. Resource Usage

Check Task Manager during typical usage:

- [ ] **aura-desktop.exe CPU:** <5% idle, <20% active
- [ ] **aura-desktop.exe RAM:** ~200-500MB
- [ ] **ollama-windows-amd64.exe CPU:** <2% idle, 30-80% during inference
- [ ] **ollama-windows-amd64.exe RAM:** ~2-4GB (with Gemma 2B loaded)

### 3. GPU Acceleration (if GPU present)

If NVIDIA/AMD GPU available:

- [ ] Open **Task Manager** → **Performance** → **GPU**
- [ ] **Generate LLM response**
- [ ] **GPU usage spikes** during inference (indicates GPU acceleration working)
- [ ] Check **Dedicated GPU memory** usage increases

**No GPU?**
- [ ] **CPU inference still works** (slower but functional)
- [ ] **No errors** related to missing GPU

## Error Handling & Edge Cases

### 1. Network Conditions

- [ ] **Disconnect internet**
- [ ] **All core features still work** (voice, LLM, TTS)
- [ ] **Ollama serves from bundled model**
- [ ] **No "connection failed" errors** for local features

### 2. Missing Model Scenarios

#### Whisper Model Missing
- [ ] **Delete** `%LOCALAPPDATA%\nivora-aura\models\ggml-tiny.bin`
- [ ] **Restart app**
- [ ] **First-run wizard appears** OR **error message prompts download**
- [ ] **Download model** via wizard
- [ ] **Voice input works** after download

#### Corrupted Database
- [ ] **Navigate to** `%LOCALAPPDATA%\nivora-aura\aura.db`
- [ ] **Delete or rename** database file
- [ ] **Restart app**
- [ ] **Fresh database created**
- [ ] **No crash or data loss errors**

### 3. Ollama Server Failures

#### Manual Ollama Stop
- [ ] **Open Task Manager**
- [ ] **End ollama-windows-amd64.exe process**
- [ ] **LLM status indicator** turns red (❌)
- [ ] **Try to send message** → Error shown gracefully
- [ ] **Restart app** → Ollama restarts automatically

## Security & Privacy Checks

### 1. Data Storage Locations

Verify data is stored locally:
- [ ] **Conversations:** `%LOCALAPPDATA%\nivora-aura\aura.db`
- [ ] **Models:** `%LOCALAPPDATA%\nivora-aura\models\`
- [ ] **API Keys:** Windows Credential Manager (`Control Panel → Credential Manager`)

### 2. Network Activity

Use Windows Firewall or Wireshark:
- [ ] **Monitor outbound connections**
- [ ] **Confirm localhost-only traffic** for Ollama (127.0.0.1:11434)
- [ ] **No telemetry or analytics** to external servers
- [ ] **No unexpected data uploads**

### 3. Windows Defender / Antivirus

- [ ] **Windows Defender does not flag** installer as malware
- [ ] **Ollama process not quarantined**
- [ ] **No false positives** for bundled binaries

## Uninstallation

### 1. Uninstall via Windows Settings

- [ ] **Windows Settings → Apps → Installed Apps**
- [ ] **Find "aura-desktop"**
- [ ] **Click "Uninstall"**
- [ ] **Uninstaller wizard appears**
- [ ] **Confirm uninstallation**
- [ ] **Progress completes successfully**

### 2. Verify Clean Removal

- [ ] **`C:\Program Files\aura-desktop`** directory removed
- [ ] **Start Menu entry** removed
- [ ] **Desktop shortcut** removed (if created)
- [ ] **Registry entries cleaned up** (optional check via regedit)

### 3. User Data Retention (Expected)

After uninstall, check if user data persists:
- [ ] **`%LOCALAPPDATA%\nivora-aura`** still exists (expected behavior)
  - Contains conversation history, models, settings
  - Should persist for re-installation or manual deletion

Manual cleanup (if desired):
- [ ] **Delete** `%LOCALAPPDATA%\nivora-aura\` folder
- [ ] **Remove API keys** from Credential Manager (search "aura" or "nivora")

## Known Issues & Workarounds

### Issue: Ollama Slow First Response
**Symptom:** First LLM response takes 30+ seconds
**Expected:** Model is loading into memory
**Workaround:** Subsequent responses will be faster (model stays in memory for 5 minutes)

### Issue: No Audio Input Detected
**Symptom:** Push-to-Talk doesn't record audio
**Solution:**
1. Check **Windows Settings → Privacy → Microphone**
2. Ensure **Aura has microphone permission**
3. Restart application

### Issue: TTS Voice Sounds Robotic
**Symptom:** Piper TTS quality is lower than expected
**Expected:** Piper uses lightweight ONNX models (medium quality)
**Note:** This is normal for offline TTS

### Issue: GPU Not Detected
**Symptom:** Inference runs on CPU despite having NVIDIA/AMD GPU
**Solutions:**
1. Ensure **latest GPU drivers** installed
2. Check **Ollama logs** (if debugging enabled)
3. Verify **CUDA/HIP libraries** bundled correctly

## Sign-Off

Upon completing all checklist items:

- [ ] **All core features functional** (STT, LLM, TTS, conversations)
- [ ] **No critical bugs** identified
- [ ] **Performance acceptable** on target hardware
- [ ] **Documentation matches actual behavior**

**Tested by:** ___________________________
**Date:** ___________________________
**VM Configuration:** ___________________________
**Build Version:** `aura-desktop_0.1.0_x64-setup.exe`
**Notes:** ___________________________

---

## Appendix: Troubleshooting Commands

### Check Ollama API Manually
```powershell
# Test Ollama server
curl http://localhost:11434/api/tags

# Generate test response
curl http://localhost:11434/api/generate -d '{
  "model": "gemma:2b",
  "prompt": "Hello, how are you?",
  "stream": false
}'
```

### View Application Logs (if implemented)
```powershell
# Navigate to logs directory
cd %LOCALAPPDATA%\nivora-aura\logs
type aura.log
```

### Reset Application State
```powershell
# Stop all Aura processes
taskkill /F /IM aura-desktop.exe
taskkill /F /IM ollama-windows-amd64.exe

# Delete user data (CAUTION: Deletes all conversations)
rmdir /S %LOCALAPPDATA%\nivora-aura

# Restart application
```
