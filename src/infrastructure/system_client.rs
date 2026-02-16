/// 系统命令客户端
///
/// 检查 Surge 进程状态、启动 Surge 等系统级操作
use crate::domain::errors::{Result, SurgeError};
use tokio::process::Command;

/// 系统命令客户端
#[derive(Clone, Copy)]
pub struct SurgeSystemClient;

impl SurgeSystemClient {
    /// 创建新的系统客户端
    pub fn new() -> Self {
        Self
    }

    /// 检查 Surge 是否运行
    pub async fn is_surge_running(&self) -> bool {
        // 使用 pgrep 检查 Surge 进程
        let output = Command::new("pgrep").args(["-x", "Surge"]).output().await;

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// 启动 Surge
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

        // 等待 Surge 启动
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(())
    }

    /// 停止 Surge
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

    /// 获取 Surge 进程 PID
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

    /// 检查 surge-cli 是否存在
    pub async fn cli_exists(&self, cli_path: &str) -> bool {
        tokio::fs::metadata(cli_path).await.is_ok()
    }

    /// 检查 Surge.app 是否安装
    pub async fn surge_app_exists(&self) -> bool {
        tokio::fs::metadata("/Applications/Surge.app").await.is_ok()
    }
}

impl Default for SurgeSystemClient {
    fn default() -> Self {
        Self::new()
    }
}
