# Surge CLI / HTTP API Technical Research Report

**Research Date**: 2026-02-14
**Research Purpose**: Provide technical foundation for SSH remote control of Surge

---

## TL;DR

Surge provides two remote control methods:
1. **CLI tool** (`surge-cli`) - Direct command-line operations, suitable for interactive scenarios
2. **HTTP API** - RESTful interface, suitable for programmatic control and UI development

Both have almost complete functional overlap, CLI is a wrapper around HTTP API. **Recommend building TUI based on HTTP API for more flexibility.**

---

## 1. CLI Tool Overview

### 1.1 Installation Location
```bash
/Applications/Surge.app/Contents/Applications/surge-cli
```

### 1.2 Remote Call Capability
Supports remote operations via `--remote/-r` parameter:
```bash
surge-cli --remote password@host:port <command>
```

**Limitation**: Requires HTTP API enabled in Surge configuration (see below)

### 1.3 Core Command Categories

| Category | Command | Description |
|----------|---------|-------------|
| **Configuration Management** | `reload` | Reload main config file |
| | `switch-profile <name>` | Switch to specified profile |
| **Status Viewing** | `dump active` | Active connection list |
| | `dump request` | Recent request history |
| | `dump rule` | Currently effective rules |
| | `dump policy` | All policies and policy groups |
| | `dump dns` | DNS cache |
| | `dump profile [original/effective]` | View config file |
| | `dump event` | Event logs |
| **Testing Tools** | `test-network` | Test network latency |
| | `test-policy <name>` | Test single proxy |
| | `test-all-policies` | Test all proxies |
| | `test-group <name>` | Retest policy group |
| **Monitoring** | `watch request` | Real-time track new requests |
| **Maintenance** | `flush dns` | Flush DNS cache |
| | `kill <id>` | Terminate specified connection |
| | `stop` | Stop Surge |
| **Diagnostics** | `diagnostics` | Run network diagnostics |
| | `set-log-level <level>` | Temporarily modify log level |

### 1.4 Output Format
- **Default**: Human-readable formatted text
- **`--raw` parameter**: Output JSON format, suitable for programmatic parsing

**Example**:
```bash
# Human readable
$ surge-cli dump active
Active Connections:
  #1 TCP 192.168.1.100:54321 → google.com:443 [Proxy]

# JSON format
$ surge-cli --raw dump active
{"connections":[{"id":1,"protocol":"TCP","src":"192.168.1.100:54321",...}]}
```

---

## 2. HTTP API Details

### 2.1 Enable Configuration
Add in Surge configuration file `[General]`:
```ini
[General]
http-api = your-secret-key@127.0.0.1:6170
http-api-tls = false  # Optional, enable HTTPS
```

**Security**:
- Default listens on `127.0.0.1`, local access only (recommended)
- Can set `0.0.0.0` to allow remote access (requires firewall coordination)
- `http-api-tls = true` enables HTTPS, uses MITM CA-signed certificate
- **Default port**: 6170 (non-standard port, avoids conflicts)

### 2.2 Authentication Methods
Choose one of two methods:
1. **HTTP Header**: `X-Key: your-secret-key`
2. **URL Parameter**: `?x-key=your-secret-key`

### 2.3 Request Specification
- **Only supports GET / POST**
- GET: Parameters passed via URL Query
- POST: Parameters passed via JSON Body
- **Response**: Always returns JSON

### 2.4 Complete Endpoint List

#### 2.4.1 Feature Toggles (GET/POST)
| Endpoint | Description | Platform Limitation |
|----------|-------------|-------------------|
| `/v1/features/mitm` | MITM feature | - |
| `/v1/features/capture` | Traffic capture | - |
| `/v1/features/rewrite` | URL rewrite | - |
| `/v1/features/scripting` | Scripting feature | - |
| `/v1/features/system_proxy` | System proxy | macOS |
| `/v1/features/enhanced_mode` | Enhanced mode | macOS |

**GET Response**:
```json
{"enabled": true}
```

**POST Request**:
```json
{"enabled": false}
```

#### 2.4.2 Outbound Mode (GET/POST)
- `GET /v1/outbound` - Returns current mode: `direct` / `proxy` / `rule`
- `POST /v1/outbound` - Switch mode, JSON Body: `{"mode": "rule"}`
- `GET /v1/outbound/global` - Get global default policy
- `POST /v1/outbound/global` - Modify global policy, JSON: `{"policy": "Proxy"}`

#### 2.4.3 Policy Management (GET/POST)
| Endpoint | Method | Parameters | Description |
|----------|--------|------------|-------------|
| `/v1/policies` | GET | - | List all policies |
| `/v1/policies/detail` | GET | `?policy_name=X` | Get policy details (⚠️ returns config text) |
| `/v1/policies/test` | POST | `{"policy_names":["X"], "url":"..."}` | Test policies |
| `/v1/policy_groups` | GET | - | List all policy groups |
| `/v1/policy_groups/select` | POST | `{"group_name":"X", "policy":"Y"}` | Switch policy group selection |
| `/v1/policy_groups/test` | POST | `{"group_name":"X"}` | Immediately test policy group |

**⚠️ Actual API Response Format Differs from Documentation**

`GET /v1/policies` actual response:
```json
{
  "proxies": ["Proxy1", "Proxy2", "..."],
  "policy-groups": ["Group1", "Group2", "..."]
}
```

`GET /v1/policy_groups` actual response (HashMap, not array):
```json
{
  "PolicyGroup1": [
    {
      "isGroup": false,
      "name": "Proxy1",
      "typeDescription": "Shadowsocks",
      "lineHash": "abc123",
      "enabled": true
    }
  ],
  "PolicyGroup2": [...]
}
```

**Note**: `/v1/policies/detail` returns config file fragment (text), not structured JSON. In practice, policy info should be extracted from `policy_groups`.

#### 2.4.4 Request Management (GET/POST)
- `GET /v1/requests/recent` - Recent request list (default 100 entries)
- `GET /v1/requests/active` - Currently active connections
- `POST /v1/requests/kill` - Terminate connection, JSON: `{"id": 123}`

**Actual Request Object Structure** (field names use camelCase):
```json
{
  "id": 123,
  "processPath": "/usr/bin/curl",
  "rule": "DOMAIN,google.com,Proxy",
  "policyName": "Proxy",            // ⚠️ Note field name
  "remoteHost": "google.com:443",
  "URL": "https://google.com",
  "method": "CONNECT",
  "status": "Completed",
  "startDate": 1708000740.5,        // Unix timestamp (seconds + decimal)
  "inBytes": 2048,                  // ⚠️ Note field name (download)
  "outBytes": 1024,                 // ⚠️ Note field name (upload)
  "completed": true,
  "failed": false
}
```

**Key Field Name Differences**:
- `policy` → `policyName`
- `upload` → `outBytes`
- `download` → `inBytes`
- `remote_address` → `remoteHost`
- `start_time` → `startDate` (Unix timestamp)

#### 2.4.5 DNS (GET/POST)
- `POST /v1/dns/flush` - Flush DNS cache
- `GET /v1/dns` - Get DNS cache contents
- `POST /v1/test/dns_delay` - Test DNS latency

#### 2.4.6 Configuration Files (GET/POST)
- `GET /v1/profiles/current?sensitive=0` - Get current config (`sensitive=1` shows sensitive info)
- `POST /v1/profiles/reload` - Reload configuration
- `POST /v1/profiles/switch` - Switch profile (macOS), JSON: `{"name": "work"}`

#### 2.4.7 Scripts & Modules
- `POST /v1/scripting/evaluate` - Execute script test
- `GET /v1/modules` - List module status
- `POST /v1/modules` - Enable/disable module, JSON: `{"module_name": "X", "enabled": true}`

#### 2.4.8 Other
- `GET /v1/mitm/ca` - Download MITM CA certificate (DER binary format)
- `POST /v1/stop` - Stop Surge engine
- `GET /v1/events` - Get event center contents
- `GET /v1/traffic` - Traffic statistics (supported in some versions)

---

## 3. CLI vs HTTP API Comparison

| Dimension | CLI | HTTP API | Recommended Scenario |
|-----------|-----|----------|---------------------|
| **Call Method** | Shell command | HTTP request | API for programmatic |
| **Output Format** | Text/JSON (--raw) | JSON | API more unified |
| **Remote Access** | Via `-r` parameter | Direct HTTP | API more direct |
| **Feature Coverage** | ~90% | 100% | API more comprehensive |
| **Real-time Monitoring** | `watch request` | Requires polling | CLI more convenient |
| **Debug Friendly** | Direct execution | Need to construct requests | CLI faster |
| **UI Development** | Need to parse text | Native JSON | API more suitable |

**Key Differences**:
1. CLI's `watch request` can continuously stream output, HTTP API can only poll
2. HTTP API provides traffic statistics, module management and other features CLI lacks
3. CLI depends on HTTP API for operation (calls API under the hood)

---

## 4. Remote Access Security Considerations

### 4.1 Risk Assessment
| Risk | Severity | Mitigation |
|------|----------|------------|
| API Key Leakage | **Critical** | Use strong key (32+ characters), rotate regularly |
| Man-in-the-Middle Attack | High | Enable `http-api-tls = true` |
| Brute Force | Medium | No built-in rate limiting, rely on firewall |
| Unauthorized Access | Critical | Only listen on `127.0.0.1`, access via SSH tunnel |

### 4.2 Recommended Configuration
**Option 1: SSH Tunnel (Most Secure)**
```ini
[General]
http-api = strong-random-key-32-chars@127.0.0.1:6171
http-api-tls = false  # SSH already encrypted, no need for duplicate encryption
```

Local forwarding:
```bash
ssh -L 6171:127.0.0.1:6171 user@mac-host
curl -H "X-Key: strong-random-key-32-chars" http://127.0.0.1:6171/v1/outbound
```

**Option 2: Direct Exposure (Requires Firewall)**
```ini
[General]
http-api = strong-random-key-32-chars@0.0.0.0:6171
http-api-tls = true  # Must enable TLS
```

Limit source IP:
```bash
# macOS firewall rules
sudo pfctl -e
echo "block drop in on en0 proto tcp from any to any port 6171" | sudo pfctl -f -
echo "pass in on en0 proto tcp from 192.168.1.100 to any port 6171" | sudo pfctl -f -
```

---

## 5. Test Verification

### 5.1 Verify HTTP API Availability
```bash
# Check if Surge is running
ps aux | grep Surge

# Test local API (default port 6170)
curl -H "X-Key: your-key" http://127.0.0.1:6170/v1/outbound

# Expected response
{"mode": "rule"}

# Test policy list (verify actual format)
curl -H "X-Key: your-key" http://127.0.0.1:6170/v1/policies

# Expected response format
{"proxies": ["Proxy1", "Proxy2"], "policy-groups": ["Group1"]}
```

### 5.2 Verify Remote Access
```bash
# Establish SSH tunnel
ssh -L 6171:127.0.0.1:6171 user@remote-mac -N &

# Test remote API
curl -H "X-Key: your-key" http://127.0.0.1:6171/v1/policies

# Expected: Returns policy list JSON
```

### 5.3 Verify CLI Remote Call
```bash
# Execute on local machine
surge-cli --remote your-key@remote-mac:6171 dump active

# Expected: Display remote Mac's active connections
```

---

## 6. Limitations & Notes

### 6.1 Known Limitations
1. **No WebSocket Support** - Cannot push events in real-time, can only poll
2. **No Batch Operations** - Each policy test requires separate call
3. **No Permission Levels** - API Key has full permissions (all or nothing)
4. **No Audit Logs** - Cannot track API call history
5. **Version Differences** - Some endpoints only available in newer versions (e.g., `/v1/traffic`)

### 6.2 Compatibility
- **macOS Exclusive Features**: `system_proxy`, `enhanced_mode`, `profiles/switch`
- **iOS Has No CLI** - Only accessible via HTTP API
- **Minimum Version Requirement**: Surge Mac 4.0+ (HTTP API v1)

### 6.3 Performance Considerations
- **Polling Frequency**: Recommend no more than once per second to avoid affecting Surge performance
- **Data Volume**: `dump request` default returns 100 entries, can be large with many requests
- **Timeout Settings**: Policy testing (`test-policy`) may take 5-30 seconds

---

## 7. Technical Selection Recommendation

**For SSH remote control Surge TUI requirements**:

| Solution | Advantages | Disadvantages | Recommendation |
|----------|------------|---------------|----------------|
| **Pure CLI Wrapper** | Rapid development, directly reuse commands | Complex output parsing, poor extensibility | ⭐⭐ |
| **HTTP API + TUI** | Structured data, easy to extend | Need to handle HTTP requests | ⭐⭐⭐⭐⭐ |
| **Hybrid Solution** | Combine advantages of both | High maintenance cost | ⭐⭐⭐ |

**Recommendation**: Build based on HTTP API, using Rust + `ratatui` + `reqwest`

**Reasons**:
1. JSON responses don't need parsing, directly map to data structures
2. HTTP connection reuse, better performance than executing CLI each time
3. Easy to extend new features (CLI features limited by official updates)
4. Can run independently of `surge-cli` (reduce dependencies)

---

## 8. References

- [Surge CLI Official Documentation](https://manual.nssurge.com/others/cli.html)
- [Surge HTTP API Official Documentation](https://manual.nssurge.com/others/http-api.html)
- [Surge Enhanced Mode Explanation](https://manual.nssurge.com/others/enhanced-mode.html)
- [Surge Official Manual](https://manual.nssurge.com/)

---

**Research Conclusion**: Surge provides comprehensive remote control capabilities, HTTP API is the best choice for building TUI. Next step is to clarify TUI functional scope and interaction design.
