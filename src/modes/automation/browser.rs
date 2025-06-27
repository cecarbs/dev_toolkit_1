use crate::app::AppMessage;
use crate::models::{FormField, LogLevel, WebsiteConfig};
use crate::modes::automation::Credentials;
use anyhow::{Context, Result};
use include_dir::{Dir, include_dir};
use serde_json;
use std::process::Stdio;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

// Embed the entire Python project directory in the binary
static PYTHON_PROJECT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/scripts");

/// Browser automation engine that runs embedded Python projects
pub struct BrowserEngine {
    message_sender: mpsc::UnboundedSender<AppMessage>,
}

impl BrowserEngine {
    pub fn new(message_sender: mpsc::UnboundedSender<AppMessage>) -> Self {
        Self { message_sender }
    }

    /// Run the embedded Python automation project
    pub async fn run_automation(
        &self,
        fields: Vec<FormField>,
        credentials: Credentials,
        website_config: WebsiteConfig,
    ) -> Result<()> {
        self.log_progress("🚀 Starting embedded Python automation project...")
            .await;

        // Extract the entire Python project to a temporary directory
        let temp_project_dir = self.extract_python_project().await?;

        // Run the main automation script
        let result = self
            .execute_python_project(&temp_project_dir, fields, credentials, website_config)
            .await;

        // Clean up the temporary directory
        let _ = fs::remove_dir_all(&temp_project_dir).await;

        result
    }

    /// Extract the embedded Python project to a temporary directory
    async fn extract_python_project(&self) -> Result<std::path::PathBuf> {
        use std::env;

        let temp_dir = env::temp_dir();
        let project_dir = temp_dir.join(format!("automation_project_{}", std::process::id()));

        self.log_progress("📦 Extracting embedded Python project...")
            .await;

        // Recursively extract all files and directories
        Self::extract_dir_recursive_impl(&PYTHON_PROJECT, &project_dir).await?;

        self.log_progress(format!(
            "✅ Python project extracted to: {}",
            project_dir.display()
        ))
        .await;
        Ok(project_dir)
    }

    /// Recursively extract a directory structure - static method to avoid self reference issues
    async fn extract_dir_recursive_impl(
        dir: &Dir<'_>,
        target_path: &std::path::Path,
    ) -> Result<()> {
        // Create the target directory
        fs::create_dir_all(target_path)
            .await
            .context("Failed to create directory")?;

        // Extract all files in this directory
        for file in dir.files() {
            let file_path = target_path.join(file.path());
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(&file_path, file.contents())
                .await
                .context(format!("Failed to write file: {}", file_path.display()))?;

            // Make Python files executable on Unix systems
            #[cfg(unix)]
            {
                if file_path.extension().and_then(|s| s.to_str()) == Some("py") {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&file_path).await?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&file_path, perms).await?;
                }
            }
        }

        // Recursively extract subdirectories using Box::pin for async recursion
        for subdir in dir.dirs() {
            let subdir_path = target_path.join(subdir.path());
            Box::pin(Self::extract_dir_recursive_impl(subdir, &subdir_path)).await?;
        }

        Ok(())
    }

    /// Execute the main Python script from the extracted project
    async fn execute_python_project(
        &self,
        project_dir: &std::path::Path,
        fields: Vec<FormField>,
        credentials: Credentials,
        website_config: WebsiteConfig,
    ) -> Result<()> {
        self.log_progress("📋 Preparing automation data...").await;

        // Prepare the data to send to Python
        let automation_data = AutomationData {
            fields,
            credentials,
            website_config,
        };

        let data_json = serde_json::to_string(&automation_data)
            .context("Failed to serialize automation data")?;

        self.log_progress("🐍 Launching Python automation...").await;

        // Find the main script (automation_script.py or main.py)
        let main_script = project_dir.join("automation_script.py");
        let alt_script = project_dir.join("main.py");

        let script_path = if main_script.exists() {
            main_script
        } else if alt_script.exists() {
            alt_script
        } else {
            return Err(anyhow::anyhow!(
                "No main script found (automation_script.py or main.py)"
            ));
        };

        self.log_progress(format!(
            "📄 Running: {}",
            script_path.file_name().unwrap().to_string_lossy()
        ))
        .await;

        // Spawn Python process with the project directory as working directory
        let mut child = Command::new("python3")
            .arg(&script_path)
            .arg("--json-input")
            .current_dir(project_dir) // Important: Set working directory for imports
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn Python process. Make sure python3 is installed.")?;

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

    /// Test Python integration with embedded project
    pub async fn test_python_integration(&self) -> Result<()> {
        self.log_progress("🧪 Starting Python integration test...")
            .await;

        // Create minimal test data
        let test_fields = vec![
            FormField::new("Test Field", "#test", crate::models::FieldType::Text)
                .with_value("Integration Test Value"),
        ];

        let test_credentials = Credentials {
            username: "test_user".to_string(),
            password: "test_password".to_string(),
        };

        let test_website_config = WebsiteConfig::default();

        // Run automation with test data
        self.run_automation(test_fields, test_credentials, test_website_config)
            .await
    }

    /// Process Python stdout and parse special message formats
    async fn process_python_output(
        stdout: tokio::process::ChildStdout,
        sender: mpsc::UnboundedSender<AppMessage>,
    ) {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            // Parse different message formats from Python script
            if line.starts_with("PROGRESS:") {
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
}

/// Data structure to send to Python script
#[derive(serde::Serialize)]
struct AutomationData {
    fields: Vec<FormField>,
    credentials: Credentials,
    website_config: WebsiteConfig,
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
