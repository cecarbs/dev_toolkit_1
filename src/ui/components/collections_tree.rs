use crate::app::{App, FocusedPane};
use crate::models::{NodeType, TreeNode};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{BorderType, Paragraph};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

/// Render the collections tree panel
pub fn render_collections_tree(f: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focused_pane == FocusedPane::Collections;

    // Split area to show clipboard status if there's something in clipboard
    let (tree_area, status_area) = if app.clipboard.is_some() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),    // Tree
                Constraint::Length(3), // Clipboard status
            ])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    // Get border style based on focus
    let border_style = if is_focused {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::White)
    };

    // Get border thickness based on focus
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Plain
    };

    // Get all visible nodes from the tree
    let visible_nodes = app.tree_state.get_visible_nodes();

    // Create list items for each visible node
    let items: Vec<ListItem> = visible_nodes
        .iter()
        .enumerate()
        .map(|(index, node)| render_tree_node(node, index, &app))
        .collect();

    // Show instructions if tree is empty
    let list = if items.is_empty() {
        let empty_items = vec![
            ListItem::new(Line::from(Span::styled(
                "No templates found",
                Style::default().fg(Color::DarkGray),
            ))),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(Span::styled(
                "Press Ctrl+N to create template",
                Style::default().fg(Color::Gray),
            ))),
            ListItem::new(Line::from(Span::styled(
                "Press Ctrl+F to create folder",
                Style::default().fg(Color::Gray),
            ))),
        ];

        List::new(empty_items)
    } else {
        List::new(items)
    };

    let title = if is_focused {
        ">>> Collections <<<"
    } else {
        "Collections"
    };

    let title_style = if is_focused {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::Gray)
    };

    let list_widget = list.block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(title_style)
            .border_type(border_type)
            .border_style(border_style),
    );
    f.render_widget(list_widget, tree_area);

    // Render clipboard status if there's something in clipboard
    if let Some(status_area) = status_area {
        render_clipboard_status(f, status_area, app);
    }
}

/// Render clipboard status bar
fn render_clipboard_status(f: &mut Frame, area: Rect, app: &App) {
    if let Some(status) = app.get_clipboard_status() {
        let status_text = vec![Line::from(vec![
            Span::styled("📋 ", Style::default().fg(Color::Yellow)),
            Span::styled(status, Style::default().fg(Color::White)),
            Span::styled(" | Press Ctrl+V to paste", Style::default().fg(Color::Gray)),
        ])];

        let status_widget = Paragraph::new(status_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Clipboard")
                .title_style(Style::default().fg(Color::Yellow))
                .border_style(Style::default().fg(Color::Yellow)),
        );

        f.render_widget(status_widget, area);
    }
}

/// Render a single tree node as a list item
fn render_tree_node<'a>(node: &'a TreeNode, index: usize, app: &App) -> ListItem<'a> {
    let is_focused =
        app.focused_pane == FocusedPane::Collections && app.tree_state.focused_index == index;
    let is_selected = node.is_selected;

    // Create indentation based on depth
    let indent = "  ".repeat(node.depth);

    // Choose icon based on node type and expansion state
    let icon = node.get_icon();

    // Choose colors based on state
    let (name_style, icon_style) = match (&node.node_type, is_selected, is_focused) {
        (NodeType::Folder, true, _) => (
            Style::default().fg(Color::Cyan),
            Style::default().fg(Color::Cyan),
        ),
        (NodeType::Template, true, _) => (
            Style::default().fg(Color::Green),
            Style::default().fg(Color::Green),
        ),
        (NodeType::Folder, false, true) => (
            Style::default().fg(Color::White).bg(Color::DarkGray),
            Style::default().fg(Color::Blue),
        ),
        (NodeType::Template, false, true) => (
            Style::default().fg(Color::White).bg(Color::DarkGray),
            Style::default().fg(Color::Blue),
        ),
        (NodeType::Folder, false, false) => (
            Style::default().fg(Color::Yellow),
            Style::default().fg(Color::Yellow),
        ),
        (NodeType::Template, false, false) => (
            Style::default().fg(Color::White),
            Style::default().fg(Color::White),
        ),
    };

    // Add expand/collapse indicator for folders
    let expand_indicator = match node.node_type {
        NodeType::Folder if !node.children.is_empty() => {
            if node.is_expanded {
                " ▼"
            } else {
                " ▶"
            }
        }
        _ => "",
    };

    let line = Line::from(vec![
        Span::raw(indent),
        Span::styled(icon, icon_style),
        Span::raw(" "),
        Span::styled(&node.name, name_style),
        Span::styled(expand_indicator, Style::default().fg(Color::Gray)),
    ]);

    ListItem::new(line)
}

///// Render a single tree node as a list item
//fn render_tree_node<'a>(node: &'a TreeNode, index: usize, app: &App) -> ListItem<'a> {
//    let is_focused =
//        app.focused_pane == FocusedPane::Collections && app.tree_state.focused_index == index;
//    let is_selected = node.is_selected;
//
//    // Create indentation based on depth
//    let indent = "  ".repeat(node.depth);
//
//    // Choose icon based on node type and expansion state
//    let icon = node.get_icon();
//
//    // Choose colors based on state
//    let (name_style, icon_style) = match (&node.node_type, is_selected, is_focused) {
//        (NodeType::Folder, true, _) => (
//            Style::default().fg(Color::Cyan),
//            Style::default().fg(Color::Cyan),
//        ),
//        (NodeType::Template, true, _) => (
//            Style::default().fg(Color::Green),
//            Style::default().fg(Color::Green),
//        ),
//        (NodeType::Folder, false, true) => (
//            Style::default().fg(Color::White).bg(Color::DarkGray),
//            Style::default().fg(Color::Blue),
//        ),
//        (NodeType::Template, false, true) => (
//            Style::default().fg(Color::White).bg(Color::DarkGray),
//            Style::default().fg(Color::Blue),
//        ),
//        (NodeType::Folder, false, false) => (
//            Style::default().fg(Color::Yellow),
//            Style::default().fg(Color::Yellow),
//        ),
//        (NodeType::Template, false, false) => (
//            Style::default().fg(Color::White),
//            Style::default().fg(Color::White),
//        ),
//    };
//
//    // Add expand/collapse indicator for folders
//    let expand_indicator = match node.node_type {
//        NodeType::Folder if !node.children.is_empty() => {
//            if node.is_expanded {
//                " ▼"
//            } else {
//                " ▶"
//            }
//        }
//        _ => "",
//    };
//
//    let line = Line::from(vec![
//        Span::raw(indent),
//        Span::styled(icon, icon_style),
//        Span::raw(" "),
//        Span::styled(&node.name, name_style),
//        Span::styled(expand_indicator, Style::default().fg(Color::Gray)),
//    ]);
//
//    ListItem::new(line)
//}

/// Render focus indicator and help text when tree is focused
fn render_tree_focus_indicator(f: &mut Frame, area: Rect, app: &App) {
    // We could add a small indicator or help text here
    // For now, the border color change is sufficient

    // Get current focused node for context
    if let Some(focused_node) = app.tree_state.get_focused_node() {
        // Could show additional info about the focused node
        // But for now, the visual styling is enough
    }
}

/// Get help text for tree navigation
pub fn get_tree_help_text() -> Vec<String> {
    vec![
        "Tree Navigation:".to_string(),
        "  ↑/↓: Navigate".to_string(),
        "  Enter: Select/Expand".to_string(),
        "  Space: Toggle expand".to_string(),
        "  Ctrl+N: New template".to_string(),
        "  Del: Delete selected".to_string(),
    ]
}
