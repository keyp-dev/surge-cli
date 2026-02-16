/// Surge TUI - SSH remote control tool
///
/// Clean Architecture layers:
/// - domain: Core business logic (zero dependencies)
/// - infrastructure: Infrastructure implementation (HTTP/CLI/System)
/// - application: Business coordination layer
/// - ui: User interface layer
/// - config: Configuration management
pub mod application;
pub mod config;
pub mod domain;
pub mod i18n;
pub mod infrastructure;
pub mod ui;

// Re-export commonly used types
pub use application::{ClientMode, SurgeClient};
pub use config::Config;
pub use domain::{
    entities::{Alert, AppSnapshot, ViewMode},
    errors::{Result, SurgeError},
};
pub use i18n::Translate;
pub use ui::App;
