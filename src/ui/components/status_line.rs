use crate::app::{App, AppMode, FocusedPane, InputMode};
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

/// Get context-aware status text based on focused pane, mode, and current app state
fn get_contextual_status_text(app: &App) -> Line<'static> {
    // Helper function to create colored spans
    let key = |text: &str| Span::styled(String::from(text), Style::default().fg(Color::Yellow));
    let desc = |text: &str| Span::styled(String::from(text), Style::default().fg(Color::White));
    let separator = || Span::styled(String::from(" │ "), Style::default().fg(Color::DarkGray));

    match (&app.current_mode, &app.focused_pane, &app.input_mode) {
        // ========== AUTOMATION MODE ==========
        (AppMode::Automation, FocusedPane::Collections, _) => Line::from(vec![
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
            key("F4"),
            desc(":HTTP mode"),
        ]),

        (AppMode::Automation, FocusedPane::Form, InputMode::Normal) => Line::from(vec![
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
            key("F4"),
            desc(":HTTP mode"),
        ]),

        (AppMode::Automation, FocusedPane::Form, InputMode::Edit) => Line::from(vec![
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
            key("F3"),
            desc(":run automation"),
        ]),

        (AppMode::Automation, FocusedPane::Logs, _) => {
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
                    key("F4"),
                    desc(":HTTP mode"),
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
                    key("F4"),
                    desc(":HTTP mode"),
                ])
            }
        }

        // ========== HTTP CLIENT MODE ==========
        // UPDATE the HTTP collections tree status line
        (AppMode::Http, FocusedPane::Collections, _) => Line::from(vec![
            key("↑/↓"),
            desc(":navigate"),
            separator(),
            key("Enter"),
            desc(":load request"),
            separator(),
            key("Space"),
            desc(":toggle folder"),
            separator(),
            key("Ctrl+N"),
            desc(":new request"),
            separator(),
            key("Ctrl+F"),
            desc(":new folder"),
            separator(),
            key("Ctrl+I"),
            desc(":import collection"), // NEW
            separator(),
            key("F8"),
            desc(":import"), // NEW - Alternative shortcut
            separator(),
            key("F2/R"),
            desc(":rename"),
            separator(),
            key("Del"),
            desc(":delete"),
            separator(),
            key("F1"),
            desc(":automation mode"),
        ]),

        (AppMode::Http, FocusedPane::Form, InputMode::Normal) => {
            let method = &app.http_state.current_request.method.as_str();
            Line::from(vec![
                key("i/Enter"),
                desc(":edit URL"),
                separator(),
                key("m"),
                desc(&format!(":cycle method ({})", method)),
                separator(),
                key("Tab"),
                desc(":next tab"),
                separator(),
                key("1-4"),
                desc(":quick methods"),
                separator(),
                key("Space/F3"),
                desc(":send request"),
                separator(),
                key("H/L"),
                desc(":switch pane"),
                separator(),
                key("F1"),
                desc(":automation"),
            ])
        }

        (AppMode::Http, FocusedPane::Form, InputMode::Edit) => Line::from(vec![
            key("Esc"),
            desc(":normal mode"),
            separator(),
            key("Type"),
            desc(":edit URL"),
            separator(),
            key("Tab"),
            desc(":next tab"),
            separator(),
            key("F3"),
            desc(":send request"),
            separator(),
            key("Ctrl+N"),
            desc(":save request"),
            separator(),
            key("Del"),
            desc(":clear field"),
        ]),

        (AppMode::Http, FocusedPane::Logs, _) => {
            // In HTTP mode, this is the Response viewer
            if let Some(response) = &app.http_state.last_response {
                let status_code = response.status_code;
                Line::from(vec![
                    key("Tab"),
                    desc(&format!(":response tabs ({})", status_code)),
                    separator(),
                    key("Ctrl+C"),
                    desc(":copy response"),
                    separator(),
                    key("j/k"),
                    desc(":scroll content"),
                    separator(),
                    key("g/G"),
                    desc(":top/bottom"),
                    separator(),
                    key("Del"),
                    desc(":clear response"),
                    separator(),
                    key("H/L"),
                    desc(":switch pane"),
                ])
            } else if app.http_state.is_sending {
                Line::from(vec![
                    key("⏳"),
                    desc("Sending request..."),
                    separator(),
                    key("H/L"),
                    desc(":switch pane"),
                    separator(),
                    key("F1"),
                    desc(":automation mode"),
                ])
            } else {
                Line::from(vec![
                    key("Space/F3"),
                    desc(":send request"),
                    separator(),
                    key("H"),
                    desc(":back to request"),
                    separator(),
                    key("F1"),
                    desc(":automation mode"),
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
    match (&app.current_mode, &app.focused_pane, &app.input_mode) {
        // Automation mode compact status
        (AppMode::Automation, FocusedPane::Collections, _) => {
            "↑/↓:nav Enter:select Ctrl+N:new F4:HTTP ?:help".to_string()
        }
        (AppMode::Automation, FocusedPane::Form, InputMode::Normal) => {
            "i:edit j/k:nav Tab:next Ctrl+N:save F3:run F4:HTTP".to_string()
        }
        (AppMode::Automation, FocusedPane::Form, InputMode::Edit) => {
            "Esc:normal ←/→:cursor Tab:next F3:run".to_string()
        }
        (AppMode::Automation, FocusedPane::Logs, _) => {
            if app.log_search_mode {
                "Type:search Esc:exit j/k:scroll F4:HTTP".to_string()
            } else {
                "/:search j/k:scroll g/G:top/bottom F4:HTTP".to_string()
            }
        }

        // HTTP mode compact status
        (AppMode::Http, FocusedPane::Collections, _) => {
            "↑/↓:nav Enter:load Ctrl+N:new F1:auto ?:help".to_string()
        }
        (AppMode::Http, FocusedPane::Form, InputMode::Normal) => {
            let method = app.http_state.current_request.method.as_str();
            format!("i:edit m:{} Tab:tabs F3:send F1:auto", method)
        }
        (AppMode::Http, FocusedPane::Form, InputMode::Edit) => {
            "Esc:normal Type:URL F3:send Del:clear".to_string()
        }
        (AppMode::Http, FocusedPane::Logs, _) => {
            if app.http_state.last_response.is_some() {
                "Tab:tabs Ctrl+C:copy j/k:scroll Del:clear".to_string()
            } else if app.http_state.is_sending {
                "⏳ Sending request...".to_string()
            } else {
                "F3:send H:request F1:auto ?:help".to_string()
            }
        }
    }
}

/// Get mode indicator for the focused pane title
pub fn get_mode_indicator(app: &App) -> String {
    match app.current_mode {
        AppMode::Automation => {
            if app.focused_pane == FocusedPane::Form {
                match app.input_mode {
                    InputMode::Normal => " [NORMAL]".to_string(),
                    InputMode::Edit => " [EDIT]".to_string(),
                }
            } else {
                String::new()
            }
        }
        AppMode::Http => get_http_mode_indicator(app),
    }
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
            InputMode::Normal => format!(" [{}]", method.as_str()),
            InputMode::Edit => format!(" [EDIT {}]", method.as_str()),
        }
    } else if !is_valid {
        " [INVALID]".to_string()
    } else {
        format!(" [{}]", method.as_str())
    }
}
