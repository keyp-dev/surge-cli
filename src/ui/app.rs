/// UI 应用状态和事件处理
use crate::application::SurgeClient;
use crate::domain::entities::{AlertAction, AppSnapshot, ViewMode};
use crate::domain::models::PolicyDetail;
use chrono::{DateTime, Local};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;

// 导入 Notification 类型
use super::components::notifications::{Notification, NotificationLevel};

/// 后台测试任务的消息类型
#[derive(Debug)]
enum TestMessage {
    /// 测试开始
    Started,
    /// 测试完成
    Completed {
        group_name: String,
        results: Vec<PolicyDetail>,
    },
    /// 测试失败
    Failed { error: String },
}

// Notification 辅助函数
impl Notification {
    fn new(message: String, level: NotificationLevel) -> Self {
        Self {
            message,
            level,
            created_at: Local::now(),
        }
    }

    fn info(message: String) -> Self {
        Self::new(message, NotificationLevel::Info)
    }

    fn success(message: String) -> Self {
        Self::new(message, NotificationLevel::Success)
    }

    fn error(message: String) -> Self {
        Self::new(message, NotificationLevel::Error)
    }
}

/// 应用状态
pub struct App {
    /// Surge 客户端
    client: SurgeClient,
    /// 当前视图
    current_view: ViewMode,
    /// 应用快照
    snapshot: AppSnapshot,
    /// 是否应该退出
    should_quit: bool,
    /// 刷新间隔
    refresh_interval: Duration,
    /// 当前选择的索引（用于列表导航）
    selected_index: usize,
    /// 策略组内部选中的策略索引（None 表示在策略组列表，Some(idx) 表示在策略组内部）
    policy_detail_index: Option<usize>,
    /// 正在测试的策略组名称（用于显示测试状态）
    testing_policy_group: Option<String>,
    /// 后台测试消息接收端
    test_rx: mpsc::Receiver<TestMessage>,
    /// 后台测试消息发送端
    test_tx: mpsc::Sender<TestMessage>,
    /// 通知列表（最多保留 10 条）
    notifications: Vec<Notification>,
    /// 是否显示通知历史弹窗
    show_notification_history: bool,
    /// 是否显示 DevTools 面板
    show_devtools: bool,
    /// DevTools 日志条目
    devtools_logs: Vec<DevToolsLog>,
    /// 策略延迟测试结果缓存（key: 策略名, value: 测试结果）
    /// 缓存不会因为 refresh 而丢失，只在新测试时更新
    policy_test_cache: HashMap<String, PolicyDetail>,
    /// 翻译器实例（编译时确定语言）
    t: &'static dyn crate::i18n::Translate,
    /// 搜索模式标志
    search_mode: bool,
    /// 搜索关键词（策略组列表）
    search_query: String,
    /// 策略详情搜索关键词
    policy_detail_search: String,
    /// 分组模式标志（仅用于 Requests 和 ActiveConnections）
    grouped_mode: bool,
    /// 分组模式下选中的应用索引
    grouped_app_index: usize,
    /// 是否显示帮助弹窗
    show_help: bool,
    /// 待确认终止的连接 ID（Some(id) 时显示确认框）
    show_kill_confirm: Option<u64>,
}

/// DevTools 日志条目
#[derive(Debug, Clone)]
struct DevToolsLog {
    timestamp: DateTime<Local>,
    level: LogLevel,
    message: String,
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl DevToolsLog {
    fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            message,
        }
    }

    fn color(&self) -> Color {
        match self.level {
            LogLevel::Debug => Color::DarkGray,
            LogLevel::Info => Color::Cyan,
            LogLevel::Warning => Color::Yellow,
            LogLevel::Error => Color::Red,
        }
    }

    fn level_str(&self) -> &str {
        match self.level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO ",
            LogLevel::Warning => "WARN ",
            LogLevel::Error => "ERROR",
        }
    }
}

impl App {
    /// 创建新应用
    pub fn new(client: SurgeClient, refresh_interval_secs: u64) -> Self {
        // 创建后台测试消息通道（缓冲区大小为 1）
        let (test_tx, test_rx) = mpsc::channel(1);

        Self {
            client,
            current_view: ViewMode::Overview,
            snapshot: AppSnapshot::new(),
            should_quit: false,
            refresh_interval: Duration::from_secs(refresh_interval_secs),
            selected_index: 0,
            policy_detail_index: None,
            testing_policy_group: None,
            test_rx,
            test_tx,
            notifications: Vec::new(),
            show_notification_history: false,
            show_devtools: false,
            devtools_logs: Vec::new(),
            policy_test_cache: HashMap::new(),
            t: crate::i18n::current(),
            search_mode: false,
            search_query: String::new(),
            policy_detail_search: String::new(),
            grouped_mode: false,
            grouped_app_index: 0,
            show_help: false,
            show_kill_confirm: None,
        }
    }

    /// 添加通知
    fn add_notification(&mut self, notification: Notification) {
        self.notifications.push(notification);
        // 保留最近 50 条历史
        if self.notifications.len() > 50 {
            self.notifications.remove(0);
        }
    }

    /// 添加 DevTools 日志
    fn add_devtools_log(&mut self, level: LogLevel, message: String) {
        self.devtools_logs.push(DevToolsLog::new(level, message));
        // 保留最近 200 条
        if self.devtools_logs.len() > 200 {
            self.devtools_logs.remove(0);
        }
    }

    /// 清理过期通知（仅清理状态栏显示的，历史保留）
    fn clean_expired_notifications(&mut self) {
        // 不再自动清理，保留历史
    }

    /// 运行应用
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        // 初始加载
        self.refresh().await;

        loop {
            // 清理过期通知
            self.clean_expired_notifications();

            // 渲染 UI
            terminal.draw(|f| self.render(f))?;

            // 处理后台测试消息（非阻塞）
            let mut has_test_message = false;
            while let Ok(msg) = self.test_rx.try_recv() {
                self.handle_test_message(msg);
                has_test_message = true;
            }

            // 如果处理了测试消息，立即重绘 UI（不等用户交互）
            if has_test_message {
                terminal.draw(|f| self.render(f))?;
            }

            // 处理事件（非阻塞，使用超时）
            // 只在超时（无按键）时刷新数据，避免用户操作时列表内容变化
            if event::poll(self.refresh_interval)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key).await;
                }
            } else {
                // 超时了才刷新，保持用户操作时列表稳定
                self.refresh().await;
            }

            // 检查是否退出
            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// 刷新数据
    async fn refresh(&mut self) {
        self.snapshot = self.client.get_snapshot().await;

        // 从缓存恢复测试结果（避免刷新后丢失）
        if !self.policy_test_cache.is_empty() {
            self.snapshot.policies = self.policy_test_cache.values().cloned().collect();
        }

        // 确保选择索引有效
        let max_index = self.get_current_list_len();
        if max_index > 0 && self.selected_index >= max_index {
            self.selected_index = max_index - 1;
        }
        // Clear test status after refresh (only if test is completed)
        if let Some(ref status) = self.testing_policy_group {
            // If status is not "testing" text, test is completed or failed, clear it
            if status != self.t.policy_testing() {
                self.testing_policy_group = None;
            }
        }
    }

    /// 处理后台测试消息
    fn handle_test_message(&mut self, msg: TestMessage) {
        match msg {
            TestMessage::Started => {
                self.testing_policy_group = Some(self.t.policy_testing().to_string());
                self.add_notification(Notification::info(
                    self.t.notification_test_started().to_string(),
                ));
                self.add_devtools_log(
                    LogLevel::Info,
                    self.t.notification_test_started().to_string(),
                );
                tracing::info!("Test started");
            }
            TestMessage::Completed {
                group_name,
                results,
            } => {
                let alive_count = results.iter().filter(|p| p.alive).count();
                tracing::info!(
                    "✓ Test completed: {} policies, {} available",
                    results.len(),
                    alive_count
                );

                // Debug: 先收集所有需要记录的数据（避免借用冲突）
                let test_result_names: Vec<String> = results
                    .iter()
                    .take(5)
                    .map(|p| {
                        format!(
                            "'{}' - {}ms (alive={})",
                            p.name,
                            p.latency
                                .map(|l| l.to_string())
                                .unwrap_or("N/A".to_string()),
                            p.alive
                        )
                    })
                    .collect();

                let group_policy_names: Vec<String> = self
                    .snapshot
                    .policy_groups
                    .iter()
                    .find(|g| g.name == group_name)
                    .map(|g| g.policies.iter().take(5).map(|p| p.name.clone()).collect())
                    .unwrap_or_default();

                // Debug: 记录测试结果中的策略名称
                self.add_devtools_log(
                    LogLevel::Info,
                    format!("=== Test results policy names (first 5) ==="),
                );
                for (i, name_info) in test_result_names.iter().enumerate() {
                    self.add_devtools_log(LogLevel::Info, format!("  [{}] {}", i, name_info));
                }

                // Debug: 记录策略组中的策略名称（用于对比）
                if !group_policy_names.is_empty() {
                    self.add_devtools_log(
                        LogLevel::Info,
                        format!("=== Policy names in group '{}' (first 5) ===", group_name),
                    );
                    for (i, name) in group_policy_names.iter().enumerate() {
                        self.add_devtools_log(LogLevel::Info, format!("  [{}] '{}'", i, name));
                    }
                }

                // 更新测试结果缓存（不会因为 refresh 而丢失）
                for policy in &results {
                    self.policy_test_cache
                        .insert(policy.name.clone(), policy.clone());
                }

                self.add_devtools_log(
                    LogLevel::Info,
                    format!(
                        "Test results cached: {} policies (total cache: {})",
                        results.len(),
                        self.policy_test_cache.len()
                    ),
                );

                // 同时更新 snapshot.policies 以便立即显示
                self.snapshot.policies = results.clone();

                // 更新策略组的可用策略列表
                let group_policies: Vec<String> = self
                    .snapshot
                    .policy_groups
                    .iter()
                    .find(|g| g.name == group_name)
                    .map(|g| g.policies.iter().map(|p| p.name.clone()).collect())
                    .unwrap_or_default();

                let available: Vec<String> = results
                    .iter()
                    .filter(|p| p.alive && group_policies.contains(&p.name))
                    .map(|p| p.name.clone())
                    .collect();

                // Debug: 记录名称匹配结果
                self.add_devtools_log(
                    LogLevel::Info,
                    format!(
                        "Name matching: group has {} policies, matched {} available in results",
                        group_policies.len(),
                        available.len()
                    ),
                );

                if let Some(group) = self
                    .snapshot
                    .policy_groups
                    .iter_mut()
                    .find(|g| g.name == group_name)
                {
                    group.available_policies = Some(available.clone());
                }

                self.testing_policy_group = None; // 清除测试状态
                self.add_notification(Notification::success(
                    self.t
                        .notification_test_completed(alive_count, results.len()),
                ));
            }
            TestMessage::Failed { error } => {
                tracing::error!("Test failed: {}", error);
                self.add_devtools_log(LogLevel::Error, self.t.notification_test_failed(&error));
                self.testing_policy_group = None;
                self.add_notification(Notification::error(self.t.notification_test_failed(&error)));
            }
        }
    }

    /// 获取当前视图的列表长度（考虑显示限制和搜索过滤）
    fn get_current_list_len(&self) -> usize {
        match self.current_view {
            ViewMode::Overview => 0,
            ViewMode::Policies => self.snapshot.policy_groups.len(),
            ViewMode::Dns => {
                // DNS 视图：返回 DNS 缓存数量（考虑搜索过滤）
                if self.search_query.is_empty() {
                    self.snapshot.dns_cache.len()
                } else {
                    let query_lower = self.search_query.to_lowercase();
                    self.snapshot
                        .dns_cache
                        .iter()
                        .filter(|r| r.domain.to_lowercase().contains(&query_lower))
                        .count()
                }
            }
            ViewMode::Requests | ViewMode::ActiveConnections => {
                if self.grouped_mode {
                    // 分组模式：返回当前选中应用的请求数量（考虑搜索过滤）
                    self.get_grouped_request_count(&self.search_query)
                } else {
                    // 普通模式：返回全部请求数量（限制 50 条，考虑搜索过滤）
                    let requests = match self.current_view {
                        ViewMode::Requests => &self.snapshot.recent_requests,
                        ViewMode::ActiveConnections => &self.snapshot.active_connections,
                        _ => return 0,
                    };

                    // 应用搜索过滤
                    if self.search_query.is_empty() {
                        requests.len().min(50)
                    } else {
                        let query_lower = self.search_query.to_lowercase();
                        requests
                            .iter()
                            .filter(|r| {
                                r.url
                                    .as_ref()
                                    .map(|u| u.to_lowercase().contains(&query_lower))
                                    .unwrap_or(false)
                                    || r.policy_name
                                        .as_ref()
                                        .map(|p| p.to_lowercase().contains(&query_lower))
                                        .unwrap_or(false)
                                    || r.process_path
                                        .as_ref()
                                        .map(|p| p.to_lowercase().contains(&query_lower))
                                        .unwrap_or(false)
                            })
                            .count()
                            .min(50)
                    }
                }
            }
        }
    }

    /// 获取分组模式下的应用数量
    fn get_grouped_app_count(&self) -> usize {
        use std::collections::HashSet;
        let requests = match self.current_view {
            ViewMode::Requests => &self.snapshot.recent_requests,
            ViewMode::ActiveConnections => &self.snapshot.active_connections,
            _ => return 0,
        };

        // 统计唯一的应用名称
        let apps: HashSet<String> = requests
            .iter()
            .filter_map(|r| {
                r.process_path
                    .as_ref()
                    .map(|p| p.split('/').last().unwrap_or(p).to_string())
            })
            .collect();

        apps.len()
            + if requests.iter().any(|r| r.process_path.is_none()) {
                1
            } else {
                0
            } // +1 for "Unknown"
    }

    /// 获取分组模式下当前选中应用的请求数量（考虑搜索过滤）
    fn get_grouped_request_count(&self, search_query: &str) -> usize {
        use std::collections::HashMap;

        let requests = match self.current_view {
            ViewMode::Requests => &self.snapshot.recent_requests,
            ViewMode::ActiveConnections => &self.snapshot.active_connections,
            _ => return 0,
        };

        // 按 process_path 分组（复用 render_grouped_view 的逻辑）
        let mut app_groups: HashMap<String, Vec<&crate::domain::models::Request>> = HashMap::new();
        for req in requests {
            let app_name = req
                .process_path
                .as_ref()
                .map(|p| p.split('/').last().unwrap_or(p).to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            app_groups.entry(app_name).or_default().push(req);
        }

        // 排序应用列表（和 render_grouped_view 保持一致）
        let mut apps: Vec<(String, usize)> = app_groups
            .iter()
            .map(|(name, reqs)| (name.clone(), reqs.len()))
            .collect();
        apps.sort_by(|a, b| match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => a.0.cmp(&b.0),
            other => other,
        });

        // 获取第 grouped_app_index 个应用的请求
        if self.grouped_app_index >= apps.len() {
            return 0;
        }

        let (selected_app_name, _) = &apps[self.grouped_app_index];
        let app_requests = app_groups.get(selected_app_name).unwrap();

        // 应用搜索过滤
        if search_query.is_empty() {
            app_requests.len().min(50)
        } else {
            let query_lower = search_query.to_lowercase();
            app_requests
                .iter()
                .filter(|r| {
                    r.url
                        .as_ref()
                        .map(|u| u.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                        || r.policy_name
                            .as_ref()
                            .map(|p| p.to_lowercase().contains(&query_lower))
                            .unwrap_or(false)
                })
                .count()
                .min(50)
        }
    }

    /// 处理按键事件
    async fn handle_key(&mut self, key: KeyEvent) {
        // Kill confirmation popup handling
        if let Some(connection_id) = self.show_kill_confirm {
            match key.code {
                KeyCode::Enter => {
                    // 执行终止连接
                    if let Err(e) = self.client.kill_connection(connection_id).await {
                        self.add_notification(Notification::error(
                            self.t.notification_kill_failed(&e.to_string()),
                        ));
                    } else {
                        self.add_notification(Notification::success(
                            self.t.notification_connection_killed().to_string(),
                        ));
                        // 刷新列表
                        self.refresh().await;
                    }
                    self.show_kill_confirm = None;
                    return;
                }
                KeyCode::Esc => {
                    // 取消操作
                    self.show_kill_confirm = None;
                    return;
                }
                _ => {
                    // 阻止其他按键
                    return;
                }
            }
        }

        // Popup mode handling - only allow ESC to close
        if self.show_help || self.show_notification_history || self.show_devtools {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    // Close any open popup
                    if self.show_help {
                        self.show_help = false;
                    } else if self.show_notification_history {
                        self.show_notification_history = false;
                    } else if self.show_devtools {
                        self.show_devtools = false;
                    }
                    return;
                }
                _ => {
                    // Block all other keys when popup is showing
                    return;
                }
            }
        }

        // Search mode handling - completely block all other keys
        if self.search_mode {
            match key.code {
                KeyCode::Char(c) => {
                    // Determine which search query to update based on current view
                    if self.current_view == ViewMode::Policies && self.policy_detail_index.is_some()
                    {
                        // In policy detail mode - search policies
                        self.policy_detail_search.push(c);
                    } else {
                        // In policy group list or other views - search groups/requests
                        self.search_query.push(c);
                    }
                    self.selected_index = 0;
                    return;
                }
                KeyCode::Backspace => {
                    if self.current_view == ViewMode::Policies && self.policy_detail_index.is_some()
                    {
                        self.policy_detail_search.pop();
                    } else {
                        self.search_query.pop();
                    }
                    self.selected_index = 0;
                    return;
                }
                KeyCode::Esc => {
                    // Clear the appropriate search query and exit search mode
                    self.search_mode = false;
                    if self.current_view == ViewMode::Policies && self.policy_detail_index.is_some()
                    {
                        self.policy_detail_search.clear();
                    } else {
                        self.search_query.clear();
                    }
                    self.selected_index = 0;
                    return;
                }
                KeyCode::Enter => {
                    // Exit search mode but keep the query
                    self.search_mode = false;
                    return;
                }
                _ => {
                    // CRITICAL: Block ALL other keys in search mode
                    // This prevents number keys, letters, etc. from triggering shortcuts
                    return;
                }
            }
        }

        match key.code {
            // Enter search mode
            KeyCode::Char('/') => {
                // Allow search in Policies, Requests, ActiveConnections, and Dns views
                let can_search = matches!(
                    self.current_view,
                    ViewMode::Policies
                        | ViewMode::Requests
                        | ViewMode::ActiveConnections
                        | ViewMode::Dns
                );

                if can_search && !self.show_notification_history && !self.show_devtools {
                    self.search_mode = true;
                    // Clear the appropriate search query based on context
                    if self.current_view == ViewMode::Policies && self.policy_detail_index.is_some()
                    {
                        self.policy_detail_search.clear();
                    } else {
                        self.search_query.clear();
                    }
                }
            }

            // 退出或返回
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Esc | KeyCode::Backspace => {
                // Clear search if active (check both search queries)
                if !self.policy_detail_search.is_empty() {
                    // Clear policy detail search first
                    self.policy_detail_search.clear();
                    self.selected_index = 0;
                } else if !self.search_query.is_empty() {
                    // Then clear main search
                    self.search_query.clear();
                    self.selected_index = 0;
                } else if self.show_notification_history {
                    // 优先关闭弹窗
                    self.show_notification_history = false;
                } else if self.show_devtools {
                    self.show_devtools = false;
                } else if self.current_view == ViewMode::Policies
                    && self.policy_detail_index.is_some()
                {
                    // 策略组内部视图：返回策略组列表
                    self.policy_detail_index = None;
                } else {
                    // 其他情况：退出程序
                    self.should_quit = true;
                }
            }

            // N 键：打开通知历史
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.show_notification_history = !self.show_notification_history;
            }

            // ` 键：打开 DevTools
            KeyCode::Char('`') | KeyCode::Char('~') => {
                self.show_devtools = !self.show_devtools;
            }

            // ? 键：打开帮助
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
            }

            // 视图切换
            KeyCode::Char('1') => {
                self.current_view = ViewMode::Overview;
                self.selected_index = 0;
                self.policy_detail_index = None;
            }
            KeyCode::Char('2') => {
                self.current_view = ViewMode::Policies;
                self.selected_index = 0;
                self.policy_detail_index = None;
            }
            KeyCode::Char('3') => {
                self.current_view = ViewMode::Requests;
                self.selected_index = 0;
                self.policy_detail_index = None;
            }
            KeyCode::Char('4') => {
                self.current_view = ViewMode::ActiveConnections;
                self.selected_index = 0;
                self.policy_detail_index = None;
            }
            KeyCode::Char('5') => {
                self.current_view = ViewMode::Dns;
                self.selected_index = 0;
                self.policy_detail_index = None;
            }

            // Toggle grouping mode (for Requests and Connections views)
            KeyCode::Char('g') | KeyCode::Char('G') => {
                if matches!(
                    self.current_view,
                    ViewMode::Requests | ViewMode::ActiveConnections
                ) {
                    self.grouped_mode = !self.grouped_mode;
                    self.selected_index = 0;
                    self.grouped_app_index = 0;
                }
            }

            // Kill connection (Connections view only)
            KeyCode::Char('k') | KeyCode::Char('K') => {
                if self.current_view == ViewMode::ActiveConnections {
                    // 获取选中的连接
                    let connections = &self.snapshot.active_connections;
                    if !connections.is_empty() {
                        let selected_connection = if self.grouped_mode {
                            // 分组模式：获取当前应用下选中的连接
                            use std::collections::HashMap;
                            let mut app_groups: HashMap<
                                String,
                                Vec<&crate::domain::models::Request>,
                            > = HashMap::new();
                            for conn in connections {
                                let app_name = conn
                                    .process_path
                                    .as_ref()
                                    .map(|p| p.split('/').last().unwrap_or(p).to_string())
                                    .unwrap_or_else(|| "Unknown".to_string());
                                app_groups.entry(app_name).or_default().push(conn);
                            }

                            // 排序应用列表
                            let mut apps: Vec<(String, usize)> = app_groups
                                .iter()
                                .map(|(name, conns)| (name.clone(), conns.len()))
                                .collect();
                            apps.sort_by(|a, b| match b.1.cmp(&a.1) {
                                std::cmp::Ordering::Equal => a.0.cmp(&b.0),
                                other => other,
                            });

                            if self.grouped_app_index < apps.len() {
                                let (selected_app_name, _) = &apps[self.grouped_app_index];
                                let app_connections: Vec<_> = app_groups
                                    .get(selected_app_name)
                                    .unwrap()
                                    .iter()
                                    .copied()
                                    .collect();

                                // 应用搜索过滤
                                let filtered: Vec<_> = if self.search_query.is_empty() {
                                    app_connections
                                } else {
                                    let query_lower = self.search_query.to_lowercase();
                                    app_connections
                                        .into_iter()
                                        .filter(|c| {
                                            c.url
                                                .as_ref()
                                                .map(|u| u.to_lowercase().contains(&query_lower))
                                                .unwrap_or(false)
                                                || c.policy_name
                                                    .as_ref()
                                                    .map(|p| {
                                                        p.to_lowercase().contains(&query_lower)
                                                    })
                                                    .unwrap_or(false)
                                        })
                                        .collect()
                                };

                                filtered
                                    .get(self.selected_index.min(filtered.len().saturating_sub(1)))
                                    .map(|c| c.id)
                            } else {
                                None
                            }
                        } else {
                            // 普通模式：直接获取选中的连接
                            // 应用搜索过滤
                            let filtered: Vec<_> = if self.search_query.is_empty() {
                                connections.iter().collect()
                            } else {
                                let query_lower = self.search_query.to_lowercase();
                                connections
                                    .iter()
                                    .filter(|c| {
                                        c.url
                                            .as_ref()
                                            .map(|u| u.to_lowercase().contains(&query_lower))
                                            .unwrap_or(false)
                                            || c.policy_name
                                                .as_ref()
                                                .map(|p| p.to_lowercase().contains(&query_lower))
                                                .unwrap_or(false)
                                            || c.process_path
                                                .as_ref()
                                                .map(|p| p.to_lowercase().contains(&query_lower))
                                                .unwrap_or(false)
                                    })
                                    .collect()
                            };

                            filtered
                                .get(self.selected_index.min(filtered.len().saturating_sub(1)))
                                .map(|c| c.id)
                        };

                        if let Some(id) = selected_connection {
                            self.show_kill_confirm = Some(id);
                        }
                    }
                }
            }

            // 列表导航
            KeyCode::Up => {
                if self.current_view == ViewMode::Policies {
                    if let Some(policy_idx) = self.policy_detail_index {
                        // 在策略组内部：导航策略列表
                        if policy_idx > 0 {
                            self.policy_detail_index = Some(policy_idx - 1);
                        }
                    } else {
                        // 在策略组列表：正常导航
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                    }
                } else {
                    // 其他视图：正常导航（请求列表）
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.current_view == ViewMode::Policies {
                    if let Some(policy_idx) = self.policy_detail_index {
                        // 在策略组内部：导航策略列表
                        if self.selected_index < self.snapshot.policy_groups.len() {
                            let group = &self.snapshot.policy_groups[self.selected_index];
                            if policy_idx < group.policies.len() - 1 {
                                self.policy_detail_index = Some(policy_idx + 1);
                            }
                        }
                    } else {
                        // 在策略组列表：正常导航
                        let max_index = self.get_current_list_len();
                        if max_index > 0 && self.selected_index < max_index - 1 {
                            self.selected_index += 1;
                        }
                    }
                } else {
                    // 其他视图：正常导航（请求列表）
                    let max_index = self.get_current_list_len();
                    if max_index > 0 && self.selected_index < max_index - 1 {
                        self.selected_index += 1;
                    }
                }
            }

            // 左右键导航应用列表（仅在分组模式下）
            KeyCode::Left | KeyCode::Char('h') => {
                if self.grouped_mode
                    && matches!(
                        self.current_view,
                        ViewMode::Requests | ViewMode::ActiveConnections
                    )
                {
                    if self.grouped_app_index > 0 {
                        self.grouped_app_index -= 1;
                        self.selected_index = 0; // 切换应用时重置请求索引
                    }
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.grouped_mode
                    && matches!(
                        self.current_view,
                        ViewMode::Requests | ViewMode::ActiveConnections
                    )
                {
                    let max_app_index = self.get_grouped_app_count();
                    if max_app_index > 0 && self.grouped_app_index < max_app_index - 1 {
                        self.grouped_app_index += 1;
                        self.selected_index = 0; // 切换应用时重置请求索引
                    }
                }
            }

            // Enter 键：进入策略组或切换策略
            KeyCode::Enter => {
                if self.current_view == ViewMode::Policies {
                    if let Some(policy_idx) = self.policy_detail_index {
                        // 在策略组内部：切换到选中的策略
                        if self.selected_index < self.snapshot.policy_groups.len() {
                            let group = &self.snapshot.policy_groups[self.selected_index];
                            if policy_idx < group.policies.len() {
                                let selected_policy = &group.policies[policy_idx];
                                // 调用 API 切换策略
                                let _ = self
                                    .client
                                    .select_policy_group(&group.name, &selected_policy.name)
                                    .await;
                                // 退出策略组内部视图
                                self.policy_detail_index = None;
                                // 刷新数据
                                self.refresh().await;
                            }
                        }
                    } else {
                        // 在策略组列表：进入策略组内部
                        if self.selected_index < self.snapshot.policy_groups.len() {
                            let group = &self.snapshot.policy_groups[self.selected_index];
                            if !group.policies.is_empty() {
                                // 找到当前选中的策略索引作为初始选择
                                let initial_idx = if let Some(current_selected) = &group.selected {
                                    group
                                        .policies
                                        .iter()
                                        .position(|p| &p.name == current_selected)
                                        .unwrap_or(0)
                                } else {
                                    0
                                };
                                self.policy_detail_index = Some(initial_idx);
                            }
                        }
                    }
                }
            }

            // T 键：测试所有策略延迟（异步后台执行，不阻塞 UI）
            KeyCode::Char('t') | KeyCode::Char('T') => {
                if self.current_view == ViewMode::Policies {
                    if self.selected_index < self.snapshot.policy_groups.len() {
                        let group_name = self.snapshot.policy_groups[self.selected_index]
                            .name
                            .clone();
                        let client = self.client.clone();
                        let tx = self.test_tx.clone();

                        // 启动后台测试任务
                        tokio::spawn(async move {
                            // 发送测试开始消息
                            let _ = tx.send(TestMessage::Started).await;

                            tracing::info!(
                                "Background test task started: testing policy group {}",
                                group_name
                            );

                            // 执行测试（在后台，不阻塞 UI）
                            match client.test_all_policies_with_latency().await {
                                Ok(policy_details) => {
                                    // 发送测试完成消息
                                    let _ = tx
                                        .send(TestMessage::Completed {
                                            group_name,
                                            results: policy_details,
                                        })
                                        .await;
                                }
                                Err(e) => {
                                    // 发送测试失败消息
                                    let _ = tx
                                        .send(TestMessage::Failed {
                                            error: e.to_string(),
                                        })
                                        .await;
                                }
                            }
                        });

                        tracing::info!("Test task started, UI continues to respond");
                    }
                }
            }

            // F 键：清空 DNS 缓存（仅 DNS 视图）
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if self.current_view == ViewMode::Dns && self.snapshot.http_api_available {
                    match self.client.flush_dns().await {
                        Ok(_) => {
                            self.add_notification(Notification::success(
                                self.t.notification_dns_flushed().to_string(),
                            ));
                            self.refresh().await;
                        }
                        Err(e) => {
                            self.add_notification(Notification::error(
                                self.t.notification_dns_flush_failed(&e.to_string()),
                            ));
                        }
                    }
                }
            }

            // 切换出站模式
            KeyCode::Char('m') | KeyCode::Char('M') => {
                use crate::domain::models::OutboundMode;
                if let Some(ref current_mode) = self.snapshot.outbound_mode {
                    // 循环切换：Direct → Proxy → Rule → Direct
                    let next_mode = match current_mode {
                        OutboundMode::Direct => OutboundMode::Proxy,
                        OutboundMode::Proxy => OutboundMode::Rule,
                        OutboundMode::Rule => OutboundMode::Direct,
                    };
                    if self
                        .client
                        .set_outbound_mode(next_mode.clone())
                        .await
                        .is_ok()
                    {
                        // 刷新以获取真实状态
                        self.refresh().await;
                    }
                }
            }

            // 切换 MITM 状态（仅 Overview 视图）
            KeyCode::Char('i') | KeyCode::Char('I') => {
                if self.current_view == ViewMode::Overview && self.snapshot.http_api_available {
                    if let Some(current_status) = self.snapshot.mitm_enabled {
                        let new_status = !current_status;
                        match self.client.set_mitm_status(new_status).await {
                            Ok(_) => {
                                // 显示通知
                                let msg = if new_status {
                                    self.t.notification_mitm_enabled()
                                } else {
                                    self.t.notification_mitm_disabled()
                                };
                                self.add_notification(Notification::success(msg.to_string()));
                                // 刷新以获取真实状态
                                self.refresh().await;
                            }
                            Err(e) => {
                                self.add_notification(Notification::error(
                                    self.t.notification_feature_toggle_failed(&e.to_string()),
                                ));
                            }
                        }
                    }
                }
            }

            // 切换 Capture 状态（仅 Overview 视图）
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if self.current_view == ViewMode::Overview && self.snapshot.http_api_available {
                    if let Some(current_status) = self.snapshot.capture_enabled {
                        let new_status = !current_status;
                        match self.client.set_capture_status(new_status).await {
                            Ok(_) => {
                                // 显示通知
                                let msg = if new_status {
                                    self.t.notification_capture_enabled()
                                } else {
                                    self.t.notification_capture_disabled()
                                };
                                self.add_notification(Notification::success(msg.to_string()));
                                // 刷新以获取真实状态
                                self.refresh().await;
                            }
                            Err(e) => {
                                self.add_notification(Notification::error(
                                    self.t.notification_feature_toggle_failed(&e.to_string()),
                                ));
                            }
                        }
                    }
                }
            }

            // Alert 操作
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // 启动 Surge
                if let Some(alert) = self.snapshot.alerts.first() {
                    if matches!(alert.action, AlertAction::StartSurge) {
                        let _ = self.client.start_surge().await;
                        self.refresh().await;
                    }
                }
            }

            KeyCode::Char('r') | KeyCode::Char('R') => {
                // 优先处理 Alert 的 ReloadConfig 操作
                if let Some(alert) = self.snapshot.alerts.first() {
                    if matches!(alert.action, AlertAction::ReloadConfig) {
                        let _ = self.client.reload_config().await;
                        self.refresh().await;
                        return;
                    }
                }
                // 否则作为手动刷新
                self.refresh().await;
            }

            _ => {}
        }
    }

    /// 渲染 UI
    fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content area
                Constraint::Length(1), // Status bar
            ])
            .split(f.area());

        // Render Tabs
        self.render_tabs(f, chunks[0]);

        // Render content (including Alerts)
        self.render_content(f, chunks[1]);

        // Render status bar
        self.render_status_bar(f, chunks[2]);
    }

    /// 渲染 Tabs
    fn render_tabs(&self, f: &mut Frame, area: Rect) {
        let titles: Vec<Line> = ViewMode::all()
            .iter()
            .map(|mode| {
                let (key_num, title) = match mode {
                    ViewMode::Overview => ("1", self.t.view_overview()),
                    ViewMode::Policies => ("2", self.t.view_policies()),
                    ViewMode::Requests => ("3", self.t.view_requests()),
                    ViewMode::ActiveConnections => ("4", self.t.view_connections()),
                    ViewMode::Dns => ("5", self.t.view_dns()),
                };

                // btop 风格：[数字] 标题
                Line::from(vec![
                    Span::raw(" ["),
                    Span::styled(
                        key_num,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("]"),
                    Span::raw(format!(" {} ", title)),
                ])
            })
            .collect();

        let selected = ViewMode::all()
            .iter()
            .position(|m| m == &self.current_view)
            .unwrap_or(0);

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.t.views_title()),
            )
            .select(selected)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(tabs, area);
    }

    /// 渲染内容
    fn render_content(&self, f: &mut Frame, area: Rect) {
        // 如果有 Alerts，分割区域
        if !self.snapshot.alerts.is_empty() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4), // Alerts
                    Constraint::Min(0),    // 实际内容
                ])
                .split(area);

            // 渲染 Alerts
            super::components::alerts::render(f, chunks[0], &self.snapshot.alerts, self.t);

            // 渲染实际内容
            self.render_view_content(f, chunks[1]);
        } else {
            // 没有 Alerts，直接渲染内容
            self.render_view_content(f, area);
        }
    }

    /// 渲染视图内容
    fn render_view_content(&self, f: &mut Frame, area: Rect) {
        // 直接渲染主内容
        self.render_main_view(f, area);

        // 渲染弹窗（覆盖在主内容之上）
        if self.show_notification_history {
            self.render_notification_history(f, area);
        }
        if self.show_devtools {
            self.render_devtools(f, area);
        }
        if self.show_help {
            self.render_help(f, area);
        }
        if self.show_kill_confirm.is_some() {
            self.render_kill_confirm(f, area);
        }
    }

    /// 渲染主视图内容
    fn render_main_view(&self, f: &mut Frame, area: Rect) {
        match self.current_view {
            ViewMode::Overview => {
                super::components::overview::render(f, area, &self.snapshot, self.t);
            }
            ViewMode::Policies => {
                super::components::policies::render(
                    f,
                    area,
                    &self.snapshot,
                    self.selected_index,
                    self.policy_detail_index,
                    self.testing_policy_group.as_deref(),
                    &self.search_query,
                    &self.policy_detail_search,
                    self.search_mode,
                    self.t,
                );
            }
            ViewMode::Requests => {
                super::components::requests::render(
                    f,
                    area,
                    &self.snapshot.recent_requests,
                    self.selected_index,
                    &self.search_query,
                    self.search_mode,
                    self.grouped_mode,
                    self.grouped_app_index,
                    false, // is_connection_view
                    self.t,
                );
            }
            ViewMode::ActiveConnections => {
                super::components::requests::render(
                    f,
                    area,
                    &self.snapshot.active_connections,
                    self.selected_index,
                    &self.search_query,
                    self.search_mode,
                    self.grouped_mode,
                    self.grouped_app_index,
                    true, // is_connection_view
                    self.t,
                );
            }
            ViewMode::Dns => {
                super::components::dns::render(
                    f,
                    area,
                    &self.snapshot.dns_cache,
                    self.selected_index,
                    &self.search_query,
                    self.search_mode,
                    self.t,
                );
            }
        }
    }

    /// 渲染状态栏
    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_text = if self.snapshot.surge_running {
            if self.snapshot.http_api_available {
                format!(
                    "{} {}",
                    self.t.ui_status_running(),
                    self.t.ui_status_http_api()
                )
            } else {
                format!(
                    "{} {}",
                    self.t.ui_status_running(),
                    self.t.ui_status_cli_mode()
                )
            }
        } else {
            self.t.ui_status_stopped().to_string()
        };

        // 构建快捷键提示（简化版）
        let mut spans = vec![
            Span::styled(
                format!(" {} ", status_text),
                if self.snapshot.surge_running {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
            Span::raw("  "),
        ];

        // 只显示最基本的快捷键提示
        spans.push(Span::raw(self.t.key_quit()));
        spans.push(Span::raw("  "));
        spans.push(Span::raw(self.t.key_help()));

        // Alert 操作快捷键（优先级高）
        if let Some(alert) = self.snapshot.alerts.first() {
            match alert.action {
                AlertAction::StartSurge => {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(
                        self.t.key_start(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                AlertAction::ReloadConfig => {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(
                        self.t.key_reload(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                AlertAction::None => {}
            }
        }

        // 状态栏左侧
        let left_line = Line::from(spans);

        // 状态栏右侧：显示最新通知（简洁模式）
        let right_spans = if let Some(latest) = self.notifications.last() {
            // 只显示最新一条，显示时间（HH:MM:SS）
            let now = Local::now();
            let elapsed = (now - latest.created_at).num_seconds().max(0);
            let time_str = latest.created_at.format("%H:%M:%S").to_string();

            let display_msg = if latest.message.len() > 30 {
                format!("{}...", &latest.message[..27])
            } else {
                latest.message.clone()
            };

            vec![
                Span::styled(
                    latest.icon(),
                    Style::default()
                        .fg(latest.color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(display_msg, Style::default().fg(latest.color())),
                if elapsed < 60 {
                    // 60秒内显示时间
                    Span::styled(
                        format!(" ({})", time_str),
                        Style::default().fg(Color::DarkGray),
                    )
                } else {
                    Span::raw("")
                },
            ]
        } else {
            vec![]
        };

        let right_line = Line::from(right_spans);

        // 分割状态栏：左侧快捷键 | 右侧通知
        let status_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),     // 左侧快捷键
                Constraint::Length(50), // 右侧通知区域
            ])
            .split(area);

        f.render_widget(Paragraph::new(left_line), status_chunks[0]);
        f.render_widget(
            Paragraph::new(right_line).alignment(Alignment::Right),
            status_chunks[1],
        );
    }

    /// 渲染通知历史弹窗
    fn render_notification_history(&self, f: &mut Frame, area: Rect) {
        // 居中弹窗 80% 宽度，70% 高度
        let popup_area = self.centered_rect(80, 70, area);

        // 构建通知列表
        let mut lines = Vec::new();
        for (i, notification) in self.notifications.iter().rev().enumerate() {
            if i > 0 {
                lines.push(Line::from(""));
            }

            // 格式化为绝对时间：YYYY-MM-DD HH:MM:SS
            let time_str = notification
                .created_at
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();

            let line = Line::from(vec![
                Span::styled(
                    format!("[{}]", time_str),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" "),
                Span::styled(
                    notification.icon(),
                    Style::default()
                        .fg(notification.color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    &notification.message,
                    Style::default().fg(notification.color()),
                ),
            ]);
            lines.push(line);
        }

        if lines.is_empty() {
            lines.push(Line::from(Span::styled(
                self.t.notification_history_empty(),
                Style::default().fg(Color::DarkGray),
            )));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", self.t.notification_history_title()))
                    .style(Style::default().bg(Color::Black).fg(Color::White)),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(ratatui::widgets::Clear, popup_area);
        f.render_widget(paragraph, popup_area);
    }

    /// 渲染 DevTools 面板
    fn render_devtools(&self, f: &mut Frame, area: Rect) {
        // 底部 70% 高度
        let devtools_area = Rect {
            x: area.x,
            y: area.y + (area.height * 30 / 100),
            width: area.width,
            height: area.height * 70 / 100,
        };

        // 构建日志列表
        let mut lines = Vec::new();
        for log in self.devtools_logs.iter().rev().take(100) {
            // 格式化为绝对时间：YYYY-MM-DD HH:MM:SS
            let time_str = log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

            let line = Line::from(vec![
                Span::styled(
                    format!("[{}]", time_str),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" "),
                Span::styled(
                    log.level_str(),
                    Style::default()
                        .fg(log.color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(&log.message, Style::default().fg(Color::White)),
            ]);
            lines.push(line);
        }

        if lines.is_empty() {
            lines.push(Line::from(Span::styled(
                self.t.devtools_no_logs(),
                Style::default().fg(Color::DarkGray),
            )));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", self.t.devtools_title()))
                    .style(Style::default().bg(Color::Black).fg(Color::White)),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(ratatui::widgets::Clear, devtools_area);
        f.render_widget(paragraph, devtools_area);
    }

    /// 渲染帮助弹窗
    fn render_help(&self, f: &mut Frame, area: Rect) {
        // 居中弹窗 70% 宽度，60% 高度
        let popup_area = self.centered_rect(70, 60, area);

        // 构建帮助内容
        let mut lines = Vec::new();

        // 全局快捷键部分
        lines.push(Line::from(vec![Span::styled(
            format!("【{}】", self.t.help_global_section()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from("  q          - 退出程序"));
        lines.push(Line::from("  r          - 刷新数据"));
        lines.push(Line::from("  1-4        - 切换视图"));
        lines.push(Line::from("  m          - 切换出站模式"));
        lines.push(Line::from("  n          - 通知历史"));
        lines.push(Line::from("  `          - 开发工具"));
        lines.push(Line::from("  ?          - 此帮助"));
        lines.push(Line::from(""));

        // 当前视图快捷键
        lines.push(Line::from(vec![Span::styled(
            format!("【{}】", self.t.help_view_section()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        match self.current_view {
            ViewMode::Overview => {
                lines.push(Line::from("  m          - 切换出站模式"));
                if self.snapshot.http_api_available {
                    lines.push(Line::from("  i          - 切换 MITM"));
                    lines.push(Line::from("  c          - 切换流量捕获"));
                }
            }
            ViewMode::Policies => {
                lines.push(Line::from("  /          - 搜索"));
                lines.push(Line::from("  t          - 测试延迟"));
                lines.push(Line::from("  Enter      - 进入/选择策略"));
                lines.push(Line::from("  ESC        - 返回"));
            }
            ViewMode::Requests | ViewMode::ActiveConnections => {
                lines.push(Line::from("  /          - 搜索"));
                lines.push(Line::from("  g          - 切换分组模式"));
                if self.grouped_mode {
                    lines.push(Line::from("  h/l        - 切换应用"));
                }
            }
            ViewMode::Dns => {
                lines.push(Line::from("  /          - 搜索"));
                if self.snapshot.http_api_available {
                    lines.push(Line::from("  f          - 清空 DNS 缓存"));
                }
            }
        }

        lines.push(Line::from(""));

        // 导航快捷键
        lines.push(Line::from(vec![Span::styled(
            format!("【{}】", self.t.help_navigation_section()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from("  j/k 或 ↓/↑  - 上下移动"));
        if self.grouped_mode
            && matches!(
                self.current_view,
                ViewMode::Requests | ViewMode::ActiveConnections
            )
        {
            lines.push(Line::from("  h/l 或 ←/→  - 左右切换应用"));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.t.help_title())
                    .style(Style::default().bg(Color::Black).fg(Color::White)),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(ratatui::widgets::Clear, popup_area);
        f.render_widget(paragraph, popup_area);
    }

    /// 渲染终止连接确认对话框
    fn render_kill_confirm(&self, f: &mut Frame, area: Rect) {
        if let Some(connection_id) = self.show_kill_confirm {
            // 查找待终止的连接
            let connection = self
                .snapshot
                .active_connections
                .iter()
                .find(|c| c.id == connection_id);

            if let Some(conn) = connection {
                // 小弹窗：50% 宽度，30% 高度
                let popup_area = self.centered_rect(50, 30, area);

                let mut lines = Vec::new();

                // 标题行
                lines.push(Line::from(vec![Span::styled(
                    self.t
                        .confirm_kill_message(conn.url.as_deref().unwrap_or("Unknown")),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(""));

                // 连接详情
                if let Some(ref url) = conn.url {
                    lines.push(Line::from(vec![
                        Span::styled("目标: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(url),
                    ]));
                }

                if let Some(ref process) = conn.process_path {
                    lines.push(Line::from(vec![
                        Span::styled("进程: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(process),
                    ]));
                }

                let upload_kb = conn.out_bytes / 1024;
                let download_kb = conn.in_bytes / 1024;
                lines.push(Line::from(vec![
                    Span::styled("流量: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!("↑{}KB ↓{}KB", upload_kb, download_kb),
                        Style::default().fg(Color::Green),
                    ),
                ]));

                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    self.t.confirm_kill_hint(),
                    Style::default().fg(Color::DarkGray),
                )]));

                let paragraph = Paragraph::new(lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(self.t.confirm_kill_title())
                            .style(Style::default().bg(Color::Black).fg(Color::White)),
                    )
                    .wrap(ratatui::widgets::Wrap { trim: false });

                f.render_widget(ratatui::widgets::Clear, popup_area);
                f.render_widget(paragraph, popup_area);
            }
        }
    }

    /// 计算居中矩形区域
    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
