/// Domain layer error types
///
/// Zero dependency - No dependencies on infrastructure layer implementation
use std::fmt;

/// Surge operation error
#[derive(Debug, Clone)]
pub enum SurgeError {
    /// Surge process not running
    SurgeNotRunning,

    /// HTTP API unavailable (not enabled or cannot connect)
    HttpApiUnavailable { reason: String },

    /// CLI execution failed
    CliExecutionFailed { command: String, error: String },

    /// Config file error
    ConfigError { message: String },

    /// Policy not found
    PolicyNotFound { name: String },

    /// Policy group not found
    PolicyGroupNotFound { name: String },

    /// Connection not found
    ConnectionNotFound { id: u64 },

    /// Parse error (JSON or text)
    ParseError { source: String, error: String },

    /// Network error
    NetworkError { message: String },

    /// Permission denied
    PermissionDenied { message: String },

    /// Other unknown error
    Unknown { message: String },
}

impl fmt::Display for SurgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurgeNotRunning => {
                write!(f, "Surge is not running")
            }
            Self::HttpApiUnavailable { reason } => {
                write!(f, "HTTP API unavailable: {}", reason)
            }
            Self::CliExecutionFailed { command, error } => {
                write!(f, "CLI command failed: {} - {}", command, error)
            }
            Self::ConfigError { message } => {
                write!(f, "Config error: {}", message)
            }
            Self::PolicyNotFound { name } => {
                write!(f, "Policy not found: {}", name)
            }
            Self::PolicyGroupNotFound { name } => {
                write!(f, "Policy group not found: {}", name)
            }
            Self::ConnectionNotFound { id } => {
                write!(f, "Connection not found: #{}", id)
            }
            Self::ParseError { source, error } => {
                write!(f, "Parse error ({}): {}", source, error)
            }
            Self::NetworkError { message } => {
                write!(f, "Network error: {}", message)
            }
            Self::PermissionDenied { message } => {
                write!(f, "Permission denied: {}", message)
            }
            Self::Unknown { message } => {
                write!(f, "Unknown error: {}", message)
            }
        }
    }
}

impl std::error::Error for SurgeError {}

/// Result type alias
pub type Result<T> = std::result::Result<T, SurgeError>;
