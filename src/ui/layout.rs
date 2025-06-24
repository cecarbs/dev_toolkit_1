use crate::app::{App, AppMode, FocusedPane};
use crate::ui::components::{render_automation_form, render_logging_panel, render_login_popup};
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

    // Main layout: Header → Content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main content
        ])
        .split(size);

    // Render header
    render_header(f, main_chunks[0], app);

    // Render main content area
    render_main_content(f, main_chunks[1], app);

    // Render login popup if shown (modal overlay)
    if app.show_login_popup {
        render_login_popup(f, size, app);
    }
}

/// Render the header with mode tabs
fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let mode_titles = vec!["Automation (F1)", "HTTP Client (F4)"];
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

/// THIS IS THE MISSING FUNCTION - Controls the main layout
fn render_main_content(f: &mut Frame, area: Rect, app: &App) {
    if app.show_logs {
        // When logs are shown: Collections (left) | Form (top-right) | Logs (bottom-right)
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Collections (left)
                Constraint::Percentage(75), // Form+Logs (right)
            ])
            .split(area);

        // Collections on the left
        render_collections_panel(f, horizontal_chunks[0], app);

        // Split right side: Form (top) | Logs (bottom)
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Form (top)
                Constraint::Percentage(40), // Logs (bottom)
            ])
            .split(horizontal_chunks[1]);

        // Form in top-right
        match app.current_mode {
            AppMode::Automation => {
                render_automation_form(
                    f,
                    vertical_chunks[0],
                    &app.automation_state,
                    &app.auth_service,
                );
            }
            AppMode::Http => {
                render_http_placeholder(f, vertical_chunks[0]);
            }
        }

        // Logs in bottom-right
        render_logging_panel(f, vertical_chunks[1], app);
    } else {
        // When logs are hidden: Collections (left) | Form (right)
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Collections (left)
                Constraint::Percentage(75), // Form (right)
            ])
            .split(area);

        // Collections on the left
        render_collections_panel(f, horizontal_chunks[0], app);

        // Form on the right
        match app.current_mode {
            AppMode::Automation => {
                render_automation_form(
                    f,
                    horizontal_chunks[1],
                    &app.automation_state,
                    &app.auth_service,
                );
            }
            AppMode::Http => {
                render_http_placeholder(f, horizontal_chunks[1]);
            }
        }
    }
}

/// Render the collections panel (left side)
fn render_collections_panel(f: &mut Frame, area: Rect, app: &App) {
    // Focus indicator
    let border_style = if app.focused_pane == FocusedPane::Collections {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::White)
    };

    let collections_text = vec![
        Line::from(Span::styled(
            "📁 Collections",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from("📂 Customer"),
        Line::from("  📂 Add"),
        Line::from("    📄 Email"),
        Line::from("    📄 Phone"),
        Line::from("  📂 Update"),
        Line::from("    📄 Profile"),
        Line::from(""),
        Line::from("📂 Reports"),
        Line::from("  📄 Daily Summary"),
        Line::from("  📄 Weekly Report"),
        Line::from(""),
        Line::from(Span::styled(
            "F5: Focus this pane",
            Style::default().fg(Color::Gray),
        )),
    ];

    let collections_panel = Paragraph::new(collections_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Collections")
            .title_style(Style::default().fg(Color::Green))
            .border_style(border_style),
    );

    f.render_widget(collections_panel, area);
}

/// Placeholder for HTTP mode
fn render_http_placeholder(f: &mut Frame, area: Rect) {
    let placeholder_text = vec![
        Line::from("🚧 HTTP Client Mode"),
        Line::from(""),
        Line::from("This mode will include:"),
        Line::from("• HTTP request builder"),
        Line::from("• Request/Response viewer"),
        Line::from("• Collection management"),
        Line::from(""),
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
        focused_border: Color::Blue,
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
    pub focused_border: Color,
}
