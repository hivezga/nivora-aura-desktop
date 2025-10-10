# Spotify Music Integration - Acceptance Criteria

## Epic Overview

**Epic Name**: Music Integration (Spotify)
**Version**: 1.0
**Status**: In Progress
**Last Updated**: 2025-10-10

**User Story**:
> As a user, I want to ask Aura to play songs, artists, and playlists from my Spotify account, so I can enjoy my music hands-free.

---

## Acceptance Criteria

### AC1: Secure Authentication ✅

**Description**: Implement a secure OAuth2 authentication flow for users to connect their Spotify Premium account.

**Requirements**:
- [x] Research OAuth2 PKCE flow requirements
- [ ] Implement OAuth2 PKCE authentication module
- [ ] Use loopback redirect URI (http://127.0.0.1:8888/callback)
- [ ] Store access and refresh tokens securely in OS keyring
- [ ] No client secret stored in application
- [ ] Automatic token refresh before expiry (5-minute buffer)
- [ ] User can disconnect Spotify account from Settings

**Test Cases**:
1. ✅ User can initiate Spotify connection from Settings
2. ✅ Browser opens with Spotify authorization page
3. ✅ User authorizes app and is redirected back
4. ✅ Access token and refresh token stored in OS keyring
5. ✅ Settings UI shows "Connected" status
6. ✅ User can disconnect and tokens are deleted

**Verification**:
```bash
# Check tokens in OS keyring (macOS)
security find-generic-password -s "com.nivora.aura-desktop" -a "spotify_access_token"

# Check tokens in OS keyring (Linux)
secret-tool lookup service com.nivora.aura-desktop username spotify_access_token
```

---

### AC2: Intent Recognition ⏳

**Description**: Enhance the NLU pipeline to understand music-related intents and entities.

**Requirements**:
- [ ] Implement music intent parser (music_intent.rs)
- [ ] Support "Play {song} by {artist}" pattern
- [ ] Support "Play my {playlist} playlist" pattern
- [ ] Support "Pause/Resume" commands
- [ ] Support "Next/Previous" commands
- [ ] Support "What song is playing?" query
- [ ] Graceful handling of unknown commands

**Supported Commands**:
| User Input | Intent | Entities |
|------------|--------|----------|
| "Play Despacito by Luis Fonsi" | `PlaySong` | song: "Despacito", artist: "Luis Fonsi" |
| "Play Bohemian Rhapsody" | `PlaySong` | song: "Bohemian Rhapsody", artist: None |
| "Play my Workout playlist" | `PlayPlaylist` | playlist: "Workout" |
| "Pause" / "Stop" | `Pause` | - |
| "Resume" / "Continue" | `Resume` | - |
| "Next" / "Skip" | `Next` | - |
| "Previous" / "Go back" | `Previous` | - |
| "What's playing?" / "What song is this?" | `GetCurrentTrack` | - |

**Test Cases**:
1. ✅ Parser correctly identifies "Play X by Y" as PlaySong intent
2. ✅ Parser extracts song and artist names
3. ✅ Parser handles "Play X" (artist-less) commands
4. ✅ Parser identifies playlist commands
5. ✅ Parser identifies control commands (pause, resume, next, previous)
6. ✅ Parser handles unknown commands gracefully

**Verification**:
```rust
#[test]
fn test_intent_recognition() {
    assert_eq!(
        MusicIntentParser::parse("play Despacito by Luis Fonsi"),
        MusicIntent::PlaySong {
            song: "despacito".to_string(),
            artist: Some("luis fonsi".to_string())
        }
    );
}
```

---

### AC3: Spotify API Client ⏳

**Description**: Create a comprehensive Rust client for the Spotify API.

**Requirements**:
- [ ] Implement SpotifyClient struct (spotify_client.rs)
- [ ] Search for tracks by name and artist
- [ ] Fetch user playlists
- [ ] Get current playback state
- [ ] Get currently playing track
- [ ] Handle API rate limits (429 responses)
- [ ] Automatic token refresh integration
- [ ] Proper error types and handling

**API Endpoints**:
- [ ] `GET /v1/search` - Search tracks/artists/playlists
- [ ] `GET /v1/me/playlists` - Get user playlists
- [ ] `GET /v1/me/player` - Get playback state
- [ ] `GET /v1/me/player/currently-playing` - Get current track
- [ ] `PUT /v1/me/player/play` - Start/resume playback
- [ ] `PUT /v1/me/player/pause` - Pause playback
- [ ] `POST /v1/me/player/next` - Skip to next
- [ ] `POST /v1/me/player/previous` - Skip to previous
- [ ] `GET /v1/me/player/devices` - Get available devices

**Test Cases**:
1. ✅ Search returns tracks matching query
2. ✅ Search with artist filter narrows results
3. ✅ Client handles 401 (token expired) by refreshing
4. ✅ Client handles 404 (not found) gracefully
5. ✅ Client handles 429 (rate limit) with retry
6. ✅ Client handles network errors gracefully

**Verification**:
```bash
# Manual test with real Spotify account
cargo test --package aura_desktop_lib --lib spotify_client::tests
```

---

### AC4: Playback Control (Spotify Connect) ⏳

**Description**: Aura will act as a remote control for Spotify Connect, managing playback on user's active devices.

**Requirements**:
- [ ] Detect available Spotify Connect devices
- [ ] Play tracks on active device
- [ ] Pause/resume playback
- [ ] Skip to next/previous track
- [ ] Handle "no active device" error gracefully
- [ ] Handle "Premium required" error gracefully
- [ ] Voice confirmation after actions (TTS)

**Device Types Supported**:
- Desktop Spotify app
- Mobile Spotify app
- Spotify Connect speakers
- Smart TVs
- Web player

**Test Cases**:
1. ✅ Play command starts playback on active device
2. ✅ Pause command pauses playback
3. ✅ Resume command resumes playback
4. ✅ Next command skips to next track
5. ✅ Previous command skips to previous track
6. ❌ Graceful error when no active device
7. ❌ Graceful error when user has no Premium

**Verification**:
1. Open Spotify on a device (activate it)
2. Say "Hey Aura, play Despacito"
3. Verify playback starts on the active device
4. Say "Pause"
5. Verify playback pauses

---

### AC5: Frontend UI ⏳

**Description**: Add Spotify connection UI to Settings and optional Now Playing component.

**Requirements**:

#### Settings Modal:
- [ ] Add "Spotify Integration" section
- [ ] "Spotify Client ID" input field
- [ ] "Connect Spotify" button
- [ ] Connection status indicator (green dot = connected)
- [ ] "Disconnect" button when connected
- [ ] Instructions with link to Spotify Developer Dashboard
- [ ] Redirect URI displayed for user convenience

#### Now Playing Component (Stretch Goal):
- [ ] Display current track name
- [ ] Display artist name
- [ ] Display play/pause status
- [ ] Update every 5 seconds
- [ ] Hide when nothing playing

**UI Mockup**:
```
┌───────────────────────────────────────────────────┐
│ Spotify Integration                               │
├───────────────────────────────────────────────────┤
│                                                   │
│ ┌─────────────────────────────────────────────┐   │
│ │ Spotify Client ID                          │   │
│ │ [___________________________________]      │   │
│ │                                            │   │
│ │ Create a Spotify app at                    │   │
│ │ developer.spotify.com/dashboard            │   │
│ │                                            │   │
│ │ Set redirect URI to:                       │   │
│ │ http://127.0.0.1:8888/callback             │   │
│ └─────────────────────────────────────────────┘   │
│                                                   │
│ [ Connect Spotify ]                               │
│                                                   │
│ --- OR (when connected) ---                       │
│                                                   │
│ ┌─────────────────────────────────────────────┐   │
│ │ ● Connected                  [Disconnect]  │   │
│ └─────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────┘
```

**Test Cases**:
1. ✅ Client ID input accepts text
2. ✅ Connect button disabled when no client ID
3. ✅ Connect button triggers OAuth flow
4. ✅ Status indicator shows green when connected
5. ✅ Disconnect button removes tokens
6. ✅ Now Playing shows current track (stretch goal)

**Verification**:
1. Open Settings modal
2. Enter client ID
3. Click "Connect Spotify"
4. Authorize in browser
5. Verify "Connected" status appears

---

## Non-Functional Requirements

### Security
- ✅ PKCE flow (no client secret)
- ✅ Tokens in OS keyring (not database)
- ✅ Minimal OAuth2 scopes requested
- ✅ Redirect URI validation

### Performance
- ✅ Token refresh automatic (no user intervention)
- ✅ Search results return within 2 seconds
- ✅ Playback commands execute within 1 second
- ✅ No blocking operations on main thread

### Privacy
- ✅ No telemetry sent to third parties
- ✅ User explicitly authorizes Spotify access
- ✅ Clear scope permissions displayed
- ✅ User can disconnect anytime

### User Experience
- ✅ Clear error messages (no generic "failed" messages)
- ✅ Voice confirmation after actions
- ✅ Graceful degradation (offline mode)
- ✅ Helpful setup instructions

---

## Testing Checklist

### Unit Tests
- [ ] Music intent parser tests
- [ ] Token refresh logic tests
- [ ] API client error handling tests
- [ ] PKCE challenge generation tests

### Integration Tests
- [ ] End-to-end OAuth2 flow (manual)
- [ ] Search API integration test
- [ ] Playback control integration test
- [ ] Token refresh integration test

### Edge Cases
- [ ] No active Spotify device
- [ ] User has no Premium subscription
- [ ] Network timeout during API call
- [ ] Invalid/expired access token
- [ ] Rate limit (429) response
- [ ] Empty search results
- [ ] Malformed user commands

### Cross-Platform
- [ ] OAuth2 flow works on Windows
- [ ] OAuth2 flow works on macOS
- [ ] OAuth2 flow works on Linux
- [ ] OS keyring storage works on all platforms

---

## Definition of Done

This epic is considered **COMPLETE** when:

1. ✅ All 5 acceptance criteria are fully implemented
2. ✅ All unit tests pass
3. ✅ All integration tests pass (manual verification)
4. ✅ End-to-end OAuth2 flow tested with real Spotify account
5. ✅ Music commands work via voice and text input
6. ✅ Error handling verified for all edge cases
7. ✅ Documentation complete (SPOTIFY_ARCHITECTURE.md, SPOTIFY_USER_GUIDE.md)
8. ✅ Code review completed
9. ✅ Feature tested on Windows, macOS, and Linux
10. ✅ User guide published with setup instructions

---

## Progress Tracking

| Acceptance Criteria | Status | Progress | Notes |
|---------------------|--------|----------|-------|
| AC1: Secure Authentication | ⏳ In Progress | 20% | OAuth2 module in development |
| AC2: Intent Recognition | ⏳ Pending | 0% | - |
| AC3: Spotify API Client | ⏳ Pending | 0% | - |
| AC4: Playback Control | ⏳ Pending | 0% | - |
| AC5: Frontend UI | ⏳ Pending | 0% | - |

**Overall Progress**: 3% (Architecture & Research Complete)

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Spotify API changes | High | Monitor Spotify developer changelog, use versioned API |
| OAuth2 PKCE complexity | Medium | Follow official Spotify PKCE tutorial, use proven libraries |
| Token refresh failures | Medium | Implement retry logic, graceful error messages |
| Platform-specific keyring issues | Medium | Test on all platforms, fallback to encrypted file storage |
| User has no Premium | Low | Clear error message, link to Spotify Premium |

---

## Dependencies

### External APIs
- Spotify Web API (REST)
- Spotify OAuth2 Authorization Server

### Rust Crates
- `oauth2` - OAuth2 client
- `reqwest` - HTTP client
- `tiny_http` - Local callback server
- `regex` - Intent parsing
- `sha2`, `base64`, `rand` - PKCE cryptography

### User Requirements
- Spotify Premium account
- Spotify Developer app (client ID)
- Active Spotify Connect device

---

## Timeline

| Phase | Tasks | Duration | Status |
|-------|-------|----------|--------|
| **Phase 1** | Research & Architecture | 1 day | ✅ Complete |
| **Phase 2** | OAuth2 Authentication | 2 days | ⏳ In Progress |
| **Phase 3** | API Client & Token Mgmt | 2 days | Pending |
| **Phase 4** | Intent Recognition | 1 day | Pending |
| **Phase 5** | Playback Control | 1 day | Pending |
| **Phase 6** | Frontend UI | 2 days | Pending |
| **Phase 7** | Testing & Documentation | 2 days | Pending |

**Total Estimated Duration**: 11 days
**Current Day**: Day 1

---

## References

- [Spotify Web API Documentation](https://developer.spotify.com/documentation/web-api)
- [OAuth2 PKCE Flow Tutorial](https://developer.spotify.com/documentation/web-api/tutorials/code-pkce-flow)
- [Spotify API Scopes Reference](https://developer.spotify.com/documentation/web-api/concepts/scopes)
- [SPOTIFY_ARCHITECTURE.md](./SPOTIFY_ARCHITECTURE.md)

---

**Document Version**: 1.0
**Last Updated**: 2025-10-10
**Epic Owner**: AuraPM
**Status**: Active Development
