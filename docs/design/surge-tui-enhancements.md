# Surge TUI Feature Enhancement Design

**Status:** Draft
**Date:** 2026-02-16
**Author:** Claude with Happy

---

## Problem

Current Surge TUI has three functional gaps:

1. **Connections view lacks actions:** Users can see active connection lists, including connections consuming significant bandwidth, but cannot terminate them. This violates the "monitoring is control" principle. Users can see but cannot kill - this is incomplete functionality.

2. **Missing core feature toggles:** MITM and Capture are core Surge features that users frequently need to toggle temporarily (disable MITM when accessing banks, enable Capture when debugging APIs). Currently, users must leave TUI to go to Surge Dashboard or modify configuration files, which violates TUI's "quick control" design intent.

3. **DNS debugging capability missing:** When encountering DNS pollution or validating DNS configuration, users need external tools or CLI commands. After modifying DNS rules, cannot immediately flush cache for validation - must restart Surge.

Consequences of not solving:
- Fragmented user experience, TUI becomes a "read-only dashboard" instead of "control panel"
- Frequent tool switching reduces work efficiency
- Critical operation paths too long (3+ steps), violates TUI's quick operation design goal

---

## Solution

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                   Surge TUI Feature Hierarchy                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Overview   │  │   Policies   │  │   Requests   │      │
│  │ (Enhanced)  │  │              │  │              │      │
│  │  - MITM     │  │  - Groups    │  │  - History   │      │
│  │  - Capture  │  │  - Latency   │  │  - Search    │      │
│  └─────────────┘  └──────────────┘  └──────────────┘      │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐                        │
│  │ Connections  │  │   DNS (New)  │ ← [FEATURE]           │
│  │ (Enhanced)   │  │              │                        │
│  │  - Kill Conn │  │  - DNS Cache │                        │
│  └──────────────┘  │  - Flush     │                        │
│        ↑           └──────────────┘                        │
│   [FEATURE]                                                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────┐
│              SurgeClient (Application Layer)                │
│  - kill_connection()   ← Implemented but unused            │
│  - get/set_mitm_status()   ← Implemented but unused        │
│  - get/set_capture_status()   ← Implemented but unused     │
│  - get_dns_cache()   ← Implemented but unused              │
│  - flush_dns()   ← Implemented but unused                  │
└─────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────┐
│         SurgeHttpClient (Infrastructure Layer)              │
│  Surge HTTP API v1 - Fully wrapped                         │
└─────────────────────────────────────────────────────────────┘
```

---

## Key Decisions

### Decision 1: Phased Implementation, Prioritize Fixing Incomplete Features

**Choice:** Phase 1 (Connection Kill) → Phase 2 (Feature Toggles) → Phase 3 (DNS)

**Rationale:**
- Phase 1 fixes existing incomplete functionality, highest ROI (15 minutes vs immediate user value)
- Phase 2 completes control panel, aligns with TUI design goals
- Phase 3 is independent new feature, can decide based on actual demand

**Cost:**
- Phased implementation means features won't launch all at once
- Need to maintain i18n and documentation across multiple iterations

**Reconsider When:**
- If Phase 1 implementation reveals architectural issues, need to re-evaluate subsequent phase designs

---

### Decision 2: Connection Kill Uses Confirmation Dialog, Not Direct Execution

**Choice:** Press `k` to show confirmation dialog with connection details, execute after confirmation

**Rationale:**
- Killing connections is destructive, high cost of accidental operation (interrupt downloads, disconnect SSH)
- Confirmation dialog can show connection details (target, process) for user double-check
- Follows "dangerous operations need confirmation" UX best practice

**Cost:**
- Adds one key press (`k` → `Enter`)
- Requires additional confirmation dialog component implementation

**Reconsider When:**
- If users feedback "confirmation dialog annoying when frequently killing connections", can add config option to disable

---

### Decision 3: MITM/Capture Toggles Display in Overview, Not Separate View

**Choice:** Add two status rows + shortcuts in Overview view, similar to Outbound Mode

**Rationale:**
- Overview is positioned as "system status overview + quick control"
- MITM/Capture are system-level toggles, not specific resources (policies/requests/connections)
- Avoids too many views (5 tabs would complicate shortcut mapping)

**Cost:**
- Overview view content increases, may need layout adjustment
- If more system-level toggles in future, Overview becomes crowded

**Reconsider When:**
- If system-level toggles exceed 5, consider separate "Settings" view

---

### Decision 4: DNS Feature as 5th Independent Tab

**Choice:** Add new `[5] DNS` view, showing cache list + shortcut to flush

**Rationale:**
- DNS cache is independent resource (like Requests/Connections), not suitable for Overview
- Cache list may be long (100+ entries), needs independent space to display
- Supports search and detail view, follows existing UI pattern

**Cost:**
- Adds one tab, shortcut mapping extends from `1-4` to `1-5`
- Requires complete list rendering, search, i18n implementation
- Implementation time ~1-2 hours

**Reconsider When:**
- If users don't frequently debug DNS, this feature can be skipped
- If DNS cache count usually small (<20 entries), can place in Overview instead of separate view

---

## Technical Specifications

### Phase 1: Connection Kill Feature

#### Modified Files
- `src/ui/app.rs` - Keyboard event handling
- `src/ui/components/requests.rs` - Connection list display
- `src/i18n/*.rs` - Translation text

#### Implementation Details

**1. Keyboard Binding**
```rust
// src/ui/app.rs handle_key()
KeyCode::Char('k') | KeyCode::Char('K') => {
    if matches!(self.current_view, ViewMode::ActiveConnections) {
        // Get selected connection
        if let Some(connection) = self.get_selected_connection() {
            self.show_kill_confirm = Some(connection.id);
        }
    }
}
```

**2. Confirmation Dialog**
```rust
struct App {
    // ... existing fields
    show_kill_confirm: Option<u64>, // Show confirm dialog when Some(connection_id)
}

fn render_kill_confirm(&self, f: &mut Frame, connection: &Request) {
    // Centered popup showing:
    // - Connection ID
    // - Target URL/Host
    // - Process path
    // - Upload/Download traffic
    // [Enter] Confirm  [ESC] Cancel
}
```

**3. Execute Kill**
```rust
// Execute after confirmation
if self.show_kill_confirm.is_some() {
    match key.code {
        KeyCode::Enter => {
            let id = self.show_kill_confirm.unwrap();
            if let Err(e) = self.client.kill_connection(id).await {
                self.add_notification(Notification::error(
                    format!("Failed to kill connection: {}", e)
                ));
            } else {
                self.add_notification(Notification::success("Connection killed"));
                self.refresh().await; // Refresh list
            }
            self.show_kill_confirm = None;
        }
        KeyCode::Esc => {
            self.show_kill_confirm = None;
        }
        _ => {}
    }
    return; // Block other keys
}
```

**4. UI Shortcut Hint**
```rust
// Connections list title
Line::from(vec![
    Span::raw(" "),
    Span::raw(t.view_connections()),
    Span::raw(" ["),
    Span::styled("↑↓", Style::default().fg(Color::Yellow)),
    Span::raw("]"),
    Span::raw(t.action_select()),
    Span::raw(" ["),
    Span::styled("k", Style::default().fg(Color::Yellow)),
    Span::raw("]"),
    Span::raw(t.action_kill()),  // New translation: "Kill"
    Span::raw(" "),
])
```

#### i18n Additions
```rust
// src/i18n/mod.rs
fn action_kill(&self) -> &'static str;
fn confirm_kill_title(&self) -> &'static str;
fn confirm_kill_message(&self, url: &str) -> String;

// src/i18n/zh_cn.rs
fn action_kill(&self) -> &'static str { "终止" }
fn confirm_kill_title(&self) -> &'static str { " 确认终止连接 " }
fn confirm_kill_message(&self, url: &str) -> String {
    format!("确定要终止到 {} 的连接吗?", url)
}

// src/i18n/en_us.rs
fn action_kill(&self) -> &'static str { "Kill" }
fn confirm_kill_title(&self) -> &'static str { " Confirm Kill Connection " }
fn confirm_kill_message(&self, url: &str) -> String {
    format!("Are you sure to kill connection to {}?", url)
}
```

---

### Phase 2: MITM/Capture Feature Toggles

#### Modified Files
- `src/ui/app.rs` - State management, keyboard events
- `src/ui/components/overview.rs` - Display status and shortcuts
- `src/application/surge_client.rs` - Expose MITM/Capture API
- `src/domain/entities.rs` - Add AppSnapshot fields
- `src/i18n/*.rs` - Translation text

#### Implementation Details

**1. Add AppSnapshot Fields**
```rust
// src/domain/entities.rs
pub struct AppSnapshot {
    // ... existing fields
    pub mitm_enabled: Option<bool>,
    pub capture_enabled: Option<bool>,
}
```

**2. Get Status**
```rust
// src/application/surge_client.rs get_snapshot()
// Get when HTTP API available
if snapshot.http_api_available {
    if let Ok(mitm) = self.http_client.get_mitm_status().await {
        snapshot.mitm_enabled = Some(mitm);
    }
    if let Ok(capture) = self.http_client.get_capture_status().await {
        snapshot.capture_enabled = Some(capture);
    }
}
```

**3. Overview Display**
```rust
// src/ui/components/overview.rs
// Add after Outbound Mode

// MITM status
if let Some(mitm) = snapshot.mitm_enabled {
    let status = if mitm { "✓ Enabled" } else { "✗ Disabled" };
    let color = if mitm { Color::Green } else { Color::Red };
    lines.push(Line::from(vec![
        Span::styled("MITM: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(status, Style::default().fg(color)),
        Span::raw("  ["),
        Span::styled("i", Style::default().fg(Color::Yellow)),
        Span::raw("]"),
        Span::raw(t.action_toggle()),
    ]));
}

// Capture status
if let Some(capture) = snapshot.capture_enabled {
    let status = if capture { "✓ Enabled" } else { "✗ Disabled" };
    let color = if capture { Color::Green } else { Color::Red };
    lines.push(Line::from(vec![
        Span::styled("Capture: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(status, Style::default().fg(color)),
        Span::raw("  ["),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw("]"),
        Span::raw(t.action_toggle()),
    ]));
}
```

**4. Keyboard Binding**
```rust
// src/ui/app.rs handle_key()
KeyCode::Char('i') | KeyCode::Char('I') => {
    if matches!(self.current_view, ViewMode::Overview) {
        if let Some(enabled) = self.snapshot.mitm_enabled {
            if self.client.set_mitm_status(!enabled).await.is_ok() {
                self.add_notification(Notification::success(
                    format!("MITM {}", if enabled { "disabled" } else { "enabled" })
                ));
                self.refresh().await;
            }
        }
    }
}

KeyCode::Char('c') | KeyCode::Char('C') => {
    if matches!(self.current_view, ViewMode::Overview) {
        if let Some(enabled) = self.snapshot.capture_enabled {
            if self.client.set_capture_status(!enabled).await.is_ok() {
                self.add_notification(Notification::success(
                    format!("Capture {}", if enabled { "disabled" } else { "enabled" })
                ));
                self.refresh().await;
            }
        }
    }
}
```

#### i18n Additions
```rust
// src/i18n/mod.rs
fn feature_mitm(&self) -> &'static str;
fn feature_capture(&self) -> &'static str;
fn status_enabled(&self) -> &'static str;
fn status_disabled(&self) -> &'static str;

// src/i18n/zh_cn.rs
fn feature_mitm(&self) -> &'static str { "MITM" }
fn feature_capture(&self) -> &'static str { "流量捕获" }
fn status_enabled(&self) -> &'static str { "✓ 已启用" }
fn status_disabled(&self) -> &'static str { "✗ 已禁用" }
```

---

### Phase 3: DNS Cache View [FEATURE]

#### Modified Files
- `src/ui/app.rs` - Add ViewMode::Dns
- `src/ui/components/dns.rs` - New file, DNS view rendering
- `src/application/surge_client.rs` - Expose DNS API
- `src/domain/entities.rs` - Add dns_cache field to AppSnapshot
- `src/i18n/*.rs` - Translation text

#### Implementation Details

**1. ViewMode Extension**
```rust
// src/domain/models.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Overview,
    Policies,
    Requests,
    ActiveConnections,
    Dns, // New
}
```

**2. Add AppSnapshot Field**
```rust
pub struct AppSnapshot {
    // ... existing fields
    pub dns_cache: Vec<DnsRecord>,
}

// DnsRecord already defined in domain/models.rs
pub struct DnsRecord {
    pub domain: String,
    pub record_type: String, // A, AAAA, CNAME, etc.
    pub value: String,
    pub ttl: u64,
}
```

**3. DNS View Component**
```rust
// src/ui/components/dns.rs
pub fn render(
    f: &mut Frame,
    area: Rect,
    records: &[DnsRecord],
    selected: usize,
    search_query: &str,
    search_mode: bool,
    t: &'static dyn Translate,
) {
    // Split: record list | detail info
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_dns_list(f, chunks[0], records, selected, search_query, search_mode, t);
    render_dns_detail(f, chunks[1], records, selected, t);
}

fn render_dns_list(/* ... */) {
    // Display: Domain | Type | Value | TTL
    // Support search (filter domain and value)
    // Title shows [f]Flush shortcut
}
```

**4. Keyboard Binding**
```rust
// Tab switch
KeyCode::Char('5') => {
    self.current_view = ViewMode::Dns;
    self.selected_index = 0;
}

// Flush DNS cache
KeyCode::Char('f') | KeyCode::Char('F') => {
    if matches!(self.current_view, ViewMode::Dns) {
        if self.client.flush_dns().await.is_ok() {
            self.add_notification(Notification::success("DNS cache flushed"));
            self.refresh().await;
        }
    }
}
```

#### i18n Additions
```rust
fn view_dns(&self) -> &'static str;
fn dns_list_title(&self) -> &'static str;
fn dns_detail_title(&self) -> &'static str;
fn dns_no_records(&self) -> &'static str;
fn dns_label_domain(&self) -> &'static str;
fn dns_label_type(&self) -> &'static str;
fn dns_label_value(&self) -> &'static str;
fn dns_label_ttl(&self) -> &'static str;
fn action_flush(&self) -> &'static str;
```

---

## Known Issues

### Critical (Must Fix)

**Issue 1: Connection kill may fail without detailed error messages**

- **Phenomenon:** `kill_connection()` Surge API call may fail due to permissions, connection already closed, etc.
- **Impact:** User sees "Failed to kill connection" but doesn't know why
- **Workaround:** Show generic error message, suggest refreshing list to confirm status
- **Fix Plan:** After Phase 1 completion, add detailed error code mapping based on actual testing

**Issue 2: MITM/Capture toggle lacks instant visual feedback**

- **Phenomenon:** After toggling, need to wait for next refresh (or manual `r`) to see status change
- **Impact:** User unsure if operation succeeded
- **Workaround:** Show notification "MITM enabled", prompt user operation executed
- **Fix Plan:** Call `refresh()` immediately after successful toggle to update snapshot

---

### Important (Affects Performance But Can Ship)

**Issue 3: DNS cache list may be long, affecting rendering performance**

- **Phenomenon:** DNS cache may have 100+ records, full rendering may lag
- **Impact:** Scrolling list may not be smooth on low-end machines
- **Workaround:** Use existing `.take(50)` limit, only show first 50 entries
- **Fix Plan:** Implement virtual scrolling or pagination, decide during Phase 3 based on actual performance

**Issue 4: Degraded behavior when feature status fetch fails**

- **Phenomenon:** If `get_mitm_status()` fails, Overview doesn't show toggle
- **Impact:** User may mistakenly think feature doesn't exist
- **Workaround:** Only show toggles when HTTP API available
- **Fix Plan:** Show "unknown" status + disable shortcuts on failure, evaluate during Phase 2

---

### Nice to have (Optimizations)

**Issue 5: Confirmation dialog cannot be operated with mouse**

- **Phenomenon:** Connection kill confirmation dialog only supports keyboard (Enter/ESC), no mouse support
- **Impact:** Some users prefer mouse operations
- **Workaround:** Document shortcuts
- **Fix Plan:** No mouse support for now, maintain TUI's pure keyboard operation consistency

**Issue 6: DNS cache lacks export feature**

- **Phenomenon:** Users cannot export DNS cache to file
- **Impact:** Cannot offline analyze or share
- **Workaround:** Users can export via Surge CLI
- **Fix Plan:** If Phase 3 implemented, can consider adding export feature

---

## Implementation Plan

### Phase 1: Connection Kill (Priority 1)

**Estimated Time:** 15-30 minutes

**Task Checklist:**
- [ ] Add `show_kill_confirm` state field
- [ ] Implement confirmation dialog rendering
- [ ] Add keyboard binding (`k` key)
- [ ] Add i18n translations
- [ ] Update Connections view title to show shortcut
- [ ] Test: kill connection, cancel operation, error handling

---

### Phase 2: MITM/Capture Toggles (Priority 2)

**Estimated Time:** 30-45 minutes

**Task Checklist:**
- [ ] Add `mitm_enabled`, `capture_enabled` fields to AppSnapshot
- [ ] Get status in SurgeClient::get_snapshot()
- [ ] Display status and shortcuts in Overview component
- [ ] Add keyboard bindings (`i`, `c` keys)
- [ ] Add i18n translations
- [ ] Test: toggle switches, status refresh, error handling

---

### Phase 3: DNS Cache View (Priority 3, Optional)

**Estimated Time:** 1-2 hours

**Task Checklist:**
- [ ] Add Dns enum to ViewMode
- [ ] Create `src/ui/components/dns.rs`
- [ ] Implement DNS list and detail rendering
- [ ] Add search functionality support
- [ ] Add `dns_cache` field to AppSnapshot
- [ ] Get DNS cache in SurgeClient::get_snapshot()
- [ ] Add keyboard bindings (`5`, `f` keys)
- [ ] Add i18n translations
- [ ] Test: list display, search, flush cache

**Pre-implementation Confirmation:** Do users frequently need to debug DNS? If not, this phase can be skipped.

---

## Acceptance Criteria

### Phase 1

- [ ] Press `k` in Connections view shows confirmation dialog
- [ ] Confirmation dialog shows connection details (URL, process, traffic)
- [ ] Press Enter successfully kills connection, list refreshes
- [ ] Press ESC cancels operation, returns to list
- [ ] Shows error notification when kill fails
- [ ] Shortcut hint displays correctly in title bar

### Phase 2

- [ ] Overview displays MITM and Capture status (enabled/disabled)
- [ ] Press `i` toggles MITM, status updates immediately
- [ ] Press `c` toggles Capture, status updates immediately
- [ ] Shows notification on successful toggle
- [ ] Shows error notification on failed toggle
- [ ] Shortcut hints display correctly in status rows

### Phase 3

- [ ] Press `5` enters DNS view
- [ ] DNS list displays domain, type, value, TTL
- [ ] Search functionality works normally
- [ ] Press `f` flushes cache, shows notification
- [ ] List refreshes after flush
- [ ] Shortcut hints display correctly in title bar

---

## References

- Surge HTTP API documentation: `src/infrastructure/http_client.rs`
- Existing UI component patterns: `src/ui/components/requests.rs`
- i18n system: `src/i18n/mod.rs`
- Notification system: `src/domain/entities.rs` Notification

---

*Generated with [Claude Code](https://claude.com/claude-code) via [Happy](https://happy.engineering)*
