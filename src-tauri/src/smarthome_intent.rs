/// Smart Home Intent Recognition
///
/// Parses natural language commands into structured intents for Home Assistant control.
/// Extracts key information: action, room, device type, device name, and parameters.

use once_cell::sync::Lazy;
use regex::Regex;

/// Smart home intent types
#[derive(Debug, Clone, PartialEq)]
pub enum SmartHomeIntent {
    /// Turn on a device
    TurnOn {
        room: Option<String>,
        device_type: Option<String>,
        device_name: Option<String>,
    },

    /// Turn off a device
    TurnOff {
        room: Option<String>,
        device_type: Option<String>,
        device_name: Option<String>,
    },

    /// Toggle a device state
    Toggle {
        room: Option<String>,
        device_type: Option<String>,
        device_name: Option<String>,
    },

    /// Set brightness for lights (0-100%)
    SetBrightness {
        room: Option<String>,
        device_name: Option<String>,
        brightness: u8,
    },

    /// Set temperature for climate devices
    SetTemperature {
        room: Option<String>,
        temperature: f32,
        unit: TemperatureUnit,
    },

    /// Get current state of a device or room
    GetState {
        room: Option<String>,
        device_type: Option<String>,
        device_name: Option<String>,
    },

    /// Open a cover (blinds, garage door)
    OpenCover {
        room: Option<String>,
        device_name: Option<String>,
    },

    /// Close a cover
    CloseCover {
        room: Option<String>,
        device_name: Option<String>,
    },

    /// Lock a door/lock
    Lock {
        room: Option<String>,
        device_name: Option<String>,
    },

    /// Unlock a door/lock
    Unlock {
        room: Option<String>,
        device_name: Option<String>,
    },

    /// Activate a scene
    ActivateScene {
        scene_name: String,
    },

    /// Request setup guide/onboarding help
    SetupGuide,

    /// Unknown/unparseable command
    Unknown,
}

/// Temperature unit
#[derive(Debug, Clone, PartialEq)]
pub enum TemperatureUnit {
    Fahrenheit,
    Celsius,
}

/// Smart home intent parser
pub struct SmartHomeIntentParser;

impl SmartHomeIntentParser {
    /// Parse a natural language command into a SmartHomeIntent
    pub fn parse(text: &str) -> SmartHomeIntent {
        let text_lower = text.to_lowercase();

        // Setup guide / onboarding (check first as it's very specific)
        if text_lower.contains("help") && (text_lower.contains("set up") || text_lower.contains("setup"))
            || text_lower.contains("guide me through")
            || text_lower.contains("how do i add")
            || text_lower.contains("onboarding")
            || (text_lower.contains("setup") && text_lower.contains("guide"))
        {
            return SmartHomeIntent::SetupGuide;
        }

        // Activate scene (check first as it's specific)
        if let Some(scene_name) = Self::extract_scene(&text_lower) {
            return SmartHomeIntent::ActivateScene { scene_name };
        }

        // Set brightness
        if let Some(brightness) = Self::extract_brightness(&text_lower) {
            let room = Self::extract_room(&text_lower);
            let device_name = Self::extract_device_name(&text_lower, "light");
            return SmartHomeIntent::SetBrightness {
                room,
                device_name,
                brightness,
            };
        }

        // Set temperature
        if let Some((temperature, unit)) = Self::extract_temperature(&text_lower) {
            let room = Self::extract_room(&text_lower);
            return SmartHomeIntent::SetTemperature {
                room,
                temperature,
                unit,
            };
        }

        // Open/Close covers
        if text_lower.contains("open") {
            if let Some(device_type) = Self::extract_device_type(&text_lower) {
                if device_type == "cover" || device_type == "blind" || device_type == "garage" {
                    let room = Self::extract_room(&text_lower);
                    let device_name = Self::extract_device_name(&text_lower, &device_type);
                    return SmartHomeIntent::OpenCover { room, device_name };
                }
            }
        }

        if text_lower.contains("close") {
            if let Some(device_type) = Self::extract_device_type(&text_lower) {
                if device_type == "cover" || device_type == "blind" || device_type == "garage" {
                    let room = Self::extract_room(&text_lower);
                    let device_name = Self::extract_device_name(&text_lower, &device_type);
                    return SmartHomeIntent::CloseCover { room, device_name };
                }
            }
        }

        // Lock/Unlock
        if text_lower.contains("lock") && !text_lower.contains("unlock") {
            let room = Self::extract_room(&text_lower);
            let device_name = Self::extract_device_name(&text_lower, "lock");
            return SmartHomeIntent::Lock { room, device_name };
        }

        if text_lower.contains("unlock") {
            let room = Self::extract_room(&text_lower);
            let device_name = Self::extract_device_name(&text_lower, "lock");
            return SmartHomeIntent::Unlock { room, device_name };
        }

        // Get state (questions)
        if Self::is_question(&text_lower) {
            let room = Self::extract_room(&text_lower);
            let device_type = Self::extract_device_type(&text_lower);
            let device_name = device_type.as_ref()
                .and_then(|dtype| Self::extract_device_name(&text_lower, dtype));

            return SmartHomeIntent::GetState {
                room,
                device_type,
                device_name,
            };
        }

        // Turn on/off/toggle
        let action = Self::extract_action(&text_lower);

        let room = Self::extract_room(&text_lower);
        let device_type = Self::extract_device_type(&text_lower);
        let device_name = device_type.as_ref()
            .and_then(|dtype| Self::extract_device_name(&text_lower, dtype));

        match action {
            Action::TurnOn => SmartHomeIntent::TurnOn {
                room,
                device_type,
                device_name,
            },
            Action::TurnOff => SmartHomeIntent::TurnOff {
                room,
                device_type,
                device_name,
            },
            Action::Toggle => SmartHomeIntent::Toggle {
                room,
                device_type,
                device_name,
            },
            Action::Unknown => SmartHomeIntent::Unknown,
        }
    }

    // =========================================================================
    // Private extraction methods
    // =========================================================================

    /// Extract action (turn on, turn off, toggle)
    fn extract_action(text: &str) -> Action {
        if text.contains("turn on") || text.contains("switch on") || text.contains("enable") {
            Action::TurnOn
        } else if text.contains("turn off") || text.contains("switch off") || text.contains("disable") {
            Action::TurnOff
        } else if text.contains("toggle") {
            Action::Toggle
        } else {
            Action::Unknown
        }
    }

    /// Extract room name from text
    fn extract_room(text: &str) -> Option<String> {
        static ROOM_PATTERNS: Lazy<Vec<(&str, &str)>> = Lazy::new(|| {
            vec![
                ("kitchen", "kitchen"),
                ("bedroom", "bedroom"),
                ("living room", "living_room"),
                ("bathroom", "bathroom"),
                ("office", "office"),
                ("garage", "garage"),
                ("basement", "basement"),
                ("hallway", "hallway"),
                ("dining room", "dining_room"),
                ("guest room", "guest_room"),
                ("master bedroom", "master_bedroom"),
                ("kids room", "kids_room"),
                ("family room", "family_room"),
                ("laundry room", "laundry_room"),
            ]
        });

        for (pattern, normalized) in ROOM_PATTERNS.iter() {
            if text.contains(pattern) {
                return Some(normalized.to_string());
            }
        }

        None
    }

    /// Extract device type from text
    fn extract_device_type(text: &str) -> Option<String> {
        static DEVICE_PATTERNS: Lazy<Vec<(&str, &str)>> = Lazy::new(|| {
            vec![
                ("lights", "light"),
                ("light", "light"),
                ("lamp", "light"),
                ("switch", "switch"),
                ("switches", "switch"),
                ("fan", "fan"),
                ("thermostat", "climate"),
                ("temperature", "climate"),
                ("ac", "climate"),
                ("heat", "climate"),
                ("blinds", "cover"),
                ("blind", "cover"),
                ("shades", "cover"),
                ("garage", "cover"),
                ("door lock", "lock"),
                ("lock", "lock"),
                ("sensor", "sensor"),
                ("motion", "binary_sensor"),
            ]
        });

        for (pattern, device_type) in DEVICE_PATTERNS.iter() {
            if text.contains(pattern) {
                return Some(device_type.to_string());
            }
        }

        None
    }

    /// Extract specific device name from text
    fn extract_device_name(text: &str, device_type: &str) -> Option<String> {
        // Look for common device name patterns
        static DEVICE_NAME_PATTERNS: Lazy<Vec<&str>> = Lazy::new(|| {
            vec![
                "ceiling", "table", "desk", "floor", "bedside",
                "main", "overhead", "reading", "accent",
                "front", "back", "side", "left", "right",
            ]
        });

        for pattern in DEVICE_NAME_PATTERNS.iter() {
            if text.contains(pattern) {
                // Return normalized name
                return Some(format!("{}_{}", pattern, device_type));
            }
        }

        None
    }

    /// Extract brightness percentage (0-100)
    fn extract_brightness(text: &str) -> Option<u8> {
        static BRIGHTNESS_RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?:brightness|dim|bright).*?(\d+)\s*(?:%|percent)?").unwrap()
        });

        static SET_TO_RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?:set to|to)\s+(\d+)\s*(?:%|percent)?").unwrap()
        });

        // Try brightness-specific patterns first
        if let Some(caps) = BRIGHTNESS_RE.captures(text) {
            if let Some(num_str) = caps.get(1) {
                if let Ok(num) = num_str.as_str().parse::<u8>() {
                    return Some(num.min(100));
                }
            }
        }

        // Try "set to X%" pattern
        if let Some(caps) = SET_TO_RE.captures(text) {
            if let Some(num_str) = caps.get(1) {
                if let Ok(num) = num_str.as_str().parse::<u8>() {
                    return Some(num.min(100));
                }
            }
        }

        None
    }

    /// Extract temperature and unit
    fn extract_temperature(text: &str) -> Option<(f32, TemperatureUnit)> {
        static TEMP_RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(\d+(?:\.\d+)?)\s*(?:degrees?|°)?\s*(f|fahrenheit|c|celsius)?").unwrap()
        });

        if let Some(caps) = TEMP_RE.captures(text) {
            if let Some(temp_str) = caps.get(1) {
                if let Ok(temp) = temp_str.as_str().parse::<f32>() {
                    let unit = if let Some(unit_match) = caps.get(2) {
                        let unit_str = unit_match.as_str().to_lowercase();
                        if unit_str.starts_with('c') {
                            TemperatureUnit::Celsius
                        } else {
                            TemperatureUnit::Fahrenheit
                        }
                    } else {
                        // Default to Fahrenheit (common in US)
                        TemperatureUnit::Fahrenheit
                    };

                    return Some((temp, unit));
                }
            }
        }

        None
    }

    /// Extract scene name
    fn extract_scene(text: &str) -> Option<String> {
        static SCENE_RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?:activate|turn on|enable|set)\s+(?:the\s+)?(\w+(?:\s+\w+)?)\s+(?:scene|mode)").unwrap()
        });

        if let Some(caps) = SCENE_RE.captures(text) {
            if let Some(scene_match) = caps.get(1) {
                let scene_name = scene_match.as_str().trim();
                return Some(Self::to_title_case(scene_name));
            }
        }

        None
    }

    /// Check if text is a question (state query)
    fn is_question(text: &str) -> bool {
        text.contains("what")
            || text.contains("what's")
            || text.contains("is the")
            || text.contains("how")
            || text.contains("?")
    }

    /// Convert string to Title Case
    fn to_title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Internal action enum
#[derive(Debug, Clone, PartialEq)]
enum Action {
    TurnOn,
    TurnOff,
    Toggle,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_on_kitchen_lights() {
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
    fn test_turn_off_bedroom_lamp() {
        let intent = SmartHomeIntentParser::parse("Turn off the bedroom lamp");
        assert_eq!(
            intent,
            SmartHomeIntent::TurnOff {
                room: Some("bedroom".to_string()),
                device_type: Some("light".to_string()),
                device_name: None,
            }
        );
    }

    #[test]
    fn test_set_brightness() {
        let intent = SmartHomeIntentParser::parse("Set living room lights to 75%");
        assert_eq!(
            intent,
            SmartHomeIntent::SetBrightness {
                room: Some("living_room".to_string()),
                device_name: None,
                brightness: 75,
            }
        );
    }

    #[test]
    fn test_set_temperature_fahrenheit() {
        let intent = SmartHomeIntentParser::parse("Set bedroom to 72 degrees");
        assert_eq!(
            intent,
            SmartHomeIntent::SetTemperature {
                room: Some("bedroom".to_string()),
                temperature: 72.0,
                unit: TemperatureUnit::Fahrenheit,
            }
        );
    }

    #[test]
    fn test_set_temperature_celsius() {
        let intent = SmartHomeIntentParser::parse("Set temperature to 22 celsius");
        assert_eq!(
            intent,
            SmartHomeIntent::SetTemperature {
                room: None,
                temperature: 22.0,
                unit: TemperatureUnit::Celsius,
            }
        );
    }

    #[test]
    fn test_get_state_temperature() {
        let intent = SmartHomeIntentParser::parse("What's the temperature in the living room?");
        assert_eq!(
            intent,
            SmartHomeIntent::GetState {
                room: Some("living_room".to_string()),
                device_type: Some("climate".to_string()),
                device_name: None,
            }
        );
    }

    #[test]
    fn test_activate_scene() {
        let intent = SmartHomeIntentParser::parse("Activate movie mode");
        assert_eq!(
            intent,
            SmartHomeIntent::ActivateScene {
                scene_name: "Movie Mode".to_string(),
            }
        );
    }

    #[test]
    fn test_open_garage() {
        let intent = SmartHomeIntentParser::parse("Open the garage door");
        assert_eq!(
            intent,
            SmartHomeIntent::OpenCover {
                room: Some("garage".to_string()),
                device_name: None,
            }
        );
    }

    #[test]
    fn test_close_blinds() {
        let intent = SmartHomeIntentParser::parse("Close the bedroom blinds");
        assert_eq!(
            intent,
            SmartHomeIntent::CloseCover {
                room: Some("bedroom".to_string()),
                device_name: None,
            }
        );
    }

    #[test]
    fn test_lock_door() {
        let intent = SmartHomeIntentParser::parse("Lock the front door");
        assert_eq!(
            intent,
            SmartHomeIntent::Lock {
                room: None,
                device_name: Some("front_lock".to_string()),
            }
        );
    }

    #[test]
    fn test_unlock_door() {
        let intent = SmartHomeIntentParser::parse("Unlock the back door");
        assert_eq!(
            intent,
            SmartHomeIntent::Unlock {
                room: None,
                device_name: Some("back_lock".to_string()),
            }
        );
    }

    #[test]
    fn test_toggle_switch() {
        let intent = SmartHomeIntentParser::parse("Toggle the office fan");
        assert_eq!(
            intent,
            SmartHomeIntent::Toggle {
                room: Some("office".to_string()),
                device_type: Some("fan".to_string()),
                device_name: None,
            }
        );
    }

    #[test]
    fn test_turn_on_all_lights() {
        let intent = SmartHomeIntentParser::parse("Turn on all lights");
        assert_eq!(
            intent,
            SmartHomeIntent::TurnOn {
                room: None,
                device_type: Some("light".to_string()),
                device_name: None,
            }
        );
    }

    #[test]
    fn test_brightness_dim() {
        let intent = SmartHomeIntentParser::parse("Dim the lights to 30 percent");
        assert_eq!(
            intent,
            SmartHomeIntent::SetBrightness {
                room: None,
                device_name: None,
                brightness: 30,
            }
        );
    }

    #[test]
    fn test_unknown_command() {
        let intent = SmartHomeIntentParser::parse("Play some music");
        assert_eq!(intent, SmartHomeIntent::Unknown);
    }

    #[test]
    fn test_room_extraction() {
        assert_eq!(
            SmartHomeIntentParser::extract_room("in the kitchen"),
            Some("kitchen".to_string())
        );
        assert_eq!(
            SmartHomeIntentParser::extract_room("living room lights"),
            Some("living_room".to_string())
        );
        assert_eq!(
            SmartHomeIntentParser::extract_room("master bedroom"),
            Some("master_bedroom".to_string())
        );
    }

    #[test]
    fn test_device_type_extraction() {
        assert_eq!(
            SmartHomeIntentParser::extract_device_type("turn on lights"),
            Some("light".to_string())
        );
        assert_eq!(
            SmartHomeIntentParser::extract_device_type("the thermostat"),
            Some("climate".to_string())
        );
        assert_eq!(
            SmartHomeIntentParser::extract_device_type("garage door"),
            Some("cover".to_string())
        );
    }

    #[test]
    fn test_temperature_extraction() {
        assert_eq!(
            SmartHomeIntentParser::extract_temperature("set to 72 degrees"),
            Some((72.0, TemperatureUnit::Fahrenheit))
        );
        assert_eq!(
            SmartHomeIntentParser::extract_temperature("22 celsius"),
            Some((22.0, TemperatureUnit::Celsius))
        );
        assert_eq!(
            SmartHomeIntentParser::extract_temperature("75°F"),
            Some((75.0, TemperatureUnit::Fahrenheit))
        );
    }

    #[test]
    fn test_brightness_extraction() {
        assert_eq!(SmartHomeIntentParser::extract_brightness("brightness 50%"), Some(50));
        assert_eq!(SmartHomeIntentParser::extract_brightness("set to 80 percent"), Some(80));
        assert_eq!(SmartHomeIntentParser::extract_brightness("dim to 25"), Some(25));
    }

    #[test]
    fn test_title_case() {
        assert_eq!(
            SmartHomeIntentParser::to_title_case("movie mode"),
            "Movie Mode"
        );
        assert_eq!(
            SmartHomeIntentParser::to_title_case("good night"),
            "Good Night"
        );
    }
}
