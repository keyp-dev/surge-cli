/// System command client
///
/// System-level operations: checking Surge process status, starting Surge, etc.
use crate::domain::errors::{Result, SurgeError};
use tokio::process::Command;

/// System command client
#[derive(Clone, Copy)]
pub struct SurgeSystemClient;

impl SurgeSystemClient {
    /// Create new system client
    pub fn new() -> Self {
        Self
    }

    /// Check if Surge is running
    pub async fn is_surge_running(&self) -> bool {
        // Use pgrep to check for the Surge process
        let output = Command::new("pgrep").args(["-x", "Surge"]).output().await;

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Start Surge
    pub async fn start_surge(&self) -> Result<()> {
        let output = Command::new("open")
            .args(["-a", "Surge"])
            .output()
            .await
            .map_err(|e| SurgeError::Unknown {
                message: format!("Failed to start Surge: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SurgeError::Unknown {
                message: format!("Failed to start Surge: {}", stderr),
            });
        }

        // Wait for Surge to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(())
    }

    /// Stop Surge
    pub async fn stop_surge(&self) -> Result<()> {
        let output = Command::new("killall")
            .arg("Surge")
            .output()
            .await
            .map_err(|e| SurgeError::Unknown {
                message: format!("Failed to stop Surge: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // If process doesn't exist, killall returns error, but it's not a real error
            if !stderr.contains("No matching processes") {
                return Err(SurgeError::Unknown {
                    message: format!("Failed to stop Surge: {}", stderr),
                });
            }
        }

        Ok(())
    }

    /// Get Surge process PID
    pub async fn get_surge_pid(&self) -> Option<u32> {
        let output = Command::new("pgrep")
            .args(["-x", "Surge"])
            .output()
            .await
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.trim().parse().ok()
    }

    /// Check if surge-cli exists at the given path
    pub async fn cli_exists(&self, cli_path: &str) -> bool {
        tokio::fs::metadata(cli_path).await.is_ok()
    }

    /// Check if Surge.app is installed
    pub async fn surge_app_exists(&self) -> bool {
        tokio::fs::metadata("/Applications/Surge.app").await.is_ok()
    }
}

impl Default for SurgeSystemClient {
    fn default() -> Self {
        Self::new()
    }
}
