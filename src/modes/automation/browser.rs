use crate::app::AppMessage;
use crate::models::{FieldType, FormField, LogLevel, WebsiteConfig};
use crate::modes::automation::Credentials;
use anyhow::{Context, Result};
use serde_json;
use std::process::Stdio;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

// Embed your Python script directly in the binary
const PYTHON_AUTOMATION_SCRIPT: &str = include_str!("../../../scripts/automation_script.py");

/// Browser automation engine that runs embedded Python scripts
pub struct BrowserEngine {
    message_sender: mpsc::UnboundedSender<AppMessage>,
}

impl BrowserEngine {
    pub fn new(message_sender: mpsc::UnboundedSender<AppMessage>) -> Self {
        Self { message_sender }
    }

    /// Run the embedded Python automation script
    pub async fn run_automation(
        &self,
        fields: Vec<FormField>,
        credentials: Credentials,
        website_config: WebsiteConfig,
    ) -> Result<()> {
        self.log_progress("Starting embedded Python automation...")
            .await;

        // Create a temporary file for the Python script
        let temp_script_path = self.create_temp_script().await?;

        // Run the Python script with your form data
        let result = self
            .execute_python_script(&temp_script_path, fields, credentials, website_config)
            .await;

        // Clean up the temporary file
        let _ = fs::remove_file(&temp_script_path).await;

        result
    }

    /// Create temporary Python script file from embedded content
    async fn create_temp_script(&self) -> Result<std::path::PathBuf> {
        use std::env;

        let temp_dir = env::temp_dir();
        let script_path = temp_dir.join("automation_script.py");

        fs::write(&script_path, PYTHON_AUTOMATION_SCRIPT)
            .await
            .context("Failed to write temporary Python script")?;

        self.log_progress("Created temporary Python script").await;
        Ok(script_path)
    }

    /// Execute the Python script and capture output in real-time
    async fn execute_python_script(
        &self,
        script_path: &std::path::Path,
        fields: Vec<FormField>,
        credentials: Credentials,
        website_config: WebsiteConfig,
    ) -> Result<()> {
        self.log_progress("Preparing Python automation data...")
            .await;

        // Prepare the data to send to Python
        let automation_data = AutomationData {
            fields,
            credentials,
            website_config,
        };

        let data_json = serde_json::to_string(&automation_data)
            .context("Failed to serialize automation data")?;

        self.log_progress("Launching Python process...").await;

        // Spawn Python process
        let mut child = Command::new("python3")
            .arg(script_path)
            .arg("--json-input")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn Python process")?;

        // Send data to Python via stdin
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(data_json.as_bytes())
                .await
                .context("Failed to send data to Python")?;
            stdin.shutdown().await.context("Failed to close stdin")?;
        }

        // Capture stdout and stderr in parallel
        let stdout_handle = if let Some(stdout) = child.stdout.take() {
            let sender = self.message_sender.clone();
            Some(tokio::spawn(async move {
                Self::process_python_output(stdout, sender).await;
            }))
        } else {
            None
        };

        let stderr_handle = if let Some(stderr) = child.stderr.take() {
            let sender = self.message_sender.clone();
            Some(tokio::spawn(async move {
                Self::process_python_errors(stderr, sender).await;
            }))
        } else {
            None
        };

        // Wait for Python process to complete
        let status = child
            .wait()
            .await
            .context("Failed to wait for Python process")?;

        // Wait for output processing to complete
        if let Some(handle) = stdout_handle {
            let _ = handle.await;
        }
        if let Some(handle) = stderr_handle {
            let _ = handle.await;
        }

        // Handle exit status
        if status.success() {
            self.send_completion().await;
        } else {
            let error_msg = format!("Python automation exited with code: {:?}", status.code());
            self.send_failure(error_msg).await;
        }

        Ok(())
    }

    /// Process Python stdout and parse special message formats
    async fn process_python_output(
        stdout: tokio::process::ChildStdout,
        sender: mpsc::UnboundedSender<AppMessage>,
    ) {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            // Parse different message formats from your Python script
            if let Some(message) = Self::parse_python_message(&line) {
                let _ = sender.send(message);
            } else if line.starts_with("PROGRESS:") {
                let msg = line.strip_prefix("PROGRESS:").unwrap_or(&line).trim();
                let _ = sender.send(AppMessage::AutomationProgress(msg.to_string()));
            } else if line.starts_with("ERROR:") {
                let msg = line.strip_prefix("ERROR:").unwrap_or(&line).trim();
                let _ = sender.send(AppMessage::Log(LogLevel::Error, msg.to_string()));
            } else if line.starts_with("SUCCESS:") {
                let msg = line.strip_prefix("SUCCESS:").unwrap_or(&line).trim();
                let _ = sender.send(AppMessage::Log(LogLevel::Success, msg.to_string()));
            } else if line.starts_with("INFO:") {
                let msg = line.strip_prefix("INFO:").unwrap_or(&line).trim();
                let _ = sender.send(AppMessage::Log(LogLevel::Info, msg.to_string()));
            } else if line.starts_with("DEBUG:") {
                let msg = line.strip_prefix("DEBUG:").unwrap_or(&line).trim();
                let _ = sender.send(AppMessage::Log(LogLevel::Debug, msg.to_string()));
            } else if line.starts_with("WARN:") {
                let msg = line.strip_prefix("WARN:").unwrap_or(&line).trim();
                let _ = sender.send(AppMessage::Log(LogLevel::Warn, msg.to_string()));
            } else {
                // Regular Python print() output
                let _ = sender.send(AppMessage::Log(LogLevel::Info, format!("🐍 {}", line)));
            }
        }
    }

    /// Process Python stderr
    async fn process_python_errors(
        stderr: tokio::process::ChildStderr,
        sender: mpsc::UnboundedSender<AppMessage>,
    ) {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let _ = sender.send(AppMessage::Log(
                LogLevel::Error,
                format!("🐍 Error: {}", line),
            ));
        }
    }

    /// Parse structured JSON messages from Python (optional advanced format)
    fn parse_python_message(line: &str) -> Option<AppMessage> {
        // Try to parse as JSON message first
        if let Ok(msg) = serde_json::from_str::<PythonMessage>(line) {
            match msg.msg_type.as_str() {
                "progress" => Some(AppMessage::AutomationProgress(msg.content)),
                "complete" => Some(AppMessage::AutomationComplete),
                "error" => Some(AppMessage::Log(LogLevel::Error, msg.content)),
                "success" => Some(AppMessage::Log(LogLevel::Success, msg.content)),
                "info" => Some(AppMessage::Log(LogLevel::Info, msg.content)),
                "debug" => Some(AppMessage::Log(LogLevel::Debug, msg.content)),
                "warn" => Some(AppMessage::Log(LogLevel::Warn, msg.content)),
                _ => Some(AppMessage::Log(LogLevel::Info, format!("🐍 {}", line))),
            }
        } else {
            None
        }
    }

    /// Send a progress update to the UI
    async fn log_progress(&self, message: impl Into<String>) {
        let _ = self
            .message_sender
            .send(AppMessage::AutomationProgress(message.into()));
    }

    /// Send completion signal to the UI
    pub async fn send_completion(&self) {
        let _ = self.message_sender.send(AppMessage::AutomationComplete);
    }

    /// Send failure signal to the UI
    pub async fn send_failure(&self, error: impl Into<String>) {
        let _ = self
            .message_sender
            .send(AppMessage::AutomationFailed(error.into()));
    }

    /// Test the actual automation script with dummy data
    pub async fn test_real_automation_script(&self) -> Result<()> {
        self.log_progress("Testing real automation script with dummy data...")
            .await;

        // Create test data that matches your form structure
        let test_fields = vec![
            FormField::new("Project Name", "#project_name", FieldType::Text)
                .with_value("TEST: Integration Test"),
            FormField::new("Department", "#department", FieldType::Select)
                .with_value("Engineering"),
            FormField::new("Priority Level", "#priority", FieldType::Select).with_value("Medium"),
            FormField::new("Description", "#description", FieldType::Textarea).with_value(
                "This is a test of the Python integration. All logging should appear in the TUI.",
            ),
            FormField::new("Contact Email", "#contact_email", FieldType::Email)
                .with_value("test@example.com"),
        ];

        let test_credentials = Credentials {
            username: "test_user".to_string(),
            password: "test_password".to_string(),
        };

        let test_website_config = WebsiteConfig::default();

        // Run the actual automation script with test data
        self.run_automation(test_fields, test_credentials, test_website_config)
            .await
    }
}

/// Data structure to send to Python script
#[derive(serde::Serialize)]
struct AutomationData {
    fields: Vec<FormField>,
    credentials: Credentials,
    website_config: WebsiteConfig,
}

/// Structure for JSON communication with Python script (optional)
#[derive(serde::Deserialize)]
struct PythonMessage {
    msg_type: String, // "progress", "error", "success", "complete", etc.
    content: String,
    timestamp: Option<String>,
}

// Implement Serialize for Credentials to send to Python
impl serde::Serialize for Credentials {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Credentials", 2)?;
        state.serialize_field("username", &self.username)?;
        state.serialize_field("password", &self.password)?;
        state.end()
    }
}
