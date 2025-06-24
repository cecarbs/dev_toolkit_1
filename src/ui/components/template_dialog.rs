use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Render template creation dialog
pub fn render_template_creation_dialog(f: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(60, 50, area);

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Name field
            Constraint::Length(3), // Folder field
            Constraint::Length(4), // Description field
            Constraint::Length(3), // Instructions
        ])
        .split(popup_area);

    // Title
    let title = Paragraph::new("📁 Create New Template")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Template Creation")
                .title_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(title, chunks[0]);

    // Name field
    let name_focused = app.template_dialog_focused_field == 0;
    let name_style = if name_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let name = Paragraph::new(&*app.template_dialog_name)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Template Name")
                .title_style(if name_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                })
                .style(Style::default().bg(Color::DarkGray))
                .border_style(if name_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }),
        )
        .style(name_style);
    f.render_widget(name, chunks[1]);

    // Folder field
    let folder_focused = app.template_dialog_focused_field == 1;
    let folder_display = if app.template_dialog_folder.is_empty() {
        "Root"
    } else {
        &app.template_dialog_folder
    };

    let folder = Paragraph::new(folder_display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Target Folder")
                .title_style(if folder_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                })
                .style(Style::default().bg(Color::DarkGray))
                .border_style(if folder_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }),
        )
        .style(if folder_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        });
    f.render_widget(folder, chunks[2]);

    // Description field
    let desc_focused = app.template_dialog_focused_field == 2;
    let description = Paragraph::new(&*app.template_dialog_description)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Description")
                .title_style(if desc_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                })
                .style(Style::default().bg(Color::DarkGray))
                .border_style(if desc_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }),
        )
        .style(if desc_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        });
    f.render_widget(description, chunks[3]);

    // Instructions
    let instructions = vec![
        Line::from("Tab: Next field  |  Enter: Create  |  Esc: Cancel"),
        Line::from("(Folder path like: Customer/Add or empty for root)"),
    ];

    let help = Paragraph::new(instructions)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Controls")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::Gray));
    f.render_widget(help, chunks[4]);
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
