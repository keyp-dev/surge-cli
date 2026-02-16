/// Overview 组件 - 系统总览
use crate::domain::entities::AppSnapshot;
use crate::i18n::Translate;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, snapshot: &AppSnapshot, t: &'static dyn Translate) {
    let mut lines = vec![];

    // Surge 状态
    let surge_status_text = if snapshot.surge_running {
        format!("{} ✓", t.ui_status_running())
    } else {
        format!("{} ✖", t.ui_status_stopped())
    };
    let surge_status_color = if snapshot.surge_running {
        Color::Green
    } else {
        Color::Red
    };

    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", t.overview_surge_status()),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(surge_status_text, Style::default().fg(surge_status_color)),
    ]));

    // HTTP API 状态
    let api_status_text = if snapshot.http_api_available {
        format!("{} ✓", t.policy_available())
    } else {
        format!("{} ✖", t.policy_unavailable())
    };
    let api_status_color = if snapshot.http_api_available {
        Color::Green
    } else {
        Color::Red
    };

    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", t.overview_api_status()),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(api_status_text, Style::default().fg(api_status_color)),
    ]));

    // 出站模式（可切换）
    if let Some(ref mode) = snapshot.outbound_mode {
        use crate::domain::models::OutboundMode;
        let mode_text = match mode {
            OutboundMode::Direct => t.outbound_mode_direct(),
            OutboundMode::Proxy => t.outbound_mode_proxy(),
            OutboundMode::Rule => t.outbound_mode_rule(),
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", t.overview_outbound_mode()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(mode_text, Style::default().fg(Color::Cyan)),
            Span::raw("  ["),
            Span::styled("m", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_toggle()),
        ]));
    }

    // MITM 状态（可切换）
    if snapshot.http_api_available {
        if let Some(mitm_enabled) = snapshot.mitm_enabled {
            let status_text = if mitm_enabled {
                t.status_enabled()
            } else {
                t.status_disabled()
            };
            let status_color = if mitm_enabled {
                Color::Green
            } else {
                Color::Gray
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", t.feature_mitm()),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(status_text, Style::default().fg(status_color)),
                Span::raw("  ["),
                Span::styled("i", Style::default().fg(Color::Yellow)),
                Span::raw("]"),
                Span::raw(t.action_toggle()),
            ]));
        }

        // Capture 状态（可切换）
        if let Some(capture_enabled) = snapshot.capture_enabled {
            let status_text = if capture_enabled {
                t.status_enabled()
            } else {
                t.status_disabled()
            };
            let status_color = if capture_enabled {
                Color::Green
            } else {
                Color::Gray
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", t.feature_capture()),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(status_text, Style::default().fg(status_color)),
                Span::raw("  ["),
                Span::styled("c", Style::default().fg(Color::Yellow)),
                Span::raw("]"),
                Span::raw(t.action_toggle()),
            ]));
        }
    }

    lines.push(Line::from("")); // 空行

    // 统计信息
    lines.push(Line::from(vec![Span::styled(
        t.overview_stats(),
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )]));

    // 统计数据
    let stats = [
        (t.stats_policies(), snapshot.policies.len(), Color::Yellow),
        (
            t.stats_policy_groups(),
            snapshot.policy_groups.len(),
            Color::Yellow,
        ),
        (
            t.stats_active_connections(),
            snapshot.active_connections.len(),
            Color::Green,
        ),
        (
            t.stats_recent_requests(),
            snapshot.recent_requests.len(),
            Color::Blue,
        ),
    ];

    for (label, count, color) in stats {
        lines.push(Line::from(vec![
            Span::raw(format!("  {}: ", label)),
            Span::styled(count.to_string(), Style::default().fg(color)),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(t.view_overview()),
    );

    f.render_widget(paragraph, area);
}
