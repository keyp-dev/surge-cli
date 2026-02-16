/// Domain layer - Core business logic
///
/// Zero dependency principle: No dependencies on infrastructure layer (HTTP, CLI, System)
pub mod entities;
pub mod errors;
pub mod models;

// Re-export commonly used types
pub use entities::{Alert, AlertAction, AlertLevel, AppSnapshot, ViewMode};
pub use errors::{Result, SurgeError};
pub use models::*;
