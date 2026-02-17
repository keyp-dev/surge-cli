/// US English translation
use super::Translate;

pub struct EnUS;

impl Translate for EnUS {
    // ========== UI Status Bar ==========
    fn ui_status_running(&self) -> &'static str {
        "Surge Running"
    }

    fn ui_status_stopped(&self) -> &'static str {
        "Surge Stopped"
    }

    fn ui_status_http_api(&self) -> &'static str {
        "(HTTP API)"
    }

    fn ui_status_cli_mode(&self) -> &'static str {
        "(CLI Mode)"
    }

    // ========== Keyboard Shortcuts ==========
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
        "[Enter]open"
    }

    fn key_esc(&self) -> &'static str {
        "[ESC]back"
    }

    fn key_start(&self) -> &'static str {
        "[s]tart"
    }

    fn key_reload(&self) -> &'static str {
        "[r]eload"
    }

    // ========== View Titles ==========
    fn views_title(&self) -> &'static str {
        "Views"
    }

    fn view_overview(&self) -> &'static str {
        "Overview"
    }

    fn view_policies(&self) -> &'static str {
        "Policies"
    }

    fn view_requests(&self) -> &'static str {
        "Requests"
    }

    fn view_connections(&self) -> &'static str {
        "Connections"
    }

    // ========== Notifications ==========
    fn notification_test_started(&self) -> &'static str {
        "Policy latency test started..."
    }

    fn notification_test_completed(&self, alive: usize, total: usize) -> String {
        format!("Test completed: {}/{} available", alive, total)
    }

    fn notification_test_failed(&self, error: &str) -> String {
        format!("Test failed: {}", error)
    }

    // ========== Alerts ==========
    fn alert_surge_not_running(&self) -> &'static str {
        "Surge not running - Press S to start"
    }

    fn alert_http_api_disabled(&self) -> &'static str {
        "HTTP API unavailable - Press R to reload config"
    }

    // ========== Policy Groups ==========
    fn policy_group_title(&self) -> &'static str {
        "Policy Groups"
    }

    fn policy_group_enter_hint(&self) -> &'static str {
        "Policy Groups [Enter to open]"
    }

    fn policy_policies_title(&self, group_name: &str) -> String {
        format!("Group: {}", group_name)
    }

    fn policy_select_title(&self, group_name: &str) -> String {
        format!("Group: {} [Select Policy]", group_name)
    }

    fn policy_testing(&self) -> &'static str {
        "[Testing...]"
    }

    fn policy_testing_hint(&self) -> &'static str {
        " [Testing... Press R after completion]"
    }

    fn policy_available(&self) -> &'static str {
        "[Available]"
    }

    fn policy_unavailable(&self) -> &'static str {
        "[Unavailable]"
    }

    fn policy_no_groups(&self) -> &'static str {
        "No policy groups"
    }

    fn policy_no_policies(&self) -> &'static str {
        "No policies in this group"
    }

    fn policy_no_selection(&self) -> &'static str {
        "No policy group selected"
    }

    // ========== DevTools ==========
    fn devtools_title(&self) -> &'static str {
        " DevTools [ESC to close] "
    }

    fn devtools_no_logs(&self) -> &'static str {
        "No logs"
    }

    // ========== Notification History ==========
    fn notification_history_title(&self) -> &'static str {
        " Notification History [ESC to close] "
    }

    fn notification_history_empty(&self) -> &'static str {
        "No notifications"
    }

    // ========== Overview ==========
    fn overview_surge_status(&self) -> &'static str {
        "Surge Status"
    }

    fn overview_api_status(&self) -> &'static str {
        "API Status"
    }

    fn overview_outbound_mode(&self) -> &'static str {
        "Outbound Mode"
    }

    fn overview_stats(&self) -> &'static str {
        "Statistics"
    }

    // ========== OutboundMode ==========
    fn outbound_mode_direct(&self) -> &'static str {
        "Direct Mode"
    }

    fn outbound_mode_proxy(&self) -> &'static str {
        "Global Proxy"
    }

    fn outbound_mode_rule(&self) -> &'static str {
        "Rule Mode"
    }

    // ========== AlertAction ==========
    fn alert_action_start_surge(&self) -> &'static str {
        "Press S to start Surge"
    }

    fn alert_action_reload_config(&self) -> &'static str {
        "Press R to reload config"
    }

    // ========== Statistics Labels ==========
    fn stats_policies(&self) -> &'static str {
        "Policies"
    }

    fn stats_policy_groups(&self) -> &'static str {
        "Policy Groups"
    }

    fn stats_active_connections(&self) -> &'static str {
        "Active Connections"
    }

    fn stats_recent_requests(&self) -> &'static str {
        "Recent Requests"
    }

    // ========== Requests ==========
    fn request_list_title(&self) -> &'static str {
        "Request List"
    }

    fn request_detail_title(&self) -> &'static str {
        "Request Details"
    }

    fn request_no_requests(&self) -> &'static str {
        "No requests"
    }

    fn request_no_selection(&self) -> &'static str {
        "No request selected"
    }

    fn request_status_completed(&self) -> &'static str {
        "✓ Completed"
    }

    fn request_status_failed(&self) -> &'static str {
        "✗ Failed"
    }

    fn request_status_in_progress(&self) -> &'static str {
        "○ In Progress"
    }

    fn request_label_request(&self) -> &'static str {
        "Request"
    }

    fn request_label_host(&self) -> &'static str {
        "Host"
    }

    fn request_label_rule(&self) -> &'static str {
        "Rule"
    }

    fn request_label_policy(&self) -> &'static str {
        "Policy"
    }

    fn request_label_traffic(&self) -> &'static str {
        "Traffic"
    }

    fn request_label_upload(&self) -> &'static str {
        "Upload"
    }

    fn request_label_download(&self) -> &'static str {
        "Download"
    }

    fn request_label_process(&self) -> &'static str {
        "Process"
    }

    fn request_label_time(&self) -> &'static str {
        "Time"
    }

    fn request_time_seconds_ago(&self, secs: u64) -> String {
        format!("{} seconds ago", secs)
    }

    fn request_time_minutes_ago(&self, mins: u64) -> String {
        format!("{} minutes ago", mins)
    }

    fn request_time_hours_ago(&self, hours: u64) -> String {
        format!("{} hours ago", hours)
    }

    fn request_label_http_body(&self) -> &'static str {
        "HTTP Body"
    }

    fn request_has_request_body(&self) -> &'static str {
        "Has Request Body"
    }

    fn request_has_response_body(&self) -> &'static str {
        "Has Response Body"
    }

    fn request_label_notes(&self) -> &'static str {
        "Connection Logs"
    }

    fn request_notes_more(&self, count: usize) -> String {
        format!("{} more logs", count)
    }

    // ========== Grouping ==========
    fn key_group(&self) -> &'static str {
        "[g]roup"
    }

    fn request_app_list_title(&self) -> &'static str {
        "Applications"
    }

    fn request_all_mode(&self) -> &'static str {
        "All Requests"
    }

    fn request_grouped_mode(&self) -> &'static str {
        "Grouped by App"
    }

    fn request_no_app_selected(&self) -> &'static str {
        "No application selected"
    }

    // ========== Help ==========
    fn key_help(&self) -> &'static str {
        "[?]help"
    }

    fn help_title(&self) -> &'static str {
        " Keyboard Shortcuts [ESC to close] "
    }

    fn help_global_section(&self) -> &'static str {
        "Global Shortcuts"
    }

    fn help_view_section(&self) -> &'static str {
        "Current View"
    }

    fn help_navigation_section(&self) -> &'static str {
        "Navigation"
    }

    // ---- Global shortcut lines ----
    fn help_shortcut_quit(&self) -> &'static str {
        "  q          - quit"
    }

    fn help_shortcut_refresh(&self) -> &'static str {
        "  r          - refresh data"
    }

    fn help_shortcut_switch_view(&self) -> &'static str {
        "  1-5        - switch view"
    }

    fn help_shortcut_toggle_outbound(&self) -> &'static str {
        "  m          - toggle outbound mode"
    }

    fn help_shortcut_notification_history(&self) -> &'static str {
        "  n          - notification history"
    }

    fn help_shortcut_devtools(&self) -> &'static str {
        "  `          - devtools"
    }

    fn help_shortcut_help(&self) -> &'static str {
        "  ?          - this help"
    }

    // ---- View-specific shortcut lines ----
    fn help_shortcut_toggle_mitm(&self) -> &'static str {
        "  i          - toggle MITM"
    }

    fn help_shortcut_toggle_capture(&self) -> &'static str {
        "  c          - toggle capture"
    }

    fn help_shortcut_search(&self) -> &'static str {
        "  /          - search"
    }

    fn help_shortcut_test_latency(&self) -> &'static str {
        "  t          - test latency"
    }

    fn help_shortcut_enter_select_policy(&self) -> &'static str {
        "  Enter      - enter/select policy"
    }

    fn help_shortcut_esc_back(&self) -> &'static str {
        "  ESC        - back"
    }

    fn help_shortcut_toggle_group(&self) -> &'static str {
        "  g          - toggle grouped mode"
    }

    fn help_shortcut_switch_app(&self) -> &'static str {
        "  h/l        - switch app"
    }

    fn help_shortcut_flush_dns(&self) -> &'static str {
        "  f          - flush DNS cache"
    }

    // ---- Navigation lines ----
    fn help_nav_up_down(&self) -> &'static str {
        "  j/k or ↓/↑  - move up/down"
    }

    fn help_nav_left_right(&self) -> &'static str {
        "  h/l or ←/→  - switch app"
    }

    // ========== Common Actions ==========
    fn action_select(&self) -> &'static str {
        "Select"
    }

    fn action_enter(&self) -> &'static str {
        "Enter"
    }

    fn action_confirm(&self) -> &'static str {
        "Confirm"
    }

    fn action_back(&self) -> &'static str {
        "Back"
    }

    fn action_test(&self) -> &'static str {
        "Test"
    }

    fn action_search(&self) -> &'static str {
        "Search"
    }

    fn action_toggle(&self) -> &'static str {
        "Toggle"
    }

    fn action_group(&self) -> &'static str {
        "Group"
    }

    fn action_mode(&self) -> &'static str {
        "Mode"
    }

    fn action_kill(&self) -> &'static str {
        "Kill"
    }

    // ========== Kill Connection Confirmation ==========
    fn confirm_kill_title(&self) -> &'static str {
        " Confirm Kill Connection "
    }

    fn confirm_kill_message(&self, url: &str) -> String {
        format!("Are you sure to kill connection to {}?", url)
    }

    fn confirm_kill_hint(&self) -> &'static str {
        "[Enter] Confirm  [ESC] Cancel"
    }

    fn confirm_kill_label_target(&self) -> &'static str {
        "Target: "
    }

    fn confirm_kill_label_process(&self) -> &'static str {
        "Process: "
    }

    fn confirm_kill_label_traffic(&self) -> &'static str {
        "Traffic: "
    }

    fn notification_connection_killed(&self) -> &'static str {
        "Connection killed"
    }

    fn notification_kill_failed(&self, error: &str) -> String {
        format!("Failed to kill connection: {}", error)
    }

    // ========== Feature Toggles ==========
    fn feature_mitm(&self) -> &'static str {
        "MITM"
    }

    fn feature_capture(&self) -> &'static str {
        "Traffic Capture"
    }

    fn status_enabled(&self) -> &'static str {
        "Enabled"
    }

    fn status_disabled(&self) -> &'static str {
        "Disabled"
    }

    fn notification_mitm_enabled(&self) -> &'static str {
        "MITM enabled"
    }

    fn notification_mitm_disabled(&self) -> &'static str {
        "MITM disabled"
    }

    fn notification_capture_enabled(&self) -> &'static str {
        "Traffic capture enabled"
    }

    fn notification_capture_disabled(&self) -> &'static str {
        "Traffic capture disabled"
    }

    fn notification_feature_toggle_failed(&self, error: &str) -> String {
        format!("Feature toggle failed: {}", error)
    }

    // ========== DNS ==========
    fn view_dns(&self) -> &'static str {
        "DNS Cache"
    }

    fn dns_list_title(&self) -> &'static str {
        "DNS Cache List"
    }

    fn dns_detail_title(&self) -> &'static str {
        "DNS Details"
    }

    fn dns_no_records(&self) -> &'static str {
        "No DNS cache"
    }

    fn dns_label_domain(&self) -> &'static str {
        "Domain"
    }

    fn dns_label_value(&self) -> &'static str {
        "IP Address"
    }

    fn dns_label_ttl(&self) -> &'static str {
        "TTL"
    }

    fn action_flush(&self) -> &'static str {
        "Flush"
    }

    fn notification_dns_flushed(&self) -> &'static str {
        "DNS cache flushed"
    }

    fn notification_dns_flush_failed(&self, error: &str) -> String {
        format!("Failed to flush DNS cache: {}", error)
    }
}
