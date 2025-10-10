# RAG (Retrieval-Augmented Generation) Architecture

## Overview

This document describes the architecture for integrating web search capabilities into Aura Desktop using a Retrieval-Augmented Generation (RAG) pattern. This feature enables Aura to answer questions about current events and topics beyond its training data by fetching real-time information from the internet.

## Design Principles

1. **Privacy First**: Explicit user opt-in required, disabled by default
2. **Graceful Degradation**: Seamless fallback to offline mode when disabled or unavailable
3. **User Choice**: Multiple search backend options (SearXNG, Brave Search)
4. **Transparency**: Clear indication when online search is used
5. **Offline Capable**: Core functionality remains 100% offline

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Query                               │
│                  "What's the latest news about..."              │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Frontend (React/TypeScript)                   │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │  Chat Component                                            │ │
│  │  - Checks Settings: online_mode_enabled?                  │ │
│  │  - Shows "Searching web..." indicator                     │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                Tauri Command: query_llm_with_rag                │
│                    (src-tauri/src/lib.rs)                       │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │  1. Check online_mode_enabled from Settings               │ │
│  │  2. If enabled → call web_search module                   │ │
│  │  3. Format search results as context                      │ │
│  │  4. Prepend context to user query                         │ │
│  │  5. Send augmented prompt to LLM                          │ │
│  │  6. Return LLM response to frontend                       │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│              Web Search Module (web_search.rs)                  │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │  SearchBackend Enum:                                       │ │
│  │  - SearXNG(url)                                           │ │
│  │  - BraveSearch(api_key)                                   │ │
│  │                                                            │ │
│  │  pub async fn search_web(                                 │ │
│  │      query: &str,                                         │ │
│  │      backend: SearchBackend,                              │ │
│  │      max_results: usize                                   │ │
│  │  ) -> Result<Vec<SearchResult>>                           │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────┬───────────────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┐
        │                           │
        ▼                           ▼
┌──────────────────┐      ┌──────────────────┐
│  SearXNG Client  │      │  Brave Search    │
│  (searxng crate) │      │  (reqwest HTTP)  │
│                  │      │                  │
│  - Public inst.  │      │  - API key req.  │
│  - Self-hosted   │      │  - 2K free/mo    │
│  - No API key    │      │  - Independent   │
│  - 245+ engines  │      │  - 30B+ index    │
└──────────────────┘      └──────────────────┘
        │                           │
        └─────────────┬─────────────┘
                      │
                      ▼
              Internet Search
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Search Results                             │
│  Vec<SearchResult> {                                            │
│    title: String,                                               │
│    url: String,                                                 │
│    snippet: String,                                             │
│    published_date: Option<DateTime>                            │
│  }                                                              │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Context Formatting                             │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │  format_search_context(results: Vec<SearchResult>)        │ │
│  │                                                            │ │
│  │  Returns formatted string:                                │ │
│  │  """                                                       │ │
│  │  === Web Search Results ===                               │ │
│  │  [1] Title: ...                                           │ │
│  │      URL: ...                                             │ │
│  │      Content: ...                                         │ │
│  │  [2] Title: ...                                           │ │
│  │      ...                                                  │ │
│  │  """                                                       │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Augmented Prompt Construction                  │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │  System prompt:                                            │ │
│  │  "You are Aura, a helpful AI assistant. When provided     │ │
│  │   with web search results, use them to answer questions   │ │
│  │   accurately. Cite sources when relevant."                │ │
│  │                                                            │ │
│  │  User message:                                             │ │
│  │  {formatted_search_context}                               │ │
│  │                                                            │ │
│  │  User Question: {original_query}                          │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Local LLM (Ollama)                             │
│  - Processes augmented prompt                                   │
│  - Generates answer using search context                        │
│  - Returns response                                             │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Frontend Display                               │
│  - Shows LLM response                                           │
│  - Optionally shows sources used                                │
│  - Saves to conversation history                                │
└─────────────────────────────────────────────────────────────────┘
```

## Component Design

### 1. Settings Schema Extension

**Database Migration (SQLite):**
```sql
ALTER TABLE settings ADD COLUMN online_mode_enabled INTEGER DEFAULT 0;
ALTER TABLE settings ADD COLUMN search_backend TEXT DEFAULT 'searxng';
ALTER TABLE settings ADD COLUMN searxng_instance_url TEXT DEFAULT 'https://searx.be';
ALTER TABLE settings ADD COLUMN brave_search_api_key TEXT DEFAULT NULL;
ALTER TABLE settings ADD COLUMN max_search_results INTEGER DEFAULT 5;
```

**Rust Struct Update (`src-tauri/src/database.rs`):**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // ... existing fields ...

    // RAG / Online Mode Settings
    pub online_mode_enabled: bool,
    pub search_backend: SearchBackendType,
    pub searxng_instance_url: String,
    pub brave_search_api_key: Option<String>,
    pub max_search_results: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchBackendType {
    SearXNG,
    BraveSearch,
}
```

### 2. Web Search Module

**New File: `src-tauri/src/web_search.rs`**

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Search backend configuration
#[derive(Debug, Clone)]
pub enum SearchBackend {
    SearXNG { instance_url: String },
    BraveSearch { api_key: String },
}

/// Individual search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub published_date: Option<DateTime<Utc>>,
}

/// Main search function
pub async fn search_web(
    query: &str,
    backend: SearchBackend,
    max_results: usize,
) -> Result<Vec<SearchResult>, SearchError> {
    match backend {
        SearchBackend::SearXNG { instance_url } => {
            search_searxng(query, &instance_url, max_results).await
        }
        SearchBackend::BraveSearch { api_key } => {
            search_brave(query, &api_key, max_results).await
        }
    }
}

/// SearXNG implementation using searxng crate
async fn search_searxng(
    query: &str,
    instance_url: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>, SearchError> {
    // Use searxng crate
    // Implementation details...
}

/// Brave Search implementation using reqwest
async fn search_brave(
    query: &str,
    api_key: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>, SearchError> {
    // HTTP request to Brave Search API
    // Implementation details...
}

/// Format search results as context for LLM
pub fn format_search_context(results: &[SearchResult]) -> String {
    let mut context = String::from("=== Web Search Results ===\n\n");

    for (idx, result) in results.iter().enumerate() {
        context.push_str(&format!(
            "[{}] {}\n    URL: {}\n    {}\n\n",
            idx + 1,
            result.title,
            result.url,
            result.snippet
        ));
    }

    context.push_str("=== End of Search Results ===\n\n");
    context
}

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Search backend unavailable: {0}")]
    BackendUnavailable(String),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}
```

### 3. Modified LLM Query Flow

**Update: `src-tauri/src/lib.rs` (existing `query_llm` function)**

```rust
#[tauri::command]
async fn query_llm_with_rag(
    conversation_id: i32,
    message: String,
    settings: State<'_, Arc<Mutex<Settings>>>,
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<LlmResponse, AuraError> {
    let settings = settings.lock().await.clone();

    // Determine final prompt (with or without RAG)
    let augmented_message = if settings.online_mode_enabled {
        log::info!("Online mode enabled, performing web search...");

        // Perform web search
        let search_backend = match settings.search_backend {
            SearchBackendType::SearXNG => {
                web_search::SearchBackend::SearXNG {
                    instance_url: settings.searxng_instance_url.clone(),
                }
            }
            SearchBackendType::BraveSearch => {
                web_search::SearchBackend::BraveSearch {
                    api_key: settings.brave_search_api_key
                        .clone()
                        .ok_or(AuraError::MissingApiKey("Brave Search"))?,
                }
            }
        };

        match web_search::search_web(&message, search_backend, settings.max_search_results).await {
            Ok(results) => {
                log::info!("Web search successful: {} results", results.len());

                // Format context and augment prompt
                let context = web_search::format_search_context(&results);
                format!(
                    "{}\n\nUser Question: {}",
                    context,
                    message
                )
            }
            Err(e) => {
                log::warn!("Web search failed: {}, falling back to offline mode", e);
                message.clone()
            }
        }
    } else {
        log::debug!("Online mode disabled, using offline LLM");
        message.clone()
    };

    // Continue with existing LLM query logic
    // Send augmented_message to LLM via HTTP client
    // ...
}
```

### 4. Settings UI Component

**New React Component: `src/components/Settings/OnlineModeSettings.tsx`**

```tsx
import React from 'react'
import { invoke } from '@tauri-apps/api/core'

interface OnlineModeSettingsProps {
  settings: Settings
  onUpdate: (settings: Partial<Settings>) => void
}

export function OnlineModeSettings({ settings, onUpdate }: OnlineModeSettingsProps) {
  return (
    <div className="settings-section">
      <h3 className="text-lg font-semibold mb-3">Online Mode (Web Search)</h3>

      {/* Privacy Notice */}
      <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 mb-4">
        <div className="flex items-start gap-3">
          <span className="text-2xl">ℹ️</span>
          <div>
            <h4 className="font-semibold text-blue-900 dark:text-blue-100 mb-1">
              Privacy Notice
            </h4>
            <p className="text-sm text-blue-800 dark:text-blue-200">
              When enabled, Aura will connect to the internet to search for current information.
              Your queries will be sent to the selected search provider.
              <strong> This feature is disabled by default</strong> and requires your explicit consent.
            </p>
          </div>
        </div>
      </div>

      {/* Enable/Disable Toggle */}
      <div className="flex items-center justify-between mb-4">
        <div>
          <label className="font-medium text-gray-700 dark:text-gray-300">
            Enable Online Mode
          </label>
          <p className="text-sm text-gray-500 dark:text-gray-400">
            Allow Aura to search the web for real-time information
          </p>
        </div>
        <input
          type="checkbox"
          checked={settings.online_mode_enabled}
          onChange={(e) => onUpdate({ online_mode_enabled: e.target.checked })}
          className="toggle-checkbox"
        />
      </div>

      {settings.online_mode_enabled && (
        <>
          {/* Search Backend Selection */}
          <div className="mb-4">
            <label className="block font-medium text-gray-700 dark:text-gray-300 mb-2">
              Search Provider
            </label>
            <select
              value={settings.search_backend}
              onChange={(e) => onUpdate({ search_backend: e.target.value })}
              className="w-full p-2 border rounded-lg dark:bg-gray-800"
            >
              <option value="searxng">SearXNG (Privacy-Focused, No API Key)</option>
              <option value="brave">Brave Search (Requires API Key)</option>
            </select>
          </div>

          {/* Backend-Specific Settings */}
          {settings.search_backend === 'searxng' && (
            <div className="mb-4">
              <label className="block font-medium text-gray-700 dark:text-gray-300 mb-2">
                SearXNG Instance URL
              </label>
              <input
                type="url"
                value={settings.searxng_instance_url}
                onChange={(e) => onUpdate({ searxng_instance_url: e.target.value })}
                placeholder="https://searx.be"
                className="w-full p-2 border rounded-lg dark:bg-gray-800"
              />
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                Public instances: searx.be, searx.org, or self-hosted
              </p>
            </div>
          )}

          {settings.search_backend === 'brave' && (
            <div className="mb-4">
              <label className="block font-medium text-gray-700 dark:text-gray-300 mb-2">
                Brave Search API Key
              </label>
              <input
                type="password"
                value={settings.brave_search_api_key || ''}
                onChange={(e) => onUpdate({ brave_search_api_key: e.target.value })}
                placeholder="Enter API key from brave.com/search/api"
                className="w-full p-2 border rounded-lg dark:bg-gray-800"
              />
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                Free tier: 2,000 queries/month
              </p>
            </div>
          )}

          {/* Max Results */}
          <div className="mb-4">
            <label className="block font-medium text-gray-700 dark:text-gray-300 mb-2">
              Max Search Results: {settings.max_search_results}
            </label>
            <input
              type="range"
              min="3"
              max="10"
              value={settings.max_search_results}
              onChange={(e) => onUpdate({ max_search_results: parseInt(e.target.value) })}
              className="w-full"
            />
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              More results = more context for LLM (but longer processing time)
            </p>
          </div>
        </>
      )}
    </div>
  )
}
```

## Data Flow Sequence

1. **User Input**: User types question in chat interface
2. **Settings Check**: Frontend checks `online_mode_enabled` setting
3. **Conditional Search**:
   - If enabled → Show "Searching web..." indicator
   - If disabled → Skip to step 6
4. **Web Search**: Backend calls appropriate search API
5. **Context Formatting**: Format search results into structured context
6. **Prompt Augmentation**: Prepend search context to user query
7. **LLM Query**: Send augmented prompt to local LLM (Ollama)
8. **Response**: LLM generates answer using search context
9. **Display**: Show response to user, save to conversation history

## Error Handling & Fallback

### Graceful Degradation Scenarios

| Scenario | Behavior |
|----------|----------|
| **Online mode disabled** | Skip search, query LLM directly |
| **No internet connection** | Catch network error, fallback to offline mode, log warning |
| **Search API timeout** | 5-second timeout, fallback to offline mode |
| **Invalid API key** | Return error to user, suggest checking settings |
| **Rate limit exceeded** | Show error, suggest waiting or changing backend |
| **Empty search results** | Query LLM without context, log info |

### Error Logging

```rust
match web_search::search_web(&query, backend, max_results).await {
    Ok(results) if !results.is_empty() => {
        log::info!("✓ Web search successful: {} results", results.len());
        // Use results...
    }
    Ok(results) => {
        log::warn!("⚠ Web search returned 0 results, using offline mode");
        // Fallback to offline...
    }
    Err(SearchError::Network(e)) => {
        log::warn!("⚠ Network error during search: {}, fallback to offline", e);
        // Fallback to offline...
    }
    Err(e) => {
        log::error!("✗ Search failed: {}, fallback to offline", e);
        // Fallback to offline...
    }
}
```

## Privacy Considerations

### User Consent Flow

1. **Default State**: Online mode disabled by default
2. **First-Time Setup**: Settings modal shows privacy notice prominently
3. **Explicit Opt-In**: User must toggle switch AND acknowledge notice
4. **Transparent Logging**: All web searches logged (optional user setting)
5. **Easy Opt-Out**: Single toggle to disable immediately

### Data Retention

- **Search Queries**: Not stored by Aura (only conversation history)
- **Search Results**: Cached in memory only during LLM query
- **API Keys**: Stored in OS keyring (same as LLM API keys)
- **Third-Party Retention**:
  - SearXNG: Depends on instance (self-hosted = zero retention)
  - Brave Search: 90 days for billing purposes

### Privacy-First Recommendations

Display in Settings UI:

```
🔒 Privacy Recommendations:
✓ Best Privacy: SearXNG (self-hosted or trusted public instance)
✓ Good Privacy: SearXNG (public instance like searx.be)
✓ Moderate Privacy: Brave Search (90-day retention, no tracking)
```

## Performance Considerations

### Latency Targets

| Component | Target | Timeout |
|-----------|--------|---------|
| Web search | < 2 seconds | 5 seconds |
| Context formatting | < 100ms | N/A |
| LLM query (with context) | < 5 seconds | 30 seconds |
| **Total RAG flow** | **< 7 seconds** | **35 seconds** |

### Optimization Strategies

1. **Parallel Processing**: Start LLM request while formatting context
2. **Result Limiting**: Default max_results = 5 (balance context vs speed)
3. **Caching**: Consider caching search results for repeated queries (optional future enhancement)
4. **Streaming**: Stream LLM response to user while generating

## Testing Strategy

### Unit Tests

- `web_search.rs`: Test each search backend independently
- `format_search_context()`: Test formatting with various result counts
- Settings validation: Test invalid URLs, missing API keys

### Integration Tests

- End-to-end RAG flow with mock search API
- Fallback behavior when search fails
- Settings persistence and reload

### Manual Testing Checklist

- [ ] Disable online mode → verify offline LLM query works
- [ ] Enable SearXNG → verify search results appear in context
- [ ] Enable Brave Search → verify API key validation
- [ ] Disconnect internet → verify graceful fallback
- [ ] Query with 0 search results → verify fallback
- [ ] Query with 10 search results → verify all formatted correctly
- [ ] Toggle backends → verify seamless switching

## Dependencies

### New Rust Crates

```toml
[dependencies]
# Existing crates...

# Web search
searxng = "0.1.0"           # SearXNG client
reqwest = { version = "0.11", features = ["json"] }  # Already exists for LLM
chrono = { version = "0.4", features = ["serde"] }   # Already exists
thiserror = "1.0"           # Already exists

# Async runtime
tokio = { version = "1", features = ["full"] }  # Already exists
```

### Frontend Dependencies

No new dependencies required (Tauri invoke API already available).

## Future Enhancements

1. **Citation System**: Automatically cite sources in LLM response
2. **Result Caching**: Cache search results for 5 minutes to reduce API calls
3. **Custom Search Engines**: Allow users to add custom SearXNG instances
4. **Search History**: Optional search query history for debugging
5. **Multi-Query RAG**: Break complex questions into multiple searches
6. **Fact-Checking Mode**: Cross-reference multiple sources for accuracy
7. **Image Search**: Extend to image results for visual context
8. **Local Indexing**: Combine web search with local document indexing

## Security Considerations

### API Key Storage

- Use OS keyring (already implemented for LLM API keys)
- Never log API keys
- Validate API key format before storing

### Input Sanitization

- Sanitize user queries before sending to search APIs
- Prevent injection attacks in search URLs
- Validate search result URLs before displaying

### Rate Limiting

- Implement client-side rate limiting (max 10 searches/minute)
- Track API usage to avoid exceeding free tier
- Show warning when approaching rate limits

## Acceptance Criteria Mapping

| AC | Implementation |
|----|----------------|
| **AC1: Privacy First** | Settings UI with explicit privacy notice, disabled by default, opt-in required |
| **AC2: Search API Integration** | `web_search.rs` module with SearXNG and Brave Search support |
| **AC3: RAG Logic** | Modified `query_llm_with_rag` function with conditional search |
| **AC4: Contextual Prompting** | `format_search_context()` function formats results as LLM context |
| **AC5: Graceful Fallback** | Error handling in RAG flow, seamless offline fallback |

---

**Document Version**: 1.0
**Last Updated**: 2025-10-09
**Author**: Claude Code (AuraPM Epic Implementation)
