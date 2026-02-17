use chrono::{DateTime, Local};
/// Notifications component - bottom-right notification area
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Notification message
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub created_at: DateTime<Local>,
}

/// Notification level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Error,
}

impl Notification {
    pub fn color(&self) -> Color {
        match self.level {
            NotificationLevel::Info => Color::Cyan,
            NotificationLevel::Success => Color::Green,
            NotificationLevel::Error => Color::Red,
        }
    }

    pub fn icon(&self) -> &str {
        match self.level {
            NotificationLevel::Info => "ℹ",
            NotificationLevel::Success => "✓",
            NotificationLevel::Error => "✗",
        }
    }

    /// Get remaining display time in seconds
    pub fn remaining_time(&self) -> u64 {
        let now = Local::now();
        let elapsed = (now - self.created_at).num_seconds().max(0) as u64;
        5u64.saturating_sub(elapsed)
    }
}

/// Render notification area
pub fn render(f: &mut Frame, area: Rect, notifications: &[Notification]) {
    if notifications.is_empty() {
        return;
    }

    // Show only the most recent 5 notifications
    let recent_notifications: Vec<_> = notifications.iter().rev().take(5).collect();

    // Build notification text (bottom to top)
    let mut lines = Vec::new();

    for notification in recent_notifications.iter().rev() {
        let remaining = notification.remaining_time();
        let time_indicator = if remaining > 0 {
            format!(" ({}s)", remaining)
        } else {
            String::new()
        };

        let line = Line::from(vec![
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
            Span::styled(time_indicator, Style::default().fg(Color::DarkGray)),
        ]);

        lines.push(line);
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Notifications")
            .style(Style::default().fg(Color::Gray)),
    );

    f.render_widget(paragraph, area);
}
