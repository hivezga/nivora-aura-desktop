#!/bin/bash

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Add all library paths needed by Aura:
# - $SCRIPT_DIR/piper: Piper TTS libraries (libpiper_phonemize.so, libespeak-ng.so, libonnxruntime.so)
# - /usr/local/lib: Standard local libraries
# - /usr/lib: System libraries
export LD_LIBRARY_PATH="$SCRIPT_DIR/piper:/usr/local/lib:/usr/lib:$LD_LIBRARY_PATH"

# Fix for UI rendering on Wayland/Hyprland compositors.
export WEBKIT_DISABLE_COMPOSITING_MODE=1

# Launch the application in development mode.
pnpm tauri dev
