/// surge-cli 客户端
///
/// 通过 surge-cli 命令行工具与 Surge 交互
use crate::domain::errors::{Result, SurgeError};
use tokio::process::Command;

/// surge-cli 客户端
#[derive(Clone)]
pub struct SurgeCliClient {
    cli_path: String,
}

impl SurgeCliClient {
    /// 创建新的 CLI 客户端
    pub fn new(cli_path: Option<String>) -> Self {
        let cli_path = cli_path.unwrap_or_else(|| {
            "/Applications/Surge.app/Contents/Applications/surge-cli".to_string()
        });
        Self { cli_path }
    }

    /// 执行 surge-cli 命令
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

    /// 执行命令并返回 JSON
    async fn execute_json(&self, args: &[&str]) -> Result<serde_json::Value> {
        // 添加 --raw 参数以获取 JSON 输出
        let mut json_args = vec!["--raw"];
        json_args.extend_from_slice(args);

        let output = self.execute(&json_args).await?;
        serde_json::from_str(&output).map_err(|e| SurgeError::ParseError {
            source: "CLI JSON".to_string(),
            error: e.to_string(),
        })
    }

    /// 重新加载配置
    pub async fn reload_config(&self) -> Result<()> {
        self.execute(&["reload"]).await?;
        Ok(())
    }

    /// 切换配置文件
    pub async fn switch_profile(&self, name: &str) -> Result<()> {
        self.execute(&["switch-profile", name]).await?;
        Ok(())
    }

    /// 获取活跃连接
    pub async fn dump_active(&self) -> Result<serde_json::Value> {
        self.execute_json(&["dump", "active"]).await
    }

    /// 获取最近请求
    pub async fn dump_requests(&self) -> Result<serde_json::Value> {
        self.execute_json(&["dump", "request"]).await
    }

    /// 获取规则列表
    pub async fn dump_rules(&self) -> Result<serde_json::Value> {
        self.execute_json(&["dump", "rule"]).await
    }

    /// 获取策略列表
    pub async fn dump_policies(&self) -> Result<serde_json::Value> {
        self.execute_json(&["dump", "policy"]).await
    }

    /// 获取 DNS 缓存
    pub async fn dump_dns(&self) -> Result<serde_json::Value> {
        self.execute_json(&["dump", "dns"]).await
    }

    /// 获取配置文件内容
    pub async fn dump_profile(&self, effective: bool) -> Result<String> {
        let profile_type = if effective { "effective" } else { "original" };
        self.execute(&["dump", "profile", profile_type]).await
    }

    /// 测试网络延迟
    pub async fn test_network(&self) -> Result<String> {
        self.execute(&["test-network"]).await
    }

    /// 测试单个策略
    pub async fn test_policy(&self, name: &str) -> Result<String> {
        self.execute(&["test-policy", name]).await
    }

    /// 重新测试策略组
    pub async fn test_group(&self, name: &str) -> Result<String> {
        self.execute(&["test-group", name]).await
    }

    /// 清空 DNS 缓存
    pub async fn flush_dns(&self) -> Result<()> {
        self.execute(&["flush", "dns"]).await?;
        Ok(())
    }

    /// 终止连接
    pub async fn kill_connection(&self, id: u64) -> Result<()> {
        self.execute(&["kill", &id.to_string()]).await?;
        Ok(())
    }

    /// 停止 Surge
    pub async fn stop_surge(&self) -> Result<()> {
        self.execute(&["stop"]).await?;
        Ok(())
    }

    /// 设置日志级别
    pub async fn set_log_level(&self, level: &str) -> Result<()> {
        self.execute(&["set-log-level", level]).await?;
        Ok(())
    }

    /// 运行诊断
    pub async fn run_diagnostics(&self) -> Result<String> {
        self.execute(&["diagnostics"]).await
    }

    /// 测试所有策略的延迟
    ///
    /// 返回格式: Vec<(策略名, RTT延迟ms, 是否成功)>
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

    /// 解析单行测试结果
    ///
    /// 格式:
    /// - 成功: "ProxyName: RTT 123 ms, Total 456 ms"
    /// - 失败: "ProxyName: Failed"
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

            // 解析 "RTT 123 ms, Total 456 ms"
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
