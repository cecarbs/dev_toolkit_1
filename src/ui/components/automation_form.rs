use crate::app::FocusedPane;
use crate::modes::automation::AutomationState;
use crate::services::AuthService;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

/// Render the automation form with fields and send button (no templates section)
pub fn render_automation_form(
    f: &mut Frame,
    area: Rect,
    state: &AutomationState,
    auth_service: &AuthService,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Form fields area (takes most space)
            Constraint::Length(6), // Send button and auth status
        ])
        .split(area);

    // Render form fields
    render_form_fields(f, chunks[0], state);

    // Render send button and auth status
    render_send_section(f, chunks[1], state, auth_service);
}

/// Render the form fields with proper focus indicators
fn render_form_fields(f: &mut Frame, area: Rect, state: &AutomationState) {
    // Get border style based on focus - make it obvious
    let border_style = Style::default().fg(Color::White);
    let title_style = Style::default().fg(Color::Green);

    let field_items: Vec<ListItem> = state
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| render_field_item(field, i, state.focused_field))
        .collect();

    let list = List::new(field_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Form Fields (Tab/Shift+Tab to navigate, F6 to focus)")
            .title_style(title_style)
            .border_style(border_style),
    );

    f.render_widget(list, area);
}

/// Render a single form field item with appropriate input type
fn render_field_item(
    field: &crate::models::FormField,
    index: usize,
    focused_index: usize,
) -> ListItem {
    let is_focused = index == focused_index;
    let is_valid = field.is_valid();

    // Determine field display value and style
    let (display_value, value_style) = match &field.field_type {
        crate::models::FieldType::Select => {
            if field.value.is_empty() {
                (
                    "<Select Option>".to_string(),
                    Style::default().fg(Color::DarkGray),
                )
            } else {
                (field.value.clone(), Style::default().fg(Color::Cyan))
            }
        }
        _ => {
            if field.value.is_empty() {
                ("<Empty>".to_string(), Style::default().fg(Color::DarkGray))
            } else {
                (field.value.clone(), Style::default().fg(Color::White))
            }
        }
    };

    // Create the field label with validation indicator
    let label_style = if !is_valid && field.is_required {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Green)
    };

    let validation_indicator = if field.is_required && !is_valid {
        " ❌"
    } else if is_valid {
        " ✓"
    } else {
        ""
    };

    // Field type indicator
    let type_indicator = match field.field_type {
        crate::models::FieldType::Select => " 📋",
        crate::models::FieldType::Textarea => " 📝",
        crate::models::FieldType::Email => " ✉️",
        _ => " 📄",
    };

    // Background style for focused field
    let background_style = if is_focused {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };

    let line = Line::from(vec![
        Span::styled(format!("{:18}", field.get_display_label()), label_style),
        Span::raw(": "),
        Span::styled(display_value, value_style.patch(background_style)),
        Span::styled(type_indicator, Style::default().fg(Color::Blue)),
        Span::styled(validation_indicator, Style::default()),
    ]);

    // Add dropdown options if it's a select field and focused
    if is_focused && matches!(field.field_type, crate::models::FieldType::Select) {
        let options = field.get_dropdown_options();
        let options_text = if options.is_empty() {
            "No options available".to_string()
        } else {
            format!("Press 1-{}: {}", options.len(), options.join(", "))
        };

        let options_line = Line::from(vec![
            Span::raw("                    "),
            Span::styled(options_text, Style::default().fg(Color::Gray)),
        ]);

        ListItem::new(vec![line, options_line])
    } else {
        ListItem::new(line)
    }
}

/// Render send button and authentication status
fn render_send_section(
    f: &mut Frame,
    area: Rect,
    state: &AutomationState,
    auth_service: &AuthService,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Left: Big prominent send button
    render_send_button(f, chunks[0], state, auth_service);

    // Right: Authentication and template status
    render_status_info(f, chunks[1], state, auth_service);
}

/// Render the big prominent send button
fn render_send_button(
    f: &mut Frame,
    area: Rect,
    state: &AutomationState,
    auth_service: &AuthService,
) {
    let is_valid = state.is_valid();
    let has_auth = auth_service.has_credentials();
    let is_running = state.is_running;

    let (button_text, button_style) = if is_running {
        (
            "🤖 BROWSER AUTOMATION RUNNING...",
            Style::default().fg(Color::Yellow),
        )
    } else if !is_valid {
        (
            "❌ FIX VALIDATION ERRORS FIRST",
            Style::default().fg(Color::Red),
        )
    } else if !has_auth {
        (
            "🔐 PRESS SPACE TO LOGIN & SEND",
            Style::default().fg(Color::Cyan),
        )
    } else {
        ("🚀 PRESS SPACE TO SEND", Style::default().fg(Color::Green))
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(button_text, button_style)),
        Line::from(""),
        Line::from(Span::styled(
            "Space Bar or Ctrl+Enter to Send",
            Style::default().fg(Color::Gray),
        )),
    ];

    // Add validation errors if any
    if !is_valid {
        lines.push(Line::from(""));
        let errors = state.get_validation_errors();
        for error in errors.iter().take(1) {
            // Show max 1 error to fit
            lines.push(Line::from(Span::styled(
                format!("• {}", error),
                Style::default().fg(Color::Red),
            )));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("SEND")
                .title_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Render status information
fn render_status_info(
    f: &mut Frame,
    area: Rect,
    state: &AutomationState,
    auth_service: &AuthService,
) {
    let has_credentials = auth_service.has_credentials();

    let mut lines = vec![
        Line::from(Span::styled("Status:", Style::default().fg(Color::Cyan))),
        Line::from(""),
    ];

    // Authentication status
    if has_credentials {
        let username = auth_service.get_username().unwrap_or_default();
        lines.push(Line::from(vec![
            Span::raw("Login: "),
            Span::styled(username, Style::default().fg(Color::Green)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::raw("Login: "),
            Span::styled("Not logged in", Style::default().fg(Color::Red)),
        ]));
    }

    // Template status
    if let Some(template) = state.get_selected_template() {
        lines.push(Line::from(vec![
            Span::raw("Template: "),
            Span::styled(&template.name, Style::default().fg(Color::Green)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::raw("Template: "),
            Span::styled("None", Style::default().fg(Color::Gray)),
        ]));
    }

    // Field progress
    let filled_fields = state.fields.iter().filter(|f| f.is_valid()).count();
    lines.push(Line::from(vec![
        Span::raw("Fields: "),
        Span::styled(
            format!("{}/{}", filled_fields, state.fields.len()),
            if filled_fields == state.fields.len() {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            },
        ),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Info")
                .title_style(Style::default().fg(Color::Magenta)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Render login popup modal
pub fn render_login_popup(f: &mut Frame, area: Rect, app: &crate::app::App) {
    let popup_area = centered_rect(50, 40, area);

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Username field
            Constraint::Length(3), // Password field
            Constraint::Length(3), // Error message (if any)
            Constraint::Length(3), // Buttons
        ])
        .split(popup_area);

    // Title
    let title = Paragraph::new("🔐 Login Required")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Authentication")
                .title_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(title, chunks[0]);

    // Username field
    let username_text = if app.login_username.is_empty() {
        "Enter username..."
    } else {
        &app.login_username
    };
    let username = Paragraph::new(username_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Username")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(username, chunks[1]);

    // Password field (masked)
    let password_display = "*".repeat(app.login_password.len());
    let password_text = if app.login_password.is_empty() {
        "Enter password..."
    } else {
        &password_display
    };
    let password = Paragraph::new(password_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Password")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(password, chunks[2]);

    // Error message
    if let Some(error) = &app.login_error {
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
    let instructions = Paragraph::new("Enter to login, Esc to cancel")
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
