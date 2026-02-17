/// Requests component - request and connection list
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
        // Grouped mode: display requests grouped by application (supports searching within app)
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
        // Normal mode: show all requests
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

        // Split area: request list | detail panel
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

        // Connections view: show kill connection shortcut
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
        .take(50) // Limit displayed count
        .map(|req| {
            // Truncate URL to 35 characters
            let url = req
                .url
                .as_ref()
                .map(|u| truncate_text(u, 35))
                .unwrap_or_else(|| "Unknown".to_string());

            // Truncate policy name to 25 characters
            let policy = req
                .policy_name
                .as_ref()
                .map(|p| truncate_text(p, 25))
                .unwrap_or_else(|| "-".to_string());

            let upload_kb = req.out_bytes / 1024;
            let download_kb = req.in_bytes / 1024;

            // Status indicator
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

/// Truncate text to a maximum character count
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_len - 2).collect();
        format!("{}..", truncated)
    }
}

/// Compute display width of a string (simplified: non-ASCII chars count as width 2)
fn display_width(text: &str) -> usize {
    text.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

/// Pad a string to a fixed display width (handles mixed CJK/ASCII)
fn pad_to_width(text: &str, width: usize) -> String {
    let current_width = display_width(text);
    if current_width >= width {
        // Already at or over target width, return as-is
        text.to_string()
    } else {
        // Pad with spaces to reach target width
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
    // Get the selected request
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

    // Status header
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

    // Method and status
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

    // Remote host
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

    // Rule
    if let Some(ref rule) = request.rule {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", t.request_label_rule()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(rule, Style::default().fg(Color::Magenta)),
        ]));
    }

    // Policy
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

    // Traffic statistics
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

    // Process path
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

    // Timestamp
    if let Some(timestamp) = request.start_date {
        lines.push(Line::from(""));
        // Convert Unix timestamp to human-readable format
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

    // HTTP Body indicator
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

    // Notes (connection log)
    if !request.notes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            t.request_label_notes(),
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]));

        // Show only first 10 notes to avoid overly long panel
        for (i, note) in request.notes.iter().take(10).enumerate() {
            // Parse note and highlight key information
            let styled_note = format_note(note, t);
            lines.push(Line::from(styled_note));

            // Add blank line every 3 entries for readability
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

/// Format a single note entry with highlighted key information
fn format_note(note: &str, _t: &'static dyn Translate) -> Vec<Span<'static>> {
    // Parse timestamp and tag
    let parts: Vec<&str> = note.splitn(2, ' ').collect();
    if parts.len() < 2 {
        return vec![Span::raw(note.to_string())];
    }

    let timestamp = parts[0];
    let rest = parts[1];

    // Extract tag (e.g. [Connection], [TLS], [Rule])
    let mut spans = vec![];

    // Add timestamp in gray
    spans.push(Span::styled(
        format!("{} ", timestamp),
        Style::default().fg(Color::DarkGray),
    ));

    // Parse tag and content
    if let Some(tag_start) = rest.find('[') {
        if let Some(tag_end) = rest.find(']') {
            if tag_end > tag_start {
                // Content before tag
                if tag_start > 0 {
                    spans.push(Span::raw(rest[..tag_start].to_string()));
                }

                // Tag content (highlighted)
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

                // Content after tag
                if tag_end + 1 < rest.len() {
                    spans.push(Span::raw(rest[tag_end + 1..].to_string()));
                }

                return spans;
            }
        }
    }

    // No tag found: render content as-is
    spans.push(Span::raw(rest.to_string()));
    spans
}

/// Render grouped view (requests grouped by application)
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

    // Group by process_path
    let mut app_groups: HashMap<String, Vec<&Request>> = HashMap::new();
    for req in requests {
        let app_name = req
            .process_path
            .as_ref()
            .map(|p| {
                // Extract app name (strip path prefix)
                p.split('/').last().unwrap_or(p).to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());
        app_groups.entry(app_name).or_default().push(req);
    }

    // Sort app list (descending by request count, then alphabetically by name)
    let mut apps: Vec<(String, usize)> = app_groups
        .iter()
        .map(|(name, reqs)| (name.clone(), reqs.len()))
        .collect();
    apps.sort_by(|a, b| {
        // Descending by count
        match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => {
                // Ascending by name when counts are equal
                a.0.cmp(&b.0)
            }
            other => other,
        }
    });

    // Three-column layout: app list | request list | detail panel
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // App list
            Constraint::Percentage(45), // Request list
            Constraint::Percentage(30), // Detail panel
        ])
        .split(area);

    // Render app list
    render_app_list(f, chunks[0], &apps, app_selected, t);

    // Get selected app and its requests
    if app_selected < apps.len() {
        let (selected_app_name, _) = &apps[app_selected];
        let app_requests: Vec<_> = app_groups
            .get(selected_app_name)
            .unwrap()
            .iter()
            .copied()
            .collect();

        // Render request list for this app (filtering happens internally)
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

        // Render request detail (using the same filtered requests as the list)
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
        // No app selected
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

/// Render application list
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

/// Render request list for an application
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
    // Title shows search state
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

        // Connections view: show kill connection shortcut
        if is_connection_view {
            spans.push(Span::raw(" ["));
            spans.push(Span::styled("k", Style::default().fg(Color::Yellow)));
            spans.push(Span::raw("]"));
            spans.push(Span::raw(t.action_kill()));
        }

        spans.push(Span::raw(" "));
        Line::from(spans)
    };

    // Filter requests by search query
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
            // Truncate URL to 30 characters
            let url = req
                .url
                .as_ref()
                .map(|u| truncate_text(u, 30))
                .unwrap_or_else(|| "Unknown".to_string());

            let upload_kb = req.out_bytes / 1024;
            let download_kb = req.in_bytes / 1024;

            // Status indicator
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
