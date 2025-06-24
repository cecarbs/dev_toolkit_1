use crate::models::{AppConfig, AutomationTemplate, FormField};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A template file with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTemplate {
    /// The template data
    pub template: AutomationTemplate,

    /// When the template was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When the template was last modified
    pub modified_at: chrono::DateTime<chrono::Utc>,

    /// When the template was last used
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Tags for organization and search
    pub tags: Vec<String>,

    /// File format version for future compatibility
    pub version: String,
}

impl StoredTemplate {
    /// Create a new stored template from an AutomationTemplate
    pub fn new(template: AutomationTemplate) -> Self {
        let now = chrono::Utc::now();
        Self {
            template,
            created_at: now,
            modified_at: now,
            last_used_at: None,
            tags: Vec::new(),
            version: "1.0".to_string(),
        }
    }

    /// Mark template as used (updates last_used_at)
    pub fn mark_as_used(&mut self) {
        self.last_used_at = Some(chrono::Utc::now());
        self.modified_at = chrono::Utc::now();
    }

    /// Update the template data
    pub fn update_template(&mut self, new_template: AutomationTemplate) {
        self.template = new_template;
        self.modified_at = chrono::Utc::now();
    }
}

/// Template storage service for managing templates on disk
pub struct TemplateStorage {
    config: AppConfig,
}

impl TemplateStorage {
    /// Create a new template storage service
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Initialize the storage (create directories, demo templates, etc.)
    pub fn initialize(&self) -> Result<()> {
        crate::models::config::ensure_templates_directory(&self.config)?;

        // Create some demo templates if none exist
        self.create_demo_templates_if_needed()?;

        Ok(())
    }

    /// Save a template to disk
    pub fn save_template(
        &self,
        folder_path: &str,
        template_name: &str,
        template: AutomationTemplate,
    ) -> Result<PathBuf> {
        let stored_template = StoredTemplate::new(template);

        // Build the full path
        let templates_dir = self.config.get_templates_directory();
        let folder_dir = templates_dir.join(folder_path);

        // Create the folder if it doesn't exist
        std::fs::create_dir_all(&folder_dir).context("Failed to create template folder")?;

        // Create the filename (sanitize the name)
        let filename = sanitize_filename(template_name) + ".json";
        let file_path = folder_dir.join(filename);

        // Serialize and save
        let json_content = serde_json::to_string_pretty(&stored_template)
            .context("Failed to serialize template")?;

        std::fs::write(&file_path, json_content).context("Failed to write template file")?;

        Ok(file_path)
    }

    /// Load a specific template from disk
    pub fn load_template(&self, folder_path: &str, template_name: &str) -> Result<StoredTemplate> {
        let templates_dir = self.config.get_templates_directory();
        let filename = sanitize_filename(template_name) + ".json";
        let file_path = templates_dir.join(folder_path).join(filename);

        let json_content =
            std::fs::read_to_string(&file_path).context("Failed to read template file")?;

        let mut stored_template: StoredTemplate =
            serde_json::from_str(&json_content).context("Failed to parse template file")?;

        // Mark as used
        stored_template.mark_as_used();

        // Save the updated usage info
        let json_content = serde_json::to_string_pretty(&stored_template)
            .context("Failed to serialize updated template")?;
        std::fs::write(&file_path, json_content).context("Failed to update template file")?;

        Ok(stored_template)
    }

    /// Delete a template from disk
    pub fn delete_template(&self, folder_path: &str, template_name: &str) -> Result<()> {
        let templates_dir = self.config.get_templates_directory();
        let filename = sanitize_filename(template_name) + ".json";
        let file_path = templates_dir.join(folder_path).join(filename);

        std::fs::remove_file(&file_path).context("Failed to delete template file")?;

        Ok(())
    }

    /// Get all templates in a folder
    pub fn list_templates_in_folder(&self, folder_path: &str) -> Result<Vec<String>> {
        let templates_dir = self.config.get_templates_directory();
        let folder_dir = templates_dir.join(folder_path);

        if !folder_dir.exists() {
            return Ok(Vec::new());
        }

        let mut templates = Vec::new();

        for entry in std::fs::read_dir(&folder_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    templates.push(stem.to_string());
                }
            }
        }

        templates.sort();
        Ok(templates)
    }

    /// Get all folders in the templates directory
    pub fn list_all_folders(&self) -> Result<Vec<String>> {
        let templates_dir = self.config.get_templates_directory();
        let mut folders = Vec::new();

        self.scan_folders_recursive(&templates_dir, "", &mut folders)?;

        folders.sort();
        Ok(folders)
    }

    /// Recursively scan for folders
    fn scan_folders_recursive(
        &self,
        dir: &Path,
        current_path: &str,
        folders: &mut Vec<String>,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let folder_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown");

                let full_path = if current_path.is_empty() {
                    folder_name.to_string()
                } else {
                    format!("{}/{}", current_path, folder_name)
                };

                folders.push(full_path.clone());

                // Recursively scan subfolders
                self.scan_folders_recursive(&path, &full_path, folders)?;
            }
        }

        Ok(())
    }

    /// Create demo templates if the templates directory is empty
    fn create_demo_templates_if_needed(&self) -> Result<()> {
        let templates_dir = self.config.get_templates_directory();

        // Check if any templates already exist
        let existing_folders = self.list_all_folders().unwrap_or_default();
        if !existing_folders.is_empty() {
            return Ok(()); // Templates already exist, don't create demos
        }

        // Create demo templates
        self.create_demo_template(
            "Customer/Add",
            "Email",
            "Add customer email template",
            vec![
                ("Project Name", "Customer Email Addition"),
                ("Department", "Customer Service"),
                ("Priority Level", "Medium"),
                ("Description", "Add email address to customer profile"),
                ("Contact Email", "customer@example.com"),
            ],
        )?;

        self.create_demo_template(
            "Customer/Add",
            "Phone",
            "Add customer phone template",
            vec![
                ("Project Name", "Customer Phone Addition"),
                ("Department", "Customer Service"),
                ("Priority Level", "Low"),
                ("Description", "Add phone number to customer profile"),
                ("Contact Email", ""),
            ],
        )?;

        self.create_demo_template(
            "Customer/Update",
            "Profile",
            "Update customer profile template",
            vec![
                ("Project Name", "Customer Profile Update"),
                ("Department", "Customer Service"),
                ("Priority Level", "Medium"),
                ("Description", "Update customer profile information"),
                ("Contact Email", "support@company.com"),
            ],
        )?;

        self.create_demo_template(
            "Reports",
            "Daily Summary",
            "Daily report template",
            vec![
                ("Project Name", "Daily Report"),
                ("Department", "Operations"),
                ("Priority Level", "Low"),
                ("Description", "Daily operations summary report"),
                ("Contact Email", "reports@company.com"),
            ],
        )?;

        Ok(())
    }

    /// Helper to create a demo template
    fn create_demo_template(
        &self,
        folder_path: &str,
        name: &str,
        description: &str,
        field_values: Vec<(&str, &str)>,
    ) -> Result<()> {
        let mut template = AutomationTemplate::new(name, description);

        for (field_name, value) in field_values {
            template = template.with_field(field_name, value);
        }

        self.save_template(folder_path, name, template)?;
        Ok(())
    }

    /// Get the templates directory for display
    pub fn get_templates_directory_display(&self) -> String {
        self.config.get_templates_directory_display()
    }
}

/// Sanitize a filename by removing/replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Normal Name"), "Normal Name");
        assert_eq!(
            sanitize_filename("Bad/Name:With*Chars"),
            "Bad_Name_With_Chars"
        );
        assert_eq!(sanitize_filename("Test<>File"), "Test__File");
    }

    #[test]
    fn test_stored_template_creation() {
        let template = AutomationTemplate::new("Test", "Description");
        let stored = StoredTemplate::new(template);

        assert_eq!(stored.template.name, "Test");
        assert_eq!(stored.version, "1.0");
        assert!(stored.last_used_at.is_none());
    }
}
