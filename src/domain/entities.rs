/// Domain business entities
///
/// Zero dependency - Pure business logic objects
use super::models::{DnsRecord, OutboundMode, PolicyDetail, PolicyGroup, Request};

/// UI view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Overview
    Overview,
    /// Policy management
    Policies,
    /// Request list
    Requests,
    /// Active connections
    ActiveConnections,
    /// DNS cache
    Dns,
}

impl ViewMode {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Overview,
            Self::Policies,
            Self::Requests,
            Self::ActiveConnections,
            Self::Dns,
        ]
    }
}

/// Alert type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertLevel {
    /// Information
    Info,
    /// Warning
    Warning,
    /// Error (requires user action)
    Error,
}

/// User action prompt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertAction {
    /// Start Surge (press S)
    StartSurge,
    /// Reload config (press R)
    ReloadConfig,
    /// No action
    None,
}

impl AlertAction {
    // Removed as_str() - translation happens in UI layer
}

/// Alert message
#[derive(Debug, Clone)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub action: AlertAction,
}

impl Alert {
    /// Surge not running
    pub fn surge_not_running() -> Self {
        Self {
            level: AlertLevel::Error,
            message: "surge_not_running".to_string(), // Message key for i18n
            action: AlertAction::StartSurge,
        }
    }

    /// HTTP API unavailable
    pub fn http_api_disabled() -> Self {
        Self {
            level: AlertLevel::Error,
            message: "http_api_disabled".to_string(), // Message key for i18n
            action: AlertAction::ReloadConfig,
        }
    }

    /// Config error
    pub fn config_error(message: String) -> Self {
        Self {
            level: AlertLevel::Warning,
            message: format!("config_error: {}", message),
            action: AlertAction::ReloadConfig,
        }
    }

    /// General warning
    pub fn warning(message: String) -> Self {
        Self {
            level: AlertLevel::Warning,
            message,
            action: AlertAction::None,
        }
    }

    /// Info prompt
    pub fn info(message: String) -> Self {
        Self {
            level: AlertLevel::Info,
            message,
            action: AlertAction::None,
        }
    }
}

/// Application state snapshot
#[derive(Debug, Clone)]
pub struct AppSnapshot {
    /// Whether Surge is running
    pub surge_running: bool,
    /// Whether HTTP API is available
    pub http_api_available: bool,
    /// Current outbound mode
    pub outbound_mode: Option<OutboundMode>,
    /// Whether MITM is enabled
    pub mitm_enabled: Option<bool>,
    /// Whether Capture is enabled
    pub capture_enabled: Option<bool>,
    /// Policy list
    pub policies: Vec<PolicyDetail>,
    /// Policy group list
    pub policy_groups: Vec<PolicyGroup>,
    /// Recent requests
    pub recent_requests: Vec<Request>,
    /// Active connections
    pub active_connections: Vec<Request>,
    /// DNS cache
    pub dns_cache: Vec<DnsRecord>,
    /// Current alerts
    pub alerts: Vec<Alert>,
}

impl AppSnapshot {
    pub fn new() -> Self {
        Self {
            surge_running: false,
            http_api_available: false,
            outbound_mode: None,
            mitm_enabled: None,
            capture_enabled: None,
            policies: Vec::new(),
            policy_groups: Vec::new(),
            recent_requests: Vec::new(),
            active_connections: Vec::new(),
            dns_cache: Vec::new(),
            alerts: Vec::new(),
        }
    }

    /// Add alert
    pub fn add_alert(&mut self, alert: Alert) {
        self.alerts.push(alert);
    }

    /// Clear all alerts
    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }

    /// Whether there are error-level alerts
    pub fn has_errors(&self) -> bool {
        self.alerts
            .iter()
            .any(|a| matches!(a.level, AlertLevel::Error))
    }
}

impl Default for AppSnapshot {
    fn default() -> Self {
        Self::new()
    }
}
