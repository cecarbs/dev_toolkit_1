use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Directory where templates are stored
    pub templates_directory: PathBuf,

    /// Whether to show logs by default
    pub show_logs_on_startup: bool,

    /// Last used template directory (for user override)
    pub custom_templates_dir: Option<PathBuf>,
}

impl AppConfig {
    /// Create default configuration with cross-platform paths
    pub fn default() -> Self {
        Self {
            templates_directory: get_default_templates_dir(),
            show_logs_on_startup: true,
            custom_templates_dir: None,
        }
    }

    /// Load configuration from file, or create default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = get_config_file_path()?;

        if config_path.exists() {
            let config_content =
                std::fs::read_to_string(&config_path).context("Failed to read config file")?;

            let config: AppConfig =
                serde_json::from_str(&config_content).context("Failed to parse config file")?;

            Ok(config)
        } else {
            // Create default config and save it
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_file_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let config_content =
            serde_json::to_string_pretty(self).context("Failed to serialize config")?;

        std::fs::write(&config_path, config_content).context("Failed to write config file")?;

        Ok(())
    }

    /// Get the templates directory, preferring custom over default
    pub fn get_templates_directory(&self) -> &PathBuf {
        self.custom_templates_dir
            .as_ref()
            .unwrap_or(&self.templates_directory)
    }

    /// Set a custom templates directory
    pub fn set_custom_templates_directory(&mut self, path: PathBuf) -> Result<()> {
        // Validate that the directory exists or can be created
        if !path.exists() {
            std::fs::create_dir_all(&path)
                .context("Failed to create custom templates directory")?;
        }

        self.custom_templates_dir = Some(path);
        self.save()?;
        Ok(())
    }

    /// Reset to default templates directory
    pub fn reset_templates_directory(&mut self) -> Result<()> {
        self.custom_templates_dir = None;
        self.save()?;
        Ok(())
    }

    /// Get a display-friendly path for the UI
    pub fn get_templates_directory_display(&self) -> String {
        let path = self.get_templates_directory();

        // Try to show a shortened version for common directories
        if let Some(home_dir) = dirs::home_dir() {
            if let Ok(relative) = path.strip_prefix(&home_dir) {
                return format!("~/{}", relative.display());
            }
        }

        path.display().to_string()
    }
}

/// Get the default templates directory for the current platform
fn get_default_templates_dir() -> PathBuf {
    // Try platform-specific config directory first
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("automation-toolkit").join("templates")
    } else {
        // Fallback to current directory
        PathBuf::from(".").join("automation-templates")
    }
}

/// Get the path for the config file
fn get_config_file_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));

    Ok(config_dir.join("automation-toolkit").join("config.json"))
}

/// Initialize the templates directory and create demo structure if needed
pub fn ensure_templates_directory(config: &AppConfig) -> Result<()> {
    let templates_dir = config.get_templates_directory();

    // Create the main templates directory
    std::fs::create_dir_all(templates_dir).context("Failed to create templates directory")?;

    // Create demo folder structure if it doesn't exist
    create_demo_structure(templates_dir)?;

    Ok(())
}

/// Create the demo folder structure: Customer > Add > Email, etc.
fn create_demo_structure(templates_dir: &PathBuf) -> Result<()> {
    // Create Customer folder
    let customer_dir = templates_dir.join("Customer");
    std::fs::create_dir_all(&customer_dir)?;

    // Create Customer/Add folder
    let customer_add_dir = customer_dir.join("Add");
    std::fs::create_dir_all(&customer_add_dir)?;

    // Create Customer/Update folder
    let customer_update_dir = customer_dir.join("Update");
    std::fs::create_dir_all(&customer_update_dir)?;

    // Create Reports folder
    let reports_dir = templates_dir.join("Reports");
    std::fs::create_dir_all(&reports_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(!config.templates_directory.as_os_str().is_empty());
        assert_eq!(config.show_logs_on_startup, true);
        assert_eq!(config.custom_templates_dir, None);
    }

    #[test]
    fn test_templates_directory_display() {
        let config = AppConfig::default();
        let display = config.get_templates_directory_display();
        assert!(!display.is_empty());
    }
}
