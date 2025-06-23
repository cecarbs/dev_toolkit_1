use crate::models::{LogEntry, LogLevel};
use crate::modes::automation::AutomationState;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

/// Different modes the app can be in
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Automation,
    Http, // Placeholder for future implementation
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

    /// Automation mode state
    pub automation_state: AutomationState,

    /// Log entries for the logging panel
    pub log_entries: Vec<LogEntry>,

    /// Whether the logging panel is visible
    pub show_logs: bool,

    /// Search query for filtering logs
    pub log_search_query: String,

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
            automation_state: AutomationState::new(),
            log_entries: Vec::new(),
            show_logs: false,
            log_search_query: String::new(),
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
        self.log(
            LogLevel::Debug,
            format!(
                "Logging panel {}",
                if self.show_logs { "opened" } else { "closed" }
            ),
        );
    }

    /// Switch to a different mode
    pub fn switch_mode(&mut self, mode: AppMode) {
        if mode != self.current_mode {
            self.current_mode = mode.clone();
            self.log(LogLevel::Info, format!("Switched to {:?} mode", mode));
        }
    }

    /// Start automation process (this will be called from UI)
    pub async fn start_automation(&mut self) -> Result<()> {
        if self.automation_state.is_running {
            self.log(LogLevel::Warn, "Automation is already running");
            return Ok(());
        }

        if !self.automation_state.is_valid() {
            self.log(
                LogLevel::Error,
                "Cannot start automation: some fields are empty",
            );
            return Ok(());
        }

        // Check if we have credentials
        if self.automation_state.credentials.is_none() {
            self.log(
                LogLevel::Error,
                "Cannot start automation: no credentials provided",
            );
            return Ok(());
        }

        self.automation_state.set_running(true);
        self.log(LogLevel::Info, "Starting browser automation...");

        // Clone the data we need for the background task
        let fields = self.automation_state.fields.clone();
        let credentials = self.automation_state.credentials.clone().unwrap();
        let website_config = self.automation_state.website_config.clone();
        let sender = self.message_sender.clone();

        // Spawn the browser automation task
        tokio::spawn(async move {
            use crate::modes::BrowserEngine;

            let browser_engine = BrowserEngine::new(sender.clone());

            match browser_engine
                .run_automation(fields, credentials, website_config)
                .await
            {
                Ok(()) => {
                    browser_engine.send_completion().await;
                }
                Err(error) => {
                    browser_engine
                        .send_failure(format!("Browser automation failed: {}", error))
                        .await;
                }
            }
        });

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
