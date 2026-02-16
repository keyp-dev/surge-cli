# Multilingual Support Design Document

> Pragmatic solution based on Clean Architecture and SOLID principles

**Implementation Status:** ✅ Basic architecture implemented (Phase 1 complete)

**Supported Languages:** 🇨🇳 Simplified Chinese (Default) | 🇺🇸 US English

---

## Design Principles

### YAGNI - You Aren't Gonna Need It

**Don't over-engineer.** This is a TUI tool, not a web application:

- Total text volume is small (~200 translations)
- No need for plural forms (1 item / 2 items)
- No need for gender variations (he/she/it)
- No need for runtime language switching
- No need for dynamic translation file loading

**What we need:**
- Compile-time translation selection
- Type-safe translation keys
- Zero runtime overhead
- Easy to add new languages

### Open/Closed Principle

Adding new languages should **only require implementing trait**, without modifying existing code.

```rust
// Add Japanese support:
impl Translate for JaJP {
    fn ui_status_running(&self) -> &'static str {
        "Surge 実行中"
    }
    // ... other methods
}

// No need to modify any existing code
```

### Dependency Inversion

UI layer depends on abstract `Translate` trait, not concrete language implementations.

```rust
// UI code
fn render_status(t: &impl Translate) {
    println!("{}", t.ui_status_running());
}

// Concrete language injected at compile time
```

---

## Architecture Design

### File Structure

```
src/
├── i18n/
│   ├── mod.rs              # Trait definition + public interface
│   ├── zh_cn.rs            # Simplified Chinese implementation
│   ├── en_us.rs            # US English implementation
│   └── keys.rs             # Translation key constants (optional)
├── ui/
│   ├── app.rs              # Uses t.ui_xxx() methods
│   └── components/         # Uses t.component_xxx() methods
└── main.rs                 # Select language implementation
```

### Core Trait

```rust
// src/i18n/mod.rs
pub trait Translate {
    // UI status bar
    fn ui_status_running(&self) -> &'static str;
    fn ui_status_stopped(&self) -> &'static str;
    fn ui_status_http_api(&self) -> &'static str;

    // Shortcut descriptions
    fn key_quit(&self) -> &'static str;
    fn key_refresh(&self) -> &'static str;
    fn key_test(&self) -> &'static str;

    // View titles
    fn view_overview(&self) -> &'static str;
    fn view_policies(&self) -> &'static str;
    fn view_requests(&self) -> &'static str;
    fn view_connections(&self) -> &'static str;

    // Notification messages
    fn notification_test_started(&self) -> &'static str;
    fn notification_test_completed(&self, alive: usize, total: usize) -> String;
    fn notification_test_failed(&self, error: &str) -> String;

    // Alert messages
    fn alert_surge_not_running(&self) -> &'static str;
    fn alert_http_api_disabled(&self) -> &'static str;

    // Policy groups
    fn policy_group_enter(&self) -> &'static str;
    fn policy_group_testing(&self) -> &'static str;
    fn policy_available(&self) -> &'static str;
    fn policy_unavailable(&self) -> &'static str;

    // DevTools
    fn devtools_title(&self) -> &'static str;
    fn devtools_no_logs(&self) -> &'static str;

    // Notification history
    fn notification_history_title(&self) -> &'static str;
    fn notification_history_empty(&self) -> &'static str;
}

// Organize translation methods by module for easy maintenance
// Naming convention: {module}_{function}_{specific_item}
```

### Language Implementation Example

**Simplified Chinese**

```rust
// src/i18n/zh_cn.rs
pub struct ZhCN;

impl Translate for ZhCN {
    fn ui_status_running(&self) -> &'static str {
        "Surge 运行中"
    }

    fn ui_status_stopped(&self) -> &'static str {
        "Surge 未运行"
    }

    fn ui_status_http_api(&self) -> &'static str {
        "(HTTP API)"
    }

    fn key_quit(&self) -> &'static str {
        "[q]uit"
    }

    fn key_refresh(&self) -> &'static str {
        "[r]efresh"
    }

    fn key_test(&self) -> &'static str {
        "[t]est"
    }

    fn view_overview(&self) -> &'static str {
        "总览"
    }

    fn view_policies(&self) -> &'static str {
        "策略"
    }

    fn view_requests(&self) -> &'static str {
        "请求历史"
    }

    fn view_connections(&self) -> &'static str {
        "活跃连接"
    }

    fn notification_test_started(&self) -> &'static str {
        "策略延迟测试已启动..."
    }

    fn notification_test_completed(&self, alive: usize, total: usize) -> String {
        format!("测试完成: {}/{} 可用", alive, total)
    }

    fn notification_test_failed(&self, error: &str) -> String {
        format!("测试失败: {}", error)
    }

    fn alert_surge_not_running(&self) -> &'static str {
        "Surge 未运行 - 按 S 启动"
    }

    fn alert_http_api_disabled(&self) -> &'static str {
        "HTTP API 不可用 - 按 R 重载配置"
    }

    fn policy_group_enter(&self) -> &'static str {
        "[Enter进入]"
    }

    fn policy_group_testing(&self) -> &'static str {
        "[测试中...]"
    }

    fn policy_available(&self) -> &'static str {
        "[可用]"
    }

    fn policy_unavailable(&self) -> &'static str {
        "[不可用]"
    }

    fn devtools_title(&self) -> &'static str {
        "DevTools [ESC 关闭]"
    }

    fn devtools_no_logs(&self) -> &'static str {
        "无日志记录"
    }

    fn notification_history_title(&self) -> &'static str {
        "通知历史 [ESC 关闭]"
    }

    fn notification_history_empty(&self) -> &'static str {
        "无通知历史"
    }
}
```

**US English**

```rust
// src/i18n/en_us.rs
pub struct EnUS;

impl Translate for EnUS {
    fn ui_status_running(&self) -> &'static str {
        "Surge Running"
    }

    fn ui_status_stopped(&self) -> &'static str {
        "Surge Stopped"
    }

    fn ui_status_http_api(&self) -> &'static str {
        "(HTTP API)"
    }

    fn key_quit(&self) -> &'static str {
        "[q]uit"
    }

    fn key_refresh(&self) -> &'static str {
        "[r]efresh"
    }

    fn key_test(&self) -> &'static str {
        "[t]est"
    }

    fn view_overview(&self) -> &'static str {
        "Overview"
    }

    fn view_policies(&self) -> &'static str {
        "Policies"
    }

    fn view_requests(&self) -> &'static str {
        "Requests"
    }

    fn view_connections(&self) -> &'static str {
        "Connections"
    }

    fn notification_test_started(&self) -> &'static str {
        "Policy latency test started..."
    }

    fn notification_test_completed(&self, alive: usize, total: usize) -> String {
        format!("Test completed: {}/{} available", alive, total)
    }

    fn notification_test_failed(&self, error: &str) -> String {
        format!("Test failed: {}", error)
    }

    fn alert_surge_not_running(&self) -> &'static str {
        "Surge not running - Press S to start"
    }

    fn alert_http_api_disabled(&self) -> &'static str {
        "HTTP API unavailable - Press R to reload config"
    }

    fn policy_group_enter(&self) -> &'static str {
        "[Enter to open]"
    }

    fn policy_group_testing(&self) -> &'static str {
        "[Testing...]"
    }

    fn policy_available(&self) -> &'static str {
        "[Available]"
    }

    fn policy_unavailable(&self) -> &'static str {
        "[Unavailable]"
    }

    fn devtools_title(&self) -> &'static str {
        "DevTools [ESC to close]"
    }

    fn devtools_no_logs(&self) -> &'static str {
        "No logs"
    }

    fn notification_history_title(&self) -> &'static str {
        "Notification History [ESC to close]"
    }

    fn notification_history_empty(&self) -> &'static str {
        "No notifications"
    }
}
```

---

## Compile-time Language Selection

### Option 1: Feature Flags (Recommended)

**Cargo.toml**
```toml
[features]
default = ["lang-zh-cn"]
lang-zh-cn = []
lang-en-us = []
```

**src/i18n/mod.rs**
```rust
mod zh_cn;
mod en_us;

pub trait Translate {
    // ... trait definition
}

// Compile-time language selection
#[cfg(feature = "lang-zh-cn")]
pub type CurrentLang = zh_cn::ZhCN;

#[cfg(feature = "lang-en-us")]
pub type CurrentLang = en_us::EnUS;

// Provide global singleton
pub fn current() -> &'static CurrentLang {
    static INSTANCE: CurrentLang = CurrentLang;
    &INSTANCE
}
```

**Usage**
```bash
# Build Chinese version
cargo build --release

# Build English version
cargo build --release --no-default-features --features lang-en-us
```

### Option 2: Environment Variables

**build.rs** (Build script)
```rust
fn main() {
    let lang = std::env::var("SURGE_TUI_LANG").unwrap_or_else(|_| "zh-cn".to_string());

    match lang.as_str() {
        "zh-cn" => println!("cargo:rustc-cfg=lang_zh_cn"),
        "en-us" => println!("cargo:rustc-cfg=lang_en_us"),
        _ => println!("cargo:rustc-cfg=lang_zh_cn"), // Default Chinese
    }
}
```

**Usage**
```bash
# Build Chinese version
cargo build --release

# Build English version
SURGE_TUI_LANG=en-us cargo build --release
```

---

## UI Layer Integration

### App State Integration

```rust
// src/ui/app.rs
use crate::i18n::{self, Translate};

pub struct App {
    // ... existing fields
    t: &'static dyn Translate,  // Translation interface
}

impl App {
    pub fn new(client: SurgeClient, refresh_interval_secs: u64) -> Self {
        Self {
            // ... existing initialization
            t: i18n::current(),  // Get current language
        }
    }

    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_text = if self.snapshot.surge_running {
            if self.snapshot.http_api_available {
                format!("{} {}",
                    self.t.ui_status_running(),
                    self.t.ui_status_http_api()
                )
            } else {
                self.t.ui_status_running().to_string()
            }
        } else {
            self.t.ui_status_stopped().to_string()
        };

        // Shortcut text
        let shortcuts = vec![
            self.t.key_quit(),
            self.t.key_refresh(),
            self.t.key_test(),
        ];

        // ... rendering logic
    }

    fn handle_test_message(&mut self, msg: TestMessage) {
        match msg {
            TestMessage::Started => {
                self.add_notification(Notification::info(
                    self.t.notification_test_started().to_string()
                ));
            }
            TestMessage::Completed { results, .. } => {
                let alive_count = results.iter().filter(|p| p.alive).count();
                self.add_notification(Notification::success(
                    self.t.notification_test_completed(alive_count, results.len())
                ));
            }
            TestMessage::Failed { error } => {
                self.add_notification(Notification::error(
                    self.t.notification_test_failed(&error)
                ));
            }
        }
    }
}
```

### Component Layer Integration

```rust
// src/ui/components/policies.rs
pub fn render(
    f: &mut Frame,
    area: Rect,
    snapshot: &AppSnapshot,
    selected: usize,
    policy_detail_index: Option<usize>,
    testing_group: Option<&str>,
    t: &dyn Translate,  // Add translation parameter
) {
    // ... use t.policy_xxx() methods
}
```

---

## Advantages

### 1. Compile-time Checking

```rust
// ✅ Correct
t.ui_status_running()

// ❌ Compile error: method does not exist
t.ui_status_runnig()  // Spelling error

// ❌ Compile error: missing parameters
t.notification_test_completed()
```

All translation keys are methods, spelling errors detected at compile time.

### 2. Zero Runtime Overhead

- Translation selected at compile time
- No HashMap lookups
- No file I/O
- No runtime parsing

### 3. Type Safety

```rust
// Dynamic parameter type checking
fn notification_test_completed(&self, alive: usize, total: usize) -> String;

// ❌ Compile error: type mismatch
t.notification_test_completed("5", 10)
```

### 4. Easy to Extend

Adding new language only requires:
1. Implement `Translate` trait
2. Add feature flag
3. No need to modify existing code

### 5. Testable

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockTranslate;

    impl Translate for MockTranslate {
        fn ui_status_running(&self) -> &'static str {
            "TEST_RUNNING"
        }
        // ... mock other methods
    }

    #[test]
    fn test_render_status() {
        let t = MockTranslate;
        let result = render_status(&t);
        assert_eq!(result, "TEST_RUNNING");
    }
}
```

---

## Disadvantages and Tradeoffs

### Disadvantages

1. **Requires recompilation to switch language**
   - Cannot switch at runtime
   - Each language needs independent build

2. **Adding translations requires modifying code**
   - Cannot load from external files
   - Adding new translation keys requires modifying trait

3. **Translations scattered in code**
   - No centralized translation file
   - Hard to export for translators

### Why These Tradeoffs Are Acceptable?

**Runtime language switching**
- TUI tools typically don't switch languages frequently after installation
- Can support multiple languages via multiple binaries (`surge-tui-zh`, `surge-tui-en`)

**Modifying code to add translations**
- Small text volume (~200 entries)
- Add translations when adding features, stay in sync
- Compile-time checking avoids omissions

**Scattered translations**
- Can extract all translation methods via script to generate comparison table
- For small projects, translations in code easier to maintain

---

## Alternative Solutions

### fluent-rs (Mozilla Fluent)

**Advantages:**
- Runtime loading of translation files
- Supports complex features like plurals, gender
- Independent translation files, easy collaboration

**Disadvantages:**
- Runtime parsing overhead
- No compile-time checking (spelling errors discovered at runtime)
- Introduces complex dependencies
- Over-engineering (for TUI tools)

### rust-i18n

**Advantages:**
- Macro-generated translation functions
- YAML/JSON translation files
- Runtime language switching

**Disadvantages:**
- Macro magic, hard to debug
- No strong type checking
- Runtime overhead

### gettext

**Advantages:**
- Mature ecosystem
- Complete toolchain

**Disadvantages:**
- C dependencies
- Not idiomatic Rust
- Runtime overhead

---

## Implementation Plan

### Phase 1: Basic Architecture (1-2 days)

1. Create `src/i18n/` module
2. Define `Translate` trait
3. Implement `ZhCN` and `EnUS`
4. Add compile-time selection mechanism

### Phase 2: UI Integration (2-3 days)

1. Refactor `App` to integrate translation interface
2. Update all UI components
3. Update notification messages
4. Update DevTools

### Phase 3: Testing & Documentation (1 day)

1. Test Chinese and English versions
2. Write usage documentation
3. Update README

---

## Future Extensions

### Adding More Languages

Only need to implement trait:

```rust
// src/i18n/ja_jp.rs
pub struct JaJP;

impl Translate for JaJP {
    fn ui_status_running(&self) -> &'static str {
        "Surge 実行中"
    }
    // ... other methods
}
```

### Export Translation Comparison Table

```bash
# Generate translation comparison table script
cargo run --bin extract-translations > translations.csv
```

---

## Summary

This is a **pragmatic solution**:

- ✅ Simple and clear, no complex dependencies
- ✅ Compile-time checking, type-safe
- ✅ Zero runtime overhead
- ✅ Follows SOLID principles
- ✅ Easy to extend

For TUI tools, this is the most appropriate solution.

---

*"Truth can only be found in one place: the code."*
*— Robert C. Martin*

*Generated with [Claude Code](https://claude.ai/code)*
*via [Happy](https://happy.engineering)*

*Co-Authored-By: Claude <noreply@anthropic.com>*
