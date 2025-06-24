// Folder creation dialog component (add to ui/components/folder_dialog.rs)
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Render folder creation dialog
pub fn render_folder_creation_dialog(f: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(50, 40, area);

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Name field
            Constraint::Length(3), // Parent field
            Constraint::Length(3), // Error message (if any)
            Constraint::Length(3), // Instructions
        ])
        .split(popup_area);

    // Title
    let title = Paragraph::new("📁 Create New Folder")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Folder Creation")
                .title_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(title, chunks[0]);

    // Name field
    let name = Paragraph::new(&app.folder_dialog_name)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Folder Name")
                .title_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::DarkGray))
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(name, chunks[1]);

    // Parent field (read-only display)
    let parent_display = if app.folder_dialog_parent.is_empty() {
        "Root"
    } else {
        &app.folder_dialog_parent
    };

    let parent = Paragraph::new(parent_display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Parent Folder")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::Gray));
    f.render_widget(parent, chunks[2]);

    // Error message
    if let Some(error) = &app.folder_dialog_error {
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
    let instructions = Paragraph::new("Enter: Create folder  |  Esc: Cancel")
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
