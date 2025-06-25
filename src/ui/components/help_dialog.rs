use crate::app::{App, FocusedPane, InputMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
};

/// Help section definitions
#[derive(Debug, Clone)]
pub struct HelpSection {
    pub title: String,
    pub items: Vec<HelpItem>,
}

#[derive(Debug, Clone)]
pub struct HelpItem {
    pub keys: String,
    pub description: String,
    pub example: Option<String>,
}

impl HelpItem {
    pub fn new(keys: &str, description: &str) -> Self {
        Self {
            keys: keys.to_string(),
            description: description.to_string(),
            example: None,
        }
    }

    pub fn with_example(mut self, example: &str) -> Self {
        self.example = Some(example.to_string());
        self
    }
}

/// Render comprehensive help dialog
pub fn render_help_dialog(f: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(85, 80, area);

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Section tabs
            Constraint::Length(3), // Search bar
            Constraint::Min(5),    // Help content
            Constraint::Length(3), // Instructions
        ])
        .split(popup_area);

    // Title
    render_help_title(f, chunks[0]);

    // Section tabs
    render_help_tabs(f, chunks[1], app);

    // Search bar
    render_help_search(f, chunks[2], app);

    // Help content
    render_help_content(f, chunks[3], app);

    // Instructions
    render_help_instructions(f, chunks[4]);
}

fn render_help_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("🆘 Keybinding Reference")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help System")
                .title_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::DarkGray))
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(title, area);
}

fn render_help_tabs(f: &mut Frame, area: Rect, app: &App) {
    let section_titles = vec![
        "All Keybindings",
        "Global Shortcuts",
        "Collections Tree",
        "Form Fields",
        "Log Search",
        "Modal Editing",
    ];

    let tabs = Tabs::new(section_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sections")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).bg(Color::Blue))
        .select(app.help_selected_section);

    f.render_widget(tabs, area);
}

fn render_help_search(f: &mut Frame, area: Rect, app: &App) {
    let search_text = if app.help_search_query.is_empty() {
        "Type to search keybindings..."
    } else {
        &app.help_search_query
    };

    let style = if app.help_search_query.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let search_bar = Paragraph::new(search_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🔍 Search")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(style);

    f.render_widget(search_bar, area);
}

fn render_help_content(f: &mut Frame, area: Rect, app: &App) {
    let sections = get_help_sections();
    let filtered_sections = filter_help_sections(&sections, app);

    let items: Vec<ListItem> = filtered_sections
        .iter()
        .flat_map(|section| create_section_items(section))
        .collect();

    let help_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Keybindings")
            .style(Style::default().bg(Color::DarkGray)),
    );

    f.render_widget(help_list, area);
}

fn render_help_instructions(f: &mut Frame, area: Rect) {
    let instructions = vec![Line::from(vec![
        Span::styled("Tab/Shift+Tab", Style::default().fg(Color::Yellow)),
        Span::raw(": Switch sections  "),
        Span::styled("Type", Style::default().fg(Color::Yellow)),
        Span::raw(": Search  "),
        Span::styled("Esc/?", Style::default().fg(Color::Yellow)),
        Span::raw(": Close help"),
    ])];

    let help_widget = Paragraph::new(instructions)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Navigation")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(help_widget, area);
}

/// Get all help sections with comprehensive keybindings
fn get_help_sections() -> Vec<HelpSection> {
    vec![
        HelpSection {
            title: "Global Shortcuts".to_string(),
            items: vec![
                HelpItem::new("Ctrl+Q", "Quit application"),
                HelpItem::new("Ctrl+H", "Focus left pane (Collections)"),
                HelpItem::new("Ctrl+L", "Focus right pane (Form)"),
                HelpItem::new("F1", "Switch to Automation mode"),
                HelpItem::new("F4", "Switch to HTTP mode"),
                HelpItem::new("F2", "Toggle logs panel"),
                HelpItem::new("?", "Show this help dialog"),
            ],
        },
        HelpSection {
            title: "Collections Tree".to_string(),
            items: vec![
                HelpItem::new("↑/↓", "Navigate up/down in tree"),
                HelpItem::new("Enter", "Load template or expand/collapse folder"),
                HelpItem::new("Space", "Toggle folder expansion only"),
                HelpItem::new("Tab", "Focus next pane"),
                HelpItem::new("s", "Select current item"),
                HelpItem::new("Ctrl+N", "Create new template from current form")
                    .with_example("Fill form, then Ctrl+N to save as template"),
                HelpItem::new("Ctrl+F", "Create new folder")
                    .with_example("Creates subfolder in currently selected location"),
                HelpItem::new("F2 or R", "Rename selected item"),
                HelpItem::new("Delete", "Delete selected item (with confirmation)"),
                HelpItem::new("Ctrl+X", "Cut item to clipboard"),
                HelpItem::new("Ctrl+C", "Copy item to clipboard"),
                HelpItem::new("Ctrl+V", "Paste item from clipboard"),
                HelpItem::new("F12", "Refresh tree from storage"),
            ],
        },
        HelpSection {
            title: "Form Fields - Normal Mode".to_string(),
            items: vec![
                HelpItem::new("i or Enter", "Enter edit mode for current field"),
                HelpItem::new("j/k", "Navigate to next/previous field (Vim style)"),
                HelpItem::new("Tab/Shift+Tab", "Navigate to next/previous field"),
                HelpItem::new("Delete", "Clear current field completely"),
                HelpItem::new("Ctrl+N", "Save current form as new template"),
                HelpItem::new("F3", "Start automation with current form data"),
                HelpItem::new("Ctrl+C", "Set demo credentials (temporary)"),
                HelpItem::new("Ctrl+X", "Clear credentials"),
            ],
        },
        HelpSection {
            title: "Form Fields - Edit Mode".to_string(),
            items: vec![
                HelpItem::new("Esc", "Exit edit mode, return to normal mode"),
                HelpItem::new("←/→", "Move cursor left/right within field"),
                HelpItem::new("Home/End", "Jump to start/end of field"),
                HelpItem::new("Backspace", "Delete character before cursor"),
                HelpItem::new(
                    "Tab/Shift+Tab",
                    "Move to next/previous field (stay in edit mode)",
                ),
                HelpItem::new("Ctrl+N", "Save template (works in edit mode too)"),
                HelpItem::new("Any letter", "Type normally, including Shift for capitals")
                    .with_example("Shift+A produces 'A', just like normal typing"),
            ],
        },
        HelpSection {
            title: "Log Search".to_string(),
            items: vec![
                HelpItem::new("Type", "Search through log messages"),
                HelpItem::new("Delete", "Clear search query"),
                HelpItem::new("Esc", "Close logs panel"),
                HelpItem::new("F2", "Toggle logs panel"),
            ],
        },
        HelpSection {
            title: "Modal Editing Concepts".to_string(),
            items: vec![
                HelpItem::new("Normal Mode", "Navigate and execute commands")
                    .with_example("Like Vim's normal mode - keys are commands"),
                HelpItem::new("Edit Mode", "Type text into form fields")
                    .with_example("Like Vim's insert mode - keys insert text"),
                HelpItem::new("Pane Focus", "Only one pane receives input at a time")
                    .with_example("Blue borders show which pane is focused"),
                HelpItem::new("Visual Feedback", "Mode shown in pane titles")
                    .with_example("[NORMAL] or [EDIT] appears in Form Fields title"),
                HelpItem::new("Context Help", "Status line shows relevant keys")
                    .with_example("Different commands shown based on current mode"),
            ],
        },
    ]
}

/// Filter sections based on selected tab and search query
fn filter_help_sections(sections: &[HelpSection], app: &App) -> Vec<HelpSection> {
    let mut filtered = sections.to_vec();

    // Filter by selected section (0 = all)
    if app.help_selected_section > 0 && app.help_selected_section <= sections.len() {
        filtered = vec![sections[app.help_selected_section - 1].clone()];
    }

    // Filter by search query
    if !app.help_search_query.is_empty() {
        let query = app.help_search_query.to_lowercase();
        filtered = filtered
            .into_iter()
            .map(|mut section| {
                section.items = section
                    .items
                    .into_iter()
                    .filter(|item| {
                        item.keys.to_lowercase().contains(&query)
                            || item.description.to_lowercase().contains(&query)
                            || item
                                .example
                                .as_ref()
                                .map_or(false, |ex| ex.to_lowercase().contains(&query))
                    })
                    .collect();
                section
            })
            .filter(|section| !section.items.is_empty())
            .collect();
    }

    filtered
}

/// Create list items for a help section
fn create_section_items(section: &HelpSection) -> Vec<ListItem> {
    let mut items = Vec::new();

    // Section header
    items.push(ListItem::new(Line::from(vec![Span::styled(
        format!("▶ {}", section.title),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )])));

    // Section items
    for item in &section.items {
        // Main keybinding line
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(&item.keys, Style::default().fg(Color::Yellow)),
            Span::raw(" : "),
            Span::styled(&item.description, Style::default().fg(Color::White)),
        ])));

        // Example line (if present)
        if let Some(example) = &item.example {
            items.push(ListItem::new(Line::from(vec![
                Span::raw("    "),
                Span::styled("Example: ", Style::default().fg(Color::Green)),
                Span::styled(example, Style::default().fg(Color::Gray)),
            ])));
        }
    }

    // Add spacing after section
    items.push(ListItem::new(Line::from("")));

    items
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
