# Spotify Music Integration - User Guide

**Welcome to Spotify integration for Nivora Aura!** 🎵

Control your Spotify playback with natural voice commands, completely hands-free. This guide will walk you through setup and usage.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Step-by-Step Setup](#step-by-step-setup)
4. [Voice Commands](#voice-commands)
5. [Troubleshooting](#troubleshooting)
6. [Privacy & Security](#privacy--security)
7. [FAQ](#faq)

---

## Prerequisites

Before you begin, make sure you have:

✅ **Spotify Premium Account** - Required for playback control
✅ **Active Spotify Device** - Open Spotify on your desktop, phone, or speaker
✅ **Internet Connection** - For OAuth2 authentication and API calls

---

## Quick Start

**TL;DR:**
1. Create a Spotify app at [developer.spotify.com/dashboard](https://developer.spotify.com/dashboard)
2. Copy your Client ID
3. Open Aura Settings → Spotify Integration
4. Paste Client ID and click "Connect Spotify"
5. Authorize in the browser
6. Say "Hey Aura, play Despacito"

---

## Step-by-Step Setup

### Step 1: Create a Spotify Developer App

1. **Go to Spotify Developer Dashboard**
   - Visit: [https://developer.spotify.com/dashboard](https://developer.spotify.com/dashboard)
   - Log in with your Spotify account

2. **Create a New App**
   - Click the **"Create app"** button
   - Fill in the app details:
     - **App name**: `Nivora Aura` (or any name you prefer)
     - **App description**: `Personal voice assistant music control`
     - **Website**: Leave blank or use `http://localhost`
     - **Redirect URI**: **IMPORTANT!** Enter exactly:
       ```
       http://127.0.0.1:8888/callback
       ```
     - **API/SDKs**: Check ✅ **Web API**
   - Accept Spotify's Terms of Service
   - Click **"Save"**

3. **Get Your Client ID**
   - You'll be taken to your app's dashboard
   - Find the **"Client ID"** field (a long string of letters and numbers)
   - Click **"Show client secret"** - You **DON'T need the secret** (we use PKCE for security)
   - Copy the **Client ID** (it looks like: `a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6`)

### Step 2: Connect Spotify in Aura

1. **Open Aura Settings**
   - Click the Settings icon in Aura
   - Scroll down to **"Spotify Music Integration"**

2. **Enter Your Client ID**
   - Paste your Client ID into the **"Spotify Client ID"** field
   - Triple-check it's correct (no spaces, no typos)

3. **Click "Connect Spotify"**
   - Your default browser will open
   - You'll see Spotify's authorization page
   - **Review the permissions** - Aura requests:
     - View your Spotify account data
     - View your currently playing content
     - Control playback on your Spotify devices
     - Access your playlists
   - Click **"Agree"** or **"Accept"**

4. **Success!**
   - You'll see a green success page: "✓ Spotify Connected!"
   - Close the browser tab and return to Aura
   - You should see a green **"Connected to Spotify"** indicator in Settings

---

## Voice Commands

Once connected, you can use these natural voice commands:

### Play Music

| Say This | What Happens |
|----------|--------------|
| "Play Despacito by Luis Fonsi" | Searches for the track and plays it |
| "Play Bohemian Rhapsody" | Plays the first match for that song |
| "Play Queen" | Plays a song by Queen (top result) |
| "Play my Workout playlist" | Searches your playlists and plays it |

### Control Playback

| Say This | What Happens |
|----------|--------------|
| "Pause" or "Stop" | Pauses current playback |
| "Resume" or "Continue" or "Play" | Resumes paused playback |
| "Next" or "Skip" | Skips to next track |
| "Previous" or "Go back" | Skips to previous track |

### Get Information

| Say This | What Happens |
|----------|--------------|
| "What's playing?" | Tells you the current track and artist |
| "What song is this?" | Same as above |

---

## Troubleshooting

### "No active Spotify device found"

**Problem:** Aura can't find a device to play music on.

**Solution:**
1. Open Spotify on **any device** (desktop app, phone, tablet, speaker)
2. Play a song for a few seconds, then pause it
3. This "activates" the device for Spotify Connect
4. Try your Aura voice command again

**Note:** Aura uses Spotify Connect, meaning it controls playback on your existing Spotify devices. It doesn't stream audio directly.

---

### "Spotify Premium required"

**Problem:** You don't have Spotify Premium.

**Solution:**
- Unfortunately, Spotify's API requires Premium for playback control
- Upgrade to Spotify Premium at [spotify.com/premium](https://www.spotify.com/premium)
- Free accounts can still browse but cannot control playback

---

### "Spotify not connected" Error

**Problem:** OAuth tokens expired or were deleted.

**Solution:**
1. Go to Settings → Spotify Integration
2. Click "Disconnect" if you see it
3. Click "Connect Spotify" again
4. Re-authorize in the browser

---

### Songs Not Playing / Wrong Song

**Problem:** Voice recognition isn't perfect, or song isn't available.

**Solutions:**
- Speak clearly and include the artist name: "Play [song] by [artist]"
- Use the exact song title if possible
- Check if the song is available in your region on Spotify
- Try searching manually in Spotify first to verify the exact name

---

### Authorization Page Doesn't Open

**Problem:** Browser doesn't open when clicking "Connect Spotify".

**Solution:**
1. Check your default browser is set correctly
2. Manually open: [https://accounts.spotify.com/authorize](https://accounts.spotify.com/authorize)
3. Try a different browser
4. Check firewall/antivirus isn't blocking the connection

---

### "Failed to refresh access token"

**Problem:** Automatic token refresh failed.

**Solution:**
1. Check your internet connection
2. Go to Settings → Spotify Integration
3. Disconnect and reconnect Spotify
4. If the problem persists, check [Spotify Status](https://status.spotify.com) for outages

---

## Privacy & Security

### Your Data is Safe

**Nivora Aura takes your privacy seriously:**

✅ **No Client Secret Stored** - We use PKCE (Proof Key for Code Exchange), the most secure OAuth2 flow
✅ **Tokens in OS Keyring** - Your access tokens are stored in your system's native credential manager:
  - **macOS**: Keychain
  - **Windows**: Credential Manager
  - **Linux**: Secret Service (libsecret)

✅ **No Telemetry** - We never send your listening data or commands to any server
✅ **Local Processing** - All voice recognition happens on your device
✅ **Minimal Permissions** - We only request the scopes needed for playback control

### What Spotify Knows

When you authorize Aura:
- Spotify knows that "Nivora Aura" (your app name) is accessing your account
- Spotify logs API calls (what songs you play, when you pause, etc.)
- **Aura does not send any additional analytics to anyone**

### Disconnecting

To revoke access at any time:
1. **In Aura**: Settings → Spotify Integration → Disconnect
2. **In Spotify**: [Account Settings](https://www.spotify.com/account/apps/) → Remove "Nivora Aura"

---

## FAQ

### Q: Do I need to keep the browser open after connecting?

**A:** No! Once you see "Spotify Connected!" you can close the browser. Aura has the tokens it needs.

---

### Q: How long does the connection last?

**A:** Indefinitely! Aura automatically refreshes your access token before it expires. You should never need to re-authorize unless you explicitly disconnect.

---

### Q: Can I use this without Spotify Premium?

**A:** Unfortunately, no. Spotify's API requires Premium for playback control. You need an active Premium subscription.

---

### Q: What if I have multiple Spotify devices?

**A:** Great! Spotify Connect works across all your devices. Music will play on whichever device was most recently active. You can change the active device in the Spotify app.

---

### Q: Can Aura play music from other services (Apple Music, YouTube Music)?

**A:** Not yet! Spotify is our first music integration. Other services may be added in the future based on user demand.

---

### Q: Does this work offline?

**A:** No. Music playback requires:
- Internet connection (to call Spotify's API)
- Active Spotify device (streaming from Spotify)

However, **voice recognition is 100% offline** - your commands are processed locally.

---

### Q: Why do I need to create a Spotify Developer app?

**A:** This is for **your security and privacy**. By creating your own app:
- You control the Client ID (we don't store it on our servers)
- You can revoke access anytime from your Spotify account
- We can't access your account without your explicit authorization
- This follows industry best practices for OAuth2 security

---

### Q: What voice models work best for music commands?

**A:** All STT models work! For best results:
- Use "Base" or "Small" Whisper models (Settings → Voice Settings)
- Speak clearly and at a normal pace
- Include artist names for more accurate results

---

## Need Help?

If you encounter issues not covered in this guide:

1. **Check Logs**: Aura's console may have detailed error messages
2. **Verify Spotify Status**: [status.spotify.com](https://status.spotify.com)
3. **Report Issues**: [github.com/hivezga/nivora-aura-desktop/issues](https://github.com/hivezga/nivora-aura-desktop/issues)
4. **Community**: Join discussions in our GitHub repository

---

## Happy Listening! 🎵

You're all set! Enjoy hands-free music control with Aura.

Try saying:
- "Hey Aura, play some music"
- "Hey Aura, play my Discover Weekly playlist"
- "Hey Aura, what's playing?"

**Remember:** Spotify Premium + active device required. Have fun! 🎧

---

**Document Version**: 1.0
**Last Updated**: 2025-10-10
**Aura Version**: 0.1.0+
