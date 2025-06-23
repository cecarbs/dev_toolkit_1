use crate::app::{App, AppMode};
use crate::events::get_help_text;
use crate::ui::components::{render_automation_form, render_log_summary, render_logging_panel};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

/// Render the main application layout
pub fn render_app(f: &mut Frame, app: &App) {
    let size = f.area();

    // Create the main layout with header, content, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                                 // Header with mode tabs
            Constraint::Min(10),                                   // Main content area
            Constraint::Length(if app.show_logs { 8 } else { 3 }), // Footer/logs area
        ])
        .split(size);

    // Render header with mode tabs
    render_header(f, chunks[0], app);

    // Render main content based on current mode
    if app.show_logs {
        // Split main area between content and logs when logging panel is open
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(chunks[1]);

        render_mode_content(f, content_chunks[0], app);
        render_logging_panel(f, content_chunks[1], app);
    } else {
        render_mode_content(f, chunks[1], app);
    }

    // Render footer
    render_footer(f, chunks[2], app);
}

/// Render the header with mode tabs
fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let mode_titles = vec!["Automation (F1)", "HTTP Client (F2)"];
    let selected_tab = match app.current_mode {
        AppMode::Automation => 0,
        AppMode::Http => 1,
    };

    let tabs = Tabs::new(mode_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🛠️  Developer Toolkit")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
        .select(selected_tab);

    f.render_widget(tabs, area);
}

/// Render the main content area based on the current mode
fn render_mode_content(f: &mut Frame, area: Rect, app: &App) {
    match app.current_mode {
        AppMode::Automation => {
            render_automation_form(f, area, &app.automation_state);
        }
        AppMode::Http => {
            render_http_placeholder(f, area);
        }
    }
}

/// Placeholder for HTTP mode (to be implemented later)
fn render_http_placeholder(f: &mut Frame, area: Rect) {
    let placeholder_text = vec![
        Line::from("🚧 HTTP Client Mode"),
        Line::from(""),
        Line::from("This mode will include:"),
        Line::from("• HTTP request builder"),
        Line::from("• Request/Response viewer"),
        Line::from("• Collection management"),
        Line::from("• Environment variables"),
        Line::from(""),
        Line::from("Focus on Automation mode for now!"),
        Line::from("Press F1 to switch back to Automation mode."),
    ];

    let paragraph = Paragraph::new(placeholder_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("HTTP Client Mode (Coming Soon)")
                .title_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(paragraph, area);
}

/// Render the footer with status information
fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    if app.show_logs {
        // When logs are open, show a compact summary in the footer
        render_footer_with_log_summary(f, area, app);
    } else {
        // When logs are closed, show help text and status
        render_footer_with_help(f, area, app);
    }
}

/// Render footer when logging panel is open
fn render_footer_with_log_summary(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Status on the left
    render_status_line(f, chunks[0], app);

    // Log summary on the right
    render_log_summary(f, chunks[1], app);
}

/// Render footer with help text when logging panel is closed
fn render_footer_with_help(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Status on the left
    render_status_line(f, chunks[0], app);

    // Help text on the right
    render_help_section(f, chunks[1], app);
}

/// Render the status line with current mode and key info
fn render_status_line(f: &mut Frame, area: Rect, app: &App) {
    let status_text = match app.current_mode {
        AppMode::Automation => {
            let template_info = if let Some(template) = app.automation_state.get_selected_template()
            {
                format!(" | Template: {}", template.name)
            } else {
                " | No template".to_string()
            };

            let running_info = if app.automation_state.is_running {
                " | 🤖 Running"
            } else {
                ""
            };

            format!("Mode: Automation{}{}", template_info, running_info)
        }
        AppMode::Http => "Mode: HTTP Client (Not implemented)".to_string(),
    };

    let logs_info = format!(" | Logs: {} entries", app.log_entries.len());
    let quit_info = " | Ctrl+Q: Quit";

    let full_status = format!("{}{}{}", status_text, logs_info, quit_info);

    let paragraph = Paragraph::new(full_status)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Status")
                .title_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

/// Render the help section with key bindings
fn render_help_section(f: &mut Frame, area: Rect, app: &App) {
    let help_lines = get_help_text(app);

    // Take only the first few lines to fit in the footer
    let display_lines: Vec<Line> = help_lines
        .iter()
        .take(3)
        .map(|line| Line::from(line.as_str()))
        .collect();

    let paragraph = Paragraph::new(display_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Key Bindings")
                .title_style(Style::default().fg(Color::Magenta)),
        )
        .style(Style::default().fg(Color::Gray));

    f.render_widget(paragraph, area);
}

/// Calculate layout for different screen sizes
pub fn calculate_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Footer
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2])
}

/// Helper function to get theme colors
pub fn get_theme_colors() -> ThemeColors {
    ThemeColors {
        primary: Color::Cyan,
        secondary: Color::Green,
        accent: Color::Yellow,
        error: Color::Red,
        warning: Color::Yellow,
        success: Color::Green,
        text: Color::White,
        text_dim: Color::Gray,
        background: Color::Black,
        border: Color::White,
    }
}

/// Theme color definitions
pub struct ThemeColors {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub text: Color,
    pub text_dim: Color,
    pub background: Color,
    pub border: Color,
}
