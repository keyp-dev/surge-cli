/// UI application state and event handling
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

// Import Notification type
use super::components::notifications::{Notification, NotificationLevel};

/// Message type for background test tasks
#[derive(Debug)]
enum TestMessage {
    /// Test started
    Started,
    /// Test completed
    Completed {
        group_name: String,
        results: Vec<PolicyDetail>,
    },
    /// Test failed
    Failed { error: String },
}

// Notification helper functions
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

/// Application state
pub struct App {
    /// Surge client
    client: SurgeClient,
    /// Current view
    current_view: ViewMode,
    /// Application snapshot
    snapshot: AppSnapshot,
    /// Whether to quit
    should_quit: bool,
    /// Refresh interval
    refresh_interval: Duration,
    /// Currently selected index (for list navigation)
    selected_index: usize,
    /// Selected policy index within a group (None = in group list; Some(idx) = inside group)
    policy_detail_index: Option<usize>,
    /// Name of the policy group being tested (used to display test status)
    testing_policy_group: Option<String>,
    /// Background test message receiver
    test_rx: mpsc::Receiver<TestMessage>,
    /// Background test message sender
    test_tx: mpsc::Sender<TestMessage>,
    /// Notification list (keep at most 50)
    notifications: Vec<Notification>,
    /// Whether to show the notification history popup
    show_notification_history: bool,
    /// Whether to show the DevTools panel
    show_devtools: bool,
    /// DevTools log entries
    devtools_logs: Vec<DevToolsLog>,
    /// Policy latency test result cache (key: policy name, value: test result)
    /// Cache is not cleared on refresh; only updated when a new test runs
    policy_test_cache: HashMap<String, PolicyDetail>,
    /// Translator instance (language determined at compile time)
    t: &'static dyn crate::i18n::Translate,
    /// Search mode flag
    search_mode: bool,
    /// Search query (for policy group list)
    search_query: String,
    /// Search query for policy detail view
    policy_detail_search: String,
    /// Grouped mode flag (only for Requests and ActiveConnections)
    grouped_mode: bool,
    /// Selected application index in grouped mode
    grouped_app_index: usize,
    /// Whether to show the help popup
    show_help: bool,
    /// Connection ID pending kill confirmation (shows confirm dialog when Some)
    show_kill_confirm: Option<u64>,
}

/// DevTools log entry
#[derive(Debug, Clone)]
struct DevToolsLog {
    timestamp: DateTime<Local>,
    level: LogLevel,
    message: String,
}

/// Log level
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
    /// Create a new application
    pub fn new(client: SurgeClient, refresh_interval_secs: u64) -> Self {
        // Create background test message channel (buffer size 1)
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

    /// Add a notification
    fn add_notification(&mut self, notification: Notification) {
        self.notifications.push(notification);
        // Keep at most 50 history entries
        if self.notifications.len() > 50 {
            self.notifications.remove(0);
        }
    }

    /// Add a DevTools log entry
    fn add_devtools_log(&mut self, level: LogLevel, message: String) {
        self.devtools_logs.push(DevToolsLog::new(level, message));
        // Keep at most 200 entries
        if self.devtools_logs.len() > 200 {
            self.devtools_logs.remove(0);
        }
    }

    /// Clean expired notifications (only status bar ones; history is kept)
    fn clean_expired_notifications(&mut self) {
        // No longer auto-cleaning; history is preserved
    }

    /// Run the application
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        // Initial load
        self.refresh().await;

        loop {
            // Clean expired notifications
            self.clean_expired_notifications();

            // Render UI
            terminal.draw(|f| self.render(f))?;

            // Process background test messages (non-blocking)
            let mut has_test_message = false;
            while let Ok(msg) = self.test_rx.try_recv() {
                self.handle_test_message(msg);
                has_test_message = true;
            }

            // If test messages were processed, redraw immediately (don't wait for user input)
            if has_test_message {
                terminal.draw(|f| self.render(f))?;
            }

            // Handle events (non-blocking with timeout)
            // Only refresh data on timeout (no keypress) to keep list stable during user interaction
            if event::poll(self.refresh_interval)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key).await;
                }
            } else {
                // Only refresh on timeout to keep list stable while user is interacting
                self.refresh().await;
            }

            // Check if we should quit
            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Refresh data
    async fn refresh(&mut self) {
        self.snapshot = self.client.get_snapshot().await;

        // Restore test results from cache (prevents loss after refresh)
        if !self.policy_test_cache.is_empty() {
            self.snapshot.policies = self.policy_test_cache.values().cloned().collect();
        }

        // Ensure selected index is valid
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

    /// Handle background test messages
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

                // Debug: collect all data to log first (avoid borrow conflicts)
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

                // Debug: log policy names from test results
                self.add_devtools_log(
                    LogLevel::Info,
                    format!("=== Test results policy names (first 5) ==="),
                );
                for (i, name_info) in test_result_names.iter().enumerate() {
                    self.add_devtools_log(LogLevel::Info, format!("  [{}] {}", i, name_info));
                }

                // Debug: log policy names in the group (for comparison)
                if !group_policy_names.is_empty() {
                    self.add_devtools_log(
                        LogLevel::Info,
                        format!("=== Policy names in group '{}' (first 5) ===", group_name),
                    );
                    for (i, name) in group_policy_names.iter().enumerate() {
                        self.add_devtools_log(LogLevel::Info, format!("  [{}] '{}'", i, name));
                    }
                }

                // Update test result cache (persists across refreshes)
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

                // Also update snapshot.policies for immediate display
                self.snapshot.policies = results.clone();

                // Update available policies list for the group
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

                // Debug: log name matching results
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

                self.testing_policy_group = None; // Clear test status
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

    /// Get the length of the current view's list (accounting for display limits and search)
    fn get_current_list_len(&self) -> usize {
        match self.current_view {
            ViewMode::Overview => 0,
            ViewMode::Policies => self.snapshot.policy_groups.len(),
            ViewMode::Dns => {
                // DNS view: return filtered DNS cache count
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
                    // Grouped mode: return filtered request count for selected app
                    self.get_grouped_request_count(&self.search_query)
                } else {
                    // Normal mode: return total request count (capped at 50, with search filter)
                    let requests = match self.current_view {
                        ViewMode::Requests => &self.snapshot.recent_requests,
                        ViewMode::ActiveConnections => &self.snapshot.active_connections,
                        _ => return 0,
                    };

                    // Apply search filter
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

    /// Get the number of applications in grouped mode
    fn get_grouped_app_count(&self) -> usize {
        use std::collections::HashSet;
        let requests = match self.current_view {
            ViewMode::Requests => &self.snapshot.recent_requests,
            ViewMode::ActiveConnections => &self.snapshot.active_connections,
            _ => return 0,
        };

        // Count unique application names
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

    /// Get request count for the selected app in grouped mode (with search filter)
    fn get_grouped_request_count(&self, search_query: &str) -> usize {
        use std::collections::HashMap;

        let requests = match self.current_view {
            ViewMode::Requests => &self.snapshot.recent_requests,
            ViewMode::ActiveConnections => &self.snapshot.active_connections,
            _ => return 0,
        };

        // Group by process_path (mirrors render_grouped_view logic)
        let mut app_groups: HashMap<String, Vec<&crate::domain::models::Request>> = HashMap::new();
        for req in requests {
            let app_name = req
                .process_path
                .as_ref()
                .map(|p| p.split('/').last().unwrap_or(p).to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            app_groups.entry(app_name).or_default().push(req);
        }

        // Sort app list (consistent with render_grouped_view)
        let mut apps: Vec<(String, usize)> = app_groups
            .iter()
            .map(|(name, reqs)| (name.clone(), reqs.len()))
            .collect();
        apps.sort_by(|a, b| match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => a.0.cmp(&b.0),
            other => other,
        });

        // Get requests for the app at grouped_app_index
        if self.grouped_app_index >= apps.len() {
            return 0;
        }

        let (selected_app_name, _) = &apps[self.grouped_app_index];
        let app_requests = app_groups.get(selected_app_name).unwrap();

        // Apply search filter
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

    /// Handle keyboard events
    async fn handle_key(&mut self, key: KeyEvent) {
        // Kill confirmation popup handling
        if let Some(connection_id) = self.show_kill_confirm {
            match key.code {
                KeyCode::Enter => {
                    // Execute kill connection
                    if let Err(e) = self.client.kill_connection(connection_id).await {
                        self.add_notification(Notification::error(
                            self.t.notification_kill_failed(&e.to_string()),
                        ));
                    } else {
                        self.add_notification(Notification::success(
                            self.t.notification_connection_killed().to_string(),
                        ));
                        // Refresh list
                        self.refresh().await;
                    }
                    self.show_kill_confirm = None;
                    return;
                }
                KeyCode::Esc => {
                    // Cancel
                    self.show_kill_confirm = None;
                    return;
                }
                _ => {
                    // Block other keys
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

            // Quit or go back
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
                    // Close popup first
                    self.show_notification_history = false;
                } else if self.show_devtools {
                    self.show_devtools = false;
                } else if self.current_view == ViewMode::Policies
                    && self.policy_detail_index.is_some()
                {
                    // Inside policy group: return to group list
                    self.policy_detail_index = None;
                } else {
                    // Otherwise: quit
                    self.should_quit = true;
                }
            }

            // N key: open notification history
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.show_notification_history = !self.show_notification_history;
            }

            // ` key: open DevTools
            KeyCode::Char('`') | KeyCode::Char('~') => {
                self.show_devtools = !self.show_devtools;
            }

            // ? key: open help
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
            }

            // View switching
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
                    // Get the selected connection
                    let connections = &self.snapshot.active_connections;
                    if !connections.is_empty() {
                        let selected_connection = if self.grouped_mode {
                            // Grouped mode: get the selected connection in the current app
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

                            // Sort app list
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

                                // Apply search filter
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
                            // Normal mode: directly get the selected connection
                            // Apply search filter
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

            // List navigation
            KeyCode::Up => {
                if self.current_view == ViewMode::Policies {
                    if let Some(policy_idx) = self.policy_detail_index {
                        // Inside policy group: navigate policy list
                        if policy_idx > 0 {
                            self.policy_detail_index = Some(policy_idx - 1);
                        }
                    } else {
                        // In group list: normal navigation
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                    }
                } else {
                    // Other views: normal navigation (request list)
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.current_view == ViewMode::Policies {
                    if let Some(policy_idx) = self.policy_detail_index {
                        // Inside policy group: navigate policy list
                        if self.selected_index < self.snapshot.policy_groups.len() {
                            let group = &self.snapshot.policy_groups[self.selected_index];
                            if policy_idx < group.policies.len() - 1 {
                                self.policy_detail_index = Some(policy_idx + 1);
                            }
                        }
                    } else {
                        // In group list: normal navigation
                        let max_index = self.get_current_list_len();
                        if max_index > 0 && self.selected_index < max_index - 1 {
                            self.selected_index += 1;
                        }
                    }
                } else {
                    // Other views: normal navigation (request list)
                    let max_index = self.get_current_list_len();
                    if max_index > 0 && self.selected_index < max_index - 1 {
                        self.selected_index += 1;
                    }
                }
            }

            // Left/right keys navigate app list (grouped mode only)
            KeyCode::Left | KeyCode::Char('h') => {
                if self.grouped_mode
                    && matches!(
                        self.current_view,
                        ViewMode::Requests | ViewMode::ActiveConnections
                    )
                {
                    if self.grouped_app_index > 0 {
                        self.grouped_app_index -= 1;
                        self.selected_index = 0; // Reset request index when switching apps
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
                        self.selected_index = 0; // Reset request index when switching apps
                    }
                }
            }

            // Enter key: enter policy group or switch policy
            KeyCode::Enter => {
                if self.current_view == ViewMode::Policies {
                    if let Some(policy_idx) = self.policy_detail_index {
                        // Inside policy group: switch to selected policy
                        if self.selected_index < self.snapshot.policy_groups.len() {
                            let group = &self.snapshot.policy_groups[self.selected_index];
                            if policy_idx < group.policies.len() {
                                let selected_policy = &group.policies[policy_idx];
                                // Call API to switch policy
                                let _ = self
                                    .client
                                    .select_policy_group(&group.name, &selected_policy.name)
                                    .await;
                                // Exit policy group detail view
                                self.policy_detail_index = None;
                                // Refresh data
                                self.refresh().await;
                            }
                        }
                    } else {
                        // In group list: enter the policy group
                        if self.selected_index < self.snapshot.policy_groups.len() {
                            let group = &self.snapshot.policy_groups[self.selected_index];
                            if !group.policies.is_empty() {
                                // Find the currently selected policy index as the initial selection
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

            // T key: test all policy latencies (async background task, non-blocking)
            KeyCode::Char('t') | KeyCode::Char('T') => {
                if self.current_view == ViewMode::Policies {
                    if self.selected_index < self.snapshot.policy_groups.len() {
                        let group_name = self.snapshot.policy_groups[self.selected_index]
                            .name
                            .clone();
                        let client = self.client.clone();
                        let tx = self.test_tx.clone();

                        // Start background test task
                        tokio::spawn(async move {
                            // Send test started message
                            let _ = tx.send(TestMessage::Started).await;

                            tracing::info!(
                                "Background test task started: testing policy group {}",
                                group_name
                            );

                            // Execute test in background (non-blocking)
                            match client.test_all_policies_with_latency().await {
                                Ok(policy_details) => {
                                    // Send test completed message
                                    let _ = tx
                                        .send(TestMessage::Completed {
                                            group_name,
                                            results: policy_details,
                                        })
                                        .await;
                                }
                                Err(e) => {
                                    // Send test failed message
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

            // F key: flush DNS cache (DNS view only)
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

            // Toggle outbound mode
            KeyCode::Char('m') | KeyCode::Char('M') => {
                use crate::domain::models::OutboundMode;
                if let Some(ref current_mode) = self.snapshot.outbound_mode {
                    // Cycle: Direct → Proxy → Rule → Direct
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
                        // Refresh to get real state
                        self.refresh().await;
                    }
                }
            }

            // Toggle MITM status (Overview view only)
            KeyCode::Char('i') | KeyCode::Char('I') => {
                if self.current_view == ViewMode::Overview && self.snapshot.http_api_available {
                    if let Some(current_status) = self.snapshot.mitm_enabled {
                        let new_status = !current_status;
                        match self.client.set_mitm_status(new_status).await {
                            Ok(_) => {
                                // Show notification
                                let msg = if new_status {
                                    self.t.notification_mitm_enabled()
                                } else {
                                    self.t.notification_mitm_disabled()
                                };
                                self.add_notification(Notification::success(msg.to_string()));
                                // Refresh to get real state
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

            // Toggle Capture status (Overview view only)
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if self.current_view == ViewMode::Overview && self.snapshot.http_api_available {
                    if let Some(current_status) = self.snapshot.capture_enabled {
                        let new_status = !current_status;
                        match self.client.set_capture_status(new_status).await {
                            Ok(_) => {
                                // Show notification
                                let msg = if new_status {
                                    self.t.notification_capture_enabled()
                                } else {
                                    self.t.notification_capture_disabled()
                                };
                                self.add_notification(Notification::success(msg.to_string()));
                                // Refresh to get real state
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

            // Alert actions
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Start Surge
                if let Some(alert) = self.snapshot.alerts.first() {
                    if matches!(alert.action, AlertAction::StartSurge) {
                        let _ = self.client.start_surge().await;
                        self.refresh().await;
                    }
                }
            }

            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Handle Alert ReloadConfig action first
                if let Some(alert) = self.snapshot.alerts.first() {
                    if matches!(alert.action, AlertAction::ReloadConfig) {
                        let _ = self.client.reload_config().await;
                        self.refresh().await;
                        return;
                    }
                }
                // Otherwise treat as manual refresh
                self.refresh().await;
            }

            _ => {}
        }
    }

    /// Render UI
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

    /// Render tabs
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

                // btop style: [number] title
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

    /// Render content area
    fn render_content(&self, f: &mut Frame, area: Rect) {
        // If there are alerts, split the area
        if !self.snapshot.alerts.is_empty() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4), // Alerts
                    Constraint::Min(0),    // Main content
                ])
                .split(area);

            // Render alerts
            super::components::alerts::render(f, chunks[0], &self.snapshot.alerts, self.t);

            // Render main content
            self.render_view_content(f, chunks[1]);
        } else {
            // No alerts: render content directly
            self.render_view_content(f, area);
        }
    }

    /// Render view content
    fn render_view_content(&self, f: &mut Frame, area: Rect) {
        // Render main content
        self.render_main_view(f, area);

        // Render popups (overlay on top of main content)
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

    /// Render main view content
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

    /// Render status bar
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

        // Build shortcut hints (simplified)
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

        // Show only the most basic shortcut hints
        spans.push(Span::raw(self.t.key_quit()));
        spans.push(Span::raw("  "));
        spans.push(Span::raw(self.t.key_help()));

        // Alert action shortcuts (high priority)
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

        // Status bar left side
        let left_line = Line::from(spans);

        // Status bar right side: show latest notification (compact mode)
        let right_spans = if let Some(latest) = self.notifications.last() {
            // Show only the latest notification with time (HH:MM:SS)
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
                    // Show time within 60 seconds
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

        // Split status bar: left shortcuts | right notification
        let status_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),     // Left: shortcuts
                Constraint::Length(50), // Right: notification area
            ])
            .split(area);

        f.render_widget(Paragraph::new(left_line), status_chunks[0]);
        f.render_widget(
            Paragraph::new(right_line).alignment(Alignment::Right),
            status_chunks[1],
        );
    }

    /// Render notification history popup
    fn render_notification_history(&self, f: &mut Frame, area: Rect) {
        // Centered popup: 80% width, 70% height
        let popup_area = self.centered_rect(80, 70, area);

        // Build notification list
        let mut lines = Vec::new();
        for (i, notification) in self.notifications.iter().rev().enumerate() {
            if i > 0 {
                lines.push(Line::from(""));
            }

            // Format as absolute time: YYYY-MM-DD HH:MM:SS
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

    /// Render DevTools panel
    fn render_devtools(&self, f: &mut Frame, area: Rect) {
        // Bottom 70% height
        let devtools_area = Rect {
            x: area.x,
            y: area.y + (area.height * 30 / 100),
            width: area.width,
            height: area.height * 70 / 100,
        };

        // Build log list
        let mut lines = Vec::new();
        for log in self.devtools_logs.iter().rev().take(100) {
            // Format as absolute time: YYYY-MM-DD HH:MM:SS
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

    /// Render help popup
    fn render_help(&self, f: &mut Frame, area: Rect) {
        // Centered popup: 70% width, 60% height
        let popup_area = self.centered_rect(70, 60, area);

        // Build help content
        let mut lines = Vec::new();

        // Global shortcuts section
        lines.push(Line::from(vec![Span::styled(
            format!("[{}]", self.t.help_global_section()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(self.t.help_shortcut_quit()));
        lines.push(Line::from(self.t.help_shortcut_refresh()));
        lines.push(Line::from(self.t.help_shortcut_switch_view()));
        lines.push(Line::from(self.t.help_shortcut_toggle_outbound()));
        lines.push(Line::from(self.t.help_shortcut_notification_history()));
        lines.push(Line::from(self.t.help_shortcut_devtools()));
        lines.push(Line::from(self.t.help_shortcut_help()));
        lines.push(Line::from(""));

        // Current view shortcuts
        lines.push(Line::from(vec![Span::styled(
            format!("[{}]", self.t.help_view_section()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        match self.current_view {
            ViewMode::Overview => {
                lines.push(Line::from(self.t.help_shortcut_toggle_outbound()));
                if self.snapshot.http_api_available {
                    lines.push(Line::from(self.t.help_shortcut_toggle_mitm()));
                    lines.push(Line::from(self.t.help_shortcut_toggle_capture()));
                }
            }
            ViewMode::Policies => {
                lines.push(Line::from(self.t.help_shortcut_search()));
                lines.push(Line::from(self.t.help_shortcut_test_latency()));
                lines.push(Line::from(self.t.help_shortcut_enter_select_policy()));
                lines.push(Line::from(self.t.help_shortcut_esc_back()));
            }
            ViewMode::Requests | ViewMode::ActiveConnections => {
                lines.push(Line::from(self.t.help_shortcut_search()));
                lines.push(Line::from(self.t.help_shortcut_toggle_group()));
                if self.grouped_mode {
                    lines.push(Line::from(self.t.help_shortcut_switch_app()));
                }
            }
            ViewMode::Dns => {
                lines.push(Line::from(self.t.help_shortcut_search()));
                if self.snapshot.http_api_available {
                    lines.push(Line::from(self.t.help_shortcut_flush_dns()));
                }
            }
        }

        lines.push(Line::from(""));

        // Navigation shortcuts
        lines.push(Line::from(vec![Span::styled(
            format!("[{}]", self.t.help_navigation_section()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(self.t.help_nav_up_down()));
        if self.grouped_mode
            && matches!(
                self.current_view,
                ViewMode::Requests | ViewMode::ActiveConnections
            )
        {
            lines.push(Line::from(self.t.help_nav_left_right()));
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

    /// Render kill connection confirmation dialog
    fn render_kill_confirm(&self, f: &mut Frame, area: Rect) {
        if let Some(connection_id) = self.show_kill_confirm {
            // Find the connection to be killed
            let connection = self
                .snapshot
                .active_connections
                .iter()
                .find(|c| c.id == connection_id);

            if let Some(conn) = connection {
                // Small popup: 50% width, 30% height
                let popup_area = self.centered_rect(50, 30, area);

                let mut lines = Vec::new();

                // Title line
                lines.push(Line::from(vec![Span::styled(
                    self.t
                        .confirm_kill_message(conn.url.as_deref().unwrap_or("Unknown")),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(""));

                // Connection details
                if let Some(ref url) = conn.url {
                    lines.push(Line::from(vec![
                        Span::styled(self.t.confirm_kill_label_target(), Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(url),
                    ]));
                }

                if let Some(ref process) = conn.process_path {
                    lines.push(Line::from(vec![
                        Span::styled(self.t.confirm_kill_label_process(), Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(process),
                    ]));
                }

                let upload_kb = conn.out_bytes / 1024;
                let download_kb = conn.in_bytes / 1024;
                lines.push(Line::from(vec![
                    Span::styled(self.t.confirm_kill_label_traffic(), Style::default().add_modifier(Modifier::BOLD)),
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

    /// Calculate a centered rectangular area
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
