use crate::models::{AppConfig, LogEntry, LogLevel, NodeType, TreeState};
use crate::modes::automation::AutomationState;
use crate::services::{AuthService, TemplateStorage};
use anyhow::Result;
use std::collections::HashMap;
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
    /// Application configuration
    pub config: AppConfig,

    /// Template storage service
    pub template_storage: TemplateStorage,

    /// Collections tree state
    pub tree_state: TreeState,

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

    /// Template creation dialog state
    pub show_template_dialog: bool,
    pub template_dialog_name: String,
    pub template_dialog_folder: String,
    pub template_dialog_description: String,
    pub template_dialog_focused_field: usize, // 0=name, 1=folder, 2=description

    /// Folder creation dialog state
    pub show_folder_dialog: bool,
    pub folder_dialog_name: String,
    pub folder_dialog_parent: String,
    pub folder_dialog_error: Option<String>,
}

impl App {
    /// Create a new app instance
    pub fn new() -> Self {
        let (message_sender, message_receiver) = mpsc::unbounded_channel();

        // Load or create configuration
        let config = AppConfig::load().unwrap_or_else(|e| {
            eprintln!("Failed to load config: {}, using defaults", e);
            AppConfig::default()
        });

        // Initialize template storage
        let template_storage = TemplateStorage::new(config.clone());
        if let Err(e) = template_storage.initialize() {
            eprintln!("Failed to initialize template storage: {}", e);
        }

        // Build the initial tree state
        let tree_state = Self::build_initial_tree_state(&template_storage);

        let show_logs = config.show_logs_on_startup;

        let mut app = Self {
            config,
            template_storage,
            tree_state,
            current_mode: AppMode::Automation,
            focused_pane: FocusedPane::Form, // Start with form focused
            automation_state: AutomationState::new(),
            auth_service: AuthService::new(),
            log_entries: Vec::new(),
            show_logs,
            log_search_query: String::new(),
            show_login_popup: false,
            login_username: String::new(),
            login_password: String::new(),
            login_error: None,
            should_quit: false,
            message_receiver: Arc::new(Mutex::new(message_receiver)),
            message_sender,
            show_template_dialog: false,
            template_dialog_name: String::new(),
            template_dialog_folder: String::new(),
            template_dialog_description: String::new(),
            template_dialog_focused_field: 0,
            show_folder_dialog: false,
            folder_dialog_name: String::new(),
            folder_dialog_parent: String::new(),
            folder_dialog_error: None,
        };

        // Log initial startup message
        app.log(LogLevel::Info, "Application started");
        app.log(
            LogLevel::Info,
            format!(
                "Templates directory: {}",
                app.template_storage.get_templates_directory_display()
            ),
        );
        app
    }

    /// Build the initial tree state from template storage
    fn build_initial_tree_state(template_storage: &TemplateStorage) -> TreeState {
        // Get all folders and templates
        let folders = template_storage.list_all_folders().unwrap_or_default();
        let mut templates_by_folder = HashMap::new();

        // Get templates for each folder
        for folder in &folders {
            if let Ok(templates) = template_storage.list_templates_in_folder(folder) {
                if !templates.is_empty() {
                    templates_by_folder.insert(folder.clone(), templates);
                }
            }
        }

        // Also check for templates in the root directory
        if let Ok(root_templates) = template_storage.list_templates_in_folder("") {
            if !root_templates.is_empty() {
                templates_by_folder.insert("".to_string(), root_templates);
            }
        }

        TreeState::build_from_storage(folders, templates_by_folder)
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
    /// Refresh tree state from template storage
    pub async fn refresh_tree_from_storage(&mut self) -> Result<()> {
        use std::collections::HashMap;

        // Get all folders and templates from storage
        let folders = self.template_storage.list_all_folders().unwrap_or_default();
        let mut templates_by_folder = HashMap::new();

        // Get templates for each folder
        for folder in &folders {
            if let Ok(templates) = self.template_storage.list_templates_in_folder(folder) {
                if !templates.is_empty() {
                    templates_by_folder.insert(folder.clone(), templates);
                }
            }
        }

        // Also check for templates in the root directory
        if let Ok(root_templates) = self.template_storage.list_templates_in_folder("") {
            if !root_templates.is_empty() {
                templates_by_folder.insert("".to_string(), root_templates);
            }
        }

        // Rebuild tree state
        self.tree_state =
            crate::models::TreeState::build_from_storage(folders, templates_by_folder);

        self.log(LogLevel::Debug, "Tree state refreshed from storage");
        Ok(())
    }

    /// Load a template from storage into the automation form
    pub async fn load_template_into_form(&mut self, template_path: &str) -> Result<()> {
        // Parse the template path to get folder and template name
        let (folder_path, template_name) = if let Some(pos) = template_path.rfind('/') {
            (&template_path[..pos], &template_path[pos + 1..])
        } else {
            ("", template_path)
        };

        // Load the template from storage
        match self
            .template_storage
            .load_template(folder_path, template_name)
        {
            Ok(stored_template) => {
                // Apply the template to the form fields
                stored_template
                    .template
                    .apply_to_fields(&mut self.automation_state.fields);
                self.log(
                    LogLevel::Success,
                    format!("Loaded template: {}", template_name),
                );
                Ok(())
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("Failed to load template: {}", e));
                Err(e)
            }
        }
    }

    /// Create a new template from current form state
    pub async fn create_template_from_form(
        &mut self,
        folder_path: &str,
        template_name: &str,
    ) -> Result<()> {
        use crate::models::AutomationTemplate;

        // Create template from current form values
        let mut template = AutomationTemplate::new(template_name, "Template created from form");

        // Add current field values to template
        for field in &self.automation_state.fields {
            if !field.value.is_empty() {
                template = template.with_field(&field.name, &field.value);
            }
        }

        // Save to storage
        match self
            .template_storage
            .save_template(folder_path, template_name, template)
        {
            Ok(_) => {
                self.log(
                    LogLevel::Success,
                    format!("Created template: {}", template_name),
                );
                // Refresh tree to show new template
                self.refresh_tree_from_storage().await?;
                Ok(())
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("Failed to save template: {}", e));
                Err(e)
            }
        }
    }

    /// Delete a template from storage
    pub async fn delete_template(&mut self, template_path: &str) -> Result<()> {
        let (folder_path, template_name) = if let Some(pos) = template_path.rfind('/') {
            (&template_path[..pos], &template_path[pos + 1..])
        } else {
            ("", template_path)
        };

        self.template_storage
            .delete_template(folder_path, template_name)?;
        self.log(
            LogLevel::Success,
            format!("Deleted template: {}", template_name),
        );
        self.refresh_tree_from_storage().await?;
        Ok(())
    }

    /// Show the template creation dialog
    pub fn show_template_creation_dialog(&mut self) {
        // Pre-populate with smart defaults
        let focused_folder = if let Some(focused_node) = self.tree_state.get_focused_node() {
            match focused_node.node_type {
                NodeType::Folder => focused_node.path.clone(),
                NodeType::Template => {
                    // Get parent folder
                    if let Some(parent_pos) = focused_node.path.rfind('/') {
                        focused_node.path[..parent_pos].to_string()
                    } else {
                        "".to_string()
                    }
                }
            }
        } else {
            "".to_string()
        };

        self.show_template_dialog = true;
        self.template_dialog_name = "New Template".to_string();
        self.template_dialog_folder = focused_folder;
        self.template_dialog_description = "Template created from form".to_string();
        self.template_dialog_focused_field = 0;

        self.log(LogLevel::Debug, "Template creation dialog opened");
    }

    /// Hide the template creation dialog
    pub fn hide_template_creation_dialog(&mut self) {
        self.show_template_dialog = false;
        self.template_dialog_name.clear();
        self.template_dialog_folder.clear();
        self.template_dialog_description.clear();
        self.template_dialog_focused_field = 0;

        self.log(LogLevel::Debug, "Template creation dialog closed");
    }

    /// Create template with dialog values
    pub async fn create_template_from_dialog(&mut self) -> Result<()> {
        if self.template_dialog_name.trim().is_empty() {
            self.log(LogLevel::Error, "Template name cannot be empty");
            return Ok(());
        }

        use crate::models::AutomationTemplate;

        // Create template from current form values
        let mut template = AutomationTemplate::new(
            &self.template_dialog_name,
            &self.template_dialog_description,
        );

        // Add current field values to template
        for field in &self.automation_state.fields {
            if !field.value.is_empty() {
                template = template.with_field(&field.name, &field.value);
            }
        }

        // Save to storage
        match self.template_storage.save_template(
            &self.template_dialog_folder,
            &self.template_dialog_name,
            template,
        ) {
            Ok(_) => {
                self.log(
                    LogLevel::Success,
                    format!(
                        "Created template '{}' in folder '{}'",
                        self.template_dialog_name,
                        if self.template_dialog_folder.is_empty() {
                            "Root"
                        } else {
                            &self.template_dialog_folder
                        }
                    ),
                );

                // Refresh tree and hide dialog
                self.refresh_tree_from_storage().await?;
                self.hide_template_creation_dialog();
                Ok(())
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("Failed to save template: {}", e));
                Err(e)
            }
        }
    }

    pub fn show_folder_creation_dialog(&mut self) {
        // Pre-populate with smart defaults based on focused node
        let parent_folder = if let Some(focused_node) = self.tree_state.get_focused_node() {
            match focused_node.node_type {
                NodeType::Folder => focused_node.path.clone(),
                NodeType::Template => {
                    // Get parent folder of the template
                    if let Some(parent_pos) = focused_node.path.rfind('/') {
                        focused_node.path[..parent_pos].to_string()
                    } else {
                        "".to_string()
                    }
                }
            }
        } else {
            "".to_string()
        };

        self.show_folder_dialog = true;
        self.folder_dialog_name = String::new();
        self.folder_dialog_parent = parent_folder;
        self.folder_dialog_error = None;

        self.log(LogLevel::Debug, "Folder creation dialog opened");
    }

    /// Hide the folder creation dialog
    pub fn hide_folder_creation_dialog(&mut self) {
        self.show_folder_dialog = false;
        self.folder_dialog_name.clear();
        self.folder_dialog_parent.clear();
        self.folder_dialog_error = None;

        self.log(LogLevel::Debug, "Folder creation dialog closed");
    }

    /// Create folder with dialog values
    pub async fn create_folder_from_dialog(&mut self) -> Result<()> {
        // Validate folder name
        if let Err(error) =
            self.validate_folder_name(&self.folder_dialog_name, &self.folder_dialog_parent)
        {
            self.folder_dialog_error = Some(error);
            return Ok(());
        }

        // Create the full folder path
        let full_path = if self.folder_dialog_parent.is_empty() {
            self.folder_dialog_name.clone()
        } else {
            format!("{}/{}", self.folder_dialog_parent, self.folder_dialog_name)
        };

        // Create the folder on disk
        let templates_dir = self.config.get_templates_directory();
        let folder_path = templates_dir.join(&full_path);

        match std::fs::create_dir_all(&folder_path) {
            Ok(()) => {
                self.log(
                    LogLevel::Success,
                    format!(
                        "Created folder: {}",
                        if full_path.is_empty() {
                            "Root"
                        } else {
                            &full_path
                        }
                    ),
                );

                // Refresh tree and hide dialog
                self.refresh_tree_from_storage().await?;
                self.hide_folder_creation_dialog();
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to create folder: {}", e);
                self.folder_dialog_error = Some(error_msg.clone());
                self.log(LogLevel::Error, error_msg);
                Ok(())
            }
        }
    }

    /// Validate folder name and path
    fn validate_folder_name(&self, name: &str, parent: &str) -> Result<(), String> {
        // Check if name is empty
        if name.trim().is_empty() {
            return Err("Folder name cannot be empty".to_string());
        }

        // Check for invalid characters
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        if name.chars().any(|c| invalid_chars.contains(&c)) {
            return Err("Folder name contains invalid characters".to_string());
        }

        // Check if folder already exists
        let full_path = if parent.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", parent, name)
        };

        let templates_dir = self.config.get_templates_directory();
        let folder_path = templates_dir.join(&full_path);

        if folder_path.exists() {
            return Err("Folder already exists".to_string());
        }

        Ok(())
    }
}
