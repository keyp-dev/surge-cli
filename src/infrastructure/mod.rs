/// Infrastructure 层 - 基础设施实现
///
/// 依赖外部服务: HTTP API, CLI, System
pub mod cli_client;
pub mod http_client;
pub mod system_client;

// 重新导出客户端
pub use cli_client::SurgeCliClient;
pub use http_client::SurgeHttpClient;
pub use system_client::SurgeSystemClient;
