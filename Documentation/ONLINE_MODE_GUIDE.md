# Online Mode User Guide

## Overview

**Online Mode** enables Aura to search the web for real-time information to answer your questions. This feature uses Retrieval-Augmented Generation (RAG) to combine web search results with Aura's local AI capabilities, dramatically expanding its knowledge beyond its training data.

**Privacy First:** This feature is **disabled by default** and requires your explicit consent to enable.

## What is Online Mode?

When you enable Online Mode, Aura can:

✅ Answer questions about current events
✅ Provide up-to-date information on recent topics
✅ Access facts beyond its training data cutoff
✅ Cite sources from the web

**How it works:**
1. You ask Aura a question
2. If Online Mode is enabled, Aura searches the web using your chosen search provider
3. Search results are formatted as context and provided to the local LLM
4. The LLM uses this context to generate an accurate, up-to-date answer
5. If the search fails or is disabled, Aura falls back to its built-in knowledge

## Enabling Online Mode

### Step 1: Open Settings

1. Launch Aura Desktop
2. Click the **Settings** icon (⚙️) in the sidebar
3. Scroll to the **Online Mode** section

### Step 2: Review Privacy Notice

Read the privacy notice carefully:

> **Privacy Notice**
>
> When enabled, Aura will connect to the internet to search for current information. Your queries will be sent to the selected search provider. This feature is disabled by default and requires your explicit consent.

### Step 3: Enable Online Mode

1. Toggle **"Enable Online Mode"** to ON
2. The toggle will turn green/blue when enabled
3. Additional configuration options will appear

### Step 4: Choose Search Provider

Aura supports two search backends:

#### **SearXNG (Recommended for Privacy)**

- **Privacy:** Best (no tracking, no data retention if self-hosted)
- **API Key:** Not required
- **Cost:** Free
- **Setup:** Just enter instance URL

**Default Instance:** `https://searx.be`

**Other Public Instances:**
- `https://searx.org`
- `https://search.sapti.me`
- `https://searx.fmac.xyz`

**Self-Hosted:** You can run your own SearXNG instance for maximum privacy. See [SearXNG Documentation](https://docs.searxng.org/)

#### **Brave Search**

- **Privacy:** Good (90-day retention, no user tracking)
- **API Key:** Required
- **Cost:** Free tier (2,000 queries/month), then $3 per 1,000 queries
- **Setup:** Register at [brave.com/search/api](https://brave.com/search/api)

### Step 5: Configure Settings

**For SearXNG:**
1. Select **"SearXNG"** from the dropdown
2. Enter instance URL (default: `https://searx.be`)
3. Adjust **Max Search Results** (1-20, default: 5)

**For Brave Search:**
1. Select **"Brave Search"** from the dropdown
2. Enter your **Brave Search API Key**
3. Adjust **Max Search Results** (1-20, default: 5)

### Step 6: Save and Test

1. Click **"Save Settings"**
2. Try asking a current events question:
   - "What's the latest news about SpaceX?"
   - "What are the current COVID-19 statistics?"
   - "What happened in the stock market today?"

## Using Online Mode

### When to Use Online Mode

✅ **Use Online Mode for:**
- Current events and news
- Recent developments in technology, science, politics
- Real-time data (stock prices, sports scores, weather)
- Facts that change frequently
- Topics after the AI's training cutoff date

❌ **Don't need Online Mode for:**
- General knowledge questions
- Historical facts
- Programming concepts
- Math and logic problems
- Personal conversations

### How to Tell if Online Mode is Active

When you send a message with Online Mode enabled:

1. **In Logs:** Check application logs for:
   ```
   [INFO] Online mode enabled, performing web search for RAG...
   [INFO] ✓ Web search successful: 5 results found
   ```

2. **In Responses:** The LLM may include source citations like:
   ```
   According to recent web search results [Source 1], ...
   ```

3. **Settings Indicator:** The Settings modal shows Online Mode as enabled

### Understanding Search Results

When Online Mode performs a search:

- **5 results by default** (configurable 1-20)
- Results are **ranked by relevance** by the search engine
- Each result includes:
  - Title
  - URL (source)
  - Snippet (preview text)
  - Published date (when available)

The LLM receives these results as context and uses them to answer your question.

## Privacy Considerations

### What Data is Sent?

When you ask a question with Online Mode enabled:

**Sent to Search Provider:**
- Your question text
- Your IP address (standard web request)

**NOT Sent:**
- Previous conversation history
- Your name or personal information
- Other settings or data from Aura

### Data Retention

| Provider | Data Retention | Tracking | Anonymous |
|----------|---------------|----------|-----------|
| **SearXNG (public)** | Varies by instance | No | Yes |
| **SearXNG (self-hosted)** | You control | No | Yes |
| **Brave Search** | 90 days (for billing) | No | Yes |

### Privacy Best Practices

🔒 **Maximum Privacy:**
- Use a **self-hosted SearXNG** instance
- Run SearXNG behind a VPN or Tor
- Only enable Online Mode when needed

🔒 **High Privacy:**
- Use a **trusted public SearXNG** instance (e.g., searx.be)
- Review the instance's privacy policy
- Disable Online Mode after use

🔒 **Good Privacy:**
- Use **Brave Search** with their privacy-focused API
- Be aware of 90-day retention
- Brave does not track or profile users

### Disabling Online Mode

To disable:

1. Open **Settings**
2. Toggle **"Enable Online Mode"** to OFF
3. Save settings

Aura will immediately stop sending queries to search providers.

## Troubleshooting

### "Web search failed, falling back to offline mode"

**Causes:**
- No internet connection
- Search provider is down
- API key is invalid (Brave Search)
- Rate limit exceeded

**Solutions:**
1. Check your internet connection
2. Try a different SearXNG instance or switch to Brave
3. Verify your Brave API key is correct
4. Wait if rate limited (Brave: 2,000/month on free tier)

### "Brave Search selected but no API key configured"

**Solution:**
1. Go to Settings → Online Mode
2. Enter your Brave Search API key
3. Get a free key at: https://brave.com/search/api

### Search returns 0 results

**What happens:**
- Aura logs: `⚠ Web search returned 0 results, using offline mode`
- The LLM answers using built-in knowledge

**Why it happens:**
- Query is too specific or unusual
- Search engine couldn't find relevant results
- SearXNG instance has limited indexed content

**Solutions:**
- Rephrase your question to be more general
- Try a different SearXNG instance
- Switch to Brave Search (30B+ page index)

### Slow response times

**Normal behavior:**
- Web search adds 2-5 seconds to response time
- Total time: 5-10 seconds (search + LLM)

**If slower than 10 seconds:**
- SearXNG instance might be slow (try another)
- Check your internet speed
- Reduce Max Search Results to 3

### SearXNG instance is unreachable

**Symptoms:**
```
[ERROR] Could not connect to SearXNG instance: https://...
```

**Solutions:**
1. Verify the URL is correct (must be HTTPS)
2. Try a different public instance:
   - `https://searx.org`
   - `https://search.sapti.me`
3. Check if the instance is online: visit URL in web browser
4. Switch to Brave Search as alternative

## Advanced Configuration

### Max Search Results

**Range:** 1-20
**Default:** 5
**Recommendation:** 5-8 for best balance

**Considerations:**
- **More results (10-20):**
  - More context for LLM
  - Better accuracy for complex questions
  - Slower response time (more data to process)
  - Higher LLM context usage

- **Fewer results (1-3):**
  - Faster response time
  - Less context (may miss important info)
  - Lower LLM context usage

### Custom SearXNG Instances

You can use any SearXNG instance that provides JSON API access.

**Requirements:**
- Instance must support `/search?format=json`
- HTTPS recommended (HTTP will work but less secure)
- Instance must allow API requests (some block them)

**Testing an instance:**
1. Visit: `https://[instance-url]/search?q=test&format=json`
2. Should return JSON with `results` array
3. If working, use this URL in Aura settings

### Self-Hosting SearXNG

For maximum privacy, run your own SearXNG instance:

**Quick Start (Docker):**
```bash
docker run -d -p 8080:8080 \
  --name searxng \
  searxng/searxng:latest
```

**Then in Aura Settings:**
- SearXNG Instance URL: `http://localhost:8080`
- Enable Online Mode
- Save

**Full Documentation:** https://docs.searxng.org/

### Brave Search API Key Management

**Getting a Free API Key:**
1. Visit: https://brave.com/search/api/
2. Sign up for an account
3. Navigate to API Dashboard
4. Generate a new API key
5. Copy and paste into Aura Settings

**Free Tier Limits:**
- 2,000 queries per month
- Resets on the 1st of each month
- Upgrade: $3 per 1,000 additional queries

**Key Security:**
- Stored securely in Aura's database
- Not logged or transmitted except to Brave API
- Can be deleted in Settings at any time

## Frequently Asked Questions

### Is Online Mode required to use Aura?

No. Aura works perfectly offline. Online Mode is an optional enhancement for current events and real-time data.

### Does this violate Aura's "offline-first" principle?

No. Aura remains 100% offline-capable for all core functions. Online Mode is:
- **Optional** (disabled by default)
- **Explicit opt-in** (requires user consent)
- **Graceful fallback** (works without internet)
- **Privacy-focused** (uses privacy-respecting search providers)

### Can I use multiple search providers?

Not simultaneously, but you can switch between SearXNG and Brave Search in Settings at any time.

### What happens if my internet connection drops?

Aura will log a warning and fall back to offline mode for that query. No error is shown to the user.

### Does this use more battery/CPU?

Minimally. The web search is handled by external servers. The only additional processing is formatting search results as context for the LLM, which is negligible.

### Can I see what search queries are sent?

Yes. Enable debug logging:
```bash
RUST_LOG=info ./aura-desktop
```

Look for log entries like:
```
[INFO] Performing web search: "your question here"
```

### Does this work with voice input?

Yes! If you ask a question via voice and Online Mode is enabled, Aura will perform a web search just like text input.

### Can I disable Online Mode for specific conversations?

Currently no. Online Mode is a global setting. When enabled, it applies to all queries. Disable it in Settings to turn it off.

### How does this compare to ChatGPT's web browsing?

Similar concept, but different implementation:

**Aura + Online Mode:**
- Privacy-focused (no data sent to OpenAI/Anthropic)
- Local LLM processes search results
- Full control over search provider
- Transparent source citations
- 100% local execution

**ChatGPT Web Browsing:**
- Requires OpenAI account and subscription
- Data sent to OpenAI servers
- Uses Bing search
- Proprietary implementation
- Requires internet for all operations

## Example Queries

### Current Events

**Query:** "What's the latest news about the James Webb Space Telescope?"

**With Online Mode:**
```
Aura: According to recent reports [Source 1], the James Webb Space Telescope
discovered the most distant galaxy ever observed, dating back to just
300 million years after the Big Bang. NASA announced this finding on
October 5th, 2025 [Source 2]...
```

**Without Online Mode:**
```
Aura: The James Webb Space Telescope was launched on December 25, 2021.
It's designed to observe the universe in infrared wavelengths. I don't have
information about recent discoveries beyond my training data.
```

### Real-Time Data

**Query:** "What's the current price of Bitcoin?"

**With Online Mode:**
```
Aura: As of today [Source 1], Bitcoin is trading at $42,350, up 3.2%
from yesterday. The 24-hour high was $43,100 and the low was $41,500
[Source 2]...
```

**Without Online Mode:**
```
Aura: I don't have access to real-time cryptocurrency prices. My training
data ended in January 2025, so I cannot provide current Bitcoin prices.
You may want to check a cryptocurrency exchange or financial website.
```

### Recent Developments

**Query:** "What are the new features in Python 3.13?"

**With Online Mode:**
```
Aura: Python 3.13, released in October 2024, introduced several new features:
1. Improved error messages with better tracebacks [Source 1]
2. Enhanced performance with the new JIT compiler [Source 2]
3. New syntax for pattern matching improvements [Source 3]
...
```

**Without Online Mode:**
```
Aura: Python 3.13 hasn't been released yet according to my training data
(last updated January 2025). The latest stable version I'm aware of is
Python 3.12, which included...
```

## Support

If you encounter issues with Online Mode:

1. **Check this guide** for troubleshooting steps
2. **Review logs** with `RUST_LOG=info`
3. **Test search provider** in a web browser
4. **Try alternative provider** (switch between SearXNG and Brave)
5. **Report bugs** on GitHub: https://github.com/nivora-ai/aura-desktop/issues

## Summary

Online Mode empowers Aura to answer questions about current events and real-time data while maintaining privacy through carefully selected search providers:

✅ **Privacy-First Design:** Disabled by default, explicit opt-in
✅ **Flexible Providers:** Choose SearXNG (maximum privacy) or Brave Search (reliable)
✅ **Graceful Fallback:** Always works offline
✅ **Transparent:** Clear logging and source citations
✅ **User Control:** Easy to enable/disable anytime

---

**Last Updated:** October 2025
**Feature Version:** 1.0
**Aura Desktop Version:** 0.1.0+
