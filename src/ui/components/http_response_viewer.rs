use crate::{
    app::{App, FocusedPane},
    models::{
        http::{HttpResponseTab, HttpState},
        http_client::HttpResponse,
    },
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
};

/// Render the HTTP response viewer with tabbed content
pub fn render_http_response_viewer(f: &mut Frame, area: Rect, state: &HttpState, app: &App) {
    let is_focused = app.focused_pane == FocusedPane::Logs; // Reuse Logs focus for response viewer

    if let Some(response) = &state.last_response {
        render_response_with_tabs(f, area, response, state, is_focused);
    } else {
        render_empty_response(f, area, state, is_focused);
    }
}

/// Render response viewer when no response is available
fn render_empty_response(f: &mut Frame, area: Rect, state: &HttpState, is_focused: bool) {
    let empty_text = if state.is_sending {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "⏳ Sending request...",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Please wait for response",
                Style::default().fg(Color::Gray),
            )),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "📭 No response yet",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Send a request to see the response here",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press F3 or Space to send",
                Style::default().fg(Color::Blue),
            )),
        ]
    };

    let empty_widget = Paragraph::new(empty_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Response")
                .title_style(Style::default().fg(Color::Magenta))
                .border_style(if is_focused {
                    Style::default().fg(Color::Blue)
                } else {
                    Style::default().fg(Color::White)
                }),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(empty_widget, area);
}

/// Render response viewer with tabs when response is available
fn render_response_with_tabs(
    f: &mut Frame,
    area: Rect,
    response: &HttpResponse,
    state: &HttpState,
    is_focused: bool,
) {
    let tab_titles: Vec<String> = HttpResponseTab::all()
        .iter()
        .map(|tab| {
            let title = tab.title();
            // Add indicators for tabs with content
            let indicator = match tab {
                HttpResponseTab::Headers if !response.headers.is_empty() => " •",
                HttpResponseTab::Body if !response.body.is_empty() => " •",
                _ => "",
            };
            format!("{}{}", title, indicator)
        })
        .collect();

    let selected_tab = HttpResponseTab::all()
        .iter()
        .position(|t| t == &state.current_response_tab)
        .unwrap_or(0);

    // Create title with status code
    let status_color = response.status_color();
    let title = format!(
        "Response - {} {}",
        response.status_code, response.status_text
    );

    let tabs_widget = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ))
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
    match state.current_response_tab {
        HttpResponseTab::Body => render_response_body_tab(f, content_area, response),
        HttpResponseTab::Headers => render_response_headers_tab(f, content_area, response),
        HttpResponseTab::Info => render_response_info_tab(f, content_area, response),
    }
}

/// Render response body tab
fn render_response_body_tab(f: &mut Frame, area: Rect, response: &HttpResponse) {
    if response.body.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Empty response body",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let empty_widget = Paragraph::new(empty_text);
        f.render_widget(empty_widget, area);
    } else {
        // Try to format the body based on content type
        let formatted_body = format_response_body(&response.body, &response.content_type);

        let body_widget = Paragraph::new(formatted_body)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .scroll((0, 0)); // TODO: Add scrolling support

        f.render_widget(body_widget, area);
    }
}

/// Render response headers tab
fn render_response_headers_tab(f: &mut Frame, area: Rect, response: &HttpResponse) {
    if response.headers.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No response headers",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let empty_widget = Paragraph::new(empty_text);
        f.render_widget(empty_widget, area);
    } else {
        let header_items: Vec<ListItem> = response
            .headers
            .iter()
            .map(|header| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{}: ", header.name),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(&header.value, Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let headers_list = List::new(header_items);
        f.render_widget(headers_list, area);
    }
}

/// Render response info tab (status, timing, etc.)
fn render_response_info_tab(f: &mut Frame, area: Rect, response: &HttpResponse) {
    let status_color = response.status_color();

    let info_lines = vec![
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(
                format!("{} {}", response.status_code, response.status_text),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Content-Type: "),
            Span::styled(&response.content_type, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Response Time: "),
            Span::styled(
                format!("{} ms", response.duration_ms),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Body Size: "),
            Span::styled(
                format_bytes(response.body.len()),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Headers Count: "),
            Span::styled(
                response.headers.len().to_string(),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    let info_widget = Paragraph::new(info_lines);
    f.render_widget(info_widget, area);
}

/// Format response body based on content type
fn format_response_body(body: &str, content_type: &str) -> String {
    if content_type.contains("application/json") {
        // Try to pretty-print JSON
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
            if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                return pretty;
            }
        }
    }

    // Return body as-is if we can't format it
    body.to_string()
}

/// Format byte count in human-readable format
fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Render status indicator for quick reference
pub fn render_response_status_indicator(f: &mut Frame, area: Rect, response: &HttpResponse) {
    let status_text = format!("{}", response.status_code);
    let status_style = Style::default()
        .fg(response.status_color())
        .add_modifier(Modifier::BOLD);

    let status_widget = Paragraph::new(status_text).style(status_style);

    f.render_widget(status_widget, area);
}
