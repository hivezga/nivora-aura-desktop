#!/bin/bash
# Nivora Aura Desktop Launcher Script
# This script sets necessary environment variables for proper operation on Linux

# Fix for UI rendering on Wayland/Hyprland compositors
export WEBKIT_DISABLE_COMPOSITING_MODE=1

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Execute the actual Aura Desktop binary
exec "$SCRIPT_DIR/aura-desktop" "$@"
