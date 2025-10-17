#!/bin/bash

echo "🔑 Clearing Aura keyring credentials..."
echo ""

# List of keys to clear
KEYS=(
    "nivora-aura-llm-api-key"
    "nivora-aura-spotify-access-token"
    "nivora-aura-spotify-refresh-token"
    "nivora-aura-ha-access-token"
)

for key in "${KEYS[@]}"; do
    echo "Checking: $key"
    # Try to retrieve the key
    if secret-tool lookup service "nivora-aura" key "$key" &>/dev/null; then
        # Key exists, clear it
        secret-tool clear service "nivora-aura" key "$key" 2>/dev/null && \
            echo "  ✓ Cleared $key" || \
            echo "  ✗ Failed to clear $key"
    else
        echo "  ℹ $key not found (already cleared)"
    fi
done

echo ""
echo "✅ Keyring cleanup complete!"
