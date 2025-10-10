# Spotify Music Integration - Testing Guide

**Comprehensive testing procedures for the Spotify Music Integration feature.**

This document outlines all test cases, expected results, and verification procedures to ensure the feature meets all acceptance criteria and performs reliably in production.

---

## Table of Contents

1. [Test Environment Setup](#test-environment-setup)
2. [AC1: OAuth2 Authentication Tests](#ac1-oauth2-authentication-tests)
3. [AC2: Intent Recognition Tests](#ac2-intent-recognition-tests)
4. [AC3: Spotify API Client Tests](#ac3-spotify-api-client-tests)
5. [AC4: Playback Control Tests](#ac4-playback-control-tests)
6. [AC5: Frontend UI Tests](#ac5-frontend-ui-tests)
7. [Edge Case & Error Handling Tests](#edge-case--error-handling-tests)
8. [Performance & Reliability Tests](#performance--reliability-tests)
9. [Security Tests](#security-tests)
10. [Cross-Platform Tests](#cross-platform-tests)

---

## Test Environment Setup

### Prerequisites

Before testing, ensure you have:

- ✅ Aura Desktop built from latest source (`cargo build`)
- ✅ Spotify Premium account with active subscription
- ✅ Spotify Developer app created with correct redirect URI
- ✅ At least one active Spotify Connect device (desktop, mobile, speaker)
- ✅ Internet connection (for OAuth and API calls)
- ✅ Test playlists created in Spotify account:
  - "Test Playlist" (public or private)
  - "Workout" (for playlist command testing)

### Test Data Preparation

**Known Songs for Testing:**
- "Despacito" by Luis Fonsi
- "Bohemian Rhapsody" by Queen
- "Shape of You" by Ed Sheeran
- "Blinding Lights" by The Weeknd

**Test Playlist:** Create a playlist named "Test Playlist" with 5+ songs

---

## AC1: OAuth2 Authentication Tests

### Test 1.1: Initial OAuth2 Connection Flow

**Objective:** Verify complete OAuth2 PKCE flow from start to finish.

**Prerequisites:**
- Spotify not connected
- Valid Client ID obtained from Spotify Developer Dashboard

**Steps:**
1. Launch Aura Desktop
2. Open Settings → Spotify Music Integration
3. Verify "Disconnected" state is displayed
4. Enter Spotify Client ID in the input field
5. Click "Connect Spotify" button
6. Verify browser opens to Spotify authorization page
7. Log in to Spotify (if not already logged in)
8. Review requested permissions
9. Click "Agree" or "Accept"
10. Verify browser redirects to success page ("✓ Spotify Connected!")
11. Return to Aura Settings
12. Verify "Connected to Spotify" indicator is displayed (green dot, pulsing)

**Expected Results:**
✅ Browser opens automatically
✅ Spotify authorization page loads correctly
✅ After authorization, success page displays
✅ Aura Settings shows "Connected" status
✅ Client ID is truncated in display (e.g., "a1b2c3d4e5f6...")
✅ No errors in console

**Verification:**
```bash
# macOS - Check keyring
security find-generic-password -s "com.nivora.aura-desktop" -a "spotify_access_token"

# Linux - Check keyring
secret-tool lookup service com.nivora.aura-desktop username spotify_access_token

# Should return: password exists (don't display it)
```

---

### Test 1.2: OAuth2 Callback Timeout

**Objective:** Verify timeout handling when user doesn't authorize within 120 seconds.

**Steps:**
1. Click "Connect Spotify"
2. Browser opens to authorization page
3. Wait 121 seconds **without** clicking "Agree"
4. Verify timeout error is displayed in Aura

**Expected Results:**
✅ After 120 seconds, error message displayed: "Timeout waiting for OAuth callback"
✅ Settings still shows "Disconnected" state
✅ User can retry connection

---

### Test 1.3: OAuth2 Cancellation

**Objective:** Verify graceful handling when user denies authorization.

**Steps:**
1. Click "Connect Spotify"
2. Browser opens
3. Click "Cancel" or "Deny" on Spotify authorization page
4. Verify error handling in Aura

**Expected Results:**
✅ Error message displayed: "Authorization failed: access_denied"
✅ Settings returns to "Disconnected" state
✅ User can retry connection

---

### Test 1.4: Token Storage Verification

**Objective:** Verify tokens are stored securely in OS keyring.

**Steps:**
1. Complete OAuth2 connection
2. Verify 3 entries in OS keyring:
   - `spotify_access_token`
   - `spotify_refresh_token`
   - `spotify_token_expiry`

**Expected Results:**
✅ All 3 entries exist in keyring
✅ Tokens are NOT visible in:
  - Database file (~/Library/Application Support/nivora-aura/aura.db)
  - Application logs
  - Settings UI (Client ID truncated)

---

### Test 1.5: Disconnect Flow

**Objective:** Verify complete disconnection and token removal.

**Steps:**
1. With Spotify connected, click "Disconnect" button
2. Confirm disconnection in popup
3. Verify "Disconnected" state
4. Check keyring for token removal

**Expected Results:**
✅ Confirmation dialog appears
✅ After confirmation, "Disconnected" state displayed
✅ All tokens removed from OS keyring
✅ Database updated (`spotify_connected = false`)
✅ User can reconnect without errors

---

### Test 1.6: Automatic Token Refresh

**Objective:** Verify automatic token refresh before expiry.

**Steps:**
1. Connect Spotify
2. Note token expiry time (typically 1 hour from connection)
3. Use Aura for 55 minutes (wait or use other features)
4. Make a Spotify API call (e.g., "What's playing?")
5. Verify token is refreshed automatically

**Expected Results:**
✅ No user intervention required
✅ API call succeeds
✅ New token expiry time is updated (another hour from refresh)
✅ No error messages

**Note:** This test requires patience or manual manipulation of token expiry time in keyring for faster testing.

---

## AC2: Intent Recognition Tests

### Test 2.1: Play Song with Artist

**Objective:** Verify "Play [song] by [artist]" pattern recognition.

**Test Cases:**

| Input | Expected Intent | Expected Entities |
|-------|----------------|-------------------|
| "play Despacito by Luis Fonsi" | PlaySong | song: "Despacito", artist: "Luis Fonsi" |
| "Play Bohemian Rhapsody by Queen" | PlaySong | song: "Bohemian Rhapsody", artist: "Queen" |
| "PLAY SHAPE OF YOU BY ED SHEERAN" | PlaySong | song: "Shape Of You", artist: "Ed Sheeran" (title case) |

**Steps:**
1. Say or type each command
2. Verify correct song plays
3. Verify response message includes song and artist

**Expected Results:**
✅ Correct song identified and played
✅ Response: "Now playing: [Song] by [Artist]"

---

### Test 2.2: Play Song Without Artist

**Objective:** Verify "Play [song]" pattern recognition.

**Test Cases:**

| Input | Expected Intent | Expected Behavior |
|-------|----------------|-------------------|
| "play Despacito" | PlaySong | Searches for "Despacito", plays top result |
| "Play Imagine" | PlaySong | Searches for "Imagine", plays top result |

**Expected Results:**
✅ Song plays (may vary if multiple songs have same name)
✅ Response indicates which song was chosen

---

### Test 2.3: Play Playlist

**Objective:** Verify "Play my [playlist] playlist" pattern recognition.

**Test Cases:**

| Input | Expected Intent | Expected Entities |
|-------|----------------|-------------------|
| "play my Test Playlist playlist" | PlayPlaylist | playlist_name: "Test Playlist" |
| "Play my workout playlist" | PlayPlaylist | playlist_name: "Workout" |
| "play chill vibes playlist" | PlayPlaylist | playlist_name: "Chill Vibes" |

**Expected Results:**
✅ Playlist found in user's Spotify playlists
✅ Response: "Found playlist '[name]' with [X] tracks"

**Note:** Current implementation finds playlists but full playback requires context URI (future enhancement).

---

### Test 2.4: Playback Control Commands

**Objective:** Verify pause, resume, next, previous recognition.

**Test Cases:**

| Input | Expected Intent | Expected Action |
|-------|----------------|-----------------|
| "pause" | Pause | Playback pauses |
| "stop" | Pause | Playback pauses |
| "resume" | Resume | Playback resumes |
| "continue" | Resume | Playback resumes |
| "play" (when paused) | Resume | Playback resumes |
| "next" | Next | Skips to next track |
| "skip" | Next | Skips to next track |
| "previous" | Previous | Skips to previous track |
| "go back" | Previous | Skips to previous track |

**Expected Results:**
✅ All variations recognized correctly
✅ Playback control executes successfully
✅ Response confirms action ("Music paused", "Next track", etc.)

---

### Test 2.5: Get Current Track

**Objective:** Verify "What's playing?" recognition.

**Test Cases:**

| Input | Expected Intent | Expected Response |
|-------|----------------|-------------------|
| "what's playing?" | GetCurrentTrack | "Now playing: [Song] by [Artist]" |
| "what song is this?" | GetCurrentTrack | "Now playing: [Song] by [Artist]" |
| "what's this song?" | GetCurrentTrack | "Now playing: [Song] by [Artist]" |

**Expected Results:**
✅ Current track info retrieved
✅ Response includes song name and artist
✅ If paused, includes "Paused: [Song] by [Artist]"

---

### Test 2.6: Unknown Commands

**Objective:** Verify graceful handling of unrecognized music commands.

**Test Cases:**

| Input | Expected Intent | Expected Response |
|-------|----------------|-------------------|
| "make me a sandwich" | Unknown | Helpful error message |
| "play music backwards" | Unknown | Helpful error message |

**Expected Results:**
✅ Response: "I didn't understand that music command. Try 'play [song] by [artist]', 'pause', 'next', or 'what's playing?'"
✅ No crashes or exceptions

---

## AC3: Spotify API Client Tests

### Test 3.1: Search API

**Objective:** Verify track search functionality.

**Steps:**
1. Use backend directly or trigger via voice command
2. Search for "Despacito" with artist "Luis Fonsi"
3. Verify results returned

**Expected Results:**
✅ Returns array of tracks (up to 10)
✅ First result is "Despacito" by Luis Fonsi
✅ Track objects include: id, name, uri, artists, album
✅ Search completes within 2 seconds

---

### Test 3.2: Playback Control API

**Objective:** Verify all playback endpoints work correctly.

**Test Cases:**

| Endpoint | Method | Test Action | Expected Result |
|----------|--------|-------------|-----------------|
| /me/player/play | PUT | Start playback | 204 No Content or 202 Accepted |
| /me/player/pause | PUT | Pause playback | 204 No Content |
| /me/player/next | POST | Skip to next | 204 No Content |
| /me/player/previous | POST | Skip to previous | 204 No Content |

**Expected Results:**
✅ All endpoints return success codes
✅ Actions execute within 1 second
✅ Playback state updates correctly on Spotify devices

---

### Test 3.3: Get Current Track API

**Objective:** Verify currently playing endpoint.

**Steps:**
1. Play a song on Spotify
2. Call `spotify_get_current_track` command
3. Verify response data

**Expected Results:**
✅ Returns track info: name, artist, album
✅ Includes `is_playing` boolean
✅ Includes progress_ms and duration_ms
✅ Returns 204 (nothing playing) when no active playback

---

### Test 3.4: Get Devices API

**Objective:** Verify device discovery.

**Steps:**
1. Open Spotify on desktop
2. Open Spotify on mobile
3. Call `spotify_get_devices` command
4. Verify both devices listed

**Expected Results:**
✅ Returns array of devices
✅ Each device includes: id, name, type, is_active
✅ Active device has `is_active: true`

---

### Test 3.5: Rate Limit Handling

**Objective:** Verify exponential backoff on rate limits.

**Steps:**
1. Make rapid API calls to trigger rate limiting (429 response)
2. Verify automatic retry with backoff

**Expected Results:**
✅ Retries up to 5 times with exponential delays (1s, 2s, 4s, 8s, 16s)
✅ If rate limited after 5 retries, returns error: "Rate limit exceeded"
✅ No crashes or infinite loops

**Note:** Difficult to test without intentionally triggering rate limits. Monitor logs for retry behavior.

---

### Test 3.6: Token Expiry Handling

**Objective:** Verify automatic token refresh on 401 errors.

**Steps:**
1. Manually expire access token in keyring (or wait 1 hour)
2. Make any API call
3. Verify token is automatically refreshed

**Expected Results:**
✅ API call succeeds after automatic refresh
✅ New token saved to keyring
✅ User sees no errors
✅ Log shows: "Spotify token expired or expiring soon, refreshing..."

---

## AC4: Playback Control Tests

### Test 4.1: Play Track on Active Device

**Objective:** Verify track playback on Spotify Connect device.

**Prerequisites:**
- Spotify open on at least one device
- Device is "active" (recently played music on it)

**Steps:**
1. Say "play Despacito by Luis Fonsi"
2. Verify music starts playing on active Spotify device
3. Verify Aura responds with confirmation

**Expected Results:**
✅ Track plays on active device within 2-3 seconds
✅ Aura response: "Now playing: Despacito by Luis Fonsi"
✅ If device was paused, playback resumes

---

### Test 4.2: Pause/Resume Playback

**Objective:** Verify playback control commands.

**Steps:**
1. Play a song
2. Say "pause"
3. Verify playback pauses
4. Say "resume"
5. Verify playback resumes from same position

**Expected Results:**
✅ Pause happens immediately (<1 second)
✅ Resume continues from where it left off
✅ Aura confirms each action

---

### Test 4.3: Skip Forward/Backward

**Objective:** Verify track navigation.

**Steps:**
1. Play a playlist or album
2. Say "next"
3. Verify skip to next track
4. Say "previous"
5. Verify return to previous track

**Expected Results:**
✅ Skip happens immediately
✅ Correct track plays
✅ Track info updated

---

### Test 4.4: No Active Device Error

**Objective:** Verify graceful error when no Spotify device is active.

**Steps:**
1. Close Spotify on all devices
2. Say "play Despacito"
3. Verify helpful error message

**Expected Results:**
✅ Error message: "No active Spotify device found - please open Spotify on one of your devices"
✅ Aura doesn't crash
✅ User can retry after opening Spotify

---

### Test 4.5: Premium Required Error

**Objective:** Verify error handling for non-Premium accounts.

**Prerequisites:**
- Test with Spotify Free account (or simulate 403 response)

**Steps:**
1. Attempt playback control with Free account
2. Verify error message

**Expected Results:**
✅ Error: "Spotify Premium required for playback control"
✅ Helpful message explaining why

**Note:** Difficult to test without Free account. Can be simulated by modifying client to return 403.

---

## AC5: Frontend UI Tests

### Test 5.1: Settings Modal - Disconnected State

**Objective:** Verify disconnected UI displays correctly.

**Steps:**
1. Disconnect Spotify (if connected)
2. Open Settings → Spotify Integration
3. Verify UI elements

**Expected Results:**
✅ "Spotify Client ID" input field visible
✅ Placeholder text: "Enter your Spotify app client ID"
✅ "Connect Spotify" button visible and clickable
✅ Help text with link to developer.spotify.com/dashboard
✅ Redirect URI displayed: `http://127.0.0.1:8888/callback`
✅ Instructions numbered 1, 2, 3

---

### Test 5.2: Settings Modal - Connected State

**Objective:** Verify connected UI displays correctly.

**Steps:**
1. Connect Spotify
2. Open Settings → Spotify Integration
3. Verify UI elements

**Expected Results:**
✅ Green pulsing dot indicator
✅ "Connected to Spotify" text
✅ Truncated Client ID displayed (e.g., "a1b2c3d4e5f6...")
✅ "Disconnect" button visible (red, outline style)
✅ Voice commands list displayed with examples
✅ No "Connect" button or Client ID input

---

### Test 5.3: Connect Button Interaction

**Objective:** Verify button states during connection.

**Steps:**
1. Click "Connect Spotify"
2. Observe button state changes

**Expected Results:**
✅ Button disabled immediately after click
✅ Button text changes to "Connecting..."
✅ Help text appears: "Your browser will open for authorization..."
✅ Button re-enables after authorization or error

---

### Test 5.4: Disconnect Button Confirmation

**Objective:** Verify disconnect confirmation dialog.

**Steps:**
1. Click "Disconnect" button
2. Verify confirmation popup

**Expected Results:**
✅ Confirmation dialog appears: "Are you sure you want to disconnect Spotify?"
✅ If "Cancel" clicked, nothing happens (stays connected)
✅ If "OK" clicked, disconnection proceeds

---

### Test 5.5: Client ID Validation

**Objective:** Verify input validation for Client ID.

**Steps:**
1. Leave Client ID field empty
2. Attempt to click "Connect Spotify"
3. Verify button is disabled

**Expected Results:**
✅ "Connect Spotify" button disabled when field is empty
✅ Button enables when user types in field
✅ Trimming whitespace works correctly

---

## Edge Case & Error Handling Tests

### Test 6.1: Network Disconnection During OAuth

**Objective:** Verify handling of network loss during authentication.

**Steps:**
1. Click "Connect Spotify"
2. Disable network/Wi-Fi immediately
3. Observe error handling

**Expected Results:**
✅ Error message displayed: "Network error: ..."
✅ Aura doesn't crash
✅ User can retry after reconnecting

---

### Test 6.2: Invalid Client ID

**Objective:** Verify error handling for incorrect Client ID.

**Steps:**
1. Enter invalid Client ID (e.g., "invalid123")
2. Click "Connect Spotify"
3. Authorize in browser (if it gets that far)

**Expected Results:**
✅ Error from Spotify: "Invalid client ID" or similar
✅ Aura displays error to user
✅ User can correct and retry

---

### Test 6.3: Search Returns No Results

**Objective:** Verify handling when song not found.

**Steps:**
1. Say "play asdfghjklqwerty by nobody"
2. Verify error message

**Expected Results:**
✅ Error: "No tracks found for 'asdfghjklqwerty by nobody'"
✅ No playback attempted
✅ User can retry with different song

---

### Test 6.4: Spotify API Outage

**Objective:** Verify handling of Spotify service disruption.

**Prerequisites:**
- Simulate by blocking api.spotify.com in /etc/hosts or firewall

**Steps:**
1. Attempt playback command
2. Observe timeout and error handling

**Expected Results:**
✅ Timeout after 30 seconds
✅ Error message: "Network error: ..." or "API request failed"
✅ Aura doesn't crash
✅ User can retry when service is restored

---

### Test 6.5: Concurrent Playback Requests

**Objective:** Verify handling of rapid successive commands.

**Steps:**
1. Say "play Despacito"
2. Immediately say "pause" before first command completes
3. Verify no race conditions or crashes

**Expected Results:**
✅ Commands execute in order or second command succeeds
✅ No crashes or deadlocks
✅ Final state is consistent (paused in this case)

---

## Performance & Reliability Tests

### Test 7.1: Search Response Time

**Objective:** Verify search completes within acceptable time.

**Steps:**
1. Measure time from command to first result
2. Test with 10 different songs

**Expected Results:**
✅ Average search time < 2 seconds
✅ 95th percentile < 3 seconds

---

### Test 7.2: Playback Control Latency

**Objective:** Verify playback actions are responsive.

**Steps:**
1. Measure time from "pause" command to actual pause
2. Test 10 times

**Expected Results:**
✅ Average latency < 1 second
✅ 95th percentile < 1.5 seconds

---

### Test 7.3: Memory Leak Test

**Objective:** Verify no memory leaks during extended use.

**Steps:**
1. Connect/disconnect Spotify 50 times
2. Play 100 songs in sequence
3. Monitor memory usage

**Expected Results:**
✅ Memory usage remains stable
✅ No significant increase after repetitions

---

## Security Tests

### Test 8.1: Token Not in Logs

**Objective:** Verify tokens never appear in logs or console.

**Steps:**
1. Enable debug logging
2. Complete OAuth flow
3. Make API calls
4. Review all logs

**Expected Results:**
✅ Access token never logged
✅ Refresh token never logged
✅ Only "token refreshed successfully" type messages

---

### Test 8.2: Token Not in Database

**Objective:** Verify tokens are NOT stored in database.

**Steps:**
1. Connect Spotify
2. Open database file: `~/Library/Application Support/nivora-aura/aura.db`
3. Search for token strings

**Expected Results:**
✅ No access_token in database
✅ No refresh_token in database
✅ Only `spotify_connected = true` and `spotify_client_id` present

---

### Test 8.3: PKCE Challenge Entropy

**Objective:** Verify PKCE verifier is cryptographically random.

**Steps:**
1. Generate 100 PKCE verifiers
2. Verify no duplicates
3. Verify length = 128 characters
4. Verify character set is correct ([A-Z], [a-z], [0-9], -, ., _, ~)

**Expected Results:**
✅ All verifiers unique
✅ All exactly 128 characters
✅ All use valid character set
✅ High entropy (no obvious patterns)

---

## Cross-Platform Tests

### Test 9.1: macOS Keychain Integration

**Platform:** macOS

**Steps:**
1. Connect Spotify on macOS
2. Open Keychain Access app
3. Search for "com.nivora.aura-desktop"
4. Verify 3 entries

**Expected Results:**
✅ Tokens stored in Keychain
✅ Tokens marked as "application password" type
✅ Access restricted to Aura app

---

### Test 9.2: Windows Credential Manager Integration

**Platform:** Windows

**Steps:**
1. Connect Spotify on Windows
2. Open Credential Manager (Control Panel → Credential Manager)
3. Search for "com.nivora.aura-desktop"

**Expected Results:**
✅ Tokens stored in Credential Manager
✅ Visible under "Generic Credentials"

---

### Test 9.3: Linux Secret Service Integration

**Platform:** Linux (GNOME/KDE)

**Steps:**
1. Connect Spotify on Linux
2. Use `secret-tool` to verify storage:
   ```bash
   secret-tool lookup service com.nivora.aura-desktop username spotify_access_token
   ```

**Expected Results:**
✅ Tokens stored in Secret Service
✅ Accessible via libsecret

---

## Test Summary Template

After completing all tests, fill out this summary:

### Test Execution Summary

**Date:** [Date]
**Tester:** [Name]
**Aura Version:** [Version]
**Platform:** [macOS / Windows / Linux]

| Test Category | Total Tests | Passed | Failed | Skipped | Pass Rate |
|---------------|-------------|--------|--------|---------|-----------|
| OAuth2 Authentication | 6 | _ | _ | _ | _% |
| Intent Recognition | 6 | _ | _ | _ | _% |
| Spotify API Client | 6 | _ | _ | _ | _% |
| Playback Control | 5 | _ | _ | _ | _% |
| Frontend UI | 5 | _ | _ | _ | _% |
| Edge Cases & Errors | 5 | _ | _ | _ | _% |
| Performance | 3 | _ | _ | _ | _% |
| Security | 3 | _ | _ | _ | _% |
| Cross-Platform | 3 | _ | _ | _ | _% |
| **TOTAL** | **42** | **_** | **_** | **_** | **_%** |

### Issues Found

| Issue ID | Severity | Description | Status |
|----------|----------|-------------|--------|
| SPOT-001 | High | [Description] | Open/Fixed |
| ... | ... | ... | ... |

### Recommendations

- [ ] All critical (High/Critical severity) issues resolved before release
- [ ] Performance benchmarks meet targets
- [ ] Cross-platform testing complete on all supported OSes
- [ ] User documentation reviewed and accurate
- [ ] Code review complete

---

## Conclusion

This testing guide ensures comprehensive coverage of all Spotify integration functionality. All tests should pass before the feature is considered production-ready.

**Happy Testing!** 🧪

---

**Document Version**: 1.0
**Last Updated**: 2025-10-10
**Epic**: Spotify Music Integration
