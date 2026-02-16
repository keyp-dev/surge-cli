# Project Structure Guidelines

surge-tui follows Clean Architecture. Dependency flow is unidirectional: **UI → Application → Infrastructure → Domain**.

## Directory Structure

```
src/
├── domain/          # Business core (zero dependencies)
│   ├── models.rs    # Data models
│   ├── entities.rs  # Business entities
│   └── errors.rs    # Error definitions
├── infrastructure/  # External dependency implementations
│   ├── http_client.rs   # Surge HTTP API client
│   ├── cli_client.rs    # surge-cli client
│   └── system_client.rs # System command client
├── application/     # Business coordination layer
│   └── surge_client.rs  # Unified facade, fallback strategy
├── ui/             # User interface
│   ├── app.rs       # Main application state
│   └── components/  # TUI components
├── i18n/           # Internationalization support
│   ├── mod.rs       # Translate trait
│   ├── zh_cn.rs     # Simplified Chinese
│   └── en_us.rs     # US English
└── config/         # Configuration management
    └── config.rs
```

## Layering Principles

### Domain (Business Core)

**Zero external dependencies.** Does not depend on tokio, reqwest, ratatui, or any other layer.

**Only contains:**
- Data models (Surge API response structures)
- Business entities (policies, policy groups, requests)
- Error types (business errors, not HTTP errors)

**判断标准 (Criterion):** Can this code be reused in a completely different framework? If yes, it belongs to domain. If no, it's infrastructure or UI.

### Infrastructure (Foundation)

**Interacts with the external world.** HTTP requests, CLI calls, system commands, file I/O all go here.

**Each client is independent:**
- `http_client.rs` - Only calls Surge HTTP API
- `cli_client.rs` - Only calls surge-cli
- `system_client.rs` - Only executes system commands

**Depends on domain, called by application.** Not directly called by UI.

### Application (Business Coordination)

**Unified entry point.** UI only accesses all functionality through `SurgeClient`.

**Implements fallback strategy:** HTTP API → surge-cli → system commands.

**Does not contain UI logic.** Only coordinates multiple infrastructure clients.

### UI (User Interface)

**ratatui components.** Only responsible for rendering and user interaction.

**Depends on application and i18n.** Does not directly call infrastructure.

**Components are independent:** One file per view (overview, policies, requests, alerts, notifications).

### i18n (Internationalization)

**Compile-time language selection.** Through Cargo feature flags: `lang-zh-cn`, `lang-en-us`.

**Zero runtime overhead.** `&'static str` directly inlined, no runtime lookup.

**UI layer only.** Domain and infrastructure don't contain translations.

## Dependency Rules

**Allowed dependency directions:**
```
ui → application → infrastructure → domain
ui → i18n
ui → config
```

**Forbidden dependencies:**
- domain → any layer (domain has zero dependencies)
- infrastructure → ui
- application → ui
- i18n → domain

**Checking method:** `cargo tree --edges normal`, domain should have no dependencies except serde.

## Guide for Adding New Features

### Adding New Surge API Call

1. **models.rs** - Add response data structure
2. **http_client.rs** - Implement HTTP call
3. **surge_client.rs** - Expose in facade
4. **ui/components/** - Use in corresponding component

### Adding New UI Component

1. **ui/components/{name}.rs** - Create component
2. **ui/components/mod.rs** - Export component
3. **ui/app.rs** - Integrate in App
4. **i18n/mod.rs** - Add translation interface
5. **i18n/{lang}.rs** - Implement translations

### Adding New Language

1. **Cargo.toml** - Add feature: `lang-{code}`
2. **i18n/{code}.rs** - Implement Translate trait
3. **i18n/mod.rs** - Add `#[cfg(feature = "lang-{code}")]` block
4. **README.md** - Update documentation

## Anti-patterns

**Don't:**
- ❌ Introduce `tokio::spawn` in domain
- ❌ Introduce `ratatui::widgets` in infrastructure
- ❌ Call `reqwest::get` directly in UI
- ❌ Hardcode text in domain/infrastructure (should be in i18n)
- ❌ Cross-layer direct access (must go through application layer)

**Why?** Violates dependency inversion. Domain is depended upon by infrastructure, domain should not depend on external frameworks. UI and infrastructure should not be directly coupled.

## File Naming

- Module files: `snake_case.rs`
- Components: describe function (`policies.rs`, `overview.rs`)
- Clients: `{target}_client.rs` (`http_client.rs`)

## When to Split Modules

**Split criteria:**
- Single file exceeds 500 lines
- Single function can be independently reused
- Different reasons for change (SRP)

**Don't split prematurely:** Files smaller than 200 lines usually don't need splitting.

---

*"The only way to go fast is to go well."*
