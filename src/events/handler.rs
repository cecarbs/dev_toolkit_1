use crate::app::{App, AppMode, FocusedPane, InputMode};
use crate::models::http::HttpRequestTab;
use crate::models::http_client::{HttpMethod, HttpRequestBody};
use crate::models::{FocusDirection, LogLevel, NodeType};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard events and update app state accordingly
pub async fn handle_key_event(app: &mut App, key_event: KeyEvent) -> Result<()> {
    // Handle dialogs first (in priority order)
    if app.show_help_dialog {
        return handle_help_dialog_keys(app, key_event).await;
    }
    if app.show_import_dialog {
        return handle_import_dialog_keys(app, key_event).await;
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
        // KeyCode::F(2) => {
        //     app.log(LogLevel::Debug, "F2 detected - toggling logs");
        //     app.toggle_logs();
        //     return Ok(());
        // }
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
            app.focus_pane(FocusedPane::Logs);
            app.log(LogLevel::Debug, "Focused logs");
            return Ok(());
        }

        // Vim-style pane navigation
        KeyCode::Char('H') => {
            match app.focused_pane {
                FocusedPane::Form => {
                    app.focus_pane(FocusedPane::Collections);
                    app.log(
                        LogLevel::Debug,
                        "Focused collections tree using VIM motions",
                    );
                }
                FocusedPane::Logs => {
                    app.focus_pane(FocusedPane::Form);
                    app.log(LogLevel::Debug, "Focused form using VIM motions");
                }
                _ => {}
            }
            return Ok(());
        }
        KeyCode::Char('L') => {
            match app.focused_pane {
                FocusedPane::Collections => {
                    app.focus_pane(FocusedPane::Form);
                    app.log(LogLevel::Debug, "Focused form using VIM motions");
                }
                FocusedPane::Form => {
                    app.focus_pane(FocusedPane::Logs);
                    app.log(LogLevel::Debug, "Focused logs using VIM motions");
                }
                _ => {}
            }
            return Ok(());
        }
        KeyCode::Char('J') => {
            // Move down to next pane
            match app.focused_pane {
                FocusedPane::Collections => {
                    app.focus_pane(FocusedPane::Form);
                    app.log(LogLevel::Debug, "Focused form using VIM motions");
                }
                FocusedPane::Form => {
                    app.focus_pane(FocusedPane::Logs);
                    app.log(LogLevel::Debug, "Focused logs using VIM motions");
                }
                FocusedPane::Logs => {
                    app.focus_pane(FocusedPane::Collections);
                    app.log(LogLevel::Debug, "Focused collections using VIM motions");
                }
            }
            return Ok(());
        }
        KeyCode::Char('K') => {
            // Move up to previous pane
            match app.focused_pane {
                FocusedPane::Collections => {
                    app.focus_pane(FocusedPane::Logs);
                    app.log(LogLevel::Debug, "Focused logs using VIM motions");
                }
                FocusedPane::Form => {
                    app.focus_pane(FocusedPane::Collections);
                    app.log(LogLevel::Debug, "Focused collections using VIM motions");
                }
                FocusedPane::Logs => {
                    app.focus_pane(FocusedPane::Form);
                    app.log(LogLevel::Debug, "Focused form using VIM motions");
                }
            }
            return Ok(());
        }
        KeyCode::Char('?') => {
            app.show_help_dialog();
            return Ok(());
        }
        // TODO: remove later only used for testing
        // ADD THIS: Test Python integration
        // KeyCode::F(9) => {
        //     if let Err(e) = app.test_real_automation_script().await {
        //         app.log(LogLevel::Error, format!("Failed to start Python test: {}", e));
        //     }
        //     return Ok(());
        // }
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

/// Handle keyboard events for the import dialog
async fn handle_import_dialog_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        // Cancel import
        KeyCode::Esc => {
            app.hide_import_dialog();
        }

        // Execute import
        KeyCode::Enter => {
            if let Err(e) = app.execute_import().await {
                app.log(LogLevel::Error, format!("Import execution failed: {}", e));
            }
        }

        // Text input for file path
        KeyCode::Char(c) 
            if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT =>
        {
            app.import_dialog_add_char(c);
        }

        // Backspace for text editing
        KeyCode::Backspace => {
            app.import_dialog_backspace();
        }

        // Clear field
        KeyCode::Delete => {
            app.import_dialog_clear();
        }

        // Auto-suggest path (Tab key)
        KeyCode::Tab => {
            app.import_dialog_suggest_path();
        }

        // Quick paths (for common locations)
        KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            // Suggest Downloads folder
            if let Some(home_dir) = dirs::home_dir() {
                let downloads_path = home_dir.join("Downloads").join("collection.json");
                app.import_dialog_file_path = downloads_path.to_string_lossy().to_string();
                app.update_import_file_path(app.import_dialog_file_path.clone());
            }
        }

        KeyCode::Char('h') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            // Suggest home folder
            if let Some(home_dir) = dirs::home_dir() {
                let home_path = home_dir.join("collection.json");
                app.import_dialog_file_path = home_path.to_string_lossy().to_string();
                app.update_import_file_path(app.import_dialog_file_path.clone());
            }
        }

        KeyCode::Char('.') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            // Suggest current directory
            app.import_dialog_file_path = "./collection.json".to_string();
            app.update_import_file_path(app.import_dialog_file_path.clone());
        }

        _ => {}
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

        // Expand/collapse or load template/request
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
                        // Mode-aware loading
                        match app.current_mode {
                            AppMode::Automation => {
                                if let Err(e) = app.load_template_into_form(&path).await {
                                    app.log(
                                        LogLevel::Error,
                                        format!("Failed to load template: {}", e),
                                    );
                                } else {
                                    app.focus_pane(FocusedPane::Form);
                                }
                            }
                            AppMode::Http => {
                                if let Err(e) = app.load_http_request_into_form(&path).await {
                                    app.log(
                                        LogLevel::Error,
                                        format!("Failed to load HTTP request: {}", e),
                                    );
                                } else {
                                    app.focus_pane(FocusedPane::Form);
                                }
                            }
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

        // Create new template/request from form
        KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            match app.current_mode {
                AppMode::Automation => {
                    app.show_template_creation_dialog();
                }
                AppMode::Http => {
                    // For HTTP mode, we'll need a similar dialog for saving requests
                    // For now, let's create a simple version
                    app.show_http_request_creation_dialog(); // We'll implement this next
                }
            }
        }

        // Create new folder (works for both modes)
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

        // === DELETE OPERATION (Mode-aware) ===
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

        // === IMPORT OPERATIONS (HTTP mode only) ===
        KeyCode::Char('i') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.current_mode == AppMode::Http {
                app.show_import_dialog();
            } else {
                app.log(LogLevel::Info, "Import is only available in HTTP Client mode (F4)");
            }
        }


        // Alternative import shortcut (F8 key)
        KeyCode::F(8) => {
            if app.current_mode == AppMode::Http {
                app.show_import_dialog();
            } else {
                app.log(LogLevel::Info, "Import is only available in HTTP Client mode (F4)");
            }
        }

        // === UTILITY OPERATIONS ===

        // Refresh tree
        KeyCode::F(12) => {
            if let Err(e) = app.refresh_tree_from_storage().await {
                app.log(LogLevel::Error, format!("Failed to refresh tree: {}", e));
            } else {
                let mode_name = match app.current_mode {
                    AppMode::Automation => "automation templates",
                    AppMode::Http => "HTTP collections",
                };
                app.log(LogLevel::Success, format!("Refreshed {} tree", mode_name));
            }
        }

        // Show help
        KeyCode::F(1) => {
            let mode_name = match app.current_mode {
                AppMode::Automation => "Automation",
                AppMode::Http => "HTTP Client",
            };
            app.log(LogLevel::Info, 
                format!("{} Tree Help: ↑/↓=Navigate, Enter=Load/Expand, Space=Toggle, Ctrl+N=New, Ctrl+F=Folder, F2/R=Rename, Del=Delete, F12=Refresh", mode_name)
            );
        }

        _ => {}
    }

    Ok(())
}

async fn handle_form_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match app.current_mode {
        AppMode::Automation => handle_automation_form_keys(app, key_event).await?,
        AppMode::Http => handle_http_form_keys(app, key_event).await?,
    }

    Ok(())
}

/// Handle keyboard events for the form pane (both automation and HTTP modes)
// async fn handle_form_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
//     // // Handle log search when logs pane is focused
//     // if app.focused_pane == FocusedPane::Logs && app.show_logs && key_event.code != KeyCode::Esc {
//     //     return handle_log_search_keys(app, key_event);
//     // }
//
//     // TODO: previous version
//     // Handle keys based on input mode
//     // match app.input_mode {
//     //     InputMode::Normal => handle_normal_mode_keys(app, key_event).await?,
//     //     InputMode::Edit => handle_edit_mode_keys(app, key_event).await?,
//     // }
//     match app.current_mode {
//         AppMode::Automation => handle_automation_form_keys(app, key_event).await,
//         AppMode::Http => handle_http_form_keys(app, key_event).await,
//     }
//
//     Ok(())
// }

/// Handle HTTP form keys in normal mode
async fn handle_http_normal_mode_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        // Enter edit mode
        KeyCode::Enter => {
            app.enter_edit_mode();
        }

        // Alternative: 'i' for insert mode (like Vim)
        KeyCode::Char('i') => {
            app.enter_edit_mode();
        }

        // Tab navigation for request editor
        KeyCode::Tab => {
            app.http_state.next_request_tab();
            app.log(
                LogLevel::Debug,
                format!(
                    "Switched to {} tab",
                    app.http_state.current_request_tab.title()
                ),
            );
        }
        KeyCode::BackTab => {
            app.http_state.prev_request_tab();
            app.log(
                LogLevel::Debug,
                format!(
                    "Switched to {} tab",
                    app.http_state.current_request_tab.title()
                ),
            );
        }

        // Method selection (M key)
        KeyCode::Char('m') => {
            cycle_http_method(app);
        }

        // Quick method shortcuts
        KeyCode::Char('1') => {
            app.http_state.set_method(HttpMethod::GET);
            app.log(LogLevel::Debug, "Set method to GET");
        }
        KeyCode::Char('2') => {
            app.http_state.set_method(HttpMethod::POST);
            app.log(LogLevel::Debug, "Set method to POST");
        }
        KeyCode::Char('3') => {
            app.http_state.set_method(HttpMethod::PUT);
            app.log(LogLevel::Debug, "Set method to PUT");
        }
        KeyCode::Char('4') => {
            app.http_state.set_method(HttpMethod::DELETE);
            app.log(LogLevel::Debug, "Set method to DELETE");
        }

        // Clear current field/content
        KeyCode::Delete => {
            match app.http_state.current_request_tab {
                HttpRequestTab::Headers => {
                    // Clear all headers
                    app.http_state.current_request.headers.clear();
                    app.log(LogLevel::Debug, "Cleared all headers");
                }
                HttpRequestTab::Body => {
                    // Clear body content
                    app.http_state.set_body(HttpRequestBody::None);
                    app.log(LogLevel::Debug, "Cleared request body");
                }
                HttpRequestTab::QueryParams => {
                    // Clear all query params
                    app.http_state.current_request.query_params.clear();
                    app.log(LogLevel::Debug, "Cleared all query parameters");
                }
                _ => {}
            }
        }

        // Send HTTP request (F3 or Space)
        KeyCode::F(3) | KeyCode::Char(' ') => {
            if let Err(e) = app.send_http_request().await {
                app.log(
                    LogLevel::Error,
                    format!("Failed to send HTTP request: {}", e),
                );
            }
        }

        // New request
        KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.http_state.new_request();
            app.log(LogLevel::Info, "Created new HTTP request");
        }

        _ => {}
    }

    Ok(())
}

/// Handle HTTP form keys in edit mode
async fn handle_http_edit_mode_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        // Exit edit mode
        KeyCode::Esc => {
            app.exit_edit_mode();
        }

        // Text input (URL editing for now - we'll expand this)
        KeyCode::Char(c)
            if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT =>
        {
            // For now, just handle URL editing
            let mut url = app.http_state.current_request.url.clone();
            url.push(c);
            app.http_state.set_url(url);
        }

        // Backspace
        KeyCode::Backspace if key_event.modifiers.is_empty() => {
            let mut url = app.http_state.current_request.url.clone();
            url.pop();
            app.http_state.set_url(url);
        }

        // Navigate to next tab (but stay in edit mode)
        KeyCode::Tab => {
            app.http_state.next_request_tab();
            app.log(
                LogLevel::Debug,
                format!(
                    "Moved to {} tab (staying in edit mode)",
                    app.http_state.current_request_tab.title()
                ),
            );
        }
        KeyCode::BackTab => {
            app.http_state.prev_request_tab();
            app.log(
                LogLevel::Debug,
                format!(
                    "Moved to {} tab (staying in edit mode)",
                    app.http_state.current_request_tab.title()
                ),
            );
        }

        // Send request even in edit mode
        KeyCode::F(3) => {
            if let Err(e) = app.send_http_request().await {
                app.log(
                    LogLevel::Error,
                    format!("Failed to send HTTP request: {}", e),
                );
            }
        }

        _ => {}
    }

    Ok(())
}

/// Handle keyboard events for HTTP request editor
async fn handle_http_form_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    // Handle keys based on input mode
    match app.input_mode {
        InputMode::Normal => handle_http_normal_mode_keys(app, key_event).await?,
        InputMode::Edit => handle_http_edit_mode_keys(app, key_event).await?,
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
// fn handle_log_search_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
//     match key_event.code {
//         KeyCode::Char(c) => {
//             app.log_search_query.push(c);
//         }
//         KeyCode::Backspace => {
//             app.log_search_query.pop();
//         }
//         KeyCode::Delete => {
//             app.log_search_query.clear();
//         }
//         _ => {}
//     }
//
//     Ok(())
// }
/// Handle keyboard events for response viewer (reusing logs pane focus)
async fn handle_log_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match app.current_mode {
        AppMode::Automation => handle_automation_log_keys(app, key_event).await,
        AppMode::Http => handle_http_response_keys(app, key_event).await,
    }
}

/// Cycle through HTTP methods
fn cycle_http_method(app: &mut App) {
    let methods = HttpMethod::all();
    let current_index = methods
        .iter()
        .position(|m| m == &app.http_state.current_request.method)
        .unwrap_or(0);
    let next_index = (current_index + 1) % methods.len();

    app.http_state.set_method(methods[next_index].clone());
    app.log(
        LogLevel::Debug,
        format!(
            "Cycled method to {}",
            app.http_state.current_request.method.as_str()
        ),
    );
}

/// Handle the original automation form keys (renamed for clarity)
async fn handle_automation_form_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    // Handle keys based on input mode
    match app.input_mode {
        InputMode::Normal => handle_normal_mode_keys(app, key_event).await?,
        InputMode::Edit => handle_edit_mode_keys(app, key_event).await?,
    }

    Ok(())
}

/// Handle the original automation log keys (renamed for clarity)
async fn handle_automation_log_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    // If in search mode, handle search input
    if app.log_search_mode {
        match key_event.code {
            // Exit search mode
            KeyCode::Esc => {
                app.toggle_log_search_mode();
                app.log(LogLevel::Debug, "Exited log search mode");
            }

            // Search input
            KeyCode::Char(c)
                if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT =>
            {
                app.log_search_query.push(c);
            }
            KeyCode::Backspace => {
                app.log_search_query.pop();
            }
            KeyCode::Delete => {
                app.log_search_query.clear();
            }

            // Allow scrolling even in search mode
            KeyCode::Up | KeyCode::Char('k') => {
                app.scroll_logs_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.scroll_logs_down();
            }
            KeyCode::Home | KeyCode::Char('g') => {
                app.scroll_logs_to_top();
            }
            KeyCode::End | KeyCode::Char('G') => {
                app.scroll_logs_to_bottom();
            }

            _ => {}
        }
    } else {
        // Normal log navigation mode
        match key_event.code {
            // Enter search mode
            KeyCode::Char('/') => {
                app.toggle_log_search_mode();
                app.log(LogLevel::Debug, "Entered log search mode");
            }

            // Scroll navigation (vim-style)
            KeyCode::Up | KeyCode::Char('k') => {
                app.scroll_logs_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.scroll_logs_down();
            }

            // Page navigation
            KeyCode::PageUp | KeyCode::Char('u')
                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                for _ in 0..10 {
                    app.scroll_logs_up();
                }
            }
            KeyCode::PageDown | KeyCode::Char('d')
                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                for _ in 0..10 {
                    app.scroll_logs_down();
                }
            }

            // Jump to top/bottom (vim-style)
            KeyCode::Home | KeyCode::Char('g') => {
                app.scroll_logs_to_top();
                app.log(LogLevel::Debug, "Jumped to top of logs");
            }
            KeyCode::End | KeyCode::Char('G') => {
                app.scroll_logs_to_bottom();
                app.log(LogLevel::Debug, "Jumped to bottom of logs");
            }

            // Clear search (if any)
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if !app.log_search_query.is_empty() {
                    app.log_search_query.clear();
                    app.log(LogLevel::Debug, "Cleared log search");
                }
            }

            _ => {}
        }
    }

    Ok(())
}

/// Handle keyboard events for HTTP response viewer
async fn handle_http_response_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        // Tab navigation for response viewer
        KeyCode::Tab => {
            app.http_state.next_response_tab();
            app.log(
                LogLevel::Debug,
                format!(
                    "Switched to {} response tab",
                    app.http_state.current_response_tab.title()
                ),
            );
        }
        KeyCode::BackTab => {
            app.http_state.prev_response_tab();
            app.log(
                LogLevel::Debug,
                format!(
                    "Switched to {} response tab",
                    app.http_state.current_response_tab.title()
                ),
            );
        }

        // Copy response (TODO: implement clipboard)
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(response) = &app.http_state.last_response {
                // TODO: Copy response body to clipboard
                app.log(LogLevel::Info, "Response copied to clipboard");
            }
        }

        // Clear response
        KeyCode::Delete => {
            app.http_state.last_response = None;
            app.log(LogLevel::Debug, "Cleared response");
        }

        _ => {}
    }

    Ok(())
}
// TODO: previous version
/// Handle keyboard events for the logs pane
// async fn handle_log_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
//     // If in search mode, handle search input
//     if app.log_search_mode {
//         match key_event.code {
//             // Exit search mode
//             KeyCode::Esc => {
//                 app.toggle_log_search_mode();
//                 app.log(LogLevel::Debug, "Exited log search mode");
//             }
//
//             // Search input
//             KeyCode::Char(c)
//                 if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT =>
//             {
//                 app.log_search_query.push(c);
//             }
//             KeyCode::Backspace => {
//                 app.log_search_query.pop();
//             }
//             KeyCode::Delete => {
//                 app.log_search_query.clear();
//             }
//
//             // Allow scrolling even in search mode
//             KeyCode::Up | KeyCode::Char('k') => {
//                 app.scroll_logs_up();
//             }
//             KeyCode::Down | KeyCode::Char('j') => {
//                 app.scroll_logs_down();
//             }
//             KeyCode::Home | KeyCode::Char('g') => {
//                 app.scroll_logs_to_top();
//             }
//             KeyCode::End | KeyCode::Char('G') => {
//                 app.scroll_logs_to_bottom();
//             }
//
//             _ => {}
//         }
//     } else {
//         // Normal log navigation mode
//         match key_event.code {
//             // Enter search mode
//             KeyCode::Char('/') => {
//                 app.toggle_log_search_mode();
//                 app.log(LogLevel::Debug, "Entered log search mode");
//             }
//
//             // Scroll navigation (vim-style)
//             KeyCode::Up | KeyCode::Char('k') => {
//                 app.scroll_logs_up();
//             }
//             KeyCode::Down | KeyCode::Char('j') => {
//                 app.scroll_logs_down();
//             }
//
//             // Page navigation
//             KeyCode::PageUp | KeyCode::Char('u')
//                 if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
//             {
//                 // Page up - scroll multiple lines
//                 for _ in 0..10 {
//                     app.scroll_logs_up();
//                 }
//             }
//             KeyCode::PageDown | KeyCode::Char('d')
//                 if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
//             {
//                 // Page down - scroll multiple lines
//                 for _ in 0..10 {
//                     app.scroll_logs_down();
//                 }
//             }
//
//             // Jump to top/bottom (vim-style)
//             KeyCode::Home | KeyCode::Char('g') => {
//                 app.scroll_logs_to_top();
//                 app.log(LogLevel::Debug, "Jumped to top of logs");
//             }
//             KeyCode::End | KeyCode::Char('G') => {
//                 app.scroll_logs_to_bottom();
//                 app.log(LogLevel::Debug, "Jumped to bottom of logs");
//             }
//
//             // Clear search (if any)
//             KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
//                 if !app.log_search_query.is_empty() {
//                     app.log_search_query.clear();
//                     app.log(LogLevel::Debug, "Cleared log search");
//                 }
//             }
//
//             _ => {}
//         }
//     }
//
//     Ok(())
// }
//
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
