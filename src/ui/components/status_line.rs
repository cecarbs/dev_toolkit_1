use crate::app::{App, FocusedPane, InputMode};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Render the contextual status line showing relevant keybindings
pub fn render_status_line(f: &mut Frame, area: Rect, app: &App) {
    let status_text = get_contextual_status_text(app);

    let status_paragraph = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Keybindings")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(status_paragraph, area);
}

/// Get context-aware status text based on focused pane and mode
fn get_contextual_status_text(app: &App) -> Line<'static> {
    // Helper function to create colored spans
    let key = |text: &str| Span::styled(String::from(text), Style::default().fg(Color::Yellow));
    let desc = |text: &str| Span::styled(String::from(text), Style::default().fg(Color::White));
    let separator = || Span::styled(String::from(" │ "), Style::default().fg(Color::DarkGray));

    match (&app.focused_pane, &app.input_mode) {
        // Collections tree focused
        (FocusedPane::Collections, _) => Line::from(vec![
            key("↑/↓"),
            desc(":navigate"),
            separator(),
            key("Enter"),
            desc(":load/expand"),
            separator(),
            key("Space"),
            desc(":toggle"),
            separator(),
            key("Ctrl+N"),
            desc(":new template"),
            separator(),
            key("Ctrl+F"),
            desc(":new folder"),
            separator(),
            key("F2/R"),
            desc(":rename"),
            separator(),
            key("Del"),
            desc(":delete"),
            separator(),
            key("?"),
            desc(":help"),
        ]),

        // Form in normal mode
        (FocusedPane::Form, InputMode::Normal) => Line::from(vec![
            key("i/Enter"),
            desc(":edit field"),
            separator(),
            key("j/k"),
            desc(":navigate fields"),
            separator(),
            key("Tab"),
            desc(":next field"),
            separator(),
            key("Ctrl+N"),
            desc(":save template"),
            separator(),
            key("H/L"),
            desc(":switch pane"),
            separator(),
            key("F3"),
            desc(":run automation"),
            separator(),
            key("?"),
            desc(":help"),
        ]),

        // Form in edit mode
        (FocusedPane::Form, InputMode::Edit) => Line::from(vec![
            key("Esc"),
            desc(":normal mode"),
            separator(),
            key("←/→"),
            desc(":move cursor"),
            separator(),
            key("Tab"),
            desc(":next field"),
            separator(),
            key("Home/End"),
            desc(":start/end"),
            separator(),
            key("Ctrl+N"),
            desc(":save template"),
            separator(),
            key("?"),
            desc(":help"),
        ]),

        // Logs focused - NEW AND IMPROVED
        (FocusedPane::Logs, _) => {
            if app.log_search_mode {
                Line::from(vec![
                    key("Type"),
                    desc(":search"),
                    separator(),
                    key("Esc"),
                    desc(":exit search"),
                    separator(),
                    key("j/k"),
                    desc(":scroll"),
                    separator(),
                    key("g/G"),
                    desc(":top/bottom"),
                    separator(),
                    key("H/L"),
                    desc(":switch pane"),
                    separator(),
                    key("?"),
                    desc(":help"),
                ])
            } else {
                Line::from(vec![
                    key("/"),
                    desc(":search"),
                    separator(),
                    key("j/k"),
                    desc(":scroll"),
                    separator(),
                    key("Ctrl+U/D"),
                    desc(":page up/down"),
                    separator(),
                    key("g/G"),
                    desc(":top/bottom"),
                    separator(),
                    key("H/L"),
                    desc(":switch pane"),
                    separator(),
                    key("?"),
                    desc(":help"),
                ])
            }
        }
    }
}
/// Get a compact status text for very narrow screens
pub fn get_compact_status_text(app: &App) -> String {
    match (&app.focused_pane, &app.input_mode) {
        (FocusedPane::Collections, _) => "↑/↓:nav Enter:select Ctrl+N:new ?:help".to_string(),
        (FocusedPane::Form, InputMode::Normal) => {
            "i:edit j/k:nav Tab:next Ctrl+N:save ?:help".to_string()
        }
        (FocusedPane::Form, InputMode::Edit) => "Esc:normal ←/→:cursor Tab:next ?:help".to_string(),
        (FocusedPane::Logs, _) => {
            if app.log_search_mode {
                "Type:search Esc:exit j/k:scroll ?:help".to_string()
            } else {
                "/:search j/k:scroll g/G:top/bottom ?:help".to_string()
            }
        }
    }
}

/// Get mode indicator for the focused pane title
pub fn get_mode_indicator(app: &App) -> String {
    if app.focused_pane == FocusedPane::Form {
        match app.input_mode {
            InputMode::Normal => " [NORMAL]".to_string(),
            InputMode::Edit => " [EDIT]".to_string(),
        }
    } else {
        String::new()
    }
}
