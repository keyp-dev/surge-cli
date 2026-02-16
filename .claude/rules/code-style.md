# Code Style Guidelines

surge-tui follows Rust best practices and SOLID principles. Code should be clear, concise, and testable.

## Core Principles

### 1. Single Responsibility Principle

**Each module, struct, and function should do one thing only.**

**Good example:**
```rust
// ✅ http_client only handles HTTP calls
pub struct HttpClient {
    base_url: String,
    api_key: String,
}

// ✅ surge_client coordinates multiple clients
pub struct SurgeClient {
    http: HttpClient,
    cli: CliClient,
    system: SystemClient,
}
```

**Bad example:**
```rust
// ❌ Mixing HTTP calls and UI logic
pub struct PolicyManager {
    http: HttpClient,
    selected_index: usize,  // UI state
    test_results: HashMap<String, u32>,  // Test data
}
```

**Why?** Single responsibility reduces coupling, makes testing easier, and provides a single reason to change.

### 2. Dependency Inversion Principle

**Depend on abstractions, not concretions.** High-level modules should not depend on low-level modules.

**Manifestation:**
- Domain has zero dependencies
- Infrastructure implements concrete clients
- Application coordinates abstract interfaces
- UI calls application, not infrastructure directly

**Check:** Domain should not `use reqwest` or `use ratatui`.

### 3. Open/Closed Principle

**Open for extension, closed for modification.**

**Good example:**
```rust
// ✅ Adding new language doesn't require trait modification
pub trait Translate: Send + Sync {
    fn ui_status_running(&self) -> &'static str;
}

// Extension: just add a new language file
pub struct FrFR;
impl Translate for FrFR { /* ... */ }
```

**Bad example:**
```rust
// ❌ Adding new language requires modifying all matches
fn get_text(lang: &str, key: &str) -> &str {
    match (lang, key) {
        ("zh", "running") => "运行中",
        ("en", "running") => "Running",
        // Need to modify here for new language
    }
}
```

## Rust Code Style

### Naming

**Follow Rust standards:**
- `snake_case` - functions, variables, modules, fields
- `PascalCase` - types, traits, enum variants
- `SCREAMING_SNAKE_CASE` - constants, static variables

**Descriptive naming:**
```rust
// ✅ Clear
let selected_policy_index = 0;
let http_api_enabled = true;

// ❌ Vague
let idx = 0;
let enabled = true;
```

**Boolean variables:**
```rust
// ✅ Question form
is_running: bool
has_error: bool
can_retry: bool

// ❌ Verb form
check_running: bool
```

### Functions

**Keep functions small.** Functions should be short and do one thing.

**Standards:**
- < 20 lines - ideal
- 20-50 lines - acceptable
- \> 50 lines - consider splitting

**Single level of abstraction:**
```rust
// ✅ Single abstraction level
async fn get_policies(&self) -> Result<Vec<Policy>> {
    let response = self.http.get("/policies").await?;
    self.parse_policies(response)
}

// ❌ Mixed abstraction levels
async fn get_policies(&self) -> Result<Vec<Policy>> {
    let response = self.http.get("/policies").await?;
    let json: Value = serde_json::from_str(&response)?;
    let policies = json["policies"].as_array().unwrap();
    // ... more parsing logic
}
```

**Extract functions instead of comments:**
```rust
// ✅ Extract function
fn is_policy_available(latency: Option<u32>) -> bool {
    latency.is_some()
}

if is_policy_available(policy.latency) {
    // ...
}

// ❌ Comment
// Check if policy is available
if policy.latency.is_some() {
    // ...
}
```

### Error Handling

**Use the type system to express errors.** Don't use `panic!`, `unwrap()` (unless it truly can't fail).

**Use `Result` and `?` operator:**
```rust
// ✅ Correct
pub async fn get_status(&self) -> Result<Status> {
    let response = self.http.get("/status").await?;
    Ok(serde_json::from_str(&response)?)
}

// ❌ Wrong
pub async fn get_status(&self) -> Status {
    let response = self.http.get("/status").await.unwrap();
    serde_json::from_str(&response).unwrap()
}
```

**Domain errors should be clear:**
```rust
// ✅ Business errors
#[derive(Debug, thiserror::Error)]
pub enum SurgeError {
    #[error("Surge is not running")]
    NotRunning,

    #[error("HTTP API is disabled")]
    ApiDisabled,

    #[error("Policy group '{0}' not found")]
    PolicyGroupNotFound(String),
}
```

**Use `anyhow` to simplify application layer errors:**
```rust
// application/infrastructure can use anyhow::Result
pub async fn test_policy(&self, name: &str) -> anyhow::Result<u32> {
    // ...
}
```

### Types and Structs

**Don't over-abstract.** Only use traits when truly needed.

**Prefer concrete types:**
```rust
// ✅ Use concrete types for simple scenarios
pub struct App {
    client: SurgeClient,
}

// ❌ Unnecessary abstraction
pub struct App<C: Client> {
    client: C,
}
```

**Use newtype for type safety:**
```rust
// ✅ Type safe
pub struct PolicyName(String);
pub struct GroupName(String);

fn select_policy(group: GroupName, policy: PolicyName) {
    // Compiler prevents parameter order mistakes
}

// ❌ Error prone
fn select_policy(group: String, policy: String) {
    // Might swap parameters
}
```

**Builder pattern for complex construction:**
```rust
// ✅ Use builder for many parameters
let client = SurgeClient::builder()
    .http_api("127.0.0.1:6171", "key")
    .cli_path("/path/to/surge-cli")
    .build()?;
```

### Async Code

**async functions should return `Result`:**
```rust
// ✅ Clear
pub async fn fetch_policies(&self) -> Result<Vec<Policy>> {
    // ...
}

// ❌ Hides errors
pub async fn fetch_policies(&self) -> Vec<Policy> {
    // Errors are swallowed
}
```

**Avoid `.await` chains:**
```rust
// ✅ Step by step
let response = self.http.get("/policies").await?;
let policies = self.parse(response)?;
Ok(policies)

// ❌ Hard to debug
Ok(self.parse(self.http.get("/policies").await?)?)
```

**Use `tokio::spawn` for concurrent tasks:**
```rust
// ✅ Non-blocking latency test
for policy in policies {
    tokio::spawn(async move {
        test_latency(policy).await
    });
}

// ❌ Blocking
for policy in policies {
    test_latency(policy).await;  // Sequential execution
}
```

### Comments

**Code should be self-explanatory.** Comments explain "why", not "what".

**Good comments:**
```rust
// ✅ Explain why
// Surge CLI returns JSON missing group info, need to fetch from HTTP API
let groups = self.http.get_policy_groups().await?;

// ✅ Important context
/// Fallback strategy: HTTP API → surge-cli → system commands
/// HTTP API provides most complete data, use it first
pub async fn get_status(&self) -> Result<Status> {
    // ...
}
```

**Bad comments:**
```rust
// ❌ Repeating code
// Get policy list
let policies = self.get_policies().await?;

// ❌ Outdated comment
// TODO: Implement error handling
pub fn process(&self) -> Result<()> {
    // Already implemented, but comment not updated
}
```

**Use `///` for doc comments:**
```rust
/// Test latency for all policies
///
/// # Non-blocking
/// Uses tokio::spawn for concurrent testing, doesn't block UI
///
/// # Errors
/// Returns `SurgeError::ApiDisabled` if HTTP API is unavailable
pub async fn test_all_policies(&self) -> Result<TestResults> {
    // ...
}
```

### Module Organization

**Module file structure:**
```rust
// Small module (< 200 lines)
mod.rs  // Direct implementation

// Large module (> 200 lines)
mod.rs           // pub use
submodule_a.rs   // Implementation
submodule_b.rs   // Implementation
```

**Clear exports:**
```rust
// mod.rs
mod http_client;
mod cli_client;
mod system_client;

// Only export public interface
pub use http_client::HttpClient;
pub use cli_client::CliClient;
pub use system_client::SystemClient;

// Don't export internal types
```

## Code Checks

**Pre-compilation checks:**
```bash
# Format
cargo fmt

# Clippy check
cargo clippy -- -D warnings

# Compile all features
cargo check --all-features

# Run tests
cargo test
```

**CI should run:**
- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build --release`

## Performance

**Don't optimize prematurely.** Clear code first, optimize after performance issues appear.

**Use `&str` instead of `String` (when possible):**
```rust
// ✅ Avoid allocation
fn display_name(&self) -> &str {
    &self.name
}

// ❌ Unnecessary allocation
fn display_name(&self) -> String {
    self.name.clone()
}
```

**Reuse buffers:**
```rust
// ✅ Reuse
let mut buffer = Vec::new();
for item in items {
    buffer.clear();
    write!(&mut buffer, "{}", item)?;
}

// ❌ Repeated allocation
for item in items {
    let buffer = format!("{}", item);
}
```

**Lazy computation:**
```rust
// ✅ Only compute when needed
if condition {
    let expensive = compute_expensive();
}

// ❌ Always compute
let expensive = compute_expensive();
if condition {
    // use expensive
}
```

## Testing

**Unit tests in the same file:**
```rust
// http_client.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response() {
        // ...
    }
}
```

**Clear test naming:**
```rust
#[test]
fn test_http_client_retries_on_timeout() {
    // ...
}

#[test]
fn test_fallback_to_cli_when_http_fails() {
    // ...
}
```

## Anti-patterns

**Don't:**
- ❌ Overuse `clone()` - consider references or `Rc`/`Arc`
- ❌ Use `.unwrap()` - use `?` or `unwrap_or_default()`
- ❌ Ignore compiler warnings - fix all warnings
- ❌ Excessive nesting - use early returns (guard clauses)
- ❌ Huge match statements - split into functions
- ❌ Mutable global state - use parameter passing

---

*"Clean code is simple and direct. Clean code reads like well-written prose."*
