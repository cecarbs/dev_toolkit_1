use crate::models::http::HttpState;
use crate::models::http_client::{
    HttpAuth, HttpHeader, HttpMethod, HttpRequest, HttpRequestBody, HttpResponse,
};
use crate::models::{
    AppConfig, ClipboardItem, ClipboardOperation, LogEntry, LogLevel, NodeType, TreeState,
};
use crate::modes::automation::AutomationState;
use crate::services::{AuthService, TemplateStorage};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Edit,
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
    /// Response from API call
    HttpResponseReceived(HttpResponse),
    /// HTTP request failed - clear sending state
    HttpRequestFailed(String),
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

    /// HTTP client mode state
    pub http_state: HttpState,

    /// Authentication service
    pub auth_service: AuthService,

    /// Log entries for the logging panel
    pub log_entries: Vec<LogEntry>,

    /// Whether the logging panel is visible
    // pub show_logs: bool,

    /// Current scroll position in logs (0 = bottom/newest)
    pub log_scroll_position: usize,

    /// Whether we're in log search mode
    pub log_search_mode: bool,

    /// Search query for filtering logs
    pub log_search_query: String,

    /// Whether the login popup is visible
    pub show_login_popup: bool,

    /// Login form state
    pub login_username: String,
    pub login_password: String,
    pub login_error: Option<String>,
    pub login_focused_field: usize, // 0 - username, 1 - password

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

    pub show_rename_dialog: bool,
    pub rename_dialog_original_name: String,
    pub rename_dialog_new_name: String,
    pub rename_dialog_path: String,
    pub rename_dialog_is_folder: bool,
    pub rename_dialog_error: Option<String>,

    /// Clipboard for cut/copy/paste operations
    pub clipboard: Option<ClipboardItem>,

    /// Folder deletion confirmation dialog
    pub show_delete_confirmation_dialog: bool,
    pub delete_confirmation_item_name: String,
    pub delete_confirmation_item_path: String,
    pub delete_confirmation_is_folder: bool,
    pub delete_confirmation_contents: Vec<String>, // List of what will be deleted

    /// Current input mode for form fields
    pub input_mode: InputMode,

    /// Cursor position within the currently focused field
    pub form_field_cursor_index: usize,

    /// Help dialog state
    pub show_help_dialog: bool,
    pub help_search_query: String,
    pub help_selected_section: usize, // 0 = All, 1 = Global, 2 = Collections, etc.
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
            http_state: HttpState::new(),
            auth_service: AuthService::new(),
            log_entries: Vec::new(),
            // show_logs,
            log_search_query: String::new(),
            log_scroll_position: 0,
            log_search_mode: false,
            show_login_popup: false,
            login_username: String::new(),
            login_password: String::new(),
            login_error: None,
            login_focused_field: 0,
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
            show_rename_dialog: false,
            rename_dialog_original_name: String::new(),
            rename_dialog_new_name: String::new(),
            rename_dialog_path: String::new(),
            rename_dialog_is_folder: false,
            rename_dialog_error: None,
            clipboard: None,
            show_delete_confirmation_dialog: false,
            delete_confirmation_item_name: String::new(),
            delete_confirmation_item_path: String::new(),
            delete_confirmation_is_folder: false,
            delete_confirmation_contents: Vec::new(),
            input_mode: InputMode::Normal, // Prevent editing texting until explicitly in InputMode
            show_help_dialog: false,
            help_search_query: String::new(),
            help_selected_section: 0,
            form_field_cursor_index: 0,
        };

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
                AppMessage::HttpResponseReceived(response) => {
                    self.http_state.last_response = Some(response);
                    self.http_state.is_sending = false;
                }
                AppMessage::HttpRequestFailed(error) => {
                    // NEW: Add this case
                    self.http_state.is_sending = false;
                    self.log(LogLevel::Error, error);
                }
            }
        }

        Ok(())
    }

    /// Add a log entry
    // pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
    //     self.log_entries.push(LogEntry::new(level, message));
    //
    //     // Keep log entries under a reasonable limit
    //     if self.log_entries.len() > 1000 {
    //         self.log_entries.drain(0..100);
    //     }
    // }

    /// Get filtered log entries based on search query
    pub fn get_filtered_logs(&self) -> Vec<&LogEntry> {
        self.log_entries
            .iter()
            .filter(|entry| entry.matches_search(&self.log_search_query))
            .collect()
    }

    /// Scroll up in logs (towards older entries)
    pub fn scroll_logs_up(&mut self) {
        let filtered_logs = self.get_filtered_logs();
        if filtered_logs.len() > 1 {
            self.log_scroll_position = (self.log_scroll_position + 1).min(filtered_logs.len() - 1);
        }
    }

    /// Scroll down in logs (towards newer entries)
    pub fn scroll_logs_down(&mut self) {
        if self.log_scroll_position > 0 {
            self.log_scroll_position -= 1;
        }
    }

    /// Jump to top of logs (oldest)
    pub fn scroll_logs_to_top(&mut self) {
        let filtered_logs = self.get_filtered_logs();
        if !filtered_logs.is_empty() {
            self.log_scroll_position = filtered_logs.len() - 1;
        }
    }

    /// Jump to bottom of logs (newest)
    pub fn scroll_logs_to_bottom(&mut self) {
        self.log_scroll_position = 0;
    }

    /// Toggle log search mode
    pub fn toggle_log_search_mode(&mut self) {
        self.log_search_mode = !self.log_search_mode;
        if !self.log_search_mode {
            self.log_search_query.clear();
        }
    }

    /// Get logs visible in current view (accounting for scroll and search)
    pub fn get_visible_logs_for_display(
        &self,
        display_height: usize,
    ) -> (Vec<&LogEntry>, bool, bool) {
        let filtered_logs = self.get_filtered_logs();

        if filtered_logs.is_empty() {
            return (Vec::new(), false, false);
        }

        let total_logs = filtered_logs.len();
        let scroll_pos = self.log_scroll_position;

        // Calculate which logs to show
        let end_index = total_logs.saturating_sub(scroll_pos);
        let start_index = end_index.saturating_sub(display_height);

        let visible_logs = filtered_logs[start_index..end_index].to_vec();

        // Calculate scroll indicators
        let can_scroll_up = scroll_pos < total_logs.saturating_sub(1);
        let can_scroll_down = scroll_pos > 0;

        (visible_logs, can_scroll_up, can_scroll_down)
    }

    // Update the log method to auto-scroll to bottom when new logs arrive
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        self.log_entries.push(LogEntry::new(level, message));

        // Keep log entries under a reasonable limit
        if self.log_entries.len() > 1000 {
            self.log_entries.drain(0..100);
        }

        // Auto-scroll to bottom (newest) when new logs arrive, unless user has scrolled up
        if self.log_scroll_position == 0 {
            // User is at bottom, keep them there
            self.log_scroll_position = 0;
        }
        // If user has scrolled up, don't auto-scroll (let them stay where they are)
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
        self.login_focused_field = 0; // Start with username focused
        self.log(LogLevel::Debug, "Login popup opened");
    }

    /// Hide the login popup
    pub fn hide_login(&mut self) {
        self.show_login_popup = false;
        self.login_username.clear();
        self.login_password.clear();
        self.login_error = None;
        self.login_focused_field = 0;
        self.log(LogLevel::Debug, "Login popup closed");
    }

    /// Attempt to login with current form data
    pub fn attempt_login(&mut self) -> bool {
        // Clear previous error
        self.login_error = None;

        // Validate credentials format
        if self.login_username.trim().is_empty() {
            self.login_error = Some("Username cannot be empty".to_string());
            return false;
        }

        if self.login_password.is_empty() {
            self.login_error = Some("Password cannot be empty".to_string());
            return false;
        }

        if self.login_username.len() < 3 {
            self.login_error = Some("Username must be at least 3 characters".to_string());
            return false;
        }

        if self.login_password.len() < 3 {
            self.login_error = Some("Password must be at least 3 characters".to_string());
            return false;
        }

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

    /// Attempt to login with current form data
    // pub fn attempt_login(&mut self) -> bool {
    //     // Validate credentials format
    //     match crate::services::AuthService::validate_credentials(
    //         &self.login_username,
    //         &self.login_password,
    //     ) {
    //         Ok(()) => {
    //             // Store credentials in auth service
    //             match self
    //                 .auth_service
    //                 .store_credentials(self.login_username.clone(), self.login_password.clone())
    //             {
    //                 Ok(()) => {
    //                     self.log(
    //                         LogLevel::Success,
    //                         format!("Logged in as: {}", self.login_username),
    //                     );
    //                     self.hide_login();
    //                     true
    //                 }
    //                 Err(err) => {
    //                     self.login_error = Some(err);
    //                     false
    //                 }
    //             }
    //         }
    //         Err(err) => {
    //             self.login_error = Some(err);
    //             false
    //         }
    //     }
    // }

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

    /// Show the rename dialog for the currently focused node
    pub fn show_rename_dialog(&mut self) {
        if let Some(focused_node) = self.tree_state.get_focused_node() {
            self.show_rename_dialog = true;
            self.rename_dialog_original_name = focused_node.name.clone();
            self.rename_dialog_new_name = focused_node.name.clone();
            self.rename_dialog_path = focused_node.path.clone();
            self.rename_dialog_is_folder = focused_node.node_type == NodeType::Folder;
            self.rename_dialog_error = None;

            let item_type = if self.rename_dialog_is_folder {
                "folder"
            } else {
                "template"
            };
            self.log(
                LogLevel::Debug,
                format!(
                    "Rename dialog opened for {}: {}",
                    item_type, focused_node.name
                ),
            );
        }
    }

    /// Hide the rename dialog
    pub fn hide_rename_dialog(&mut self) {
        self.show_rename_dialog = false;
        self.rename_dialog_original_name.clear();
        self.rename_dialog_new_name.clear();
        self.rename_dialog_path.clear();
        self.rename_dialog_is_folder = false;
        self.rename_dialog_error = None;

        self.log(LogLevel::Debug, "Rename dialog closed");
    }

    /// Perform the rename operation
    pub async fn rename_item_from_dialog(&mut self) -> Result<()> {
        // Validate new name
        if let Err(error) = self.validate_rename(
            &self.rename_dialog_new_name,
            &self.rename_dialog_path,
            self.rename_dialog_is_folder,
        ) {
            self.rename_dialog_error = Some(error);
            return Ok(());
        }

        if self.rename_dialog_is_folder {
            self.rename_folder().await
        } else {
            self.rename_template().await
        }
    }

    /// Rename a folder
    async fn rename_folder(&mut self) -> Result<()> {
        let templates_dir = self.config.get_templates_directory();
        let old_path = templates_dir.join(&self.rename_dialog_path);

        // Calculate new path
        let new_path = if let Some(parent_pos) = self.rename_dialog_path.rfind('/') {
            let parent = &self.rename_dialog_path[..parent_pos];
            format!("{}/{}", parent, self.rename_dialog_new_name)
        } else {
            self.rename_dialog_new_name.clone()
        };
        let new_full_path = templates_dir.join(&new_path);

        match std::fs::rename(&old_path, &new_full_path) {
            Ok(()) => {
                self.log(
                    LogLevel::Success,
                    format!(
                        "Renamed folder '{}' to '{}'",
                        self.rename_dialog_original_name, self.rename_dialog_new_name
                    ),
                );

                // Refresh tree and hide dialog
                self.refresh_tree_from_storage().await?;
                self.hide_rename_dialog();
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to rename folder: {}", e);
                self.rename_dialog_error = Some(error_msg.clone());
                self.log(LogLevel::Error, error_msg);
                Ok(())
            }
        }
    }

    /// Rename a template
    async fn rename_template(&mut self) -> Result<()> {
        // Parse the template path
        let (folder_path, _old_name) = if let Some(pos) = self.rename_dialog_path.rfind('/') {
            (
                &self.rename_dialog_path[..pos],
                &self.rename_dialog_path[pos + 1..],
            )
        } else {
            ("", self.rename_dialog_path.as_str())
        };

        let templates_dir = self.config.get_templates_directory();
        let folder_dir = templates_dir.join(folder_path);

        // Build old and new file paths
        let old_filename = format!(
            "{}.json",
            sanitize_filename(&self.rename_dialog_original_name)
        );
        let new_filename = format!("{}.json", sanitize_filename(&self.rename_dialog_new_name));
        let old_file_path = folder_dir.join(old_filename);
        let new_file_path = folder_dir.join(new_filename);

        // Load the template to update its internal name
        match std::fs::read_to_string(&old_file_path) {
            Ok(json_content) => {
                let mut stored_template: crate::services::StoredTemplate =
                    serde_json::from_str(&json_content)
                        .map_err(|e| anyhow::anyhow!("Failed to parse template: {}", e))?;

                // Update the template name
                stored_template.template.name = self.rename_dialog_new_name.clone();
                stored_template.modified_at = chrono::Utc::now();

                // Save with new name and delete old file
                let updated_json = serde_json::to_string_pretty(&stored_template)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize template: {}", e))?;

                std::fs::write(&new_file_path, updated_json)
                    .map_err(|e| anyhow::anyhow!("Failed to write renamed template: {}", e))?;

                std::fs::remove_file(&old_file_path)
                    .map_err(|e| anyhow::anyhow!("Failed to remove old template file: {}", e))?;

                self.log(
                    LogLevel::Success,
                    format!(
                        "Renamed template '{}' to '{}'",
                        self.rename_dialog_original_name, self.rename_dialog_new_name
                    ),
                );

                // Refresh tree and hide dialog
                self.refresh_tree_from_storage().await?;
                self.hide_rename_dialog();
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to read template file: {}", e);
                self.rename_dialog_error = Some(error_msg.clone());
                self.log(LogLevel::Error, error_msg);
                Ok(())
            }
        }
    }

    /// Validate rename operation
    fn validate_rename(
        &self,
        new_name: &str,
        current_path: &str,
        is_folder: bool,
    ) -> Result<(), String> {
        // Check if name is empty
        if new_name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        // Check if name is unchanged
        if new_name == self.rename_dialog_original_name {
            return Err("Name is unchanged".to_string());
        }

        // Check for invalid characters
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        if new_name.chars().any(|c| invalid_chars.contains(&c)) {
            return Err("Name contains invalid characters".to_string());
        }

        // Check if item with new name already exists in the same location
        let templates_dir = self.config.get_templates_directory();

        if is_folder {
            // For folders, check if folder with new name exists
            let parent_path = if let Some(parent_pos) = current_path.rfind('/') {
                &current_path[..parent_pos]
            } else {
                ""
            };

            let new_path = if parent_path.is_empty() {
                new_name.to_string()
            } else {
                format!("{}/{}", parent_path, new_name)
            };

            let check_path = templates_dir.join(new_path);
            if check_path.exists() {
                return Err("Folder with this name already exists".to_string());
            }
        } else {
            // For templates, check if template file with new name exists
            let folder_path = if let Some(pos) = current_path.rfind('/') {
                &current_path[..pos]
            } else {
                ""
            };

            let folder_dir = templates_dir.join(folder_path);
            let new_filename = format!("{}.json", sanitize_filename(new_name));
            let check_path = folder_dir.join(new_filename);

            if check_path.exists() {
                return Err("Template with this name already exists".to_string());
            }
        }

        Ok(())
    }

    /// Cut the currently focused item to clipboard
    pub fn cut_focused_item(&mut self) {
        if let Some(focused_node) = self.tree_state.get_focused_node() {
            let templates_dir = self.config.get_templates_directory();
            let full_path = if focused_node.node_type == NodeType::Folder {
                templates_dir.join(&focused_node.path)
            } else {
                // For templates, we need the .json file path
                let (folder_path, template_name) = if let Some(pos) = focused_node.path.rfind('/') {
                    (&focused_node.path[..pos], &focused_node.path[pos + 1..])
                } else {
                    ("", focused_node.path.as_str())
                };
                let filename = format!("{}.json", sanitize_filename(template_name));
                templates_dir.join(folder_path).join(filename)
            };

            self.clipboard = Some(ClipboardItem {
                operation: ClipboardOperation::Cut,
                item_type: focused_node.node_type.clone(),
                name: focused_node.name.clone(),
                path: focused_node.path.clone(),
                full_file_path: full_path,
            });

            self.log(
                LogLevel::Info,
                format!(
                    "Cut {}: {}",
                    if focused_node.node_type == NodeType::Folder {
                        "folder"
                    } else {
                        "template"
                    },
                    focused_node.name
                ),
            );
        }
    }

    /// Copy the currently focused item to clipboard
    pub fn copy_focused_item(&mut self) {
        if let Some(focused_node) = self.tree_state.get_focused_node() {
            let templates_dir = self.config.get_templates_directory();
            let full_path = if focused_node.node_type == NodeType::Folder {
                templates_dir.join(&focused_node.path)
            } else {
                let (folder_path, template_name) = if let Some(pos) = focused_node.path.rfind('/') {
                    (&focused_node.path[..pos], &focused_node.path[pos + 1..])
                } else {
                    ("", focused_node.path.as_str())
                };
                let filename = format!("{}.json", sanitize_filename(template_name));
                templates_dir.join(folder_path).join(filename)
            };

            self.clipboard = Some(ClipboardItem {
                operation: ClipboardOperation::Copy,
                item_type: focused_node.node_type.clone(),
                name: focused_node.name.clone(),
                path: focused_node.path.clone(),
                full_file_path: full_path,
            });

            self.log(
                LogLevel::Info,
                format!(
                    "Copied {}: {}",
                    if focused_node.node_type == NodeType::Folder {
                        "folder"
                    } else {
                        "template"
                    },
                    focused_node.name
                ),
            );
        }
    }

    /// Paste clipboard item to currently focused folder
    pub async fn paste_clipboard_item(&mut self) -> Result<()> {
        if let Some(clipboard_item) = &self.clipboard.clone() {
            // Determine target folder
            let target_folder = if let Some(focused_node) = self.tree_state.get_focused_node() {
                match focused_node.node_type {
                    NodeType::Folder => focused_node.path.clone(),
                    NodeType::Template => {
                        // Get parent folder of template
                        if let Some(parent_pos) = focused_node.path.rfind('/') {
                            focused_node.path[..parent_pos].to_string()
                        } else {
                            "".to_string()
                        }
                    }
                }
            } else {
                "".to_string() // Root
            };

            match clipboard_item.operation {
                ClipboardOperation::Cut => {
                    self.move_item_to_folder(clipboard_item, &target_folder)
                        .await?;
                    self.clipboard = None; // Clear clipboard after cut operation
                }
                ClipboardOperation::Copy => {
                    self.copy_item_to_folder(clipboard_item, &target_folder)
                        .await?;
                    // Keep clipboard for multiple paste operations
                }
            }
        } else {
            self.log(LogLevel::Warn, "Nothing in clipboard to paste");
        }

        Ok(())
    }

    /// Move an item to target folder (for cut operation)
    async fn move_item_to_folder(
        &mut self,
        item: &ClipboardItem,
        target_folder: &str,
    ) -> Result<()> {
        let templates_dir = self.config.get_templates_directory();

        match item.item_type {
            NodeType::Folder => {
                // Calculate new folder path
                let new_path = if target_folder.is_empty() {
                    item.name.clone()
                } else {
                    format!("{}/{}", target_folder, item.name)
                };
                let new_full_path = templates_dir.join(&new_path);

                // Check if target already exists
                if new_full_path.exists() {
                    self.log(
                        LogLevel::Error,
                        format!("Folder '{}' already exists in target location", item.name),
                    );
                    return Ok(());
                }

                // Move the folder
                std::fs::rename(&item.full_file_path, &new_full_path)
                    .map_err(|e| anyhow::anyhow!("Failed to move folder: {}", e))?;

                self.log(
                    LogLevel::Success,
                    format!(
                        "Moved folder '{}' to '{}'",
                        item.name,
                        if target_folder.is_empty() {
                            "Root"
                        } else {
                            target_folder
                        }
                    ),
                );
            }
            NodeType::Template => {
                // Calculate new template path
                let filename = format!("{}.json", sanitize_filename(&item.name));
                let target_dir = templates_dir.join(target_folder);
                let new_file_path = target_dir.join(&filename);

                // Create target directory if it doesn't exist
                std::fs::create_dir_all(&target_dir)
                    .map_err(|e| anyhow::anyhow!("Failed to create target directory: {}", e))?;

                // Check if target file already exists
                if new_file_path.exists() {
                    self.log(
                        LogLevel::Error,
                        format!("Template '{}' already exists in target location", item.name),
                    );
                    return Ok(());
                }

                // Move the template file
                std::fs::rename(&item.full_file_path, &new_file_path)
                    .map_err(|e| anyhow::anyhow!("Failed to move template: {}", e))?;

                self.log(
                    LogLevel::Success,
                    format!(
                        "Moved template '{}' to '{}'",
                        item.name,
                        if target_folder.is_empty() {
                            "Root"
                        } else {
                            target_folder
                        }
                    ),
                );
            }
        }

        // Refresh tree to show changes
        self.refresh_tree_from_storage().await?;
        Ok(())
    }

    /// Copy an item to target folder (for copy operation)
    async fn copy_item_to_folder(
        &mut self,
        item: &ClipboardItem,
        target_folder: &str,
    ) -> Result<()> {
        let templates_dir = self.config.get_templates_directory();

        match item.item_type {
            NodeType::Folder => {
                // For folders, we need to recursively copy the entire directory tree
                let new_path = if target_folder.is_empty() {
                    format!("{}_copy", item.name)
                } else {
                    format!("{}/{}_copy", target_folder, item.name)
                };
                let new_full_path = templates_dir.join(&new_path);

                self.copy_directory_recursive(&item.full_file_path, &new_full_path)?;

                self.log(
                    LogLevel::Success,
                    format!(
                        "Copied folder '{}' to '{}'",
                        item.name,
                        if target_folder.is_empty() {
                            "Root"
                        } else {
                            target_folder
                        }
                    ),
                );
            }
            NodeType::Template => {
                // Find a unique name for the copy
                let base_name = format!("{}_copy", item.name);
                let mut copy_name = base_name.clone();
                let mut counter = 1;

                let target_dir = templates_dir.join(target_folder);
                std::fs::create_dir_all(&target_dir)
                    .map_err(|e| anyhow::anyhow!("Failed to create target directory: {}", e))?;

                // Find unique filename
                while target_dir
                    .join(format!("{}.json", sanitize_filename(&copy_name)))
                    .exists()
                {
                    copy_name = format!("{}_{}", base_name, counter);
                    counter += 1;
                }

                let filename = format!("{}.json", sanitize_filename(&copy_name));
                let new_file_path = target_dir.join(&filename);

                // Load and modify the template
                let json_content = std::fs::read_to_string(&item.full_file_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read template: {}", e))?;

                let mut stored_template: crate::services::StoredTemplate =
                    serde_json::from_str(&json_content)
                        .map_err(|e| anyhow::anyhow!("Failed to parse template: {}", e))?;

                // Update name and timestamps
                stored_template.template.name = copy_name.clone();
                stored_template.created_at = chrono::Utc::now();
                stored_template.modified_at = chrono::Utc::now();
                stored_template.last_used_at = None;

                // Save the copy
                let updated_json = serde_json::to_string_pretty(&stored_template)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize template: {}", e))?;

                std::fs::write(&new_file_path, updated_json)
                    .map_err(|e| anyhow::anyhow!("Failed to write template copy: {}", e))?;

                self.log(
                    LogLevel::Success,
                    format!(
                        "Copied template '{}' as '{}' to '{}'",
                        item.name,
                        copy_name,
                        if target_folder.is_empty() {
                            "Root"
                        } else {
                            target_folder
                        }
                    ),
                );
            }
        }

        // Refresh tree to show changes
        self.refresh_tree_from_storage().await?;
        Ok(())
    }

    /// Recursively copy a directory and all its contents
    fn copy_directory_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        if !src.exists() {
            return Err(anyhow::anyhow!("Source directory does not exist"));
        }

        std::fs::create_dir_all(dst)
            .map_err(|e| anyhow::anyhow!("Failed to create directory: {}", e))?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                self.copy_directory_recursive(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)
                    .map_err(|e| anyhow::anyhow!("Failed to copy file: {}", e))?;
            }
        }

        Ok(())
    }

    /// Clear the clipboard
    pub fn clear_clipboard(&mut self) {
        if self.clipboard.is_some() {
            self.clipboard = None;
            self.log(LogLevel::Info, "Clipboard cleared");
        }
    }

    /// Get clipboard status for UI display
    pub fn get_clipboard_status(&self) -> Option<String> {
        self.clipboard.as_ref().map(|item| {
            let operation = match item.operation {
                ClipboardOperation::Cut => "Cut",
                ClipboardOperation::Copy => "Copied",
            };
            let item_type = if item.item_type == NodeType::Folder {
                "folder"
            } else {
                "template"
            };
            format!("{} {}: {}", operation, item_type, item.name)
        })
    }
    /// Show deletion confirmation dialog
    pub fn show_delete_confirmation_dialog(
        &mut self,
        item_path: &str,
        item_name: &str,
        is_folder: bool,
    ) {
        self.show_delete_confirmation_dialog = true;
        self.delete_confirmation_item_name = item_name.to_string();
        self.delete_confirmation_item_path = item_path.to_string();
        self.delete_confirmation_is_folder = is_folder;

        // If it's a folder, scan its contents
        if is_folder {
            self.delete_confirmation_contents = self.scan_folder_contents(item_path);
        } else {
            self.delete_confirmation_contents = vec![];
        }

        let item_type = if is_folder { "folder" } else { "template" };
        self.log(
            LogLevel::Debug,
            format!(
                "Delete confirmation dialog opened for {}: {}",
                item_type, item_name
            ),
        );
    }

    /// Hide deletion confirmation dialog
    pub fn hide_delete_confirmation_dialog(&mut self) {
        self.show_delete_confirmation_dialog = false;
        self.delete_confirmation_item_name.clear();
        self.delete_confirmation_item_path.clear();
        self.delete_confirmation_is_folder = false;
        self.delete_confirmation_contents.clear();

        self.log(LogLevel::Debug, "Delete confirmation dialog closed");
    }

    /// Perform the confirmed deletion
    pub async fn confirm_deletion(&mut self) -> Result<()> {
        if self.delete_confirmation_is_folder {
            self.delete_folder_confirmed(&self.delete_confirmation_item_path.clone())
                .await
        } else {
            self.delete_template(&self.delete_confirmation_item_path.clone())
                .await
        }
    }

    /// Delete folder after confirmation
    async fn delete_folder_confirmed(&mut self, folder_path: &str) -> Result<()> {
        let templates_dir = self.config.get_templates_directory();
        let full_folder_path = templates_dir.join(folder_path);

        if !full_folder_path.exists() {
            self.log(LogLevel::Error, "Folder no longer exists");
            self.hide_delete_confirmation_dialog();
            return Ok(());
        }

        // Recursively delete the folder and all its contents
        match std::fs::remove_dir_all(&full_folder_path) {
            Ok(()) => {
                self.log(
                    LogLevel::Success,
                    format!(
                        "Deleted folder '{}' and all its contents",
                        self.delete_confirmation_item_name
                    ),
                );

                // Refresh tree and hide dialog
                self.refresh_tree_from_storage().await?;
                self.hide_delete_confirmation_dialog();
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to delete folder: {}", e);
                self.log(LogLevel::Error, error_msg);
                self.hide_delete_confirmation_dialog();
                Ok(())
            }
        }
    }

    /// Scan folder contents for confirmation dialog
    fn scan_folder_contents(&self, folder_path: &str) -> Vec<String> {
        let templates_dir = self.config.get_templates_directory();
        let full_folder_path = templates_dir.join(folder_path);

        let mut contents = Vec::new();

        if let Ok(entries) = self.scan_directory_recursive(&full_folder_path, folder_path) {
            contents = entries;
        }

        contents
    }

    /// Recursively scan directory for contents display
    fn scan_directory_recursive(&self, dir: &Path, relative_path: &str) -> Result<Vec<String>> {
        let mut items = Vec::new();

        if !dir.exists() {
            return Ok(items);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                // Add folder
                let folder_path = if relative_path.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", relative_path, name)
                };
                items.push(format!("📁 {}", folder_path));

                // Recursively scan subfolder
                if let Ok(sub_items) = self.scan_directory_recursive(&path, &folder_path) {
                    items.extend(sub_items);
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Add template (remove .json extension for display)
                let template_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or(&name);
                let template_path = if relative_path.is_empty() {
                    template_name.to_string()
                } else {
                    format!("{}/{}", relative_path, template_name)
                };
                items.push(format!("📄 {}", template_path));
            }
        }

        Ok(items)
    }

    /// Get count of items that will be deleted
    pub fn get_deletion_count(&self) -> (usize, usize) {
        let folders = self
            .delete_confirmation_contents
            .iter()
            .filter(|item| item.starts_with("📁"))
            .count();
        let templates = self
            .delete_confirmation_contents
            .iter()
            .filter(|item| item.starts_with("📄"))
            .count();

        (folders, templates)
    }

    // Add cursor movement methods
    pub fn move_field_cursor_left(&mut self) {
        if let Some(field) = self.automation_state.get_focused_field() {
            let cursor_moved_left = self.form_field_cursor_index.saturating_sub(1);
            self.form_field_cursor_index = cursor_moved_left.min(field.value.chars().count());
        }
    }

    pub fn move_field_cursor_right(&mut self) {
        if let Some(field) = self.automation_state.get_focused_field() {
            let cursor_moved_right = self.form_field_cursor_index.saturating_add(1);
            self.form_field_cursor_index = cursor_moved_right.min(field.value.chars().count());
        }
    }

    pub fn insert_char_at_cursor(&mut self, c: char) {
        if let Some(field) = self.automation_state.get_focused_field_mut() {
            // Extract the current value first to avoid borrow checker issues
            let current_value = field.value.clone();
            let byte_index = Self::get_byte_index_from_cursor_static(
                &current_value,
                self.form_field_cursor_index,
            );
            field.value.insert(byte_index, c);
            self.form_field_cursor_index += 1;
        }
    }

    pub fn delete_char_at_cursor(&mut self) {
        if self.form_field_cursor_index > 0 {
            if let Some(field) = self.automation_state.get_focused_field_mut() {
                let current_index = self.form_field_cursor_index;
                let from_left_to_current_index = current_index - 1;

                // Split string and rebuild without the character at cursor-1
                let before_char_to_delete = field.value.chars().take(from_left_to_current_index);
                let after_char_to_delete = field.value.chars().skip(current_index);

                field.value = before_char_to_delete.chain(after_char_to_delete).collect();
                self.move_field_cursor_left();
            }
        }
    }

    fn get_byte_index_from_cursor_static(text: &str, cursor_index: usize) -> usize {
        text.char_indices()
            .map(|(i, _)| i)
            .nth(cursor_index)
            .unwrap_or(text.len())
    }

    pub fn reset_field_cursor(&mut self) {
        self.form_field_cursor_index = 0;
    }

    pub fn set_cursor_to_end_of_field(&mut self) {
        if let Some(field) = self.automation_state.get_focused_field() {
            self.form_field_cursor_index = field.value.chars().count();
        }
    }

    pub fn enter_edit_mode(&mut self) {
        self.input_mode = InputMode::Edit;
        self.set_cursor_to_end_of_field(); // Start at end of existing text
        self.log(LogLevel::Debug, "Entered edit mode");
    }

    pub fn exit_edit_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.reset_field_cursor();
        self.log(LogLevel::Debug, "Exited edit mode");
    }

    /// Show the help dialog
    pub fn show_help_dialog(&mut self) {
        self.show_help_dialog = true;
        self.help_search_query.clear();
        self.help_selected_section = 0;
        self.log(LogLevel::Debug, "Help dialog opened");
    }

    /// Hide the help dialog
    pub fn hide_help_dialog(&mut self) {
        self.show_help_dialog = false;
        self.help_search_query.clear();
        self.help_selected_section = 0;
        self.log(LogLevel::Debug, "Help dialog closed");
    }

    /// Send HTTP request (for HTTP mode)
    pub async fn send_http_request(&mut self) -> Result<()> {
        self.log(LogLevel::Debug, "send_http_request() called");

        if self.http_state.is_sending {
            self.log(LogLevel::Warn, "HTTP request is already being sent");
            return Ok(());
        }

        // Check if request is valid
        if !self.http_state.is_valid() {
            let errors = self.http_state.get_validation_errors();
            for error in &errors {
                self.log(LogLevel::Error, error.clone());
            }
            return Ok(());
        }

        self.http_state.is_sending = true;
        self.log(LogLevel::Info, "🌐 Sending HTTP request...");

        // Clone the data we need for the background task
        let request = self.http_state.current_request.clone();
        let sender = self.message_sender.clone();

        self.log(LogLevel::Debug, "Spawning HTTP request task...");

        // Spawn the HTTP request task
        tokio::spawn(async move {
            match send_http_request_impl(request).await {
                Ok(response) => {
                    let _ = sender.send(AppMessage::Log(
                        LogLevel::Success,
                        format!("✅ HTTP {} {}", response.status_code, response.status_text),
                    ));

                    let _ = sender.send(AppMessage::Log(
                        LogLevel::Info,
                        format!("Response received in {} ms", response.duration_ms),
                    ));

                    // FIXED: Send the actual response to update the UI
                    let _ = sender.send(AppMessage::HttpResponseReceived(response));
                }
                Err(error) => {
                    let error_msg = format!("HTTP request failed: {}", error);

                    // FIXED: Use new message type to clear sending state
                    let _ = sender.send(AppMessage::HttpRequestFailed(error_msg));
                }
            }
        });

        self.log(LogLevel::Debug, "HTTP request task spawned");
        Ok(())
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

// HTTP request implementation using reqwest
async fn send_http_request_impl(request: HttpRequest) -> Result<HttpResponse> {
    use std::time::Instant;

    let start_time = Instant::now();

    // Create reqwest client
    let client = reqwest::Client::new();

    // Build the request
    let mut req_builder = match request.method {
        HttpMethod::GET => client.get(&request.url),
        HttpMethod::POST => client.post(&request.url),
        HttpMethod::PUT => client.put(&request.url),
        HttpMethod::PATCH => client.patch(&request.url),
        HttpMethod::DELETE => client.delete(&request.url),
        HttpMethod::HEAD => client.head(&request.url),
        HttpMethod::OPTIONS => client.request(reqwest::Method::OPTIONS, &request.url),
    };

    // Add headers
    for header in &request.headers {
        if header.enabled {
            req_builder = req_builder.header(&header.name, &header.value);
        }
    }

    // Add query parameters
    let mut query_params = Vec::new();
    for param in &request.query_params {
        if param.enabled {
            query_params.push((&param.name, &param.value));
        }
    }
    if !query_params.is_empty() {
        req_builder = req_builder.query(&query_params);
    }

    // Add body
    req_builder = match &request.body {
        HttpRequestBody::None => req_builder,
        HttpRequestBody::Text {
            content,
            content_type,
        } => req_builder
            .header("Content-Type", content_type)
            .body(content.clone()),
        HttpRequestBody::Json { content } => req_builder
            .header("Content-Type", "application/json")
            .body(content.clone()),
        HttpRequestBody::Raw { content } => req_builder.body(content.clone()),
        HttpRequestBody::Form { fields } => {
            let mut form_data = std::collections::HashMap::new();
            for field in fields {
                if field.enabled {
                    form_data.insert(&field.name, &field.value);
                }
            }
            req_builder.form(&form_data)
        }
    };

    // Add authentication
    req_builder = match &request.auth {
        HttpAuth::None => req_builder,
        HttpAuth::Basic { username, password } => req_builder.basic_auth(username, Some(password)),
        HttpAuth::Bearer { token } => req_builder.bearer_auth(token),
        HttpAuth::ApiKey {
            key,
            value,
            location,
        } => match location {
            crate::models::ApiKeyLocation::Header => req_builder.header(key, value),
            crate::models::ApiKeyLocation::QueryParam => req_builder.query(&[(key, value)]),
        },
    };

    // Send the request
    let response = req_builder.send().await?;
    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Extract response data
    let status_code = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    // Extract headers
    let headers: Vec<HttpHeader> = response
        .headers()
        .iter()
        .map(|(name, value)| {
            HttpHeader::new(name.as_str(), value.to_str().unwrap_or("<invalid utf8>"))
        })
        .collect();

    // Extract content type
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("text/plain")
        .to_string();

    // Extract body
    let body = response.text().await?;

    Ok(HttpResponse {
        status_code,
        status_text,
        headers,
        body,
        content_type,
        duration_ms,
    })
}
