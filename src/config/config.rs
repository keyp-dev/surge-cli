/// Configuration management
///
/// Load configuration from config file or environment variables
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub surge: SurgeConfig,
    pub ui: UiConfig,
}

/// Surge-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurgeConfig {
    /// HTTP API host
    #[serde(default = "default_http_api_host")]
    pub http_api_host: String,

    /// HTTP API port
    #[serde(default = "default_http_api_port")]
    pub http_api_port: u16,

    /// HTTP API key
    #[serde(default)]
    pub http_api_key: String,

    /// surge-cli path
    #[serde(default = "default_cli_path")]
    pub cli_path: Option<String>,
}

/// UI-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Refresh interval (seconds)
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,

    /// Maximum request history count
    #[serde(default = "default_max_requests")]
    pub max_requests: usize,
}

// === Default values ===

fn default_http_api_host() -> String {
    "127.0.0.1".to_string()
}

fn default_http_api_port() -> u16 {
    6171
}

fn default_cli_path() -> Option<String> {
    Some("/Applications/Surge.app/Contents/Applications/surge-cli".to_string())
}

fn default_refresh_interval() -> u64 {
    1 // 1 second
}

fn default_max_requests() -> usize {
    100
}

impl Default for Config {
    fn default() -> Self {
        Self {
            surge: SurgeConfig {
                http_api_host: default_http_api_host(),
                http_api_port: default_http_api_port(),
                http_api_key: String::new(), // Must be provided by user
                cli_path: default_cli_path(),
            },
            ui: UiConfig {
                refresh_interval: default_refresh_interval(),
                max_requests: default_max_requests(),
            },
        }
    }
}

impl Config {
    /// Load config from file
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load config (file + env var overlay)
    pub fn load(config_path: Option<PathBuf>) -> anyhow::Result<Self> {
        let file_config = if let Some(path) = config_path {
            if path.exists() {
                Self::from_file(&path).ok()
            } else {
                None
            }
        } else {
            // Build default paths with proper ~ expansion
            let home = std::env::var("HOME").unwrap_or_default();
            let default_paths = vec![
                PathBuf::from("surge-tui.toml"),
                PathBuf::from(format!("{}/.config/surge-tui/surge-tui.toml", home)),
                PathBuf::from(format!("{}/.config/surge-tui/config.toml", home)),
            ];

            default_paths
                .into_iter()
                .filter(|p| p.exists())
                .find_map(|p| Self::from_file(&p).ok())
        };

        // Start from file config or defaults
        let mut config = file_config.unwrap_or_default();

        // Always overlay env vars (env takes precedence over file for key/host/port)
        if let Ok(host) = std::env::var("SURGE_HTTP_API_HOST") {
            config.surge.http_api_host = host;
        }
        if let Ok(port) = std::env::var("SURGE_HTTP_API_PORT") {
            if let Ok(port) = port.parse() {
                config.surge.http_api_port = port;
            }
        }
        if let Ok(key) = std::env::var("SURGE_HTTP_API_KEY") {
            config.surge.http_api_key = key;
        }
        if let Ok(path) = std::env::var("SURGE_CLI_PATH") {
            config.surge.cli_path = Some(path);
        }

        Ok(config)
    }

    /// Generate example config file
    pub fn example() -> String {
        r#"[surge]
# HTTP API 配置
http_api_host = "127.0.0.1"
http_api_port = 6171
http_api_key = "your-secret-key"  # 必填

# surge-cli 路径（可选，默认自动查找）
# cli_path = "/Applications/Surge.app/Contents/Applications/surge-cli"

[ui]
# UI 刷新间隔（秒）
refresh_interval = 1

# 最大请求历史条数
max_requests = 100
"#
        .to_string()
    }
}
