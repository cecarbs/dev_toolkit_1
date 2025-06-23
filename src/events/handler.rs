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
        // Toggle logging panel
        KeyCode::Char('l') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.toggle_logs();
            return Ok(());
        }
        // Switch modes
        KeyCode::F(1) => {
            app.switch_mode(AppMode::Automation);
            return Ok(());
        }
        KeyCode::F(2) => {
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

    match key_event.code {
        // Close logging panel
        KeyCode::Esc => {
            if app.show_logs {
                app.show_logs = false;
                app.log_search_query.clear();
            }
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

        // Template selection
        KeyCode::Char('1') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            {
                app.automation_state.selected_template = Some(0);
                app.automation_state.apply_selected_template();
            }
            app.log(LogLevel::Info, "Applied template: Quick Task");
        }
        KeyCode::Char('2') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            {
                app.automation_state.selected_template = Some(1);
                app.automation_state.apply_selected_template();
            }
            app.log(LogLevel::Info, "Applied template: Urgent Request");
        }
        KeyCode::Char('3') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            {
                app.automation_state.selected_template = Some(2);
                app.automation_state.apply_selected_template();
            }
            app.log(LogLevel::Info, "Applied template: Weekly Report");
        }

        // Start automation
        KeyCode::Enter if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            if !app.automation_state.is_running {
                // Set demo credentials if none are provided (temporary)
                if app.automation_state.credentials.is_none() {
                    app.automation_state
                        .set_credentials("demo_user".to_string(), "demo_password".to_string());
                    app.log(LogLevel::Info, "Using demo credentials for testing");
                }

                app.start_automation().await?;
            }
        }

        // Set demo credentials manually (temporary hotkey)
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.automation_state
                .set_credentials("demo_user".to_string(), "demo_password".to_string());
            app.log(LogLevel::Info, "Demo credentials set");
        }

        // Clear credentials
        KeyCode::Char('x') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.automation_state.clear_credentials();
            app.log(LogLevel::Info, "Credentials cleared");
        }

        // Text input for the focused field
        KeyCode::Char(c) => {
            if let Some(field) = app.automation_state.get_focused_field_mut() {
                field.value.push(c);
            }
        }

        // Backspace for text editing
        KeyCode::Backspace => {
            if let Some(field) = app.automation_state.get_focused_field_mut() {
                field.value.pop();
            }
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
        "  Ctrl+L: Toggle logging panel".to_string(),
        "  F1: Switch to Automation mode".to_string(),
        "  F2: Switch to HTTP mode".to_string(),
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
                "  Ctrl+Enter: Start automation".to_string(),
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
        ]);
    }

    help
}
