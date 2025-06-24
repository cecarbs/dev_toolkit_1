use crate::models::{LogEntry, LogLevel};
use crate::modes::automation::AutomationState;
use crate::services::AuthService;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

/// Different modes the app can be in
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Automation,
    Http, // Placeholder for future implementation
}

/// Different UI panes that can be focused
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusedPane {
    Collections,
    Form,
    Logs,
}

/// Messages that can be sent to the app from background tasks
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Log a message to the logging panel
    Log(LogLevel, String),
    /// Automation has completed successfully
    AutomationComplete,
    /// Automation has failed with an error
    AutomationFailed(String),
    /// Progress update from automation
    AutomationProgress(String),
    /// Request to quit the application
    Quit,
}

/// Global app state that coordinates everything
pub struct App {
    /// Current mode (Automation or HTTP)
    pub current_mode: AppMode,

    /// Currently focused pane
    pub focused_pane: FocusedPane,

    /// Automation mode state
    pub automation_state: AutomationState,

    /// Authentication service
    pub auth_service: AuthService,

    /// Log entries for the logging panel
    pub log_entries: Vec<LogEntry>,

    /// Whether the logging panel is visible
    pub show_logs: bool,

    /// Search query for filtering logs
    pub log_search_query: String,

    /// Whether the login popup is visible
    pub show_login_popup: bool,

    /// Login form state
    pub login_username: String,
    pub login_password: String,
    pub login_error: Option<String>,

    /// Whether the app should quit
    pub should_quit: bool,

    /// Channel for receiving messages from background tasks
    pub message_receiver: Arc<Mutex<mpsc::UnboundedReceiver<AppMessage>>>,

    /// Channel for sending messages to background tasks
    pub message_sender: mpsc::UnboundedSender<AppMessage>,
}

impl App {
    /// Create a new app instance
    pub fn new() -> Self {
        let (message_sender, message_receiver) = mpsc::unbounded_channel();

        let mut app = Self {
            current_mode: AppMode::Automation,
            focused_pane: FocusedPane::Form, // Start with form focused
            automation_state: AutomationState::new(),
            auth_service: AuthService::new(),
            log_entries: Vec::new(),
            show_logs: true, // Changed from false to true - logs open by default
            log_search_query: String::new(),
            show_login_popup: false,
            login_username: String::new(),
            login_password: String::new(),
            login_error: None,
            should_quit: false,
            message_receiver: Arc::new(Mutex::new(message_receiver)),
            message_sender,
        };

        // Log initial startup message
        app.log(LogLevel::Info, "Application started");
        app
    }

    /// Process any pending messages from background tasks
    pub async fn process_messages(&mut self) -> Result<()> {
        // Collect all pending messages first, then drop the receiver lock
        let messages = {
            let mut receiver = self.message_receiver.lock().await;
            let mut collected_messages = Vec::new();

            // Collect all available messages without blocking
            while let Ok(message) = receiver.try_recv() {
                collected_messages.push(message);
            }

            collected_messages
            // receiver lock is dropped here
        };

        // Now process the collected messages without holding the lock
        for message in messages {
            match message {
                AppMessage::Log(level, message) => {
                    self.log_entries.push(LogEntry::new(level, message));

                    // Keep log entries under a reasonable limit to prevent memory issues
                    if self.log_entries.len() > 1000 {
                        self.log_entries.drain(0..100); // Remove oldest 100 entries
                    }
                }
                AppMessage::AutomationComplete => {
                    self.automation_state.set_running(false);
                    self.log(LogLevel::Success, "Automation completed successfully");
                }
                AppMessage::AutomationFailed(error) => {
                    self.automation_state.set_running(false);
                    self.log(LogLevel::Error, format!("Automation failed: {}", error));
                }
                AppMessage::AutomationProgress(progress) => {
                    self.log(LogLevel::Info, progress);
                }
                AppMessage::Quit => {
                    self.should_quit = true;
                }
            }
        }

        Ok(())
    }

    /// Add a log entry
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        self.log_entries.push(LogEntry::new(level, message));

        // Keep log entries under a reasonable limit
        if self.log_entries.len() > 1000 {
            self.log_entries.drain(0..100);
        }
    }

    /// Get filtered log entries based on search query
    pub fn get_filtered_logs(&self) -> Vec<&LogEntry> {
        self.log_entries
            .iter()
            .filter(|entry| entry.matches_search(&self.log_search_query))
            .collect()
    }

    /// Toggle the logging panel visibility
    pub fn toggle_logs(&mut self) {
        self.show_logs = !self.show_logs;
        let status = if self.show_logs { "opened" } else { "closed" };

        // Add some debug info to help troubleshoot
        self.log_entries.push(LogEntry::new(
            LogLevel::Info,
            format!("Logging panel {} (show_logs={})", status, self.show_logs),
        ));

        // Keep log entries under a reasonable limit
        if self.log_entries.len() > 1000 {
            self.log_entries.drain(0..100);
        }
    }

    /// Switch to a different mode
    pub fn switch_mode(&mut self, mode: AppMode) {
        if mode != self.current_mode {
            self.current_mode = mode.clone();
            self.log(LogLevel::Info, format!("Switched to {:?} mode", mode));
        }
    }

    /// Switch focus to a different pane
    pub fn focus_pane(&mut self, pane: FocusedPane) {
        self.focused_pane = pane.clone();
        self.log(LogLevel::Debug, format!("Focused {:?} pane", pane));
    }

    /// Show the login popup
    pub fn show_login(&mut self) {
        self.show_login_popup = true;
        self.login_username.clear();
        self.login_password.clear();
        self.login_error = None;
        self.log(LogLevel::Debug, "Login popup opened");
    }

    /// Hide the login popup
    pub fn hide_login(&mut self) {
        self.show_login_popup = false;
        self.login_username.clear();
        self.login_password.clear();
        self.login_error = None;
        self.log(LogLevel::Debug, "Login popup closed");
    }

    /// Attempt to login with current form data
    pub fn attempt_login(&mut self) -> bool {
        // Validate credentials format
        match crate::services::AuthService::validate_credentials(
            &self.login_username,
            &self.login_password,
        ) {
            Ok(()) => {
                // Store credentials in auth service
                match self
                    .auth_service
                    .store_credentials(self.login_username.clone(), self.login_password.clone())
                {
                    Ok(()) => {
                        self.log(
                            LogLevel::Success,
                            format!("Logged in as: {}", self.login_username),
                        );
                        self.hide_login();
                        true
                    }
                    Err(err) => {
                        self.login_error = Some(err);
                        false
                    }
                }
            }
            Err(err) => {
                self.login_error = Some(err);
                false
            }
        }
    }

    /// Start automation process (this will be called from Send button)
    pub async fn start_automation(&mut self) -> Result<()> {
        self.log(LogLevel::Debug, "start_automation() called");

        if self.automation_state.is_running {
            self.log(LogLevel::Warn, "Automation is already running");
            return Ok(());
        }

        // Check if fields are valid
        self.log(LogLevel::Debug, "Checking if fields are valid...");
        if !self.automation_state.is_valid() {
            let errors = self.automation_state.get_validation_errors();
            for error in &errors {
                self.log(LogLevel::Error, error.clone());
            }
            return Ok(());
        }
        self.log(LogLevel::Debug, "Fields validation passed");

        // Check if we have credentials
        self.log(LogLevel::Debug, "Checking credentials...");
        if !self.auth_service.has_credentials() {
            self.log(
                LogLevel::Error,
                "Cannot start automation: no credentials provided",
            );
            self.show_login();
            return Ok(());
        }
        self.log(LogLevel::Debug, "Credentials check passed");

        // Get credentials from auth service
        let credentials = self.auth_service.get_credentials().unwrap();

        self.automation_state.set_running(true);
        self.log(LogLevel::Info, "🚀 Starting browser automation...");

        // Clone the data we need for the background task
        let fields = self.automation_state.fields.clone();
        let website_config = self.automation_state.website_config.clone();
        let sender = self.message_sender.clone();

        self.log(LogLevel::Debug, "Spawning browser automation task...");

        // Spawn the browser automation task
        tokio::spawn(async move {
            use crate::modes::BrowserEngine;

            let browser_engine = BrowserEngine::new(sender.clone());

            // Send initial message to confirm task started
            let _ = sender.send(AppMessage::Log(
                LogLevel::Debug,
                "Browser automation task spawned successfully".to_string(),
            ));

            match browser_engine
                .run_automation(fields, credentials, website_config)
                .await
            {
                Ok(()) => {
                    browser_engine.send_completion().await;
                }
                Err(error) => {
                    let error_msg = format!("Browser automation failed: {}", error);
                    let _ = sender.send(AppMessage::Log(LogLevel::Error, error_msg.clone()));
                    browser_engine.send_failure(error_msg).await;
                }
            }
        });

        self.log(LogLevel::Debug, "Browser automation task spawned");
        Ok(())
    }

    /// Request app shutdown
    pub fn quit(&mut self) {
        self.should_quit = true;
        self.log(LogLevel::Info, "Application shutting down");
    }

    /// Get a clone of the message sender for background tasks
    pub fn get_message_sender(&self) -> mpsc::UnboundedSender<AppMessage> {
        self.message_sender.clone()
    }
}
