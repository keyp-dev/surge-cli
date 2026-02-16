/// Domain data models
///
/// Corresponds to Surge HTTP API response structures
use serde::{Deserialize, Serialize};

/// Outbound mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutboundMode {
    /// Direct connection
    Direct,
    /// Proxy all
    Proxy,
    /// Rule mode
    Rule,
}

impl OutboundMode {
    // Removed as_str() - translation happens in UI layer
}

/// Outbound mode response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundModeResponse {
    pub mode: OutboundMode,
}

/// Feature toggle status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStatus {
    pub enabled: bool,
}

/// Policy type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PolicyType {
    #[serde(rename = "ss")]
    Shadowsocks,
    #[serde(rename = "vmess")]
    Vmess,
    #[serde(rename = "trojan")]
    Trojan,
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "socks5")]
    Socks5,
    #[serde(rename = "direct")]
    Direct,
    #[serde(rename = "reject")]
    Reject,
    #[serde(rename = "select")]
    Select,
    #[serde(rename = "url-test")]
    UrlTest,
    #[serde(rename = "fallback")]
    Fallback,
    #[serde(rename = "load-balance")]
    LoadBalance,
    #[serde(other)]
    Unknown,
}

impl PolicyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Shadowsocks => "Shadowsocks",
            Self::Vmess => "VMess",
            Self::Trojan => "Trojan",
            Self::Http => "HTTP",
            Self::Socks5 => "SOCKS5",
            Self::Direct => "Direct",
            Self::Reject => "Reject",
            Self::Select => "Select",
            Self::UrlTest => "URL-Test",
            Self::Fallback => "Fallback",
            Self::LoadBalance => "Load-Balance",
            Self::Unknown => "Unknown",
        }
    }
}

/// Policy detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDetail {
    pub name: String,
    #[serde(rename = "type")]
    pub policy_type: PolicyType,
    #[serde(default)]
    pub alive: bool,
    #[serde(default)]
    pub latency: Option<u32>, // ms
    #[serde(default)]
    pub last_test_at: Option<String>,
}

/// Policy list response (real API format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoliciesResponse {
    pub proxies: Vec<String>,
    #[serde(rename = "policy-groups")]
    pub policy_groups: Vec<String>,
}

/// Policy item in policy group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyItem {
    #[serde(rename = "isGroup")]
    pub is_group: bool,
    pub name: String,
    #[serde(rename = "typeDescription")]
    pub type_description: String,
    #[serde(rename = "lineHash")]
    pub line_hash: String,
    pub enabled: bool,
}

/// Policy group (internal representation)
#[derive(Debug, Clone)]
pub struct PolicyGroup {
    pub name: String,
    pub policies: Vec<PolicyItem>,
    pub selected: Option<String>, // Currently selected policy (from API)
    pub available_policies: Option<Vec<String>>, // Available policies after test (obtained after pressing T)
}

/// Policy group list response (real API format: HashMap<group_name, policy_array>)
pub type PolicyGroupsResponse = std::collections::HashMap<String, Vec<PolicyItem>>;

/// Policy group selected policy response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyGroupSelectResponse {
    pub policy: String,
}

/// Request detail (real API format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    #[serde(default, rename = "processPath")]
    pub process_path: Option<String>,
    #[serde(default)]
    pub rule: Option<String>,
    #[serde(default, rename = "policyName")]
    pub policy_name: Option<String>,
    #[serde(default, rename = "remoteHost")]
    pub remote_host: Option<String>,
    #[serde(default, rename = "URL")]
    pub url: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, rename = "startDate")]
    pub start_date: Option<f64>,
    #[serde(default, rename = "inBytes")]
    pub in_bytes: u64, // Download bytes
    #[serde(default, rename = "outBytes")]
    pub out_bytes: u64, // Upload bytes
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub failed: bool,
    #[serde(default)]
    pub notes: Vec<String>, // Connection log
    #[serde(default, rename = "streamHasRequestBody")]
    pub stream_has_request_body: bool, // Has request body
    #[serde(default, rename = "streamHasResponseBody")]
    pub stream_has_response_body: bool, // Has response body
}

/// Request list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestsResponse {
    pub requests: Vec<Request>,
}

/// Active connections response (same format as RequestsResponse)
pub type ActiveConnectionsResponse = RequestsResponse;

/// DNS cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub domain: String,
    #[serde(rename = "data")]
    pub ip: Vec<String>,
    #[serde(rename = "expiresTime", default)]
    pub ttl: Option<f64>, // Unix timestamp with milliseconds
    #[serde(default)]
    pub server: Option<String>,
    #[serde(default)]
    pub logs: Vec<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default, rename = "timeCost")]
    pub time_cost: Option<f64>,
}

/// DNS response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsResponse {
    #[serde(rename = "dnsCache")]
    pub records: Vec<DnsRecord>,
}

/// Profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub name: String,
    #[serde(default)]
    pub content: Option<String>,
}

/// Traffic statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    #[serde(default)]
    pub upload: u64, // bytes
    #[serde(default)]
    pub download: u64, // bytes
    #[serde(default)]
    pub upload_speed: u64, // bytes/s
    #[serde(default)]
    pub download_speed: u64, // bytes/s
}
