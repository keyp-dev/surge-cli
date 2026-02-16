/// Requests 组件 - 请求和连接列表
use crate::domain::models::Request;
use crate::i18n::Translate;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn render(
    f: &mut Frame,
    area: Rect,
    requests: &[Request],
    selected: usize,
    search_query: &str,
    search_mode: bool,
    grouped_mode: bool,
    grouped_app_index: usize,
    is_connection_view: bool,
    t: &'static dyn Translate,
) {
    if grouped_mode {
        // 分组模式：按应用分组显示（支持搜索当前应用的请求）
        render_grouped_view(
            f,
            area,
            requests,
            selected,
            grouped_app_index,
            search_query,
            search_mode,
            is_connection_view,
            t,
        );
    } else {
        // 普通模式：显示所有请求
        // Filter requests by search query
        let filtered_requests: Vec<_> = if search_query.is_empty() {
            requests.iter().collect()
        } else {
            let query_lower = search_query.to_lowercase();
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
                .collect()
        };

        // 分割区域：请求列表 | 详细信息
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        render_request_list(
            f,
            chunks[0],
            &filtered_requests,
            selected,
            search_query,
            search_mode,
            is_connection_view,
            t,
        );
        render_request_detail(f, chunks[1], &filtered_requests, selected, t);
    }
}

fn render_request_list(
    f: &mut Frame,
    area: Rect,
    requests: &[&Request],
    selected: usize,
    search_query: &str,
    search_mode: bool,
    is_connection_view: bool,
    t: &'static dyn Translate,
) {
    let title = if search_mode {
        Line::from(vec![
            Span::raw(" "),
            Span::raw(t.request_list_title()),
            Span::raw(" [Search: "),
            Span::raw(search_query),
            Span::raw("█] "),
        ])
    } else if !search_query.is_empty() {
        Line::from(vec![
            Span::raw(" "),
            Span::raw(t.request_list_title()),
            Span::raw(" [Search: "),
            Span::raw(search_query),
            Span::raw("] "),
        ])
    } else {
        let mut spans = vec![
            Span::raw(" "),
            Span::raw(t.request_list_title()),
            Span::raw(" ["),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_select()),
            Span::raw(" ["),
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_search()),
            Span::raw(" ["),
            Span::styled("g", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_group()),
        ];

        // Connections 视图显示终止连接快捷键
        if is_connection_view {
            spans.push(Span::raw(" ["));
            spans.push(Span::styled("k", Style::default().fg(Color::Yellow)));
            spans.push(Span::raw("]"));
            spans.push(Span::raw(t.action_kill()));
        }

        spans.push(Span::raw(" "));
        Line::from(spans)
    };

    if requests.is_empty() {
        let empty = Paragraph::new(t.request_no_requests())
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = requests
        .iter()
        .take(50) // 限制显示数量
        .map(|req| {
            // URL 截断到 35 字符
            let url = req
                .url
                .as_ref()
                .map(|u| truncate_text(u, 35))
                .unwrap_or_else(|| "Unknown".to_string());

            // 策略名截断到 25 字符
            let policy = req
                .policy_name
                .as_ref()
                .map(|p| truncate_text(p, 25))
                .unwrap_or_else(|| "-".to_string());

            let upload_kb = req.out_bytes / 1024;
            let download_kb = req.in_bytes / 1024;

            // 状态指示器
            let status_char = if req.completed {
                "✓"
            } else if req.failed {
                "✗"
            } else {
                "○"
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", status_char),
                    Style::default().fg(if req.completed {
                        Color::Green
                    } else if req.failed {
                        Color::Red
                    } else {
                        Color::Yellow
                    }),
                ),
                Span::styled(
                    pad_to_width(&url, 40),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    pad_to_width(&policy, 25),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("↑{:>4}K ↓{:>4}K", upload_kb, download_kb),
                    Style::default().fg(Color::Green),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !requests.is_empty() {
        state.select(Some(selected.min(requests.len().min(50) - 1)));
    }

    f.render_stateful_widget(list, area, &mut state);
}

/// 截断文本
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_len - 2).collect();
        format!("{}..", truncated)
    }
}

/// 计算字符串的显示宽度（简化版：非ASCII字符算2宽度）
fn display_width(text: &str) -> usize {
    text.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

/// 填充字符串到固定显示宽度（处理中英文混合）
fn pad_to_width(text: &str, width: usize) -> String {
    let current_width = display_width(text);
    if current_width >= width {
        // 已经超过宽度，返回原文本
        text.to_string()
    } else {
        // 填充空格到目标宽度
        format!("{}{}", text, " ".repeat(width - current_width))
    }
}

fn render_request_detail(
    f: &mut Frame,
    area: Rect,
    requests: &[&Request],
    selected: usize,
    t: &'static dyn Translate,
) {
    // 获取选中的请求
    let request = if selected < requests.len().min(50) {
        requests[selected]
    } else {
        let empty = Paragraph::new(t.request_no_selection()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.request_detail_title()),
        );
        f.render_widget(empty, area);
        return;
    };

    let mut lines = vec![];

    // 状态标题
    let status_symbol = if request.completed {
        (t.request_status_completed(), Color::Green)
    } else if request.failed {
        (t.request_status_failed(), Color::Red)
    } else {
        (t.request_status_in_progress(), Color::Yellow)
    };
    lines.push(Line::from(vec![Span::styled(
        status_symbol.0,
        Style::default()
            .fg(status_symbol.1)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    // URL
    if let Some(ref url) = request.url {
        lines.push(Line::from(vec![Span::styled(
            "URL: ",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            url,
            Style::default().fg(Color::Cyan),
        )]));
        lines.push(Line::from(""));
    }

    // 方法和状态
    let method_status = format!(
        "{} → {}",
        request.method.as_ref().unwrap_or(&"GET".to_string()),
        request.status.as_ref().unwrap_or(&"-".to_string())
    );
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", t.request_label_request()),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(method_status, Style::default().fg(Color::Yellow)),
    ]));

    // 远程主机
    if let Some(ref host) = request.remote_host {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", t.request_label_host()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(host),
        ]));
    }

    lines.push(Line::from(""));

    // 规则
    if let Some(ref rule) = request.rule {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", t.request_label_rule()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(rule, Style::default().fg(Color::Magenta)),
        ]));
    }

    // 策略
    if let Some(ref policy) = request.policy_name {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", t.request_label_policy()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(policy, Style::default().fg(Color::Yellow)),
        ]));
    }

    lines.push(Line::from(""));

    // 流量统计
    lines.push(Line::from(vec![Span::styled(
        t.request_label_traffic(),
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )]));

    let upload_kb = request.out_bytes / 1024;
    let download_kb = request.in_bytes / 1024;
    lines.push(Line::from(vec![
        Span::raw(format!("  {}: ", t.request_label_upload())),
        Span::styled(
            format!("{} KB", upload_kb),
            Style::default().fg(Color::Green),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw(format!("  {}: ", t.request_label_download())),
        Span::styled(
            format!("{} KB", download_kb),
            Style::default().fg(Color::Green),
        ),
    ]));

    // 进程路径
    if let Some(ref process) = request.process_path {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!("{}: ", t.request_label_process()),
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            process,
            Style::default().fg(Color::DarkGray),
        )]));
    }

    // 时间戳
    if let Some(timestamp) = request.start_date {
        lines.push(Line::from(""));
        // 将 Unix 时间戳转换为可读格式
        use std::time::UNIX_EPOCH;
        let duration = std::time::Duration::from_secs_f64(timestamp);
        if let Some(time) = UNIX_EPOCH.checked_add(duration) {
            if let Ok(elapsed) = time.elapsed() {
                let secs = elapsed.as_secs();
                let time_str = if secs < 60 {
                    t.request_time_seconds_ago(secs)
                } else if secs < 3600 {
                    t.request_time_minutes_ago(secs / 60)
                } else {
                    t.request_time_hours_ago(secs / 3600)
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}: ", t.request_label_time()),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(time_str),
                ]));
            }
        }
    }

    // HTTP Body 标记
    if request.stream_has_request_body || request.stream_has_response_body {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            t.request_label_http_body(),
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]));

        if request.stream_has_request_body {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("✓", Style::default().fg(Color::Green)),
                Span::raw(format!(" {}", t.request_has_request_body())),
            ]));
        }
        if request.stream_has_response_body {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("✓", Style::default().fg(Color::Green)),
                Span::raw(format!(" {}", t.request_has_response_body())),
            ]));
        }
    }

    // Notes（连接日志）
    if !request.notes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            t.request_label_notes(),
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]));

        // 只显示前 10 条 notes，避免面板过长
        for (i, note) in request.notes.iter().take(10).enumerate() {
            // 解析 note 并高亮关键信息
            let styled_note = format_note(note, t);
            lines.push(Line::from(styled_note));

            // 每 3 条添加空行，增强可读性
            if i % 3 == 2 && i < request.notes.len().min(10) - 1 {
                lines.push(Line::from(""));
            }
        }

        if request.notes.len() > 10 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                format!(
                    "  ... {} {}",
                    t.request_notes_more(request.notes.len() - 10),
                    ""
                ),
                Style::default().fg(Color::DarkGray),
            )]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.request_detail_title()),
        )
        .wrap(Wrap { trim: true })
        .scroll((0, 0));

    f.render_widget(paragraph, area);
}

/// 格式化单条 note，高亮关键信息
fn format_note(note: &str, _t: &'static dyn Translate) -> Vec<Span<'static>> {
    // 解析时间戳和标签
    let parts: Vec<&str> = note.splitn(2, ' ').collect();
    if parts.len() < 2 {
        return vec![Span::raw(note.to_string())];
    }

    let timestamp = parts[0];
    let rest = parts[1];

    // 提取标签（如 [Connection], [TLS], [Rule] 等）
    let mut spans = vec![];

    // 添加时间戳（灰色）
    spans.push(Span::styled(
        format!("{} ", timestamp),
        Style::default().fg(Color::DarkGray),
    ));

    // 解析标签和内容
    if let Some(tag_start) = rest.find('[') {
        if let Some(tag_end) = rest.find(']') {
            if tag_end > tag_start {
                // 标签前的内容
                if tag_start > 0 {
                    spans.push(Span::raw(rest[..tag_start].to_string()));
                }

                // 标签内容（高亮）
                let tag = &rest[tag_start..=tag_end];
                let tag_color = match tag {
                    "[Connection]" => Color::Cyan,
                    "[TLS]" => Color::Green,
                    "[DNS]" => Color::Magenta,
                    "[Rule]" => Color::Yellow,
                    "[Socket]" => Color::Blue,
                    "[HTTP]" => Color::LightGreen,
                    "[Policy]" => Color::LightYellow,
                    _ => Color::White,
                };
                spans.push(Span::styled(
                    tag.to_string(),
                    Style::default().fg(tag_color).add_modifier(Modifier::BOLD),
                ));

                // 标签后的内容
                if tag_end + 1 < rest.len() {
                    spans.push(Span::raw(rest[tag_end + 1..].to_string()));
                }

                return spans;
            }
        }
    }

    // 如果没有标签，直接显示内容
    spans.push(Span::raw(rest.to_string()));
    spans
}

/// 渲染分组视图（按应用分组）
fn render_grouped_view(
    f: &mut Frame,
    area: Rect,
    requests: &[Request],
    request_selected: usize,
    app_selected: usize,
    search_query: &str,
    search_mode: bool,
    is_connection_view: bool,
    t: &'static dyn Translate,
) {
    use std::collections::HashMap;

    // 按 process_path 分组
    let mut app_groups: HashMap<String, Vec<&Request>> = HashMap::new();
    for req in requests {
        let app_name = req
            .process_path
            .as_ref()
            .map(|p| {
                // 提取应用名称（去掉路径）
                p.split('/').last().unwrap_or(p).to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());
        app_groups.entry(app_name).or_default().push(req);
    }

    // 排序应用列表（按请求数量降序，数量相同时按名称字母序）
    let mut apps: Vec<(String, usize)> = app_groups
        .iter()
        .map(|(name, reqs)| (name.clone(), reqs.len()))
        .collect();
    apps.sort_by(|a, b| {
        // 先按数量降序
        match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => {
                // 数量相同时按名称升序（字母序）
                a.0.cmp(&b.0)
            }
            other => other,
        }
    });

    // 三列布局：应用列表 | 请求列表 | 详细信息
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // 应用列表
            Constraint::Percentage(45), // 请求列表
            Constraint::Percentage(30), // 详细信息
        ])
        .split(area);

    // 渲染应用列表
    render_app_list(f, chunks[0], &apps, app_selected, t);

    // 获取选中的应用及其请求
    if app_selected < apps.len() {
        let (selected_app_name, _) = &apps[app_selected];
        let app_requests: Vec<_> = app_groups
            .get(selected_app_name)
            .unwrap()
            .iter()
            .copied()
            .collect();

        // 渲染该应用的请求列表（支持搜索，会在内部过滤）
        render_app_request_list(
            f,
            chunks[1],
            &app_requests,
            request_selected,
            selected_app_name,
            search_query,
            search_mode,
            is_connection_view,
            t,
        );

        // 渲染请求详情（使用过滤后的请求）
        // 需要在这里也做同样的过滤，保持和列表一致
        let filtered_app_requests: Vec<_> = if search_query.is_empty() {
            app_requests
        } else {
            let query_lower = search_query.to_lowercase();
            app_requests
                .into_iter()
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
                .collect()
        };

        render_request_detail(f, chunks[2], &filtered_app_requests, request_selected, t);
    } else {
        // 没有选中应用
        let empty = Paragraph::new(t.request_no_app_selected()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.request_list_title()),
        );
        f.render_widget(empty, chunks[1]);

        let empty_detail = Paragraph::new(t.request_no_selection()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.request_detail_title()),
        );
        f.render_widget(empty_detail, chunks[2]);
    }
}

/// 渲染应用列表
fn render_app_list(
    f: &mut Frame,
    area: Rect,
    apps: &[(String, usize)],
    selected: usize,
    t: &'static dyn Translate,
) {
    let title = Line::from(vec![
        Span::raw(" "),
        Span::raw(t.request_app_list_title()),
        Span::raw(" ["),
        Span::styled("h/l", Style::default().fg(Color::Yellow)),
        Span::raw("]"),
        Span::raw(t.action_toggle()),
        Span::raw(" ["),
        Span::styled("g", Style::default().fg(Color::Yellow)),
        Span::raw("]"),
        Span::raw(t.action_mode()),
        Span::raw(" "),
    ]);

    if apps.is_empty() {
        let empty = Paragraph::new("No applications")
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = apps
        .iter()
        .map(|(app_name, count)| {
            let line = Line::from(vec![
                Span::styled(
                    truncate_text(app_name, 20),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(format!("({})", count), Style::default().fg(Color::DarkGray)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !apps.is_empty() {
        state.select(Some(selected.min(apps.len() - 1)));
    }

    f.render_stateful_widget(list, area, &mut state);
}

/// 渲染应用的请求列表
fn render_app_request_list(
    f: &mut Frame,
    area: Rect,
    requests: &[&Request],
    selected: usize,
    app_name: &str,
    search_query: &str,
    search_mode: bool,
    is_connection_view: bool,
    t: &'static dyn Translate,
) {
    // 标题显示搜索状态
    let title = if search_mode {
        Line::from(vec![
            Span::raw(" "),
            Span::raw(t.request_list_title()),
            Span::raw(": "),
            Span::styled(
                truncate_text(app_name, 15),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" [Search: "),
            Span::raw(search_query),
            Span::raw("█] "),
        ])
    } else if !search_query.is_empty() {
        Line::from(vec![
            Span::raw(" "),
            Span::raw(t.request_list_title()),
            Span::raw(": "),
            Span::styled(
                truncate_text(app_name, 15),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" [Search: "),
            Span::raw(search_query),
            Span::raw("] "),
        ])
    } else {
        let mut spans = vec![
            Span::raw(" "),
            Span::raw(t.request_list_title()),
            Span::raw(": "),
            Span::styled(
                truncate_text(app_name, 15),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ["),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_select()),
            Span::raw(" ["),
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_search()),
        ];

        // Connections 视图显示终止连接快捷键
        if is_connection_view {
            spans.push(Span::raw(" ["));
            spans.push(Span::styled("k", Style::default().fg(Color::Yellow)));
            spans.push(Span::raw("]"));
            spans.push(Span::raw(t.action_kill()));
        }

        spans.push(Span::raw(" "));
        Line::from(spans)
    };

    // 根据搜索查询过滤请求
    let filtered_requests: Vec<_> = if search_query.is_empty() {
        requests.iter().copied().collect()
    } else {
        let query_lower = search_query.to_lowercase();
        requests
            .iter()
            .copied()
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
            .collect()
    };

    if filtered_requests.is_empty() {
        let empty = Paragraph::new(t.request_no_requests())
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = filtered_requests
        .iter()
        .take(50)
        .map(|req| {
            // URL 截断到 30 字符
            let url = req
                .url
                .as_ref()
                .map(|u| truncate_text(u, 30))
                .unwrap_or_else(|| "Unknown".to_string());

            let upload_kb = req.out_bytes / 1024;
            let download_kb = req.in_bytes / 1024;

            // 状态指示器
            let status_char = if req.completed {
                "✓"
            } else if req.failed {
                "✗"
            } else {
                "○"
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", status_char),
                    Style::default().fg(if req.completed {
                        Color::Green
                    } else if req.failed {
                        Color::Red
                    } else {
                        Color::Yellow
                    }),
                ),
                Span::styled(
                    pad_to_width(&url, 35),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("↑{:>3}K ↓{:>3}K", upload_kb, download_kb),
                    Style::default().fg(Color::Green),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !filtered_requests.is_empty() {
        state.select(Some(selected.min(filtered_requests.len().min(50) - 1)));
    }

    f.render_stateful_widget(list, area, &mut state);
}
