/// Policies 组件 - 策略和策略组显示
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
    // 分割区域：策略组 | 策略列表
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

/// 递归查找策略组最终选中的真实策略（不是策略组）
///
/// 例如：Proxy → US_Servers → us-bwg-la-dc1-vmess
/// 返回：Some("us-bwg-la-dc1-vmess")
fn resolve_final_policy(
    snapshot: &AppSnapshot,
    policy_name: &str,
    visited: &mut HashSet<String>,
) -> Option<String> {
    // 防止循环引用
    if visited.contains(policy_name) || visited.len() > 10 {
        return None;
    }
    visited.insert(policy_name.to_string());

    // 查找是否是策略组
    if let Some(group) = snapshot
        .policy_groups
        .iter()
        .find(|g| g.name == policy_name)
    {
        // 是策略组，继续递归查找它的选中策略
        if let Some(selected) = &group.selected {
            return resolve_final_policy(snapshot, selected, visited);
        } else {
            // 策略组没有选中任何策略
            return None;
        }
    }

    // 不是策略组，就是真实策略
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

            // 检查是否正在测试
            let is_testing = testing_group.map(|tg| tg == group.name).unwrap_or(false);

            let mut spans = vec![Span::styled(
                &group.name,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )];

            if is_testing {
                // 显示测试中状态
                spans.push(Span::styled(
                    " [Testing... Press R to refresh]",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                // 显示选中的策略
                spans.push(Span::styled(
                    selected_text,
                    Style::default().fg(Color::Green),
                ));

                // 递归查找最终选中的真实策略（处理策略组嵌套）
                if let Some(selected_policy_name) = &group.selected {
                    let mut visited = HashSet::new();
                    if let Some(final_policy_name) =
                        resolve_final_policy(snapshot, selected_policy_name, &mut visited)
                    {
                        // 查找最终策略的测试结果
                        if let Some(policy_detail) = snapshot
                            .policies
                            .iter()
                            .find(|p| p.name == final_policy_name)
                        {
                            // 有测试结果：显示延迟或可用状态
                            if policy_detail.alive {
                                if let Some(latency) = policy_detail.latency {
                                    // 根据延迟值设置颜色
                                    // < 100ms 青色，100-300ms 黄色，> 300ms 红色
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

    // btop 风格的标题：嵌入带颜色的快捷键
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
        // 在策略组内部时，降低策略组列表的突出度
        Style::default().bg(Color::DarkGray)
    } else {
        // 在策略组列表时，正常高亮
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

    // 获取选中的策略组（从过滤后的列表）
    let selected_group = if selected < filtered_groups.len() {
        filtered_groups[selected]
    } else {
        // 无效索引，显示空
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
            // 只有当前选中的策略才显示 ✓
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

            // 查找延迟数据（支持嵌套策略组）
            let (status_text, status_color) = {
                // 先尝试递归查找最终策略（处理策略组嵌套）
                let mut visited = HashSet::new();
                let final_policy_name =
                    resolve_final_policy(snapshot, &policy_item.name, &mut visited)
                        .unwrap_or_else(|| policy_item.name.clone());

                // 查找最终策略的测试结果
                if let Some(detail) = snapshot
                    .policies
                    .iter()
                    .find(|p| p.name == final_policy_name)
                {
                    // 有延迟数据：显示延迟或失败
                    if detail.alive {
                        if let Some(latency) = detail.latency {
                            // 根据延迟值设置颜色
                            // < 100ms 青色，100-300ms 黄色，> 300ms 红色
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
                    // 无延迟数据：检查 available_policies
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

            // 根据终端宽度动态计算列宽
            let (name_width, protocol_width, _status_width) =
                calculate_policy_column_widths(area.width);

            // 截断策略名和类型描述以防重叠
            let truncated_name = truncate_text(&policy_item.name, name_width);
            let truncated_type = truncate_text(&policy_item.type_description, protocol_width);

            // 根据协议类型选择颜色
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

    // btop 风格的标题：嵌入带颜色的快捷键
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

    // 如果在策略组内部，使用 stateful 渲染高亮选中的策略
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

/// 截断文本到指定显示宽度（考虑 CJK 字符占 2 宽度）
///
/// 使用 unicode-width 正确计算中英文混排的显示宽度
fn truncate_text(text: &str, max_width: usize) -> String {
    let current_width = text.width();

    if current_width <= max_width {
        return text.to_string();
    }

    // 需要截断：逐字符累加宽度，保留 ".." 的空间
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

/// 根据终端宽度计算策略列表的列宽
///
/// 返回: (name_width, protocol_width, status_width)
fn calculate_policy_column_widths(area_width: u16) -> (usize, usize, usize) {
    // 减去边框、padding、选择标记等固定开销
    // 边框 2, 选择标记 "✓ " 2, 列间空格 2, padding 4
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
