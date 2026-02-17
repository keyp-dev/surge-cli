/// Infrastructure layer - external service implementations
///
/// Depends on external services: HTTP API, CLI, System
pub mod cli_client;
pub mod http_client;
pub mod system_client;

// Re-export clients
pub use cli_client::SurgeCliClient;
pub use http_client::SurgeHttpClient;
pub use system_client::SurgeSystemClient;
