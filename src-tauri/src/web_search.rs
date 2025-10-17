use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Search backend configuration
#[derive(Debug, Clone)]
pub enum SearchBackend {
    /// SearXNG meta-search engine (privacy-focused, no API key required)
    SearXNG { instance_url: String },
    /// Brave Search API (independent index, requires API key)
    BraveSearch { api_key: String },
}

/// Individual search result from web search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Title of the search result
    pub title: String,
    /// URL of the source
    pub url: String,
    /// Snippet/excerpt of content
    pub snippet: String,
    /// Published date (if available)
    pub published_date: Option<DateTime<Utc>>,
}

/// Custom error types for web search operations
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

    #[error("No results found")]
    NoResults,

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("SearXNG error: {0}")]
    SearXNGError(String),
}

/// Main web search function
///
/// Performs a web search using the specified backend and returns a list of search results.
///
/// # Arguments
/// * `query` - The search query string
/// * `backend` - The search backend to use (SearXNG or Brave Search)
/// * `max_results` - Maximum number of results to return (1-20)
///
/// # Returns
/// * `Ok(Vec<SearchResult>)` - Vector of search results
/// * `Err(SearchError)` - Error if search fails
pub async fn search_web(
    query: &str,
    backend: SearchBackend,
    max_results: usize,
) -> Result<Vec<SearchResult>, SearchError> {
    log::info!("Performing web search: \"{}\" (max: {})", query, max_results);

    // Validate inputs
    if query.trim().is_empty() {
        log::warn!("Empty search query provided");
        return Ok(Vec::new());
    }

    let clamped_max = max_results.clamp(1, 20);

    // Route to appropriate backend
    match backend {
        SearchBackend::SearXNG { instance_url } => {
            search_searxng(query, &instance_url, clamped_max).await
        }
        SearchBackend::BraveSearch { api_key } => {
            search_brave(query, &api_key, clamped_max).await
        }
    }
}

/// Search using SearXNG meta-search engine
async fn search_searxng(
    query: &str,
    instance_url: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>, SearchError> {
    log::info!("Using SearXNG instance: {}", instance_url);

    // Build SearXNG search URL
    // SearXNG API endpoint: /search?q=<query>&format=json&pageno=1
    let search_url = format!("{}/search", instance_url.trim_end_matches('/'));

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| SearchError::Network(e))?;

    // Execute search request
    log::debug!("GET {} (q={}, format=json)", search_url, query);
    let response = client
        .get(&search_url)
        .query(&[
            ("q", query),
            ("format", "json"),
            ("pageno", "1"),
            ("categories", "general"),
        ])
        .send()
        .await
        .map_err(|e| {
            log::error!("SearXNG request failed: {}", e);
            if e.is_timeout() {
                SearchError::BackendUnavailable("SearXNG request timed out".to_string())
            } else if e.is_connect() {
                SearchError::BackendUnavailable(format!(
                    "Could not connect to SearXNG instance: {}",
                    instance_url
                ))
            } else {
                SearchError::Network(e)
            }
        })?;

    // Check HTTP status
    if !response.status().is_success() {
        log::error!("SearXNG returned error status: {}", response.status());
        return Err(SearchError::BackendUnavailable(format!(
            "SearXNG returned status: {}",
            response.status()
        )));
    }

    // Parse JSON response
    #[derive(Deserialize)]
    struct SearXNGResponse {
        results: Vec<SearXNGResult>,
    }

    #[derive(Deserialize)]
    struct SearXNGResult {
        title: String,
        url: String,
        content: Option<String>,
        #[serde(rename = "publishedDate")]
        published_date: Option<String>,
    }

    let searxng_response: SearXNGResponse = response.json().await.map_err(|e| {
        log::error!("Failed to parse SearXNG JSON: {}", e);
        SearchError::SearXNGError(format!("Invalid JSON response: {}", e))
    })?;

    log::info!(
        "SearXNG returned {} results",
        searxng_response.results.len()
    );

    // Convert SearXNG results to our SearchResult format
    let results: Vec<SearchResult> = searxng_response
        .results
        .into_iter()
        .take(max_results)
        .map(|r| SearchResult {
            title: r.title,
            url: r.url,
            snippet: r.content.unwrap_or_else(|| "No snippet available".to_string()),
            published_date: r.published_date.and_then(|d| {
                DateTime::parse_from_rfc3339(&d)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
        })
        .collect();

    if results.is_empty() {
        log::warn!("SearXNG search returned 0 results");
        return Err(SearchError::NoResults);
    }

    log::info!("✓ SearXNG search successful: {} results", results.len());
    Ok(results)
}

/// Search using Brave Search API
async fn search_brave(
    query: &str,
    api_key: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>, SearchError> {
    log::info!("Using Brave Search API");

    // Brave Search API endpoint
    const BRAVE_SEARCH_URL: &str = "https://api.search.brave.com/res/v1/web/search";

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| SearchError::Network(e))?;

    // Execute search request
    log::debug!("GET {} (q={})", BRAVE_SEARCH_URL, query);
    let response = client
        .get(BRAVE_SEARCH_URL)
        .header("Accept", "application/json")
        .header("X-Subscription-Token", api_key)
        .query(&[("q", query), ("count", &max_results.to_string())])
        .send()
        .await
        .map_err(|e| {
            log::error!("Brave Search request failed: {}", e);
            if e.is_timeout() {
                SearchError::BackendUnavailable("Brave Search request timed out".to_string())
            } else if e.is_connect() {
                SearchError::BackendUnavailable(
                    "Could not connect to Brave Search API".to_string(),
                )
            } else {
                SearchError::Network(e)
            }
        })?;

    // Check HTTP status
    match response.status().as_u16() {
        200 => {
            // Success, continue
        }
        401 | 403 => {
            log::error!("Brave Search authentication failed (status 401/403)");
            return Err(SearchError::InvalidApiKey);
        }
        429 => {
            log::error!("Brave Search rate limit exceeded");
            return Err(SearchError::RateLimitExceeded);
        }
        status => {
            log::error!("Brave Search returned error status: {}", status);
            return Err(SearchError::BackendUnavailable(format!(
                "Brave Search returned status: {}",
                status
            )));
        }
    }

    // Parse JSON response
    #[derive(Deserialize)]
    struct BraveSearchResponse {
        web: Option<BraveWebResults>,
    }

    #[derive(Deserialize)]
    struct BraveWebResults {
        results: Vec<BraveResult>,
    }

    #[derive(Deserialize)]
    struct BraveResult {
        title: String,
        url: String,
        description: Option<String>,
        age: Option<String>, // ISO 8601 date string
    }

    let brave_response: BraveSearchResponse = response.json().await.map_err(|e| {
        log::error!("Failed to parse Brave Search JSON: {}", e);
        SearchError::Network(e)
    })?;

    // Extract results
    let results: Vec<SearchResult> = brave_response
        .web
        .and_then(|web| {
            if web.results.is_empty() {
                None
            } else {
                Some(web.results)
            }
        })
        .map(|brave_results| {
            brave_results
                .into_iter()
                .take(max_results)
                .map(|r| SearchResult {
                    title: r.title,
                    url: r.url,
                    snippet: r
                        .description
                        .unwrap_or_else(|| "No description available".to_string()),
                    published_date: r.age.and_then(|d| {
                        DateTime::parse_from_rfc3339(&d)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc))
                    }),
                })
                .collect()
        })
        .ok_or_else(|| {
            log::warn!("Brave Search returned 0 results");
            SearchError::NoResults
        })?;

    log::info!("✓ Brave Search successful: {} results", results.len());
    Ok(results)
}

/// Format search results as contextual text for LLM prompt
///
/// Converts a vector of search results into a structured text block
/// that can be prepended to a user's query for RAG (Retrieval-Augmented Generation).
///
/// # Arguments
/// * `results` - Vector of search results to format
///
/// # Returns
/// Formatted string with search results in a structured format
pub fn format_search_context(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return String::new();
    }

    let mut context = String::from("=== Web Search Results ===\n\n");
    context.push_str("The following information was retrieved from the web to help answer your question:\n\n");

    for (idx, result) in results.iter().enumerate() {
        // Format each result with source citation
        context.push_str(&format!("[Source {}]\n", idx + 1));
        context.push_str(&format!("Title: {}\n", result.title));
        context.push_str(&format!("URL: {}\n", result.url));

        // Add published date if available
        if let Some(date) = result.published_date {
            context.push_str(&format!("Date: {}\n", date.format("%Y-%m-%d")));
        }

        // Add snippet/content
        context.push_str(&format!("Content: {}\n", result.snippet));
        context.push_str("\n");
    }

    context.push_str("=== End of Search Results ===\n\n");
    context.push_str("Please use the above information to answer the user's question accurately. ");
    context.push_str("Cite sources using [Source N] notation when referencing specific information.\n\n");

    context
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_search_context_empty() {
        let results = Vec::new();
        let context = format_search_context(&results);
        assert_eq!(context, "");
    }

    #[test]
    fn test_format_search_context_single_result() {
        let results = vec![SearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Test snippet content".to_string(),
            published_date: None,
        }];

        let context = format_search_context(&results);

        assert!(context.contains("=== Web Search Results ==="));
        assert!(context.contains("[Source 1]"));
        assert!(context.contains("Title: Test Title"));
        assert!(context.contains("URL: https://example.com"));
        assert!(context.contains("Content: Test snippet content"));
        assert!(context.contains("=== End of Search Results ==="));
    }

    #[test]
    fn test_format_search_context_multiple_results() {
        let results = vec![
            SearchResult {
                title: "First Result".to_string(),
                url: "https://first.com".to_string(),
                snippet: "First snippet".to_string(),
                published_date: None,
            },
            SearchResult {
                title: "Second Result".to_string(),
                url: "https://second.com".to_string(),
                snippet: "Second snippet".to_string(),
                published_date: None,
            },
        ];

        let context = format_search_context(&results);

        assert!(context.contains("[Source 1]"));
        assert!(context.contains("[Source 2]"));
        assert!(context.contains("First Result"));
        assert!(context.contains("Second Result"));
    }
}
