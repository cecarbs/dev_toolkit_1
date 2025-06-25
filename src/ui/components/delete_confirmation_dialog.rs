use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Render delete confirmation dialog
pub fn render_delete_confirmation_dialog(f: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(70, 60, area);

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Item being deleted
            Constraint::Length(4), // Warning and count
            Constraint::Min(5),    // Contents list (if folder)
            Constraint::Length(3), // Buttons
        ])
        .split(popup_area);

    let item_type = if app.delete_confirmation_is_folder {
        "Folder"
    } else {
        "Template"
    };

    // Title
    let title = Paragraph::new(format!("🗑️ Delete {}", item_type))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Confirm Deletion"))
                .title_style(Style::default().fg(Color::Red))
                .style(Style::default().bg(Color::DarkGray))
                .border_style(Style::default().fg(Color::Red)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(title, chunks[0]);

    // Item being deleted
    let item_info = Paragraph::new(format!("Delete: {}", app.delete_confirmation_item_name))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Item")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(item_info, chunks[1]);

    // Warning and count
    let (folder_count, template_count) = app.get_deletion_count();
    let warning_lines = if app.delete_confirmation_is_folder {
        vec![
            Line::from(Span::styled(
                "⚠️  This will permanently delete:",
                Style::default().fg(Color::Red),
            )),
            Line::from(Span::styled(
                format!("   • {} folder(s)", folder_count + 1), // +1 for the folder itself
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                format!("   • {} template(s)", template_count),
                Style::default().fg(Color::White),
            )),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                "⚠️  This will permanently delete this template.",
                Style::default().fg(Color::Red),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "This action cannot be undone.",
                Style::default().fg(Color::Red),
            )),
        ]
    };

    let warning = Paragraph::new(warning_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Warning")
            .style(Style::default().bg(Color::DarkGray)),
    );
    f.render_widget(warning, chunks[2]);

    // Contents list (for folders)
    if app.delete_confirmation_is_folder && !app.delete_confirmation_contents.is_empty() {
        let content_items: Vec<ListItem> = app
            .delete_confirmation_contents
            .iter()
            .take(10) // Show max 10 items to fit in dialog
            .map(|item| {
                let (icon, name) = if let Some(pos) = item.find(' ') {
                    (&item[..pos], &item[pos + 1..])
                } else {
                    ("", item.as_str())
                };

                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(icon, Style::default().fg(Color::Blue)),
                    Span::raw(" "),
                    Span::styled(name, Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let more_text = if app.delete_confirmation_contents.len() > 10 {
            format!(
                "... and {} more items",
                app.delete_confirmation_contents.len() - 10
            )
        } else {
            String::new()
        };

        let mut all_items = content_items;
        if !more_text.is_empty() {
            all_items.push(ListItem::new(Line::from(Span::styled(
                more_text,
                Style::default().fg(Color::Gray),
            ))));
        }

        let contents_list = List::new(all_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Contents to be deleted")
                .style(Style::default().bg(Color::DarkGray)),
        );
        f.render_widget(contents_list, chunks[3]);
    }

    // Buttons
    let buttons = vec![Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Red)),
        Span::raw(": "),
        Span::styled("DELETE", Style::default().fg(Color::Red)),
        Span::raw("    "),
        Span::styled("Esc", Style::default().fg(Color::Green)),
        Span::raw(": "),
        Span::styled("Cancel", Style::default().fg(Color::Green)),
    ])];

    let button_widget = Paragraph::new(buttons)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Controls")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(button_widget, chunks[4]);
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
