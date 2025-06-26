use crate::{
    app::{App, FocusedPane, InputMode},
    models::{
        http::{BodyContentType, HttpRequestTab, HttpState},
        http_client::{HttpAuth, HttpRequestBody},
    },
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
};

/// Render the HTTP request editor with method, URL, and tabbed content
pub fn render_http_request_editor(f: &mut Frame, area: Rect, state: &HttpState, app: &App) {
    let is_focused = app.focused_pane == FocusedPane::Form;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Method and URL bar
            Constraint::Min(5),    // Tabbed content
        ])
        .split(area);

    // Render method and URL bar
    render_method_url_bar(f, chunks[0], state, app, is_focused);

    // Render tabbed content
    render_request_tabs(f, chunks[1], state, app, is_focused);
}

/// Render the method selector and URL input bar
fn render_method_url_bar(
    f: &mut Frame,
    area: Rect,
    state: &HttpState,
    app: &App,
    is_focused: bool,
) {
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // Method selector
            Constraint::Min(20),    // URL input
            Constraint::Length(12), // Send button
        ])
        .split(area);

    // Method selector
    render_method_selector(f, horizontal_chunks[0], state, is_focused);

    // URL input
    render_url_input(f, horizontal_chunks[1], state, app, is_focused);

    // Send button
    render_send_button(f, horizontal_chunks[2], state, is_focused);
}

/// Render method selector dropdown
fn render_method_selector(f: &mut Frame, area: Rect, state: &HttpState, is_focused: bool) {
    let method_style = if is_focused {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let method_text = vec![Line::from(vec![
        Span::styled(state.current_request.method.as_str(), method_style),
        Span::raw(" ▼"),
    ])];

    let method_widget = Paragraph::new(method_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Method")
            .title_style(Style::default().fg(Color::Green))
            .border_style(if is_focused {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::White)
            }),
    );

    f.render_widget(method_widget, area);
}

/// Render URL input field
fn render_url_input(f: &mut Frame, area: Rect, state: &HttpState, app: &App, is_focused: bool) {
    let url_value = if state.current_request.url.is_empty() {
        "https://api.example.com/endpoint"
    } else {
        &state.current_request.url
    };

    let url_style = if is_focused && app.input_mode == InputMode::Edit {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else if state.current_request.url.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let url_widget = Paragraph::new(url_value)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("URL")
                .title_style(Style::default().fg(Color::Green))
                .border_style(if is_focused {
                    Style::default().fg(Color::Blue)
                } else {
                    Style::default().fg(Color::White)
                }),
        )
        .style(url_style);

    f.render_widget(url_widget, area);
}

/// Render send button
fn render_send_button(f: &mut Frame, area: Rect, state: &HttpState, is_focused: bool) {
    let (button_text, button_style) = if state.is_sending {
        ("⏳ SENDING...", Style::default().fg(Color::Yellow))
    } else if !state.is_valid() {
        ("❌ INVALID", Style::default().fg(Color::Red))
    } else {
        (
            "🚀 SEND",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    };

    let send_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(button_text, button_style)),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Action")
            .title_style(Style::default().fg(Color::Magenta))
            .border_style(if is_focused {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::White)
            }),
    );

    f.render_widget(send_widget, area);
}

/// Render the tabbed request content (headers, body, etc.)
fn render_request_tabs(f: &mut Frame, area: Rect, state: &HttpState, app: &App, is_focused: bool) {
    let tab_titles: Vec<String> = HttpRequestTab::all()
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let title = tab.title();
            // Add indicators for tabs with content
            let indicator = match tab {
                HttpRequestTab::Headers if !state.current_request.headers.is_empty() => " •",
                HttpRequestTab::Body if state.current_request.body != HttpRequestBody::None => " •",
                HttpRequestTab::QueryParams if !state.current_request.query_params.is_empty() => {
                    " •"
                }
                HttpRequestTab::Auth if state.current_request.auth != HttpAuth::None => " •",
                _ => "",
            };
            format!("{}{}", title, indicator)
        })
        .collect();

    let selected_tab = HttpRequestTab::all()
        .iter()
        .position(|t| t == &state.current_request_tab)
        .unwrap_or(0);

    let tabs_widget = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Request")
                .title_style(Style::default().fg(Color::Green))
                .border_style(if is_focused {
                    Style::default().fg(Color::Blue)
                } else {
                    Style::default().fg(Color::White)
                }),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
        .select(selected_tab);

    // Split area for tabs and content
    let tab_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab headers
            Constraint::Min(3),    // Tab content
        ])
        .split(area);

    f.render_widget(tabs_widget, tab_chunks[0]);

    // Render the content for the selected tab
    let content_area = tab_chunks[1].inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    match state.current_request_tab {
        HttpRequestTab::Headers => render_headers_tab(f, content_area, state, app, is_focused),
        HttpRequestTab::Body => render_body_tab(f, content_area, state, app, is_focused),
        HttpRequestTab::QueryParams => {
            render_query_params_tab(f, content_area, state, app, is_focused)
        }
        HttpRequestTab::Auth => render_auth_tab(f, content_area, state, app, is_focused),
        HttpRequestTab::Settings => render_settings_tab(f, content_area, state, app, is_focused),
    }
}

/// Render headers tab content
fn render_headers_tab(f: &mut Frame, area: Rect, state: &HttpState, _app: &App, _is_focused: bool) {
    if state.current_request.headers.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No headers added",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'a' to add a header",
                Style::default().fg(Color::Gray),
            )),
        ];

        let empty_widget = Paragraph::new(empty_text);
        f.render_widget(empty_widget, area);
    } else {
        let header_items: Vec<ListItem> = state
            .current_request
            .headers
            .iter()
            .enumerate()
            .map(|(i, header)| {
                let enabled_indicator = if header.enabled { "✓" } else { "✗" };
                let style = if header.enabled {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", enabled_indicator), style),
                    Span::styled(
                        format!("{}: ", header.name),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(&header.value, style),
                ]))
            })
            .collect();

        let headers_list = List::new(header_items);
        f.render_widget(headers_list, area);
    }
}

/// Render body tab content  
fn render_body_tab(f: &mut Frame, area: Rect, state: &HttpState, app: &App, is_focused: bool) {
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Body type selector
            Constraint::Min(3),    // Body content
        ])
        .split(area);

    // Body type selector
    render_body_type_selector(f, body_chunks[0], state, is_focused);

    // Body content based on type
    match state.current_body_type {
        BodyContentType::None => {
            let no_body_text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No request body",
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            let no_body_widget = Paragraph::new(no_body_text);
            f.render_widget(no_body_widget, body_chunks[1]);
        }
        BodyContentType::Json | BodyContentType::Text | BodyContentType::Raw => {
            render_body_text_editor(f, body_chunks[1], state, app, is_focused);
        }
        BodyContentType::Form => {
            render_form_fields_editor(f, body_chunks[1], state, app, is_focused);
        }
    }
}

/// Render body type selector
fn render_body_type_selector(f: &mut Frame, area: Rect, state: &HttpState, _is_focused: bool) {
    let type_text = format!("Type: {}", state.current_body_type.title());
    let type_widget = Paragraph::new(type_text).style(Style::default().fg(Color::Yellow));

    f.render_widget(type_widget, area);
}

/// Render body text editor
fn render_body_text_editor(
    f: &mut Frame,
    area: Rect,
    state: &HttpState,
    app: &App,
    is_focused: bool,
) {
    let content = state.get_body_content();
    let placeholder = match state.current_body_type {
        BodyContentType::Json => "{\n  \"key\": \"value\"\n}",
        BodyContentType::Text => "Enter text content here...",
        BodyContentType::Raw => "Raw content...",
        _ => "",
    };

    let display_content = if content.is_empty() {
        placeholder
    } else {
        &content
    };
    let style = if content.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else if is_focused && app.input_mode == InputMode::Edit {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let body_widget = Paragraph::new(display_content)
        .style(style)
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(body_widget, area);
}

/// Render form fields editor (for form-data)
fn render_form_fields_editor(
    f: &mut Frame,
    area: Rect,
    _state: &HttpState,
    _app: &App,
    _is_focused: bool,
) {
    let form_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Form data editor",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Key-value pairs for form submission",
            Style::default().fg(Color::Gray),
        )),
    ];

    let form_widget = Paragraph::new(form_text);
    f.render_widget(form_widget, area);
}

/// Render query parameters tab
fn render_query_params_tab(
    f: &mut Frame,
    area: Rect,
    state: &HttpState,
    _app: &App,
    _is_focused: bool,
) {
    if state.current_request.query_params.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No query parameters",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'a' to add a parameter",
                Style::default().fg(Color::Gray),
            )),
        ];

        let empty_widget = Paragraph::new(empty_text);
        f.render_widget(empty_widget, area);
    } else {
        let param_items: Vec<ListItem> = state
            .current_request
            .query_params
            .iter()
            .map(|param| {
                let enabled_indicator = if param.enabled { "✓" } else { "✗" };
                let style = if param.enabled {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", enabled_indicator), style),
                    Span::styled(format!("{}=", param.name), Style::default().fg(Color::Cyan)),
                    Span::styled(&param.value, style),
                ]))
            })
            .collect();

        let params_list = List::new(param_items);
        f.render_widget(params_list, area);
    }
}

/// Render authentication tab
fn render_auth_tab(f: &mut Frame, area: Rect, state: &HttpState, _app: &App, _is_focused: bool) {
    let auth_text = match &state.current_request.auth {
        HttpAuth::None => vec![
            Line::from(""),
            Line::from(Span::styled(
                "No authentication",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Available types: Bearer Token, Basic Auth, API Key",
                Style::default().fg(Color::Gray),
            )),
        ],
        HttpAuth::Bearer { token } => vec![
            Line::from(Span::styled(
                "Bearer Token Authentication",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("Token: "),
                Span::styled(
                    if token.len() > 20 {
                        format!("{}...", &token[..20])
                    } else {
                        token.clone()
                    },
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ],
        HttpAuth::Basic { username, .. } => vec![
            Line::from(Span::styled(
                "Basic Authentication",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("Username: "),
                Span::styled(username, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("Password: "),
                Span::styled("••••••••", Style::default().fg(Color::DarkGray)),
            ]),
        ],
        HttpAuth::ApiKey {
            key,
            value,
            location,
        } => vec![
            Line::from(Span::styled(
                "API Key Authentication",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("Key: "),
                Span::styled(key, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("Location: "),
                Span::styled(
                    match location {
                        crate::models::ApiKeyLocation::Header => "Header",
                        crate::models::ApiKeyLocation::QueryParam => "Query Parameter",
                    },
                    Style::default().fg(Color::Green),
                ),
            ]),
        ],
    };

    let auth_widget = Paragraph::new(auth_text);
    f.render_widget(auth_widget, area);
}

/// Render settings tab
fn render_settings_tab(
    f: &mut Frame,
    area: Rect,
    _state: &HttpState,
    _app: &App,
    _is_focused: bool,
) {
    let settings_text = vec![
        Line::from(Span::styled(
            "Request Settings",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from("• Follow redirects: Yes"),
        Line::from("• Verify SSL: Yes"),
        Line::from("• Timeout: 30 seconds"),
        Line::from("• User Agent: Custom HTTP Client"),
    ];

    let settings_widget = Paragraph::new(settings_text);
    f.render_widget(settings_widget, area);
}
