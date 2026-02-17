/// surge-cli client
///
/// Interacts with Surge via the surge-cli command-line tool
use crate::domain::errors::{Result, SurgeError};
use tokio::process::Command;

/// surge-cli client
#[derive(Clone)]
pub struct SurgeCliClient {
    cli_path: String,
}

impl SurgeCliClient {
    /// Create new CLI client
    pub fn new(cli_path: Option<String>) -> Self {
        let cli_path = cli_path.unwrap_or_else(|| {
            "/Applications/Surge.app/Contents/Applications/surge-cli".to_string()
        });
        Self { cli_path }
    }

    /// Execute a surge-cli command
    async fn execute(&self, args: &[&str]) -> Result<String> {
        let output = Command::new(&self.cli_path)
            .args(args)
            .output()
            .await
            .map_err(|e| SurgeError::CliExecutionFailed {
                command: format!("{} {}", self.cli_path, args.join(" ")),
                error: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SurgeError::CliExecutionFailed {
                command: format!("{} {}", self.cli_path, args.join(" ")),
                error: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }

    /// Execute a command and return JSON output
    async fn execute_json(&self, args: &[&str]) -> Result<serde_json::Value> {
        // Add --raw flag to get JSON output
        let mut json_args = vec!["--raw"];
        json_args.extend_from_slice(args);

        let output = self.execute(&json_args).await?;
        serde_json::from_str(&output).map_err(|e| SurgeError::ParseError {
            source: "CLI JSON".to_string(),
            error: e.to_string(),
        })
    }

    /// Reload configuration
    pub async fn reload_config(&self) -> Result<()> {
        self.execute(&["reload"]).await?;
        Ok(())
    }

    /// Switch profile
    pub async fn switch_profile(&self, name: &str) -> Result<()> {
        self.execute(&["switch-profile", name]).await?;
        Ok(())
    }

    /// Get active connections
    pub async fn dump_active(&self) -> Result<serde_json::Value> {
        self.execute_json(&["dump", "active"]).await
    }

    /// Get recent requests
    pub async fn dump_requests(&self) -> Result<serde_json::Value> {
        self.execute_json(&["dump", "request"]).await
    }

    /// Get rule list
    pub async fn dump_rules(&self) -> Result<serde_json::Value> {
        self.execute_json(&["dump", "rule"]).await
    }

    /// Get policy list
    pub async fn dump_policies(&self) -> Result<serde_json::Value> {
        self.execute_json(&["dump", "policy"]).await
    }

    /// Get DNS cache
    pub async fn dump_dns(&self) -> Result<serde_json::Value> {
        self.execute_json(&["dump", "dns"]).await
    }

    /// Get profile content
    pub async fn dump_profile(&self, effective: bool) -> Result<String> {
        let profile_type = if effective { "effective" } else { "original" };
        self.execute(&["dump", "profile", profile_type]).await
    }

    /// Test network latency
    pub async fn test_network(&self) -> Result<String> {
        self.execute(&["test-network"]).await
    }

    /// Test a single policy
    pub async fn test_policy(&self, name: &str) -> Result<String> {
        self.execute(&["test-policy", name]).await
    }

    /// Re-test a policy group
    pub async fn test_group(&self, name: &str) -> Result<String> {
        self.execute(&["test-group", name]).await
    }

    /// Flush DNS cache
    pub async fn flush_dns(&self) -> Result<()> {
        self.execute(&["flush", "dns"]).await?;
        Ok(())
    }

    /// Kill a connection
    pub async fn kill_connection(&self, id: u64) -> Result<()> {
        self.execute(&["kill", &id.to_string()]).await?;
        Ok(())
    }

    /// Stop Surge
    pub async fn stop_surge(&self) -> Result<()> {
        self.execute(&["stop"]).await?;
        Ok(())
    }

    /// Set log level
    pub async fn set_log_level(&self, level: &str) -> Result<()> {
        self.execute(&["set-log-level", level]).await?;
        Ok(())
    }

    /// Run diagnostics
    pub async fn run_diagnostics(&self) -> Result<String> {
        self.execute(&["diagnostics"]).await
    }

    /// Test all policies and return latency data
    ///
    /// Returns: Vec<(policy_name, RTT_latency_ms, success)>
    pub async fn test_all_policies(&self) -> Result<Vec<(String, Option<u32>, bool)>> {
        let output = self.execute(&["test-all-policies"]).await?;

        let mut results = Vec::new();
        for line in output.lines() {
            if let Some(result) = Self::parse_test_line(line) {
                results.push(result);
            }
        }

        tracing::info!("✓ CLI test completed: {} policies", results.len());
        Ok(results)
    }

    /// Parse a single test result line
    ///
    /// Format:
    /// - success: "ProxyName: RTT 123 ms, Total 456 ms"
    /// - failure: "ProxyName: Failed"
    fn parse_test_line(line: &str) -> Option<(String, Option<u32>, bool)> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        if let Some(colon_pos) = line.find(':') {
            let name = line[..colon_pos].trim().to_string();
            let rest = line[colon_pos + 1..].trim();

            if rest == "Failed" {
                return Some((name, None, false));
            }

            // Parse "RTT 123 ms, Total 456 ms"
            if rest.starts_with("RTT") {
                if let Some(rtt_start) = rest.find("RTT") {
                    if let Some(ms_pos) = rest[rtt_start..].find(" ms") {
                        let rtt_str = &rest[rtt_start + 4..rtt_start + ms_pos];
                        if let Ok(rtt) = rtt_str.trim().parse::<u32>() {
                            return Some((name, Some(rtt), true));
                        }
                    }
                }
            }
        }

        None
    }
}
