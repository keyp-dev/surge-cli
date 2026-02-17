/// Alerts component - displays warnings and error messages
use crate::domain::entities::{Alert, AlertLevel};
use crate::i18n::Translate;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, alerts: &[Alert], t: &'static dyn Translate) {
    if alerts.is_empty() {
        return;
    }

    // Show only the first (most important) alert
    let alert = &alerts[0];

    let (color, prefix) = match alert.level {
        AlertLevel::Info => (Color::Blue, "ℹ"),
        AlertLevel::Warning => (Color::Yellow, "⚠"),
        AlertLevel::Error => (Color::Red, "✖"),
    };

    // Translate by message key (domain layer uses keys; UI layer translates)
    let message = match alert.message.as_str() {
        "surge_not_running" => t.alert_surge_not_running().to_string(),
        "http_api_disabled" => t.alert_http_api_disabled().to_string(),
        _ => alert.message.clone(), // Dynamic messages are passed through as-is
    };

    let mut spans = vec![
        Span::styled(
            format!("{} ", prefix),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(message, Style::default().fg(color)),
    ];

    // Add action hint
    use crate::domain::entities::AlertAction;
    let action_text = match &alert.action {
        AlertAction::StartSurge => Some(t.alert_action_start_surge()),
        AlertAction::ReloadConfig => Some(t.alert_action_reload_config()),
        AlertAction::None => None,
    };
    if let Some(action) = action_text {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(
            action,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let alert_line = Line::from(spans);

    let paragraph =
        Paragraph::new(alert_line).block(Block::default().borders(Borders::ALL).title("Alert"));

    f.render_widget(paragraph, area);
}
