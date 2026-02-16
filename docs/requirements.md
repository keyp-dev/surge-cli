# Surge TUI - Requirements Document

**Version**: 1.0 (Implementation Complete)
**Date**: 2026-02-14
**Status**: Implemented

---

## Problem (Problem Statement)

### Current Situation

Surge is a powerful network proxy tool on macOS, but it lacks a command-line graphical interface. When working in a terminal environment, I face the following problems:

**Problem 1: Cannot view Surge status in the terminal**
- Cannot see the currently used policy groups and selected nodes
- Cannot view real-time request traffic
- Cannot view node latency and availability

**Problem 2: Low terminal operation efficiency**
- Switching policies requires opening the GUI or remembering complex CLI commands
- Testing node latency requires executing multiple commands
- Viewing connection status requires parsing plain text output

**Problem 3: Lack of integrated workflow**
- Common operations are scattered across multiple commands
- Need to frequently switch contexts (view → decision → operation → verification)
- Cannot quickly respond to network issues (node failure, policy anomalies)

### Impact Scope

**Frequency**: 5-10 times per day needing to operate Surge (switch nodes, view connections, troubleshoot)
**Time Cost**: Average 2-5 minutes per operation (remember commands, parse output, execute operations)
**Users**: Primarily myself, but valuable for all users who need to manage Surge in terminal

### Why Solve Now

1. **Work Mode Change** - Increasingly more time working in terminal environment, cannot rely on GUI
2. **Existing Technical Foundation** - Surge provides comprehensive HTTP API (see `research.md`)
3. **High Feasibility** - Rust + ratatui ecosystem is mature, actual development cycle 3-4 days
4. **Reuse Value** - Solution can be open-sourced, benefiting the community

### Consequences of Not Solving

- Continue inefficient operations, wasting 30-50 minutes daily
- Cannot quickly respond to network issues, affecting work continuity
- Long-term dependence on GUI, limited remote work capabilities

---

## Solution (Design Proposal)

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    macOS Local Environment                        │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                surge-tui (Rust Binary)                     │ │
│  │  ┌──────────┐  ┌───────────────────────────────────────┐  │ │
│  │  │ ratatui  │  │    SurgeClient (Application Layer)    │  │ │
│  │  │(TUI render)│◄─┤  ┌─────────────────────────────────┐  │  │ │
│  │  └──────────┘  │  │ Infrastructure Layer             │  │  │ │
│  │                │  │  - HttpClient (reqwest)          │  │  │ │
│  │                │  │  - CliClient (surge-cli)         │  │  │ │
│  │                │  │  - SystemClient (pgrep/open)     │  │  │ │
│  │                │  └─────────────────────────────────┘  │  │ │
│  │                │          │                             │  │ │
│  │                │          │ Domain Layer (models)       │  │ │
│  │                └──────────┼─────────────────────────────┘  │ │
│  └───────────────────────────┼─────────────────────────────────┘ │
│                               │                                   │
│                               │ HTTP API (127.0.0.1:6170)         │
│                               │ surge-cli commands                │
│                               ▼                                   │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                        Surge.app                           │  │
│  │  ┌────────────────────┐  ┌──────────────────────────┐     │  │
│  │  │ HTTP API Server    │  │  IPC (XPC)               │     │  │
│  │  │ (127.0.0.1:6170)   │  │  ◄─── surge-cli          │     │  │
│  │  └────────────────────┘  └──────────────────────────┘     │  │
│  │  ┌──────────────────────────────────────────────────┐     │  │
│  │  │  Config: ~/Library/Application Support/Surge/   │     │  │
│  │  │  [General]                                        │     │  │
│  │  │  http-api = key@127.0.0.1:6170                   │     │  │
│  │  └──────────────────────────────────────────────────┘     │  │
│  └────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘

Communication flow (with fallback):
1. Primary: HTTP API (127.0.0.1:6170) - Fast, full-featured, real-time data
2. Fallback: surge-cli - Used to reload config when HTTP API unavailable
3. Last resort: System commands (pgrep/open) - Detect and start Surge when not running

All calls are local, TUI runs directly in macOS terminal
```

### Core Design Decisions

#### Decision 1: Hybrid Architecture (HTTP API + CLI) instead of HTTP API Only

**Choice**: Three-tier fallback mechanism (all local calls)
1. **Primary**: HTTP API (127.0.0.1:6170) - Normal scenarios, real-time data
2. **Fallback**: Local `surge-cli` commands - Reload config when HTTP API unavailable
3. **Last resort**: Local system commands (pgrep/open) - Detect and start Surge

**Reasons**:
- HTTP API has good performance, full features, provides real-time data (requests, connections, policies)
- **Reliability requirement**: Must handle scenarios where Surge is not running or HTTP API is not configured
- `surge-cli` communicates with Surge.app via IPC (XPC), doesn't depend on HTTP API
- System commands can detect processes and start Surge
- **Alert mechanism**: Don't auto-fix, show warnings and provide user action buttons

**Cost**:
- Slightly increased complexity (three clients, but unified interface)
- Limited functionality in CLI mode (cannot get real-time request data)

**Actual implementation**:
- Show Alert when HTTP API fails, prompting user to press R to reload config
- Show Alert when Surge not running, prompting user to press S to start
- No auto-fix, user explicit actions

**Reconsideration conditions**:
- Surge provides more reliable self-healing mechanism
- Surge provides unified Unix socket API

#### Decision 2: Using Rust + ratatui

**Choice**: Rust language + ratatui TUI framework
**Reasons**:
- Rust compiles to single binary, no runtime dependencies
- ratatui is mature and stable, active community (2600+ stars)
- Type-safe, reduces runtime errors
- Excellent performance, suitable for real-time refresh scenarios

**Cost**:
- Steep learning curve for Rust (but I'm already familiar)
- Longer compile times (incremental compilation helps)

**Alternative**: Python + textual (fast prototype, but needs runtime)

**Reconsideration conditions**:
- Need cross-platform GUI (Electron/Tauri)
- Team's main language is Python/Go

#### Decision 3: Polling Mode instead of Real-time Push

**Choice**: Poll API once per second to update data
**Reasons**:
- Surge HTTP API doesn't support WebSocket
- 1 second refresh rate is sufficient for human interaction
- Simple implementation, no need to maintain long connections

**Cost**:
- Latency up to 1 second
- Consumes some CPU/network even when idle

**Optimization**: Reduce polling frequency to 5 seconds when no user interaction

**Reconsideration conditions**:
- Surge provides WebSocket/SSE API
- Need millisecond-level real-time (like network monitoring tools)

---

## Specification (Technical Specifications)

### Data Models

#### 1. Application Configuration (Actual Implementation)
```rust
#[derive(Debug, Clone, Deserialize)]
struct Config {
    surge: SurgeConfig,
    ui: UiConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct SurgeConfig {
    http_api_host: String,      // "127.0.0.1"
    http_api_port: u16,          // 6170
    http_api_key: String,        // Surge API Key
    cli_path: Option<String>,    // surge-cli path, optional
}

#[derive(Debug, Clone, Deserialize)]
struct UiConfig {
    refresh_interval: u64,      // Refresh interval (seconds), default 10
    max_requests: usize,        // Max request history, default 100
}

// Config file: surge-tui.toml (project root)
// [surge]
// http_api_host = "127.0.0.1"
// http_api_port = 6170
// http_api_key = "keyp"
// cli_path = "/Applications/Surge.app/Contents/Applications/surge-cli"
//
// [ui]
// refresh_interval = 10  # seconds
// max_requests = 100
```

#### 2. Outbound Mode
```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum OutboundMode {
    Direct,  // Direct connection
    Proxy,   // Global proxy
    Rule,    // Rule mode
}

#[derive(Debug, Clone, Deserialize)]
struct OutboundState {
    mode: OutboundMode,
    global_policy: String,  // Global default policy name
}
```

#### 3. Policies and Policy Groups (Actual API Response)
```rust
// GET /v1/policies actual response
#[derive(Debug, Clone, Deserialize)]
struct PoliciesResponse {
    proxies: Vec<String>,          // Proxy list
    #[serde(rename = "policy-groups")]
    policy_groups: Vec<String>,    // Policy group name list
}

// GET /v1/policy_groups actual response (HashMap!)
type PolicyGroupsResponse = HashMap<String, Vec<PolicyItem>>;

#[derive(Debug, Clone, Deserialize)]
struct PolicyItem {
    #[serde(rename = "isGroup")]
    is_group: bool,                 // Is policy group
    name: String,
    #[serde(rename = "typeDescription")]
    type_description: String,       // "Shadowsocks", "VMess", etc.
    #[serde(rename = "lineHash")]
    line_hash: String,
    enabled: bool,                  // Is selected
}

// Internal policy group model
#[derive(Debug, Clone)]
struct PolicyGroup {
    name: String,
    policies: Vec<PolicyItem>,
    selected: Option<String>,       // First policy with enabled=true
}

// Policy detail
#[derive(Debug, Clone)]
struct PolicyDetail {
    name: String,
    policy_type: PolicyType,
    alive: bool,
    latency: Option<u32>,
    last_test_at: Option<String>,
}
```

#### 4. Request Record (Actual API Field Names)
```rust
#[derive(Debug, Clone, Deserialize)]
struct Request {
    id: u64,
    #[serde(default, rename = "processPath")]
    process_path: Option<String>,
    #[serde(default)]
    rule: Option<String>,
    #[serde(default, rename = "policyName")]  // ⚠️ Note field name
    policy_name: Option<String>,
    #[serde(default, rename = "remoteHost")]
    remote_host: Option<String>,
    #[serde(default, rename = "URL")]
    url: Option<String>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default, rename = "startDate")]  // Unix timestamp (float)
    start_date: Option<f64>,
    #[serde(default, rename = "inBytes")]    // ⚠️ Download
    in_bytes: u64,
    #[serde(default, rename = "outBytes")]   // ⚠️ Upload
    out_bytes: u64,
    #[serde(default)]
    completed: bool,
    #[serde(default)]
    failed: bool,
}

// Key differences:
// - policy → policyName
// - upload → outBytes
// - download → inBytes
// - remote_address → remoteHost
// - start_time → startDate (Unix timestamp)
```

#### 5. Feature Toggles
```rust
#[derive(Debug, Clone)]
struct Features {
    mitm: bool,
    capture: bool,
    rewrite: bool,
    scripting: bool,
    system_proxy: bool,      // macOS only
    enhanced_mode: bool,     // macOS only
}
```

### API Client Interfaces

#### HTTP API Client

```rust
use reqwest::Client;
use anyhow::Result;

struct SurgeHttpClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl SurgeHttpClient {
    fn new(base_url: String, api_key: String, timeout_ms: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .unwrap();

        Self { client, base_url, api_key }
    }

    // Health check
    async fn health_check(&self) -> Result<bool>;

    // Get outbound mode
    async fn get_outbound(&self) -> Result<OutboundState>;

    // Switch outbound mode
    async fn set_outbound(&self, mode: OutboundMode) -> Result<()>;

    // Switch global policy
    async fn set_global_policy(&self, policy: &str) -> Result<()>;

    // Get all policies
    async fn get_policies(&self) -> Result<Vec<Policy>>;

    // Get all policy groups
    async fn get_policy_groups(&self) -> Result<Vec<PolicyGroup>>;

    // Select policy in policy group
    async fn select_policy(
        &self,
        group: &str,
        policy: &str
    ) -> Result<()>;

    // Test policy
    async fn test_policy(&self, policy: &str) -> Result<u32>; // Return latency ms

    // Test policy group
    async fn test_group(&self, group: &str) -> Result<()>;

    // Get recent requests
    async fn get_recent_requests(&self) -> Result<Vec<Request>>;

    // Get active connections
    async fn get_active_requests(&self) -> Result<Vec<Request>>;

    // Kill connection
    async fn kill_request(&self, id: u64) -> Result<()>;

    // Get feature toggle status
    async fn get_features(&self) -> Result<Features>;

    // Toggle feature
    async fn toggle_feature(&self, feature: &str, enabled: bool) -> Result<()>;

    // Reload configuration
    async fn reload_profile(&self) -> Result<()>;

    // Flush DNS cache
    async fn flush_dns(&self) -> Result<()>;
}
```

#### Local CLI Client

```rust
use tokio::process::Command;

struct SurgeCliClient {
    cli_path: String,
}

impl SurgeCliClient {
    fn new(cli_path: String) -> Self {
        Self { cli_path }
    }

    // Execute surge-cli command
    async fn execute(&self, args: &[&str]) -> Result<String> {
        let output = Command::new(&self.cli_path)
            .args(args)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "surge-cli failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    // Get policies (using --raw JSON output)
    async fn get_policies(&self) -> Result<Vec<Policy>> {
        let output = self.execute(&["--raw", "dump", "policy"]).await?;
        serde_json::from_str(&output)
            .map_err(|e| anyhow::anyhow!("Failed to parse policies: {}", e))
    }

    // Reload configuration
    async fn reload(&self) -> Result<()> {
        self.execute(&["reload"]).await?;
        Ok(())
    }
}
```

#### System Management Client

```rust
use tokio::process::Command;
use std::path::Path;

struct SurgeSystemClient {
    cli: SurgeCliClient,
}

impl SurgeSystemClient {
    fn new(cli: SurgeCliClient) -> Self {
        Self { cli }
    }

    // Check if Surge process is running
    async fn is_running(&self) -> Result<bool> {
        let output = Command::new("pgrep")
            .arg("-x")
            .arg("Surge")
            .output()
            .await?;

        Ok(output.status.success() && !output.stdout.is_empty())
    }

    // Start Surge
    async fn start_surge(&self) -> Result<()> {
        Command::new("open")
            .arg("-a")
            .arg("Surge")
            .output()
            .await?;

        // Wait for startup (max 5 seconds)
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if self.is_running().await? {
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("Surge startup timeout"))
    }

    // Get config file path
    async fn get_config_path(&self) -> Result<PathBuf> {
        // Possible Surge config file locations
        let home = std::env::var("HOME")?;
        let paths = vec![
            PathBuf::from(format!("{}/Library/Application Support/Surge/surge.conf", home)),
            PathBuf::from(format!("{}/.config/surge/surge.conf", home)),
        ];

        for path in paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!("Surge config file not found"))
    }

    // Read config file
    async fn read_config(&self) -> Result<String> {
        let path = self.get_config_path().await?;
        tokio::fs::read_to_string(path).await
            .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))
    }

    // Check if config has http-api
    async fn has_http_api_config(&self) -> Result<bool> {
        let config = self.read_config().await?;
        Ok(config.contains("http-api"))
    }

    // Reload config (user manually triggered)
    async fn reload_config(&self) -> Result<()> {
        self.cli.reload().await
    }
}
```

#### Unified Client (with Fallback)

```rust
enum SurgeClientMode {
    HttpApi(SurgeHttpClient),
    Cli(SurgeCliClient),
}

struct SurgeClient {
    mode: SurgeClientMode,
    http_client: SurgeHttpClient,  // Keep for retry
    cli_client: SurgeCliClient,
    system: SurgeSystemClient,
    config: Config,
}

impl SurgeClient {
    async fn new(config: Config) -> Result<Self> {
        let cli = SurgeCliClient::new(config.surge_cli_path.clone());
        let system = SurgeSystemClient::new(cli.clone());

        // Health check flow
        let (mode, http_client) = Self::select_mode(&config, &system).await?;

        Ok(Self {
            mode,
            http_client: http_client.clone(),
            cli_client: cli,
            system,
            config,
        })
    }

    // Select communication mode (no auto-fix)
    async fn select_mode(
        config: &Config,
        system: &SurgeSystemClient,
    ) -> Result<(SurgeClientMode, SurgeHttpClient, Option<Alert>)> {
        // 1. Check Surge process
        if !system.is_running().await? {
            // Don't auto-start, return warning
            let http_client = SurgeHttpClient::new(
                config.api_base_url.clone(),
                config.api_key.clone(),
                config.request_timeout_ms,
            );
            return Ok((
                SurgeClientMode::Cli(system.cli.clone()),
                http_client,
                Some(Alert::SurgeNotRunning)
            ));
        }

        // 2. Try HTTP API
        let http_client = SurgeHttpClient::new(
            config.api_base_url.clone(),
            config.api_key.clone(),
            config.request_timeout_ms,
        );

        if http_client.health_check().await.is_ok() {
            return Ok((
                SurgeClientMode::HttpApi(http_client.clone()),
                http_client,
                None  // No warning
            ));
        }

        // 3. HTTP API unavailable, fallback to CLI
        eprintln!("HTTP API unavailable, falling back to CLI mode");
        Ok((
            SurgeClientMode::Cli(system.cli.clone()),
            http_client,
            Some(Alert::HttpApiDisabled)  // Show warning
        ))
    }

    // User-triggered Surge start
    async fn start_surge(&self) -> Result<()> {
        self.system.start_surge().await?;
        // Wait for startup then health check
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    // User-triggered config reload
    async fn reload_config(&self) -> Result<()> {
        self.cli_client.reload().await?;
        // Wait for config to take effect
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    // Auto-fallback method example
    async fn get_policies(&self) -> Result<Vec<Policy>> {
        match &self.mode {
            SurgeClientMode::HttpApi(client) => {
                client.get_policies().await
            }
            SurgeClientMode::Cli(cli) => {
                cli.get_policies().await
            }
        }
    }

    // Health check and auto-recovery
    async fn ensure_healthy(&mut self) -> Result<()> {
        // Try health check
        if let SurgeClientMode::HttpApi(_) = &self.mode {
            if self.http_client.health_check().await.is_ok() {
                return Ok(());
            }
        }

        // Reselect mode
        let (mode, http) = Self::select_mode(&self.config, &self.system).await?;
        self.mode = mode;
        self.http_client = http;
        Ok(())
    }
}
```

### UI Layout Design

#### Overall Layout (Actual Implementation)

```
┌─ Surge TUI ──────────────────────────────────────────────────────┐
│                           Surge TUI                               │ ← Title
├──────────────────────────────────────────────────────────────────┤
│ 1.Overview  2.Policies  3.Requests  4.Active Connections        │ ← Tabs
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  System Overview                                                  │
│  Surge Status: Running ✓                                          │
│  HTTP API: Available ✓                                            │
│  Outbound Mode: Rule Mode                                         │
│                                                                   │
│  Statistics                                                       │
│    Policies: 25                                                   │
│    Policy Groups: 8                                               │
│    Active Connections: 12                                         │
│    Recent Requests: 50                                            │
│                                                                   │
├──────────────────────────────────────────────────────────────────┤
│ Surge Running (HTTP API)  [q]uit  [r]efresh  [1-4]view          │ ← btop-style status bar
└──────────────────────────────────────────────────────────────────┘

With Alert (HTTP API unavailable):
┌─ Surge TUI ──────────────────────────────────────────────────────┐
│                           Surge TUI                               │
├──────────────────────────────────────────────────────────────────┤
│ 1.Overview  2.Policies  3.Requests  4.Active Connections        │
├──────────────────────────────────────────────────────────────────┤
│ ⚠ HTTP API unavailable - Please add http-api in Surge config    │ ← Alert banner
│   Action: Press [R] to reload after modifying config            │
├──────────────────────────────────────────────────────────────────┤
│  (Content area...)                                                │
├──────────────────────────────────────────────────────────────────┤
│ Surge Running (CLI mode)  [q]uit  [r]efresh  [1-4]view  [r]eload│ ← Dynamic shortcuts
└──────────────────────────────────────────────────────────────────┘

When Surge not running:
┌─ Surge TUI ──────────────────────────────────────────────────────┐
│                           Surge TUI                               │
├──────────────────────────────────────────────────────────────────┤
│ 1.Overview  2.Policies  3.Requests  4.Active Connections        │
├──────────────────────────────────────────────────────────────────┤
│ ❌ Surge not running                                              │ ← Alert banner
│   Action: Press [S] to start Surge                               │
├──────────────────────────────────────────────────────────────────┤
│  (No data)                                                        │
├──────────────────────────────────────────────────────────────────┤
│ Surge not running  [q]uit  [s]tart                               │ ← Dynamic shortcuts
└──────────────────────────────────────────────────────────────────┘
```

#### View Hierarchy (Actual Implementation)

**Tab 1: Overview (System Overview)** - Default view
- Surge running status
- HTTP API availability
- Outbound mode
- Statistics (policy count, request count, etc.)

**Tab 2: Policies (Policy Groups)**
- Policy group list
- Each group shows policy list and selected state

**Tab 3: Requests (Recent Requests)**
- Recent request history (max 50 entries)
- Show URL, policy, upload/download size
- Status indicators: ✓ Complete / ✗ Failed / ○ In Progress

**Tab 4: Active Connections**
- Currently active network connections
- Same format as Requests view

### Interaction Design (Actual Implementation, btop Style)

#### Global Shortcuts
| Key | Function | Display Format |
|-----|----------|----------------|
| `q` / `Esc` | Quit | `[q]uit` |
| `1` / `2` / `3` / `4` | Switch Tab | `[1-4]view` |
| `r` | Manual refresh | `[r]efresh` |
| `R` | Reload Surge config (only with Alert) | `[r]eload` |
| `S` | Start Surge (only when not running) | `[s]tart` |

**Shortcut Design Principles**:
- ❌ Don't use F1-F12 function keys (user feedback: inconvenient)
- ✅ Use single-letter shortcuts (reference btop)
- ✅ Dynamic display: Show available shortcuts based on current state
- ✅ Inline display: Status bar directly shows `[x]action` format

**Status Bar Examples** (dynamically changing):
- Normal: `Surge Running (HTTP API)  [q]uit  [r]efresh  [1-4]view`
- HTTP API unavailable: `... [q]uit  [r]efresh  [1-4]view  [r]eload`
- Surge not running: `Surge not running  [q]uit  [s]tart`

#### View-Specific Shortcuts (Not Implemented)
Current version only supports global shortcuts. View-internal interactions (like policy selection, testing) are planned for future versions.

### State Management

```rust
#[derive(Debug, Clone)]
struct AppState {
    // Core data
    outbound: Option<OutboundState>,  // None means cannot fetch
    policy_groups: Vec<PolicyGroup>,
    policies: Vec<Policy>,
    recent_requests: Vec<Request>,
    active_requests: Vec<Request>,
    features: Option<Features>,       // None means cannot fetch

    // System status
    surge_running: bool,              // Surge process running
    http_api_available: bool,         // HTTP API available
    current_mode: ClientMode,         // HttpApi | Cli

    // UI state
    current_tab: Tab,
    selected_group_index: usize,
    selected_policy_index: usize,
    selected_request_index: usize,
    expanded_groups: HashSet<String>,  // Expanded policy groups
    filter_text: String,

    // Alerts and errors
    alert: Option<Alert>,             // Top warning/error banner
    last_update: Instant,
    loading: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Dashboard,
    Policies,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ClientMode {
    HttpApi,
    Cli,
}

#[derive(Debug, Clone)]
enum Alert {
    SurgeNotRunning,              // Surge not running, press S to start
    HttpApiDisabled,              // HTTP API unavailable, reload after config with R
    ConfigError(String),          // Config file error
    Warning(String),              // General warning
}
```

### Configuration File Format (Actual Implementation)

**Location**: `surge-tui.toml` (project root or specified path)

```toml
[surge]
# HTTP API configuration (required)
http_api_host = "127.0.0.1"
http_api_port = 6170
http_api_key = "keyp"  # Your Surge API Key

# surge-cli path (optional, auto-find by default)
# cli_path = "/Applications/Surge.app/Contents/Applications/surge-cli"

[ui]
# UI refresh interval (seconds), default 10
refresh_interval = 10

# Maximum request history entries, default 100
max_requests = 100
```

**Simplified Notes**:
- Removed unused `health`, `keybindings` config sections
- Refresh interval uses seconds (not milliseconds), more suitable for actual use
- Config file path can be specified with `-c` parameter

### CLI Arguments (Actual Implementation)

```bash
surge-tui [OPTIONS]

OPTIONS:
    -c, --config <PATH>     Specify config file path (default: surge-tui.toml)
    --help                  Show help
    --version               Show version
```

**Examples**:
```bash
# Use default config file (surge-tui.toml)
surge-tui

# Specify custom config
surge-tui -c ~/work/surge-config.toml

# Enable verbose logging via RUST_LOG environment variable
RUST_LOG=surge_tui=debug surge-tui
```

**Logging Configuration**:
- Default log level: `warn` (only show warnings and errors)
- Enable debug logs via `RUST_LOG=surge_tui=debug`
- Enable info logs via `RUST_LOG=surge_tui=info`

### Health Check and Fallback Strategy

#### Startup Health Check Flow

```
1. Read configuration (config.toml)
   ↓
2. Check if Surge process is running
   ├─ Yes → Continue step 3
   └─ No → Set alert = SurgeNotRunning
           Show error banner: "❌ Surge not running - Press S to start"
           Wait for user to press S or manually start
   ↓
3. Try HTTP API health check (GET /v1/outbound)
   ├─ Success → Set mode = HttpApi
           Clear alert
           Startup complete (normal mode)
   └─ Fail → Continue step 4
   ↓
4. Check if Surge config contains http-api
   ├─ No → Set alert = HttpApiDisabled
           Show warning banner: "⚠ HTTP API unavailable - Add http-api to config, then press R to reload"
           Continue step 5
   └─ Yes → Continue step 5 (API config exists but unavailable)
   ↓
5. Fallback to CLI mode
   ├─ Check surge-cli availability
   │   ├─ Success → Set mode = Cli
   │             Startup complete (CLI mode, limited functionality)
   │   └─ Fail → Show error: "surge-cli unavailable"
   │             Exit
   └─ Continue running (CLI mode, show warning banner at top)

User Actions:
- Press S (when Surge not running): Execute open -a Surge → Wait for startup → Re-run health check
- Press R (anytime): Execute surge-cli reload → Re-run health check
```

#### Runtime Health Check Flow

```
Execute every 30 seconds (configurable):

1. Is current mode HttpApi?
   ├─ Yes → GET /v1/outbound (lightweight check)
   │        ├─ Success → Continue
   │        └─ Fail → Mark for fallback, next request triggers mode reselection
   └─ No (CLI) → Try HTTP API health check
                 ├─ Success → Upgrade to HttpApi mode
                 └─ Fail → Continue using CLI
```

#### Request Failure Fallback Flow

```
User action (e.g., switch policy) → Call SurgeClient method
   ↓
1. Execute request using current mode
   ├─ HttpApi.select_policy() → Success → Return
   │                          → Fail → Continue step 2
   └─ CLI.select_policy()     → Success → Return
                               → Fail → Continue step 3
   ↓
2. Trigger ensure_healthy() to reselect mode
   ├─ Try to fix (restart Surge, fix config)
   └─ Re-execute request
       ├─ Success → Return
       └─ Fail → Step 3
   ↓
3. Return error to user
   ├─ UI shows red error banner
   └─ Log error
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
enum SurgeError {
    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Authentication failed: invalid API key")]
    AuthError,

    #[error("Connection timeout")]
    Timeout,

    #[error("Surge not running or API not enabled")]
    NotRunning,

    #[error("Invalid response format: {0}")]
    ParseError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("surge-cli command failed: {0}")]
    CliError(String),

    #[error("Surge config file not found")]
    ConfigFileNotFound,

    #[error("Failed to parse Surge config: {0}")]
    ConfigParseError(String),

    #[error("System command failed: {0}")]
    SystemCommandError(String),
}

// UI error display strategy
// - Network error: Top red banner, disappears after 3 seconds, triggers fallback
// - Auth error: Top red banner, persists, prompts to check config
// - Timeout: Top yellow banner, continues polling, marks connection unstable
// - CLI error: Top red banner, prompts to check surge-cli path
// - System command error: Top red banner, prompts to check system permissions
// - Config error: Popup prompt, blocks operation, guides user to fix
```

### Performance Requirements

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Startup Time | < 500ms | From execution to first screen render |
| Memory Usage | < 20MB | RSS after 1 hour running |
| CPU Usage | < 1% | Average during idle polling |
| Polling Latency | < 100ms | API response time (P95) |
| UI Refresh Rate | ≥ 30 FPS | ratatui render frame rate |

---

## Known Issues (Known Issues - Based on Actual Implementation)

### Critical (Must fix, otherwise unusable)

**Issue 1: API Key stored in plaintext**
- **Symptom**: API Key stored in plaintext in surge-tui.toml
- **Impact**: Config file leak would expose Surge API completely
- **Current Solution**:
  - Surge API only listens on 127.0.0.1 by default, restricting local access
  - Recommend setting file permissions to 600 (`chmod 600 surge-tui.toml`)
- **Fix Plan**: Support reading API Key from environment variable (`SURGE_API_KEY`)

### Important (Affects experience but can go live)

**Issue 2: Surge HTTP API response format doesn't match documentation**
- **Symptom**:
  - `/v1/policies` returns `{"proxies": [...], "policy-groups": [...]}`, not policy array
  - `/v1/policy_groups` returns HashMap, not array
  - Field names use camelCase (`policyName`), not snake_case
- **Impact**: Initial implementation needs actual API testing to parse correctly
- **Current Solution**: Handle actual format correctly in code (completed)
- **Fix Plan**: Report documentation issues to Surge official

**Issue 3: `/v1/policies/detail` returns config text instead of JSON**
- **Symptom**: Endpoint returns config file fragment (text), cannot use directly
- **Impact**: Cannot get policy detailed information (latency, alive status, etc.)
- **Current Solution**: Extract policy info from `policy_groups`, abandon detail endpoint
- **Fix Plan**: Implement policy test feature to get latency data

**Issue 4: CLI mode severely limited functionality**
- **Symptom**: After falling back to CLI mode, cannot get real-time requests, active connections, etc.
- **Impact**: User experience significantly degraded, can only show basic status
- **Current Solution**: Top Alert reminds user to configure HTTP API
- **Fix Plan**: None (surge-cli limitation)

**Issue 5: Polling mode latency**
- **Symptom**: Default refresh every 10 seconds, high latency
- **Impact**: Cannot see new requests in real-time
- **Current Solution**:
  - User can adjust `ui.refresh_interval` config
  - Support manual refresh by pressing `r`
- **Fix Plan**: Wait for Surge official WebSocket support

**Issue 6: Text truncation doesn't consider emoji width**
- **Symptom**: Emoji in policy names may cause display misalignment
- **Impact**: Minor display misalignment, doesn't affect usability
- **Current Solution**: Use `.chars().count()` for Unicode-safe truncation
- **Fix Plan**: Use unicode-width crate to accurately calculate display width

**Issue 7: Log info mixed into UI output**
- **Symptom**: Debug logs print to terminal, breaking TUI display
- **Impact**: UI display chaos
- **Current Solution**:
  - Default log level set to `warn`
  - Remove all debug JSON prints
- **Fix Plan**: Output logs to file instead of stderr

### Nice to have (Optimization items)

**Issue 8: No policy latency test feature**
- **Symptom**: Can only see policy list, cannot test latency
- **Impact**: Cannot judge node quality
- **Current Solution**: Manually use surge-cli test-policy
- **Fix Plan**: v1.1 add policy test interaction

**Issue 9: No policy group selection feature**
- **Symptom**: Can only view policy groups, cannot switch selected policy
- **Impact**: Cannot replace GUI operations
- **Current Solution**: Manually use surge-cli or GUI
- **Fix Plan**: v1.1 add policy group interaction

**Issue 10: No color theme customization**
- **Symptom**: Colors hardcoded in code
- **Impact**: Different terminal color schemes may not coordinate
- **Current Solution**: Use terminal theme colors
- **Fix Plan**: v1.2 support theme configuration

---

## Implementation Plan (Implementation Plan - Actual Completion Status)

### Phase 1: MVP (v0.1, Actual 3 days) ✅ Completed

- [x] Technical research (`docs/research.md`)
- [x] Project initialization (Cargo project)
- [x] **Domain layer** (models, entities, errors - zero dependencies)
- [x] **Infrastructure layer**
  - [x] HTTP API client (core endpoints)
  - [x] CLI client (surge-cli wrapper)
  - [x] System client (pgrep/open)
- [x] **Application layer** (SurgeClient - unified interface, fallback logic)
- [x] **UI layer** (ratatui + crossterm)
  - [x] Basic framework (title, tabs, status bar)
  - [x] 4 views (Overview, Policies, Requests, ActiveConnections)
  - [x] Alert component (warning banner)
  - [x] btop-style shortcuts
- [x] Config file loading (TOML - surge-tui.toml)
- [x] User actions (S start Surge, R reload config, r manual refresh)
- [x] Error handling and logging (tracing, default warn level)

**Actual Delivery**:
- ✅ Can view system overview, policy groups, recent requests, active connections
- ✅ Show warning when HTTP API unavailable, prompt user to reload config
- ✅ Show error when Surge not running, prompt user to start
- ✅ Clean log output, doesn't interfere with UI display

**Actual Issues Encountered**:
- API response format doesn't match documentation (resolved)
- Logs mixed into UI output (resolved: changed to warn level)
- F5 shortcut inappropriate (resolved: changed to r)
- Text truncation emoji issue (resolved: Unicode-safe truncation)

### Phase 2: Complete Features (v0.5, Planned) ⏳ Not Started

- [ ] Policy group interaction (select policy)
- [ ] Policy latency testing
- [ ] Connection termination feature
- [ ] Outbound mode switching
- [ ] Feature toggle switching (MITM, Capture, etc.)

**Delivery Standard**: Can fully control Surge through TUI, no GUI needed

### Phase 3: Optimization and Release (v1.0, Planned) ⏳ Not Started

- [ ] Performance optimization (virtual scrolling, caching)
- [ ] User documentation (README, usage guide)
- [ ] Package release (Homebrew, cargo install)
- [ ] Unit tests
- [ ] Integration tests

**Delivery Standard**: Stable and reliable, ready for public release

---

## Project Structure (Project Structure - Actual Implementation)

### Directory Layout

```
surge-tui/
├── Cargo.toml                 # Project config
├── Cargo.lock
├── .gitignore
├── .mcp.json                  # MCP config (development tool)
├── surge-tui.toml             # Application config file
├── docs/
│   ├── research.md            # Technical research document
│   └── requirements.md        # Requirements document (this file)
│
└── src/
    ├── main.rs                # Program entry
    ├── lib.rs                 # Library entry
    │
    ├── config/                # Configuration management
    │   ├── mod.rs
    │   └── loader.rs          # Load surge-tui.toml
    │
    ├── domain/                # Domain model (zero dependencies)
    │   ├── mod.rs
    │   ├── models.rs          # Data models (match actual API)
    │   ├── entities.rs        # AppSnapshot, Alert, ViewMode
    │   └── errors.rs          # SurgeError, Result
    │
    ├── infrastructure/        # Infrastructure layer
    │   ├── mod.rs
    │   ├── http_client.rs     # SurgeHttpClient (HTTP API)
    │   ├── cli_client.rs      # SurgeCliClient (surge-cli)
    │   └── system_client.rs   # SurgeSystemClient (pgrep/open)
    │
    ├── application/           # Application layer
    │   ├── mod.rs
    │   └── surge_client.rs    # SurgeClient (unified interface)
    │
    └── ui/                    # UI layer
        ├── mod.rs
        ├── app.rs             # App main structure, event loop
        └── components/        # UI components
            ├── mod.rs
            ├── overview.rs    # Overview component
            ├── policies.rs    # Policy group component
            ├── requests.rs    # Request list component
            └── alerts.rs      # Alert banner component
```

### Layered Architecture (Clean Architecture)

```
Dependency direction: Outer depends on inner, inner doesn't depend on outer

ui → application → domain
     ↓
infrastructure → domain
```

**Layer Responsibilities**:

| Layer | Responsibility | Dependencies |
|-------|---------------|--------------|
| **domain** | Pure data structures, business rules | Zero dependencies |
| **infrastructure** | External calls (HTTP, CLI, system commands) | domain |
| **application** | Business logic coordination, fallback strategy | domain + infrastructure |
| **ui** | Interface rendering, user interaction | application |

### Core File Responsibilities

#### `main.rs` - Program Entry

```rust
// Responsibility: Start app, assemble dependencies, top-level error handling
use surge_tui::{config, application, ui};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize logging
    tracing_subscriber::fmt::init();

    // 2. Load configuration
    let config = config::load()?;

    // 3. Create client
    let client = application::SurgeClient::new(config).await?;

    // 4. Start TUI
    let mut app = ui::App::new(client);
    app.run().await?;

    Ok(())
}
```

#### `domain/models.rs` - Core Data

```rust
// Responsibility: Define core data structures, zero dependencies, serializable
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Policy {
    pub name: String,
    #[serde(rename = "type")]
    pub policy_type: String,
    pub alive: Option<bool>,
    pub latency: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolicyGroup {
    pub name: String,
    pub group_type: String,
    pub policies: Vec<String>,
    pub selected: Option<String>,
}
```

#### `infrastructure/http_client.rs` - HTTP Implementation

```rust
// Responsibility: HTTP API calls, no business logic
use crate::domain::Policy;
use reqwest::Client;
use anyhow::Result;

pub struct SurgeHttpClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl SurgeHttpClient {
    pub fn new(base_url: String, api_key: String, timeout_ms: u64) -> Self {
        // ...
    }

    pub async fn get_policies(&self) -> Result<Vec<Policy>> {
        // GET /v1/policies
    }
}
```

#### `application/surge_client.rs` - Unified Interface

```rust
// Responsibility: Coordinate HTTP/CLI, fallback logic, health check
use crate::{infrastructure::*, domain::*};

pub enum ClientMode {
    Http(SurgeHttpClient),
    Cli(SurgeCliClient),
}

pub struct SurgeClient {
    mode: ClientMode,
    system: SurgeSystemClient,
}

impl SurgeClient {
    pub async fn get_policies(&self) -> Result<Vec<Policy>> {
        match &self.mode {
            ClientMode::Http(c) => c.get_policies().await,
            ClientMode::Cli(c) => c.get_policies().await,
        }
    }
}
```

#### `ui/app.rs` - TUI Main Loop

```rust
// Responsibility: Event loop, user input, state updates
use crate::{application::SurgeClient, ui::state::AppState};
use crossterm::event::{self, Event, KeyCode};

pub struct App {
    client: SurgeClient,
    state: AppState,
}

impl App {
    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            self.render()?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('S') => self.handle_start_surge().await?,
                    KeyCode::Char('R') => self.handle_reload().await?,
                    _ => {}
                }
            }

            self.tick().await?;
        }
        Ok(())
    }
}
```

### Initialize Project

```bash
# Create project
cargo new surge-tui
cd surge-tui

# Create directory structure
mkdir -p src/{config,domain,infrastructure,application,ui/{views,components}}
mkdir -p docs tests

# Create module files
touch src/lib.rs src/error.rs
touch src/config/{mod.rs,loader.rs}
touch src/domain/{mod.rs,models.rs,outbound.rs,alert.rs}
touch src/infrastructure/{mod.rs,http_client.rs,cli_client.rs,system_client.rs}
touch src/application/{mod.rs,surge_client.rs,health_checker.rs}
touch src/ui/{mod.rs,app.rs,state.rs,renderer.rs}
touch src/ui/views/{mod.rs,dashboard.rs,policies.rs,settings.rs}
touch src/ui/components/{mod.rs,alert_bar.rs,policy_list.rs,request_list.rs}

# Initialize Git
git init
cat > .gitignore << 'EOF'
/target
/Cargo.lock
.DS_Store
EOF
```

### Design Principles

#### 1. Single Responsibility (SRP)

Each module does one thing:
- `http_client.rs` - Only handles HTTP API calls
- `cli_client.rs` - Only handles surge-cli execution
- `system_client.rs` - Only handles system commands
- `surge_client.rs` - Coordinates the above three

#### 2. Dependency Inversion (DIP)

High-level doesn't depend on low-level, both depend on abstractions:
```rust
// Can define trait abstraction
trait PolicyProvider {
    async fn get_policies(&self) -> Result<Vec<Policy>>;
}

impl PolicyProvider for SurgeHttpClient { ... }
impl PolicyProvider for SurgeCliClient { ... }
```

#### 3. Open/Closed Principle (OCP)

Add features without modifying existing code:
- Add WebSocket client? Add `infrastructure/ws_client.rs`
- Add view? Add `ui/views/new_view.rs`
- Existing code unchanged

#### 4. Interface Segregation (ISP)

Clients only expose needed interfaces:
```rust
// HTTP client doesn't expose CLI-specific methods
impl SurgeHttpClient {
    pub async fn get_policies(&self) -> Result<Vec<Policy>>;
    // Don't expose execute() and other internal methods
}
```

---

## Dependencies (Dependencies - Actually Used)

### Rust Crates (Actual Versions)

```toml
[dependencies]
# TUI framework
ratatui = "0.29"
crossterm = "0.28"

# HTTP client
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Async runtime
tokio = { version = "1.45", features = ["full"] }

# Configuration management
toml = "0.8"

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Process management
# Call pgrep/open/surge-cli through tokio::process::Command
```

**Simplified Notes**:
- Removed unused `dirs` (config file path simplified)
- Removed dev-dependencies (no tests written yet)
- Using latest stable versions

### External Dependencies

- **Surge Mac** 4.0+ - Required, provides HTTP API and surge-cli
- **macOS** - Required, depends on pgrep, open and other system commands
- **surge-cli** - Optional, used for fallback mode (usually installed with Surge.app)

---

## Alternatives Considered (Alternative Solutions)

### Option 1: Web UI (Browser-based)

**Advantages**: Cross-platform, easy development, supports rich interactions
**Disadvantages**: Needs web server, cannot use directly via SSH, high resource usage
**Conclusion**: Doesn't meet SSH scenario requirements

### Option 2: CLI Command-line Tool (Not TUI)

**Advantages**: Simplest, no UI complexity
**Disadvantages**: Need to remember many commands, no real-time status, low operation efficiency
**Conclusion**: Official `surge-cli` already provides, no need to duplicate

### Option 3: Tmux/Screen Multi-window Script

**Advantages**: Leverage existing tools, flexible
**Disadvantages**: Complex configuration, poor interactivity, not easy to distribute
**Conclusion**: Suitable for personal customization, not for general tools

---

## Success Metrics (Success Metrics)

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **Usage Frequency** | > 3 times daily | Personal usage log |
| **Time Saved** | < 30 seconds per operation | Timing comparison (GUI vs TUI) |
| **Stability** | > 8 hours continuous run without crash | Actual usage testing |
| **User Feedback** | GitHub Stars > 50 (within 6 months) | Statistics after open source |

---

## Next Steps (Next Steps - v0.5 Plan)

### Completed (v0.1) ✅
1. ✅ Technical research and requirements document
2. ✅ Project initialization and architecture design
3. ✅ HTTP/CLI/System three-layer clients
4. ✅ Basic TUI framework (4 views)
5. ✅ Alert mechanism and fallback handling
6. ✅ Config file loading
7. ✅ Logging and error handling

### Next Steps (v0.5) ⏳
1. **Add policy group interaction**
   - Support selecting policies in Policies view
   - Call `/v1/policy_groups/select` to switch policies
   - Show operation feedback

2. **Add policy test feature**
   - Test single policy latency
   - Test policy groups
   - Show test progress and results

3. **Add connection management**
   - Support selection in Requests/ActiveConnections views
   - Terminate selected connections

4. **Optimize UI**
   - Support scrolling (long lists)
   - Support selection highlighting
   - Improve error prompts

5. **Complete documentation**
   - README usage instructions
   - Installation guide
   - FAQ

---

## Appendix (Appendix)

### A. Surge HTTP API Quick Reference

See [`research.md`](research.md) Section 2.4

### B. ratatui Example Code

```rust
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let items = vec![
                ListItem::new("Policy 1"),
                ListItem::new("Policy 2"),
            ];
            let list = List::new(items)
                .block(Block::default().title("Policies").borders(Borders::ALL));
            f.render_widget(list, f.size());
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    Ok(())
}
```

### C. Finding surge-cli Path

If `surge_cli_path` is not specified in the config file, TUI will search in the following order:

```bash
# 1. Standard installation path
/Applications/Surge.app/Contents/Applications/surge-cli

# 2. PATH environment variable
which surge-cli

# 3. Common symlinks
/usr/local/bin/surge-cli
```

**Manually create symlink**:
```bash
sudo ln -s \
  /Applications/Surge.app/Contents/Applications/surge-cli \
  /usr/local/bin/surge-cli
```

---

**Document End - Awaiting Review**
