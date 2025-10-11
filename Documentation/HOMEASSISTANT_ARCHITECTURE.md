# Home Assistant Integration - Technical Architecture

## Overview

This document describes the technical architecture for Home Assistant integration in Nivora Aura. This integration transforms Aura into a comprehensive smart home controller, enabling voice control of lights, switches, sensors, climate devices, and hundreds of other entity types.

---

## Architecture Principles

1. **Real-Time State Sync** - WebSocket connection for instant state updates
2. **Secure Authentication** - Long-Lived Access Tokens in OS keyring
3. **Comprehensive Entity Support** - All Home Assistant domains (light, switch, sensor, etc.)
4. **Natural Voice Commands** - Intelligent NLU for room-based and device-specific control
5. **Offline Graceful Degradation** - Clear error messages when Home Assistant unavailable
6. **Privacy-First** - All communication direct to user's local Home Assistant instance

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend (React)                          │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐  │
│  │ Settings Modal   │  │ Devices View     │  │ Chat View    │  │
│  │ - HA URL         │  │ - Entity list    │  │ - Voice      │  │
│  │ - Token input    │  │ - Live states    │  │   commands   │  │
│  │ - Connection     │  │ - Quick controls │  │              │  │
│  └──────────────────┘  └──────────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ▼ Tauri IPC
┌─────────────────────────────────────────────────────────────────┐
│                       Backend (Rust/Tauri)                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Tauri Commands (lib.rs)                                  │   │
│  │ - ha_connect(url, token)                                 │   │
│  │ - ha_disconnect()                                        │   │
│  │ - ha_get_entities()                                      │   │
│  │ - ha_call_service(domain, service, entity_id, data)     │   │
│  │ - ha_handle_smart_home_command(command)                 │   │
│  └──────────────────────────────────────────────────────────┘   │
│         ▼                    ▼                      ▼            │
│  ┌─────────────┐  ┌──────────────────┐  ┌──────────────────┐   │
│  │ ha_client   │  │ entity_manager   │  │ smarthome_intent │   │
│  │ .rs         │  │ .rs              │  │ .rs              │   │
│  │             │  │                  │  │                  │   │
│  │ - WebSocket │  │ - Entity cache   │  │ - Parse commands │   │
│  │   client    │  │ - State sync     │  │ - Extract room   │   │
│  │ - REST      │  │ - Query by       │  │ - Extract device │   │
│  │   client    │  │   domain/room    │  │ - Match entities │   │
│  │ - Auth      │  │                  │  │                  │   │
│  └─────────────┘  └──────────────────┘  └──────────────────┘   │
│         ▼                    ▼                                   │
│  ┌─────────────┐  ┌──────────────────┐                          │
│  │ secrets.rs  │  │ database.rs      │                          │
│  │ (Extended)  │  │ (Extended)       │                          │
│  │             │  │                  │                          │
│  │ - HA token  │  │ - HA URL         │                          │
│  │ - OS keyring│  │ - Connection     │                          │
│  └─────────────┘  └──────────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
                              ▼
                  ┌───────────────────────┐
                  │ Home Assistant        │
                  │ (User's Local Server) │
                  │ - WebSocket: /api/    │
                  │   websocket           │
                  │ - REST: /api/services │
                  └───────────────────────┘
```

---

## Authentication

**Long-Lived Access Tokens:**

1. User creates token in Home Assistant UI:
   - Profile → Long-Lived Access Tokens → Create Token
   - Copy token (only shown once)

2. User enters in Aura Settings:
   - Settings → Home Assistant → Access Token
   - Aura saves to OS keyring (never database/logs)

3. Token Usage:
   - **WebSocket:** Sent in `auth` message after connection
   - **REST:** Sent as `Authorization: Bearer <token>` header

**No OAuth2 Required** - Home Assistant uses simple bearer tokens for third-party apps.

---

## WebSocket API Client

### Connection Flow

```rust
// ha_client.rs

pub struct HomeAssistantClient {
    ws_sender: Option<SplitSink<WebSocketStream>>,
    ws_receiver: Option<SplitStream<WebSocketStream>>,
    rest_client: reqwest::Client,
    base_url: String,
    token: String,
    message_id: AtomicU64,
    subscriptions: Arc<Mutex<HashMap<u64, oneshot::Sender<Message>>>>,
}

impl HomeAssistantClient {
    pub async fn connect(base_url: String, token: String) -> Result<Self> {
        // 1. Connect to WebSocket
        let ws_url = format!("{}/api/websocket", base_url.replace("http", "ws"));
        let (ws_stream, _) = connect_async(&ws_url).await?;

        // 2. Wait for auth_required message
        let auth_required = receive_message(&mut ws_stream).await?;

        // 3. Send auth message with token
        send_message(&mut ws_stream, json!({
            "type": "auth",
            "access_token": token
        })).await?;

        // 4. Wait for auth_ok
        let auth_ok = receive_message(&mut ws_stream).await?;

        // 5. Subscribe to state_changed events
        send_message(&mut ws_stream, json!({
            "id": 1,
            "type": "subscribe_events",
            "event_type": "state_changed"
        })).await?;

        Ok(Self { /* ... */ })
    }
}
```

### Message Types

**Outgoing (Client → HA):**
- `auth` - Authentication with token
- `subscribe_events` - Subscribe to state changes
- `get_states` - Fetch all entity states
- `call_service` - Control devices

**Incoming (HA → Client):**
- `auth_required` - Server requests authentication
- `auth_ok` / `auth_invalid` - Authentication result
- `result` - Response to commands
- `event` - State change notifications

---

## Entity Management

### Entity Data Model

```rust
// entity_manager.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_id: String,        // e.g., "light.kitchen"
    pub state: String,             // e.g., "on", "off", "unavailable"
    pub attributes: EntityAttributes,
    pub last_changed: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAttributes {
    pub friendly_name: Option<String>,  // e.g., "Kitchen Light"
    pub area_id: Option<String>,         // e.g., "kitchen"
    pub device_class: Option<String>,    // e.g., "temperature", "motion"

    // Domain-specific attributes
    pub brightness: Option<u8>,          // 0-255 (lights)
    pub temperature: Option<f32>,        // Sensors
    pub hvac_mode: Option<String>,       // Climate
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

pub struct EntityManager {
    entities: Arc<RwLock<HashMap<String, Entity>>>,
    area_map: Arc<RwLock<HashMap<String, Vec<String>>>>, // area_id -> entity_ids
    domain_map: Arc<RwLock<HashMap<String, Vec<String>>>>, // domain -> entity_ids
}

impl EntityManager {
    pub async fn sync_entities(&self, client: &HomeAssistantClient) -> Result<()> {
        // Fetch all states via REST API
        let states = client.get_states().await?;

        // Update cache
        let mut entities = self.entities.write().await;
        for state in states {
            entities.insert(state.entity_id.clone(), state);
        }

        // Build indexes
        self.build_indexes().await;

        Ok(())
    }

    pub async fn handle_state_change(&self, event: StateChangedEvent) {
        let mut entities = self.entities.write().await;
        entities.insert(event.entity_id.clone(), event.new_state);
    }

    pub async fn query_entities(&self, filter: EntityFilter) -> Vec<Entity> {
        // Filter by domain, area, device_class, etc.
    }
}
```

### Supported Domains

| Domain | Description | Service Examples |
|--------|-------------|------------------|
| `light` | Lights, bulbs, LED strips | `turn_on`, `turn_off`, `toggle`, `brightness` |
| `switch` | Smart switches, plugs | `turn_on`, `turn_off`, `toggle` |
| `sensor` | Temperature, humidity, motion | *(read-only, no services)* |
| `binary_sensor` | Door/window, occupancy | *(read-only)* |
| `climate` | Thermostats, HVAC | `set_temperature`, `set_hvac_mode` |
| `cover` | Blinds, garage doors | `open_cover`, `close_cover`, `stop_cover` |
| `lock` | Smart locks | `lock`, `unlock` |
| `media_player` | TVs, speakers | `turn_on`, `turn_off`, `volume_up`, `play_media` |
| `fan` | Ceiling fans, ventilation | `turn_on`, `turn_off`, `set_speed` |
| `scene` | Predefined scenes | `turn_on` |
| `script` | Home Assistant scripts | `turn_on` |
| `automation` | Automations | `trigger`, `turn_on`, `turn_off` |

---

## REST API Client

### Service Calls

```rust
impl HomeAssistantClient {
    pub async fn call_service(
        &self,
        domain: &str,
        service: &str,
        entity_id: &str,
        data: Option<serde_json::Value>,
    ) -> Result<()> {
        let url = format!("{}/api/services/{}/{}", self.base_url, domain, service);

        let mut payload = json!({
            "entity_id": entity_id
        });

        if let Some(extra_data) = data {
            payload.as_object_mut().unwrap().extend(
                extra_data.as_object().unwrap().clone()
            );
        }

        let response = self.rest_client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HAError::ApiError(response.text().await?));
        }

        Ok(())
    }

    pub async fn get_states(&self) -> Result<Vec<Entity>> {
        let url = format!("{}/api/states", self.base_url);

        let response = self.rest_client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        let states: Vec<Entity> = response.json().await?;
        Ok(states)
    }
}
```

---

## Smart Home Intent Recognition

### Intent Types

```rust
// smarthome_intent.rs

#[derive(Debug, Clone, PartialEq)]
pub enum SmartHomeIntent {
    TurnOn { room: Option<String>, device_type: Option<String>, device_name: Option<String> },
    TurnOff { room: Option<String>, device_type: Option<String>, device_name: Option<String> },
    Toggle { room: Option<String>, device_type: Option<String>, device_name: Option<String> },
    SetBrightness { room: Option<String>, device_name: Option<String>, brightness: u8 },
    SetTemperature { room: Option<String>, temperature: f32 },
    GetState { room: Option<String>, device_type: Option<String>, device_name: Option<String> },
    Unknown,
}

impl SmartHomeIntentParser {
    pub fn parse(text: &str) -> SmartHomeIntent {
        let text_lower = text.to_lowercase();

        // Extract action
        let action = if text_lower.contains("turn on") || text_lower.contains("switch on") {
            Action::TurnOn
        } else if text_lower.contains("turn off") || text_lower.contains("switch off") {
            Action::TurnOff
        } else {
            return SmartHomeIntent::Unknown;
        };

        // Extract room
        let room = Self::extract_room(&text_lower);

        // Extract device type
        let device_type = Self::extract_device_type(&text_lower);

        // Extract device name
        let device_name = Self::extract_device_name(&text_lower);

        match action {
            Action::TurnOn => SmartHomeIntent::TurnOn { room, device_type, device_name },
            Action::TurnOff => SmartHomeIntent::TurnOff { room, device_type, device_name },
        }
    }
}
```

### Supported Commands

| User Command | Intent | Entities Matched |
|--------------|--------|------------------|
| "Turn on the kitchen lights" | TurnOn(room: kitchen, type: light) | `light.kitchen_*` |
| "Turn off bedroom lamp" | TurnOff(room: bedroom, name: lamp) | `light.bedroom_lamp` |
| "What's the temperature in the living room?" | GetState(room: living_room, type: sensor, class: temperature) | `sensor.living_room_temperature` |
| "Set living room to 72 degrees" | SetTemperature(room: living_room, temp: 72) | `climate.living_room_*` |
| "Dim the lights to 50%" | SetBrightness(brightness: 50%) | *(context-aware)* |

---

## Database Schema

### New Settings Fields

```rust
pub struct Settings {
    // ... existing fields ...

    // Home Assistant Integration
    pub ha_connected: bool,              // Connection status
    pub ha_base_url: String,             // e.g., "http://homeassistant.local:8123"
    pub ha_auto_sync: bool,              // Auto-sync entities on connect (default: true)
}
```

**Token Storage:** `ha_access_token` in OS keyring (NOT in database)

---

## Tauri Commands

### Authentication & Connection

```rust
#[tauri::command]
async fn ha_connect(
    base_url: String,
    token: String,
    db: State<'_, DatabaseState>,
) -> Result<(), AuraError> {
    // 1. Validate URL format
    // 2. Save token to keyring
    // 3. Connect WebSocket
    // 4. Fetch all entities
    // 5. Update database (ha_connected = true)
    // 6. Start background sync task
}

#[tauri::command]
async fn ha_disconnect(db: State<'_, DatabaseState>) -> Result<(), AuraError> {
    // 1. Close WebSocket connection
    // 2. Delete token from keyring
    // 3. Clear entity cache
    // 4. Update database (ha_connected = false)
}

#[tauri::command]
async fn ha_get_status() -> Result<HAStatusResponse, AuraError> {
    // Returns: { connected, base_url, entity_count, last_sync }
}
```

### Entity Management

```rust
#[tauri::command]
async fn ha_get_entities(
    filter: Option<EntityFilter>,
    entity_manager: State<'_, Arc<EntityManager>>,
) -> Result<Vec<Entity>, AuraError> {
    // Returns all entities or filtered by domain/area
}

#[tauri::command]
async fn ha_get_entity(
    entity_id: String,
    entity_manager: State<'_, Arc<EntityManager>>,
) -> Result<Entity, AuraError> {
    // Returns specific entity with current state
}
```

### Device Control

```rust
#[tauri::command]
async fn ha_call_service(
    domain: String,
    service: String,
    entity_id: String,
    data: Option<serde_json::Value>,
    ha_client: State<'_, Arc<TokioMutex<HomeAssistantClient>>>,
) -> Result<String, AuraError> {
    // Calls Home Assistant service
    // Returns confirmation message
}

#[tauri::command]
async fn ha_handle_smart_home_command(
    command: String,
    entity_manager: State<'_, Arc<EntityManager>>,
    ha_client: State<'_, Arc<TokioMutex<HomeAssistantClient>>>,
) -> Result<String, AuraError> {
    // Main NLU entry point
    // 1. Parse intent
    // 2. Match entities
    // 3. Call service
    // 4. Return friendly response
}
```

---

## Frontend UI

### Settings Modal - Home Assistant Section

**Disconnected State:**
- Home Assistant URL input (e.g., `http://homeassistant.local:8123`)
- Access Token input (password field)
- "Connect Home Assistant" button
- Instructions with link to token creation guide

**Connected State:**
- Green indicator "Connected to Home Assistant"
- Server URL display
- Entity count (e.g., "142 entities discovered")
- Last sync timestamp
- "Disconnect" button
- "Refresh Entities" button

### Devices View (New Primary View)

**Navigation:** Add new "Devices" tab/button in main UI

**Layout:**
```
┌─────────────────────────────────────────────────────────┐
│  Devices                                         [Refresh]│
├─────────────────────────────────────────────────────────┤
│  Search: [__________________]  Filter: [All ▼]          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Kitchen (5 entities)                                   │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 💡 Kitchen Lights            [🟢 ON]   [Toggle] │  │
│  │ 🔌 Kitchen Outlet            [🔴 OFF]  [Toggle] │  │
│  │ 🌡️  Kitchen Temperature       72°F              │  │
│  │ 💧 Kitchen Humidity           45%                │  │
│  │ 🚪 Kitchen Motion            [Detected]          │  │
│  └──────────────────────────────────────────────────┘  │
│                                                          │
│  Living Room (8 entities)                               │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 💡 Living Room Lights        [🟢 ON]   [Slider] │  │
│  │    Brightness: ████████░░░░░ 60%                 │  │
│  │ 📺 TV                        [🔴 OFF]  [Toggle] │  │
│  │ ...                                               │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

**Features:**
- Real-time state updates (via WebSocket)
- Grouped by room/area
- Quick toggle controls
- Brightness sliders for lights
- Temperature controls for climate
- Click to expand for details

---

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum HAError {
    #[error("Not connected to Home Assistant")]
    NotConnected,

    #[error("WebSocket connection failed: {0}")]
    WebSocketError(String),

    #[error("Authentication failed: Invalid access token")]
    AuthenticationFailed,

    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Service call failed: {0}")]
    ServiceCallFailed(String),

    #[error("Home Assistant unreachable: {0}")]
    Unreachable(String),

    #[error("API error: {0}")]
    ApiError(String),
}
```

**Graceful Degradation:**
- WebSocket disconnection: Attempt reconnect (exponential backoff)
- Service call failure: Clear error message to user
- Entity not found: Suggest alternatives based on fuzzy matching
- Home Assistant offline: Show connection status in UI

---

## Dependencies

### Cargo.toml Additions

```toml
# Home Assistant Integration
tokio-tungstenite = "0.21"  # WebSocket client (tokio-based)
tungstenite = "0.21"         # WebSocket protocol
futures-util = "0.3"         # Stream utilities
async-trait = "0.1"          # Async trait definitions
```

**Notes:**
- `tokio-tungstenite` provides async WebSocket support compatible with Tauri's Tokio runtime
- `futures-util` for WebSocket stream manipulation (SplitSink/SplitStream)
- Existing dependencies used: `reqwest` (REST), `serde_json` (messages), `tokio` (async)

---

## Testing Strategy

### Unit Tests

**Intent Parser Tests** (`smarthome_intent.rs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_turn_on_kitchen_lights() {
        let intent = SmartHomeIntentParser::parse("Turn on the kitchen lights");
        assert_eq!(
            intent,
            SmartHomeIntent::TurnOn {
                room: Some("kitchen".to_string()),
                device_type: Some("light".to_string()),
                device_name: None,
            }
        );
    }

    #[test]
    fn test_parse_set_temperature() {
        let intent = SmartHomeIntentParser::parse("Set living room to 72 degrees");
        assert_eq!(
            intent,
            SmartHomeIntent::SetTemperature {
                room: Some("living_room".to_string()),
                temperature: 72.0,
            }
        );
    }

    // 25+ additional test cases covering all intent types
}
```

**Entity Manager Tests** (`entity_manager.rs`):
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_entity_filtering_by_domain() {
        let manager = EntityManager::new();
        // Add test entities
        let lights = manager.query_entities(EntityFilter {
            domain: Some("light"),
            ..Default::default()
        }).await;
        assert_eq!(lights.len(), 5);
    }

    #[tokio::test]
    async fn test_entity_filtering_by_area() {
        // Test room-based filtering
    }
}
```

### Integration Tests

**WebSocket Authentication** (`tests/ha_websocket_test.rs`):
```rust
#[tokio::test]
#[ignore] // Requires running Home Assistant instance
async fn test_websocket_auth_flow() {
    let client = HomeAssistantClient::connect(
        "http://homeassistant.local:8123".to_string(),
        "test_token".to_string(),
    ).await;

    assert!(client.is_ok());
}
```

**REST API Calls** (`tests/ha_rest_test.rs`):
```rust
#[tokio::test]
#[ignore]
async fn test_call_light_service() {
    let client = setup_test_client().await;
    let result = client.call_service(
        "light",
        "turn_on",
        "light.test_light",
        Some(json!({ "brightness": 128 })),
    ).await;

    assert!(result.is_ok());
}
```

### Manual Test Cases

**Test Case 1: Initial Connection**
1. User enters HA URL + access token in Settings
2. Click "Connect Home Assistant"
3. **Expected:** Green "Connected" indicator, entity count displayed
4. **Verify:** WebSocket authenticated, entities fetched

**Test Case 2: Voice Command - Simple On/Off**
1. Say "Hey Aura, turn on the kitchen lights"
2. **Expected:** Light turns on, Aura responds "Kitchen lights are now on"
3. **Verify:** Service call successful, entity state updated in UI

**Test Case 3: Voice Command - Brightness**
1. Say "Hey Aura, set bedroom lights to 50%"
2. **Expected:** Brightness set to ~128 (50% of 255)
3. **Verify:** Brightness attribute updated in entity

**Test Case 4: Real-Time State Sync**
1. Manually change light state in Home Assistant UI
2. **Expected:** Aura's Devices view updates within 1 second
3. **Verify:** WebSocket event received and processed

**Test Case 5: Connection Loss Recovery**
1. Stop Home Assistant server
2. Attempt voice command
3. **Expected:** Clear error "Home Assistant unreachable"
4. Restart Home Assistant
5. **Expected:** Auto-reconnect within 30 seconds

**Test Case 6: Multi-Entity Command**
1. Say "Turn on all living room lights"
2. **Expected:** All `light.living_room_*` entities turn on
3. **Verify:** Multiple service calls sent

**Test Case 7: Entity Not Found**
1. Say "Turn on the basement lights" (no basement configured)
2. **Expected:** "I couldn't find any lights in the basement"
3. **Verify:** Fuzzy matching attempted, clear error message

---

## Security Considerations

### Token Security

✅ **Access Token Storage:**
- **NEVER** store tokens in database or logs
- **ALWAYS** use OS keyring (Keychain/Credential Manager/Secret Service)
- Delete token on disconnect
- Clear from memory after use

✅ **Token Transmission:**
- WebSocket: TLS/WSS strongly recommended for production
- Allow HTTP for local development (homeassistant.local)
- Validate SSL certificates by default

### Network Security

✅ **URL Validation:**
```rust
fn validate_ha_url(url: &str) -> Result<(), String> {
    // Must be valid HTTP/HTTPS URL
    // Reject localhost/127.0.0.1 if running in production
    // Allow .local mDNS addresses
}
```

✅ **CORS & WebSocket Security:**
- Home Assistant validates origin headers
- Aura sends appropriate origin
- WebSocket authentication required within 10 seconds

### User Privacy

✅ **Local-Only Communication:**
- All communication directly to user's Home Assistant instance
- No cloud intermediaries
- No telemetry sent to third parties

✅ **Command Processing:**
- Voice recognition happens locally (Whisper)
- Intent parsing happens locally (Rust)
- Only final service calls sent to Home Assistant

---

## Performance & Scalability

### Optimization Strategies

**Entity Caching:**
- Full sync on connect (~100-500ms for 100 entities)
- Incremental updates via WebSocket (real-time)
- Index by domain and area for O(1) lookups

**WebSocket Connection:**
- Single persistent connection (low overhead)
- Automatic ping/pong keepalive (30-second interval)
- Reconnect with exponential backoff (1s, 2s, 4s, 8s, max 60s)

**Service Call Latency:**
- REST API calls: 50-200ms typical
- WebSocket state updates: 10-50ms typical
- Total voice-to-action latency: <2 seconds (including STT + LLM)

**Memory Usage:**
- Entity cache: ~10KB per 100 entities (JSON in memory)
- WebSocket buffer: ~16KB
- Total overhead: <1MB for typical home

### Scalability Limits

| Home Size | Entities | Performance | Notes |
|-----------|----------|-------------|-------|
| Small | 1-50 | Excellent | <100ms query times |
| Medium | 50-200 | Excellent | <200ms query times |
| Large | 200-500 | Good | <500ms query times |
| Very Large | 500+ | Acceptable | May need pagination in UI |

---

## Future Enhancements

### Phase 2 (Post-MVP)

**1. Advanced Intent Recognition:**
- Multi-entity commands: "Turn off all lights except the kitchen"
- Conditional commands: "If bedroom motion detected, turn on lights"
- Scheduled commands: "Turn on porch light at sunset"

**2. Scenes & Automations:**
- Voice activation of scenes: "Activate movie mode"
- Trigger automations: "Run bedtime routine"
- Create scenes via voice: "Save current lights as 'dinner scene'"

**3. Contextual Awareness:**
- Remember last room mentioned in conversation
- Device preferences (default bedroom light)
- Time-based context (morning vs. evening routines)

**4. Advanced UI:**
- Entity history graphs (temperature over time)
- Automation builder interface
- Custom dashboards with drag-and-drop

**5. Notifications:**
- Aura announces state changes: "Front door opened"
- Critical alerts: "Smoke detected in kitchen"
- Configurable notification filters

**6. Multi-Zone Audio (Piper TTS):**
- Play TTS responses on Home Assistant media players
- Zone-specific announcements

---

## Implementation Checklist

### Backend (Rust)

- [ ] `src-tauri/src/ha_client.rs` - WebSocket + REST client (500 lines)
- [ ] `src-tauri/src/entity_manager.rs` - Entity cache and indexing (400 lines)
- [ ] `src-tauri/src/smarthome_intent.rs` - Intent parser (350 lines)
- [ ] `src-tauri/src/secrets.rs` - Extend with HA token functions (50 lines)
- [ ] `src-tauri/src/database.rs` - Extend Settings struct (30 lines)
- [ ] `src-tauri/src/error.rs` - Add HAError enum (40 lines)
- [ ] `src-tauri/src/lib.rs` - Add 8 Tauri commands (300 lines)
- [ ] `src-tauri/Cargo.toml` - Add WebSocket dependencies (5 lines)

**Estimated Backend:** 1,675 lines of production code

### Frontend (TypeScript/React)

- [ ] `src/components/HomeAssistantSettings.tsx` - Settings UI (200 lines)
- [ ] `src/components/DevicesView.tsx` - Main devices UI (350 lines)
- [ ] `src/components/EntityCard.tsx` - Reusable entity display (150 lines)
- [ ] `src/store.ts` - Extend settings interface (20 lines)
- [ ] `src/App.tsx` - Add Devices route (10 lines)

**Estimated Frontend:** 730 lines of code

### Documentation

- [x] `Documentation/HOMEASSISTANT_ARCHITECTURE.md` - Technical design (this file)
- [ ] `Documentation/HOMEASSISTANT_USER_GUIDE.md` - User setup guide (similar to Spotify guide)
- [ ] `Documentation/HOMEASSISTANT_TESTING_GUIDE.md` - Test procedures (42+ test cases)
- [ ] `Documentation/HOMEASSISTANT_ACCEPTANCE_CRITERIA.md` - AC tracking

**Estimated Documentation:** 2,500+ lines

---

## Summary

This architecture provides:

✅ **Secure** - Long-Lived Access Tokens in OS keyring, TLS support
✅ **Real-Time** - WebSocket for instant state updates
✅ **Comprehensive** - Supports all major Home Assistant domains
✅ **Natural** - Intelligent voice command recognition
✅ **Performant** - Entity caching, indexed queries, <2s voice-to-action
✅ **Privacy-First** - Local communication only, no cloud dependencies
✅ **Scalable** - Handles 500+ entities efficiently

**Total Implementation Estimate:**
- Backend: 1,675 lines
- Frontend: 730 lines
- Documentation: 2,500 lines
- Tests: 30+ unit tests, 10+ integration tests, 42+ manual test cases
- **Total:** ~4,900 lines + comprehensive testing

**Development Timeline Estimate:**
- Backend implementation: 3-4 days
- Frontend implementation: 1-2 days
- Testing & refinement: 1-2 days
- Documentation: 1 day
- **Total:** 6-9 days for full-featured implementation

---

**Document Version:** 1.0
**Last Updated:** 2025-10-10
**Author:** Claude (AI Assistant)
**Status:** ✅ Complete - Ready for Implementation