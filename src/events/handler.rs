use crate::app::{App, AppMode, FocusedPane};
use crate::models::{FocusDirection, LogLevel, NodeType};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard events and update app state accordingly
pub async fn handle_key_event(app: &mut App, key_event: KeyEvent) -> Result<()> {
    // Handle template creation dialog first if it's open
    if app.show_template_dialog {
        return handle_template_dialog_keys(app, key_event).await;
    }
    // Global keybindings that work in all modes
    match key_event.code {
        // Quit application
        KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
            return Ok(());
        }
        // Toggle logging panel
        KeyCode::F(2) => {
            app.log(LogLevel::Debug, "F2 detected - toggling logs");
            app.toggle_logs();
            return Ok(());
        }
        // Switch modes
        KeyCode::F(1) => {
            app.switch_mode(AppMode::Automation);
            return Ok(());
        }
        KeyCode::F(4) => {
            app.switch_mode(AppMode::Http);
            return Ok(());
        }
        // Focus switching
        KeyCode::F(5) => {
            app.focus_pane(FocusedPane::Collections);
            app.log(LogLevel::Debug, "Focused collections tree");
            return Ok(());
        }
        KeyCode::F(6) => {
            app.focus_pane(FocusedPane::Form);
            app.log(LogLevel::Debug, "Focused form");
            return Ok(());
        }
        KeyCode::F(7) => {
            if app.show_logs {
                app.focus_pane(FocusedPane::Logs);
                app.log(LogLevel::Debug, "Focused logs");
            }
            return Ok(());
        }
        _ => {}
    }

    // Pane-specific event handling
    match app.focused_pane {
        FocusedPane::Collections => handle_tree_keys(app, key_event).await?,
        FocusedPane::Form => handle_form_keys(app, key_event).await?,
        FocusedPane::Logs => handle_log_keys(app, key_event).await?,
    }

    Ok(())
}

/// Handle keyboard events for the template creation dialog
async fn handle_template_dialog_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        // Cancel dialog
        KeyCode::Esc => {
            app.hide_template_creation_dialog();
        }

        // Create template
        KeyCode::Enter => {
            if let Err(e) = app.create_template_from_dialog().await {
                app.log(LogLevel::Error, format!("Failed to create template: {}", e));
            }
        }

        // Navigate between fields
        KeyCode::Tab => {
            app.template_dialog_focused_field = (app.template_dialog_focused_field + 1) % 3;
        }
        KeyCode::BackTab => {
            app.template_dialog_focused_field = if app.template_dialog_focused_field == 0 {
                2
            } else {
                app.template_dialog_focused_field - 1
            };
        }

        // Text input for focused field
        KeyCode::Char(c) if key_event.modifiers.is_empty() => {
            match app.template_dialog_focused_field {
                0 => app.template_dialog_name.push(c),
                1 => app.template_dialog_folder.push(c),
                2 => app.template_dialog_description.push(c),
                _ => {}
            }
        }

        // Backspace for text editing
        KeyCode::Backspace if key_event.modifiers.is_empty() => {
            match app.template_dialog_focused_field {
                0 => {
                    app.template_dialog_name.pop();
                }
                1 => {
                    app.template_dialog_folder.pop();
                }
                2 => {
                    app.template_dialog_description.pop();
                }
                _ => {}
            }
        }

        // Clear current field
        KeyCode::Delete => match app.template_dialog_focused_field {
            0 => app.template_dialog_name.clear(),
            1 => app.template_dialog_folder.clear(),
            2 => app.template_dialog_description.clear(),
            _ => {}
        },

        _ => {}
    }

    Ok(())
}

/// Handle keyboard events for the collections tree
async fn handle_tree_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        // Navigate tree
        KeyCode::Up => {
            app.tree_state.move_focus(FocusDirection::Up);
            app.log(LogLevel::Debug, "Tree: moved focus up");
        }
        KeyCode::Down => {
            app.tree_state.move_focus(FocusDirection::Down);
            app.log(LogLevel::Debug, "Tree: moved focus down");
        }

        // Expand/collapse or load template
        KeyCode::Enter => {
            // Extract the data we need first to avoid borrow checker issues
            let node_info = app
                .tree_state
                .get_focused_node()
                .map(|node| (node.path.clone(), node.name.clone(), node.node_type.clone()));

            if let Some((path, name, node_type)) = node_info {
                match node_type {
                    NodeType::Folder => {
                        // Toggle folder expansion
                        app.tree_state.toggle_expansion(&path);
                        app.log(LogLevel::Debug, format!("Tree: toggled folder {}", name));
                    }
                    NodeType::Template => {
                        // Load template into form
                        if let Err(e) = app.load_template_into_form(&path).await {
                            app.log(LogLevel::Error, format!("Failed to load template: {}", e));
                        } else {
                            // Switch focus to form after loading
                            app.focus_pane(FocusedPane::Form);
                        }
                    }
                }
            }
        }

        // Just toggle expansion (don't load template)
        KeyCode::Char(' ') => {
            // Extract the data we need first, then drop the immutable borrow
            let folder_info = app
                .tree_state
                .get_focused_node()
                .filter(|node| node.node_type == NodeType::Folder)
                .map(|node| (node.path.clone(), node.name.clone()));

            if let Some((path, name)) = folder_info {
                app.tree_state.toggle_expansion(&path);
                app.log(LogLevel::Debug, format!("Tree: toggled folder {}", name));
            }
        }

        // Select node (for future operations)
        KeyCode::Char('s') => {
            // Extract the data we need first to avoid borrow checker issues
            let node_info = app
                .tree_state
                .get_focused_node()
                .map(|node| (node.path.clone(), node.name.clone()));

            if let Some((path, name)) = node_info {
                app.tree_state.select_node(&path);
                app.log(LogLevel::Debug, format!("Tree: selected {}", name));
            }
        }

        // Create new template from current form - Open dialog
        KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_template_creation_dialog();
        }

        // Delete selected template/folder (with confirmation)
        KeyCode::Delete => {
            // Extract the data we need first to avoid borrow checker issues
            let template_info = app
                .tree_state
                .get_focused_node()
                .filter(|node| node.node_type == NodeType::Template)
                .map(|node| (node.path.clone(), node.name.clone()));

            if let Some((template_path, name)) = template_info {
                if let Err(e) = app.delete_template(&template_path).await {
                    app.log(LogLevel::Error, format!("Failed to delete template: {}", e));
                } else {
                    app.log(LogLevel::Success, format!("Deleted template: {}", name));
                }
            }
        }

        // Refresh tree from storage
        KeyCode::F(12) => {
            if let Err(e) = app.refresh_tree_from_storage().await {
                app.log(LogLevel::Error, format!("Failed to refresh tree: {}", e));
            } else {
                app.log(LogLevel::Success, "Tree refreshed from storage");
            }
        }

        _ => {}
    }

    Ok(())
}

/// Handle keyboard events for the form
async fn handle_form_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    // If logging panel is open and we're searching, handle search input
    if app.show_logs && key_event.code != KeyCode::Esc {
        return handle_log_search_keys(app, key_event);
    }

    // Handle special key combinations FIRST before text input
    match key_event.code {
        // Close logging panel
        KeyCode::Esc => {
            if app.show_logs {
                app.show_logs = false;
                app.log_search_query.clear();
                app.log(LogLevel::Debug, "Closed logging panel");
            }
        }

        // Template selection - handle BEFORE text input
        KeyCode::Char('1') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.log(LogLevel::Debug, "Ctrl+1 detected - applying template 1");
            {
                app.automation_state.selected_template = Some(0);
                app.automation_state.apply_selected_template();
            }
            app.log(LogLevel::Info, "Applied template: Quick Task");
            return Ok(());
        }
        KeyCode::Char('2') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.log(LogLevel::Debug, "Ctrl+2 detected - applying template 2");
            {
                app.automation_state.selected_template = Some(1);
                app.automation_state.apply_selected_template();
            }
            app.log(LogLevel::Info, "Applied template: Urgent Request");
            return Ok(());
        }
        KeyCode::Char('3') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.log(LogLevel::Debug, "Ctrl+3 detected - applying template 3");
            {
                app.automation_state.selected_template = Some(2);
                app.automation_state.apply_selected_template();
            }
            app.log(LogLevel::Info, "Applied template: Weekly Report");
            return Ok(());
        }

        // Set demo credentials manually (temporary hotkey)
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.log(LogLevel::Debug, "Ctrl+C detected - setting credentials");
            if let Err(e) = app
                .auth_service
                .store_credentials("demo_user".to_string(), "demo_password".to_string())
            {
                app.log(
                    LogLevel::Error,
                    format!("Failed to store credentials: {}", e),
                );
            } else {
                app.log(LogLevel::Info, "Demo credentials set");
            }
            return Ok(());
        }

        // Clear credentials
        KeyCode::Char('x') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.log(LogLevel::Debug, "Ctrl+X detected - clearing credentials");
            if let Err(e) = app.auth_service.clear_credentials() {
                app.log(
                    LogLevel::Error,
                    format!("Failed to clear credentials: {}", e),
                );
            } else {
                app.log(LogLevel::Info, "Credentials cleared");
            }
            return Ok(());
        }

        // Start automation
        KeyCode::F(3) => {
            app.log(
                LogLevel::Debug,
                "F3 detected - attempting to start automation",
            );
            if !app.automation_state.is_running {
                // Check if we have credentials, if not set demo ones
                if !app.auth_service.has_credentials() {
                    if let Err(e) = app
                        .auth_service
                        .store_credentials("demo_user".to_string(), "demo_password".to_string())
                    {
                        app.log(
                            LogLevel::Error,
                            format!("Failed to store demo credentials: {}", e),
                        );
                    } else {
                        app.log(LogLevel::Info, "Using demo credentials for testing");
                    }
                }

                if let Err(e) = app.start_automation().await {
                    app.log(
                        LogLevel::Error,
                        format!("Failed to start automation: {}", e),
                    );
                }
            } else {
                app.log(LogLevel::Warn, "Automation is already running");
            }
            return Ok(());
        }

        // Navigation between fields
        KeyCode::Tab => {
            app.automation_state.focus_next_field();
            app.log(LogLevel::Debug, "Moved to next field");
            return Ok(());
        }
        KeyCode::BackTab => {
            app.automation_state.focus_prev_field();
            app.log(LogLevel::Debug, "Moved to previous field");
            return Ok(());
        }

        // Clear current field
        KeyCode::Delete => {
            let field_name = if let Some(field) = app.automation_state.get_focused_field() {
                field.name.clone()
            } else {
                String::new()
            };

            if !field_name.is_empty() {
                if let Some(field) = app.automation_state.get_focused_field_mut() {
                    field.value.clear();
                }
                app.log(LogLevel::Debug, format!("Cleared field: {}", field_name));
            }
            return Ok(());
        }

        // Text input for the focused field
        KeyCode::Char(c) if key_event.modifiers.is_empty() => {
            if let Some(field) = app.automation_state.get_focused_field_mut() {
                field.value.push(c);
            }
        }

        // Backspace for text editing
        KeyCode::Backspace if key_event.modifiers.is_empty() => {
            if let Some(field) = app.automation_state.get_focused_field_mut() {
                field.value.pop();
            }
        }

        _ => {}
    }

    Ok(())
}

/// Handle keyboard events for log search functionality
fn handle_log_search_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        KeyCode::Char(c) => {
            app.log_search_query.push(c);
        }
        KeyCode::Backspace => {
            app.log_search_query.pop();
        }
        KeyCode::Delete => {
            app.log_search_query.clear();
        }
        _ => {}
    }

    Ok(())
}

/// Handle keyboard events for the logs pane
async fn handle_log_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    handle_log_search_keys(app, key_event)?;
    Ok(())
}

/// Get help text for the current mode and focused pane
pub fn get_help_text(app: &App) -> Vec<String> {
    let mut help = vec![
        "Global Keybindings:".to_string(),
        "  Ctrl+Q: Quit application".to_string(),
        "  F2: Toggle logging panel".to_string(),
        "  F1: Switch to Automation mode".to_string(),
        "  F4: Switch to HTTP mode".to_string(),
        "  F5/F6/F7: Focus Collections/Form/Logs".to_string(),
        "".to_string(),
    ];

    match app.focused_pane {
        FocusedPane::Collections => {
            help.extend(vec![
                "Collections Tree:".to_string(),
                "  ↑/↓: Navigate tree".to_string(),
                "  Enter: Load template or expand folder".to_string(),
                "  Space: Toggle folder expansion".to_string(),
                "  S: Select node".to_string(),
                "  Ctrl+N: Create template from form".to_string(),
                "  Del: Delete selected template".to_string(),
                "  F12: Refresh tree".to_string(),
            ]);
        }
        FocusedPane::Form => {
            help.extend(vec![
                "Form Mode:".to_string(),
                "  Tab/Shift+Tab: Navigate between fields".to_string(),
                "  Ctrl+1/2/3: Apply templates".to_string(),
                "  Ctrl+C: Set demo credentials".to_string(),
                "  Ctrl+X: Clear credentials".to_string(),
                "  F3: Start automation".to_string(),
                "  Delete: Clear current field".to_string(),
            ]);
        }
        FocusedPane::Logs => {
            help.extend(vec![
                "Log Search:".to_string(),
                "  Type to search logs".to_string(),
                "  Delete: Clear search".to_string(),
                "  Esc: Close logging panel".to_string(),
            ]);
        }
    }

    help
}
