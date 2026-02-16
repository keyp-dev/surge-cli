/// Application layer - Business logic coordination
///
/// Dependencies: Domain + Infrastructure
pub mod surge_client;

// Re-export
pub use surge_client::{ClientMode, SurgeClient};
