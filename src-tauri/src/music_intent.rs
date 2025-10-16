// Music Intent Recognition
//
// This module parses natural language music commands and extracts structured intent data.
// It supports both regex-based pattern matching and optional LLM-based parsing for
// more sophisticated command understanding.

use regex::Regex;
use once_cell::sync::Lazy;

/// Music command intent types
///
/// **AC2: Possessive Context Support**
/// Each variant now includes an `is_possessive` field to indicate if the command
/// used possessive pronouns like "my" (e.g., "play my workout playlist").
/// This context is used for multi-user personalization to ensure the correct
/// user's playlists and preferences are accessed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MusicIntent {
    /// Play a specific song, optionally by a specific artist
    PlaySong {
        song: String,
        artist: Option<String>,
        is_possessive: bool, // NEW: True if command used "my" or similar
    },

    /// Play a user's playlist by name
    PlayPlaylist {
        playlist_name: String,
        is_possessive: bool, // NEW: True if command used "my playlist"
    },

    /// Play music by a specific artist (top tracks)
    PlayArtist {
        artist: String,
        is_possessive: bool, // NEW: True if command used "my" or similar
    },

    /// Pause current playback
    Pause,

    /// Resume paused playback
    Resume,

    /// Skip to next track
    Next,

    /// Skip to previous track
    Previous,

    /// Get information about currently playing track
    GetCurrentTrack,

    /// Unknown or unparseable command
    Unknown,
}

/// Music intent parser
pub struct MusicIntentParser;

// Compiled regex patterns (initialized once)
static RE_PLAY_SONG_WITH_ARTIST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)play\s+(.+?)\s+by\s+(.+)").unwrap()
});

static RE_PLAY_PLAYLIST: Lazy<Regex> = Lazy::new(|| {
    // Captures "my" as optional group 1, playlist name as group 2
    Regex::new(r"(?i)play\s+(my\s+)?(.+?)\s+playlist").unwrap()
});

static RE_PLAY_SONG_OR_ARTIST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)play\s+(.+)").unwrap()
});

// **AC2: Possessive pronoun detection**
static RE_POSSESSIVE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(my|mine|our|ours)\b").unwrap()
});

impl MusicIntentParser {
    /// Parse a natural language music command into a structured intent
    ///
    /// # Examples
    ///
    /// ```
    /// use aura_desktop_lib::music_intent::{MusicIntentParser, MusicIntent};
    ///
    /// let intent = MusicIntentParser::parse("play Despacito by Luis Fonsi");
    /// assert_eq!(
    ///     intent,
    ///     MusicIntent::PlaySong {
    ///         song: "Despacito".to_string(),
    ///         artist: Some("Luis Fonsi".to_string())
    ///     }
    /// );
    /// ```
    pub fn parse(text: &str) -> MusicIntent {
        let text_lower = text.to_lowercase();
        let text_trimmed = text_lower.trim();

        // **AC2: Detect possessive pronouns in the command**
        let is_possessive = Self::is_possessive(text_trimmed);

        // Check for control commands first (highest priority)
        if Self::is_pause_command(text_trimmed) {
            return MusicIntent::Pause;
        }

        if Self::is_resume_command(text_trimmed) {
            return MusicIntent::Resume;
        }

        if Self::is_next_command(text_trimmed) {
            return MusicIntent::Next;
        }

        if Self::is_previous_command(text_trimmed) {
            return MusicIntent::Previous;
        }

        if Self::is_get_current_track_command(text_trimmed) {
            return MusicIntent::GetCurrentTrack;
        }

        // Parse "play" commands
        if text_trimmed.contains("play") {
            // Try to parse playlist command
            if let Some(playlist_name) = Self::extract_playlist(text_trimmed) {
                return MusicIntent::PlayPlaylist {
                    playlist_name,
                    is_possessive, // AC2: Pass possessive context
                };
            }

            // Try to parse song with artist
            if let Some((song, artist)) = Self::extract_song_and_artist(text_trimmed) {
                return MusicIntent::PlaySong {
                    song,
                    artist,
                    is_possessive, // AC2: Pass possessive context
                };
            }

            // Fallback: treat as song or artist name
            if let Some(query) = Self::extract_generic_play_query(text_trimmed) {
                // Heuristic: if query looks like an artist name (short, capitalized), treat as artist
                // Otherwise, treat as song
                if Self::looks_like_artist_name(&query) {
                    return MusicIntent::PlayArtist {
                        artist: query,
                        is_possessive, // AC2: Pass possessive context
                    };
                } else {
                    return MusicIntent::PlaySong {
                        song: query,
                        artist: None,
                        is_possessive, // AC2: Pass possessive context
                    };
                }
            }
        }

        MusicIntent::Unknown
    }

    /// Check if the command contains possessive pronouns (AC2)
    ///
    /// Detects words like "my", "mine", "our", "ours" to identify
    /// user-specific commands that require personalization.
    fn is_possessive(text: &str) -> bool {
        RE_POSSESSIVE.is_match(text)
    }

    /// Check if the command is a pause command
    fn is_pause_command(text: &str) -> bool {
        matches!(
            text,
            "pause" | "stop" | "pause music" | "stop music" | "pause the music" | "stop the music"
        ) || text.starts_with("pause ") || text.starts_with("stop ")
    }

    /// Check if the command is a resume command
    fn is_resume_command(text: &str) -> bool {
        matches!(
            text,
            "resume" | "continue" | "unpause" | "play" | "resume music" | "continue music" | "unpause music"
        ) || text.starts_with("resume ") || text.starts_with("continue ")
    }

    /// Check if the command is a next track command
    fn is_next_command(text: &str) -> bool {
        matches!(
            text,
            "next" | "skip" | "next song" | "skip song" | "next track" | "skip track"
        )
    }

    /// Check if the command is a previous track command
    fn is_previous_command(text: &str) -> bool {
        matches!(
            text,
            "previous" | "back" | "previous song" | "go back" | "previous track" | "last song" | "last track"
        )
    }

    /// Check if the command is asking for current track info
    fn is_get_current_track_command(text: &str) -> bool {
        (text.contains("what") || text.contains("what's") || text.contains("whats"))
            && (text.contains("playing") || text.contains("song") || text.contains("track"))
    }

    /// Extract playlist name from command (AC2)
    ///
    /// Regex groups: group 1 = "my " (optional), group 2 = playlist name
    fn extract_playlist(text: &str) -> Option<String> {
        RE_PLAY_PLAYLIST.captures(text).and_then(|caps| {
            caps.get(2).map(|m| {  // Group 2 is the playlist name
                Self::title_case(m.as_str().trim())
            })
        })
    }

    /// Extract song name and artist from command
    fn extract_song_and_artist(text: &str) -> Option<(String, Option<String>)> {
        RE_PLAY_SONG_WITH_ARTIST.captures(text).and_then(|caps| {
            let song = caps.get(1)?.as_str().trim();
            let artist = caps.get(2)?.as_str().trim();

            Some((
                Self::title_case(song),
                Some(Self::title_case(artist)),
            ))
        })
    }

    /// Extract generic play query (song or artist)
    fn extract_generic_play_query(text: &str) -> Option<String> {
        RE_PLAY_SONG_OR_ARTIST.captures(text).and_then(|caps| {
            caps.get(1).map(|m| {
                let query = m.as_str().trim();
                // Filter out common filler words
                let filtered = query
                    .replace(" the ", " ")
                    .replace(" a ", " ")
                    .replace(" some ", " ")
                    .trim()
                    .to_string();

                Self::title_case(&filtered)
            })
        })
    }

    /// Heuristic to determine if a query looks like an artist name
    ///
    /// Artist names tend to be:
    /// - Short (1-3 words)
    /// - Proper nouns (capitalized)
    /// - Not contain common song title words like "love", "night", "day", etc.
    fn looks_like_artist_name(query: &str) -> bool {
        let word_count = query.split_whitespace().count();

        // Very short queries (1-2 words) are more likely to be artists
        if word_count <= 2 {
            return true;
        }

        // Longer queries (3+ words) are more likely to be song titles
        false
    }

    /// Convert string to title case (capitalize first letter of each word)
    fn title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + chars.as_str()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_play_song_with_artist() {
        let intent = MusicIntentParser::parse("play Despacito by Luis Fonsi");
        assert_eq!(
            intent,
            MusicIntent::PlaySong {
                song: "Despacito".to_string(),
                artist: Some("Luis Fonsi".to_string()),
                is_possessive: false,
            }
        );
    }

    #[test]
    fn test_parse_play_song_with_artist_case_insensitive() {
        let intent = MusicIntentParser::parse("PLAY BOHEMIAN RHAPSODY BY QUEEN");
        assert_eq!(
            intent,
            MusicIntent::PlaySong {
                song: "Bohemian Rhapsody".to_string(),
                artist: Some("Queen".to_string()),
                is_possessive: false,
            }
        );
    }

    #[test]
    fn test_parse_play_song_without_artist() {
        let intent = MusicIntentParser::parse("play Imagine");
        assert_eq!(
            intent,
            MusicIntent::PlaySong {
                song: "Imagine".to_string(),
                artist: None,
                is_possessive: false,
            }
        );
    }

    #[test]
    fn test_parse_play_playlist() {
        let intent = MusicIntentParser::parse("play my workout playlist");
        assert_eq!(
            intent,
            MusicIntent::PlayPlaylist {
                playlist_name: "Workout".to_string(),
                is_possessive: true, // AC2: "my" detected
            }
        );
    }

    #[test]
    fn test_parse_play_playlist_without_my() {
        let intent = MusicIntentParser::parse("play chill vibes playlist");
        assert_eq!(
            intent,
            MusicIntent::PlayPlaylist {
                playlist_name: "Chill Vibes".to_string(),
                is_possessive: false, // AC2: No possessive pronoun
            }
        );
    }

    #[test]
    fn test_parse_pause() {
        assert_eq!(MusicIntentParser::parse("pause"), MusicIntent::Pause);
        assert_eq!(MusicIntentParser::parse("stop"), MusicIntent::Pause);
        assert_eq!(MusicIntentParser::parse("pause music"), MusicIntent::Pause);
        assert_eq!(MusicIntentParser::parse("stop the music"), MusicIntent::Pause);
    }

    #[test]
    fn test_parse_resume() {
        assert_eq!(MusicIntentParser::parse("resume"), MusicIntent::Resume);
        assert_eq!(MusicIntentParser::parse("continue"), MusicIntent::Resume);
        assert_eq!(MusicIntentParser::parse("unpause"), MusicIntent::Resume);
    }

    #[test]
    fn test_parse_next() {
        assert_eq!(MusicIntentParser::parse("next"), MusicIntent::Next);
        assert_eq!(MusicIntentParser::parse("skip"), MusicIntent::Next);
        assert_eq!(MusicIntentParser::parse("next song"), MusicIntent::Next);
    }

    #[test]
    fn test_parse_previous() {
        assert_eq!(MusicIntentParser::parse("previous"), MusicIntent::Previous);
        assert_eq!(MusicIntentParser::parse("back"), MusicIntent::Previous);
        assert_eq!(MusicIntentParser::parse("go back"), MusicIntent::Previous);
    }

    #[test]
    fn test_parse_get_current_track() {
        assert_eq!(
            MusicIntentParser::parse("what's playing"),
            MusicIntent::GetCurrentTrack
        );
        assert_eq!(
            MusicIntentParser::parse("what song is this"),
            MusicIntent::GetCurrentTrack
        );
        assert_eq!(
            MusicIntentParser::parse("what's this song"),
            MusicIntent::GetCurrentTrack
        );
    }

    #[test]
    fn test_parse_unknown() {
        assert_eq!(
            MusicIntentParser::parse("make me a sandwich"),
            MusicIntent::Unknown
        );
    }

    #[test]
    fn test_title_case() {
        assert_eq!(
            MusicIntentParser::title_case("hello world"),
            "Hello World"
        );
        assert_eq!(
            MusicIntentParser::title_case("the quick brown fox"),
            "The Quick Brown Fox"
        );
    }
}
