/// Surge unified client
///
/// Application layer - Coordinates three infrastructure layers: HTTP/CLI/System
/// Implements fallback strategy: HTTP API → CLI → System
use crate::config::Config;
use crate::domain::{
    entities::{Alert, AppSnapshot},
    errors::{Result, SurgeError},
    models::*,
};
use crate::infrastructure::{SurgeCliClient, SurgeHttpClient, SurgeSystemClient};

/// Client mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientMode {
    /// HTTP API mode (preferred)
    HttpApi,
    /// CLI mode (fallback)
    Cli,
}

/// Surge unified client
#[derive(Clone)]
pub struct SurgeClient {
    mode: ClientMode,
    http_client: SurgeHttpClient,
    cli_client: SurgeCliClient,
    system_client: SurgeSystemClient,
}

impl SurgeClient {
    /// Create new client
    pub fn new(config: Config) -> Self {
        let http_client = SurgeHttpClient::new(
            config.surge.http_api_host.clone(),
            config.surge.http_api_port,
            config.surge.http_api_key.clone(),
        );

        let cli_client = SurgeCliClient::new(config.surge.cli_path.clone());
        let system_client = SurgeSystemClient::new();

        Self {
            mode: ClientMode::HttpApi, // Default to HTTP API
            http_client,
            cli_client,
            system_client,
        }
    }

    /// Get current mode
    pub fn mode(&self) -> ClientMode {
        self.mode
    }

    /// Detect and switch to best available mode
    pub async fn detect_mode(&mut self) -> ClientMode {
        // Try HTTP API first
        if self.http_client.is_available().await {
            self.mode = ClientMode::HttpApi;
        } else {
            // Fallback to CLI
            self.mode = ClientMode::Cli;
        }
        self.mode
    }

    /// Get application snapshot (includes all states and alerts)
    pub async fn get_snapshot(&mut self) -> AppSnapshot {
        let mut snapshot = AppSnapshot::new();

        // 1. Check Surge process
        snapshot.surge_running = self.system_client.is_surge_running().await;
        if !snapshot.surge_running {
            snapshot.add_alert(Alert::surge_not_running());
            return snapshot;
        }

        // 2. Detect best mode
        self.detect_mode().await;

        // 3. Check HTTP API availability
        snapshot.http_api_available = matches!(self.mode, ClientMode::HttpApi);
        if !snapshot.http_api_available {
            snapshot.add_alert(Alert::http_api_disabled());
        }

        // 4. Get outbound mode
        if let Ok(mode) = self.get_outbound_mode().await {
            snapshot.outbound_mode = Some(mode);
        }

        // 5. Get MITM and Capture status (HTTP API mode only)
        if snapshot.http_api_available {
            if let Ok(mitm) = self.get_mitm_status().await {
                snapshot.mitm_enabled = Some(mitm);
            }
            if let Ok(capture) = self.get_capture_status().await {
                snapshot.capture_enabled = Some(capture);
            }
        }

        // 6. Get policy information (HTTP API mode only)
        if snapshot.http_api_available {
            // Get policy groups
            match self.http_client.get_policy_groups().await {
                Ok(groups) => {
                    tracing::debug!("Fetched {} policy groups", groups.len());
                    snapshot.policy_groups = groups;

                    // Note: HTTP API does not provide latency data
                    // Latency data can only be obtained through CLI's test_all_policies_with_latency()
                    // When user presses T key, background test will be triggered and update snapshot.policies
                }
                Err(e) => tracing::error!("Failed to fetch policy groups: {}", e),
            }

            match self.http_client.get_recent_requests().await {
                Ok(requests) => {
                    tracing::debug!("Fetched {} recent requests", requests.len());
                    snapshot.recent_requests = requests;
                }
                Err(e) => tracing::error!("Failed to fetch recent requests: {}", e),
            }

            match self.http_client.get_active_connections().await {
                Ok(connections) => {
                    tracing::debug!("Fetched {} active connections", connections.len());
                    snapshot.active_connections = connections;
                }
                Err(e) => tracing::error!("Failed to fetch active connections: {}", e),
            }

            // Get DNS cache
            match self.http_client.get_dns_cache().await {
                Ok(dns_cache) => {
                    tracing::debug!("Fetched {} DNS cache entries", dns_cache.len());
                    snapshot.dns_cache = dns_cache;
                }
                Err(e) => tracing::error!("Failed to fetch DNS cache: {}", e),
            }
        }

        snapshot
    }

    // ===== Outbound mode =====

    /// Get outbound mode
    pub async fn get_outbound_mode(&self) -> Result<OutboundMode> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.get_outbound_mode().await,
            ClientMode::Cli => {
                // CLI does not directly support getting outbound mode, need to parse dump policy
                Err(SurgeError::HttpApiUnavailable {
                    reason: "CLI mode does not support this operation".to_string(),
                })
            }
        }
    }

    /// Set outbound mode
    pub async fn set_outbound_mode(&self, mode: OutboundMode) -> Result<()> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.set_outbound_mode(mode).await,
            ClientMode::Cli => Err(SurgeError::HttpApiUnavailable {
                reason: "CLI mode does not support this operation".to_string(),
            }),
        }
    }

    // ===== Policy management =====

    /// Test policy latency
    pub async fn test_policy(&self, name: &str) -> Result<()> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.test_policy(name).await,
            ClientMode::Cli => {
                self.cli_client.test_policy(name).await?;
                Ok(())
            }
        }
    }

    /// Select policy in policy group
    pub async fn select_policy_group(&self, group_name: &str, policy: &str) -> Result<()> {
        match self.mode {
            ClientMode::HttpApi => {
                self.http_client
                    .select_policy_group(group_name, policy)
                    .await
            }
            ClientMode::Cli => Err(SurgeError::HttpApiUnavailable {
                reason: "CLI mode does not support this operation".to_string(),
            }),
        }
    }

    /// Test policy group, return available policy list
    pub async fn test_policy_group(&self, group_name: &str) -> Result<Vec<String>> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.test_policy_group(group_name).await,
            ClientMode::Cli => {
                self.cli_client.test_group(group_name).await?;
                // CLI mode does not return available list
                Ok(Vec::new())
            }
        }
    }

    /// Test all policies and return latency data (CLI mode only)
    pub async fn test_all_policies_with_latency(&self) -> Result<Vec<PolicyDetail>> {
        // Only CLI mode supports latency retrieval
        let test_results = self.cli_client.test_all_policies().await?;

        let policies = test_results
            .into_iter()
            .map(|(name, latency, alive)| PolicyDetail {
                name,
                policy_type: PolicyType::Direct, // Temporary value
                alive,
                latency,
                last_test_at: None,
            })
            .collect();

        Ok(policies)
    }

    // ===== Connection management =====

    /// Kill connection
    pub async fn kill_connection(&self, id: u64) -> Result<()> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.kill_connection(id).await,
            ClientMode::Cli => self.cli_client.kill_connection(id).await,
        }
    }

    // ===== Configuration management =====

    /// Reload configuration
    pub async fn reload_config(&self) -> Result<()> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.reload_config().await,
            ClientMode::Cli => self.cli_client.reload_config().await,
        }
    }

    // ===== DNS =====

    /// Get DNS cache
    pub async fn get_dns_cache(&self) -> Result<Vec<DnsRecord>> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.get_dns_cache().await,
            ClientMode::Cli => Err(SurgeError::HttpApiUnavailable {
                reason: "CLI mode does not support this operation".to_string(),
            }),
        }
    }

    /// Flush DNS cache
    pub async fn flush_dns(&self) -> Result<()> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.flush_dns().await,
            ClientMode::Cli => self.cli_client.flush_dns().await,
        }
    }

    // ===== Feature toggles =====

    /// Get MITM status
    pub async fn get_mitm_status(&self) -> Result<bool> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.get_mitm_status().await,
            ClientMode::Cli => Err(SurgeError::HttpApiUnavailable {
                reason: "CLI mode does not support this operation".to_string(),
            }),
        }
    }

    /// Set MITM status
    pub async fn set_mitm_status(&self, enabled: bool) -> Result<()> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.set_mitm_status(enabled).await,
            ClientMode::Cli => Err(SurgeError::HttpApiUnavailable {
                reason: "CLI mode does not support this operation".to_string(),
            }),
        }
    }

    /// Get traffic capture status
    pub async fn get_capture_status(&self) -> Result<bool> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.get_capture_status().await,
            ClientMode::Cli => Err(SurgeError::HttpApiUnavailable {
                reason: "CLI mode does not support this operation".to_string(),
            }),
        }
    }

    /// Set traffic capture status
    pub async fn set_capture_status(&self, enabled: bool) -> Result<()> {
        match self.mode {
            ClientMode::HttpApi => self.http_client.set_capture_status(enabled).await,
            ClientMode::Cli => Err(SurgeError::HttpApiUnavailable {
                reason: "CLI mode does not support this operation".to_string(),
            }),
        }
    }

    // ===== System-level operations =====

    /// Start Surge
    pub async fn start_surge(&self) -> Result<()> {
        self.system_client.start_surge().await
    }

    /// Check if Surge is running
    pub async fn is_surge_running(&self) -> bool {
        self.system_client.is_surge_running().await
    }
}
