use crate::app::{App, AppMode, FocusedPane, InputMode};
use crate::models::{FocusDirection, LogLevel, NodeType};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard events and update app state accordingly
pub async fn handle_key_event(app: &mut App, key_event: KeyEvent) -> Result<()> {
    // Handle dialogs first (in priority order)
    if app.show_help_dialog {
        return handle_help_dialog_keys(app, key_event).await;
    }
    if app.show_delete_confirmation_dialog {
        return handle_delete_confirmation_keys(app, key_event).await;
    }
    if app.show_template_dialog {
        return handle_template_dialog_keys(app, key_event).await;
    }
    if app.show_folder_dialog {
        return handle_folder_dialog_keys(app, key_event).await;
    }
    if app.show_rename_dialog {
        return handle_rename_dialog_keys(app, key_event).await;
    }
    if app.show_login_popup {
        return handle_login_dialog_keys(app, key_event).await;
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
        KeyCode::Char('H') => {
            if app.focused_pane == FocusedPane::Form {
                app.focus_pane(FocusedPane::Collections);
                app.log(
                    LogLevel::Debug,
                    "Focused collections tree using VIM motions",
                );
                return Ok(());
            }
        }
        KeyCode::Char('L') => {
            if app.focused_pane == FocusedPane::Collections {
                app.focus_pane(FocusedPane::Form);
                app.log(LogLevel::Debug, "Focused form using VIM motions");
                return Ok(());
            }
        }
        KeyCode::Char('?') => {
            app.show_help_dialog();
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
            let node_info = app
                .tree_state
                .get_focused_node()
                .map(|node| (node.path.clone(), node.name.clone(), node.node_type.clone()));

            if let Some((path, name, node_type)) = node_info {
                match node_type {
                    NodeType::Folder => {
                        app.tree_state.toggle_expansion(&path);
                        app.log(LogLevel::Debug, format!("Tree: toggled folder {}", name));
                    }
                    NodeType::Template => {
                        if let Err(e) = app.load_template_into_form(&path).await {
                            app.log(LogLevel::Error, format!("Failed to load template: {}", e));
                        } else {
                            app.focus_pane(FocusedPane::Form);
                        }
                    }
                }
            }
        }

        // Toggle expansion only
        KeyCode::Char(' ') => {
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

        // Select node
        KeyCode::Char('s') => {
            let node_info = app
                .tree_state
                .get_focused_node()
                .map(|node| (node.path.clone(), node.name.clone()));

            if let Some((path, name)) = node_info {
                app.tree_state.select_node(&path);
                app.log(LogLevel::Debug, format!("Tree: selected {}", name));
            }
        }

        // === CREATION OPERATIONS ===

        // Create new template from form
        KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_template_creation_dialog();
        }

        // Create new folder
        KeyCode::Char('f') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_folder_creation_dialog();
        }

        // === RENAME OPERATION ===

        // Rename focused item (F2 is standard rename shortcut)
        KeyCode::F(2) => {
            app.show_rename_dialog();
        }

        // Alternative rename with R key
        KeyCode::Char('r') => {
            app.show_rename_dialog();
        }

        // === CLIPBOARD OPERATIONS ===

        // Cut (Ctrl+X)
        KeyCode::Char('x') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.cut_focused_item();
        }

        // Copy (Ctrl+C)
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.copy_focused_item();
        }

        // Paste (Ctrl+V)
        KeyCode::Char('v') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(e) = app.paste_clipboard_item().await {
                app.log(LogLevel::Error, format!("Failed to paste: {}", e));
            }
        }

        // Clear clipboard (Ctrl+Shift+C)
        KeyCode::Char('C') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.clear_clipboard();
        }

        // === DELETE OPERATION ===

        // Delete selected item
        // KeyCode::Delete => {
        //     let item_info = app
        //         .tree_state
        //         .get_focused_node()
        //         .map(|node| (node.path.clone(), node.name.clone(), node.node_type.clone()));
        //
        //     if let Some((template_path, name, node_type)) = item_info {
        //         match node_type {
        //             NodeType::Template => {
        //                 if let Err(e) = app.delete_template(&template_path).await {
        //                     app.log(LogLevel::Error, format!("Failed to delete template: {}", e));
        //                 }
        //             }
        //             NodeType::Folder => {
        //                 // TODO: Implement folder deletion (requires confirmation)
        //                 app.log(LogLevel::Warn, "Folder deletion not yet implemented");
        //             }
        //         }
        //     }
        // }

        // Delete selected item (shows confirmation dialog)
        KeyCode::Delete => {
            let item_info = app
                .tree_state
                .get_focused_node()
                .map(|node| (node.path.clone(), node.name.clone(), node.node_type.clone()));

            if let Some((item_path, name, node_type)) = item_info {
                let is_folder = node_type == NodeType::Folder;
                app.show_delete_confirmation_dialog(&item_path, &name, is_folder);
            }
        }

        // === UTILITY OPERATIONS ===

        // Refresh tree
        KeyCode::F(12) => {
            if let Err(e) = app.refresh_tree_from_storage().await {
                app.log(LogLevel::Error, format!("Failed to refresh tree: {}", e));
            } else {
                app.log(LogLevel::Success, "Tree refreshed from storage");
            }
        }

        // Show help
        KeyCode::F(1) => {
            app.log(LogLevel::Info, "Tree Help: ↑/↓=Navigate, Enter=Load/Expand, Space=Toggle, Ctrl+N=New Template, Ctrl+F=New Folder, F2/R=Rename, Ctrl+X/C/V=Cut/Copy/Paste, Del=Delete, F12=Refresh");
        }

        _ => {}
    }

    Ok(())
}

async fn handle_form_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    // Handle log search when logs pane is focused
    if app.focused_pane == FocusedPane::Logs && app.show_logs && key_event.code != KeyCode::Esc {
        return handle_log_search_keys(app, key_event);
    }

    // Handle keys based on input mode
    match app.input_mode {
        InputMode::Normal => handle_normal_mode_keys(app, key_event).await?,
        InputMode::Edit => handle_edit_mode_keys(app, key_event).await?,
    }

    Ok(())
}
/// TODO: rRemove the template selection
/// Handle keyboard events for the form
// async fn handle_form_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
//     // If logging panel is open and we're searching, handle search input
//     // TODO: might not need this anymore if logs are on by default
//     if app.focused_pane == FocusedPane::Logs && app.show_logs && key_event.code != KeyCode::Esc {
//         return handle_log_search_keys(app, key_event);
//     }
//
//     // Handle keys based on input mode
//     match app.input_mode {
//         InputMode::Normal => handle_normal_mode_keys(app, key_event).await,
//         InputMode::Edit => handle_edit_mode_keys(app, key_event).await,
//     }
//
//     // Handle special key combinations FIRST before text input
//     match key_event.code {
//         // Close logging panel
//         KeyCode::Esc => {
//             if app.show_logs {
//                 app.show_logs = false;
//                 app.log_search_query.clear();
//                 app.log(LogLevel::Debug, "Closed logging panel");
//             }
//         }
//
//         // KeyCode::Backspace if app.form_field_editing && key_event.modifiers.is_empty() => app.log(
//         //     LogLevel::Debug,
//         //     "Backspace was pressed; implement code later",
//         // ),
//
//         // KeyCode::Enter => {
//         //     app.form_field_editing = true;
//         //     app.log(LogLevel::Debug, "Enter key was pressed");
//         // }
//
//         // Template selection - handle BEFORE text input
//         // KeyCode::Char('1') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
//         //     app.log(LogLevel::Debug, "Ctrl+1 detected - applying template 1");
//         //     {
//         //         app.automation_state.selected_template = Some(0);
//         //         app.automation_state.apply_selected_template();
//         //     }
//         //     app.log(LogLevel::Info, "Applied template: Quick Task");
//         //     return Ok(());
//         // }
//         // KeyCode::Char('2') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
//         //     app.log(LogLevel::Debug, "Ctrl+2 detected - applying template 2");
//         //     {
//         //         app.automation_state.selected_template = Some(1);
//         //         app.automation_state.apply_selected_template();
//         //     }
//         //     app.log(LogLevel::Info, "Applied template: Urgent Request");
//         //     return Ok(());
//         // }
//         // KeyCode::Char('3') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
//         //     app.log(LogLevel::Debug, "Ctrl+3 detected - applying template 3");
//         //     {
//         //         app.automation_state.selected_template = Some(2);
//         //         app.automation_state.apply_selected_template();
//         //     }
//         //     app.log(LogLevel::Info, "Applied template: Weekly Report");
//         //     return Ok(());
//         // }
//
//         // Create a new template from the current form
//         // KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
//         //     app.show_template_creation_dialog();
//         //     return Ok(());
//         // }
//
//         // TODO: remove later
//         // Set demo credentials manually (temporary hotkey)
//         // KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
//         //     app.log(LogLevel::Debug, "Ctrl+C detected - setting credentials");
//         //     if let Err(e) = app
//         //         .auth_service
//         //         .store_credentials("demo_user".to_string(), "demo_password".to_string())
//         //     {
//         //         app.log(
//         //             LogLevel::Error,
//         //             format!("Failed to store credentials: {}", e),
//         //         );
//         //     } else {
//         //         app.log(LogLevel::Info, "Demo credentials set");
//         //     }
//         //     return Ok(());
//         // }
//
//         // TODO: remove later
//         // Clear credentials
//         // KeyCode::Char('x') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
//         //     app.log(LogLevel::Debug, "Ctrl+X detected - clearing credentials");
//         //     if let Err(e) = app.auth_service.clear_credentials() {
//         //         app.log(
//         //             LogLevel::Error,
//         //             format!("Failed to clear credentials: {}", e),
//         //         );
//         //     } else {
//         //         app.log(LogLevel::Info, "Credentials cleared");
//         //     }
//         //     return Ok(());
//         // }
//         //
//         // Start automation
//         // KeyCode::F(3) => {
//         //     app.log(
//         //         LogLevel::Debug,
//         //         "F3 detected - attempting to start automation",
//         //     );
//         //     if !app.automation_state.is_running {
//         //         // Check if we have credentials, if not set demo ones
//         //         if !app.auth_service.has_credentials() {
//         //             if let Err(e) = app
//         //                 .auth_service
//         //                 .store_credentials("demo_user".to_string(), "demo_password".to_string())
//         //             {
//         //                 app.log(
//         //                     LogLevel::Error,
//         //                     format!("Failed to store demo credentials: {}", e),
//         //                 );
//         //             } else {
//         //                 app.log(LogLevel::Info, "Using demo credentials for testing");
//         //             }
//         //         }
//         //
//         //         if let Err(e) = app.start_automation().await {
//         //             app.log(
//         //                 LogLevel::Error,
//         //                 format!("Failed to start automation: {}", e),
//         //             );
//         //         }
//         //     } else {
//         //         app.log(LogLevel::Warn, "Automation is already running");
//         //     }
//         //     return Ok(());
//         // }
//
//         // Navigation between fields
//         // KeyCode::Tab => {
//         //     app.automation_state.focus_next_field();
//         //     app.log(LogLevel::Debug, "Moved to next field");
//         //     return Ok(());
//         // }
//         // KeyCode::BackTab => {
//         //     app.automation_state.focus_prev_field();
//         //     app.log(LogLevel::Debug, "Moved to previous field");
//         //     return Ok(());
//         // }
//         //
//         // // Clear current field
//         // KeyCode::Delete => {
//         //     let field_name = if let Some(field) = app.automation_state.get_focused_field() {
//         //         field.name.clone()
//         //     } else {
//         //         String::new()
//         //     };
//         //
//         //     if !field_name.is_empty() {
//         //         if let Some(field) = app.automation_state.get_focused_field_mut() {
//         //             field.value.clear();
//         //         }
//         //         app.log(LogLevel::Debug, format!("Cleared field: {}", field_name));
//         //     }
//         //     return Ok(());
//         // }
//         //
//         // // Text input for the focused field
//         // KeyCode::Char(c)
//         //     if app.input_mode == InputMode::Edit && key_event.modifiers.is_empty()
//         //         || key_event.modifiers == KeyModifiers::SHIFT =>
//         // {
//         //     if let Some(field) = app.automation_state.get_focused_field_mut() {
//         //         field.value.push(c);
//         //     }
//         // }
//         //
//         // // Backspace for text editing
//         // KeyCode::Backspace if key_event.modifiers.is_empty() => {
//         //     if let Some(field) = app.automation_state.get_focused_field_mut() {
//         //         field.value.pop();
//         //     }
//         // }
//         //
//         _ => {}
//     }
//
//     Ok(())
// }

async fn handle_normal_mode_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        // Enter edit mode
        KeyCode::Enter => {
            app.enter_edit_mode();
        }

        // Alternative: 'i' for insert mode (like Vim)
        KeyCode::Char('i') => {
            app.enter_edit_mode();
        }

        // Navigation between fields
        KeyCode::Tab => {
            app.automation_state.focus_next_field();
            app.log(LogLevel::Debug, "Moved to next field");
        }
        KeyCode::BackTab => {
            app.automation_state.focus_prev_field();
            app.log(LogLevel::Debug, "Moved to previous field");
        }

        // Vim-style navigation
        KeyCode::Char('j') => {
            app.automation_state.focus_next_field();
            app.log(LogLevel::Debug, "Moved to next field (vim style)");
        }
        KeyCode::Char('k') => {
            app.automation_state.focus_prev_field();
            app.log(LogLevel::Debug, "Moved to previous field (vim style)");
        }

        // Clear current field
        KeyCode::Delete => {
            if let Some(field) = app.automation_state.get_focused_field_mut() {
                field.value.clear();
                // app.log(LogLevel::Debug, format!("Cleared field: {}", field.name));
            }
        }

        // Global shortcuts (work in normal mode)
        KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_template_creation_dialog();
        }

        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            // Demo credentials
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
        }

        KeyCode::F(3) => {
            // Start automation
            if !app.automation_state.is_running {
                // Check if we have credentials
                if !app.auth_service.has_credentials() {
                    // Show login popup - user must authenticate first
                    app.show_login();
                    app.log(
                        LogLevel::Info,
                        "Authentication required to start automation",
                    );
                } else {
                    // We have credentials, start automation directly
                    if let Err(e) = app.start_automation().await {
                        app.log(
                            LogLevel::Error,
                            format!("Failed to start automation: {}", e),
                        );
                    }
                }
            } else {
                app.log(LogLevel::Warn, "Automation is already running");
            }
        }

        _ => {}
    }

    Ok(())
}

async fn handle_edit_mode_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        // Exit edit mode
        KeyCode::Esc => {
            app.exit_edit_mode();
        }

        // Text input
        KeyCode::Char(c)
            if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT =>
        {
            app.insert_char_at_cursor(c);
        }

        // Backspace
        KeyCode::Backspace if key_event.modifiers.is_empty() => {
            app.delete_char_at_cursor();
        }

        // Cursor movement
        KeyCode::Left => {
            app.move_field_cursor_left();
        }
        KeyCode::Right => {
            app.move_field_cursor_right();
        }

        // Move to start/end of field
        KeyCode::Home => {
            app.reset_field_cursor();
        }
        KeyCode::End => {
            app.set_cursor_to_end_of_field();
        }

        // Navigate to next field (but stay in edit mode)
        KeyCode::Tab => {
            app.automation_state.focus_next_field();
            app.set_cursor_to_end_of_field();
            app.log(
                LogLevel::Debug,
                "Moved to next field (staying in edit mode)",
            );
        }
        KeyCode::BackTab => {
            app.automation_state.focus_prev_field();
            app.set_cursor_to_end_of_field();
            app.log(
                LogLevel::Debug,
                "Moved to previous field (staying in edit mode)",
            );
        }

        // Global shortcuts that should work even in edit mode
        KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_template_creation_dialog();
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

/// Handle keyboard events for the delete confirmation dialog
async fn handle_delete_confirmation_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        // Cancel deletion
        KeyCode::Esc => {
            app.hide_delete_confirmation_dialog();
        }
        // Confirm deletion
        KeyCode::Enter => {
            if let Err(e) = app.confirm_deletion().await {
                app.log(LogLevel::Error, format!("Failed to delete: {}", e));
            }
        }
        _ => {
            // Ignore other keys in confirmation dialog
        }
    }

    Ok(())
}

/// Handle keyboard events for the folder creation dialog
async fn handle_folder_dialog_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        KeyCode::Esc => {
            app.hide_folder_creation_dialog();
        }
        KeyCode::Enter => {
            if let Err(e) = app.create_folder_from_dialog().await {
                app.log(LogLevel::Error, format!("Failed to create folder: {}", e));
            }
        }
        KeyCode::Char(c)
            if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT =>
        {
            app.folder_dialog_name.push(c);
            app.folder_dialog_error = None; // Clear error on new input
        }
        KeyCode::Backspace
            if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT =>
        {
            app.folder_dialog_name.pop();
            app.folder_dialog_error = None;
        }
        KeyCode::Delete => {
            app.folder_dialog_name.clear();
            app.folder_dialog_error = None;
        }
        _ => {}
    }

    Ok(())
}

/// Handle keyboard events for the rename dialog
async fn handle_rename_dialog_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        KeyCode::Esc => {
            app.hide_rename_dialog();
        }
        KeyCode::Enter => {
            if let Err(e) = app.rename_item_from_dialog().await {
                app.log(LogLevel::Error, format!("Failed to rename: {}", e));
            }
        }
        KeyCode::Char(c)
            if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT =>
        {
            app.rename_dialog_new_name.push(c);
            app.rename_dialog_error = None; // Clear error on new input
        }
        KeyCode::Backspace
            if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT =>
        {
            app.rename_dialog_new_name.pop();
            app.rename_dialog_error = None;
        }
        KeyCode::Delete => {
            app.rename_dialog_new_name.clear();
            app.rename_dialog_error = None;
        }
        _ => {}
    }

    Ok(())
}

/// Updated help text with all new commands
pub fn get_tree_help_text() -> Vec<String> {
    vec![
        "Collections Tree Navigation:".to_string(),
        "  ↑/↓: Navigate tree".to_string(),
        "  Enter: Load template or expand folder".to_string(),
        "  Space: Toggle folder expansion".to_string(),
        "  S: Select node".to_string(),
        "".to_string(),
        "Creation:".to_string(),
        "  Ctrl+N: Create template from form".to_string(),
        "  Ctrl+F: Create new folder".to_string(),
        "".to_string(),
        "Editing:".to_string(),
        "  F2 or R: Rename selected item".to_string(),
        "  Del: Delete selected item".to_string(),
        "".to_string(),
        "Clipboard:".to_string(),
        "  Ctrl+X: Cut item".to_string(),
        "  Ctrl+C: Copy item".to_string(),
        "  Ctrl+V: Paste item".to_string(),
        "  Ctrl+Shift+C: Clear clipboard".to_string(),
        "".to_string(),
        "Utility:".to_string(),
        "  F12: Refresh tree from storage".to_string(),
        "  F1: Show this help".to_string(),
    ]
}

/// Handle keyboard events for the help dialog
async fn handle_help_dialog_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        // Close help dialog
        KeyCode::Esc | KeyCode::Char('?') => {
            app.hide_help_dialog();
        }

        // Navigate sections
        KeyCode::Tab => {
            let max_sections = 6; // Update this if you add more sections
            app.help_selected_section = (app.help_selected_section + 1) % max_sections;
        }
        KeyCode::BackTab => {
            let max_sections = 6;
            app.help_selected_section = if app.help_selected_section == 0 {
                max_sections - 1
            } else {
                app.help_selected_section - 1
            };
        }

        // Number keys for quick section access
        KeyCode::Char('1') => app.help_selected_section = 1,
        KeyCode::Char('2') => app.help_selected_section = 2,
        KeyCode::Char('3') => app.help_selected_section = 3,
        KeyCode::Char('4') => app.help_selected_section = 4,
        KeyCode::Char('5') => app.help_selected_section = 5,
        KeyCode::Char('0') => app.help_selected_section = 0, // All sections

        // Search functionality
        KeyCode::Char(c)
            if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT =>
        {
            app.help_search_query.push(c);
        }
        KeyCode::Backspace => {
            app.help_search_query.pop();
        }
        KeyCode::Delete => {
            app.help_search_query.clear();
        }

        _ => {}
    }

    Ok(())
}

/// Handle keyboard events for the login dialog
async fn handle_login_dialog_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        // Cancel login
        KeyCode::Esc => {
            app.hide_login();
        }

        // Attempt login
        KeyCode::Enter => {
            if app.attempt_login() {
                // Login successful, dialog closed automatically
                // Now actually start the automation
                if let Err(e) = app.start_automation().await {
                    app.log(
                        LogLevel::Error,
                        format!("Failed to start automation: {}", e),
                    );
                }
            }
            // If login failed, stay in dialog with error shown
        }

        // Navigate between username and password fields
        KeyCode::Tab => {
            app.login_focused_field = (app.login_focused_field + 1) % 2;
        }
        KeyCode::BackTab => {
            app.login_focused_field = if app.login_focused_field == 0 { 1 } else { 0 };
        }
        KeyCode::Up => {
            app.login_focused_field = if app.login_focused_field == 0 { 1 } else { 0 };
        }
        KeyCode::Down => {
            app.login_focused_field = (app.login_focused_field + 1) % 2;
        }

        // Text input for focused field
        KeyCode::Char(c)
            if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT =>
        {
            match app.login_focused_field {
                0 => app.login_username.push(c),
                1 => app.login_password.push(c),
                _ => {}
            }
            // Clear error on new input
            app.login_error = None;
        }

        // Backspace for text editing
        KeyCode::Backspace => {
            match app.login_focused_field {
                0 => {
                    app.login_username.pop();
                }
                1 => {
                    app.login_password.pop();
                }
                _ => {}
            }
            // Clear error on edit
            app.login_error = None;
        }

        // Clear current field
        KeyCode::Delete => {
            match app.login_focused_field {
                0 => app.login_username.clear(),
                1 => app.login_password.clear(),
                _ => {}
            }
            app.login_error = None;
        }

        _ => {}
    }

    Ok(())
}
