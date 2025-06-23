use crate::modes::automation::AutomationState;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

/// Render the automation form with fields and templates
pub fn render_automation_form(f: &mut Frame, area: Rect, state: &AutomationState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Main form area
            Constraint::Percentage(40), // Credentials and status area
        ])
        .split(area);

    // Top area: Form fields and templates
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[0]);

    // Left side: Form fields
    render_form_fields(f, top_chunks[0], state);

    // Right side: Templates
    render_templates_section(f, top_chunks[1], state);

    // Bottom area: Credentials and status
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    render_credentials_section(f, bottom_chunks[0], state);
    render_status_and_actions(f, bottom_chunks[1], state);
}

/// Render the form fields on the left side
fn render_form_fields(f: &mut Frame, area: Rect, state: &AutomationState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Fields
            Constraint::Length(3), // Action buttons
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Form Fields")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Automation Mode"),
        )
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(title, chunks[0]);

    // Form fields
    render_field_list(f, chunks[1], state);

    // Action buttons
    render_action_buttons(f, chunks[2], state);
}

/// Render the list of form fields with current values
fn render_field_list(f: &mut Frame, area: Rect, state: &AutomationState) {
    let items: Vec<ListItem> = state
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let is_focused = i == state.focused_field;
            let display_value = if field.value.is_empty() {
                "<empty>".to_string()
            } else {
                field.value.clone()
            };

            let style = if is_focused {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:15}", field.name),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(": "),
                Span::styled(display_value, style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Fields (Tab/Shift+Tab to navigate)"),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, area);
}

/// Render action buttons at the bottom
fn render_action_buttons(f: &mut Frame, area: Rect, state: &AutomationState) {
    let has_credentials = state.credentials.is_some();
    let fields_valid = state.is_valid();

    let (button_text, style) = if state.is_running {
        (
            "⏳ Browser automation in progress...",
            Style::default().fg(Color::Yellow),
        )
    } else if !fields_valid {
        (
            "❌ Fill all fields to continue",
            Style::default().fg(Color::Red),
        )
    } else if !has_credentials {
        (
            "🔑 Add credentials below, then press Ctrl+Enter",
            Style::default().fg(Color::Yellow),
        )
    } else {
        (
            "▶️  Press Ctrl+Enter to start automation",
            Style::default().fg(Color::Green),
        )
    };

    let paragraph = Paragraph::new(button_text)
        .block(Block::default().borders(Borders::ALL).title("Actions"))
        .style(style)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Render the credentials input section
fn render_credentials_section(f: &mut Frame, area: Rect, state: &AutomationState) {
    let has_credentials = state.credentials.is_some();

    let lines = if has_credentials {
        vec![
            Line::from(vec![
                Span::raw("Username: "),
                Span::styled("✓ Provided", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("Password: "),
                Span::styled("✓ Provided", Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Press Delete to clear credentials",
                Style::default().fg(Color::Gray),
            )),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                "No credentials provided",
                Style::default().fg(Color::Red),
            )),
            Line::from(""),
            Line::from("Add credential input functionality:"),
            Line::from("• Username field"),
            Line::from("• Password field"),
            Line::from(""),
            Line::from(Span::styled(
                "For now, using hardcoded demo credentials",
                Style::default().fg(Color::Yellow),
            )),
        ]
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Login Credentials")
                .title_style(Style::default().fg(Color::Magenta)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Render the templates section
fn render_templates_section(f: &mut Frame, area: Rect, state: &AutomationState) {
    let items: Vec<ListItem> = state
        .templates
        .iter()
        .enumerate()
        .map(|(i, template)| {
            let hotkey = format!("Ctrl+{}", i + 1);
            let is_selected = state.selected_template == Some(i);

            let style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(format!("[{}] ", hotkey), Style::default().fg(Color::Gray)),
                Span::styled(&template.name, style),
            ]);

            ListItem::new(vec![
                line,
                Line::from(Span::styled(
                    format!("  {}", template.description),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Templates"))
        .style(Style::default());

    f.render_widget(list, area);
}

/// Render status information and action buttons
fn render_status_and_actions(f: &mut Frame, area: Rect, state: &AutomationState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    // Status info
    render_status_info(f, chunks[0], state);

    // Action buttons
    render_action_buttons(f, chunks[1], state);
}

/// Render status and information
fn render_status_info(f: &mut Frame, area: Rect, state: &AutomationState) {
    let mut status_lines = vec![Line::from(Span::styled(
        "Status:",
        Style::default().fg(Color::Cyan),
    ))];

    if let Some(template) = state.get_selected_template() {
        status_lines.push(Line::from(vec![
            Span::raw("Template: "),
            Span::styled(&template.name, Style::default().fg(Color::Green)),
        ]));
    } else {
        status_lines.push(Line::from(Span::styled(
            "No template selected",
            Style::default().fg(Color::Gray),
        )));
    }

    // Credentials status
    let cred_status = if state.credentials.is_some() {
        ("✓ Credentials set", Color::Green)
    } else {
        ("❌ No credentials", Color::Red)
    };
    status_lines.push(Line::from(vec![
        Span::raw("Credentials: "),
        Span::styled(cred_status.0, Style::default().fg(cred_status.1)),
    ]));

    status_lines.push(Line::from(vec![
        Span::raw("Website: "),
        Span::styled(&state.website_config.name, Style::default().fg(Color::Blue)),
    ]));

    let filled_fields = state
        .fields
        .iter()
        .filter(|f| !f.value.trim().is_empty())
        .count();
    status_lines.push(Line::from(vec![
        Span::raw("Progress: "),
        Span::styled(
            format!("{}/{} fields filled", filled_fields, state.fields.len()),
            if filled_fields == state.fields.len() {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            },
        ),
    ]));

    if state.is_running {
        status_lines.push(Line::from(Span::styled(
            "🤖 Browser automation in progress...",
            Style::default().fg(Color::Yellow),
        )));
    }

    let paragraph = Paragraph::new(status_lines)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Render a login popup when credentials are needed
pub fn render_login_popup(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 30, area);

    f.render_widget(Clear, popup_area);

    let popup = Paragraph::new(vec![
        Line::from("Login Required"),
        Line::from(""),
        Line::from("Username: [Enter username]"),
        Line::from("Password: [Enter password]"),
        Line::from(""),
        Line::from("Press Enter to continue, Esc to cancel"),
    ])
    .block(
        Block::default()
            .title("Authentication")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray)),
    )
    .style(Style::default().fg(Color::White))
    .wrap(Wrap { trim: true });

    f.render_widget(popup, popup_area);
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
