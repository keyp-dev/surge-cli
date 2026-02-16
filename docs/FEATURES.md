# Surge TUI Feature Documentation

This document details the core features implemented in Surge TUI.

---

## Table of Contents

- [UI Interactions](#ui-interactions)
- [Policy Latency Testing](#policy-latency-testing)
- [Notification System](#notification-system)
- [Developer Tools](#developer-tools)
- [Policy Group Management](#policy-group-management)
- [Data Caching](#data-caching)
- [Connection Management](#connection-management)
- [Feature Toggles](#feature-toggles)
- [Request Details Enhancement](#request-details-enhancement)
- [DNS Cache Management](#dns-cache-management)

---

## UI Interactions

### Keyboard Shortcuts

| Key | Function | Description |
|-----|----------|-------------|
| `q` | Quit program | Exit directly |
| `r` | Refresh data | Manually refresh snapshot data |
| `1-5` | Switch views | Overview/Policies/Requests/Connections/DNS |
| `↑/↓` | Navigate list | Move selection up/down |
| `Enter` | Enter/Confirm | Enter policy group or switch policy |
| `Esc` | Back/Close | Exit policy group or close popup |
| `t` / `T` | Test latency | Non-blocking test all policy latencies |
| `m` / `M` | Toggle mode | Cycle through Direct/Proxy/Rule |
| `i` / `I` | Toggle MITM | Switch MITM status in Overview view |
| `c` / `C` | Toggle Capture | Switch traffic capture in Overview view |
| `k` / `K` | Kill connection | Terminate selected connection in Connections view |
| `f` / `F` | Flush cache | Flush DNS cache in DNS view |
| `n` / `N` | Notification history | View complete notification history |
| <code>`</code> | DevTools | Open developer debugging tools |
| `s` / `S` | Start Surge | Only available when Alert prompts |

### View Modes

#### 1. Overview

- Surge running status
- HTTP API availability
- Current outbound mode (supports `m` key quick toggle)
- MITM status (supports `i` key quick toggle)
- Capture status (supports `c` key quick toggle)
- System statistics

#### 2. Policies

**Left: Policy Group List**
- Policy group name (blue bold)
- Currently selected policy (green)
- **Latency display** (color-coded):
  - `< 100ms` - Cyan (fast)
  - `100-300ms` - Yellow (medium)
  - `> 300ms` - Red (slow)
- **Status markers**:
  - `✓` - Available
  - `✗` - Unavailable

**Right: Policies within Policy Group**
- Policy name
- Protocol type (color-coded)
- Latency and availability status
- Nested policy groups recursively display final policy latency

#### 3. Requests (Request History)

- Recent request records (limited to 50)
- URL, policy, traffic statistics
- **Request details panel**:
  - HTTP Body markers (whether request/response has data)
  - Connection logs (Notes) highlighting
  - TLS, DNS, rule matching details

#### 4. Active Connections

- Current active network connections (limited to 50)
- Supports `k` key to terminate selected connection (with confirmation dialog)
- **Connection details panel**:
  - URL, process, traffic statistics
  - HTTP Body markers
  - Connection log highlighting

#### 5. DNS Cache

- DNS cache record list
- Shows domain name, IP address, TTL (remaining time)
- Supports search filtering (`/` key)
- Supports `f` key to flush DNS cache

---

## Policy Latency Testing

### Features

**Non-blocking Architecture**
- Press `T` key to trigger test, UI remains responsive
- Uses `tokio::spawn` to execute tests in background
- Passes test results through `mpsc` message channel

**Test Flow**
```
User presses T → Background task starts → Notify "Testing"
                ↓
        Execute surge-cli test-all-policies
                ↓
        Parse test results (policy name, latency, status)
                ↓
        Send completion message → UI auto-refreshes → Display latency data
```

**Real-time Updates**
- UI immediately redraws after test completion, no need to wait for user interaction
- Status bar on right displays latest test result notification

### Nested Policy Group Support

**Recursive Policy Chain Resolution**

When policy groups contain other policy groups (e.g., `Proxy → US_Servers → us-server-1`), the system will:

1. Recursively find the final real policy
2. Display the final policy's latency data
3. Prevent circular references (max 10 levels of recursion)

**Example**
```
Policy Group View:
  Proxy → US_Servers (176ms)  ← Shows final policy latency
  AIProxy → jp-server-1 (206ms)

Policy Group Details (after entering Proxy):
  ✓ Direct [Available]
    AIProxy 281ms         ← Nested group shows its selected policy latency
    US_Servers 176ms      ← Nested group shows its selected policy latency
```

### Data Source

**surge-cli Mode**
```bash
/Applications/Surge.app/Contents/Applications/surge-cli test-all-policies
```

**Output Format Parsing**
```
ProxyName: RTT 123 ms, Total 456 ms  → Success, extract RTT value
ProxyName: Failed                     → Failure, mark unavailable
```

---

## Notification System

### Architecture Design

**Two-tier Display**
1. **Status bar real-time notifications** - Bottom right shows latest notification (displays time within 60 seconds)
2. **History popup** - Press `N` key to view complete history (max 50 entries)

**Notification Levels**
- `Info` (ℹ) - Cyan - Information prompt
- `Success` (✓) - Green - Operation successful
- `Error` (✗) - Red - Error warning

### Timestamp Format

**DevTools and History**
```
[2026-02-15 16:59:35] ✓ Test completed: 5/10 available
```

**Status Bar Notifications (within 60 seconds)**
```
✓ Test completed (16:59:35)
```

### Auto Cleanup

- Retain latest 50 notifications
- Automatically delete old notifications exceeding limit
- Won't auto-hide (unless replaced by new notifications)

---

## Developer Tools

### Features

Press <code>`</code> key to open DevTools panel (covers bottom 70%).

**Log Levels**
- `DEBUG` - Dark gray - Debug information
- `INFO` - Cyan - Regular logs
- `WARN` - Yellow - Warning information
- `ERROR` - Red - Error information

**Log Format**
```
[2026-02-15 16:59:35] INFO  Background test task started
[2026-02-15 16:59:38] INFO  Test completed: group=AIProxy, total=5, available=4
```

**Feature Capabilities**
- Display latest 100 log entries
- Support auto line-wrapping
- Absolute timestamps (year-month-day-hour-minute-second)
- Press `Esc` to close

**Use Cases**
- Debug policy name matching issues
- View test result details
- Diagnose API call errors
- Track background task status

---

## Policy Group Management

### Entering Policy Groups

**Operation Flow**
1. In Policies view (View 2), press `↑/↓` to select policy group
2. Press `Enter` to enter policy group
3. Right side displays all policies in that group
4. Press `↑/↓` to select target policy
5. Press `Enter` to switch to selected policy
6. Press `Esc` to return to policy group list

### Policy Switching

**API Call**
```rust
client.select_policy_group(group_name, policy_name).await
```

**UI Feedback**
- Immediately refresh data
- Update policy group's `selected` field
- Mark selected policy with `✓` in policy list

### Protocol Type Colors

Right-side policy list displays different colors based on protocol type:

| Protocol | Color |
|----------|-------|
| Shadowsocks | Blue |
| VMess | Magenta |
| Trojan | Yellow |
| DIRECT | Green |
| REJECT | Red |
| Other | Gray |

---

## Data Caching

### Test Result Persistence

**Problem**
- `refresh()` completely replaces `snapshot`
- Causes test results to be lost after refresh or switching pages

**Solution**
```rust
// App state
policy_test_cache: HashMap<String, PolicyDetail>

// Cache when test completes
for policy in &results {
    self.policy_test_cache.insert(policy.name.clone(), policy.clone());
}

// Restore cache during refresh
if !self.policy_test_cache.is_empty() {
    self.snapshot.policies = self.policy_test_cache.values().cloned().collect();
}
```

**Effect**
- Test results permanently saved (until overwritten by new test)
- Data refresh doesn't affect latency display
- Can still see test results after switching views

### Auto Refresh Interval

**Configuration**
```toml
[ui]
refresh_interval = 1  # seconds
```

**Refresh Strategy**
- Only refresh when user inactive and timeout occurs
- List remains stable during user operations (won't suddenly jump)
- Maintain test result cache

---

## Text Rendering Optimization

### Dynamic Column Width Calculation

**Problem**
- Fixed column width causes long text overlap
- CJK characters (Chinese) occupy 2 display widths

**Solution**
```rust
// Calculate dynamically based on terminal width
fn calculate_policy_column_widths(area_width: u16) -> (usize, usize, usize) {
    let available = (area_width as usize).saturating_sub(fixed_overhead);
    let status_width = 10;
    let name_width = (remaining * 0.6) as usize;
    let protocol_width = remaining - name_width;
    (name_width, protocol_width, status_width)
}

// Use unicode-width to calculate display width
use unicode_width::UnicodeWidthStr;

fn truncate_text(text: &str, max_width: usize) -> String {
    let mut width = 0;
    for (idx, ch) in text.char_indices() {
        width += UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]));
        if width > max_width {
            return format!("{}...", &text[..idx]);
        }
    }
    text.to_string()
}
```

**Dependency**
```toml
unicode-width = "0.2"
```

---

## Technical Implementation Details

### Async Architecture

**Event Loop**
```rust
loop {
    terminal.draw(|f| self.render(f))?;           // 1. Render UI

    while let Ok(msg) = self.test_rx.try_recv() {  // 2. Process background messages
        self.handle_test_message(msg);
    }

    if has_test_message {                          // 3. Immediate redraw
        terminal.draw(|f| self.render(f))?;
    }

    if event::poll(refresh_interval)? {            // 4. Wait for event or timeout
        self.handle_key(event::read()?).await;
    } else {
        self.refresh().await;                      // 5. Refresh on timeout
    }
}
```

### Message Passing

**TestMessage Enum**
```rust
enum TestMessage {
    Started,                              // Test started
    Completed {                           // Test completed
        group_name: String,
        results: Vec<PolicyDetail>,
    },
    Failed { error: String },             // Test failed
}
```

**Background Task**
```rust
tokio::spawn(async move {
    let _ = tx.send(TestMessage::Started).await;

    match client.test_all_policies_with_latency().await {
        Ok(results) => {
            tx.send(TestMessage::Completed { group_name, results }).await;
        }
        Err(e) => {
            tx.send(TestMessage::Failed { error: e.to_string() }).await;
        }
    }
});
```

---

## Dependencies

```toml
[dependencies]
tokio = { version = "1.45", features = ["full"] }
ratatui = "0.29"
crossterm = "0.28"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = "0.4"
unicode-width = "0.2"
```

**Key Dependency Roles**
- `tokio` - Async runtime, supports non-blocking tests
- `ratatui` - TUI framework
- `crossterm` - Terminal control
- `chrono` - Date/time handling (absolute timestamps)
- `unicode-width` - CJK character width calculation

---

## Connection Management

### Connection Termination Function

**Operation Flow**
1. In Connections view (View 4), press `↑/↓` to select connection
2. Press `k` key to popup confirmation dialog
3. Confirmation dialog shows connection details:
   - Target URL/Host
   - Process path
   - Upload/download traffic
4. Press `Enter` to confirm termination, press `Esc` to cancel

**Security Design**
- Destructive operations require confirmation to prevent accidents
- Display detailed information to help user double-check
- Supports use in both grouped and normal modes

**API Call**
```rust
client.kill_connection(connection_id).await
```

---

## Feature Toggles

### MITM and Capture Controls

**Overview View Display**
- MITM status: `✓ Enabled` / `✗ Disabled`
- Capture status: `✓ Enabled` / `✗ Disabled`
- Inline shortcut hints (btop style)

**Quick Toggle**
- Press `i` key to toggle MITM status
- Press `c` key to toggle Capture status
- Auto refresh status after successful operation
- Display notification feedback

**Use Cases**
- Quickly disable MITM when accessing bank websites
- Quickly enable Capture when debugging APIs
- No need to leave TUI to toggle features

**Refresh Strategy**
- Use real-time refresh strategy, not optimistic updates
- Immediately refresh snapshot after successful API call
- Avoid inconsistent state due to network latency

---

## Request Details Enhancement

### Connection Logs (Notes) Display

**Feature Capabilities**
- Display first 10 connection logs to avoid overly long panels
- Auto-highlight key tags
- Timestamps displayed in gray
- Add blank lines every 3 logs to enhance readability

**Highlighted Tags**

| Tag | Color | Description |
|-----|-------|-------------|
| `[Connection]` | Cyan | Connection protocol and status |
| `[TLS]` | Green | TLS handshake, SNI, cipher suites |
| `[DNS]` | Magenta | DNS resolution details |
| `[Rule]` | Yellow | Rule matching path |
| `[Socket]` | Blue | Socket connection details |
| `[HTTP]` | Light Green | HTTP connection info |
| `[Policy]` | Light Yellow | Policy decision path |

**Log Example**
```
14:49:36.011669 [Connection] Incoming proxy protocol: HTTP
14:49:36.012836 [TLS] TLS Client Hello SNI: www.googleapis.com
14:49:36.013548 [Rule] Sub-rule matched: DOMAIN-SUFFIX googleapis.com
14:49:36.013699 [Rule] Policy decision path: AIProxy -> 🇺🇸 ...
14:49:36.184591 [Socket] Connected to address 67.230.160.137 in 168.0ms
```

### HTTP Body Markers

**Function**
- Display whether request contains body data
- Display whether response contains body data
- Quickly determine HTTP transaction type

**Display Format**
- ✓ Has request data (streamHasRequestBody = true)
- ✓ Has response data (streamHasResponseBody = true)

**Debugging Value**

Although full HTTP headers/body cannot be viewed (limited by Surge API), Notes provide rich debugging information:
- **TLS issues**: Handshake failure reasons, protocol versions, cipher suites
- **DNS issues**: Resolution results, DNS servers, query times
- **Routing issues**: Rule matching path, policy decision chain
- **Performance issues**: Connection establishment time, socket connection details
- **Connection issues**: Disconnect reasons, error messages

**Complementary to Surge Dashboard**

For scenarios requiring complete HTTP headers/body viewing:
- **TUI**: Quick monitoring and control, view connection logs
- **Surge Dashboard**: Detailed HTTP debugging, view headers and body

Both complement each other, each serving its purpose.

**Advanced Feature (Optional)**

For complete HTTP headers/body data, refer to Scripting integration solution:
- Documentation: `docs/design/http-headers-capture-via-scripting.md`
- Requires manual Surge script configuration
- Estimated implementation time: 7-11 hours
- Has performance and storage overhead

---

## DNS Cache Management

### DNS View

**Feature Capabilities**
- Display all DNS cache records
- Left list: domain name, IP address, TTL
- Right details: complete information for selected record
- Support search filtering (by domain and IP)

**Data Parsing**
```rust
pub struct DnsRecord {
    pub domain: String,
    pub ip: Vec<String>,           // "data" field
    pub ttl: Option<f64>,           // "expiresTime" Unix timestamp
    pub server: Option<String>,     // DNS server
    pub logs: Vec<String>,          // Query logs
    pub path: Option<String>,       // Resolution path
    pub time_cost: Option<f64>,     // Query duration
}
```

**TTL Display**
- `expiresTime` is Unix timestamp (float)
- Calculate `expiresTime - current time` to get remaining seconds
- Display format: `123 s`

**Flush Cache**
- Press `f` key to flush all DNS cache
- Display confirmation notification
- Auto refresh list

**API Call**
```rust
client.get_dns_cache().await    // Get cache
client.flush_dns().await        // Flush cache
```

**Use Cases**
- Verify DNS configuration is in effect
- Debug DNS pollution issues
- Immediately flush cache after modifying DNS rules to verify

---

## Future Improvements

### Performance Optimization
- [ ] Policy latency test concurrency control (avoid testing too many policies simultaneously)
- [ ] Test result incremental updates (only update changed policies)

### Feature Enhancements
- [ ] Policy latency history trend chart
- [ ] Custom latency test target URL
- [ ] Policy group batch testing (instead of global testing)
- [ ] Policy search/filtering function

### User Experience
- [ ] Configurable color themes
- [ ] Latency value threshold configuration
- [ ] Notification persistence storage
- [ ] More detailed error diagnostic information

---

*Generated with [Claude Code](https://claude.ai/code)*
*via [Happy](https://happy.engineering)*

*Co-Authored-By: Claude <noreply@anthropic.com>*
