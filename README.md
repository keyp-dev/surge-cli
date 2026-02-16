# Surge TUI

[English](README.md) | [简体中文](README.zh-CN.md)

A terminal user interface (TUI) for macOS Surge proxy management.

## Features

- ✅ **Terminal-Friendly** - Pure text interface with SSH remote support
- ✅ **Multi-Language Support** - Compile-time language selection (Chinese/English), zero runtime overhead
- ✅ **Clean Architecture** - Clear layered architecture
- ✅ **Fallback Strategy** - HTTP API → CLI → System three-layer fallback
- ✅ **Alert Mechanism** - User-controlled error handling (no automatic config modification)
- ✅ **Real-Time Monitoring** - Policy, requests, connections, and DNS status
- ✅ **Search Functionality** - Independent search for policy groups/requests/connections
- ✅ **Grouping Mode** - Group requests/connections by application name
- ✅ **Connection Management** - Terminate individual connections with confirmation dialogs
- ✅ **DNS Management** - DNS cache view and one-click flush
- ✅ **Help System** - Built-in help popup with keyboard shortcuts

## Architecture

```
surge-tui/
├── src/
│   ├── domain/          # Core business logic (zero dependencies)
│   │   ├── models.rs    # Data models
│   │   ├── entities.rs  # Business entities
│   │   └── errors.rs    # Error definitions
│   ├── infrastructure/  # Infrastructure implementations
│   │   ├── http_client.rs   # HTTP API client
│   │   ├── cli_client.rs    # surge-cli client
│   │   └── system_client.rs # System command client
│   ├── application/     # Business coordination layer
│   │   └── surge_client.rs  # Unified interface
│   ├── ui/             # User interface
│   │   ├── app.rs       # Main application state
│   │   └── components/  # UI components
│   └── config/         # Configuration management
└── docs/              # Design documents
```

## Installation

### via Homebrew (Recommended)

```bash
# English version (default)
brew tap keyp-dev/tap
brew install surge-tui

# Chinese version
brew install surge-tui-zh
```

### via Nix

```bash
# English version
nix profile install github:keyp-dev/surge-cli

# Chinese version
nix profile install github:keyp-dev/surge-cli#surge-tui-zh
```

### Build from Source

```bash
# English version (default)
cargo build --release
cargo install --path .

# Chinese version
cargo build --release --no-default-features --features lang-zh-cn
cargo install --path . --no-default-features --features lang-zh-cn
```

**Supported languages:**
- 🇺🇸 American English (`lang-en-us`) - Default
- 🇨🇳 Simplified Chinese (`lang-zh-cn`)

## Quick Start

### 1. Prerequisites

Ensure Surge is installed and running on macOS, and enable HTTP API in the configuration file:

```ini
[General]
http-api = your-secret-key@127.0.0.1:6171
http-api-tls = false
```

### 2. Configuration

### 2. Configuration

Create a configuration file `surge-tui.toml`:

```toml
[surge]
http_api_host = "127.0.0.1"
http_api_port = 6171
http_api_key = "your-secret-key"  # Match the API Key in Surge config

[ui]
refresh_interval = 1
max_requests = 100
```

Or configure via environment variables:

```bash
export SURGE_HTTP_API_KEY="your-secret-key"
```

### 3. Run

```bash
# Run locally
surge-tui

# Run remotely via SSH
ssh user@mac-host surge-tui
```

## Usage

### Core Features

- ✅ **Non-Blocking Latency Testing** - Press `T` to test all policies while keeping UI responsive
- ✅ **Nested Policy Group Support** - Recursively display final policy latency in policy group chains
- ✅ **Smart Notification System** - Real-time status bar notifications + history view (`N` key)
- ✅ **Search Functionality** - Press `/` to search policy groups/requests/connections with real-time filtering
- ✅ **Grouping Mode** - Press `G` to toggle request/connection grouping by application name
- ✅ **Help System** - Press `H` to open help popup showing all keyboard shortcuts
- ✅ **Connection Management** - Press `K` to terminate selected connection with confirmation dialog
- ✅ **DNS Management** - View DNS cache in 5th view, press `F` to flush all cache
- ✅ **Feature Toggles** - Keyboard shortcuts to toggle outbound mode(`M`), MITM(`I`), traffic capture(`C`)
- ✅ **Enhanced Request Details** - Notes syntax highlighting, HTTP Body markers
- ✅ **Developer Tools** - Press <code>`</code> to open DevTools for debug logs
- ✅ **Latency Color Coding** - Cyan(<100ms) / Yellow(100-300ms) / Red(>300ms)
- ✅ **Test Result Caching** - Preserve latency data after refresh or view switching

### Keyboard Shortcuts

| Key | Function | Description |
|------|------|------|
| `q` | Quit | Exit program |
| `r` | Refresh | Manually refresh snapshot / reload config (when Alert prompts) |
| `1-5` | Switch View | Overview/Policies/Requests/Connections/DNS |
| `↑/↓` | Navigate | Move up/down in lists |
| `Enter` | Enter/Confirm | Enter policy group or switch policy |
| `Esc` | Back/Close | Exit policy group or close popup |
| `h` / `H` | Help | Open help popup showing all keyboard shortcuts |
| `/` | Search | Search policy groups/requests/connections |
| `g` / `G` | Group Mode | Group requests/connections by application name |
| `t` / `T` | Test Latency | Non-blocking test all policy latencies |
| `m` / `M` | Toggle Mode | Cycle through Direct/Proxy/Rule |
| `i` / `I` | Toggle MITM | Toggle MITM status in Overview view |
| `c` / `C` | Toggle Capture | Toggle traffic capture in Overview view |
| `k` / `K` | Kill Connection | Terminate selected connection in Connections view (with confirmation) |
| `f` / `F` | Flush Cache | Flush DNS cache in DNS view |
| `n` / `N` | Notification History | View complete notification history (50 items) |
| <code>`</code> | DevTools | Open developer debug tools |
| `s` / `S` | Start Surge | Only available when Alert prompts |

Keyboard shortcuts are displayed directly in the bottom status bar (similar to btop).

### Views

#### 1. Overview
- Surge running status, HTTP API availability
- Current outbound mode (`M` key for quick toggle)
- MITM status (`I` key for quick toggle)
- Traffic capture status (`C` key for quick toggle)
- System statistics

#### 2. Policies
- **Left**: Policy group list with currently selected policy
- **Right**: Policy details and latency (supports nested policy groups)
- **Search**: `/` key to search policy groups
- **Test**: `T` key to test all policy latencies

#### 3. Requests
- Recent request records (URL, policy, traffic stats)
- **Search**: `/` key to search requests
- **Group**: `G` key to group by application name
- **Details**: Notes highlighting, HTTP Body markers

#### 4. Connections
- Current active network connections
- **Search**: `/` key to search connections
- **Group**: `G` key to group by application name
- **Manage**: `K` key to terminate selected connection (with confirmation)

#### 5. DNS Cache
- DNS cache records (domain, IP, TTL)
- **Search**: `/` key to search domains
- **Flush**: `F` key to flush all DNS cache

## Fallback Strategy

surge-tui implements a three-layer fallback mechanism to ensure it works in various situations:

1. **HTTP API** (Priority) - Most complete features, best performance
2. **surge-cli** (Fallback) - Automatically switches when HTTP API is unavailable
3. **System Commands** (Last Resort) - Check process status, start Surge

### Alert Mechanism

Does not automatically modify configuration, but prompts users through Alerts:

- **Surge Not Running** → Shows "Press S to start Surge"
- **HTTP API Unavailable** → Shows "Press R to reload config" (user must manually add http-api config first)

## Development

### Build

```bash
cargo check  # Check code
cargo build  # Debug build
cargo build --release  # Release build
```

### Architecture Principles

- **Single Responsibility** - Each module responsible for one thing
- **Dependency Inversion** - Dependency flow: UI → Application → Infrastructure → Domain
- **Open/Closed** - Easy to extend without modifying existing code
- **Zero Domain Dependencies** - Domain layer has zero external dependencies

## Troubleshooting

### Surge Not Running

```bash
# Check Surge process
pgrep -x Surge

# Start Surge
open -a Surge
```

### HTTP API Unavailable

Check Surge configuration file (usually in `~/.config/surge/` or Surge.app config directory):

```ini
[General]
http-api = your-secret-key@127.0.0.1:6171
```

After adding, press `R` in surge-tui to reload configuration.

### surge-cli Not Found

surge-cli is located inside Surge.app:

```bash
/Applications/Surge.app/Contents/Applications/surge-cli
```

You can manually specify the path in the configuration file.

## Tech Stack

- **Rust** - Systems programming language
- **tokio** - Async runtime
- **ratatui** - Terminal UI framework
- **reqwest** - HTTP client
- **serde** - Serialization/deserialization

## License

MIT

## Documentation

Detailed documentation in the `docs/` directory:

- **[FEATURES.md](docs/FEATURES.md)** - Detailed description of implemented features (recommended reading)
- [requirements.md](docs/requirements.md) - Detailed requirements documentation
- [research.md](docs/research.md) - Surge CLI/HTTP API technical research

---

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
