use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Render rename dialog
pub fn render_rename_dialog(f: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(50, 35, area);

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Current name
            Constraint::Length(3), // New name field
            Constraint::Length(3), // Error message (if any)
            Constraint::Length(3), // Instructions
        ])
        .split(popup_area);

    let item_type = if app.rename_dialog_is_folder {
        "Folder"
    } else {
        "Template"
    };

    // Title
    let title = Paragraph::new(format!("✏️ Rename {}", item_type))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Rename {}", item_type))
                .title_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(title, chunks[0]);

    // Current name (read-only)
    let current_name = Paragraph::new(&*app.rename_dialog_original_name)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Current Name")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::Gray));
    f.render_widget(current_name, chunks[1]);

    // New name field (editable)
    let new_name = Paragraph::new(&*app.rename_dialog_new_name)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("New Name")
                .title_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::DarkGray))
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(new_name, chunks[2]);

    // Error message
    if let Some(error) = &app.rename_dialog_error {
        let error_msg = Paragraph::new(error.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Error")
                    .style(Style::default().bg(Color::DarkGray)),
            )
            .style(Style::default().fg(Color::Red));
        f.render_widget(error_msg, chunks[3]);
    }

    // Instructions
    let instructions = Paragraph::new("Enter: Rename  |  Esc: Cancel")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Controls")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::Gray));
    f.render_widget(instructions, chunks[4]);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
