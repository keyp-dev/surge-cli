/// Policies component - displays policies and policy groups
use crate::domain::entities::AppSnapshot;
use crate::i18n::Translate;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::HashSet;
use unicode_width::UnicodeWidthStr;

pub fn render(
    f: &mut Frame,
    area: Rect,
    snapshot: &AppSnapshot,
    selected: usize,
    policy_detail_index: Option<usize>,
    testing_group: Option<&str>,
    group_search_query: &str,
    policy_search_query: &str,
    search_mode: bool,
    t: &'static dyn Translate,
) {
    // Split area: policy groups | policy list
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Group list uses group_search_query, policy details use policy_search_query
    render_policy_groups(
        f,
        chunks[0],
        snapshot,
        selected,
        policy_detail_index.is_some(),
        testing_group,
        group_search_query,
        search_mode && policy_detail_index.is_none(),
        t,
    );
    render_policy_group_policies(
        f,
        chunks[1],
        snapshot,
        selected,
        policy_detail_index,
        group_search_query,
        policy_search_query,
        search_mode && policy_detail_index.is_some(),
        t,
    );
}

/// Recursively find the final real policy selected in a policy group (not another group)
///
/// Example: Proxy → US_Servers → us-bwg-la-dc1-vmess
/// Returns: Some("us-bwg-la-dc1-vmess")
fn resolve_final_policy(
    snapshot: &AppSnapshot,
    policy_name: &str,
    visited: &mut HashSet<String>,
) -> Option<String> {
    // Prevent circular references
    if visited.contains(policy_name) || visited.len() > 10 {
        return None;
    }
    visited.insert(policy_name.to_string());

    // Check if this is a policy group
    if let Some(group) = snapshot
        .policy_groups
        .iter()
        .find(|g| g.name == policy_name)
    {
        // It is a group: recursively find its selected policy
        if let Some(selected) = &group.selected {
            return resolve_final_policy(snapshot, selected, visited);
        } else {
            // Group has no selected policy
            return None;
        }
    }

    // Not a group: this is a real policy
    Some(policy_name.to_string())
}

fn render_policy_groups(
    f: &mut Frame,
    area: Rect,
    snapshot: &AppSnapshot,
    selected: usize,
    in_detail_mode: bool,
    testing_group: Option<&str>,
    search_query: &str,
    search_mode: bool,
    t: &'static dyn Translate,
) {
    // Filter policy groups by search query
    let filtered_groups: Vec<_> = if search_query.is_empty() {
        snapshot.policy_groups.iter().collect()
    } else {
        let query_lower = search_query.to_lowercase();
        snapshot
            .policy_groups
            .iter()
            .filter(|g| {
                g.name.to_lowercase().contains(&query_lower)
                    || g.selected
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .collect()
    };

    if filtered_groups.is_empty() {
        let empty = Paragraph::new(t.policy_no_groups()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.policy_group_title()),
        );
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = filtered_groups
        .iter()
        .map(|group| {
            let selected_text = group
                .selected
                .as_ref()
                .map(|s| format!(" → {}", s))
                .unwrap_or_default();

            // Check if this group is currently being tested
            let is_testing = testing_group.map(|tg| tg == group.name).unwrap_or(false);

            let mut spans = vec![Span::styled(
                &group.name,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )];

            if is_testing {
                // Show testing status
                spans.push(Span::styled(
                    " [Testing... Press R to refresh]",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                // Show selected policy
                spans.push(Span::styled(
                    selected_text,
                    Style::default().fg(Color::Green),
                ));

                // Recursively find the final real policy (handles nested groups)
                if let Some(selected_policy_name) = &group.selected {
                    let mut visited = HashSet::new();
                    if let Some(final_policy_name) =
                        resolve_final_policy(snapshot, selected_policy_name, &mut visited)
                    {
                        // Look up test results for the final resolved policy
                        if let Some(policy_detail) = snapshot
                            .policies
                            .iter()
                            .find(|p| p.name == final_policy_name)
                        {
                            // Test result available: show latency or availability
                            if policy_detail.alive {
                                if let Some(latency) = policy_detail.latency {
                                    // Color by latency: <100ms cyan, 100-300ms yellow, >300ms red
                                    let latency_color = if latency < 100 {
                                        Color::Cyan
                                    } else if latency < 300 {
                                        Color::Yellow
                                    } else {
                                        Color::Red
                                    };
                                    spans.push(Span::styled(
                                        format!(" ({}ms)", latency),
                                        Style::default()
                                            .fg(latency_color)
                                            .add_modifier(Modifier::BOLD),
                                    ));
                                } else {
                                    spans.push(Span::styled(
                                        " ✓",
                                        Style::default()
                                            .fg(Color::Green)
                                            .add_modifier(Modifier::BOLD),
                                    ));
                                }
                            } else {
                                spans.push(Span::styled(
                                    " ✗",
                                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                                ));
                            }
                        }
                    }
                }
            }

            let line = Line::from(spans);
            ListItem::new(line)
        })
        .collect();

    // btop-style title: embed colored shortcut keys
    let title = if search_mode {
        Line::from(vec![
            Span::raw(" "),
            Span::raw(t.policy_group_title()),
            Span::raw(" [Search: "),
            Span::raw(search_query),
            Span::raw("█] "),
        ])
    } else if !search_query.is_empty() {
        Line::from(vec![
            Span::raw(" "),
            Span::raw(t.policy_group_title()),
            Span::raw(" [Search: "),
            Span::raw(search_query),
            Span::raw("] "),
        ])
    } else if in_detail_mode {
        Line::from(vec![
            Span::raw(" "),
            Span::raw(t.policy_group_title()),
            Span::raw(" "),
        ])
    } else {
        Line::from(vec![
            Span::raw(" "),
            Span::raw(t.policy_group_title()),
            Span::raw(" ["),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_select()),
            Span::raw(" ["),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_enter()),
            Span::raw(" ["),
            Span::styled("t", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_test()),
            Span::raw(" ["),
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_search()),
            Span::raw(" "),
        ])
    };

    let highlight_style = if in_detail_mode {
        // In policy detail mode: reduce emphasis on group list
        Style::default().bg(Color::DarkGray)
    } else {
        // In group list mode: normal highlight
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(highlight_style)
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !filtered_groups.is_empty() {
        state.select(Some(selected.min(filtered_groups.len() - 1)));
    }

    f.render_stateful_widget(list, area, &mut state);
}

fn render_policy_group_policies(
    f: &mut Frame,
    area: Rect,
    snapshot: &AppSnapshot,
    selected: usize,
    policy_detail_index: Option<usize>,
    group_search_query: &str,
    policy_search_query: &str,
    search_mode: bool,
    t: &'static dyn Translate,
) {
    // First filter policy groups by group search query (not policy search)
    let filtered_groups: Vec<_> = if group_search_query.is_empty() {
        snapshot.policy_groups.iter().collect()
    } else {
        let query_lower = group_search_query.to_lowercase();
        snapshot
            .policy_groups
            .iter()
            .filter(|g| {
                g.name.to_lowercase().contains(&query_lower)
                    || g.selected
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .collect()
    };

    // Get the selected group from the filtered list
    let selected_group = if selected < filtered_groups.len() {
        filtered_groups[selected]
    } else {
        // Invalid index, show empty
        let empty = Paragraph::new(t.policy_no_selection()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.policy_group_title()),
        );
        f.render_widget(empty, area);
        return;
    };

    // Filter policies within the group by policy search query
    let filtered_policies: Vec<_> = if policy_search_query.is_empty() {
        selected_group.policies.iter().collect()
    } else {
        let query_lower = policy_search_query.to_lowercase();
        selected_group
            .policies
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.type_description.to_lowercase().contains(&query_lower)
            })
            .collect()
    };

    if filtered_policies.is_empty() {
        let empty = Paragraph::new(t.policy_no_policies()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.policy_policies_title(&selected_group.name)),
        );
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = filtered_policies
        .iter()
        .map(|policy_item| {
            // Only show ✓ for the currently selected policy
            let is_selected = selected_group
                .selected
                .as_ref()
                .map(|s| s == &policy_item.name)
                .unwrap_or(false);

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let selected_marker = if is_selected { "✓ " } else { "  " };

            // Look up latency data (supports nested policy groups)
            let (status_text, status_color) = {
                // First try to resolve the final policy recursively (handles nesting)
                let mut visited = HashSet::new();
                let final_policy_name =
                    resolve_final_policy(snapshot, &policy_item.name, &mut visited)
                        .unwrap_or_else(|| policy_item.name.clone());

                // Look up test results for the final resolved policy
                if let Some(detail) = snapshot
                    .policies
                    .iter()
                    .find(|p| p.name == final_policy_name)
                {
                    // Latency data available: show latency or failure
                    if detail.alive {
                        if let Some(latency) = detail.latency {
                            // Color by latency: <100ms cyan, 100-300ms yellow, >300ms red
                            let latency_color = if latency < 100 {
                                Color::Cyan
                            } else if latency < 300 {
                                Color::Yellow
                            } else {
                                Color::Red
                            };
                            (format!(" {}ms", latency), latency_color)
                        } else {
                            (" [Available]".to_string(), Color::Green)
                        }
                    } else {
                        (" [Unavailable]".to_string(), Color::Red)
                    }
                } else {
                    // No latency data: check available_policies
                    if let Some(ref available) = selected_group.available_policies {
                        if available.contains(&policy_item.name) {
                            (" [Available]".to_string(), Color::Green)
                        } else {
                            (" [Unavailable]".to_string(), Color::Red)
                        }
                    } else {
                        ("".to_string(), Color::DarkGray)
                    }
                }
            };

            // Dynamically compute column widths based on terminal width
            let (name_width, protocol_width, _status_width) =
                calculate_policy_column_widths(area.width);

            // Truncate policy name and type description to prevent overlap
            let truncated_name = truncate_text(&policy_item.name, name_width);
            let truncated_type = truncate_text(&policy_item.type_description, protocol_width);

            // Choose color based on protocol type
            let protocol_color = match policy_item.type_description.as_str() {
                s if s.contains("Shadowsocks") => Color::Blue,
                s if s.contains("VMess") => Color::Magenta,
                s if s.contains("Trojan") => Color::Yellow,
                s if s.contains("DIRECT") => Color::Green,
                s if s.contains("REJECT") => Color::Red,
                _ => Color::Gray,
            };

            let line = Line::from(vec![
                Span::styled(
                    selected_marker,
                    if is_selected {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::styled(truncated_name, name_style),
                Span::raw(" "),
                Span::styled(truncated_type, Style::default().fg(protocol_color)),
                Span::styled(status_text, Style::default().fg(status_color)),
            ]);

            ListItem::new(line)
        })
        .collect();

    // btop-style title: embed colored shortcut keys
    let title = if search_mode {
        // In search mode, show the policy search query (for detail mode)
        Line::from(vec![
            Span::raw(" "),
            Span::raw(&selected_group.name),
            Span::raw(" [Search: "),
            Span::raw(policy_search_query),
            Span::raw("█] "),
        ])
    } else if !policy_search_query.is_empty() {
        // Show active policy search filter
        Line::from(vec![
            Span::raw(" "),
            Span::raw(&selected_group.name),
            Span::raw(" [Search: "),
            Span::raw(policy_search_query),
            Span::raw("] "),
        ])
    } else if policy_detail_index.is_some() {
        // In detail mode: show navigation keys
        Line::from(vec![
            Span::raw(" "),
            Span::raw(&selected_group.name),
            Span::raw(" ["),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_select()),
            Span::raw(" ["),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_confirm()),
            Span::raw(" ["),
            Span::styled("ESC", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_back()),
            Span::raw(" ["),
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_search()),
            Span::raw(" "),
        ])
    } else {
        Line::from(vec![
            Span::raw(" "),
            Span::raw(&selected_group.name),
            Span::raw(" "),
        ])
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    // In policy detail mode: use stateful rendering to highlight selected policy
    if let Some(idx) = policy_detail_index {
        let mut state = ListState::default();
        if !filtered_policies.is_empty() {
            state.select(Some(idx.min(filtered_policies.len() - 1)));
        }
        f.render_stateful_widget(list, area, &mut state);
    } else {
        f.render_widget(list, area);
    }
}

/// Truncate text to a specified display width (CJK characters count as width 2)
///
/// Uses unicode-width to correctly compute display width for mixed-language text
fn truncate_text(text: &str, max_width: usize) -> String {
    let current_width = text.width();

    if current_width <= max_width {
        return text.to_string();
    }

    // Truncation needed: accumulate character widths, leaving room for ".."
    let ellipsis = "..";
    let ellipsis_width = 2;
    let target_width = max_width.saturating_sub(ellipsis_width);

    let mut accumulated_width = 0;
    let mut result = String::new();

    for ch in text.chars() {
        let char_width = ch.to_string().width();
        if accumulated_width + char_width > target_width {
            break;
        }
        result.push(ch);
        accumulated_width += char_width;
    }

    format!("{}{}", result, ellipsis)
}

/// Calculate policy list column widths based on terminal width
///
/// Returns: (name_width, protocol_width, status_width)
fn calculate_policy_column_widths(area_width: u16) -> (usize, usize, usize) {
    // Subtract fixed overhead for borders, padding, selection marker, etc.
    // borders 2, selection marker "✓ " 2, column spacing 2, padding 4
    let fixed_overhead = 10;
    let available = (area_width as usize).saturating_sub(fixed_overhead);

    // Status column fixed width: " 999ms" or " [Unavailable]" max ~10 chars
    let status_width = 10;

    // Remaining width split between name and protocol
    let remaining = available.saturating_sub(status_width);

    // name takes 60%, protocol takes 40%
    let name_width = (remaining as f32 * 0.6) as usize;
    let protocol_width = remaining.saturating_sub(name_width);

    (name_width.max(10), protocol_width.max(8), status_width)
}
