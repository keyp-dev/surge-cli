/// 简体中文翻译
use super::Translate;

pub struct ZhCN;

impl Translate for ZhCN {
    // ========== UI 状态栏 ==========
    fn ui_status_running(&self) -> &'static str {
        "Surge 运行中"
    }

    fn ui_status_stopped(&self) -> &'static str {
        "Surge 未运行"
    }

    fn ui_status_http_api(&self) -> &'static str {
        "(HTTP API)"
    }

    fn ui_status_cli_mode(&self) -> &'static str {
        "(CLI 模式)"
    }

    // ========== 快捷键说明 ==========
    fn key_quit(&self) -> &'static str {
        "[q]uit"
    }

    fn key_refresh(&self) -> &'static str {
        "[r]efresh"
    }

    fn key_view(&self) -> &'static str {
        "[1-4]view"
    }

    fn key_mode(&self) -> &'static str {
        "[m]ode"
    }

    fn key_test(&self) -> &'static str {
        "[t]est"
    }

    fn key_enter(&self) -> &'static str {
        "[Enter]进入"
    }

    fn key_esc(&self) -> &'static str {
        "[ESC]返回"
    }

    fn key_start(&self) -> &'static str {
        "[s]tart"
    }

    fn key_reload(&self) -> &'static str {
        "[r]eload"
    }

    // ========== 视图标题 ==========
    fn views_title(&self) -> &'static str {
        "视图"
    }

    fn view_overview(&self) -> &'static str {
        "总览"
    }

    fn view_policies(&self) -> &'static str {
        "策略"
    }

    fn view_requests(&self) -> &'static str {
        "请求历史"
    }

    fn view_connections(&self) -> &'static str {
        "活跃连接"
    }

    // ========== 通知消息 ==========
    fn notification_test_started(&self) -> &'static str {
        "策略延迟测试已启动..."
    }

    fn notification_test_completed(&self, alive: usize, total: usize) -> String {
        format!("测试完成: {}/{} 可用", alive, total)
    }

    fn notification_test_failed(&self, error: &str) -> String {
        format!("测试失败: {}", error)
    }

    // ========== Alert 消息 ==========
    fn alert_surge_not_running(&self) -> &'static str {
        "Surge 未运行 - 按 S 启动"
    }

    fn alert_http_api_disabled(&self) -> &'static str {
        "HTTP API 不可用 - 按 R 重载配置"
    }

    // ========== 策略组 ==========
    fn policy_group_title(&self) -> &'static str {
        "策略组"
    }

    fn policy_group_enter_hint(&self) -> &'static str {
        "策略组 [Enter进入]"
    }

    fn policy_policies_title(&self, group_name: &str) -> String {
        format!("策略组: {}", group_name)
    }

    fn policy_select_title(&self, group_name: &str) -> String {
        format!("策略组: {} [选择策略]", group_name)
    }

    fn policy_testing(&self) -> &'static str {
        "[测试中...]"
    }

    fn policy_testing_hint(&self) -> &'static str {
        " [测试中... 完成后按 R 刷新]"
    }

    fn policy_available(&self) -> &'static str {
        "[可用]"
    }

    fn policy_unavailable(&self) -> &'static str {
        "[不可用]"
    }

    fn policy_no_groups(&self) -> &'static str {
        "无策略组数据"
    }

    fn policy_no_policies(&self) -> &'static str {
        "该策略组无策略"
    }

    fn policy_no_selection(&self) -> &'static str {
        "无策略组选中"
    }

    // ========== DevTools ==========
    fn devtools_title(&self) -> &'static str {
        " DevTools [ESC 关闭] "
    }

    fn devtools_no_logs(&self) -> &'static str {
        "无日志记录"
    }

    // ========== 通知历史 ==========
    fn notification_history_title(&self) -> &'static str {
        " 通知历史 [ESC 关闭] "
    }

    fn notification_history_empty(&self) -> &'static str {
        "无通知历史"
    }

    // ========== Overview ==========
    fn overview_surge_status(&self) -> &'static str {
        "Surge 状态"
    }

    fn overview_api_status(&self) -> &'static str {
        "API 状态"
    }

    fn overview_outbound_mode(&self) -> &'static str {
        "出站模式"
    }

    fn overview_stats(&self) -> &'static str {
        "统计信息"
    }

    // ========== OutboundMode ==========
    fn outbound_mode_direct(&self) -> &'static str {
        "直连模式"
    }

    fn outbound_mode_proxy(&self) -> &'static str {
        "全局代理"
    }

    fn outbound_mode_rule(&self) -> &'static str {
        "规则模式"
    }

    // ========== AlertAction ==========
    fn alert_action_start_surge(&self) -> &'static str {
        "按 S 启动 Surge"
    }

    fn alert_action_reload_config(&self) -> &'static str {
        "按 R 重载配置"
    }

    // ========== Statistics Labels ==========
    fn stats_policies(&self) -> &'static str {
        "策略数量"
    }

    fn stats_policy_groups(&self) -> &'static str {
        "策略组数量"
    }

    fn stats_active_connections(&self) -> &'static str {
        "活跃连接"
    }

    fn stats_recent_requests(&self) -> &'static str {
        "最近请求"
    }

    // ========== Requests ==========
    fn request_list_title(&self) -> &'static str {
        "请求列表"
    }

    fn request_detail_title(&self) -> &'static str {
        "请求详情"
    }

    fn request_no_requests(&self) -> &'static str {
        "暂无请求"
    }

    fn request_no_selection(&self) -> &'static str {
        "无选中请求"
    }

    fn request_status_completed(&self) -> &'static str {
        "✓ 已完成"
    }

    fn request_status_failed(&self) -> &'static str {
        "✗ 失败"
    }

    fn request_status_in_progress(&self) -> &'static str {
        "○ 进行中"
    }

    fn request_label_request(&self) -> &'static str {
        "请求"
    }

    fn request_label_host(&self) -> &'static str {
        "主机"
    }

    fn request_label_rule(&self) -> &'static str {
        "规则"
    }

    fn request_label_policy(&self) -> &'static str {
        "策略"
    }

    fn request_label_traffic(&self) -> &'static str {
        "流量统计"
    }

    fn request_label_upload(&self) -> &'static str {
        "上传"
    }

    fn request_label_download(&self) -> &'static str {
        "下载"
    }

    fn request_label_process(&self) -> &'static str {
        "进程"
    }

    fn request_label_time(&self) -> &'static str {
        "时间"
    }

    fn request_time_seconds_ago(&self, secs: u64) -> String {
        format!("{}秒前", secs)
    }

    fn request_time_minutes_ago(&self, mins: u64) -> String {
        format!("{}分钟前", mins)
    }

    fn request_time_hours_ago(&self, hours: u64) -> String {
        format!("{}小时前", hours)
    }

    fn request_label_http_body(&self) -> &'static str {
        "HTTP Body"
    }

    fn request_has_request_body(&self) -> &'static str {
        "有请求数据"
    }

    fn request_has_response_body(&self) -> &'static str {
        "有响应数据"
    }

    fn request_label_notes(&self) -> &'static str {
        "连接日志"
    }

    fn request_notes_more(&self, count: usize) -> String {
        format!("还有 {} 条日志", count)
    }

    // ========== 分组 ==========
    fn key_group(&self) -> &'static str {
        "[g]分组"
    }

    fn request_app_list_title(&self) -> &'static str {
        "应用列表"
    }

    fn request_all_mode(&self) -> &'static str {
        "所有请求"
    }

    fn request_grouped_mode(&self) -> &'static str {
        "按应用分组"
    }

    fn request_no_app_selected(&self) -> &'static str {
        "未选择应用"
    }

    // ========== 帮助 ==========
    fn key_help(&self) -> &'static str {
        "[?]帮助"
    }

    fn help_title(&self) -> &'static str {
        " 快捷键帮助 [ESC 关闭] "
    }

    fn help_global_section(&self) -> &'static str {
        "全局快捷键"
    }

    fn help_view_section(&self) -> &'static str {
        "当前视图"
    }

    fn help_navigation_section(&self) -> &'static str {
        "导航"
    }

    // ---- 全局快捷键行 ----
    fn help_shortcut_quit(&self) -> &'static str {
        "  q          - 退出程序"
    }

    fn help_shortcut_refresh(&self) -> &'static str {
        "  r          - 刷新数据"
    }

    fn help_shortcut_switch_view(&self) -> &'static str {
        "  1-5        - 切换视图"
    }

    fn help_shortcut_toggle_outbound(&self) -> &'static str {
        "  m          - 切换出站模式"
    }

    fn help_shortcut_notification_history(&self) -> &'static str {
        "  n          - 通知历史"
    }

    fn help_shortcut_devtools(&self) -> &'static str {
        "  `          - 开发工具"
    }

    fn help_shortcut_help(&self) -> &'static str {
        "  ?          - 此帮助"
    }

    // ---- 视图专属快捷键行 ----
    fn help_shortcut_toggle_mitm(&self) -> &'static str {
        "  i          - 切换 MITM"
    }

    fn help_shortcut_toggle_capture(&self) -> &'static str {
        "  c          - 切换流量捕获"
    }

    fn help_shortcut_search(&self) -> &'static str {
        "  /          - 搜索"
    }

    fn help_shortcut_test_latency(&self) -> &'static str {
        "  t          - 测试延迟"
    }

    fn help_shortcut_enter_select_policy(&self) -> &'static str {
        "  Enter      - 进入/选择策略"
    }

    fn help_shortcut_esc_back(&self) -> &'static str {
        "  ESC        - 返回"
    }

    fn help_shortcut_toggle_group(&self) -> &'static str {
        "  g          - 切换分组模式"
    }

    fn help_shortcut_switch_app(&self) -> &'static str {
        "  h/l        - 切换应用"
    }

    fn help_shortcut_flush_dns(&self) -> &'static str {
        "  f          - 清空 DNS 缓存"
    }

    // ---- 导航行 ----
    fn help_nav_up_down(&self) -> &'static str {
        "  j/k 或 ↓/↑  - 上下移动"
    }

    fn help_nav_left_right(&self) -> &'static str {
        "  h/l 或 ←/→  - 左右切换应用"
    }

    // ========== 通用操作词 ==========
    fn action_select(&self) -> &'static str {
        "选择"
    }

    fn action_enter(&self) -> &'static str {
        "进入"
    }

    fn action_confirm(&self) -> &'static str {
        "确认"
    }

    fn action_back(&self) -> &'static str {
        "返回"
    }

    fn action_test(&self) -> &'static str {
        "测试"
    }

    fn action_search(&self) -> &'static str {
        "搜索"
    }

    fn action_toggle(&self) -> &'static str {
        "切换"
    }

    fn action_group(&self) -> &'static str {
        "分组"
    }

    fn action_mode(&self) -> &'static str {
        "模式"
    }

    fn action_kill(&self) -> &'static str {
        "终止"
    }

    // ========== 连接终止确认 ==========
    fn confirm_kill_title(&self) -> &'static str {
        " 确认终止连接 "
    }

    fn confirm_kill_message(&self, url: &str) -> String {
        format!("确定要终止到 {} 的连接吗？", url)
    }

    fn confirm_kill_hint(&self) -> &'static str {
        "[Enter] 确认  [ESC] 取消"
    }

    fn confirm_kill_label_target(&self) -> &'static str {
        "目标: "
    }

    fn confirm_kill_label_process(&self) -> &'static str {
        "进程: "
    }

    fn confirm_kill_label_traffic(&self) -> &'static str {
        "流量: "
    }

    fn notification_connection_killed(&self) -> &'static str {
        "连接已终止"
    }

    fn notification_kill_failed(&self, error: &str) -> String {
        format!("终止连接失败: {}", error)
    }

    // ========== 功能开关 ==========
    fn feature_mitm(&self) -> &'static str {
        "MITM"
    }

    fn feature_capture(&self) -> &'static str {
        "流量捕获"
    }

    fn status_enabled(&self) -> &'static str {
        "已启用"
    }

    fn status_disabled(&self) -> &'static str {
        "已禁用"
    }

    fn notification_mitm_enabled(&self) -> &'static str {
        "MITM 已启用"
    }

    fn notification_mitm_disabled(&self) -> &'static str {
        "MITM 已禁用"
    }

    fn notification_capture_enabled(&self) -> &'static str {
        "流量捕获已启用"
    }

    fn notification_capture_disabled(&self) -> &'static str {
        "流量捕获已禁用"
    }

    fn notification_feature_toggle_failed(&self, error: &str) -> String {
        format!("功能切换失败: {}", error)
    }

    // ========== DNS ==========
    fn view_dns(&self) -> &'static str {
        "DNS 缓存"
    }

    fn dns_list_title(&self) -> &'static str {
        "DNS 缓存列表"
    }

    fn dns_detail_title(&self) -> &'static str {
        "DNS 详情"
    }

    fn dns_no_records(&self) -> &'static str {
        "暂无 DNS 缓存"
    }

    fn dns_label_domain(&self) -> &'static str {
        "域名"
    }

    fn dns_label_value(&self) -> &'static str {
        "IP 地址"
    }

    fn dns_label_ttl(&self) -> &'static str {
        "TTL"
    }

    fn action_flush(&self) -> &'static str {
        "清空"
    }

    fn notification_dns_flushed(&self) -> &'static str {
        "DNS 缓存已清空"
    }

    fn notification_dns_flush_failed(&self, error: &str) -> String {
        format!("清空 DNS 缓存失败: {}", error)
    }
}
