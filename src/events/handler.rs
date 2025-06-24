use crate::app::{App, AppMode};
use crate::models::LogLevel;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard events and update app state accordingly
pub async fn handle_key_event(app: &mut App, key_event: KeyEvent) -> Result<()> {
    // Global keybindings that work in all modes
    match key_event.code {
        // Quit application
        KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
            return Ok(());
        }
        // Toggle logging panel - Changed to F2 for Mac compatibility
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
        _ => {}
    }

    // Mode-specific event handling
    match app.current_mode {
        AppMode::Automation => handle_automation_keys(app, key_event).await?,
        AppMode::Http => handle_http_keys(app, key_event).await?,
    }

    Ok(())
}

/// Handle keyboard events specific to automation mode
async fn handle_automation_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
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
            return Ok(()); // Return early to prevent text input
        }
        KeyCode::Char('2') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.log(LogLevel::Debug, "Ctrl+2 detected - applying template 2");
            {
                app.automation_state.selected_template = Some(1);
                app.automation_state.apply_selected_template();
            }
            app.log(LogLevel::Info, "Applied template: Urgent Request");
            return Ok(()); // Return early to prevent text input
        }
        KeyCode::Char('3') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.log(LogLevel::Debug, "Ctrl+3 detected - applying template 3");
            {
                app.automation_state.selected_template = Some(2);
                app.automation_state.apply_selected_template();
            }
            app.log(LogLevel::Info, "Applied template: Weekly Report");
            return Ok(()); // Return early to prevent text input
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

        // Start automation - Changed to F3 for Mac compatibility
        // KeyCode::F(3) => {
        //     app.log(
        //         LogLevel::Debug,
        //         "F3 detected - attempting to start automation",
        //     );
        //     if !app.automation_state.is_running {
        //         // Set demo credentials if none are provided (temporary)
        //         if app.automation_state.credentials.is_none() {
        //             app.automation_state
        //                 .set_credentials("demo_user".to_string(), "demo_password".to_string());
        //             app.log(LogLevel::Info, "Using demo credentials for testing");
        //         }
        //
        //         if let Err(e) = app.start_automation().await {
        //             app.log(
        //                 LogLevel::Error,
        //                 format!("Failed to start automation: {}", e),
        //             );
        //         }
        //     } else {
        //         app.log(LogLevel::Warn, "Automation is already running");
        //     }
        //     return Ok(()); // Return early to prevent text input
        // }

        // Navigation between fields
        KeyCode::Tab => {
            app.automation_state.focus_next_field();
            app.log(LogLevel::Debug, "Moved to next field");
            return Ok(()); // Return early to prevent text input
        }
        KeyCode::BackTab => {
            app.automation_state.focus_prev_field();
            app.log(LogLevel::Debug, "Moved to previous field");
            return Ok(()); // Return early to prevent text input
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
            return Ok(()); // Return early to prevent text input
        }

        // Text input for the focused field (only if no modifiers)
        KeyCode::Char(c) if key_event.modifiers.is_empty() => {
            if let Some(field) = app.automation_state.get_focused_field_mut() {
                field.value.push(c);
            }
        }

        // Backspace for text editing (only if no modifiers)
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

/// Handle keyboard events specific to HTTP mode (placeholder for now)
async fn handle_http_keys(app: &mut App, _key_event: KeyEvent) -> Result<()> {
    // TODO: Implement HTTP mode key handling
    app.log(LogLevel::Info, "HTTP mode is not implemented yet");
    Ok(())
}

/// Get help text for the current mode
pub fn get_help_text(app: &App) -> Vec<String> {
    let mut help = vec![
        "Global Keybindings:".to_string(),
        "  Ctrl+Q: Quit application".to_string(),
        "  F2: Toggle logging panel".to_string(),
        "  F1: Switch to Automation mode".to_string(),
        "  F4: Switch to HTTP mode".to_string(),
        "".to_string(),
    ];

    match app.current_mode {
        AppMode::Automation => {
            help.extend(vec![
                "Automation Mode:".to_string(),
                "  Tab/Shift+Tab: Navigate between fields".to_string(),
                "  Ctrl+1/2/3: Apply templates".to_string(),
                "  Ctrl+C: Set demo credentials".to_string(),
                "  Ctrl+X: Clear credentials".to_string(),
                "  F3: Start automation".to_string(),
                "  Delete: Clear current field".to_string(),
                "  Esc: Close logging panel".to_string(),
            ]);
        }
        AppMode::Http => {
            help.extend(vec![
                "HTTP Mode:".to_string(),
                "  (Not implemented yet)".to_string(),
            ]);
        }
    }

    if app.show_logs {
        help.extend(vec![
            "".to_string(),
            "Log Search:".to_string(),
            "  Type to search logs".to_string(),
            "  Delete: Clear search".to_string(),
            "  Esc: Close logging panel".to_string(),
        ]);
    }

    help
}
