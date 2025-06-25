use crate::app::{App, FocusedPane, InputMode};
use crate::models::{FieldType, FormField};
use crate::modes::automation::AutomationState;
use crate::services::AuthService;
use ratatui::layout::{Margin, Position};
use ratatui::style::Modifier;
use ratatui::widgets::BorderType;
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
    app: &App,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Form fields area (takes most space)
            Constraint::Length(6), // Send button and auth status
        ])
        .split(area);

    // Render form fields
    render_form_fields(f, chunks[0], state, app);

    // Render send button and auth status
    render_send_section(f, chunks[1], state, auth_service);
    //
    // // Set cursor position when in edit mode
    // if app.input_mode == InputMode::Edit && app.focused_pane == FocusedPane::Form {
    //     if let Some(focused_field) = state.get_focused_field() {
    //         // Calculate cursor position on screen
    //         // This is a simplified version - you might need to adjust based on your exact layout
    //         let field_y = area.y + 2 + state.focused_field as u16; // Adjust based on your layout
    //         let field_x = area.x + 20 + app.form_field_cursor_index as u16; // 20 chars for label + ": "
    //
    //         f.set_cursor_position(Position::new(field_x, field_y));
    //     }
    // }
}

/// Render the form fields with proper focus indicators and cursor positioning
fn render_form_fields(f: &mut Frame, area: Rect, state: &AutomationState, app: &App) {
    // Get border style based on focus
    let is_focused = app.focused_pane == FocusedPane::Form;
    let border_style = if is_focused {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::White)
    };
    let title_style = Style::default().fg(Color::Green);

    let field_items: Vec<ListItem> = state
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| render_field_item(field, i, state.focused_field, app))
        .collect();

    // Dynamic title based on focus and mode
    let mode_indicator = if is_focused {
        match app.input_mode {
            InputMode::Normal => " [NORMAL]",
            InputMode::Edit => " [EDIT]",
        }
    } else {
        ""
    };

    let title = format!("Form Fields{}", mode_indicator);

    let list = List::new(field_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(title_style)
            .border_style(border_style),
    );

    f.render_widget(list, area);

    // Set cursor position when in edit mode
    if app.input_mode == InputMode::Edit && app.focused_pane == FocusedPane::Form {
        if let Some(_focused_field) = state.get_focused_field() {
            // Calculate cursor position on screen
            let list_inner = area.inner(Margin {
                horizontal: 1,
                vertical: 1,
            });

            // Y position: list top + focused field index
            let cursor_y = list_inner.y + state.focused_field as u16;

            // X position: list left + label width + ": " + cursor index in field
            let label_width = 18; // From your format string "{:18}"
            let cursor_x = list_inner.x + label_width + 2 + app.form_field_cursor_index as u16;

            f.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }
}

fn render_field_item<'a>(
    field: &'a FormField,
    index: usize,
    focused_index: usize,
    app: &App,
) -> ListItem<'a> {
    let is_focused = index == focused_index;
    let is_editing = is_focused && app.input_mode == InputMode::Edit;
    let is_valid = field.is_valid();

    // Display value and style
    let (display_value, value_style) = if is_editing {
        // Show current value with editing indicator
        (
            if field.value.is_empty() {
                "<Editing...>".to_string()
            } else {
                field.value.clone()
            },
            Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        )
    } else {
        match &field.field_type {
            FieldType::Select => {
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
        }
    };

    // Label style with validation
    let label_style = if !is_valid && field.is_required {
        Style::default().fg(Color::Red)
    } else if is_editing {
        Style::default().fg(Color::Yellow)
    } else if is_focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Gray)
    };

    // Mode indicator
    // let mode_indicator = if is_editing {
    //     " [EDIT]"
    // } else if is_focused {
    //     " [SELECTED]"
    // } else {
    //     ""
    // };

    // In your field rendering, add a blinking indicator when editing
    let mode_indicator = if is_editing {
        Span::styled(" ", Style::default().add_modifier(Modifier::RAPID_BLINK))
    } else if is_focused {
        Span::styled(" ◀", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("")
    };

    // Background style for focused field
    let background_style = if is_editing {
        Style::default().bg(Color::DarkGray)
    } else if is_focused {
        Style::default().bg(Color::Blue)
    } else {
        Style::default()
    };

    let line = Line::from(vec![
        Span::styled(format!("{:18}", field.get_display_label()), label_style),
        Span::raw(": "),
        Span::styled(display_value, value_style.patch(background_style)),
        mode_indicator,
    ]);

    ListItem::new(line)
}
/// Render a single form field item with appropriate input type
// fn render_field_item<'a>(
//     field: &'a FormField,
//     index: usize,
//     focused_index: usize,
//     app: &App,
// ) -> ListItem<'a> {
//     let is_focused = index == focused_index;
//     let is_valid = field.is_valid();
//     // let is_editing = is_focused && app.form_ed
//
//     // Determine field display value and style
//     let (display_value, value_style) = match &field.field_type {
//         crate::models::FieldType::Select => {
//             if field.value.is_empty() {
//                 (
//                     "<Select Option>".to_string(),
//                     Style::default().fg(Color::DarkGray),
//                 )
//             } else {
//                 (field.value.clone(), Style::default().fg(Color::Cyan))
//             }
//         }
//         _ => {
//             if field.value.is_empty() {
//                 ("<Empty>".to_string(), Style::default().fg(Color::DarkGray))
//             } else {
//                 (field.value.clone(), Style::default().fg(Color::White))
//             }
//         }
//     };
//
//     // Create the field label with validation indicator
//     let label_style = if !is_valid && field.is_required {
//         Style::default().fg(Color::Red)
//     } else {
//         Style::default().fg(Color::Green)
//     };
//
//     let validation_indicator = if field.is_required && !is_valid {
//         " ❌"
//     } else if is_valid {
//         " ✓"
//     } else {
//         ""
//     };
//
//     // Field type indicator
//     let type_indicator = match field.field_type {
//         crate::models::FieldType::Select => " 📋",
//         crate::models::FieldType::Textarea => " 📝",
//         crate::models::FieldType::Email => " ✉️",
//         _ => " 📄",
//     };
//
//     // Background style for focused field
//     let background_style = if is_focused {
//         Style::default().bg(Color::DarkGray)
//     } else {
//         Style::default()
//     };
//
//     let line = Line::from(vec![
//         Span::styled(format!("{:18}", field.get_display_label()), label_style),
//         Span::raw(": "),
//         Span::styled(display_value, value_style.patch(background_style)),
//         Span::styled(type_indicator, Style::default().fg(Color::Blue)),
//         Span::styled(validation_indicator, Style::default()),
//     ]);
//
//     // Add dropdown options if it's a select field and focused
//     if is_focused && matches!(field.field_type, crate::models::FieldType::Select) {
//         let options = field.get_dropdown_options();
//         let options_text = if options.is_empty() {
//             "No options available".to_string()
//         } else {
//             format!("Press 1-{}: {}", options.len(), options.join(", "))
//         };
//
//         let options_line = Line::from(vec![
//             Span::raw("                    "),
//             Span::styled(options_text, Style::default().fg(Color::Gray)),
//         ]);
//
//         ListItem::new(vec![line, options_line])
//     } else {
//         ListItem::new(line)
//     }
// }

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

/// Render login popup modal with proper focus indicators
pub fn render_login_popup(f: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(50, 40, area);

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Username field
            Constraint::Length(3), // Password field
            Constraint::Length(3), // Error message (if any)
            Constraint::Length(3), // Instructions
        ])
        .split(popup_area);

    // Title
    let title = Paragraph::new("🔐 Login Required")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Authentication")
                .title_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::DarkGray))
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(title, chunks[0]);

    // Username field with focus indicator
    let username_text = if app.login_username.is_empty() {
        "Enter username..."
    } else {
        &app.login_username
    };

    let username_focused = app.login_focused_field == 0;
    let username_style = if username_focused {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let username_title = if username_focused {
        "Username [FOCUSED]"
    } else {
        "Username"
    };

    let username = Paragraph::new(username_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(username_title)
                .title_style(if username_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                })
                .style(Style::default().bg(Color::DarkGray))
                .border_style(if username_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }),
        )
        .style(username_style);
    f.render_widget(username, chunks[1]);

    // Password field with focus indicator (masked)
    let password_display = "*".repeat(app.login_password.len());
    let password_text = if app.login_password.is_empty() {
        "Enter password..."
    } else {
        &password_display
    };

    let password_focused = app.login_focused_field == 1;
    let password_style = if password_focused {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let password_title = if password_focused {
        "Password [FOCUSED]"
    } else {
        "Password"
    };

    let password = Paragraph::new(password_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(password_title)
                .title_style(if password_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                })
                .style(Style::default().bg(Color::DarkGray))
                .border_style(if password_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }),
        )
        .style(password_style);
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
    } else {
        // Show helpful tip when no error
        let tip = Paragraph::new("Demo: any username/password with 3+ characters")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Tip")
                    .style(Style::default().bg(Color::DarkGray)),
            )
            .style(Style::default().fg(Color::Gray));
        f.render_widget(tip, chunks[3]);
    }

    // Instructions
    let instructions = vec![Line::from(vec![
        Span::styled("Tab/↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(": Navigate fields  "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(": Login  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(": Cancel"),
    ])];

    let help = Paragraph::new(instructions)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Controls")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(help, chunks[4]);
}
/// Render login popup modal
// pub fn render_login_popup(f: &mut Frame, area: Rect, app: &crate::app::App) {
//     let popup_area = centered_rect(50, 40, area);
//
//     f.render_widget(Clear, popup_area);
//
//     let chunks = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Length(3), // Title
//             Constraint::Length(3), // Username field
//             Constraint::Length(3), // Password field
//             Constraint::Length(3), // Error message (if any)
//             Constraint::Length(3), // Buttons
//         ])
//         .split(popup_area);
//
//     // Title
//     let title = Paragraph::new("🔐 Login Required")
//         .block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .title("Authentication")
//                 .title_style(Style::default().fg(Color::Cyan))
//                 .style(Style::default().bg(Color::DarkGray)),
//         )
//         .style(Style::default().fg(Color::White));
//     f.render_widget(title, chunks[0]);
//
//     // Username field
//     let username_text = if app.login_username.is_empty() {
//         "Enter username..."
//     } else {
//         &app.login_username
//     };
//     let username = Paragraph::new(username_text)
//         .block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .title("Username")
//                 .style(Style::default().bg(Color::DarkGray)),
//         )
//         .style(Style::default().fg(Color::White));
//     f.render_widget(username, chunks[1]);
//
//     // Password field (masked)
//     let password_display = "*".repeat(app.login_password.len());
//     let password_text = if app.login_password.is_empty() {
//         "Enter password..."
//     } else {
//         &password_display
//     };
//     let password = Paragraph::new(password_text)
//         .block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .title("Password")
//                 .style(Style::default().bg(Color::DarkGray)),
//         )
//         .style(Style::default().fg(Color::White));
//     f.render_widget(password, chunks[2]);
//
//     // Error message
//     if let Some(error) = &app.login_error {
//         let error_msg = Paragraph::new(error.as_str())
//             .block(
//                 Block::default()
//                     .borders(Borders::ALL)
//                     .title("Error")
//                     .style(Style::default().bg(Color::DarkGray)),
//             )
//             .style(Style::default().fg(Color::Red));
//         f.render_widget(error_msg, chunks[3]);
//     }
//
//     // Instructions
//     let instructions = Paragraph::new("Enter to login, Esc to cancel")
//         .block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .title("Controls")
//                 .style(Style::default().bg(Color::DarkGray)),
//         )
//         .style(Style::default().fg(Color::Gray));
//     f.render_widget(instructions, chunks[4]);
// }

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
