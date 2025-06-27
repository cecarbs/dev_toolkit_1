// Create new file: src/ui/components/import_dialog.rs

use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Render Postman collection import dialog
pub fn render_import_dialog(f: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(70, 60, area);

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // File path field
            Constraint::Length(6), // Instructions
            Constraint::Length(3), // Error message (if any)
            Constraint::Length(4), // Preview info (if valid file)
            Constraint::Length(3), // Buttons
        ])
        .split(popup_area);

    // Title
    let title = Paragraph::new("📥 Import Postman Collection")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Import Collection")
                .title_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::DarkGray))
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(title, chunks[0]);

    // File path field
    let file_path_text = if app.import_dialog_file_path.is_empty() {
        "Enter path to .json collection file..."
    } else {
        &app.import_dialog_file_path
    };

    let file_path_style = if app.import_dialog_file_path.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let file_path = Paragraph::new(file_path_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("File Path")
                .title_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::DarkGray))
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(file_path_style);
    f.render_widget(file_path, chunks[1]);

    // Instructions
    let instructions = vec![
        Line::from(Span::styled(
            "📂 Examples:",
            Style::default().fg(Color::Green),
        )),
        Line::from("  ~/Downloads/my-collection.json"),
        Line::from("  /Users/username/Desktop/api-tests.postman_collection.json"),
        Line::from("  ./collections/sample.json"),
        Line::from(""),
        Line::from(Span::styled(
            "💡 Tip: Drag & drop files to terminal (some terminals)",
            Style::default().fg(Color::Gray),
        )),
    ];

    let instructions_widget = Paragraph::new(instructions)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Instructions")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(instructions_widget, chunks[2]);

    // Error message or validation
    if let Some(error) = &app.import_dialog_error {
        let error_msg = Paragraph::new(error.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Error")
                    .style(Style::default().bg(Color::DarkGray)),
            )
            .style(Style::default().fg(Color::Red));
        f.render_widget(error_msg, chunks[3]);
    } else if !app.import_dialog_file_path.is_empty() {
        // Show file validation status
        let (status_text, status_color) = validate_file_path_display(&app.import_dialog_file_path);
        let status_msg = Paragraph::new(status_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("File Status")
                    .style(Style::default().bg(Color::DarkGray)),
            )
            .style(Style::default().fg(status_color));
        f.render_widget(status_msg, chunks[3]);
    }

    // Preview info (if file is valid)
    if let Some(preview) = &app.import_dialog_preview {
        let preview_lines = vec![
            Line::from(vec![
                Span::raw("Collection: "),
                Span::styled(&preview.name, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("Requests: "),
                Span::styled(
                    preview.request_count.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Folders: "),
                Span::styled(
                    preview.folder_count.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let preview_widget = Paragraph::new(preview_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Preview")
                    .style(Style::default().bg(Color::DarkGray)),
            )
            .style(Style::default().fg(Color::White));
        f.render_widget(preview_widget, chunks[4]);
    } else {
        let empty_preview = Paragraph::new("Enter a valid file path to see preview")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Preview")
                    .style(Style::default().bg(Color::DarkGray)),
            )
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty_preview, chunks[4]);
    }

    // Buttons
    let can_import = app.import_dialog_preview.is_some() && app.import_dialog_error.is_none();

    let buttons = if can_import {
        vec![Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": "),
            Span::styled("Import Collection", Style::default().fg(Color::Green)),
            Span::raw("    "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": "),
            Span::styled("Cancel", Style::default().fg(Color::Red)),
        ])]
    } else {
        vec![Line::from(vec![
            Span::styled("Enter path above", Style::default().fg(Color::DarkGray)),
            Span::raw("    "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": "),
            Span::styled("Cancel", Style::default().fg(Color::Red)),
        ])]
    };

    let button_widget = Paragraph::new(buttons)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Controls")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(button_widget, chunks[5]);
}

/// Validate file path and return display text and color
fn validate_file_path_display(file_path: &str) -> (String, Color) {
    use std::path::Path;

    let path = Path::new(file_path);

    if !path.exists() {
        ("❌ File does not exist".to_string(), Color::Red)
    } else if !path.is_file() {
        ("❌ Path is not a file".to_string(), Color::Red)
    } else if path.extension().and_then(|s| s.to_str()) != Some("json") {
        ("⚠️  File should be .json format".to_string(), Color::Yellow)
    } else {
        ("✅ Valid JSON file found".to_string(), Color::Green)
    }
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
