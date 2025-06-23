use crate::app::App;
use crate::models::LogEntry;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Render the logging panel with search functionality
pub fn render_logging_panel(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(5),    // Log entries
        ])
        .split(area);

    render_search_bar(f, chunks[0], app);
    render_log_entries(f, chunks[1], app);
}

/// Render the search bar at the top of the logging panel
fn render_search_bar(f: &mut Frame, area: Rect, app: &App) {
    let search_text = if app.log_search_query.is_empty() {
        "Type to search logs...".to_string()
    } else {
        app.log_search_query.clone()
    };

    let style = if app.log_search_query.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let search_bar = Paragraph::new(search_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🔍 Search Logs (Delete to clear)")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .style(style);

    f.render_widget(search_bar, area);
}

/// Render the list of log entries
fn render_log_entries(f: &mut Frame, area: Rect, app: &App) {
    let filtered_logs = app.get_filtered_logs();

    // Take the last N logs that fit in the display area
    let display_height = area.height.saturating_sub(2) as usize; // Account for borders
    let start_index = if filtered_logs.len() > display_height {
        filtered_logs.len() - display_height
    } else {
        0
    };

    let items: Vec<ListItem> = filtered_logs
        .iter()
        .skip(start_index)
        .map(|log_entry| create_log_list_item(log_entry))
        .collect();

    let log_count_info = if app.log_search_query.is_empty() {
        format!("Logs ({} total)", app.log_entries.len())
    } else {
        format!(
            "Logs ({} of {} shown)",
            filtered_logs.len(),
            app.log_entries.len()
        )
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(log_count_info)
                .title_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default());

    f.render_widget(list, area);
}

/// Create a styled list item for a log entry
fn create_log_list_item(log_entry: &LogEntry) -> ListItem {
    let level_style = log_entry.level.style();

    let line = Line::from(vec![
        // Timestamp
        Span::styled(
            format!("[{}]", log_entry.timestamp.format("%H:%M:%S")),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(" "),
        // Log level with appropriate color
        Span::styled(format!("{:>7}", log_entry.level.as_str()), level_style),
        Span::raw(" "),
        // Message
        Span::styled(&log_entry.message, Style::default().fg(Color::White)),
    ]);

    ListItem::new(line)
}

/// Render a small log summary when the panel is closed
pub fn render_log_summary(f: &mut Frame, area: Rect, app: &App) {
    if app.log_entries.is_empty() {
        return;
    }

    // Show the last few log entries in a compact format
    let recent_logs: Vec<&LogEntry> = app.log_entries.iter().rev().take(3).collect();

    let mut lines = vec![Line::from(Span::styled(
        "Recent Logs (Ctrl+L to open):",
        Style::default().fg(Color::Cyan),
    ))];

    for log_entry in recent_logs.iter().rev() {
        let level_color = match log_entry.level {
            crate::models::LogLevel::Error => Color::Red,
            crate::models::LogLevel::Warn => Color::Yellow,
            crate::models::LogLevel::Success => Color::Green,
            crate::models::LogLevel::Info => Color::White,
            crate::models::LogLevel::Debug => Color::Gray,
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("[{}]", log_entry.timestamp.format("%H:%M:%S")),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(" "),
            Span::styled(log_entry.level.as_str(), Style::default().fg(level_color)),
            Span::raw(": "),
            Span::styled(
                truncate_string(&log_entry.message, 50),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Logs")
                .title_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default());

    f.render_widget(paragraph, area);
}

/// Helper function to truncate strings for display
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Render log level statistics
pub fn render_log_stats(f: &mut Frame, area: Rect, app: &App) {
    let mut error_count = 0;
    let mut warn_count = 0;
    let mut info_count = 0;
    let mut debug_count = 0;
    let mut success_count = 0;

    for entry in &app.log_entries {
        match entry.level {
            crate::models::LogLevel::Error => error_count += 1,
            crate::models::LogLevel::Warn => warn_count += 1,
            crate::models::LogLevel::Info => info_count += 1,
            crate::models::LogLevel::Debug => debug_count += 1,
            crate::models::LogLevel::Success => success_count += 1,
        }
    }

    let stats_text = vec![
        Line::from("Log Statistics:"),
        Line::from(vec![
            Span::styled("Errors: ", Style::default().fg(Color::Red)),
            Span::raw(error_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Warnings: ", Style::default().fg(Color::Yellow)),
            Span::raw(warn_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Success: ", Style::default().fg(Color::Green)),
            Span::raw(success_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Info: ", Style::default().fg(Color::White)),
            Span::raw(info_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Debug: ", Style::default().fg(Color::Gray)),
            Span::raw(debug_count.to_string()),
        ]),
    ];

    let paragraph = Paragraph::new(stats_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Statistics")
            .title_style(Style::default().fg(Color::Magenta)),
    );

    f.render_widget(paragraph, area);
}
