/// HTTP API client
///
/// Wraps all Surge HTTP API calls
use crate::domain::{
    errors::{Result, SurgeError},
    models::*,
};
use reqwest::Client;
use serde_json::Value;

/// HTTP API client
#[derive(Clone)]
pub struct SurgeHttpClient {
    base_url: String,
    api_key: String,
    client: Client,
}

impl SurgeHttpClient {
    /// Create new HTTP client
    pub fn new(host: String, port: u16, api_key: String) -> Self {
        let base_url = format!("http://{}:{}", host, port);
        Self {
            base_url,
            api_key,
            client: Client::new(),
        }
    }

    /// Test if HTTP API is available
    pub async fn is_available(&self) -> bool {
        self.get_outbound_mode().await.is_ok()
    }

    /// Build complete URL
    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Send GET request
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self
            .client
            .get(&self.build_url(path))
            .header("X-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| SurgeError::NetworkError {
                message: format!("HTTP GET failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(SurgeError::HttpApiUnavailable {
                reason: format!("HTTP {} returned status {}", path, response.status()),
            });
        }

        // Parse JSON directly, only read text on failure
        response.json().await.map_err(|e| {
            tracing::error!("Failed to parse {} response: {}", path, e);
            SurgeError::ParseError {
                source: format!("HTTP Response {}", path),
                error: e.to_string(),
            }
        })
    }

    /// Send POST request
    #[allow(dead_code)]
    async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: Option<Value>,
    ) -> Result<T> {
        let mut request = self
            .client
            .post(&self.build_url(path))
            .header("X-Key", &self.api_key);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(|e| SurgeError::NetworkError {
            message: format!("HTTP POST failed: {}", e),
        })?;

        if !response.status().is_success() {
            return Err(SurgeError::HttpApiUnavailable {
                reason: format!("HTTP {} returned status {}", path, response.status()),
            });
        }

        response.json().await.map_err(|e| SurgeError::ParseError {
            source: "HTTP Response".to_string(),
            error: e.to_string(),
        })
    }

    /// Send POST request (no response body)
    async fn post_empty(&self, path: &str, body: Option<Value>) -> Result<()> {
        let mut request = self
            .client
            .post(&self.build_url(path))
            .header("X-Key", &self.api_key);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(|e| SurgeError::NetworkError {
            message: format!("HTTP POST failed: {}", e),
        })?;

        if !response.status().is_success() {
            return Err(SurgeError::HttpApiUnavailable {
                reason: format!("HTTP {} returned status {}", path, response.status()),
            });
        }

        Ok(())
    }

    // ===== Outbound mode related =====

    /// Get outbound mode
    pub async fn get_outbound_mode(&self) -> Result<OutboundMode> {
        let response: OutboundModeResponse = self.get("/v1/outbound").await?;
        Ok(response.mode)
    }

    /// Set outbound mode
    pub async fn set_outbound_mode(&self, mode: OutboundMode) -> Result<()> {
        let body = serde_json::json!({ "mode": mode });
        self.post_empty("/v1/outbound", Some(body)).await
    }

    // ===== 策略相关 =====

    /// 获取所有策略列表
    pub async fn get_policies(&self) -> Result<Vec<String>> {
        let response: PoliciesResponse = self.get("/v1/policies").await?;
        // 合并 proxies 和 policy-groups
        let mut all_policies = response.proxies;
        all_policies.extend(response.policy_groups);
        Ok(all_policies)
    }

    /// 获取策略详情
    pub async fn get_policy_detail(&self, name: &str) -> Result<PolicyDetail> {
        // URL 编码策略名称
        let encoded_name = urlencoding::encode(name);
        let path = format!("/v1/policies/detail?policy_name={}", encoded_name);
        self.get(&path).await
    }

    /// 测试策略延迟
    pub async fn test_policy(&self, name: &str) -> Result<()> {
        let body = serde_json::json!({
            "policy_names": [name],
            "url": "http://www.gstatic.com/generate_204"
        });
        self.post_empty("/v1/policies/test", Some(body)).await
    }

    // ===== 策略组相关 =====

    /// 获取所有策略组
    pub async fn get_policy_groups(&self) -> Result<Vec<PolicyGroup>> {
        let response: PolicyGroupsResponse = self.get("/v1/policy_groups").await?;

        // 收集并排序策略组名称，保持稳定顺序
        let mut group_names: Vec<String> = response.keys().cloned().collect();
        group_names.sort();

        // 为每个策略组获取当前选中的策略
        let mut selected_map = std::collections::HashMap::new();
        for group_name in &group_names {
            if let Ok(Some(policy)) = self.get_policy_group_selected(group_name).await {
                selected_map.insert(group_name.clone(), policy);
            }
        }

        // 按排序后的顺序构建 Vec<PolicyGroup>
        let groups = group_names
            .into_iter()
            .filter_map(|name| {
                response.get(&name).map(|policies| {
                    let selected = selected_map.get(&name).cloned();

                    PolicyGroup {
                        name,
                        policies: policies.clone(),
                        selected,
                        available_policies: None, // 初始为 None，需要测试后才有数据
                    }
                })
            })
            .collect();

        Ok(groups)
    }

    /// 获取策略组的当前选中策略
    pub async fn get_policy_group_selected(&self, group_name: &str) -> Result<Option<String>> {
        use crate::domain::models::PolicyGroupSelectResponse;

        let url = format!(
            "/v1/policy_groups/select?group_name={}",
            urlencoding::encode(group_name)
        );

        match self.get::<PolicyGroupSelectResponse>(&url).await {
            Ok(response) => Ok(Some(response.policy)),
            Err(_) => {
                // 如果获取失败（可能是非 select 类型的策略组），返回 None
                Ok(None)
            }
        }
    }

    /// 选择策略组中的策略
    pub async fn select_policy_group(&self, group_name: &str, policy: &str) -> Result<()> {
        let body = serde_json::json!({
            "group_name": group_name,
            "policy": policy
        });
        self.post_empty("/v1/policy_groups/select", Some(body))
            .await
    }

    /// 测试策略组，返回可用策略列表
    pub async fn test_policy_group(&self, group_name: &str) -> Result<Vec<String>> {
        let body = serde_json::json!({ "group_name": group_name });
        let response: serde_json::Value = self.post("/v1/policy_groups/test", Some(body)).await?;
        tracing::debug!("Policy group {} test response: {:?}", group_name, response);

        // 解析 {"available": ["proxy1", "proxy2"]} 格式
        let available = response
            .get("available")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        tracing::info!(
            "Policy group {} test completed, {} available policies",
            group_name,
            available.len()
        );
        Ok(available)
    }

    /// 获取策略组测试结果
    pub async fn get_policy_group_test_results(&self) -> Result<serde_json::Value> {
        self.get("/v1/policy_groups/test_results").await
    }

    // ===== 请求相关 =====

    /// 获取最近请求
    pub async fn get_recent_requests(&self) -> Result<Vec<Request>> {
        let response: RequestsResponse = self.get("/v1/requests/recent").await?;
        Ok(response.requests)
    }

    /// 获取活跃连接
    pub async fn get_active_connections(&self) -> Result<Vec<Request>> {
        let response: ActiveConnectionsResponse = self.get("/v1/requests/active").await?;
        Ok(response.requests)
    }

    /// 终止连接
    pub async fn kill_connection(&self, id: u64) -> Result<()> {
        let body = serde_json::json!({ "id": id });
        self.post_empty("/v1/requests/kill", Some(body)).await
    }

    // ===== 配置相关 =====

    /// 重新加载配置
    pub async fn reload_config(&self) -> Result<()> {
        self.post_empty("/v1/profiles/reload", None).await
    }

    /// 获取当前配置
    pub async fn get_current_profile(&self, show_sensitive: bool) -> Result<ProfileInfo> {
        let sensitive = if show_sensitive { "1" } else { "0" };
        let path = format!("/v1/profiles/current?sensitive={}", sensitive);
        self.get(&path).await
    }

    // ===== DNS 相关 =====

    /// 清空 DNS 缓存
    pub async fn flush_dns(&self) -> Result<()> {
        self.post_empty("/v1/dns/flush", None).await
    }

    /// 获取 DNS 缓存
    pub async fn get_dns_cache(&self) -> Result<Vec<DnsRecord>> {
        // 先获取原始响应文本以便调试
        let response = self
            .client
            .get(&self.build_url("/v1/dns"))
            .header("X-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| SurgeError::NetworkError {
                message: format!("HTTP GET failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(SurgeError::HttpApiUnavailable {
                reason: format!("HTTP /v1/dns returned status {}", response.status()),
            });
        }

        let text = response
            .text()
            .await
            .map_err(|e| SurgeError::NetworkError {
                message: format!("Failed to read response body: {}", e),
            })?;

        tracing::debug!("DNS API raw response: {}", text);

        // 尝试解析为 DnsResponse
        serde_json::from_str::<DnsResponse>(&text)
            .map(|r| r.records)
            .map_err(|e| {
                tracing::error!(
                    "Failed to parse DNS response as DnsResponse: {}. Raw: {}",
                    e,
                    text
                );
                SurgeError::ParseError {
                    source: "DNS Response".to_string(),
                    error: e.to_string(),
                }
            })
    }

    // ===== 功能开关 =====

    /// 获取功能状态
    async fn get_feature_status(&self, feature: &str) -> Result<bool> {
        let path = format!("/v1/features/{}", feature);
        let response: FeatureStatus = self.get(&path).await?;
        Ok(response.enabled)
    }

    /// 设置功能状态
    async fn set_feature_status(&self, feature: &str, enabled: bool) -> Result<()> {
        let path = format!("/v1/features/{}", feature);
        let body = serde_json::json!({ "enabled": enabled });
        self.post_empty(&path, Some(body)).await
    }

    /// 获取 MITM 状态
    pub async fn get_mitm_status(&self) -> Result<bool> {
        self.get_feature_status("mitm").await
    }

    /// 设置 MITM 状态
    pub async fn set_mitm_status(&self, enabled: bool) -> Result<()> {
        self.set_feature_status("mitm", enabled).await
    }

    /// 获取流量捕获状态
    pub async fn get_capture_status(&self) -> Result<bool> {
        self.get_feature_status("capture").await
    }

    /// 设置流量捕获状态
    pub async fn set_capture_status(&self, enabled: bool) -> Result<()> {
        self.set_feature_status("capture", enabled).await
    }
}
