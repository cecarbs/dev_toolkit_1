use crate::app::{App, AppMode, FocusedPane};
use crate::ui::components::http_request_editor::render_http_request_editor;
use crate::ui::components::http_response_viewer::render_http_response_viewer;
use crate::ui::components::rename_dialog::render_rename_dialog;
use crate::ui::components::{
    get_mode_indicator, render_automation_form, render_collections_tree,
    render_delete_confirmation_dialog, render_folder_creation_dialog, render_help_dialog,
    render_import_dialog, render_logging_panel, render_login_popup, render_status_line,
    render_template_creation_dialog,
};
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
            Constraint::Length(3), // Status line
        ])
        .split(size);

    // Render header
    render_header(f, main_chunks[0], app);

    // Render main content area
    render_main_content(f, main_chunks[1], app);

    // Render status line
    render_status_line(f, main_chunks[2], app);

    // Render modal dialogs (in order of priority - delete confirmation has the highest priority)
    if app.show_help_dialog {
        render_help_dialog(f, size, app);
    } else if app.show_import_dialog {
        render_import_dialog(f, size, app);
    } else if app.show_delete_confirmation_dialog {
        render_delete_confirmation_dialog(f, size, app);
    } else if app.show_login_popup {
        render_login_popup(f, size, app);
    } else if app.show_template_dialog {
        render_template_creation_dialog(f, size, app);
    } else if app.show_folder_dialog {
        render_folder_creation_dialog(f, size, app);
    } else if app.show_rename_dialog {
        render_rename_dialog(f, size, app);
    }
}

/// Render the header with mode tabs and indicators
fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let mode_titles = vec![
        format!(
            "Automation (F1){}",
            if app.current_mode == AppMode::Automation {
                get_mode_indicator(app)
            } else {
                String::new()
            }
        ),
        "HTTP Client (F4)".to_string(),
    ];
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

/// Get mode indicator for HTTP mode
fn get_http_mode_indicator(app: &App) -> String {
    let method = &app.http_state.current_request.method;
    let is_valid = app.http_state.is_valid();
    let is_sending = app.http_state.is_sending;

    if is_sending {
        " [SENDING]".to_string()
    } else if app.focused_pane == FocusedPane::Form {
        match app.input_mode {
            crate::app::InputMode::Normal => format!(" [{}]", method.as_str()),
            crate::app::InputMode::Edit => format!(" [EDIT {}]", method.as_str()),
        }
    } else if !is_valid {
        " [INVALID]".to_string()
    } else {
        format!(" [{}]", method.as_str())
    }
}

/// Main content layout controller - always show 3-pane layout with mode-specific content
fn render_main_content(f: &mut Frame, area: Rect, app: &App) {
    // Always use 3-pane layout: Collections (left) | Content (top-right) | Logs/Response (bottom-right)
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Collections (left)
            Constraint::Percentage(75), // Content+Logs/Response (right)
        ])
        .split(area);

    // Collections tree on the left (same for both modes)
    render_collections_tree(f, horizontal_chunks[0], app);

    // Split right side based on current mode
    match app.current_mode {
        AppMode::Automation => {
            // Automation: Form (top) | Logs (bottom)
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(60), // Form (top)
                    Constraint::Percentage(40), // Logs (bottom)
                ])
                .split(horizontal_chunks[1]);

            render_automation_form(
                f,
                vertical_chunks[0],
                &app.automation_state,
                &app.auth_service,
                app,
            );

            render_logging_panel(f, vertical_chunks[1], app);
        }
        AppMode::Http => {
            // HTTP: Request Editor (top) | Response Viewer (bottom)
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(60), // Request Editor (top)
                    Constraint::Percentage(40), // Response Viewer (bottom)
                ])
                .split(horizontal_chunks[1]);

            render_http_request_editor(f, vertical_chunks[0], &app.http_state, app);
            render_http_response_viewer(f, vertical_chunks[1], &app.http_state, app);
        }
    }
}
// TODO: old version
/// Main content layout controller - always show 3-pane layout
// fn render_main_content(f: &mut Frame, area: Rect, app: &App) {
//     // Always use 3-pane layout: Collections (left) | Form (top-right) | Logs (bottom-right)
//     let horizontal_chunks = Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints([
//             Constraint::Percentage(25), // Collections (left)
//             Constraint::Percentage(75), // Form+Logs (right)
//         ])
//         .split(area);
//
//     // Collections tree on the left
//     render_collections_tree(f, horizontal_chunks[0], app);
//
//     // Split right side: Form (top) | Logs (bottom)
//     let vertical_chunks = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Percentage(60), // Form (top)
//             Constraint::Percentage(40), // Logs (bottom)
//         ])
//         .split(horizontal_chunks[1]);
//
//     // Form in top-right
//     match app.current_mode {
//         AppMode::Automation => {
//             render_automation_form(
//                 f,
//                 vertical_chunks[0],
//                 &app.automation_state,
//                 &app.auth_service,
//                 app,
//             );
//         }
//         AppMode::Http => {
//             render_http_placeholder(f, vertical_chunks[0]);
//         }
//     }
//
//     // Logs in bottom-right (always visible now)
//     render_logging_panel(f, vertical_chunks[1], app);
// }
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
