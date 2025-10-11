/// Home Assistant Client - WebSocket + REST API
///
/// Provides real-time communication with Home Assistant via WebSocket for state updates
/// and REST API for service calls and entity queries.

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use crate::entity_manager::{Entity, EntityManager, StateChangedEvent};

/// Home Assistant client with WebSocket and REST API support
pub struct HomeAssistantClient {
    base_url: String,
    token: String,
    rest_client: reqwest::Client,
    ws_connected: Arc<Mutex<bool>>,
    entity_manager: Arc<EntityManager>,
    message_id_counter: Arc<Mutex<u64>>,
}

/// WebSocket message types from Home Assistant
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum HAMessage {
    #[serde(rename = "auth_required")]
    AuthRequired { ha_version: String },

    #[serde(rename = "auth_ok")]
    AuthOk { ha_version: String },

    #[serde(rename = "auth_invalid")]
    AuthInvalid { message: String },

    #[serde(rename = "result")]
    Result {
        id: u64,
        success: bool,
        result: Option<serde_json::Value>,
        error: Option<HAError>,
    },

    #[serde(rename = "event")]
    Event {
        id: u64,
        event: EventData,
    },
}

#[derive(Debug, Deserialize)]
struct HAError {
    code: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct EventData {
    event_type: String,
    data: serde_json::Value,
}

impl HomeAssistantClient {
    /// Create a new Home Assistant client
    pub fn new(base_url: String, token: String, entity_manager: Arc<EntityManager>) -> Self {
        Self {
            base_url,
            token,
            rest_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            ws_connected: Arc::new(Mutex::new(false)),
            entity_manager,
            message_id_counter: Arc::new(Mutex::new(1)),
        }
    }

    /// Connect to Home Assistant WebSocket and start event loop
    ///
    /// This method:
    /// 1. Connects to WebSocket
    /// 2. Authenticates with access token
    /// 3. Subscribes to state_changed events
    /// 4. Starts event loop to process incoming messages
    /// 5. Performs initial entity sync via REST API
    pub async fn connect(&self) -> Result<(), String> {
        log::info!("Connecting to Home Assistant at {}", self.base_url);

        // Convert http:// to ws:// or https:// to wss://
        let ws_url = self.base_url.replace("http://", "ws://").replace("https://", "wss://");
        let ws_url = format!("{}/api/websocket", ws_url);

        log::debug!("WebSocket URL: {}", ws_url);

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| format!("WebSocket connection failed: {}", e))?;

        log::info!("WebSocket connected, authenticating...");

        let (mut write, mut read) = ws_stream.split();

        // Step 1: Wait for auth_required message
        if let Some(msg) = read.next().await {
            let msg = msg.map_err(|e| format!("Failed to receive auth_required: {}", e))?;
            let text = msg.to_text().map_err(|e| format!("Invalid message format: {}", e))?;

            log::debug!("Received: {}", text);

            let parsed: HAMessage = serde_json::from_str(text)
                .map_err(|e| format!("Failed to parse auth_required: {}", e))?;

            match parsed {
                HAMessage::AuthRequired { ha_version } => {
                    log::info!("Home Assistant version: {}", ha_version);
                }
                _ => return Err("Expected auth_required message".to_string()),
            }
        } else {
            return Err("Connection closed before auth_required".to_string());
        }

        // Step 2: Send auth message
        let auth_msg = json!({
            "type": "auth",
            "access_token": self.token
        });

        write
            .send(Message::Text(auth_msg.to_string()))
            .await
            .map_err(|e| format!("Failed to send auth message: {}", e))?;

        log::debug!("Sent auth message");

        // Step 3: Wait for auth_ok or auth_invalid
        if let Some(msg) = read.next().await {
            let msg = msg.map_err(|e| format!("Failed to receive auth response: {}", e))?;
            let text = msg.to_text().map_err(|e| format!("Invalid message format: {}", e))?;

            log::debug!("Received: {}", text);

            let parsed: HAMessage = serde_json::from_str(text)
                .map_err(|e| format!("Failed to parse auth response: {}", e))?;

            match parsed {
                HAMessage::AuthOk { ha_version } => {
                    log::info!("✓ Authenticated to Home Assistant {}", ha_version);
                }
                HAMessage::AuthInvalid { message } => {
                    return Err(format!("Authentication failed: {}", message));
                }
                _ => return Err("Expected auth_ok or auth_invalid message".to_string()),
            }
        } else {
            return Err("Connection closed before auth response".to_string());
        }

        // Step 4: Subscribe to state_changed events
        let subscribe_msg = json!({
            "id": 1,
            "type": "subscribe_events",
            "event_type": "state_changed"
        });

        write
            .send(Message::Text(subscribe_msg.to_string()))
            .await
            .map_err(|e| format!("Failed to subscribe to events: {}", e))?;

        log::debug!("Subscribed to state_changed events");

        // Wait for subscription confirmation
        if let Some(msg) = read.next().await {
            let msg = msg.map_err(|e| format!("Failed to receive subscription response: {}", e))?;
            let text = msg.to_text().map_err(|e| format!("Invalid message format: {}", e))?;

            log::debug!("Received: {}", text);

            let parsed: HAMessage = serde_json::from_str(text)
                .map_err(|e| format!("Failed to parse subscription response: {}", e))?;

            match parsed {
                HAMessage::Result { success: true, .. } => {
                    log::info!("✓ Subscribed to state_changed events");
                }
                HAMessage::Result { success: false, error, .. } => {
                    return Err(format!("Subscription failed: {:?}", error));
                }
                _ => return Err("Expected result message for subscription".to_string()),
            }
        }

        // Mark as connected
        *self.ws_connected.lock().await = true;

        // Step 5: Perform initial entity sync via REST API
        log::info!("Performing initial entity sync...");
        self.sync_all_entities().await?;

        // Step 6: Start event loop (spawn background task)
        let entity_manager = self.entity_manager.clone();
        let ws_connected = self.ws_connected.clone();

        tokio::spawn(async move {
            log::info!("Starting WebSocket event loop...");

            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(msg) => {
                        if let Ok(text) = msg.to_text() {
                            log::debug!("WebSocket received: {}", text);

                            // Parse and handle message
                            if let Ok(parsed) = serde_json::from_str::<HAMessage>(text) {
                                match parsed {
                                    HAMessage::Event { event, .. } => {
                                        if event.event_type == "state_changed" {
                                            // Parse state_changed event
                                            if let Ok(state_event) = serde_json::from_value::<StateChangedEvent>(event.data.clone()) {
                                                entity_manager.handle_state_change(state_event).await;
                                            } else {
                                                log::warn!("Failed to parse state_changed event");
                                            }
                                        }
                                    }
                                    HAMessage::Result { .. } => {
                                        // Ignore result messages (handled synchronously)
                                    }
                                    _ => {
                                        log::debug!("Received unexpected message type");
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("WebSocket error: {}", e);
                        *ws_connected.lock().await = false;
                        break;
                    }
                }
            }

            log::warn!("WebSocket connection closed");
            *ws_connected.lock().await = false;
        });

        log::info!("✓ Home Assistant client connected and synced");

        Ok(())
    }

    /// Disconnect from Home Assistant
    pub async fn disconnect(&self) {
        log::info!("Disconnecting from Home Assistant");
        *self.ws_connected.lock().await = false;
        self.entity_manager.clear().await;
    }

    /// Check if WebSocket is connected
    pub async fn is_connected(&self) -> bool {
        *self.ws_connected.lock().await
    }

    // =========================================================================
    // REST API Methods
    // =========================================================================

    /// Get all entity states from Home Assistant via REST API
    pub async fn get_states(&self) -> Result<Vec<Entity>, String> {
        let url = format!("{}/api/states", self.base_url);

        log::debug!("GET {}", url);

        let response = self
            .rest_client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("HTTP {} - {}", status, body));
        }

        let states: Vec<Entity> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse states response: {}", e))?;

        log::debug!("Fetched {} entities", states.len());

        Ok(states)
    }

    /// Call a Home Assistant service
    ///
    /// # Arguments
    /// * `domain` - Service domain (e.g., "light", "switch")
    /// * `service` - Service name (e.g., "turn_on", "turn_off")
    /// * `entity_id` - Target entity ID (e.g., "light.kitchen")
    /// * `data` - Optional service data (e.g., brightness, temperature)
    pub async fn call_service(
        &self,
        domain: &str,
        service: &str,
        entity_id: &str,
        data: Option<serde_json::Value>,
    ) -> Result<(), String> {
        let url = format!("{}/api/services/{}/{}", self.base_url, domain, service);

        log::info!("Calling service: {}.{} on {}", domain, service, entity_id);

        let mut payload = json!({
            "entity_id": entity_id
        });

        // Merge additional data if provided
        if let Some(extra_data) = data {
            if let Some(payload_obj) = payload.as_object_mut() {
                if let Some(extra_obj) = extra_data.as_object() {
                    for (key, value) in extra_obj {
                        payload_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        log::debug!("Service call payload: {}", payload);

        let response = self
            .rest_client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Service call failed: HTTP {} - {}", status, body));
        }

        log::info!("✓ Service call successful");

        Ok(())
    }

    /// Sync all entities from Home Assistant to EntityManager
    async fn sync_all_entities(&self) -> Result<(), String> {
        let states = self.get_states().await?;
        self.entity_manager.sync_entities(states).await?;
        log::info!("✓ Synced {} entities", self.entity_manager.get_entity_count().await);
        Ok(())
    }

    /// Get the next message ID for WebSocket requests
    #[allow(dead_code)]
    async fn next_message_id(&self) -> u64 {
        let mut counter = self.message_id_counter.lock().await;
        let id = *counter;
        *counter += 1;
        id
    }
}

/// Helper struct for service call responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceCallResponse {
    pub success: bool,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_url_conversion() {
        let base_url = "http://homeassistant.local:8123";
        let ws_url = base_url.replace("http://", "ws://");
        assert_eq!(ws_url, "ws://homeassistant.local:8123");

        let base_url_https = "https://homeassistant.local:8123";
        let ws_url_secure = base_url_https.replace("https://", "wss://");
        assert_eq!(ws_url_secure, "wss://homeassistant.local:8123");
    }

    #[tokio::test]
    async fn test_client_creation() {
        let entity_manager = Arc::new(EntityManager::new());
        let client = HomeAssistantClient::new(
            "http://homeassistant.local:8123".to_string(),
            "test_token".to_string(),
            entity_manager,
        );

        assert_eq!(client.base_url, "http://homeassistant.local:8123");
        assert_eq!(client.token, "test_token");
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_message_id_counter() {
        let entity_manager = Arc::new(EntityManager::new());
        let client = HomeAssistantClient::new(
            "http://homeassistant.local:8123".to_string(),
            "test_token".to_string(),
            entity_manager,
        );

        let id1 = client.next_message_id().await;
        let id2 = client.next_message_id().await;
        let id3 = client.next_message_id().await;

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }
}
