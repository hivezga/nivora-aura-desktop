#!/bin/bash

echo "🔄 Resetting Aura Desktop Development Environment..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Stop any running instances
echo -e "${YELLOW}1. Stopping any running Aura instances...${NC}"
pkill -f "aura-desktop" 2>/dev/null && echo "   ✓ Stopped running instance" || echo "   ℹ No running instance found"
sleep 1

# Backup database (optional)
DB_PATH="$HOME/.local/share/com.nivora.aura-desktop/aura_storage.db"
if [ -f "$DB_PATH" ]; then
    BACKUP_PATH="$HOME/.local/share/com.nivora.aura-desktop/aura_storage.db.backup.$(date +%Y%m%d_%H%M%S)"
    echo -e "${YELLOW}2. Backing up database...${NC}"
    cp "$DB_PATH" "$BACKUP_PATH" && echo "   ✓ Backup saved to: $BACKUP_PATH" || echo "   ✗ Backup failed"
else
    echo -e "${YELLOW}2. No database found to backup${NC}"
fi

# Delete SQLite database
echo -e "${YELLOW}3. Deleting SQLite database...${NC}"
if [ -f "$DB_PATH" ]; then
    rm -f "$DB_PATH" && echo "   ✓ Database deleted" || echo "   ✗ Failed to delete database"
else
    echo "   ℹ Database already deleted"
fi

# Clear localStorage
echo -e "${YELLOW}4. Clearing localStorage...${NC}"
LOCALSTORAGE_PATH="$HOME/.local/share/com.nivora.aura-desktop/localstorage"
if [ -d "$LOCALSTORAGE_PATH" ]; then
    rm -rf "$LOCALSTORAGE_PATH" && echo "   ✓ localStorage cleared" || echo "   ✗ Failed to clear localStorage"
else
    echo "   ℹ localStorage already cleared"
fi

# Clear voice biometrics
echo -e "${YELLOW}5. Clearing voice biometrics data...${NC}"
VOICES_PATH="$HOME/.local/share/nivora-aura/voices"
if [ -d "$VOICES_PATH" ]; then
    rm -rf "$VOICES_PATH"/* && echo "   ✓ Voice biometrics cleared" || echo "   ✗ Failed to clear voice data"
else
    echo "   ℹ No voice data found"
fi

# Clear temp recordings
echo -e "${YELLOW}6. Clearing temporary recordings...${NC}"
TEMP_WAV="$HOME/.local/share/com.nivora.aura-desktop/temp_recording.wav"
if [ -f "$TEMP_WAV" ]; then
    rm -f "$TEMP_WAV" && echo "   ✓ Temp recordings cleared" || echo "   ✗ Failed to clear temp files"
else
    echo "   ℹ No temp recordings found"
fi

# Note about keyring (can't be easily automated)
echo -e "${YELLOW}7. OS Keyring credentials...${NC}"
echo "   ℹ Manual step required: Clear these entries from your system keyring:"
echo "     - nivora-aura-llm-api-key"
echo "     - nivora-aura-spotify-access-token"
echo "     - nivora-aura-spotify-refresh-token"
echo "     - nivora-aura-ha-access-token"
echo "   On Linux: Use 'seahorse' (GNOME Keyring) or 'kwalletmanager' (KDE)"
echo "   On macOS: Use 'Keychain Access' app"
echo "   On Windows: Use 'Credential Manager'"

# Models info (keep by default)
echo -e "${YELLOW}8. ML Models...${NC}"
MODELS_PATH="$HOME/.local/share/nivora-aura/models"
if [ -d "$MODELS_PATH" ]; then
    MODEL_SIZE=$(du -sh "$MODELS_PATH" | cut -f1)
    echo "   ℹ Keeping ML models ($MODEL_SIZE) - these are large downloads"
    echo "   To delete models: rm -rf $MODELS_PATH"
else
    echo "   ℹ No models directory found"
fi

echo ""
echo -e "${GREEN}✅ Development environment reset complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Clear OS keyring entries manually (see step 7 above)"
echo "  2. Run: pnpm tauri dev"
echo "  3. Experience the fresh user onboarding flow"
echo ""
