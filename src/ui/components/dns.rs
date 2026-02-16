/// DNS 组件 - DNS 缓存列表
use crate::domain::models::DnsRecord;
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
    records: &[DnsRecord],
    selected: usize,
    search_query: &str,
    search_mode: bool,
    t: &'static dyn Translate,
) {
    // 根据搜索过滤记录
    let filtered_records: Vec<_> = if search_query.is_empty() {
        records.iter().collect()
    } else {
        let query_lower = search_query.to_lowercase();
        records
            .iter()
            .filter(|r| r.domain.to_lowercase().contains(&query_lower))
            .collect()
    };

    // 分割区域：DNS 列表 | 详细信息
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_dns_list(
        f,
        chunks[0],
        &filtered_records,
        selected,
        search_query,
        search_mode,
        t,
    );
    render_dns_detail(f, chunks[1], &filtered_records, selected, t);
}

fn render_dns_list(
    f: &mut Frame,
    area: Rect,
    records: &[&DnsRecord],
    selected: usize,
    search_query: &str,
    search_mode: bool,
    t: &'static dyn Translate,
) {
    let title = if search_mode {
        Line::from(vec![
            Span::raw(" "),
            Span::raw(t.dns_list_title()),
            Span::raw(" [Search: "),
            Span::raw(search_query),
            Span::raw("█] "),
        ])
    } else if !search_query.is_empty() {
        Line::from(vec![
            Span::raw(" "),
            Span::raw(t.dns_list_title()),
            Span::raw(" [Search: "),
            Span::raw(search_query),
            Span::raw("] "),
        ])
    } else {
        // 显示快捷键提示（btop 风格）
        Line::from(vec![
            Span::raw(" "),
            Span::raw(t.dns_list_title()),
            Span::raw(" ["),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_select()),
            Span::raw(" ["),
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_search()),
            Span::raw(" ["),
            Span::styled("f", Style::default().fg(Color::Yellow)),
            Span::raw("]"),
            Span::raw(t.action_flush()),
            Span::raw(" "),
        ])
    };

    let items: Vec<ListItem> = if records.is_empty() {
        vec![ListItem::new(Span::styled(
            t.dns_no_records(),
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        records
            .iter()
            .map(|record| {
                let domain = &record.domain;
                let ips = record.ip.join(", ");
                let ip_preview = if ips.len() > 40 {
                    format!("{}...", &ips[..37])
                } else {
                    ips
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:<40}", domain), Style::default().fg(Color::Cyan)),
                    Span::raw(" → "),
                    Span::styled(ip_preview, Style::default().fg(Color::Green)),
                ]))
            })
            .collect()
    };

    let mut state = ListState::default();
    if !records.is_empty() {
        state.select(Some(selected));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, area, &mut state);
}

fn render_dns_detail(
    f: &mut Frame,
    area: Rect,
    records: &[&DnsRecord],
    selected: usize,
    t: &'static dyn Translate,
) {
    let record = records.get(selected);

    let mut lines = vec![];

    if let Some(record) = record {
        // 域名
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", t.dns_label_domain()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(&record.domain, Style::default().fg(Color::Cyan)),
        ]));

        lines.push(Line::from(""));

        // IP 地址列表
        lines.push(Line::from(vec![Span::styled(
            format!("{}: ", t.dns_label_value()),
            Style::default().add_modifier(Modifier::BOLD),
        )]));

        for ip in &record.ip {
            lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::styled(ip, Style::default().fg(Color::Green)),
            ]));
        }

        lines.push(Line::from(""));

        // TTL (expiresTime 是 Unix 时间戳，需要转换为剩余秒数)
        if let Some(expires_time) = record.ttl {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            let remaining = (expires_time - now).max(0.0);

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", t.dns_label_ttl()),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:.0} s", remaining),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            t.dns_no_records(),
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.dns_detail_title()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
