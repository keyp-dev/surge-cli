# Surge TUI - HTTP Headers/Body Capture Solution (Scripting Integration)

**Status:** Draft (for future reference)
**Date:** 2026-02-16
**Priority:** P2 (Non-urgent)

---

## Problem

Currently, Surge TUI can only access request metadata (URL, policy, traffic statistics) through HTTP API v1, but cannot view HTTP request/response headers and body.

### User Requirements

When MITM/Capture is enabled, debugging APIs requires viewing:
- Request headers (User-Agent, Authorization, Content-Type, etc.)
- Response headers (Set-Cookie, Cache-Control, Content-Type, etc.)
- Request body (POST data, JSON payload)
- Response body (API JSON responses, HTML, etc.)

### Current Limitations

**HTTP API v1 does not provide this data:**
- `GET /v1/requests/recent` only returns metadata
- `GET /v1/requests/active` only has metadata
- No endpoint to get detailed HTTP data for individual requests

**But Surge Scripting API can access:**
- `$request.headers<Object>` - Complete headers
- `$request.body<String|Uint8Array>` - Complete body
- `$response.headers<Object>` - Response headers
- `$response.body` - Response body

---

## Solution

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Surge Configuration File                  │
│  [Script]                                                   │
│  surge-tui-capture = type=http-request,                     │
│                      pattern=^https?://,                    │
│                      script-path=capture.js,                │
│                      requires-body=true                     │
│                                                             │
│  surge-tui-response = type=http-response,                   │
│                       pattern=^https?://,                   │
│                       script-path=capture-response.js,      │
│                       requires-body=true                    │
└─────────────────────────────────────────────────────────────┘
         │
         │ Intercept all requests/responses
         ↓
┌─────────────────────────────────────────────────────────────┐
│                      capture.js                             │
│  // Extract headers and body                                │
│  let data = {                                               │
│    id: $request.id,                                         │
│    url: $request.url,                                       │
│    method: $request.method,                                 │
│    headers: $request.headers,                               │
│    body: $request.body                                      │
│  };                                                         │
│  // Write to persistent storage                            │
│  $persistentStore.write(JSON.stringify(data), $request.id);│
│  console.log("Captured request: " + $request.id);          │
│  $done({});                                                │
└─────────────────────────────────────────────────────────────┘
         │
         │ Write to file
         ↓
┌─────────────────────────────────────────────────────────────┐
│  ~/Library/Application Support/                             │
│    com.nssurge.surge-mac/SGJSVMPersistentStore/            │
│                                                             │
│  <request-id-1>                                            │
│  <request-id-2>                                            │
│  ...                                                       │
└─────────────────────────────────────────────────────────────┘
         │
         │ Read files
         ↓
┌─────────────────────────────────────────────────────────────┐
│                      Surge TUI                              │
│  1. Get request list from HTTP API (includes request.id)   │
│  2. Read ~/Library/.../SGJSVMPersistentStore/<id>          │
│  3. Parse JSON to get headers/body                         │
│  4. Display in detail panel                                │
└─────────────────────────────────────────────────────────────┘
```

---

## Key Decisions

### Decision 1: Use Surge Scripting + Persistent Storage

**Choice:** Intercept requests via Surge scripts, write to `$persistentStore`, TUI reads from filesystem

**Rationale:**
- Surge Scripting API is the only official way to access headers/body
- `$persistentStore` writes to local filesystem, TUI can read directly
- No need to modify Surge itself, pure configuration solution

**Cost:**
- Requires manual user configuration of scripts (high complexity)
- Script execution overhead per request
- Large storage usage (if capturing all requests)
- Need periodic cleanup of old data

**Reconsider When:**
- Surge officially provides HTTP API v2 with headers/body support
- Performance testing shows script overhead > 10% latency increase

---

### Decision 2: Only Capture User-Interested Requests

**Choice:** Don't capture all requests, only those matching specific patterns

**Rationale:**
- Reduces performance overhead and storage usage
- Users typically only care about specific domains or APIs
- Flexibly adjustable via configuration

**Cost:**
- Requires user configuration of capture rules (pattern)
- Misses requests outside the pattern

**Implementation:**
```ini
[Script]
# Only capture specific domain
surge-tui-capture-api = type=http-request,pattern=^https://api\.example\.com,script-path=capture.js,requires-body=true

# Or capture all HTTPS
surge-tui-capture-all = type=http-request,pattern=^https://,script-path=capture.js,requires-body=true
```

---

### Decision 3: Data Expiration Strategy

**Choice:** Keep detailed data for latest 100 requests, automatic cleanup

**Rationale:**
- Prevents unlimited storage growth
- 100 requests sufficient for debugging
- Corresponds to TUI's displayed request count (50 recent + 50 active)

**Implementation:**
- TUI startup cleans files > 1 hour old
- Or use LRU strategy, keep latest 100

---

## Technical Specifications

### 1. Surge Scripts

#### capture.js (Request Capture)

```javascript
// ==Script==
// @name Surge TUI Request Capture
// @description Capture HTTP request headers and body for Surge TUI
// @author Claude with Happy
// ==Script==

let data = {
  id: $request.id,
  url: $request.url,
  method: $request.method,
  headers: $request.headers,
  body: null,
  timestamp: Date.now(),
  type: 'request'
};

// Capture body (if exists)
if ($request.body) {
  // Try to parse as text
  if (typeof $request.body === 'string') {
    data.body = $request.body;
  } else if ($request.body instanceof Uint8Array) {
    // Binary data to base64
    data.body = btoa(String.fromCharCode.apply(null, $request.body));
    data.bodyEncoding = 'base64';
  }
}

// Write to persistent storage
try {
  $persistentStore.write(JSON.stringify(data), $request.id);
  console.log(`[Surge TUI] Captured request ${$request.id}: ${$request.method} ${$request.url}`);
} catch (e) {
  console.log(`[Surge TUI] Failed to capture request ${$request.id}: ${e}`);
}

// Don't modify request, pass through
$done({});
```

#### capture-response.js (Response Capture)

```javascript
// ==Script==
// @name Surge TUI Response Capture
// @description Capture HTTP response headers and body for Surge TUI
// @author Claude with Happy
// ==Script==

let data = {
  id: $request.id, // Use same ID to correlate request and response
  status: $response.status,
  headers: $response.headers,
  body: null,
  timestamp: Date.now(),
  type: 'response'
};

// Capture body
if ($response.body) {
  if (typeof $response.body === 'string') {
    data.body = $response.body;
  } else if ($response.body instanceof Uint8Array) {
    data.body = btoa(String.fromCharCode.apply(null, $response.body));
    data.bodyEncoding = 'base64';
  }
}

// Write to storage (append to request data)
try {
  // Read request data
  let requestData = $persistentStore.read($request.id);
  if (requestData) {
    let combined = JSON.parse(requestData);
    combined.response = data;
    $persistentStore.write(JSON.stringify(combined), $request.id);
  } else {
    // If request data doesn't exist, save response separately
    $persistentStore.write(JSON.stringify(data), $request.id + '_response');
  }
  console.log(`[Surge TUI] Captured response ${$request.id}: ${$response.status}`);
} catch (e) {
  console.log(`[Surge TUI] Failed to capture response ${$request.id}: ${e}`);
}

$done({});
```

### 2. TUI Read Logic

#### infrastructure/persistent_store.rs (New File)

```rust
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedRequest {
    pub id: String,
    pub url: String,
    pub method: String,
    pub headers: serde_json::Map<String, serde_json::Value>,
    pub body: Option<String>,
    #[serde(rename = "bodyEncoding")]
    pub body_encoding: Option<String>,
    pub timestamp: u64,
    pub response: Option<CapturedResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedResponse {
    pub status: u32,
    pub headers: serde_json::Map<String, serde_json::Value>,
    pub body: Option<String>,
    #[serde(rename = "bodyEncoding")]
    pub body_encoding: Option<String>,
}

pub struct PersistentStoreClient {
    base_path: PathBuf,
}

impl PersistentStoreClient {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let base_path = PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("com.nssurge.surge-mac")
            .join("SGJSVMPersistentStore");

        Self { base_path }
    }

    pub fn get_captured_request(&self, request_id: u64) -> Result<Option<CapturedRequest>> {
        let file_path = self.base_path.join(request_id.to_string());

        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(file_path)?;
        let captured: CapturedRequest = serde_json::from_str(&content)?;
        Ok(Some(captured))
    }

    pub fn cleanup_old_files(&self, max_age_secs: u64) -> Result<usize> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let mut deleted = 0;

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let file_age = now - modified
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs();

                    if file_age > max_age_secs {
                        fs::remove_file(entry.path())?;
                        deleted += 1;
                    }
                }
            }
        }

        Ok(deleted)
    }
}
```

#### application/surge_client.rs (Integration)

```rust
// Add new method
pub async fn get_request_details(&self, request_id: u64) -> Result<Option<CapturedRequest>> {
    let store = PersistentStoreClient::new();
    store.get_captured_request(request_id)
}
```

#### UI Display

In `requests.rs` detail panel:

```rust
// Try to get detailed information
if let Ok(Some(details)) = app.client.get_request_details(request.id).await {
    // Display Request Headers
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Request Headers:", Style::default().bold()),
    ]));
    for (key, value) in &details.headers {
        lines.push(Line::from(format!("  {}: {}", key, value)));
    }

    // Display Request Body
    if let Some(ref body) = details.body {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Request Body:", Style::default().bold()),
        ]));
        lines.push(Line::from(body.chars().take(500).collect::<String>()));
    }

    // Display Response
    if let Some(ref response) = details.response {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("Response: {}", response.status),
                Style::default().bold().fg(Color::Green)
            ),
        ]));

        // Response Headers
        for (key, value) in &response.headers {
            lines.push(Line::from(format!("  {}: {}", key, value)));
        }

        // Response Body
        if let Some(ref body) = response.body {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Response Body:", Style::default().bold()),
            ]));
            lines.push(Line::from(body.chars().take(500).collect::<String>()));
        }
    }
}
```

---

## Known Issues

### Critical

**Issue 1: High User Configuration Complexity**

- **Phenomenon:** Requires manual editing of Surge configuration file to add scripts
- **Impact:** Non-technical users find it difficult to configure
- **Workaround:** Provide detailed documentation and configuration templates
- **Fix Plan:** Consider automated configuration tools

**Issue 2: Performance Overhead**

- **Phenomenon:** Script execution per request may add latency
- **Impact:** May affect browsing speed with high request volume
- **Workaround:** Only capture requests matching specific patterns
- **Fix Plan:** Performance testing to evaluate actual overhead

### Important

**Issue 3: Storage Space Usage**

- **Phenomenon:** Capturing many requests occupies significant storage
- **Impact:** Long-running may consume hundreds of MB
- **Workaround:** TUI startup automatically cleans files > 1 hour old
- **Fix Plan:** Implement LRU cache strategy, keep latest 100

**Issue 4: Base64 Encoding Binary Data**

- **Phenomenon:** Images, videos, etc. stored as base64
- **Impact:** 33% storage overhead, unfriendly display
- **Workaround:** Only display size and type for binary data
- **Fix Plan:** Intelligent handling based on Content-Type

### Nice to have

**Issue 5: Unable to Sync request_id**

- **Phenomenon:** Surge script's `$request.id` format unknown, may differ from API's `id`
- **Impact:** May fail to correlate requests correctly
- **Workaround:** Use URL + timestamp combination for matching
- **Fix Plan:** Test to verify if ID formats match

---

## Implementation Plan

### Phase 1: Feasibility Validation (1-2 hours)

- [ ] Write simple capture.js script
- [ ] Configure in Surge and test
- [ ] Verify `$persistentStore` file location and format
- [ ] Confirm if `$request.id` matches API's `id`

### Phase 2: Complete Script Implementation (2-3 hours)

- [ ] Implement capture.js (request capture)
- [ ] Implement capture-response.js (response capture)
- [ ] Handle base64 encoding
- [ ] Add error handling and logging

### Phase 3: TUI Integration (3-4 hours)

- [ ] Implement `PersistentStoreClient`
- [ ] Integrate into `SurgeClient`
- [ ] UI display headers and body
- [ ] Add data cleanup logic

### Phase 4: Documentation and Optimization (1-2 hours)

- [ ] Write user configuration documentation
- [ ] Provide configuration templates
- [ ] Performance testing and optimization
- [ ] Update FEATURES.md

**Total Estimated Time:** 7-11 hours

---

## Alternative Solutions

### Option A: Wait for Surge HTTP API v2

**Wait for official support:** If Surge releases API v2 in the future, it may directly support headers/body access.

**Pros:** Zero configuration, directly usable
**Cons:** Uncertain timeline, may never happen

### Option B: Integrate mitmproxy

**Use independent capture tool:** Run mitmproxy with Surge as upstream proxy.

**Pros:** Complete HTTP debugging capabilities
**Cons:** Complex architecture, requires running additional process

---

## Acceptance Criteria

- [ ] After user configures script, TUI displays request headers
- [ ] Can display response headers and status code
- [ ] Can display request/response body (text)
- [ ] Binary data displays as type and size
- [ ] Automatically cleans old data
- [ ] Performance overhead < 10% latency increase
- [ ] Provides complete configuration documentation

---

## References

- [Surge Scripting - HTTP Request](https://manual.nssurge.com/scripting/http-request.html)
- [Surge Scripting - Common Functions](https://manual.nssurge.com/scripting/common.html)
- Surge persistent storage path: `~/Library/Application Support/com.nssurge.surge-mac/SGJSVMPersistentStore/`

---

*Generated with [Claude Code](https://claude.ai/code) via [Happy](https://happy.engineering)*

*Co-Authored-By: Claude <noreply@anthropic.com>*
