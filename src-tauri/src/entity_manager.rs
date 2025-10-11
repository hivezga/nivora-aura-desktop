/// Entity Manager for Home Assistant Integration
///
/// Provides entity caching, state management, and indexing for efficient queries.
/// Entities are synchronized via WebSocket state_changed events and periodic full syncs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Represents a Home Assistant entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_id: String,              // e.g., "light.kitchen"
    pub state: String,                   // e.g., "on", "off", "unavailable"
    pub attributes: EntityAttributes,
    pub last_changed: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Entity attributes (domain-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAttributes {
    pub friendly_name: Option<String>,   // e.g., "Kitchen Light"
    pub area_id: Option<String>,         // e.g., "kitchen" (lowercase, normalized)
    pub device_class: Option<String>,    // e.g., "temperature", "motion"

    // Domain-specific attributes (lights)
    pub brightness: Option<u8>,          // 0-255
    pub rgb_color: Option<Vec<u8>>,      // [R, G, B]
    pub color_temp: Option<u16>,         // Mireds

    // Domain-specific attributes (sensors)
    pub unit_of_measurement: Option<String>, // e.g., "°F", "%"
    pub temperature: Option<f32>,        // Temperature value
    pub humidity: Option<f32>,           // Humidity value

    // Domain-specific attributes (climate)
    pub hvac_mode: Option<String>,       // e.g., "heat", "cool", "auto"
    pub current_temperature: Option<f32>,
    pub target_temperature: Option<f32>,

    // Domain-specific attributes (covers)
    pub current_position: Option<u8>,    // 0-100 (blinds, garage doors)

    // Generic attributes (catch-all for other domains)
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// State changed event from Home Assistant WebSocket
#[derive(Debug, Clone, Deserialize)]
pub struct StateChangedEvent {
    pub entity_id: String,
    pub new_state: Entity,
}

/// Filter for querying entities
#[derive(Debug, Clone, Default)]
pub struct EntityFilter {
    pub domain: Option<String>,           // e.g., "light", "sensor"
    pub area: Option<String>,             // e.g., "kitchen", "bedroom"
    pub device_class: Option<String>,     // e.g., "temperature", "motion"
    pub state: Option<String>,            // e.g., "on", "off"
}

/// Entity Manager - manages entity cache and indexing
pub struct EntityManager {
    // Main entity cache (entity_id -> Entity)
    entities: Arc<RwLock<HashMap<String, Entity>>>,

    // Indexes for fast lookups
    area_index: Arc<RwLock<HashMap<String, Vec<String>>>>,     // area_id -> entity_ids
    domain_index: Arc<RwLock<HashMap<String, Vec<String>>>>,   // domain -> entity_ids
}

impl EntityManager {
    /// Create a new EntityManager
    pub fn new() -> Self {
        Self {
            entities: Arc::new(RwLock::new(HashMap::new())),
            area_index: Arc::new(RwLock::new(HashMap::new())),
            domain_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Synchronize all entities (full sync)
    ///
    /// This is called on connection and periodically to ensure cache is up-to-date.
    /// Entities are provided by the ha_client after fetching from REST API.
    pub async fn sync_entities(&self, entities: Vec<Entity>) -> Result<(), String> {
        log::info!("Syncing {} entities from Home Assistant", entities.len());

        let mut cache = self.entities.write().await;
        cache.clear();

        for entity in entities {
            cache.insert(entity.entity_id.clone(), entity);
        }

        drop(cache); // Release write lock before rebuilding indexes

        // Rebuild indexes
        self.build_indexes().await;

        log::info!("Entity sync complete");
        Ok(())
    }

    /// Handle a state_changed event from WebSocket
    ///
    /// Updates a single entity in the cache and updates indexes if needed.
    pub async fn handle_state_change(&self, event: StateChangedEvent) {
        log::debug!("State changed: {} -> {}", event.entity_id, event.new_state.state);

        let entity_id = event.entity_id.clone();
        let new_entity = event.new_state;

        // Update entity in cache
        let mut cache = self.entities.write().await;
        cache.insert(entity_id.clone(), new_entity.clone());
        drop(cache);

        // Update indexes if area changed
        self.update_entity_in_indexes(&entity_id, &new_entity).await;
    }

    /// Get a specific entity by ID
    pub async fn get_entity(&self, entity_id: &str) -> Option<Entity> {
        let cache = self.entities.read().await;
        cache.get(entity_id).cloned()
    }

    /// Query entities with optional filters
    pub async fn query_entities(&self, filter: EntityFilter) -> Vec<Entity> {
        let cache = self.entities.read().await;

        // Start with all entities or filter by domain/area using indexes
        let candidate_ids = if let Some(domain) = &filter.domain {
            let domain_index = self.domain_index.read().await;
            domain_index.get(domain).cloned().unwrap_or_default()
        } else if let Some(area) = &filter.area {
            let area_index = self.area_index.read().await;
            area_index.get(area).cloned().unwrap_or_default()
        } else {
            cache.keys().cloned().collect()
        };

        // Apply additional filters
        candidate_ids
            .into_iter()
            .filter_map(|id| cache.get(&id))
            .filter(|entity| {
                // Filter by domain (if not already filtered by index)
                if let Some(ref domain) = filter.domain {
                    let entity_domain = entity.entity_id.split('.').next().unwrap_or("");
                    if entity_domain != domain {
                        return false;
                    }
                }

                // Filter by area (if not already filtered by index)
                if let Some(ref area) = filter.area {
                    match &entity.attributes.area_id {
                        Some(entity_area) if entity_area == area => {},
                        _ => return false,
                    }
                }

                // Filter by device_class
                if let Some(ref device_class) = filter.device_class {
                    match &entity.attributes.device_class {
                        Some(entity_class) if entity_class == device_class => {},
                        _ => return false,
                    }
                }

                // Filter by state
                if let Some(ref state) = filter.state {
                    if &entity.state != state {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    /// Get all entities (no filter)
    pub async fn get_all_entities(&self) -> Vec<Entity> {
        let cache = self.entities.read().await;
        cache.values().cloned().collect()
    }

    /// Get entity count
    pub async fn get_entity_count(&self) -> usize {
        let cache = self.entities.read().await;
        cache.len()
    }

    /// Clear all entities and indexes
    pub async fn clear(&self) {
        let mut cache = self.entities.write().await;
        cache.clear();
        drop(cache);

        let mut area_index = self.area_index.write().await;
        area_index.clear();
        drop(area_index);

        let mut domain_index = self.domain_index.write().await;
        domain_index.clear();

        log::info!("Entity cache cleared");
    }

    // =========================================================================
    // Private helper methods
    // =========================================================================

    /// Rebuild all indexes from current entity cache
    async fn build_indexes(&self) {
        let cache = self.entities.read().await;

        let mut new_area_index: HashMap<String, Vec<String>> = HashMap::new();
        let mut new_domain_index: HashMap<String, Vec<String>> = HashMap::new();

        for (entity_id, entity) in cache.iter() {
            // Index by domain (e.g., "light", "sensor")
            if let Some(domain) = entity.entity_id.split('.').next() {
                new_domain_index
                    .entry(domain.to_string())
                    .or_insert_with(Vec::new)
                    .push(entity_id.clone());
            }

            // Index by area (if available)
            if let Some(area_id) = &entity.attributes.area_id {
                new_area_index
                    .entry(area_id.clone())
                    .or_insert_with(Vec::new)
                    .push(entity_id.clone());
            }
        }

        drop(cache);

        // Replace indexes
        let mut area_index = self.area_index.write().await;
        *area_index = new_area_index;
        drop(area_index);

        let mut domain_index = self.domain_index.write().await;
        *domain_index = new_domain_index;

        log::debug!("Entity indexes rebuilt");
    }

    /// Update a single entity in indexes (called on state change)
    async fn update_entity_in_indexes(&self, entity_id: &str, entity: &Entity) {
        // Update domain index
        if let Some(domain) = entity.entity_id.split('.').next() {
            let mut domain_index = self.domain_index.write().await;
            let entry = domain_index
                .entry(domain.to_string())
                .or_insert_with(Vec::new);

            if !entry.contains(&entity_id.to_string()) {
                entry.push(entity_id.to_string());
            }
        }

        // Update area index
        if let Some(area_id) = &entity.attributes.area_id {
            let mut area_index = self.area_index.write().await;
            let entry = area_index
                .entry(area_id.clone())
                .or_insert_with(Vec::new);

            if !entry.contains(&entity_id.to_string()) {
                entry.push(entity_id.to_string());
            }
        }
    }
}

/// Helper function to extract domain from entity_id
pub fn extract_domain(entity_id: &str) -> Option<String> {
    entity_id.split('.').next().map(|s| s.to_string())
}

/// Helper function to extract entity name from entity_id
pub fn extract_name(entity_id: &str) -> Option<String> {
    entity_id.split('.').nth(1).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entity(entity_id: &str, state: &str, area: Option<&str>) -> Entity {
        Entity {
            entity_id: entity_id.to_string(),
            state: state.to_string(),
            attributes: EntityAttributes {
                friendly_name: Some(format!("Test {}", entity_id)),
                area_id: area.map(|s| s.to_string()),
                device_class: None,
                brightness: None,
                rgb_color: None,
                color_temp: None,
                unit_of_measurement: None,
                temperature: None,
                humidity: None,
                hvac_mode: None,
                current_temperature: None,
                target_temperature: None,
                current_position: None,
                extra: HashMap::new(),
            },
            last_changed: Utc::now(),
            last_updated: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_sync_entities() {
        let manager = EntityManager::new();

        let entities = vec![
            create_test_entity("light.kitchen", "on", Some("kitchen")),
            create_test_entity("light.bedroom", "off", Some("bedroom")),
            create_test_entity("sensor.kitchen_temperature", "72", Some("kitchen")),
        ];

        manager.sync_entities(entities).await.unwrap();

        assert_eq!(manager.get_entity_count().await, 3);
    }

    #[tokio::test]
    async fn test_query_by_domain() {
        let manager = EntityManager::new();

        let entities = vec![
            create_test_entity("light.kitchen", "on", Some("kitchen")),
            create_test_entity("light.bedroom", "off", Some("bedroom")),
            create_test_entity("sensor.kitchen_temperature", "72", Some("kitchen")),
        ];

        manager.sync_entities(entities).await.unwrap();

        let lights = manager.query_entities(EntityFilter {
            domain: Some("light".to_string()),
            ..Default::default()
        }).await;

        assert_eq!(lights.len(), 2);
    }

    #[tokio::test]
    async fn test_query_by_area() {
        let manager = EntityManager::new();

        let entities = vec![
            create_test_entity("light.kitchen", "on", Some("kitchen")),
            create_test_entity("light.bedroom", "off", Some("bedroom")),
            create_test_entity("sensor.kitchen_temperature", "72", Some("kitchen")),
        ];

        manager.sync_entities(entities).await.unwrap();

        let kitchen_entities = manager.query_entities(EntityFilter {
            area: Some("kitchen".to_string()),
            ..Default::default()
        }).await;

        assert_eq!(kitchen_entities.len(), 2);
    }

    #[tokio::test]
    async fn test_state_change() {
        let manager = EntityManager::new();

        let entities = vec![
            create_test_entity("light.kitchen", "off", Some("kitchen")),
        ];

        manager.sync_entities(entities).await.unwrap();

        // Simulate state change
        let event = StateChangedEvent {
            entity_id: "light.kitchen".to_string(),
            new_state: create_test_entity("light.kitchen", "on", Some("kitchen")),
        };

        manager.handle_state_change(event).await;

        let entity = manager.get_entity("light.kitchen").await.unwrap();
        assert_eq!(entity.state, "on");
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("light.kitchen"), Some("light".to_string()));
        assert_eq!(extract_domain("sensor.temperature"), Some("sensor".to_string()));
        assert_eq!(extract_domain("invalid"), None);
    }

    #[test]
    fn test_extract_name() {
        assert_eq!(extract_name("light.kitchen"), Some("kitchen".to_string()));
        assert_eq!(extract_name("sensor.temperature"), Some("temperature".to_string()));
        assert_eq!(extract_name("invalid"), None);
    }
}
