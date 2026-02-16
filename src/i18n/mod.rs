mod en_us;
/// 多语言支持模块
///
/// 基于 trait 的编译时翻译选择，零运行时开销
mod zh_cn;

pub use en_us::EnUS;
pub use zh_cn::ZhCN;

/// 翻译接口
///
/// 所有文本通过此 trait 提供，支持编译时类型检查
pub trait Translate: Send + Sync {
    // ========== UI 状态栏 ==========
    fn ui_status_running(&self) -> &'static str;
    fn ui_status_stopped(&self) -> &'static str;
    fn ui_status_http_api(&self) -> &'static str;
    fn ui_status_cli_mode(&self) -> &'static str;

    // ========== 快捷键说明 ==========
    fn key_quit(&self) -> &'static str;
    fn key_refresh(&self) -> &'static str;
    fn key_view(&self) -> &'static str;
    fn key_mode(&self) -> &'static str;
    fn key_test(&self) -> &'static str;
    fn key_enter(&self) -> &'static str;
    fn key_esc(&self) -> &'static str;
    fn key_start(&self) -> &'static str;
    fn key_reload(&self) -> &'static str;

    // ========== 视图标题 ==========
    fn views_title(&self) -> &'static str;
    fn view_overview(&self) -> &'static str;
    fn view_policies(&self) -> &'static str;
    fn view_requests(&self) -> &'static str;
    fn view_connections(&self) -> &'static str;

    // ========== 通知消息 ==========
    fn notification_test_started(&self) -> &'static str;
    fn notification_test_completed(&self, alive: usize, total: usize) -> String;
    fn notification_test_failed(&self, error: &str) -> String;

    // ========== Alert 消息 ==========
    fn alert_surge_not_running(&self) -> &'static str;
    fn alert_http_api_disabled(&self) -> &'static str;

    // ========== 策略组 ==========
    fn policy_group_title(&self) -> &'static str;
    fn policy_group_enter_hint(&self) -> &'static str;
    fn policy_policies_title(&self, group_name: &str) -> String;
    fn policy_select_title(&self, group_name: &str) -> String;
    fn policy_testing(&self) -> &'static str;
    fn policy_testing_hint(&self) -> &'static str;
    fn policy_available(&self) -> &'static str;
    fn policy_unavailable(&self) -> &'static str;
    fn policy_no_groups(&self) -> &'static str;
    fn policy_no_policies(&self) -> &'static str;
    fn policy_no_selection(&self) -> &'static str;

    // ========== DevTools ==========
    fn devtools_title(&self) -> &'static str;
    fn devtools_no_logs(&self) -> &'static str;

    // ========== 通知历史 ==========
    fn notification_history_title(&self) -> &'static str;
    fn notification_history_empty(&self) -> &'static str;

    // ========== Overview ==========
    fn overview_surge_status(&self) -> &'static str;
    fn overview_api_status(&self) -> &'static str;
    fn overview_outbound_mode(&self) -> &'static str;
    fn overview_stats(&self) -> &'static str;

    // ========== OutboundMode ==========
    fn outbound_mode_direct(&self) -> &'static str;
    fn outbound_mode_proxy(&self) -> &'static str;
    fn outbound_mode_rule(&self) -> &'static str;

    // ========== AlertAction ==========
    fn alert_action_start_surge(&self) -> &'static str;
    fn alert_action_reload_config(&self) -> &'static str;

    // ========== Statistics Labels ==========
    fn stats_policies(&self) -> &'static str;
    fn stats_policy_groups(&self) -> &'static str;
    fn stats_active_connections(&self) -> &'static str;
    fn stats_recent_requests(&self) -> &'static str;

    // ========== Requests ==========
    fn request_list_title(&self) -> &'static str;
    fn request_detail_title(&self) -> &'static str;
    fn request_no_requests(&self) -> &'static str;
    fn request_no_selection(&self) -> &'static str;
    fn request_status_completed(&self) -> &'static str;
    fn request_status_failed(&self) -> &'static str;
    fn request_status_in_progress(&self) -> &'static str;
    fn request_label_request(&self) -> &'static str;
    fn request_label_host(&self) -> &'static str;
    fn request_label_rule(&self) -> &'static str;
    fn request_label_policy(&self) -> &'static str;
    fn request_label_traffic(&self) -> &'static str;
    fn request_label_upload(&self) -> &'static str;
    fn request_label_download(&self) -> &'static str;
    fn request_label_process(&self) -> &'static str;
    fn request_label_time(&self) -> &'static str;
    fn request_time_seconds_ago(&self, secs: u64) -> String;
    fn request_time_minutes_ago(&self, mins: u64) -> String;
    fn request_time_hours_ago(&self, hours: u64) -> String;
    fn request_label_http_body(&self) -> &'static str;
    fn request_has_request_body(&self) -> &'static str;
    fn request_has_response_body(&self) -> &'static str;
    fn request_label_notes(&self) -> &'static str;
    fn request_notes_more(&self, count: usize) -> String;

    // ========== Grouping ==========
    fn key_group(&self) -> &'static str;
    fn request_app_list_title(&self) -> &'static str;
    fn request_all_mode(&self) -> &'static str;
    fn request_grouped_mode(&self) -> &'static str;
    fn request_no_app_selected(&self) -> &'static str;

    // ========== Help ==========
    fn key_help(&self) -> &'static str;
    fn help_title(&self) -> &'static str;
    fn help_global_section(&self) -> &'static str;
    fn help_view_section(&self) -> &'static str;
    fn help_navigation_section(&self) -> &'static str;

    // ========== 通用操作词 ==========
    fn action_select(&self) -> &'static str;
    fn action_enter(&self) -> &'static str;
    fn action_confirm(&self) -> &'static str;
    fn action_back(&self) -> &'static str;
    fn action_test(&self) -> &'static str;
    fn action_search(&self) -> &'static str;
    fn action_toggle(&self) -> &'static str;
    fn action_group(&self) -> &'static str;
    fn action_mode(&self) -> &'static str;
    fn action_kill(&self) -> &'static str;

    // ========== 连接终止确认 ==========
    fn confirm_kill_title(&self) -> &'static str;
    fn confirm_kill_message(&self, url: &str) -> String;
    fn confirm_kill_hint(&self) -> &'static str;
    fn notification_connection_killed(&self) -> &'static str;
    fn notification_kill_failed(&self, error: &str) -> String;

    // ========== 功能开关 ==========
    fn feature_mitm(&self) -> &'static str;
    fn feature_capture(&self) -> &'static str;
    fn status_enabled(&self) -> &'static str;
    fn status_disabled(&self) -> &'static str;
    fn notification_mitm_enabled(&self) -> &'static str;
    fn notification_mitm_disabled(&self) -> &'static str;
    fn notification_capture_enabled(&self) -> &'static str;
    fn notification_capture_disabled(&self) -> &'static str;
    fn notification_feature_toggle_failed(&self, error: &str) -> String;

    // ========== DNS ==========
    fn view_dns(&self) -> &'static str;
    fn dns_list_title(&self) -> &'static str;
    fn dns_detail_title(&self) -> &'static str;
    fn dns_no_records(&self) -> &'static str;
    fn dns_label_domain(&self) -> &'static str;
    fn dns_label_value(&self) -> &'static str;
    fn dns_label_ttl(&self) -> &'static str;
    fn action_flush(&self) -> &'static str;
    fn notification_dns_flushed(&self) -> &'static str;
    fn notification_dns_flush_failed(&self, error: &str) -> String;
}

// 编译时语言选择
#[cfg(feature = "lang-zh-cn")]
pub type CurrentLang = ZhCN;

#[cfg(feature = "lang-en-us")]
pub type CurrentLang = EnUS;

// 如果没有指定语言特性，默认使用中文
#[cfg(not(any(feature = "lang-zh-cn", feature = "lang-en-us")))]
pub type CurrentLang = ZhCN;

/// 获取当前语言实例
#[cfg(feature = "lang-zh-cn")]
pub fn current() -> &'static dyn Translate {
    static INSTANCE: ZhCN = ZhCN;
    &INSTANCE
}

#[cfg(feature = "lang-en-us")]
pub fn current() -> &'static dyn Translate {
    static INSTANCE: EnUS = EnUS;
    &INSTANCE
}

#[cfg(not(any(feature = "lang-zh-cn", feature = "lang-en-us")))]
pub fn current() -> &'static dyn Translate {
    static INSTANCE: ZhCN = ZhCN;
    &INSTANCE
}
